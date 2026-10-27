//! The gateway: authenticates, rate-limits, then forwards `/api/v1/<service>/<path>`
//! to the configured upstream for that service.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use serde_json::json;

use crate::auth;
use crate::config::{Config, SERVICES};
use crate::ratelimit::RateLimiter;

pub const HEADER_REQUEST_ID: &str = "x-request-id";
pub const HEADER_CLIENT_ID: &str = "x-client-id";
const API_PREFIX: &str = "/api/v1/";
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(30);

pub struct AppState {
    config: Config,
    limiter: RateLimiter,
    http: reqwest::Client,
}

pub fn router(config: Config) -> Router {
    let state = Arc::new(AppState {
        limiter: RateLimiter::new(
            config.rate_limit.requests_per_second,
            config.rate_limit.burst,
        ),
        config,
        http: reqwest::Client::builder()
            .timeout(UPSTREAM_TIMEOUT)
            .build()
            .expect("building HTTP client"),
    });
    Router::new()
        .route(
            "/health",
            get(|| async { axum::Json(json!({ "status": "ok" })) }),
        )
        .route("/api/v1/{*rest}", any(handle))
        .with_state(state)
}

fn error(status: StatusCode, code: &str, message: impl Into<String>, request_id: &str) -> Response {
    let mut resp = (
        status,
        axum::Json(json!({ "code": code, "message": message.into(), "requestId": request_id })),
    )
        .into_response();
    resp.headers_mut().insert(
        HEADER_REQUEST_ID,
        HeaderValue::from_str(request_id).unwrap(),
    );
    resp
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

async fn handle(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let path_and_query = uri
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or(uri.path());

    // 1. Auth / signature verification
    let api_key = match auth::authenticate(
        &state.config.keys,
        state.config.recv_window_ms,
        now_ms(),
        &headers,
        method.as_str(),
        path_and_query,
        &body,
    ) {
        Ok(key) => key,
        Err(e) => {
            return error(
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                e.message(),
                &request_id,
            );
        }
    };

    // 2. Rate limiting
    if let Err(wait) = state.limiter.check(&api_key, Instant::now()) {
        let mut resp = error(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "too many requests",
            &request_id,
        );
        let secs = wait.as_secs_f64().ceil().max(1.0) as u64;
        resp.headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from(secs));
        return resp;
    }

    // 3. Routing
    let rest = uri.path().strip_prefix(API_PREFIX).unwrap_or_default();
    let (service, service_path) = rest.split_once('/').unwrap_or((rest, ""));
    if !SERVICES.contains(&service) {
        return error(
            StatusCode::NOT_FOUND,
            "UNKNOWN_SERVICE",
            format!("unknown service '{service}'"),
            &request_id,
        );
    }
    let Some(upstream) = state.config.upstreams.get(service) else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "SERVICE_NOT_CONFIGURED",
            format!("no upstream configured for '{service}'"),
            &request_id,
        );
    };

    let mut url = format!("{}/{}", upstream.trim_end_matches('/'), service_path);
    if let Some(query) = uri.query() {
        url.push('?');
        url.push_str(query);
    }

    let mut upstream_req = state
        .http
        .request(method, &url)
        .header(HEADER_REQUEST_ID, &request_id)
        .header(HEADER_CLIENT_ID, &api_key)
        .body(body);
    for name in [header::CONTENT_TYPE, header::ACCEPT] {
        if let Some(value) = headers.get(&name) {
            upstream_req = upstream_req.header(name, value);
        }
    }

    let upstream_resp = match upstream_req.send().await {
        Ok(r) => r,
        Err(e) => {
            let (status, code) = if e.is_timeout() {
                (StatusCode::GATEWAY_TIMEOUT, "UPSTREAM_TIMEOUT")
            } else {
                (StatusCode::BAD_GATEWAY, "UPSTREAM_ERROR")
            };
            return error(
                status,
                code,
                format!("{service} upstream: {e}"),
                &request_id,
            );
        }
    };

    let status = upstream_resp.status();
    let content_type = upstream_resp.headers().get(header::CONTENT_TYPE).cloned();
    let body = match upstream_resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return error(
                StatusCode::BAD_GATEWAY,
                "UPSTREAM_ERROR",
                format!("{service} upstream: {e}"),
                &request_id,
            );
        }
    };

    let mut resp = (status, body).into_response();
    if let Some(ct) = content_type {
        resp.headers_mut().insert(header::CONTENT_TYPE, ct);
    }
    resp.headers_mut().insert(
        HEADER_REQUEST_ID,
        HeaderValue::from_str(&request_id).unwrap(),
    );
    resp
}
