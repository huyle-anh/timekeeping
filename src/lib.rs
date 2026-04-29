pub mod auth;
pub mod config;
pub mod db;
pub mod deposit;
pub mod errors;
pub mod handlers;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// Shared application state.
pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
    pub config: config::Config,
    /// Per-employee attendance rate limiter: employee_id → list of request timestamps.
    pub rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
}
