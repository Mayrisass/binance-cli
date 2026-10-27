# binance-gateway

A small API gateway that sits in front of five backend services. It handles
authentication, signature verification and rate limiting, then forwards each
request to the right backend.

```
Client
  │
  ▼
Gateway        ← Auth / Signature Verification / Rate Limiting
  │
  ├──→ Market API        /api/v1/market/...
  ├──→ Trading API       /api/v1/trading/...
  ├──→ Transaction API   /api/v1/transaction/...
  ├──→ Wallet API        /api/v1/wallet/...
  └──→ DeFi API          /api/v1/defi/...
```

This is a standalone crate; it does not change the `binance-cli` build.

## Quick start

```sh
cd gateway
cargo build

# 1. Mock backends for all five services on :9000 (canned sample data)
cargo run -- mock

# 2. The gateway on :8080, routed to the mocks
cargo run -- serve --config gateway.example.json

# 3. Call one endpoint on every service
export GATEWAY_API_KEY=demo-key GATEWAY_API_SECRET=demo-secret-change-me
cargo run -- tour
cargo run -- call market "price?symbol=ETH"
cargo run -- call transaction simulate -X POST -d '{"to":"0x...","value":"0"}'
```

To use real backends, point `upstreams` in the config at them. A service with
no upstream configured returns `503 SERVICE_NOT_CONFIGURED`.

## Request signing

Every request under `/api/v1/` needs three headers:

| Header        | Value |
|---------------|-------|
| `X-API-KEY`   | Your API key |
| `X-TIMESTAMP` | Current time in milliseconds since the epoch |
| `X-SIGNATURE` | `hex(HMAC-SHA256(secret, timestamp + METHOD + pathAndQuery + body))` |

`pathAndQuery` is the exact path the client sends, e.g.
`/api/v1/market/price?symbol=BNB`. The timestamp must be within `recvWindowMs`
(default 5000) of server time. Signatures are compared in constant time.

`GatewayClient` in `src/client.rs` does this for you.

## Rate limiting

A token bucket per API key: `rateLimit.burst` requests at once, refilled at
`rateLimit.requestsPerSecond`. Over the limit returns `429` with `Retry-After`.

## Forwarding

`/api/v1/<service>/<path>?<query>` goes to `<upstream>/<path>?<query>` with the
same method and body. `Content-Type` and `Accept` are passed through; the
client's auth headers are not. The gateway adds `X-Request-Id` (also returned
to the client) and `X-Client-Id` (the API key). Upstream timeout is 30s.

## Errors

All gateway errors are JSON: `{"code", "message", "requestId"}`.

| Status | Code |
|--------|------|
| 401 | `UNAUTHORIZED` (missing headers, unknown key, stale timestamp, bad signature) |
| 404 | `UNKNOWN_SERVICE` |
| 429 | `RATE_LIMITED` |
| 502 | `UPSTREAM_ERROR` |
| 503 | `SERVICE_NOT_CONFIGURED` |
| 504 | `UPSTREAM_TIMEOUT` |

Errors from a backend are passed through unchanged.

## Configuration

See `gateway.example.json`. Keep real secrets out of version control.

## Tests

```sh
cargo test
```

Unit tests cover signing and the rate limiter. `tests/e2e.rs` runs the gateway
against the mock backends and checks routing to all five services, auth
failures, rate limiting, unknown and unconfigured services, and a down upstream.
