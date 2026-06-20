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

// Check GitHub Releases for a newer signed version and install it silently, then
// restart. Any failure (offline, no update, endpoint unreachable) is non-fatal.
async fn check_for_update(app: tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return,
    };
    if let Ok(Some(update)) = updater.check().await {
        if update
            .download_and_install(|_, _| {}, || {})
            .await
            .is_ok()
        {
            app.restart();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Autostart on login. LaunchAgent is the macOS strategy; on Linux this
        // writes a ~/.config/autostart entry, on Windows a registry Run key.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Register autostart on first run (idempotent).
            use tauri_plugin_autostart::ManagerExt;
            let _ = app.autolaunch().enable();

            // Check for updates in the background so launch is never blocked.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(check_for_update(handle));
            Ok(())
        })
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
