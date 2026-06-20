//! Wi-Fi collector. On Linux reads the active Wi-Fi connection via `nmcli`.
//! On any other platform, or on any failure, returns `Unavailable`.

use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum WifiData {
    Ok { ssid: String, signal_percent: u8 },
    Unavailable,
}

/// Pure dispatch: Linux runs the real tool, everything else is Unavailable.
pub fn read() -> WifiData {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID,SIGNAL", "dev", "wifi"])
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                match parse_nmcli(&text) {
                    Some((ssid, signal_percent)) => WifiData::Ok {
                        ssid,
                        signal_percent,
                    },
                    None => WifiData::Unavailable,
                }
            }
            _ => WifiData::Unavailable,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        WifiData::Unavailable
    }
}

/// Parse `nmcli -t -f ACTIVE,SSID,SIGNAL dev wifi` output. Each line is colon
/// separated: `ACTIVE:SSID:SIGNAL`. Returns the row whose first field is "yes"
/// as `(ssid, signal_percent)`, or `None` when no active row / parse fails.
fn parse_nmcli(text: &str) -> Option<(String, u8)> {
    for line in text.lines() {
        // ACTIVE:SSID:SIGNAL — split into at most 3 parts so an SSID with no
        // colon is preserved; nmcli escapes colons inside fields as "\:".
        let mut parts = line.splitn(3, ':');
        let active = parts.next()?;
        if active != "yes" {
            continue;
        }
        let ssid = parts.next()?;
        let signal = parts.next()?;
        if ssid.is_empty() {
            continue;
        }
        let signal_percent: u8 = signal.trim().parse().ok()?;
        return Some((ssid.to_string(), signal_percent.min(100)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_active_row() {
        let input = "no:Neighbor:40\nyes:HomeWifi:78\nno:Cafe:22\n";
        assert_eq!(parse_nmcli(input), Some(("HomeWifi".to_string(), 78)));
    }

    #[test]
    fn none_when_no_active_row() {
        let input = "no:Neighbor:40\nno:Cafe:22\n";
        assert_eq!(parse_nmcli(input), None);
    }

    #[test]
    fn none_when_empty() {
        assert_eq!(parse_nmcli(""), None);
    }

    #[test]
    fn skips_active_with_empty_ssid() {
        let input = "yes::55\nno:Cafe:22\n";
        assert_eq!(parse_nmcli(input), None);
    }

    #[test]
    fn clamps_signal_over_100() {
        let input = "yes:HomeWifi:150\n";
        assert_eq!(parse_nmcli(input), Some(("HomeWifi".to_string(), 100)));
    }

    #[test]
    fn none_when_signal_unparseable() {
        let input = "yes:HomeWifi:abc\n";
        assert_eq!(parse_nmcli(input), None);
    }
}
