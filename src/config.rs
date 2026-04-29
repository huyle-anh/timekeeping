use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::Path;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub db_pool_size: u32,
    pub bind_addr: SocketAddr,
    /// JWT signing secret. Set JWT_SECRET env var in production.
    pub jwt_secret: String,
    /// Admin username. Set ADMIN_USERNAME env var in production.
    pub admin_username: String,
    /// Admin password. Set ADMIN_PASSWORD env var in production.
    pub admin_password: String,
    /// Allowed CORS origin (e.g. http://localhost:5173).
    pub allowed_origin: String,
    /// Max attendance requests per minute per employee (rate limiting).
    pub rate_limit_per_minute: u32,
}

impl Config {
    /// Create a Config from environment variables, using sensible defaults.
    pub fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "timekeeping.db".to_string());

        // Ensure parent directory exists for the database file
        if let Some(parent) = Path::new(&database_url).parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create database directory: {:?}", parent))?;
            }
        }

        let db_pool_size: u32 = std::env::var("DB_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("DB_POOL_SIZE must be a valid u32")?;

        let bind_addr: SocketAddr = std::env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
            .parse()
            .context("BIND_ADDR must be a valid SocketAddr")?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "timekeeping-secret-key-change-in-production".to_string());

        let admin_username = std::env::var("ADMIN_USERNAME")
            .unwrap_or_else(|_| "admin".to_string());

        let admin_password = std::env::var("ADMIN_PASSWORD")
            .unwrap_or_else(|_| "admin123".to_string());

        let allowed_origin = std::env::var("CORS_ALLOWED_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let rate_limit_per_minute: u32 = std::env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .context("RATE_LIMIT_PER_MINUTE must be a valid u32")?;

        Ok(Config {
            database_url,
            db_pool_size,
            bind_addr,
            jwt_secret,
            admin_username,
            admin_password,
            allowed_origin,
            rate_limit_per_minute,
        })
    }
}
