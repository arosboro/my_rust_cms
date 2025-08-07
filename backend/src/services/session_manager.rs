use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::time::{interval, sleep};
use tracing::{info, warn, error};
use chrono::{Duration, NaiveDateTime, Utc};
use uuid::Uuid;

use diesel::prelude::*;
use crate::{
    database::DbPool,
    models::{Session, NewSession, User},
    middleware::errors::{AppError, ApiResult},
    services::SessionSigner,
};

#[derive(Clone)]
pub struct SessionConfig {
    pub session_duration_hours: i64,
    pub cleanup_interval_minutes: u64,
    pub max_sessions_per_user: usize,
    pub enable_session_refresh: bool,
    pub refresh_threshold_minutes: i64,
    pub enable_token_signing: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_duration_hours: 24,        // 24 hour sessions
            cleanup_interval_minutes: 15,      // Clean up every 15 minutes
            max_sessions_per_user: 5,         // Max 5 concurrent sessions per user
            enable_session_refresh: true,      // Allow automatic session refresh
            refresh_threshold_minutes: 60,     // Refresh if less than 1 hour remaining
            enable_token_signing: true,        // Enable HMAC-SHA256 token signing
        }
    }
}

#[derive(Clone)]
pub struct SessionManager {
    pool: Arc<DbPool>,
    config: SessionConfig,
    signer: Option<SessionSigner>,
}

#[derive(Debug)]
pub struct SessionStats {
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub expired_cleaned: usize,
    pub last_cleanup: NaiveDateTime,
}

impl SessionManager {
    pub fn new(pool: Arc<DbPool>, config: SessionConfig) -> Self {
        Self { 
            pool, 
            config,
            signer: None,
        }
    }

    pub fn new_with_signing(pool: Arc<DbPool>, config: SessionConfig, session_secret: &str) -> Self {
        let signer = if config.enable_token_signing {
            Some(SessionSigner::new(session_secret))
        } else {
            None
        };
        
        Self {
            pool,
            config,
            signer,
        }
    }

    pub fn new_with_defaults(pool: Arc<DbPool>) -> Self {
        Self::new(pool, SessionConfig::default())
    }

    /// Create a new session for a user with automatic cleanup of old sessions
    pub async fn create_session(&self, user_id: i32) -> ApiResult<Session> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // Check if user exists
        User::find_by_id(&mut conn, user_id)?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Clean up old sessions for this user if they exceed the limit
        let current_session_count = Session::count_active_sessions_for_user(&mut conn, user_id)?;
        
        if current_session_count >= self.config.max_sessions_per_user as i64 {
            let removed = Session::delete_old_sessions_for_user(&mut conn, user_id, self.config.max_sessions_per_user - 1)?;
            info!("Removed {} old sessions for user {} to stay within limit", removed, user_id);
        }

        // Create new session token - always store UUID in database, return signed token if signing enabled
        let uuid_token = Uuid::new_v4().to_string();
        
        let expires_at = Utc::now().naive_utc() + Duration::hours(self.config.session_duration_hours);

        let new_session = NewSession {
            user_id: Some(user_id),
            session_token: uuid_token.clone(),
            expires_at: Some(expires_at),
        };

        let mut session = Session::create(&mut conn, new_session)?;
        
        // If signing is enabled, return signed token to caller
        if let Some(ref signer) = self.signer {
            let signed_token = signer.create_signed_token_from_uuid(&uuid_token)
                .map_err(|e| AppError::InternalError(format!("Failed to sign token: {}", e)))?;
            session.session_token = signed_token;
        }
        info!("Created new session for user {}: {}", user_id, session.id);
        
        Ok(session)
    }

    /// Validate and optionally refresh a session
    pub async fn validate_session(&self, token: &str) -> ApiResult<Session> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // Extract the actual token to look up in database
        let lookup_token = if let Some(ref signer) = self.signer {
            // If we have a signer, check if this is a signed token
            if crate::services::SessionSigner::is_signed_token(token) {
                // Verify signature and extract UUID
                signer.verify_signed_token(token)
                    .ok_or(AppError::InvalidToken)?
            } else {
                // Handle unsigned tokens (for backward compatibility)
                token.to_string()
            }
        } else {
            // No signing enabled, use token directly
            token.to_string()
        };
        
        let session = Session::find_by_token(&mut conn, &lookup_token)?
            .ok_or(AppError::InvalidToken)?;

        // Check if session is expired
        if let Some(expires_at) = session.expires_at {
            let now = Utc::now().naive_utc();
            
            if expires_at <= now {
                // Clean up expired session
                let _ = Session::delete(&mut conn, session.id);
                return Err(AppError::ExpiredToken);
            }

            // Check if session should be refreshed
            if self.config.enable_session_refresh {
                let time_remaining = expires_at.signed_duration_since(now);
                let refresh_threshold = Duration::minutes(self.config.refresh_threshold_minutes);
                
                if time_remaining < refresh_threshold {
                    // Refresh the session
                    let new_expires_at = now + Duration::hours(self.config.session_duration_hours);
                    return Ok(Session::refresh_expiration(&mut conn, session.id, new_expires_at)?);
                }
            }
        }

        Ok(session)
    }

    /// Get detailed session information
    pub async fn get_session_info(&self, token: &str) -> ApiResult<crate::models::session::SessionInfo> {
        let session = self.validate_session(token).await?;
        Ok(session.get_session_info())
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: i32) -> ApiResult<Vec<crate::models::session::SessionInfo>> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let sessions = Session::find_by_user_id(&mut conn, user_id)?;
        let session_infos: Vec<_> = sessions.into_iter()
            .map(|s| s.get_session_info())
            .filter(|info| !info.is_expired)
            .collect();
        
        Ok(session_infos)
    }

    /// Logout a specific session
    pub async fn logout_session(&self, token: &str) -> ApiResult<()> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let deleted = Session::delete_by_token(&mut conn, token)?;
        if deleted == 0 {
            return Err(AppError::NotFound("Session not found".to_string()));
        }
        
        info!("Logged out session: {}", token);
        Ok(())
    }

    /// Logout all sessions for a user
    pub async fn logout_all_user_sessions(&self, user_id: i32) -> ApiResult<usize> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let deleted = Session::delete_user_sessions(&mut conn, user_id)?;
        info!("Logged out {} sessions for user {}", deleted, user_id);
        
        Ok(deleted)
    }

    /// Manually trigger session cleanup
    pub async fn cleanup_expired_sessions(&self) -> ApiResult<SessionStats> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let (deleted, total_before, active_remaining) = Session::cleanup_and_get_stats(&mut conn)?;
        
        let stats = SessionStats {
            total_sessions: total_before,
            active_sessions: active_remaining,
            expired_cleaned: deleted,
            last_cleanup: Utc::now().naive_utc(),
        };

        if deleted > 0 {
            info!("Session cleanup: removed {} expired sessions, {} active sessions remaining", 
                deleted, active_remaining);
        }

        Ok(stats)
    }

    /// Start background session cleanup task
    pub async fn start_background_cleanup(self) -> tokio::task::JoinHandle<()> {
        let cleanup_interval = StdDuration::from_secs(self.config.cleanup_interval_minutes * 60);
        
        tokio::spawn(async move {
            info!("Starting session cleanup background task (interval: {} minutes)", 
                self.config.cleanup_interval_minutes);
            
            let mut cleanup_timer = interval(cleanup_interval);
            
            loop {
                cleanup_timer.tick().await;
                
                match self.cleanup_expired_sessions().await {
                    Ok(stats) => {
                        if stats.expired_cleaned > 0 {
                            info!("Background cleanup: removed {} expired sessions", stats.expired_cleaned);
                        }
                    }
                    Err(e) => {
                        error!("Background session cleanup failed: {}", e);
                        // On error, wait a bit before trying again
                        sleep(StdDuration::from_secs(60)).await;
                    }
                }
            }
        })
    }

    /// Get session statistics
    pub async fn get_session_statistics(&self) -> ApiResult<SessionStats> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let now = Utc::now().naive_utc();
        let total_sessions: i64 = diesel::QueryDsl::count(crate::schema::sessions::table)
            .get_result(&mut conn)?;
        
        let active_sessions: i64 = crate::schema::sessions::table
            .filter(crate::schema::sessions::expires_at.gt(now))
            .count()
            .get_result(&mut conn)?;

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            expired_cleaned: 0,
            last_cleanup: now,
        })
    }

    /// Force expire all sessions for a user (useful for security incidents)
    pub async fn force_expire_user_sessions(&self, user_id: i32, reason: &str) -> ApiResult<usize> {
        let mut conn = self.pool.get().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        // Set all user sessions to expire immediately
        let now = Utc::now().naive_utc();
        let updated = diesel::update(crate::schema::sessions::table
            .filter(crate::schema::sessions::user_id.eq(user_id)))
            .set(crate::schema::sessions::expires_at.eq(now))
            .execute(&mut conn)?;

        warn!("Force expired {} sessions for user {} - reason: {}", updated, user_id, reason);
        
        Ok(updated)
    }
}