//! Volume collector. On Linux reads the default audio sink via PipeWire's `wpctl`.
//! On any other platform, or if `wpctl` is missing/fails, returns `Unavailable`.

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum VolumeData {
    Ok { level_percent: u8, muted: bool },
    Unavailable,
}

/// Pure dispatch: Linux runs the real tool, everything else is Unavailable.
/// Any failure (missing binary, non-zero exit, unparsable output) maps to Unavailable.
pub fn read() -> VolumeData {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        match Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                match parse_wpctl(&text) {
                    Some((level_percent, muted)) => VolumeData::Ok {
                        level_percent,
                        muted,
                    },
                    None => VolumeData::Unavailable,
                }
            }
            _ => VolumeData::Unavailable,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        VolumeData::Unavailable
    }
}

/// Parse `wpctl get-volume` output, e.g. "Volume: 0.65" or "Volume: 0.65 [MUTED]".
/// Returns (level_percent rounded 0-100, muted). None if no fractional value found.
pub fn parse_wpctl(s: &str) -> Option<(u8, bool)> {
    let muted = s.contains("[MUTED]");
    // Find the token after "Volume:" and parse it as a float fraction.
    let frac: f64 = s
        .split_whitespace()
        .skip_while(|t| !t.starts_with("Volume:"))
        .nth(1)
        .and_then(|t| t.parse::<f64>().ok())?;
    let clamped = frac.clamp(0.0, 1.0);
    let level = (clamped * 100.0).round() as u8;
    Some((level, muted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unmuted_level() {
        assert_eq!(parse_wpctl("Volume: 0.65\n"), Some((65, false)));
    }

    #[test]
    fn parses_muted_level() {
        assert_eq!(parse_wpctl("Volume: 0.65 [MUTED]\n"), Some((65, true)));
    }

    #[test]
    fn rounds_to_nearest_percent() {
        assert_eq!(parse_wpctl("Volume: 0.555"), Some((56, false)));
        assert_eq!(parse_wpctl("Volume: 0.004"), Some((0, false)));
        assert_eq!(parse_wpctl("Volume: 1.00"), Some((100, false)));
    }

    #[test]
    fn handles_garbage() {
        assert_eq!(parse_wpctl("not real output"), None);
        assert_eq!(parse_wpctl(""), None);
    }
}
