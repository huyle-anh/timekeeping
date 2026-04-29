use anyhow::Context;
use axum::{
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method},
    routing::{delete, get, post, put},
    Router,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use timekeeping::{auth, config, db, handlers, AppState};
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let cfg = config::Config::from_env()?;

    // Create database connection pool
    let manager = SqliteConnectionManager::file(&cfg.database_url);
    let pool = Pool::builder()
        .max_size(cfg.db_pool_size)
        .build(manager)
        .context("Failed to create database connection pool")?;

    // Run migrations
    {
        let conn = pool.get().context("Failed to get connection for migration")?;
        db::schema::run_migrations(&conn)?;
    }

    // Parse allowed CORS origin
    let allowed_origin: HeaderValue = cfg
        .allowed_origin
        .parse()
        .context("Invalid CORS_ALLOWED_ORIGIN value")?;

    // Build application state
    let state = Arc::new(AppState {
        pool,
        config: cfg.clone(),
        rate_limiter: Mutex::new(HashMap::new()),
    });

    // CORS layer — restricted to configured origin
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    // Build router — public routes first (no auth)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/auth/login", post(auth::login))
        // Employee read routes are public so the check-in dropdown works without a token
        .route("/employees", get(handlers::list_employees))
        .route("/employees/:id", get(handlers::get_employee))
        // Attendance routes (public — employees check in/out without a token)
        .route("/attendance/check-in", post(handlers::check_in))
        .route("/attendance/check-out", post(handlers::check_out))
        .route("/attendance", get(handlers::list_attendance))
        .route("/employees/:id/attendance", get(handlers::get_employee_attendance));

    // Protected routes (require Admin JWT)
    let protected_routes = Router::new()
        .route("/employees", post(handlers::create_employee))
        .route("/employees/:id", put(handlers::update_employee))
        .route("/employees/:id", delete(handlers::delete_employee))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::verify_admin_middleware,
        ));

    // Merge all routes
    let app = public_routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = cfg.bind_addr;
    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Server error")?;

    Ok(())
}

