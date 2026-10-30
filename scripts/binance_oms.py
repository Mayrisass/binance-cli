#!/usr/bin/env python3
"""Make signed (HMAC-SHA256) requests to the Binance REST API.

Usage:
    export BINANCE_API_KEY="your-key"
    export BINANCE_API_SECRET="your-secret"
    python binance_oms.py METHOD PATH [key=value ...]

Examples:
    python binance_oms.py GET  /fapi/v1/apiReferral/overview type=1
    python binance_oms.py GET  /sapi/v1/apiReferral/ifNewUser apiAgentCode=YOURCODE
    python binance_oms.py POST /sapi/v1/apiReferral/userCustomization customerId=user001 apiAgentCode=YOURCODE

The base URL is picked from the path prefix (/fapi -> fapi.binance.com,
/dapi -> dapi.binance.com, /eapi -> eapi.binance.com, /papi -> papi.binance.com,
everything else -> api.binance.com). Override with BINANCE_BASE_URL.
"""

import hashlib
import hmac
import json
import os
import sys
import time
from urllib.parse import urlencode

import requests

BASE_URLS = {
    "/fapi/": "https://fapi.binance.com",
    "/dapi/": "https://dapi.binance.com",
    "/eapi/": "https://eapi.binance.com",
    "/papi/": "https://papi.binance.com",
}
DEFAULT_BASE_URL = "https://api.binance.com"
METHODS = {"GET", "POST", "PUT", "DELETE"}


def base_url_for(path):
    override = os.environ.get("BINANCE_BASE_URL")
    if override:
        return override.rstrip("/")
    for prefix, url in BASE_URLS.items():
        if path.startswith(prefix):
            return url
    return DEFAULT_BASE_URL


def parse_params(args):
    params = {}
    for arg in args:
        if "=" not in arg:
            sys.exit(f"error: parameter '{arg}' must be in key=value form")
        key, value = arg.split("=", 1)
        params[key] = value
    return params


def signed_request(method, path, params, api_key, api_secret):
    params = dict(params)
    params.setdefault("recvWindow", os.environ.get("BINANCE_RECV_WINDOW", "5000"))
    params["timestamp"] = int(time.time() * 1000)

    query = urlencode(params)
    signature = hmac.new(api_secret.encode(), query.encode(), hashlib.sha256).hexdigest()
    query = f"{query}&signature={signature}"

    url = f"{base_url_for(path)}{path}"
    headers = {"X-MBX-APIKEY": api_key}

    # Binance accepts signed params in the query string for every method;
    # POST/PUT/DELETE also accept a form body, which keeps URLs short.
    if method == "GET":
        return requests.get(f"{url}?{query}", headers=headers, timeout=15)
    headers["Content-Type"] = "application/x-www-form-urlencoded"
    return requests.request(method, url, data=query, headers=headers, timeout=15)


def main(argv):
    if len(argv) < 3 or argv[1] in ("-h", "--help"):
        print(__doc__)
        return 0 if len(argv) > 1 and argv[1] in ("-h", "--help") else 2

    method = argv[1].upper()
    path = argv[2]
    if method not in METHODS:
        sys.exit(f"error: method must be one of {', '.join(sorted(METHODS))}")
    if not path.startswith("/"):
        sys.exit("error: path must start with '/' (e.g. /sapi/v1/...)")

    api_key = os.environ.get("BINANCE_API_KEY")
    api_secret = os.environ.get("BINANCE_API_SECRET")
    if not api_key or not api_secret:
        sys.exit("error: BINANCE_API_KEY and BINANCE_API_SECRET must be set")

    resp = signed_request(method, path, parse_params(argv[3:]), api_key, api_secret)

    print(f"HTTP {resp.status_code}", file=sys.stderr)
    for h in ("x-mbx-used-weight-1m", "x-mbx-order-count-1m"):
        if h in resp.headers:
            print(f"{h}: {resp.headers[h]}", file=sys.stderr)
    try:
        print(json.dumps(resp.json(), indent=2))
    except ValueError:
        print(resp.text)
    return 0 if resp.ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
