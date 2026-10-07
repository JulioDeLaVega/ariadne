use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::Next,
    web, Error, HttpResponse,
};

struct Bucket {
    tokens: f64,
    last: Instant,
}

pub struct RateLimiter {
    capacity: f64,       // max burst size
    refill_per_sec: f64, // sustained rate
    buckets: Mutex<HashMap<IpAddr, Bucket>>,
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_per_sec: f64) -> Self {
        Self {
            capacity: capacity as f64,
            refill_per_sec,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Ok(()) if allowed, Err(wait) with how long until a token is available.
    pub fn check(&self, ip: IpAddr) -> Result<(), Duration> {
        let now = Instant::now();
        let mut map = self.buckets.lock().unwrap_or_else(|e| e.into_inner());

        let b = map.entry(ip).or_insert(Bucket { tokens: self.capacity, last: now });
        let elapsed = now.duration_since(b.last).as_secs_f64();
        b.tokens = (b.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        b.last = now;

        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            Ok(())
        } else {
            Err(Duration::from_secs_f64((1.0 - b.tokens) / self.refill_per_sec))
        }
    }

    /// Drop buckets idle longer than `idle` so memory doesn't grow unbounded.
    pub fn evict_idle(&self, idle: Duration) {
        let mut map = self.buckets.lock().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, b| b.last.elapsed() < idle);
    }
}

pub async fn rate_limit<B: MessageBody + 'static>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let limiter = req.app_data::<web::Data<RateLimiter>>().cloned();
    // peer_addr is the real TCP peer; X-Forwarded-For is client-spoofable.
    let ip = req.peer_addr().map(|a| a.ip());

    if let (Some(limiter), Some(ip)) = (limiter, ip) {
        if let Err(wait) = limiter.check(ip) {
            let resp = HttpResponse::TooManyRequests()
                .insert_header((header::RETRY_AFTER, wait.as_secs().max(1).to_string()))
                .body("rate limit exceeded");
            return Ok(req.into_response(resp));
        }
    }

    next.call(req).await.map(|r| r.map_into_boxed_body())
}