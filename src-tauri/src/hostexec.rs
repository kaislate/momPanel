//! Spawning HOST programs from inside an AppImage.
//!
//! The AppImage runtime (AppRun) exports its mounted squashfs into the app's
//! environment — LD_LIBRARY_PATH, GDK/GTK/GST module paths, PATH, XDG_DATA_DIRS —
//! so the bundled app finds its bundled libraries. A child process like
//! `gnome-control-center` inherits all of it, resolves host libraries against the
//! AppImage's OLDER bundled copies, and dies instantly with a symbol lookup error
//! (verified on the target Zorin machine: libcurl-gnutls vs bundled libnghttp2).
//! `spawn()` still returns Ok because exec succeeded — the button press "does
//! nothing" with a clean log.
//!
//! Every spawn of a host binary must therefore go through [`host_command`], which
//! strips the AppImage-injected variables and scrubs the mount dirs out of the
//! list-valued ones. Harmless outside an AppImage (the variables simply aren't
//! there). The updater's relaunch chain can stack SEVERAL generations of mounts
//! into one process' env (seen live: three), so scrubbing filters every component,
//! not just $APPDIR.

/// Env vars AppRun points into the mounted AppImage; children must not see them.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // consumed by host_command (Linux-only)
const APPIMAGE_VARS: &[&str] = &[
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "APPDIR",
    "APPIMAGE",
    "OWD",
    "GSETTINGS_SCHEMA_DIR",
    "GDK_PIXBUF_MODULE_FILE",
    "GDK_PIXBUF_MODULEDIR",
    "GST_PLUGIN_SYSTEM_PATH",
    "GST_PLUGIN_SYSTEM_PATH_1_0",
    "GIO_EXTRA_MODULES",
    "GTK_PATH",
    "PYTHONPATH",
    "PERLLIB",
    "QT_PLUGIN_PATH",
];

/// Whether one `:`-list component points into an AppImage mount. AppImage mounts
/// live at /tmp/.mount_XXXXXX; `appdir` (the $APPDIR of the CURRENT mount) is also
/// matched in case a runtime ever mounts elsewhere.
fn is_appimage_component(component: &str, appdir: Option<&str>) -> bool {
    if component.starts_with("/tmp/.mount_") {
        return true;
    }
    match appdir {
        Some(dir) if !dir.is_empty() => component.starts_with(dir),
        _ => false,
    }
}

/// Drop AppImage-mount components from a `:`-separated list (PATH, XDG_DATA_DIRS).
/// Order and everything else are preserved.
pub fn scrub_path_list(value: &str, appdir: Option<&str>) -> String {
    value
        .split(':')
        .filter(|c| !c.is_empty() && !is_appimage_component(c, appdir))
        .collect::<Vec<_>>()
        .join(":")
}

/// A `Command` for `program` with the AppImage environment stripped: the module
/// variables removed, PATH and XDG_DATA_DIRS scrubbed, and the working directory
/// moved out of the (read-only, soon-to-vanish) mount to the user's home.
#[cfg(target_os = "linux")]
pub fn host_command(program: &str) -> std::process::Command {
    let appdir = std::env::var("APPDIR").ok();
    let mut cmd = std::process::Command::new(program);
    for var in APPIMAGE_VARS {
        cmd.env_remove(var);
    }
    for var in ["PATH", "XDG_DATA_DIRS"] {
        if let Ok(v) = std::env::var(var) {
            cmd.env(var, scrub_path_list(&v, appdir.as_deref()));
        }
    }
    if let Some(home) = dirs::home_dir() {
        cmd.current_dir(home);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_drops_mount_components_keeps_host_ones() {
        let path = "/tmp/.mount_momPanaFliko/usr/bin/:/usr/local/bin:/usr/bin:/tmp/.mount_momPanhfkGlg/bin:/bin";
        assert_eq!(scrub_path_list(path, None), "/usr/local/bin:/usr/bin:/bin");
    }

    #[test]
    fn scrub_handles_stacked_update_mounts() {
        // The updater's relaunch chain stacks several generations of mounts.
        let ld = "/tmp/.mount_momPanaFliko/usr/lib/:/tmp/.mount_momPanhfkGlg/usr/lib/:/tmp/.mount_momPanpFFoNC/usr/lib/";
        assert_eq!(scrub_path_list(ld, None), "");
    }

    #[test]
    fn scrub_respects_custom_appdir() {
        let path = "/opt/appimage-mount/usr/bin:/usr/bin";
        assert_eq!(
            scrub_path_list(path, Some("/opt/appimage-mount")),
            "/usr/bin"
        );
    }

    #[test]
    fn scrub_untouched_outside_appimage() {
        let path = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
        assert_eq!(scrub_path_list(path, None), path);
        // Empty appdir must not match everything.
        assert_eq!(scrub_path_list(path, Some("")), path);
    }
}
