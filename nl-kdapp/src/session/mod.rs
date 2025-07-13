//! Session Management
//! 
//! Handles user sessions and links web users to blockchain identities.
//! Based on patterns from kasperience's kaspa-auth implementation.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

pub mod token;

pub use token::{SessionToken, Claims};

/// Manages active sessions for the NL interface
pub struct Manager {
    sessions: RwLock<HashMap<String, Session>>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub episode_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_session(&self) -> Result<SessionToken> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        let session = Session {
            id: session_id.clone(),
            user_id: None,
            episode_ids: Vec::new(),
            created_at: now,
            expires_at,
        };

        self.sessions.write().unwrap().insert(session_id.clone(), session);

        SessionToken::create(&session_id, expires_at)
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().unwrap().get(session_id).cloned()
    }

    pub fn add_episode_to_session(&self, session_id: &str, episode_id: String) -> Result<()> {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.episode_ids.push(episode_id);
            Ok(())
        } else {
            anyhow::bail!("Session not found")
        }
    }

    pub fn cleanup_expired(&self) {
        let now = Utc::now();
        self.sessions.write().unwrap().retain(|_, session| {
            session.expires_at > now
        });
    }
}