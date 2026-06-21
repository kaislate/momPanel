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

/// Split one line of `nmcli -t` terse output into fields. nmcli escapes the field
/// separator inside values as `\:` and a literal backslash as `\\`, so a naive
/// `split(':')` corrupts any SSID containing a colon. This splits on UNescaped
/// colons and unescapes each field.
fn split_terse(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Emit the escaped char literally (\: -> :, \\ -> \).
                if let Some(n) = chars.next() {
                    cur.push(n);
                } else {
                    cur.push('\\');
                }
            }
            ':' => fields.push(std::mem::take(&mut cur)),
            _ => cur.push(c),
        }
    }
    fields.push(cur);
    fields
}

/// Parse `nmcli -t -f ACTIVE,SSID,SIGNAL dev wifi` output. Returns the row whose
/// ACTIVE field is "yes" as `(ssid, signal_percent)`, or `None` when no active row
/// / parse fails. SSIDs containing colons are handled via `split_terse`.
fn parse_nmcli(text: &str) -> Option<(String, u8)> {
    for line in text.lines() {
        let fields = split_terse(line);
        if fields.len() < 3 {
            continue;
        }
        if fields[0] != "yes" {
            continue;
        }
        // ACTIVE = first, SIGNAL = last, SSID = everything between (defensively
        // re-joined with ':' should an SSID ever yield extra fields).
        let last = fields.len() - 1;
        let ssid = fields[1..last].join(":");
        if ssid.is_empty() {
            continue;
        }
        let signal_percent: u8 = fields[last].trim().parse().ok()?;
        return Some((ssid, signal_percent.min(100)));
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

    #[test]
    fn handles_ssid_with_escaped_colon() {
        // nmcli terse escapes ':' in an SSID as '\:'
        let input = "no:Other:30\nyes:Cafe\\:Guest:64\n";
        assert_eq!(parse_nmcli(input), Some(("Cafe:Guest".to_string(), 64)));
    }

    #[test]
    fn handles_ssid_with_escaped_backslash() {
        let input = "yes:Home\\\\Net:50\n";
        assert_eq!(parse_nmcli(input), Some(("Home\\Net".to_string(), 50)));
    }
}
