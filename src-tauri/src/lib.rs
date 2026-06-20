mod collectors;
mod config;
mod shortcuts;

use config::AppConfig;

// Dispatch a simple (no-arg) tile collector by name. Unknown names -> "unavailable".
// The integration step adds one match arm per collector, e.g.:
//   "memory" => serde_json::to_value(collectors::memory::read()).unwrap_or(unavail()),
#[tauri::command]
fn read_tile(name: String) -> serde_json::Value {
    match name.as_str() {
        "memory" => serde_json::to_value(collectors::memory::read()).unwrap_or(unavail()),
        "storage" => serde_json::to_value(collectors::storage::read()).unwrap_or(unavail()),
        "wifi" => serde_json::to_value(collectors::wifi::read())
            .unwrap_or(serde_json::json!({"state":"unavailable"})),
        "internet" => serde_json::to_value(collectors::internet::read())
            .unwrap_or(serde_json::json!({"state":"unavailable"})),
        "volume" => serde_json::to_value(collectors::volume::read()).unwrap_or(unavail()),
        "printers" => serde_json::to_value(collectors::printers::read()).unwrap_or(unavail()),
        _ => unavail(),
    }
}

#[tauri::command]
fn read_weather(zip: String) -> serde_json::Value {
    serde_json::to_value(crate::collectors::weather::read(&zip))
        .unwrap_or_else(|_| serde_json::json!({"state":"unavailable"}))
}

fn unavail() -> serde_json::Value {
    serde_json::json!({ "state": "unavailable" })
}

#[tauri::command]
fn get_config() -> AppConfig {
    config::load()
}

// Merge a PARTIAL config update into the existing config so that, e.g., toggling the
// clock does not wipe the saved weather ZIP (and vice-versa). Only the keys present in
// `patch` are applied; everything else is preserved.
#[tauri::command]
fn set_config(cfg: serde_json::Value) -> Result<AppConfig, String> {
    let mut current = config::load();
    if let Some(v) = cfg.get("zip") {
        if let Some(s) = v.as_str() {
            current.zip = Some(s.to_string());
        } else if v.is_null() {
            current.zip = None;
        }
    }
    if let Some(s) = cfg.get("clock_mode").and_then(|v| v.as_str()) {
        current.clock_mode = s.to_string();
    }
    config::save(&current)?;
    Ok(current)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_tile,
            get_config,
            set_config,
            read_weather,
            shortcuts::open_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running momPanel");
}
