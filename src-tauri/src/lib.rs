mod collectors;
mod config;
mod shortcuts;

use config::AppConfig;

// Dispatch a simple (no-arg) tile collector by name. Unknown names -> "unavailable".
// Collectors shell out to system tools (nmcli/lpstat/wpctl) or do a TCP probe, which
// can block; the async command offloads the work to a blocking thread pool so the
// IPC worker thread is never stalled.
#[tauri::command]
async fn read_tile(name: String) -> serde_json::Value {
    tauri::async_runtime::spawn_blocking(move || read_tile_sync(&name))
        .await
        .unwrap_or_else(|_| unavail())
}

fn read_tile_sync(name: &str) -> serde_json::Value {
    match name {
        "cpu" => serde_json::to_value(collectors::cpu::read()).unwrap_or_else(|_| unavail()),
        "memory" => serde_json::to_value(collectors::memory::read()).unwrap_or_else(|_| unavail()),
        "storage" => {
            serde_json::to_value(collectors::storage::read()).unwrap_or_else(|_| unavail())
        }
        "wifi" => serde_json::to_value(collectors::wifi::read()).unwrap_or_else(|_| unavail()),
        "internet" => {
            serde_json::to_value(collectors::internet::read()).unwrap_or_else(|_| unavail())
        }
        "volume" => serde_json::to_value(collectors::volume::read()).unwrap_or_else(|_| unavail()),
        "printers" => {
            serde_json::to_value(collectors::printers::read()).unwrap_or_else(|_| unavail())
        }
        _ => unavail(),
    }
}

/// US ZIP must be exactly five digits. This is the real trust boundary — the frontend
/// modal check is UX only and cannot be relied upon (any IPC caller can reach here).
fn is_valid_zip(zip: &str) -> bool {
    zip.len() == 5 && zip.bytes().all(|b| b.is_ascii_digit())
}

#[tauri::command]
async fn read_weather(zip: String) -> serde_json::Value {
    if !is_valid_zip(&zip) {
        return unavail();
    }
    tauri::async_runtime::spawn_blocking(move || {
        serde_json::to_value(crate::collectors::weather::read(&zip)).unwrap_or_else(|_| unavail())
    })
    .await
    .unwrap_or_else(|_| unavail())
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
            if !is_valid_zip(s) {
                return Err("zip must be exactly five digits".into());
            }
            current.zip = Some(s.to_string());
        } else if v.is_null() {
            current.zip = None;
        }
    }
    // clock_mode is constrained to the two known values; ignore anything else.
    if let Some(s) = cfg.get("clock_mode").and_then(|v| v.as_str()) {
        if s == "analog" || s == "digital" {
            current.clock_mode = s.to_string();
        }
    }
    // ui_scale is one of three known sizes; ignore anything else.
    if let Some(s) = cfg.get("ui_scale").and_then(|v| v.as_str()) {
        if s == "normal" || s == "big" || s == "biggest" {
            current.ui_scale = s.to_string();
        }
    }
    if let Some(b) = cfg.get("hide_controls").and_then(|v| v.as_bool()) {
        current.hide_controls = b;
    }
    if let Some(b) = cfg.get("auto_update").and_then(|v| v.as_bool()) {
        current.auto_update = b;
    }
    if let Some(b) = cfg.get("hide_help").and_then(|v| v.as_bool()) {
        current.hide_help = b;
    }
    config::save(&current)?;
    Ok(current)
}

// App version (from tauri.conf.json), shown in the info panel.
#[tauri::command]
fn app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

// Open the project's GitHub page in the user's default browser. Fixed URL (no
// arbitrary input), so there is nothing to inject.
#[tauri::command]
fn open_github() -> Result<(), String> {
    let url = "https://github.com/kaislate/momPanel";
    #[cfg(target_os = "linux")]
    let r = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let r: std::io::Result<std::process::Child> =
        Err(std::io::Error::new(std::io::ErrorKind::Other, "unsupported"));
    r.map(|_| ()).map_err(|e| e.to_string())
}

// Manual "check for updates": returns a short status for the info panel. If an update
// is found it installs and restarts.
#[tauri::command]
async fn check_updates(app: tauri::AppHandle) -> String {
    use tauri_plugin_updater::UpdaterExt;
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return "Updater unavailable".into(),
    };
    match updater.check().await {
        Ok(Some(update)) => {
            if update.download_and_install(|_, _| {}, || {}).await.is_ok() {
                app.restart();
            }
            "Update failed".into()
        }
        Ok(None) => "You're up to date".into(),
        Err(_) => "Couldn't check (are you online?)".into(),
    }
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

// Read whether the app is set to launch on login.
#[tauri::command]
fn get_autostart(app: tauri::AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

// Turn launch-on-login on or off (the About panel's "Start at login" toggle).
#[tauri::command]
fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    let r = if enabled { mgr.enable() } else { mgr.disable() };
    r.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Work around a WebKitGTK DMABUF rendering bug that causes glitchy rendering and
    // flaky mouse input on many newer Wayland/Mesa setups (common with AppImages).
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Autostart on login. LaunchAgent is the macOS strategy; on Linux this
        // writes a ~/.config/autostart entry, on Windows a registry Run key.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            use tauri_plugin_autostart::ManagerExt;
            // Enable autostart ONCE on first run; after that respect the user's choice
            // (via the "Start at login" toggle) instead of forcing it on every launch.
            let mut cfg = config::load();
            if !cfg.autostart_initialized {
                let _ = app.autolaunch().enable();
                cfg.autostart_initialized = true;
                let _ = config::save(&cfg);
            }

            // Auto-check for updates on launch, unless the user turned it off.
            if cfg.auto_update {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(check_for_update(handle));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read_tile,
            get_config,
            set_config,
            read_weather,
            app_version,
            open_github,
            check_updates,
            get_autostart,
            set_autostart,
            shortcuts::open_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running momPanel");
}
