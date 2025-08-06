//! Database service for async operations
//! 
//! This module provides async wrappers around blocking Diesel operations
//! using tokio::task::spawn_blocking to avoid blocking the async runtime.

use std::sync::Arc;
use crate::database::DbPool;
use crate::middleware::errors::AppError;
use tokio::task;

/// Async database service that wraps blocking operations
pub struct DbService {
    pool: Arc<DbPool>,
}

impl DbService {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Execute a blocking database operation asynchronously
    pub async fn execute<F, R>(&self, operation: F) -> Result<R, AppError>
    where
        F: FnOnce(&mut diesel::PgConnection) -> Result<R, diesel::result::Error> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            
            operation(&mut conn)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }

    /// Execute a blocking database operation that might return None
    pub async fn execute_optional<F, R>(&self, operation: F) -> Result<Option<R>, AppError>
    where
        F: FnOnce(&mut diesel::PgConnection) -> Result<Option<R>, diesel::result::Error> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let mut conn = pool.get()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            
            operation(&mut conn)
                .map_err(|e| AppError::DatabaseError(e.to_string()))
        })
        .await
        .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    }
}

impl Clone for DbService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
