// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use std::sync::Mutex;
// use tauri_plugin_shell;

// Import goose_cli
// use goose_cli;
//
// struct AppState {
//     session_count: Mutex<i32>,
// }
//
// #[derive(Serialize)]
// struct SessionResponse {
//     id: i32,
//     message: String,
// }

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// #[tauri::command]
// fn create_session(state: tauri::State<AppState>) -> SessionResponse {
//     let mut count = state.session_count.lock().unwrap();
//     *count += 1;
//
//     // In a real implementation, you would call goose_cli to create a session
//     // This is a placeholder
//
//     SessionResponse {
//         id: *count,
//         message: format!("Created new session with ID: {}", count),
//     }
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let app_state = AppState {
    //     session_count: Mutex::new(0),
    // };

    tauri::Builder::default()
        // .plugin(tauri_plugin_shell::init())
        // .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        // .invoke_handler(tauri::generate_handler![greet, create_session])
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
