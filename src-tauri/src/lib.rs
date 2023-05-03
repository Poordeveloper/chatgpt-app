mod app;
use app::cmd;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![cmd::call, cmd::set, cmd::get])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
