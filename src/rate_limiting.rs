use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, RateLimiter};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type KeyedRateLimiter =
    Arc<Mutex<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>>;
pub const QUOTA_PER_SECOND: u32 = 50; // 50 requests per user per second
