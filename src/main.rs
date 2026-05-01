#[tokio::main]
async fn main() -> anyhow::Result<()> {
    timekeeping::run_server_until(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        tracing::info!("Shutdown signal received, stopping server...");
    })
    .await
}

