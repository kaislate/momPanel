//! Wi-Fi collector. Reads the active Wi-Fi connection via `nmcli` (Linux) or
//! `netsh wlan show interfaces` (Windows). Any failure / no Wi-Fi -> `Unavailable`.

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
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000; // no console flash
        let output = Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                match parse_netsh(&text) {
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

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        WifiData::Unavailable
    }
}

/// Parse `netsh wlan show interfaces` output (English labels). Returns the connected
/// SSID + signal percent, or None if not connected / no wireless interface. Guards
/// against matching the BSSID line for the SSID.
#[allow(dead_code)]
fn parse_netsh(text: &str) -> Option<(String, u8)> {
    let mut ssid: Option<String> = None;
    let mut signal: Option<u8> = None;
    for line in text.lines() {
        let mut parts = line.splitn(2, ':');
        let key = parts.next().unwrap_or("").trim();
        let val = parts.next().unwrap_or("").trim();
        if key.eq_ignore_ascii_case("SSID") {
            if !val.is_empty() {
                ssid = Some(val.to_string());
            }
        } else if key.eq_ignore_ascii_case("Signal") {
            let digits: String = val.chars().take_while(|c| c.is_ascii_digit()).collect();
            signal = digits.parse().ok();
        }
    }
    match (ssid, signal) {
        (Some(s), Some(sig)) if !s.is_empty() => Some((s, sig.min(100))),
        _ => None,
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

    #[test]
    fn parse_netsh_connected() {
        let input = "\
    Name                   : Wi-Fi
    State                  : connected
    SSID                   : HomeWifi
    BSSID                  : 11:22:33:44:55:66
    Signal                 : 84%
    Profile                : HomeWifi
";
        // BSSID must not be mistaken for SSID.
        assert_eq!(super::parse_netsh(input), Some(("HomeWifi".to_string(), 84)));
    }

    #[test]
    fn parse_netsh_disconnected_or_none() {
        assert_eq!(super::parse_netsh("There is no wireless interface on the system."), None);
        assert_eq!(super::parse_netsh("    State : disconnected\n    Signal : 0%\n"), None);
    }
}
