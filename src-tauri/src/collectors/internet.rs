//! Internet connectivity collector.
//!
//! `read()` returns `InternetData::Ok { online }` where `online` reflects whether
//! the machine appears to have working internet access. Note that *offline* is a
//! perfectly valid state, so failures here resolve to `Ok { online: false }` and
//! NOT `Unavailable` (there is no `Unavailable` path in practice).
//!
//! Strategy:
//!   * On Linux, first ask NetworkManager via `nmcli networking connectivity`.
//!     A result of `"full"` means real connectivity.
//!   * On all platforms, fall back to a short TCP connect to a well-known public
//!     resolver (1.1.1.1:53). Reaching it within the timeout implies online.

use serde::Serialize;
use std::net::{SocketAddr, TcpStream};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long a TCP-probe verdict is reused before we re-probe.
const PROBE_TTL: Duration = Duration::from_secs(20);

/// Last probe: (when it ran, whether it succeeded). The probe can block up to 1.5s and
/// the tile polls every few seconds, so caching keeps most ticks off the network.
static PROBE_CACHE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum InternetData {
    Ok { online: bool },
    Unavailable,
}

/// Classify the trimmed output of `nmcli networking connectivity`.
/// Only `"full"` counts as truly online; everything else (none, limited,
/// portal, unknown, ...) is treated as offline.
pub fn classify_connectivity(out: &str) -> bool {
    out.trim().eq_ignore_ascii_case("full")
}

/// Probe internet via a short-lived TCP connection to a well-known public host,
/// reusing a recent verdict within `PROBE_TTL`.
fn tcp_probe() -> bool {
    if let Some((at, ok)) = *PROBE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) {
        if at.elapsed() < PROBE_TTL {
            return ok;
        }
    }
    // Try 443 first — some networks block outbound port 53 to Cloudflare — and fall back
    // to 53 as a second chance before declaring offline.
    let ok = connect_ok(443) || connect_ok(53);
    *PROBE_CACHE.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), ok));
    ok
}

/// One short-lived TCP connect attempt to 1.1.1.1 on the given port.
fn connect_ok(port: u16) -> bool {
    let addr: SocketAddr = ([1, 1, 1, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(1500)).is_ok()
}

#[cfg(target_os = "linux")]
fn nmcli_connectivity() -> Option<bool> {
    use std::process::Command;
    // Pin the locale: nmcli localizes the connectivity word ("full", etc.) that
    // classify_connectivity matches literally.
    let output = Command::new("nmcli")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .args(["networking", "connectivity"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(classify_connectivity(&text))
}

#[cfg(target_os = "linux")]
pub fn read() -> InternetData {
    // Prefer NetworkManager's verdict when it reports a definitive "full".
    if let Some(true) = nmcli_connectivity() {
        return InternetData::Ok { online: true };
    }
    // Otherwise (nmcli missing, errored, or non-full) confirm with a TCP probe.
    InternetData::Ok { online: tcp_probe() }
}

#[cfg(not(target_os = "linux"))]
pub fn read() -> InternetData {
    InternetData::Ok { online: tcp_probe() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_is_online() {
        assert!(classify_connectivity("full"));
        assert!(classify_connectivity("full\n"));
        assert!(classify_connectivity("  full  "));
        assert!(classify_connectivity("FULL"));
    }

    #[test]
    fn non_full_is_offline() {
        assert!(!classify_connectivity("none"));
        assert!(!classify_connectivity("limited"));
        assert!(!classify_connectivity("portal"));
        assert!(!classify_connectivity("unknown"));
        assert!(!classify_connectivity(""));
        assert!(!classify_connectivity("fullish"));
    }

    #[test]
    fn serializes_to_envelope() {
        let s = serde_json::to_string(&InternetData::Ok { online: true }).unwrap();
        assert_eq!(s, r#"{"state":"ok","online":true}"#);
        let s = serde_json::to_string(&InternetData::Ok { online: false }).unwrap();
        assert_eq!(s, r#"{"state":"ok","online":false}"#);
        let s = serde_json::to_string(&InternetData::Unavailable).unwrap();
        assert_eq!(s, r#"{"state":"unavailable"}"#);
    }
}
