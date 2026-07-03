// Persisted user config: weather ZIP (prompted once on first run) and clock mode.
// Stored as JSON at <os-config-dir>/momPanel/config.json. Missing/!corrupt -> defaults.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    /// Show an always-on-top banner when RAM usage crosses `mem_warn_percent`.
    #[serde(default = "default_true")]
    pub mem_warn_enabled: bool,
    /// RAM used-% that triggers the high-memory warning banner. One of 70/75/80/85/90.
    #[serde(default = "default_mem_pct")]
    pub mem_warn_percent: f32,
    /// Banner background color as `#RRGGBB` (text color is auto-contrasted).
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

pub fn save(cfg: &AppConfig) -> Result<(), String> {
    let s = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), s).map_err(|e| e.to_string())
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
}
