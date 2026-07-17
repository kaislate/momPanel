//! Volume collector. On Linux reads the default audio sink via PipeWire's `wpctl`;
//! on Windows via Core Audio; on macOS via `osascript` ("get volume settings").
//! On any other platform, or if the underlying query is missing/fails, returns
//! `Unavailable`.

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
        // Pin the locale: parse_wpctl looks for the literal "Volume:" / "[MUTED]" tokens
        // that wpctl would otherwise translate.
        match crate::hostexec::host_command("wpctl")
            .env("LC_ALL", "C")
            .env("LANG", "C")
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

    #[cfg(target_os = "windows")]
    {
        match win_volume() {
            Some((level_percent, muted)) => VolumeData::Ok {
                level_percent,
                muted,
            },
            None => VolumeData::Unavailable,
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        // `get volume settings` is a stable AppleScript one-liner that needs no extra
        // permissions; its output is fixed English tokens, so no locale pinning needed.
        match Command::new("osascript")
            .args(["-e", "get volume settings"])
            .output()
        {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                match parse_osascript(&text) {
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

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        VolumeData::Unavailable
    }
}

/// Windows master volume + mute via Core Audio (IAudioEndpointVolume on the default
/// playback device). Returns None on any COM error.
#[cfg(target_os = "windows")]
fn win_volume() -> Option<(u8, bool)> {
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    };
    unsafe {
        // COM may already be initialized on this thread; ignore the result.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let endpoint: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
        let level = endpoint.GetMasterVolumeLevelScalar().ok()?; // 0.0..=1.0
        let muted = endpoint.GetMute().ok()?.as_bool();
        Some(((level * 100.0).round().clamp(0.0, 100.0) as u8, muted))
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

/// Parse `osascript -e "get volume settings"` output, e.g.
/// "output volume:45, input volume:75, alert volume:100, output muted:false".
/// The output volume is already a 0-100 integer. Returns (level_percent, muted),
/// or None if the "output volume" field is missing. Kept unconditional so it is
/// unit-tested on every platform.
pub fn parse_osascript(s: &str) -> Option<(u8, bool)> {
    let mut level: Option<u8> = None;
    let mut muted = false;
    for part in s.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("output volume:") {
            level = v.trim().parse::<u16>().ok().map(|n| n.min(100) as u8);
        } else if let Some(v) = part.strip_prefix("output muted:") {
            muted = v.trim().eq_ignore_ascii_case("true");
        }
    }
    level.map(|l| (l, muted))
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

    #[test]
    fn osascript_parses_unmuted() {
        let s = "output volume:45, input volume:75, alert volume:100, output muted:false";
        assert_eq!(parse_osascript(s), Some((45, false)));
    }

    #[test]
    fn osascript_parses_muted() {
        let s = "output volume:0, input volume:75, alert volume:100, output muted:true\n";
        assert_eq!(parse_osascript(s), Some((0, true)));
    }

    #[test]
    fn osascript_clamps_and_handles_full() {
        assert_eq!(
            parse_osascript("output volume:100, input volume:50, alert volume:100, output muted:false"),
            Some((100, false))
        );
        // Defensive: a value above 100 is clamped rather than wrapping the u8.
        assert_eq!(
            parse_osascript("output volume:150, output muted:false"),
            Some((100, false))
        );
    }

    #[test]
    fn osascript_handles_garbage() {
        assert_eq!(parse_osascript("not real output"), None);
        assert_eq!(parse_osascript(""), None);
    }
}
