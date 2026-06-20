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
use std::time::Duration;

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

/// Probe internet via a short-lived TCP connection to a public DNS resolver.
fn tcp_probe() -> bool {
    let addr: SocketAddr = ([1, 1, 1, 1], 53).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(1500)).is_ok()
}

#[cfg(target_os = "linux")]
fn nmcli_connectivity() -> Option<bool> {
    use std::process::Command;
    let output = Command::new("nmcli")
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
