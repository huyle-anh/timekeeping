// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, Child};
use std::sync::Mutex;

struct BackendProcess(Mutex<Option<Child>>);

fn main() {
    tauri::Builder::default()
        .manage(BackendProcess(Mutex::new(None)))
        .setup(|app| {
            // Start backend server
            let backend = Command::new(
                std::env::current_exe()?
                    .parent()
                    .unwrap()
                    .join("timekeeping")
            )
            .env("BIND_ADDR", "127.0.0.1:3001")
            .spawn()
            .expect("Failed to start backend");

            let state = app.state::<BackendProcess>();
            *state.0.lock().unwrap() = Some(backend);

            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                let state = event.window().state::<BackendProcess>();
                if let Some(mut child) = state.0.lock().unwrap().take() {
                    let _ = child.kill();
                }
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
