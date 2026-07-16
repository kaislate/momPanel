//! Safe shortcuts backend. Opens a native settings screen for a known target.
//! Never performs anything destructive: it only spawns the OS settings UI and
//! returns. Unknown targets are rejected.

/// Map a logical settings target to a Linux command + args (GNOME).
/// Returns `None` for unknown targets so callers can reject them.
pub fn linux_cmd(t: &str) -> Option<(&'static str, Vec<&'static str>)> {
    match t {
        "wifi" => Some(("gnome-control-center", vec!["wifi"])),
        "printers" => Some(("gnome-control-center", vec!["printers"])),
        "sound" => Some(("gnome-control-center", vec!["sound"])),
        "storage" => Some(("gnome-disks", vec![])),
        _ => None,
    }
}

/// Map a logical settings target to the argument vector for macOS `open`.
/// Ventura+ (macOS 13) System Settings panes are addressed by extension URL. `storage`
/// has no dependable Settings pane across macOS versions, so we open Finder — a target
/// that always exists and so can't silently fail — instead of a pane that may not.
/// Returns `None` for unknown targets so callers can reject them. Kept unconditional
/// (like `linux_cmd`) so the allow-list is unit-tested on every platform.
pub fn macos_open_args(t: &str) -> Option<Vec<&'static str>> {
    match t {
        "wifi" => Some(vec!["x-apple.systempreferences:com.apple.wifi-settings-extension"]),
        "sound" => Some(vec!["x-apple.systempreferences:com.apple.Sound-Settings.extension"]),
        "printers" => {
            Some(vec!["x-apple.systempreferences:com.apple.Print-Scan-Settings.extension"])
        }
        "storage" => Some(vec!["-a", "Finder"]),
        _ => None,
    }
}

/// Append one line to <config>/momPanel/shortcuts.log — a field-diagnosable trace of
/// every settings-button press (the Linux "buttons do nothing" report is otherwise
/// invisible: failures are returned to the webview and silently swallowed there).
/// Best-effort; starts over past 20 KB so it can never grow unbounded.
fn trace(line: &str) {
    let mut p = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("momPanel");
    let _ = std::fs::create_dir_all(&p);
    p.push("shortcuts.log");
    let fresh = std::fs::metadata(&p).map(|m| m.len() > 20_000).unwrap_or(false);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = format!("{ts} {line}\n");
    let _ = if fresh {
        std::fs::write(&p, entry)
    } else {
        use std::io::Write;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
            .and_then(|mut f| f.write_all(entry.as_bytes()))
    };
}

/// Open a native settings screen for `target`. Best-effort, never destructive.
/// Unknown targets return `Err`.
#[tauri::command]
pub fn open_settings(target: String) -> Result<(), String> {
    trace(&format!("open_settings invoked: {target}"));
    #[cfg(target_os = "linux")]
    {
        // hostexec: gnome-control-center/gnome-disks are HOST binaries; spawned with
        // the raw AppImage env they die instantly on bundled-library symbol clashes
        // (spawn still returns Ok, so this log used to show success for dead buttons).
        let r = match linux_cmd(&target) {
            Some((cmd, args)) => crate::hostexec::host_command(cmd)
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("failed to open settings for {}: {}", target, e)),
            None => Err(format!("unknown settings target: {}", target)),
        };
        trace(&format!("open_settings result: {:?}", r));
        r
    }

    #[cfg(target_os = "windows")]
    {
        // Map to the closest ms-settings: page. Best-effort only.
        let page = match target.as_str() {
            "wifi" | "internet" => "network-wifi",
            "printers" => "printers",
            "sound" | "volume" => "sound",
            "storage" => "storagepolicies",
            _ => return Err(format!("unknown settings target: {}", target)),
        };
        let arg = format!("start ms-settings:{}", page);
        std::process::Command::new("cmd")
            .args(["/C", &arg])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("failed to open settings for {}: {}", target, e))
    }

    #[cfg(target_os = "macos")]
    {
        match macos_open_args(&target) {
            Some(args) => std::process::Command::new("open")
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("failed to open settings for {}: {}", target, e)),
            None => Err(format!("unknown settings target: {}", target)),
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(format!("settings shortcuts unsupported on this platform: {}", target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_cmd_known_targets() {
        assert_eq!(linux_cmd("wifi"), Some(("gnome-control-center", vec!["wifi"])));
        assert_eq!(
            linux_cmd("printers"),
            Some(("gnome-control-center", vec!["printers"]))
        );
        assert_eq!(linux_cmd("sound"), Some(("gnome-control-center", vec!["sound"])));
        assert_eq!(linux_cmd("storage"), Some(("gnome-disks", vec![])));
    }

    #[test]
    fn linux_cmd_unknown_target() {
        assert_eq!(linux_cmd("rm-rf"), None);
        assert_eq!(linux_cmd(""), None);
        assert_eq!(linux_cmd("weather"), None);
    }

    #[test]
    fn macos_open_args_known_targets() {
        assert_eq!(
            macos_open_args("wifi"),
            Some(vec!["x-apple.systempreferences:com.apple.wifi-settings-extension"])
        );
        assert_eq!(
            macos_open_args("sound"),
            Some(vec!["x-apple.systempreferences:com.apple.Sound-Settings.extension"])
        );
        assert_eq!(
            macos_open_args("printers"),
            Some(vec!["x-apple.systempreferences:com.apple.Print-Scan-Settings.extension"])
        );
        assert_eq!(macos_open_args("storage"), Some(vec!["-a", "Finder"]));
    }

    #[test]
    fn macos_open_args_unknown_target() {
        assert_eq!(macos_open_args("rm-rf"), None);
        assert_eq!(macos_open_args(""), None);
        assert_eq!(macos_open_args("weather"), None);
    }
}
