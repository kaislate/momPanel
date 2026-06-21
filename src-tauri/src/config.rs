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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            zip: None,
            clock_mode: default_clock(),
            ui_scale: default_scale(),
            hide_controls: false,
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
}
