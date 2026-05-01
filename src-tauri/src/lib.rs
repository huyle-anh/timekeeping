use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|_app| {
      std::env::set_var("BIND_ADDR", "127.0.0.1:3001");
      std::env::set_var(
        "CORS_ALLOWED_ORIGIN",
        if cfg!(debug_assertions) {
          "http://localhost:5173"
        } else {
          "tauri://localhost"
        },
      );

      tauri::async_runtime::spawn(async {
        if let Err(err) = timekeeping::run_server_until(std::future::pending::<()>()).await {
          eprintln!("embedded backend failed: {err:#}");
        }
      });

      for _ in 0..30 {
        if TcpStream::connect("127.0.0.1:3001").is_ok() {
          return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
      }

      Err(Box::<dyn std::error::Error>::from(
        "embedded backend did not start on 127.0.0.1:3001 in time",
      ))
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
