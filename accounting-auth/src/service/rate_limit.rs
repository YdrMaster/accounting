//! 登录频控：IP + 用户名双维度滑动窗口，进程内内存实现（trait 可替换）

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 默认窗口长度：1 分钟
pub const DEFAULT_WINDOW: Duration = Duration::from_secs(60);
/// 默认窗口内最大尝试次数
pub const DEFAULT_MAX_ATTEMPTS: usize = 5;

/// 频控抽象：单进程内存实现可满足自用，产品化可替换为 Redis 等外部存储。
pub trait RateLimiter: Send + Sync {
    /// 记录一次尝试；放行返回 `Ok(())`，超限返回 `Err(重试等待秒数)`。
    fn check(&self, key: &str) -> Result<(), u64>;
}

/// 基于内存滑动窗口的频控实现。
pub struct MemoryRateLimiter {
    max_attempts: usize,
    window: Duration,
    /// key → 窗口内的尝试时间戳
    attempts: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl MemoryRateLimiter {
    /// 构造指定参数的限流器。
    pub fn new(max_attempts: usize, window: Duration) -> Self {
        Self {
            max_attempts,
            window,
            attempts: Mutex::new(HashMap::new()),
        }
    }

    fn check_at(&self, key: &str, now: Instant) -> Result<(), u64> {
        let mut map = self.attempts.lock().expect("频控锁中毒");
        let deque = map.entry(key.to_string()).or_default();
        // 滑出窗口的旧记录丢弃
        while deque
            .front()
            .is_some_and(|t| now.duration_since(*t) >= self.window)
        {
            deque.pop_front();
        }
        if deque.len() >= self.max_attempts {
            // 最早一次尝试滑出窗口的剩余时间即为 Retry-After
            let oldest = *deque.front().expect("超限队列非空");
            let retry = self.window - now.duration_since(oldest);
            return Err(retry.as_secs().max(1));
        }
        deque.push_back(now);
        // 空 key 清理，防内存缓慢膨胀
        map.retain(|_, v| !v.is_empty());
        Ok(())
    }
}

impl Default for MemoryRateLimiter {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ATTEMPTS, DEFAULT_WINDOW)
    }
}

impl RateLimiter for MemoryRateLimiter {
    fn check(&self, key: &str) -> Result<(), u64> {
        self.check_at(key, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_max_then_rejects_with_retry_after() {
        let limiter = MemoryRateLimiter::new(5, Duration::from_secs(60));
        for i in 0..5 {
            assert!(limiter.check("ip|user").is_ok(), "第 {} 次应放行", i + 1);
        }
        let err = limiter.check("ip|user").unwrap_err();
        assert!((1..=60).contains(&err), "Retry-After 应在窗口内: {err}");
        // 不同 key 互不影响（IP、用户名两个维度独立）
        assert!(limiter.check("ip|other").is_ok());
        assert!(limiter.check("other-ip|user").is_ok());
    }

    #[test]
    fn window_slides() {
        let limiter = MemoryRateLimiter::new(2, Duration::from_secs(60));
        let t0 = Instant::now();
        assert!(limiter.check_at("k", t0).is_ok());
        assert!(limiter.check_at("k", t0 + Duration::from_secs(30)).is_ok());
        assert!(limiter.check_at("k", t0 + Duration::from_secs(59)).is_err());
        // 窗口滑过后放行
        assert!(limiter.check_at("k", t0 + Duration::from_secs(61)).is_ok());
        assert!(limiter.check_at("k", t0 + Duration::from_secs(61)).is_err());
        // 第二次尝试在 t0+30，t0+91 时滑出
        assert!(limiter.check_at("k", t0 + Duration::from_secs(91)).is_ok());
    }
}
