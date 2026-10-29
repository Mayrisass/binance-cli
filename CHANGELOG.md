# Change log

## Unreleased

- Added `gateway/`, a standalone API gateway crate that sits in front of the Market, Trading, Transaction, Wallet and DeFi APIs. It provides HMAC-SHA256 signature verification, per-key token-bucket rate limiting, configurable upstream routing, mock backends for local development, a signing client and end-to-end tests. It is built separately and does not change the `binance-cli` build.

## 2.2.0 - 2026-10-20

- Added Stocks REST API and WebSocket streams commands (`stocks`, `stocks-streams`).
- Added CMS WebSocket streams commands (`cms-streams`).
- Added W3W Prediction WebSocket streams commands (`w3w-prediction-streams`).
- Updated `binance-sdk` to `76.0.0`.

**Algo**

### Added (1)

- `algo volume-participation` (`POST /sapi/v1/algo/spot/newOrderVp`)

### Changed (2)

- Added parameter `clientAlgoId`
  - affected methods:
    - `algo cancel-algo-order-future-algo`
    - `algo cancel-algo-order-spot-algo`

**CMS WebSocket Streams**

### Added (1)

- `cms-streams com-announcement-en`

**Derivatives Trading COIN Futures**

### Changed (4)

- Added parameter `contractType`
  - affected methods:
    - `futures-coin long-short-ratio`
    - `futures-coin top-trader-long-short-ratio-positions`
- `futures-coin top-trader-long-short-ratio-accounts`: added parameters `pair` and `contractType`, removed parameter `symbol`
- `futures-coin get-income-history`: removed parameter `page`

**Derivatives Trading Portfolio Margin**

### Changed (2)

- `derivatives-trading-portfolio-margin fund-collection-by-asset`: added parameters `amount` and `futureType`
- `derivatives-trading-portfolio-margin new-um-algo-order`: added parameter `closePosition`

### Removed (7)

- `derivatives-trading-portfolio-margin cancel-all-um-open-conditional-orders` (`DELETE /papi/v1/um/conditional/allOpenOrders`)
- `derivatives-trading-portfolio-margin cancel-um-conditional-order` (`DELETE /papi/v1/um/conditional/order`)
- `derivatives-trading-portfolio-margin new-um-conditional-order` (`POST /papi/v1/um/conditional/order`)
- `derivatives-trading-portfolio-margin query-all-current-um-open-conditional-orders` (`GET /papi/v1/um/conditional/openOrders`)
- `derivatives-trading-portfolio-margin query-all-um-conditional-orders` (`GET /papi/v1/um/conditional/allOrders`)
- `derivatives-trading-portfolio-margin query-current-um-open-conditional-order` (`GET /papi/v1/um/conditional/openOrder`)
- `derivatives-trading-portfolio-margin query-um-conditional-order-history` (`GET /papi/v1/um/conditional/orderHistory`)

**Derivatives Trading Portfolio Margin Pro**

### Changed (1)

- `derivatives-trading-portfolio-margin-pro fund-collection-by-asset`: added parameters `amount` and `futureType`

**Derivatives Trading USDS Futures**

### Changed (2)

- `futures-usds modify-order`: added parameter `reduceOnly`
- `futures-usds get-income-history`: removed parameter `page`

**Dual Investment**

### Changed (1)

- `dual-investment subscribe-dual-investment-products`: removed parameter `autoCompoundPlan`

### Removed (1)

- `dual-investment change-auto-compound-status` (`POST /sapi/v1/dci/product/auto_compound/edit-status`)

**Margin Trading**

### Added (1)

- `margin-trading query-margin-accounts-open-otootoco-order-lists` (`GET /sapi/v1/margin/oto/openOrderList`)

**Stocks**

### Added (17)

- `stocks cancel-all-equity-orders` (`POST /sapi/v1/equity/order/cancel-all`)
- `stocks cancel-equity-order` (`POST /sapi/v1/equity/order/cancel`)
- `stocks create-renew-listen-key` (`POST /sapi/v1/equity/listenKey`)
- `stocks current-open-orders` (`GET /sapi/v1/equity/order/open-orders`)
- `stocks equity-order-detail` (`GET /sapi/v1/equity/order/detail`)
- `stocks equity-order-history` (`GET /sapi/v1/equity/order/history`)
- `stocks equity-trade-history` (`GET /sapi/v1/equity/trade/history`)
- `stocks exchange-info` (`GET /sapi/v1/equity/market/exchangeInfo`)
- `stocks latest-quote` (`GET /sapi/v1/equity/market/quote`)
- `stocks place-equity-order` (`POST /sapi/v1/equity/order/place`)
- `stocks sign-us-equity-disclaimer` (`POST /sapi/v1/equity/account/disclaimer`)
- `stocks tokenized-assets` (`GET /sapi/v1/equity/market/tokenized-assets`)
- `stocks tokenized-convert-history` (`GET /sapi/v1/equity/tokenized/history`)
- `stocks tokenized-convert-status` (`GET /sapi/v1/equity/tokenized/convert-status`)
- `stocks tokenized-mint` (`POST /sapi/v1/equity/tokenized/mint`)
- `stocks tokenized-redeem` (`POST /sapi/v1/equity/tokenized/redeem`)
- `stocks-streams`: `calendar-stream`, `kline-stream`, `order-report-stream`, `price-stream`, `quote-stream`, `tradability-stream`, `trading-status-stream`

**Sub Account**

### Changed (1)

- `sub-account get-move-position-history-for-sub-account`: added parameter `productType`

**W3W Prediction**

### Added (11)

- `w3w-prediction apply-mm-deposit` (`POST /sapi/v1/w3w/wallet/prediction/deposit/apply`)
- `w3w-prediction apply-mm-withdraw` (`POST /sapi/v1/w3w/wallet/prediction/withdraw/apply`)
- `w3w-prediction create-otc-blocktrade` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/create`)
- `w3w-prediction fulfil-otc-blocktrade` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/fulfil`)
- `w3w-prediction get-otc-blocktrade-detail` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/detail`)
- `w3w-prediction get-otc-blocktrade-events` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/events`)
- `w3w-prediction get-otc-reserved-balances` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/reserved-balances`)
- `w3w-prediction list-otc-blocktrades` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/list`)
- `w3w-prediction preview-otc-blocktrade` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/preview`)
- `w3w-prediction remove-otc-blocktrades` (`POST /sapi/v1/w3w/wallet/prediction/otc/blocktrade/remove`)
- `w3w-prediction-streams`: `batch-claim-partially-succeeded`, `fund-transfer-failed`, `fund-transfer-succeeded`, `limit-order-fully-filled`, `limit-order-partially-filled`, `limit-order-submission-failed`, `limit-order-submitted-successfully`, `market-buy-order-failed`, `market-buy-order-fully-filled`, `market-resolution-notification`, `market-sell-order-failed-expired`, `market-sell-order-fully-filled`, `prediction-order-book-by-market-id`, `reward-claim-failed`, `reward-claim-succeeded`

**Wallet**

### Added (1)

- `wallet get-spot-asset-tags` (`GET /sapi/v1/spot/asset/tags`)

### Changed (1)

- `wallet query-user-wallet-balance`: added parameter `needBalanceDetail`

## 2.1.1 - 2026-08-11

- Invalid JSON passed via stdin or `--json` now reports the underlying parse error.
- Fixed credential detection so that an empty API secret is no longer treated as a private key.
- Clearer error message when `BINANCE_API_ENV` is set to an unsupported value.

## 2.1.0 - 2026-08-06

- Added WebSocket streams commands to the Rust CLI:
  - `alpha-streams` (13 streams)
  - `spot-streams` (15 market streams, plus `user-data`)
  - `futures-usds-streams` (20 market streams, plus `user-data`)
  - `futures-coin-streams` (19 market streams, plus `user-data`)
  - `derivatives-trading-options-streams` (10 market streams, plus `user-data`)
  - `derivatives-trading-portfolio-margin-streams` (`user-data`)
  - `derivatives-trading-portfolio-margin-pro-streams` (`user-data`)
  - `margin-trading-streams` (`risk-data`, `trade-data`)
- Re-enabled the `fiat` commands.
- `custom` requests now return an error instead of panicking when no profile or `BINANCE_API_KEY`/`BINANCE_SECRET_KEY` environment variables are set.
- Updated `binance-sdk` to `67.0.0`.

**Fiat**

### Added (1)

- `fiat get-order-detail` (`GET /sapi/v1/fiat/get-order-detail`)

### Changed (4)

- `fiat deposit`, `fiat fiat-withdraw`: request fields (`currency`, `amount`, `apiPaymentMethod`, and `accountInfo` for withdrawals) are now individual parameters instead of a single request object
- Added parameters `transactionType`, `beginTime`, `endTime`, `page` and `rows`
  - affected methods:
    - `fiat get-fiat-deposit-withdraw-history` (`GET /sapi/v1/fiat/orders`)
    - `fiat get-fiat-payments-history` (`GET /sapi/v1/fiat/payments`)

## 2.0.0 - 2026-07-28

- Rewrote binance-cli in Rust

### Added (11)

- `alpha full-depth` (`GET /bapi/defi/v1/public/alpha-trade/fullDepth`)
- `derivatives-trading-options tradfi-options-contract` (`POST /eapi/v1/stock/contract`)
- `margin-trading query-liquidation-loan-repay-history` (`GET /sapi/v1/margin/liquidation-loan/repay-history`)
- `margin-trading query-liquidation-loan` (`GET /sapi/v1/margin/liquidation-loan`)
- `margin-trading liquidation-loan-repay` (`POST /sapi/v1/margin/exit-special-key-mode/sapi/v1/margin/liquidation-loan/repay`)
- `sub-account modify-sub-account-api-key-permission` (`POST /sapi/v1/sub-account/subAccountApiPermission`)
- `sub-account delete-sub-account-api-key` (`DELETE /sapi/v1/sub-account/subAccountApi`)
- `sub-account query-sub-account-api-key` (`GET /sapi/v1/sub-account/subAccountApi`)
- `sub-account create-sub-account-api-key` (`POST /sapi/v1/sub-account/subAccountApi`)
- `vip-loan get-viploan-repayment-history` (`GET /sapi/v1/loan/vip/repay/history`)
- `vip-loan query-viploan-fixed-rate-market` (`GET /sapi/v1/loan/vip/fixed/market`)
- `vip-loan vip-loan-fixed-rate-borrow` (`POST /sapi/v1/loan/vip/fixed/borrow`)
- Added W3W Prediction API collection of endpoints.

### Removed (1)

- `check-collateral-repay-rate-stable-rate` `GET /sapi/v1/loan/repay/collateral/rate`


## 1.3.0 - 2026-06-04

**Derivatives Trading Usds Futures**

**Simple Earn**

### Added (1)

- `get-yield-arena-activities` (`GET /sapi/v1/earn/arena/activities`)

**Wallet**

### Added (2)

- `get-country-list` (`GET /sapi/v1/localentity/country/list`)
- `get-region-list` (`GET /sapi/v1/localentity/region/list`)

### Changed (1)

- Added parameter `accountType`
  - affected methods:
    - `dust-convert` (`POST /sapi/v1/asset/dust-convert/convert`)
    - `dust-convertible-assets` (`POST /sapi/v1/asset/dust-convert/query-convertible-assets`)

## 1.2.0 - 2026-05-18

- Added WebSocket streams commands.
- Show short description by default, use "--full-description" to show the full one.

**Derivatives Trading Portfolio Margin**

### Added (7)

- `cancel-all-um-algo-open-orders` (`DELETE /papi/v1/um/algo/allOpenOrders`)
- `cancel-um-algo-order` (`DELETE /papi/v1/um/algo/order`)
- `futures-tradfi-perps-contract` (`POST /papi/v1/um/stock/contract`)
- `new-um-algo-order` (`POST /papi/v1/um/algo/order`)
- `query-all-current-um-open-algo-orders` (`GET /papi/v1/um/algo/openAlgoOrders`)
- `query-current-um-open-algo-order` (`GET /papi/v1/um/algo/algoOrder`)
- `query-um-algo-order-history` (`GET /papi/v1/um/algo/allAlgoOrders`)

**Derivatives Trading Portfolio Margin Pro**

### Added (3)

- `delete-margin-call-level` (`DELETE /sapi/v1/portfolio/margin-call-level`)
- `get-margin-call-level` (`GET /sapi/v1/portfolio/margin-call-level`)
- `set-margin-call-level` (`POST /sapi/v1/portfolio/margin-call-level`)

**Derivatives Trading Usds Futures**

### Changed (3)

- Deleted parameter `page`
  - affected methods:
    - `query-all-algo-orders` (`GET /fapi/v1/allAlgoOrders`)
- Modified parameter `interval`:
  - enum added: `1s`
  - affected methods:
    - `continuous-contract-kline-candlestick-data` (`GET /fapi/v1/continuousKlines`)
    - `index-price-kline-candlestick-data` (`GET /fapi/v1/indexPriceKlines`)
    - `kline-candlestick-data` (`GET /fapi/v1/klines`)
    - `mark-price-kline-candlestick-data` (`GET /fapi/v1/markPriceKlines`)
    - `premium-index-kline-data` (`GET /fapi/v1/premiumIndexKlines`)
- Modified parameter `limit`:
  - required: `true` → `false`
  - affected methods:
    - `basis` (`GET /futures/data/basis`)

**Spot**

### Added (1)

- `historical-block-trades` (`GET /api/v3/historicalBlockTrades`)

### Changed (1)

- Modified parameter `selfTradePreventionMode`:
  - enum added: `TRANSFER`
  - affected methods:
    - `new-order` (`POST /api/v3/order`)
    - `order-cancel-replace` (`POST /api/v3/order/cancelReplace`)
    - `order-oco` (`POST /api/v3/order/oco`)
    - `order-test` (`POST /api/v3/order/test`)
    - `order-list-oco` (`POST /api/v3/orderList/oco`)
    - `order-list-opo` (`POST /api/v3/orderList/opo`)
    - `order-list-opoco` (`POST /api/v3/orderList/opoco`)
    - `order-list-oto` (`POST /api/v3/orderList/oto`)
    - `order-list-otoco` (`POST /api/v3/orderList/otoco`)
    - `sor-order` (`POST /api/v3/sor/order`)
    - `sor-order-test` (`POST /api/v3/sor/order/test`)

**Sub Account**

### Changed (2)

- Added parameter `rows`
  - affected methods:
    - `get-move-position-history-for-sub-account` (`GET /sapi/v1/sub-account/futures/move-position`)
- Deleted parameter `row`
  - affected methods:
    - `get-move-position-history-for-sub-account` (`GET /sapi/v1/sub-account/futures/move-position`)

## 1.1.1 - 2026-04-13

### Updated

- Argument "--profile" takes precedence over env variables
- Validate profile name
- Use console.error for error messages

## 1.1.0 - 2026-04-11

### Added

- Added command to delete profile.

- Exit with error code on failed command.

- Added support for the following products:
    - Algo Trading
    - Alpha
    - C2C
    - Copy Trading
    - Crypto Loan
    - Derivatives Trading (COIN-M Futures)
    - Derivatives Trading (Options)
    - Derivatives Trading (Portfolio Margin)
    - Derivatives Trading (Portfolio Margin Pro)
    - Dual Investment
    - Fiat
    - Gift Card
    - Margin Trading
    - Mining
    - Pay
    - Rebate
    - Simple Earn
    - Staking
    - Sub Account
    - VIP Loan
    - Wallet

### Updated

- Add alias `profile select` for `profile change`
- Add option `--select` for `profile create` to select the profile after creation
- Add option `--force` for `profile create` to overwrite profile if existing

## 1.0.4 - 2026-04-09

### Updated

- Different User-Agent if used from AI Agent

## 1.0.3 - 2026-04-09

### Updated

- Simplified checks for required params

## 1.0.2 - 2026-04-09

### Added

- Add environment to profile commands

### Updated

- Add the request fields as options for POST/PUT endpoints
- Renamed `BINANCE_API_SECRET` to `BINANCE_SECRET_KEY`

## 1.0.1 - 2026-04-07

### Updated

- Accept json or list of params for POST/PUT endpoints

## 1.0.0 - 2026-04-07

- First release
