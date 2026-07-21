// Persisted user config: weather ZIP (prompted once on first run) and clock mode.
// Stored as JSON at <os-config-dir>/momPanel/config.json. Missing/!corrupt -> defaults.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Bumped on every successful save so cheap pollers (e.g. memwatch) can cache the
/// loaded config and reload only when it actually changed instead of re-reading and
/// re-parsing config.json on every tick.
pub static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Serializes the whole load-modify-save so two overlapping patches can't clobber
/// each other's keys, and so the atomic write below is never interleaved.
static SAVE_LOCK: Mutex<()> = Mutex::new(());

fn default_clock() -> String {
    "digital".into()
}

fn default_scale() -> String {
    "normal".into()
}

fn default_true() -> bool {
    true
}

fn default_mem_pct() -> f32 {
    85.0
}

fn default_mem_color() -> String {
    "#D97706".into() // amber
}

fn default_mem_sound() -> String {
    "suspend-error".into()
}

fn default_volume_floor() -> f32 {
    0.60
}

fn tp() -> String {
    "midnight".into()
}

fn t_accent() -> String {
    "#5b8cff".into()
}

fn t_bg() -> String {
    "#0e1119".into()
}

fn t_tile() -> String {
    "#1b2030".into()
}

fn t_ok() -> String {
    "#5bd6a0".into()
}

fn t_warn() -> String {
    "#ffb347".into()
}

fn t_bad() -> String {
    "#ff5d5d".into()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    #[serde(default = "tp")]
    pub preset: String,
    #[serde(default = "t_accent")]
    pub accent: String,
    #[serde(default = "t_bg")]
    pub bg: String,
    #[serde(default = "t_tile")]
    pub tile: String,
    #[serde(default = "t_ok")]
    pub gauge_ok: String,
    #[serde(default = "t_warn")]
    pub gauge_warn: String,
    #[serde(default = "t_bad")]
    pub gauge_bad: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            preset: tp(),
            accent: t_accent(),
            bg: t_bg(),
            tile: t_tile(),
            gauge_ok: t_ok(),
            gauge_warn: t_warn(),
            gauge_bad: t_bad(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default)]
    pub zip: Option<String>,
    #[serde(default = "default_clock")]
    pub clock_mode: String,
    /// UI size for low-vision users: "normal" | "big" | "biggest".
    #[serde(default = "default_scale")]
    pub ui_scale: String,
    /// When true, hide all buttons/links for a clean info-only view.
    #[serde(default)]
    pub hide_controls: bool,
    /// Automatically check for and install updates on launch (on by default).
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Hide the per-tile "?" help dots when true (hidden by default — tap "?" to show).
    #[serde(default = "default_true")]
    pub hide_help: bool,
    /// One-time flag: whether we've done the "enable autostart on first run" step.
    #[serde(default)]
    pub autostart_initialized: bool,
    /// The app version we last showed a "what's new" note for (to detect updates).
    #[serde(default)]
    pub last_seen_version: String,
    /// Show an alert when RAM usage crosses `mem_warn_percent`.
    #[serde(default = "default_true")]
    pub mem_warn_enabled: bool,
    /// RAM used-% that triggers the high-memory warning banner. One of 70/75/80/85/90.
    #[serde(default = "default_mem_pct")]
    pub mem_warn_percent: f32,
    /// Escalation-modal background color as `#RRGGBB` (text color is auto-contrasted).
    #[serde(default = "default_mem_color")]
    pub mem_warn_color: String,
    /// Play an alert tone when the warning fires.
    #[serde(default = "default_true")]
    pub mem_warn_sound_enabled: bool,
    /// Alert tone id (freedesktop sound name, e.g. "suspend-error").
    #[serde(default = "default_mem_sound")]
    pub mem_warn_sound: String,
    /// Volume floor 0.0–1.0 forced only while the alert plays (never lowers).
    #[serde(default = "default_volume_floor")]
    pub mem_warn_volume_floor: f32,
    /// Speak the warning (naming the top process) via speech-dispatcher.
    #[serde(default = "default_true")]
    pub mem_warn_speech_enabled: bool,
    /// Re-fire the alert every ~30 s while usage stays critical.
    #[serde(default = "default_true")]
    pub mem_warn_pulse_enabled: bool,
    /// Escalate to a centered modal dialog if the alert is ignored.
    #[serde(default = "default_true")]
    pub mem_warn_escalate_enabled: bool,
    /// Color theme (curated slots + active preset name).
    #[serde(default)]
    pub theme: Theme,
    /// Last known panel position (outer, physical pixels) so it reopens where the
    /// user left it. `None` until moved once — then startup restores it.
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    /// Cached ZIP->lat/lon geocode so the weather refresh doesn't hit the geocoder on
    /// every poll. Reused while `geo_zip` matches the active ZIP.
    #[serde(default)]
    pub geo_zip: Option<String>,
    #[serde(default)]
    pub geo_lat: Option<String>,
    #[serde(default)]
    pub geo_lon: Option<String>,
    #[serde(default)]
    pub geo_place: Option<String>,
    /// Opt-in flag for unfinished UI the frontend can gate on. Off by default.
    #[serde(default)]
    pub experimental_ui: bool,
    /// Companion-mode background opacity, 0.0 (invisible) to 1.0 (solid).
    #[serde(default = "default_companion_bg_opacity")]
    pub companion_bg_opacity: f64,
    /// Companion mode: draw a solid panel behind the time/weather section, so it
    /// stays readable when a busy wallpaper shows through a clear sky.
    #[serde(default)]
    pub companion_solid_hero: bool,
    /// Companion mode: same, behind the "All is well" health card.
    #[serde(default)]
    pub companion_solid_health: bool,
    /// Companion mode: stretch the "All is well" card to the hero section's height
    /// so the two sides read as one congruent layout.
    #[serde(default)]
    pub companion_match_heights: bool,
    /// Companion mode: frosted-glass panes behind the hero and health sections — a
    /// blurred slice of the desktop wallpaper under a translucent tint, while the
    /// surrounding sky stays clear.
    #[serde(default)]
    pub companion_frosted_panels: bool,
    /// Companion mode: frosted-glass background — the whole sky becomes a blurred,
    /// partially-transparent pane of the wallpaper instead of fully clear or solid.
    #[serde(default)]
    pub companion_frosted_bg: bool,
}

fn default_companion_bg_opacity() -> f64 {
    1.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            zip: None,
            clock_mode: default_clock(),
            ui_scale: default_scale(),
            hide_controls: false,
            auto_update: true,
            hide_help: true,
            autostart_initialized: false,
            last_seen_version: String::new(),
            mem_warn_enabled: true,
            mem_warn_percent: 85.0,
            mem_warn_color: default_mem_color(),
            mem_warn_sound_enabled: true,
            mem_warn_sound: default_mem_sound(),
            mem_warn_volume_floor: 0.60,
            mem_warn_speech_enabled: true,
            mem_warn_pulse_enabled: true,
            mem_warn_escalate_enabled: true,
            theme: Theme::default(),
            window_x: None,
            window_y: None,
            geo_zip: None,
            geo_lat: None,
            geo_lon: None,
            geo_place: None,
            experimental_ui: false,
            companion_bg_opacity: 1.0,
            companion_solid_hero: false,
            companion_solid_health: false,
            companion_match_heights: false,
            companion_frosted_panels: false,
            companion_frosted_bg: false,
        }
    }
}

fn config_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("momPanel");
    let _ = std::fs::create_dir_all(&p);
    p.push("config.json");
    p
}

pub fn load() -> AppConfig {
    std::fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Atomic write: serialize to a sibling temp file, then rename over the target so a
/// concurrent reader (memwatch, another set_config) sees either the old or the new
/// file in full, never a half-truncated one. Bumps `GENERATION` on success.
fn write_atomic(cfg: &AppConfig) -> Result<(), String> {
    let s = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    let path = config_path();
    // Same directory as the target so `rename` stays on one filesystem (atomic).
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, s).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    GENERATION.fetch_add(1, Ordering::Release);
    Ok(())
}

pub fn save(cfg: &AppConfig) -> Result<(), String> {
    let _guard = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    write_atomic(cfg)
}

/// Serialized load-modify-save. The closure edits the loaded config under the save
/// lock, so overlapping callers can't drop each other's changes. Returns the saved
/// config. A closure error aborts the save and is propagated unchanged.
pub fn update<F>(f: F) -> Result<AppConfig, String>
where
    F: FnOnce(&mut AppConfig) -> Result<(), String>,
{
    let _guard = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut cfg = load();
    f(&mut cfg)?;
    write_atomic(&cfg)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_missing() {
        let c = AppConfig::default();
        assert_eq!(c.clock_mode, "digital");
        assert_eq!(c.ui_scale, "normal");
        assert!(!c.hide_controls);
        assert!(c.zip.is_none());
    }

    #[test]
    fn ui_scale_defaults_on_partial_json() {
        let c: AppConfig = serde_json::from_str(r#"{"clock_mode":"analog"}"#).unwrap();
        assert_eq!(c.ui_scale, "normal");
    }

    #[test]
    fn parses_partial_json_filling_defaults() {
        let c: AppConfig = serde_json::from_str(r#"{"zip":"90210"}"#).unwrap();
        assert_eq!(c.zip.as_deref(), Some("90210"));
        assert_eq!(c.clock_mode, "digital");
    }

    #[test]
    fn alert_channel_defaults() {
        let c = AppConfig::default();
        assert!(c.mem_warn_sound_enabled);
        assert_eq!(c.mem_warn_sound, "suspend-error");
        assert_eq!(c.mem_warn_volume_floor, 0.60);
        assert!(c.mem_warn_speech_enabled);
        assert!(c.mem_warn_pulse_enabled);
        assert!(c.mem_warn_escalate_enabled);
    }

    #[test]
    fn alert_channels_default_on_partial_json() {
        let c: AppConfig = serde_json::from_str(r#"{"mem_warn_percent":80}"#).unwrap();
        assert_eq!(c.mem_warn_sound, "suspend-error");
        assert!(c.mem_warn_speech_enabled);
    }

    #[test]
    fn theme_defaults_to_midnight() {
        let c = AppConfig::default();
        assert_eq!(c.theme.preset, "midnight");
        assert_eq!(c.theme.accent, "#5b8cff");
        assert_eq!(c.theme.gauge_bad, "#ff5d5d");
    }

    #[test]
    fn theme_fills_defaults_on_partial_json() {
        let c: AppConfig = serde_json::from_str(r##"{"theme":{"accent":"#112233"}}"##).unwrap();
        assert_eq!(c.theme.accent, "#112233");
        assert_eq!(c.theme.preset, "midnight");
        assert_eq!(c.theme.bg, "#0e1119");
    }

    #[test]
    fn new_fields_default_when_missing() {
        let c = AppConfig::default();
        assert!(c.window_x.is_none());
        assert!(c.window_y.is_none());
        assert!(c.geo_zip.is_none());
        assert!(!c.experimental_ui);
    }

    #[test]
    fn new_fields_absent_from_old_config_json() {
        // A config.json written by an older build lacks these keys entirely.
        let c: AppConfig = serde_json::from_str(r#"{"zip":"90210"}"#).unwrap();
        assert!(c.window_x.is_none());
        assert!(c.geo_lat.is_none());
        assert!(!c.experimental_ui);
    }

    #[test]
    fn companion_solid_panels_default_off_and_round_trip() {
        // Off by default, absent from old configs, and survive a save/load cycle.
        let c = AppConfig::default();
        assert!(!c.companion_solid_hero);
        assert!(!c.companion_solid_health);
        let old: AppConfig = serde_json::from_str(r#"{"zip":"90210"}"#).unwrap();
        assert!(!old.companion_solid_hero);
        assert!(!old.companion_solid_health);
        let mut c = AppConfig::default();
        c.companion_solid_hero = true;
        c.companion_solid_health = true;
        let s = serde_json::to_string(&c).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert!(back.companion_solid_hero);
        assert!(back.companion_solid_health);
    }

    #[test]
    fn companion_frosted_default_off_and_round_trip() {
        // Off by default, absent from old configs, and survive a save/load cycle.
        let c = AppConfig::default();
        assert!(!c.companion_frosted_panels);
        assert!(!c.companion_frosted_bg);
        let old: AppConfig = serde_json::from_str(r#"{"zip":"90210"}"#).unwrap();
        assert!(!old.companion_frosted_panels);
        assert!(!old.companion_frosted_bg);
        let mut c = AppConfig::default();
        c.companion_frosted_panels = true;
        c.companion_frosted_bg = true;
        let s = serde_json::to_string(&c).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert!(back.companion_frosted_panels);
        assert!(back.companion_frosted_bg);
    }

    #[test]
    fn companion_match_heights_default_off_and_round_trip() {
        assert!(!AppConfig::default().companion_match_heights);
        let old: AppConfig = serde_json::from_str(r#"{"zip":"90210"}"#).unwrap();
        assert!(!old.companion_match_heights);
        let mut c = AppConfig::default();
        c.companion_match_heights = true;
        let s = serde_json::to_string(&c).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert!(back.companion_match_heights);
    }

    #[test]
    fn window_and_geo_round_trip() {
        let mut c = AppConfig::default();
        c.window_x = Some(120);
        c.window_y = Some(-40);
        c.geo_zip = Some("90210".into());
        c.geo_lat = Some("34.0901".into());
        c.geo_lon = Some("-118.4065".into());
        c.geo_place = Some("Beverly Hills, CA".into());
        c.experimental_ui = true;
        let s = serde_json::to_string(&c).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert_eq!(back.window_x, Some(120));
        assert_eq!(back.window_y, Some(-40));
        assert_eq!(back.geo_zip.as_deref(), Some("90210"));
        assert_eq!(back.geo_lat.as_deref(), Some("34.0901"));
        assert_eq!(back.geo_place.as_deref(), Some("Beverly Hills, CA"));
        assert!(back.experimental_ui);
    }
}
