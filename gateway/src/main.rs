use std::path::PathBuf;

use anyhow::{Context, Result};
use binance_gateway::client::{ApiResponse, GatewayClient};
use binance_gateway::{config::Config, gateway, mock};
use clap::{Args, Parser, Subcommand};
use reqwest::Method;
use serde_json::{Value, json};

#[derive(Parser)]
#[command(
    version,
    about = "API gateway for the Market, Trading, Transaction, Wallet and DeFi APIs"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the gateway
    Serve {
        #[arg(long, default_value = "gateway.json")]
        config: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8080")]
        listen: String,
    },
    /// Run mock backends for all five services (http://<listen>/<service>/...)
    Mock {
        #[arg(long, default_value = "127.0.0.1:9000")]
        listen: String,
    },
    /// Send one signed request through the gateway
    Call {
        #[command(flatten)]
        conn: Connection,
        /// market | trading | transaction | wallet | defi
        service: String,
        /// Path under the service, with optional query string, e.g. "price?symbol=BNB"
        path: String,
        #[arg(short = 'X', long, default_value = "GET")]
        method: Method,
        /// JSON request body
        #[arg(short, long)]
        data: Option<String>,
    },
    /// Call one endpoint on each of the five services
    Tour {
        #[command(flatten)]
        conn: Connection,
        #[arg(long, default_value = "0x0000000000000000000000000000000000000000")]
        address: String,
    },
}

#[derive(Args)]
struct Connection {
    #[arg(long, env = "GATEWAY_URL", default_value = "http://127.0.0.1:8080")]
    gateway: String,
    #[arg(long, env = "GATEWAY_API_KEY")]
    api_key: String,
    #[arg(long, env = "GATEWAY_API_SECRET", hide_env_values = true)]
    api_secret: String,
}

impl Connection {
    fn client(&self) -> GatewayClient {
        GatewayClient::new(&self.gateway, &self.api_key, &self.api_secret)
    }
}

fn print(label: &str, resp: &ApiResponse) {
    println!(
        "{label}: {} (request {})\n{}",
        resp.status,
        resp.request_id.as_deref().unwrap_or("-"),
        serde_json::to_string_pretty(&resp.body).unwrap_or_default()
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Serve { config, listen } => {
            let config = Config::load(&config)?;
            let listener = tokio::net::TcpListener::bind(&listen).await?;
            eprintln!("gateway listening on http://{}", listener.local_addr()?);
            axum::serve(listener, gateway::router(config)).await?;
        }
        Command::Mock { listen } => {
            let listener = tokio::net::TcpListener::bind(&listen).await?;
            eprintln!(
                "mock backends listening on http://{}/<service>",
                listener.local_addr()?
            );
            axum::serve(listener, mock::router()).await?;
        }
        Command::Call {
            conn,
            service,
            path,
            method,
            data,
        } => {
            let body: Option<Value> = data
                .map(|d| serde_json::from_str(&d).context("--data must be valid JSON"))
                .transpose()?;
            let resp = conn
                .client()
                .call(method, &service, &path, body.as_ref())
                .await?;
            print(&service, &resp);
            if !resp.status.is_success() {
                std::process::exit(1);
            }
        }
        Command::Tour { conn, address } => {
            let c = conn.client();
            print("market", &c.market_price("BNB").await?);
            print(
                "trading",
                &c.trading_quote("56", "USDT", "BNB", "100").await?,
            );
            let tx = json!({ "chainId": "56", "from": address, "to": address, "value": "0" });
            print("transaction", &c.transaction_simulate(&tx).await?);
            print("wallet", &c.wallet_balance(&address).await?);
            print("defi", &c.defi_positions(&address).await?);
        }
    }
    Ok(())
}
