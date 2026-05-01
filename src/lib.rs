pub mod auth;
pub mod config;
pub mod db;
pub mod deposit;
pub mod errors;
pub mod handlers;

use anyhow::Context;
use axum::{
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method},
    routing::{delete, get, post, put, get_service},
    Router,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Once;
use std::time::Instant;
use std::{sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

/// Shared application state.
pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
    pub config: config::Config,
    /// Per-employee attendance rate limiter: employee_id → list of request timestamps.
    pub rate_limiter: Mutex<HashMap<String, Vec<Instant>>>,
}

static TRACING_INIT: Once = Once::new();

pub fn init_tracing() {
    TRACING_INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .try_init();
    });
}

pub async fn run_server_until<F>(shutdown_signal: F) -> anyhow::Result<()>
where
    F: Future<Output = ()> + Send + 'static,
{
    init_tracing();

    let cfg = config::Config::from_env()?;

    let manager = SqliteConnectionManager::file(&cfg.database_url);
    let pool = Pool::builder()
        .max_size(cfg.db_pool_size)
        .build(manager)
        .context("Failed to create database connection pool")?;

    {
        let conn = pool.get().context("Failed to get connection for migration")?;
        db::schema::run_migrations(&conn)?;
    }

    let allowed_origin: HeaderValue = cfg
        .allowed_origin
        .parse()
        .context("Invalid CORS_ALLOWED_ORIGIN value")?;

    let state = Arc::new(AppState {
        pool,
        config: cfg.clone(),
        rate_limiter: Mutex::new(HashMap::new()),
    });

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    let frontend_dir = std::env::var("FRONTEND_DIR")
        .unwrap_or_else(|_| {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            exe_dir.join("frontend-dist").to_string_lossy().to_string()
        });

    let frontend_path = std::path::Path::new(&frontend_dir);
    if !frontend_path.exists() {
        tracing::warn!("Frontend directory not found: {}", frontend_dir);
    }

    let index_path = format!("{}/index.html", frontend_dir);
    if !std::path::Path::new(&index_path).exists() {
        tracing::warn!("Frontend index.html not found: {}", index_path);
    }

    let serve_dir = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(format!("{}/index.html", frontend_dir)));

    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/auth/login", post(auth::login))
        .route("/employees", get(handlers::list_employees))
        .route("/employees/:id", get(handlers::get_employee))
        .route("/attendance/check-in", post(handlers::check_in))
        .route("/attendance/check-out", post(handlers::check_out))
        .route("/attendance", get(handlers::list_attendance))
        .route("/employees/:id/attendance", get(handlers::get_employee_attendance));

    let protected_routes = Router::new()
        .route("/employees", post(handlers::create_employee))
        .route("/employees/:id", put(handlers::update_employee))
        .route("/employees/:id", delete(handlers::delete_employee))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::verify_admin_middleware,
        ));

    let app = public_routes
        .merge(protected_routes)
        .route("/", get_service(serve_dir.clone()))
        .route("/assets/*path", get_service(serve_dir))
        .layer(cors)
        .with_state(state);

    let addr = cfg.bind_addr;
    tracing::info!("Starting server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::warn!("Address {} already in use, waiting 2 seconds and retrying...", addr);
            tokio::time::sleep(Duration::from_secs(2)).await;
            tokio::net::TcpListener::bind(&addr)
                .await
                .context("Failed to bind after retry")?
        }
        Err(e) => return Err(e).context("Failed to bind address"),
    };

    axum::Server::from_tcp(listener.into_std()?)?
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal)
        .await
        .context("Server error")?;

    Ok(())
}
