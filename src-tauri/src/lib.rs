mod collectors;
mod config;

use config::AppConfig;

// Dispatch a simple (no-arg) tile collector by name. Unknown names -> "unavailable".
// The integration step adds one match arm per collector, e.g.:
//   "memory" => serde_json::to_value(collectors::memory::read()).unwrap_or(unavail()),
#[tauri::command]
fn read_tile(name: String) -> serde_json::Value {
    match name.as_str() {
        // collector arms are inserted here during integration
        _ => unavail(),
    }
}

fn unavail() -> serde_json::Value {
    serde_json::json!({ "state": "unavailable" })
}

#[tauri::command]
fn get_config() -> AppConfig {
    config::load()
}

#[tauri::command]
fn set_config(cfg: AppConfig) -> Result<(), String> {
    config::save(&cfg)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // additional commands (read_weather, open_settings) are added to this list
        // during integration of the weather and shortcuts tasks.
        .invoke_handler(tauri::generate_handler![read_tile, get_config, set_config])
        .run(tauri::generate_context!())
        .expect("error while running momPanel");
}
