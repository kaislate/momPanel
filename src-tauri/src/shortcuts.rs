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

/// Open a native settings screen for `target`. Best-effort, never destructive.
/// Unknown targets return `Err`.
#[tauri::command]
pub fn open_settings(target: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        match linux_cmd(&target) {
            Some((cmd, args)) => std::process::Command::new(cmd)
                .args(&args)
                .spawn()
                .map(|_| ())
                .map_err(|e| format!("failed to open settings for {}: {}", target, e)),
            None => Err(format!("unknown settings target: {}", target)),
        }
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

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
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
}
