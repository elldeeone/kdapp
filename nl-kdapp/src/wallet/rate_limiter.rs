//! Rate limiting for server wallet to prevent abuse

use anyhow::{Result, bail};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct SessionLimits {
    pub transactions_per_hour: u32,
    pub episodes_per_day: u32,
    pub max_total_transactions: u32,
}

impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            transactions_per_hour: 20,
            episodes_per_day: 5,
            max_total_transactions: 100,
        }
    }
}

#[derive(Debug)]
struct SessionUsage {
    transactions_this_hour: Vec<Instant>,
    episodes_today: Vec<Instant>,
    total_transactions: u32,
    first_seen: Instant,
}

impl Default for SessionUsage {
    fn default() -> Self {
        Self {
            transactions_this_hour: Vec::new(),
            episodes_today: Vec::new(),
            total_transactions: 0,
            first_seen: Instant::now(),
        }
    }
}

pub struct RateLimiter {
    limits: SessionLimits,
    sessions: HashMap<String, SessionUsage>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let limits = SessionLimits {
            transactions_per_hour: std::env::var("MAX_TX_PER_SESSION_HOUR")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(20),
            episodes_per_day: std::env::var("MAX_EPISODES_PER_SESSION_DAY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            max_total_transactions: 100,
        };
        
        Self {
            limits,
            sessions: HashMap::new(),
        }
    }
    
    /// Check if session can make a transaction and update usage
    pub fn check_and_update(&mut self, session_id: &str) -> Result<()> {
        let now = Instant::now();
        let usage = self.sessions.entry(session_id.to_string())
            .or_insert_with(SessionUsage::default);
        
        // Clean up old entries
        usage.transactions_this_hour.retain(|t| now.duration_since(*t) < Duration::from_secs(3600));
        usage.episodes_today.retain(|t| now.duration_since(*t) < Duration::from_secs(86400));
        
        // Check hourly transaction limit
        if usage.transactions_this_hour.len() >= self.limits.transactions_per_hour as usize {
            bail!(
                "Rate limit exceeded: Maximum {} transactions per hour",
                self.limits.transactions_per_hour
            );
        }
        
        // Check total transaction limit
        if usage.total_transactions >= self.limits.max_total_transactions {
            bail!(
                "Session limit exceeded: Maximum {} total transactions",
                self.limits.max_total_transactions
            );
        }
        
        // Update usage
        usage.transactions_this_hour.push(now);
        usage.total_transactions += 1;
        
        Ok(())
    }
    
    /// Check if session can create a new Episode
    pub fn check_episode_creation(&mut self, session_id: &str) -> Result<()> {
        let now = Instant::now();
        let usage = self.sessions.entry(session_id.to_string())
            .or_insert_with(SessionUsage::default);
        
        // Clean up old entries
        usage.episodes_today.retain(|t| now.duration_since(*t) < Duration::from_secs(86400));
        
        // Check daily Episode limit
        if usage.episodes_today.len() >= self.limits.episodes_per_day as usize {
            bail!(
                "Rate limit exceeded: Maximum {} Episodes per day",
                self.limits.episodes_per_day
            );
        }
        
        // Update usage
        usage.episodes_today.push(now);
        
        Ok(())
    }
    
    /// Get current usage stats for a session
    pub fn get_usage(&self, session_id: &str) -> Option<(u32, u32, u32)> {
        self.sessions.get(session_id).map(|usage| {
            let now = Instant::now();
            let recent_tx = usage.transactions_this_hour.iter()
                .filter(|t| now.duration_since(**t) < Duration::from_secs(3600))
                .count() as u32;
            let recent_episodes = usage.episodes_today.iter()
                .filter(|t| now.duration_since(**t) < Duration::from_secs(86400))
                .count() as u32;
            (recent_tx, recent_episodes, usage.total_transactions)
        })
    }
    
    /// Clean up old sessions (call periodically)
    pub fn cleanup_old_sessions(&mut self) {
        let now = Instant::now();
        let cutoff = Duration::from_secs(86400 * 7); // 7 days
        
        self.sessions.retain(|_, usage| {
            now.duration_since(usage.first_seen) < cutoff
        });
    }
}