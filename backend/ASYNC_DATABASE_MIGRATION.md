# Async Database Operations Migration

## Issue

The original codebase had blocking database operations in async handlers, which is an anti-pattern in async Rust that can block the entire async runtime and severely impact performance.

## Solution

Implemented a `DbService` that wraps blocking Diesel operations in `tokio::task::spawn_blocking` to prevent blocking the async runtime.

## Implementation

### New DbService

Created `src/services/db_service.rs` with async wrappers:

```rust
use tokio::task;

pub struct DbService {
    pool: Arc<DbPool>,
}

impl DbService {
    // Execute blocking operations asynchronously
    pub async fn execute<F, R>(&self, operation: F) -> Result<R, AppError>
    where
        F: FnOnce(&mut diesel::PgConnection) -> Result<R, diesel::result::Error> + Send + 'static,
        R: Send + 'static,
    {
        let pool = self.pool.clone();
        
        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            operation(&mut conn)
        }).await?
    }
}
```

### Updated AppServices

Added `DbService` to the main application services:

```rust
#[derive(Clone)]
pub struct AppServices {
    pub db_pool: Arc<DbPool>,
    pub session_manager: SessionManager,
    pub db_service: services::DbService, // Added this
}
```

### Migration Pattern

**Before (Blocking):**
```rust
pub async fn get_posts(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<FrontendPost>>, AppError> {
    let mut conn = services.db_pool.get()?; // BLOCKS ASYNC RUNTIME
    let posts = Post::list(&mut conn)?;     // BLOCKS ASYNC RUNTIME
    Ok(ResponseJson(posts))
}
```

**After (Non-blocking):**
```rust
pub async fn get_posts(
    State(services): State<AppServices>
) -> Result<ResponseJson<Vec<FrontendPost>>, AppError> {
    let posts = services.db_service.execute(|conn| {
        Post::list(conn)  // Runs in background thread
    }).await?;
    Ok(ResponseJson(posts))
}
```

## Controllers Migrated

- ✅ `posts.rs` - Fully migrated (example implementation)

## Controllers Remaining

The following controllers still need migration to use `services.db_service.execute()`:

- `media_controller.rs` - ~3 operations
- `navigation.rs` - ~15 operations  
- `system.rs` - ~6 operations
- `media.rs` - ~3 operations
- `auth.rs` - ~1 operation
- `users.rs` - ~4 operations
- `admin.rs` - ~6 operations

## Migration Steps for Remaining Controllers

1. Replace direct `services.db_pool.get()` calls
2. Wrap database operations in `services.db_service.execute()`
3. Use `move` closures to transfer ownership of parameters
4. Handle error mapping for NotFound cases

## Benefits

- **Performance**: No longer blocks the async runtime
- **Scalability**: Better concurrent request handling
- **Compatibility**: Maintains existing Diesel model interfaces
- **Safety**: Proper async/await patterns

## Example Patterns

### Simple Read Operation
```rust
let items = services.db_service.execute(|conn| {
    Model::list(conn)
}).await?;
```

### Read with Parameter
```rust
let item = services.db_service.execute(move |conn| {
    Model::find_by_id(conn, id)
}).await?;
```

### Create Operation
```rust
let created = services.db_service.execute(move |conn| {
    Model::create(conn, new_item)
}).await?;
```

### Optional Result with Error Mapping
```rust
let item = services.db_service.execute_optional(move |conn| {
    Ok(Model::find_by_id(conn, id)?)
}).await?
    .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;
```

This migration ensures the application follows Rust async best practices while maintaining compatibility with the existing Diesel ORM.
