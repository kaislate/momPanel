mod background;
mod collectors;
mod config;
mod diag;
mod hostexec;
mod memwatch;
mod notifier;
mod shortcuts;

use config::AppConfig;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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

/// A `#RRGGBB` hex color (the only form the color picker emits).
fn is_valid_hex_color(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 7 && b[0] == b'#' && b[1..].iter().all(|c| c.is_ascii_hexdigit())
}

/// The preset names the UI may send (plus "custom" for edited palettes).
fn valid_preset(p: &str) -> bool {
    matches!(p, "midnight" | "warm" | "high-contrast" | "custom")
}

/// Clamp a volume floor to the 0.0–1.0 range.
fn clamp_floor(v: f64) -> f32 {
    v.clamp(0.0, 1.0) as f32
}

#[cfg(test)]
mod alert_cfg_tests {
    use super::clamp_floor;
    #[test]
    fn floor_clamps() {
        assert_eq!(clamp_floor(1.5), 1.0);
        assert_eq!(clamp_floor(-0.2), 0.0);
        assert_eq!(clamp_floor(0.6), 0.6);
    }
}

#[cfg(test)]
mod patch_tests {
    use super::apply_patch;
    use crate::config::AppConfig;

    #[test]
    fn companion_solid_panels_patch_applies() {
        let mut c = AppConfig::default();
        let patch = serde_json::json!({
            "companion_solid_hero": true,
            "companion_solid_health": true
        });
        apply_patch(&mut c, &patch).unwrap();
        assert!(c.companion_solid_hero);
        assert!(c.companion_solid_health);
        // And back off again — the merge honors explicit false too.
        let patch = serde_json::json!({ "companion_solid_hero": false });
        apply_patch(&mut c, &patch).unwrap();
        assert!(!c.companion_solid_hero);
        assert!(c.companion_solid_health); // untouched key preserved
    }

    #[test]
    fn companion_match_heights_patch_applies() {
        let mut c = AppConfig::default();
        apply_patch(&mut c, &serde_json::json!({ "companion_match_heights": true })).unwrap();
        assert!(c.companion_match_heights);
        apply_patch(&mut c, &serde_json::json!({ "companion_match_heights": false })).unwrap();
        assert!(!c.companion_match_heights);
    }
}

#[cfg(test)]
mod theme_cfg_tests {
    use super::valid_preset;
    #[test]
    fn known_presets_only() {
        assert!(valid_preset("midnight"));
        assert!(valid_preset("warm"));
        assert!(valid_preset("high-contrast"));
        assert!(valid_preset("custom"));
        assert!(!valid_preset("rainbow"));
    }
}

/// Hide the high-memory escalation modal now and suppress it until memory recovers.
#[tauri::command]
fn dismiss_mem_warn(app: tauri::AppHandle) {
    memwatch::request_dismiss();
    // Dismiss means "stop bugging me": retract the tray notification too, or its
    // unread badge lingers on the dock until manually cleared.
    notifier::retract_notification();
    if let Some(w) = app.get_webview_window("memwarn") {
        let _ = w.hide();
    }
}

/// Bring the main momPanel window to the foreground (escalation modal's "Open" button).
#[tauri::command]
fn open_main_window(app: tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(m) = app.get_webview_window("memwarn") {
        let _ = m.hide();
    }
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

/// Which display backend the Linux build should use. Decided once at startup.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(PartialEq, Eq, Debug)]
enum LinuxBackend {
    X11,
    Wayland,
}

/// Pure backend decision (unit-tested on every platform). Prefer X11/Xwayland so we
/// get real window alpha + stacking control, but only when a display is actually
/// reachable — forcing GDK_BACKEND=x11 with no X server makes GTK fail to open a
/// display and the app won't start. `MOMPANEL_FORCE_WAYLAND` is the escape hatch.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn choose_linux_backend(force_wayland: bool, have_display: bool) -> LinuxBackend {
    if !force_wayland && have_display {
        LinuxBackend::X11
    } else {
        LinuxBackend::Wayland
    }
}

/// Whether the Linux build ended up on X11/Xwayland. Set once in `run()`; read by
/// `supports_transparency()` and the main-window builder. Never set on non-Linux.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
static ON_X11: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Whether this platform supports a real transparent window. False on Linux again
/// as of 0.6.4: 0.6.2 blamed the ghost/stale frames on the legacy render path and
/// re-enabled real transparency, but the field report from the target Zorin 18.1
/// machine (Wayland, WebKitGTK 2.52) shows the MODERN path ghosts too — closed
/// About panels and notification animation trails stay visible through transparent
/// regions (upstream: tauri-apps/tauri#14924). Linux keeps an opaque window and
/// simulates see-through with the wallpaper backdrop (desktop_background()); the
/// webview input, unlike 0.6.1's diagnosis, was never the problem — see hostexec.rs.
#[tauri::command]
fn supports_transparency() -> bool {
    cfg!(not(target_os = "linux"))
}

#[cfg(test)]
mod transparency_tests {
    use super::supports_transparency;

    #[test]
    fn linux_is_opaque_other_platforms_transparent() {
        assert_eq!(supports_transparency(), cfg!(not(target_os = "linux")));
        #[cfg(target_os = "linux")]
        assert!(!supports_transparency());
    }
}

#[cfg(test)]
mod backend_tests {
    use super::{choose_linux_backend, LinuxBackend};

    #[test]
    fn x11_only_when_display_present_and_wayland_not_forced() {
        assert_eq!(choose_linux_backend(false, true), LinuxBackend::X11);
        // No X server/Xwayland reachable -> must stay Wayland (else GTK can't open a display).
        assert_eq!(choose_linux_backend(false, false), LinuxBackend::Wayland);
        // Explicit escape hatch always wins, even with a display available.
        assert_eq!(choose_linux_backend(true, true), LinuxBackend::Wayland);
        assert_eq!(choose_linux_backend(true, false), LinuxBackend::Wayland);
    }
}

/// The user's desktop wallpaper as a base64 `data:` URL, or null if it can't be
/// determined/read. The Linux backdrop draws this behind an opaque webview to fake the
/// see-through look; on Win/mac the frontend uses the real transparent window instead.
/// Offloaded to a blocking thread: it shells out and reads a (possibly multi-MB) file.
#[tauri::command]
async fn desktop_background() -> Option<String> {
    tauri::async_runtime::spawn_blocking(background::resolve)
        .await
        .ok()
        .flatten()
}

// Holds the panel's latest position between throttled writes so a drag (which emits a
// burst of Moved events) touches disk at most every 2s instead of on every pixel.
struct PosThrottle {
    pending: Option<(i32, i32)>,
    last_write: std::time::Instant,
}

impl PosThrottle {
    fn new() -> Self {
        Self { pending: None, last_write: std::time::Instant::now() }
    }
}

#[cfg(test)]
mod position_tests {
    use super::plausible_position;

    #[test]
    fn minimized_parking_spot_is_rejected() {
        // Windows parks minimized windows at (-32000, -32000); persisting that spot
        // makes the panel restore off-screen (invisible app) on the next launch.
        assert!(!plausible_position(-32000, -32000));
        assert!(!plausible_position(-32000, 200));
        assert!(!plausible_position(200, -32000));
    }

    #[test]
    fn ordinary_positions_are_kept() {
        assert!(plausible_position(0, 0));
        assert!(plausible_position(1298, 60));
        // Mildly negative is real on multi-monitor setups (a display left of primary).
        assert!(plausible_position(-1920, -200));
    }
}

// Whether a reported outer position is a real on-screen spot worth remembering.
// Minimizing on Windows "moves" the window to its parking spot (-32000, -32000) and
// WindowEvent::Moved dutifully reports it; persisting that made the next launch
// restore the panel off-screen — an invisible app. Mildly negative coordinates are
// legitimate (monitors left of/above the primary), so only the far-out parking
// range is rejected.
fn plausible_position(x: i32, y: i32) -> bool {
    x > -30000 && y > -30000
}

// Persist the panel's outer position through the serialized config path.
fn persist_position(x: i32, y: i32) {
    let _ = config::update(|c| {
        c.window_x = Some(x);
        c.window_y = Some(y);
        Ok(())
    });
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
    // The whole read-modify-write runs under config::update's lock so two concurrent
    // patches (e.g. clock toggle + weather ZIP) can't drop each other's change.
    config::update(|current| apply_patch(current, &cfg))
}

fn apply_patch(current: &mut AppConfig, cfg: &serde_json::Value) -> Result<(), String> {
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
    if let Some(s) = cfg.get("last_seen_version").and_then(|v| v.as_str()) {
        current.last_seen_version = s.to_string();
    }
    if let Some(b) = cfg.get("mem_warn_enabled").and_then(|v| v.as_bool()) {
        current.mem_warn_enabled = b;
    }
    // Snap to the allowed 70..=90 in steps of 5; ignore anything out of range.
    if let Some(n) = cfg.get("mem_warn_percent").and_then(|v| v.as_f64()) {
        let snapped = (n / 5.0).round() * 5.0;
        if (70.0..=90.0).contains(&snapped) {
            current.mem_warn_percent = snapped as f32;
        }
    }
    // Accept only a well-formed #RRGGBB hex color.
    if let Some(s) = cfg.get("mem_warn_color").and_then(|v| v.as_str()) {
        if is_valid_hex_color(s) {
            current.mem_warn_color = s.to_uppercase();
        }
    }
    if let Some(b) = cfg.get("mem_warn_sound_enabled").and_then(|v| v.as_bool()) {
        current.mem_warn_sound_enabled = b;
    }
    if let Some(s) = cfg.get("mem_warn_sound").and_then(|v| v.as_str()) {
        // Accept a short sound id (letters, digits, hyphen) to avoid path injection.
        if !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
            current.mem_warn_sound = s.to_string();
        }
    }
    if let Some(n) = cfg.get("mem_warn_volume_floor").and_then(|v| v.as_f64()) {
        current.mem_warn_volume_floor = clamp_floor(n);
    }
    if let Some(b) = cfg.get("mem_warn_speech_enabled").and_then(|v| v.as_bool()) {
        current.mem_warn_speech_enabled = b;
    }
    if let Some(b) = cfg.get("mem_warn_pulse_enabled").and_then(|v| v.as_bool()) {
        current.mem_warn_pulse_enabled = b;
    }
    if let Some(b) = cfg.get("mem_warn_escalate_enabled").and_then(|v| v.as_bool()) {
        current.mem_warn_escalate_enabled = b;
    }
    if let Some(theme) = cfg.get("theme").and_then(|v| v.as_object()) {
        let hex = |k: &str, cur: &mut String| {
            if let Some(s) = theme.get(k).and_then(|v| v.as_str()) {
                if is_valid_hex_color(s) {
                    *cur = s.to_uppercase();
                }
            }
        };
        if let Some(p) = theme.get("preset").and_then(|v| v.as_str()) {
            if valid_preset(p) {
                current.theme.preset = p.to_string();
            }
        }
        hex("accent", &mut current.theme.accent);
        hex("bg", &mut current.theme.bg);
        hex("tile", &mut current.theme.tile);
        hex("gauge_ok", &mut current.theme.gauge_ok);
        hex("gauge_warn", &mut current.theme.gauge_warn);
        hex("gauge_bad", &mut current.theme.gauge_bad);
    }
    if let Some(b) = cfg.get("experimental_ui").and_then(|v| v.as_bool()) {
        current.experimental_ui = b;
    }
    if let Some(b) = cfg.get("companion_solid_hero").and_then(|v| v.as_bool()) {
        current.companion_solid_hero = b;
    }
    if let Some(b) = cfg.get("companion_solid_health").and_then(|v| v.as_bool()) {
        current.companion_solid_health = b;
    }
    if let Some(b) = cfg.get("companion_match_heights").and_then(|v| v.as_bool()) {
        current.companion_match_heights = b;
    }
    if let Some(o) = cfg.get("companion_bg_opacity").and_then(|v| v.as_f64()) {
        // Allow a fully-invisible sky (0.0): the frontend now draws a real backdrop
        // behind it — the actual desktop through a transparent window on Win/mac, or a
        // simulated wallpaper (desktop_background()) inside the opaque webview on Linux.
        current.companion_bg_opacity = o.clamp(0.0, 1.0);
    }
    Ok(())
}

// App version (from tauri.conf.json), shown in the info panel.
#[tauri::command]
fn app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

// A friendly OS label for the About panel (e.g. "Windows 11 Pro", "Linux Zorin OS 17").
#[tauri::command]
fn os_info() -> String {
    sysinfo::System::long_os_version()
        .or_else(sysinfo::System::name)
        .unwrap_or_else(|| {
            let os = std::env::consts::OS;
            os[..1].to_uppercase() + &os[1..]
        })
}

// Open the project's GitHub page in the user's default browser. Fixed URL (no
// arbitrary input), so there is nothing to inject.
#[tauri::command]
fn open_github() -> Result<(), String> {
    let url = "https://github.com/kaislate/momPanel";
    // xdg-open is a host shell script chaining into host tools — it needs the same
    // AppImage env scrub as the settings shortcuts (see hostexec.rs).
    #[cfg(target_os = "linux")]
    let r = hostexec::host_command("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
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
    // Linux rendering history, so nobody re-debugs it from a stale premise:
    //   - 0.3.1 forced WEBKIT_DISABLE_DMABUF_RENDERER=1 to work around glitchy
    //     rendering/input on 2023-era WebKitGTK.
    //   - 0.6.1 kept the Linux window opaque (a tauri.linux.conf.json dropped
    //     `transparent: true`) because WebKitGTK's LEGACY render path ghosted stale
    //     frames and ate pointer input in transparent regions (tauri-apps/tauri#14924,
    //     #13157), breaking buttons on Zorin. See-through was simulated by drawing the
    //     wallpaper inside the webview (desktop_background()).
    //   - 0.6.2: removed tauri.linux.conf.json — window transparent on every platform,
    //     believing the DMABUF path composites alpha correctly and only the legacy
    //     path ghosts. The DMABUF-disable override below became opt-in.
    //   - 0.6.4 (current): the field report from the target machine (Zorin 18.1,
    //     Wayland, WebKitGTK 2.52) shows the MODERN path ghosts too — stale frames of
    //     closed panels/notifications persist in transparent regions (tauri#14924).
    //     tauri.linux.conf.json is BACK (opaque window, no transparent key) and
    //     supports_transparency() returns false on Linux, so companion mode uses the
    //     simulated wallpaper backdrop. The renderer override stays opt-in: the modern
    //     path renders fine; only window ALPHA is broken. The 0.6.1 "transparency eats
    //     input" half of the diagnosis was wrong — dead buttons were the AppImage env
    //     poisoning spawned host tools (see hostexec.rs), fixed independently.
    #[cfg(target_os = "linux")]
    if std::env::var_os("MOMPANEL_LEGACY_RENDERER").is_some()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // Prefer X11/Xwayland on Linux: WebKitGTK ghosts window alpha under Wayland
    // (tauri#14924) and Wayland forbids client-controlled stacking. Xwayland runs
    // inside the user's normal Wayland session, so this needs no Xorg login. Falls
    // back to Wayland (opaque + wallpaper sim) if forced or if no display is reachable.
    #[cfg(target_os = "linux")]
    {
        let force_wayland = std::env::var_os("MOMPANEL_FORCE_WAYLAND").is_some();
        let have_display = std::env::var_os("DISPLAY").is_some();
        let on_x11 = choose_linux_backend(force_wayland, have_display) == LinuxBackend::X11;
        if on_x11 {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        let _ = ON_X11.set(on_x11);
    }

    tauri::Builder::default()
        // Registered first so a second launch is intercepted before any window work.
        // This app autostarts on login, so a manual re-launch must surface the running
        // panel rather than spawn a second process.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
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

            // Pre-create the escalation modal (hidden until a memory spike). Keeping it
            // as a persistent hidden window means the watcher only shows/hides it, never
            // builds a window while memory is under pressure.
            let warn = WebviewWindowBuilder::new(
                app.handle(),
                "memwarn",
                WebviewUrl::App("warn.html".into()),
            )
            .title("Memory warning")
            .inner_size(460.0, 200.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .center()
            .visible(false)
            .focused(false)
            .build()?;
            let _ = warn.set_visible_on_all_workspaces(true);

            // Restore the panel to where the user last left it, then reveal it. The
            // window is created hidden (and center:true has already centered it), so a
            // saved position wins without a visible jump; with no saved position the
            // centered default shows through. Coordinates are physical outer pixels,
            // matching what WindowEvent::Moved reports. Best-effort: GNOME Wayland
            // ignores client-side positioning, so restore only takes effect on X11 (and
            // Windows) — Wayland keeps the compositor's placement.
            if let Some(main) = app.get_webview_window("main") {
                if let (Some(x), Some(y)) = (cfg.window_x, cfg.window_y) {
                    // Ignore (and thereby heal) a config poisoned by the pre-0.6.3
                    // bug that saved the minimized parking spot: keep the centered
                    // default instead of restoring off-screen.
                    if plausible_position(x, y) {
                        let _ = main.set_position(tauri::PhysicalPosition::new(x, y));
                    }
                }
                let _ = main.show();
                let _ = main.set_focus();

                // Persist moves at most every 2s (a drag emits a flood of Moved events),
                // and flush the last position on close so the final spot is never lost.
                let throttle = std::sync::Arc::new(std::sync::Mutex::new(PosThrottle::new()));
                let h = app.handle().clone();
                main.on_window_event(move |e| match e {
                    tauri::WindowEvent::Moved(p) => {
                        if !plausible_position(p.x, p.y) {
                            return; // minimize parking, not a user move
                        }
                        let mut st = throttle.lock().unwrap_or_else(|e| e.into_inner());
                        st.pending = Some((p.x, p.y));
                        if st.last_write.elapsed() >= std::time::Duration::from_secs(2) {
                            if let Some((x, y)) = st.pending.take() {
                                persist_position(x, y);
                                st.last_write = std::time::Instant::now();
                            }
                        }
                    }
                    // Closing the MAIN window quits the app (the hidden banner window
                    // would otherwise keep the process alive after the panel is closed).
                    tauri::WindowEvent::CloseRequested { .. } => {
                        let st = throttle.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some((x, y)) = st.pending {
                            persist_position(x, y);
                        }
                        h.exit(0);
                    }
                    _ => {}
                });
            }

            // Start the background RAM watcher (runs even when the panel is hidden).
            memwatch::spawn(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            read_tile,
            get_config,
            set_config,
            read_weather,
            app_version,
            os_info,
            open_github,
            check_updates,
            get_autostart,
            set_autostart,
            dismiss_mem_warn,
            open_main_window,
            supports_transparency,
            desktop_background,
            shortcuts::open_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running momPanel");
}
