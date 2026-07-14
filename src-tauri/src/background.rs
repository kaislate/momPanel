//! Desktop wallpaper -> base64 `data:` URL.
//!
//! On Linux the main window is opaque (WebKitGTK transparency ghosts frames and eats
//! input — see lib.rs), so the "see-through" companion look is simulated by drawing the
//! real wallpaper inside the webview. The CSP forbids `file://` and the asset protocol
//! isn't configured, so the image is inlined as a base64 data URL the webview can render.
//!
//! Everything here is best-effort: any failure resolves to `None` and the frontend falls
//! back to a solid sky. The pure parsers/helpers are unit-tested on every platform; the
//! per-OS wallpaper lookup shells out and is reviewed by eye.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// Ceiling on the wallpaper we'll inline over IPC. A 4K JPEG is a few MB; 20 MB is a
/// generous cap that still guards against a pathological (e.g. huge PNG) file bloating
/// the IPC message and the webview's memory.
const MAX_WALLPAPER_BYTES: u64 = 20 * 1024 * 1024;

/// Last successful encode, memoized by (path, mtime). Re-reading a multi-MB file and
/// re-base64-ing it on every call would be wasteful, so we only redo the work when the
/// wallpaper path or its modification time changes.
struct CacheEntry {
    path: PathBuf,
    mtime: SystemTime,
    data_url: String,
}

fn cache() -> &'static Mutex<Option<CacheEntry>> {
    static CACHE: OnceLock<Mutex<Option<CacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

/// Resolve the current desktop wallpaper to a `data:<mime>;base64,...` URL, or `None`.
/// Synchronous (shells out + reads a file); lib.rs wraps this in spawn_blocking so the
/// IPC worker thread is never stalled.
pub fn resolve() -> Option<String> {
    let path = wallpaper_path()?;
    encode_path(&path)
}

/// Map a file extension to an image MIME the webview can render. Unknown -> None: we
/// won't guess a type, and the frontend simply falls back to a solid sky.
fn mime_from_ext(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        // Zorin ships several default wallpapers as SVG; data: SVG renders fine as a
        // CSS background under the img-src data: CSP.
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// Read `path` and encode it into a `data:` URL, honoring the size cap and the cache.
/// Any failure (missing/oversized/unreadable/unknown-type) -> None.
fn encode_path(path: &Path) -> Option<String> {
    let mime = mime_from_ext(path)?;
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_WALLPAPER_BYTES {
        return None;
    }
    let mtime = meta.modified().ok()?;

    // Fast path: same file, same mtime -> reuse the cached data URL.
    let mut guard = cache().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = guard.as_ref() {
        if entry.path == path && entry.mtime == mtime {
            return Some(entry.data_url.clone());
        }
    }

    let bytes = std::fs::read(path).ok()?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", mime, b64);
    *guard = Some(CacheEntry { path: path.to_path_buf(), mtime, data_url: data_url.clone() });
    Some(data_url)
}

// ---- Per-OS wallpaper path lookup --------------------------------------------------

/// GNOME: prefer the dark-scheme wallpaper (GNOME swaps the picture with the color
/// scheme), falling back to the light one. Returns None if neither resolves to a local
/// file:// URI.
#[cfg(target_os = "linux")]
fn wallpaper_path() -> Option<PathBuf> {
    for key in ["picture-uri-dark", "picture-uri"] {
        // The dark-scheme key can point at a file that doesn't exist (seen on the
        // target Zorin machine) — only accept a candidate that's actually there, so
        // a stale dark URI falls through to the light wallpaper.
        if let Some(p) = gsettings_uri_path(key) {
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Run `gsettings get org.gnome.desktop.background <key>` and parse its quoted file URI.
#[cfg(target_os = "linux")]
fn gsettings_uri_path(key: &str) -> Option<PathBuf> {
    use std::process::Command;
    let out = Command::new("gsettings")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .args(["get", "org.gnome.desktop.background", key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_gsettings_file_uri(&String::from_utf8_lossy(&out.stdout))
}

/// Windows: HKCU\Control Panel\Desktop value `Wallpaper` holds the current image path.
/// `reg query` is simpler than pulling in a registry crate and needs no extra deps.
#[cfg(target_os = "windows")]
fn wallpaper_path() -> Option<PathBuf> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000; // no console flash
    let out = Command::new("reg")
        .args(["query", "HKCU\\Control Panel\\Desktop", "/v", "Wallpaper"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_reg_wallpaper(&String::from_utf8_lossy(&out.stdout))
}

/// macOS: ask System Events for the current desktop picture (a plain POSIX path).
#[cfg(target_os = "macos")]
fn wallpaper_path() -> Option<PathBuf> {
    use std::process::Command;
    let out = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get picture of current desktop"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

/// Unsupported platforms have no wallpaper source.
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn wallpaper_path() -> Option<PathBuf> {
    None
}

// ---- Pure parsing helpers (unit-tested on every platform) --------------------------

/// Parse gsettings' quoted value (e.g. `'file:///home/mom/My%20Pics/bg.jpg'`) into a
/// filesystem path. Returns None if it isn't a local `file://` URI.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_gsettings_file_uri(raw: &str) -> Option<PathBuf> {
    let s = raw.trim();
    // gsettings wraps string values in single quotes (older builds used double quotes).
    let s = match s.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')) {
        Some(inner) => inner,
        None => s.strip_prefix('"').and_then(|x| x.strip_suffix('"')).unwrap_or(s),
    };
    let path = s.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode(path)))
}

/// Minimal percent-decoder for file URIs (handles `%20` etc. in wallpaper paths).
/// Malformed escapes are passed through literally.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Extract the wallpaper path from `reg query ... /v Wallpaper` output. The value line
/// is `    Wallpaper    REG_SZ    C:\path\to\file.jpg`.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn parse_reg_wallpaper(text: &str) -> Option<PathBuf> {
    for line in text.lines() {
        if let Some(idx) = line.find("REG_SZ") {
            let val = line[idx + "REG_SZ".len()..].trim();
            if !val.is_empty() {
                return Some(PathBuf::from(val));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mime_by_extension() {
        assert_eq!(mime_from_ext(Path::new("/a/b.jpg")), Some("image/jpeg"));
        assert_eq!(mime_from_ext(Path::new("/a/b.JPEG")), Some("image/jpeg"));
        assert_eq!(mime_from_ext(Path::new("/a/b.png")), Some("image/png"));
        assert_eq!(mime_from_ext(Path::new("/a/b.WebP")), Some("image/webp"));
        // Unknown / extensionless -> None (e.g. Windows' TranscodedWallpaper cache).
        assert_eq!(mime_from_ext(Path::new("/a/b.bmp")), None);
        assert_eq!(mime_from_ext(Path::new("/a/TranscodedWallpaper")), None);
    }

    #[test]
    fn gsettings_uri_stripped_and_decoded() {
        assert_eq!(
            parse_gsettings_file_uri("'file:///home/mom/bg.jpg'\n"),
            Some(PathBuf::from("/home/mom/bg.jpg"))
        );
        // Percent-encoded spaces are decoded.
        assert_eq!(
            parse_gsettings_file_uri("'file:///home/mom/My%20Pics/a%20b.png'"),
            Some(PathBuf::from("/home/mom/My Pics/a b.png"))
        );
        // A non-file value (e.g. gsettings returns a bare '' when unset) -> None.
        assert_eq!(parse_gsettings_file_uri("''"), None);
        assert_eq!(parse_gsettings_file_uri("'none'"), None);
    }

    #[test]
    fn percent_decode_passes_through_bad_escapes() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("100%done"), "100%done"); // %do is not valid hex
        assert_eq!(percent_decode("trailing%"), "trailing%");
    }

    #[test]
    fn reg_wallpaper_value_extracted() {
        let sample = "\r\nHKEY_CURRENT_USER\\Control Panel\\Desktop\r\n    \
                      Wallpaper    REG_SZ    C:\\Users\\mom\\Pictures\\bg.jpg\r\n";
        assert_eq!(
            parse_reg_wallpaper(sample),
            Some(PathBuf::from("C:\\Users\\mom\\Pictures\\bg.jpg"))
        );
        // No Wallpaper value present -> None.
        assert_eq!(parse_reg_wallpaper("HKEY_CURRENT_USER\\Control Panel\\Desktop\r\n"), None);
    }
}
