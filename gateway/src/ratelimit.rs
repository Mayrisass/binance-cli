//! Per-API-key token bucket.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

pub struct RateLimiter {
    rate: f64,
    burst: f64,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64, burst: u32) -> Self {
        Self {
            rate: requests_per_second,
            burst: burst as f64,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Takes one token for `key`, or returns how long to wait before retrying.
    pub fn check(&self, key: &str, now: Instant) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.entry(key.to_string()).or_insert(Bucket {
            tokens: self.burst,
            last_refill: now,
        });
        let elapsed = now
            .saturating_duration_since(bucket.last_refill)
            .as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.rate).min(self.burst);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(())
        } else {
            Err(Duration::from_secs_f64((1.0 - bucket.tokens) / self.rate))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_burst_then_refills() {
        let limiter = RateLimiter::new(2.0, 3);
        let t0 = Instant::now();
        for _ in 0..3 {
            assert!(limiter.check("a", t0).is_ok());
        }
        let wait = limiter.check("a", t0).unwrap_err();
        assert!(wait > Duration::ZERO && wait <= Duration::from_millis(500));

        // Other keys have their own bucket.
        assert!(limiter.check("b", t0).is_ok());

        // 0.5s at 2 req/s refills one token.
        assert!(limiter.check("a", t0 + Duration::from_millis(500)).is_ok());
        assert!(limiter.check("a", t0 + Duration::from_millis(500)).is_err());
    }
}
