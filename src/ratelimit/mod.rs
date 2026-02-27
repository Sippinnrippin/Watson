use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct RateLimiter {
    delays: HashMap<String, Instant>,
    delay_ms: u64,
}

impl RateLimiter {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delays: HashMap::new(),
            delay_ms,
        }
    }

    pub async fn wait_for(&mut self, domain: &str) {
        if self.delay_ms == 0 {
            return;
        }

        let now = Instant::now();
        
        if let Some(last_request) = self.delays.get(domain) {
            let elapsed = now.duration_since(*last_request);
            let delay = Duration::from_millis(self.delay_ms);
            
            if elapsed < delay {
                let sleep_time = delay - elapsed;
                tokio::time::sleep(sleep_time).await;
            }
        }
        
        self.delays.insert(domain.to_string(), Instant::now());
    }
}

pub type RateLimiterHandle = Arc<RwLock<RateLimiter>>;

pub fn create_rate_limiter(delay_ms: u64) -> RateLimiterHandle {
    Arc::new(RwLock::new(RateLimiter::new(delay_ms)))
}
