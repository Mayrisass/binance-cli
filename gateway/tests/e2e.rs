use std::collections::HashMap;

use binance_gateway::client::GatewayClient;
use binance_gateway::config::{Config, RateLimitConfig};
use binance_gateway::{gateway, mock};
use reqwest::{Method, StatusCode};
use serde_json::json;

async fn spawn(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

async fn start(burst: u32, services: &[&str]) -> String {
    let mock_url = spawn(mock::router()).await;
    let config = Config {
        keys: HashMap::from([("test-key".into(), "test-secret".into())]),
        upstreams: services
            .iter()
            .map(|s| (s.to_string(), format!("{mock_url}/{s}")))
            .collect(),
        rate_limit: RateLimitConfig {
            requests_per_second: 0.001,
            burst,
        },
        recv_window_ms: 5_000,
    };
    config.validate().unwrap();
    spawn(gateway::router(config)).await
}

const ALL: [&str; 5] = ["market", "trading", "transaction", "wallet", "defi"];

#[tokio::test]
async fn routes_each_service_to_its_upstream() {
    let url = start(100, &ALL).await;
    let c = GatewayClient::new(&url, "test-key", "test-secret");
    let addr = "0xabc";

    let r = c.market_price("ETH").await.unwrap();
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["service"], "market");
    assert_eq!(r.body["path"], "/price");
    assert_eq!(r.body["data"]["symbol"], "ETH");
    assert_eq!(r.body["clientId"], "test-key");
    assert_eq!(r.body["requestId"].as_str(), r.request_id.as_deref());

    let r = c.trading_quote("56", "USDT", "BNB", "100").await.unwrap();
    assert_eq!(r.body["service"], "trading");
    assert_eq!(r.body["query"]["fromToken"], "USDT");

    let tx = json!({ "to": addr, "value": "1" });
    let r = c.transaction_simulate(&tx).await.unwrap();
    assert_eq!(r.body["service"], "transaction");
    assert_eq!(r.body["method"], "POST");
    assert_eq!(r.body["body"], tx);

    let r = c.wallet_balance(addr).await.unwrap();
    assert_eq!(r.body["service"], "wallet");
    assert_eq!(r.body["data"]["address"], addr);

    let r = c.defi_positions(addr).await.unwrap();
    assert_eq!(r.body["service"], "defi");
}

#[tokio::test]
async fn rejects_bad_credentials() {
    let url = start(100, &ALL).await;

    let r = GatewayClient::new(&url, "test-key", "wrong")
        .market_price("BNB")
        .await
        .unwrap();
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(r.body["code"], "UNAUTHORIZED");

    let r = GatewayClient::new(&url, "other", "test-secret")
        .market_price("BNB")
        .await
        .unwrap();
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);

    let r = reqwest::get(format!("{url}/api/v1/market/price"))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    let r = reqwest::get(format!("{url}/health")).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn rate_limits_per_key() {
    let url = start(2, &ALL).await;
    let c = GatewayClient::new(&url, "test-key", "test-secret");
    assert_eq!(c.market_price("BNB").await.unwrap().status, StatusCode::OK);
    assert_eq!(c.market_price("BNB").await.unwrap().status, StatusCode::OK);
    let r = c.market_price("BNB").await.unwrap();
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(r.body["code"], "RATE_LIMITED");
}

#[tokio::test]
async fn reports_unknown_and_unconfigured_services() {
    let url = start(100, &["market"]).await;
    let c = GatewayClient::new(&url, "test-key", "test-secret");

    let r = c.call(Method::GET, "nft", "list", None).await.unwrap();
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert_eq!(r.body["code"], "UNKNOWN_SERVICE");

    let r = c.wallet_balance("0x1").await.unwrap();
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(r.body["code"], "SERVICE_NOT_CONFIGURED");
}

#[tokio::test]
async fn returns_bad_gateway_when_upstream_is_down() {
    let config = Config {
        keys: HashMap::from([("k".into(), "s".into())]),
        // Port 9 (discard) on localhost is not listening.
        upstreams: HashMap::from([("market".into(), "http://127.0.0.1:9".into())]),
        rate_limit: RateLimitConfig::default(),
        recv_window_ms: 5_000,
    };
    let url = spawn(gateway::router(config)).await;
    let r = GatewayClient::new(&url, "k", "s")
        .market_price("BNB")
        .await
        .unwrap();
    assert_eq!(r.status, StatusCode::BAD_GATEWAY);
    assert_eq!(r.body["code"], "UPSTREAM_ERROR");
}
