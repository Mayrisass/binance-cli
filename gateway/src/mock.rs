//! Stand-in backends for local development. Mount every service on one server:
//! `http://<listen>/<service>/...`. Responses are canned sample data, marked `"mock": true`.

use std::collections::HashMap;

use axum::Router;
use axum::body::Bytes;
use axum::extract::Query;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use crate::gateway::{HEADER_CLIENT_ID, HEADER_REQUEST_ID};

pub fn router() -> Router {
    Router::new().fallback(handle)
}

fn sample(service: &str, q: &HashMap<String, String>) -> Option<Value> {
    let get = |k: &str, default: &str| q.get(k).cloned().unwrap_or_else(|| default.to_string());
    Some(match service {
        "market" => json!({
            "symbol": get("symbol", "BNB"),
            "price": "612.45",
            "change24h": "1.82",
            "candles": [
                { "openTime": 1761580800000u64, "open": "608.10", "high": "615.00", "low": "604.20", "close": "612.45", "volume": "18234.5" }
            ]
        }),
        "trading" => json!({
            "chainId": get("chainId", "56"),
            "fromToken": get("fromToken", "USDT"),
            "toToken": get("toToken", "BNB"),
            "fromAmount": get("amount", "100"),
            "toAmount": "0.16327",
            "route": ["PancakeSwap V3", "Uniswap V3"],
            "slippage": "auto"
        }),
        "transaction" => {
            json!({ "simulated": true, "success": true, "gasUsed": "21000", "txHash": null })
        }
        "wallet" => json!({
            "address": get("address", "0x0000000000000000000000000000000000000000"),
            "balances": [
                { "token": "BNB", "chainId": "56", "amount": "1.25" },
                { "token": "USDT", "chainId": "56", "amount": "350.00" }
            ]
        }),
        "defi" => json!({
            "positions": [
                { "protocol": "Venus", "type": "lending", "asset": "USDT", "amount": "200.00", "apy": "4.1" }
            ]
        }),
        _ => return None,
    })
}

async fn handle(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Bytes,
) -> Response {
    let path = uri.path().trim_start_matches('/');
    let (service, rest) = path.split_once('/').unwrap_or((path, ""));
    let Some(data) = sample(service, &query) else {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({ "message": "no such mock service" })),
        )
            .into_response();
    };
    let echo_body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).into()))
    };
    let header_str = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    };
    axum::Json(json!({
        "mock": true,
        "service": service,
        "method": method.as_str(),
        "path": format!("/{rest}"),
        "query": query,
        "body": echo_body,
        "requestId": header_str(HEADER_REQUEST_ID),
        "clientId": header_str(HEADER_CLIENT_ID),
        "data": data,
    }))
    .into_response()
}
