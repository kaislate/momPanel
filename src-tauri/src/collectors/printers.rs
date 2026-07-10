//! Printers collector. Uses `lpstat -p -d` (CUPS) on Linux and PowerShell/CIM
//! (`Win32_Printer`) on Windows. Any failure -> Unavailable.
//!
//! Serialized shape:
//!   { "state": "ok", "printers": [{ "name": ..., "status": ... }],
//!     "default_name": "..." | null }
//! or { "state": "unavailable" }.

use serde::Serialize;

#[derive(Serialize)]
pub struct PrinterInfo {
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum PrintersData {
    Ok {
        printers: Vec<PrinterInfo>,
        default_name: Option<String>,
    },
    Unavailable,
}

/// Dispatch: Linux uses `lpstat` (CUPS), Windows uses PowerShell/CIM, else Unavailable.
pub fn read() -> PrintersData {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Pin the locale: lpstat's status text is gettext-translated, and parse_lpstat
        // keys off the English words ("printer", "idle", "disabled", ...).
        let mut data = match Command::new("lpstat")
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .arg("-p")
            .arg("-d")
            .output()
        {
            Ok(out) => parse_lpstat(&String::from_utf8_lossy(&out.stdout)),
            Err(_) => return PrintersData::Unavailable,
        };
        // A *permanent* CUPS queue (e.g. a driverless ipp:// queue backed by ipp-usb)
        // always reports "idle" -> "ready" even when the physical printer is powered
        // off. For network-backed queues, actively probe the device endpoint: if it's
        // unreachable, downgrade the reported "ready" to "offline". (ipp-usb only
        // listens on its port while the printer is on, so this is a reliable tell.)
        if let PrintersData::Ok { printers, .. } = &mut data {
            let devices = device_uris();
            for p in printers.iter_mut() {
                if p.status != "ready" {
                    continue; // never override out_of_paper / already-offline
                }
                if let Some((host, port)) =
                    devices.get(&p.name).and_then(|uri| network_endpoint(uri))
                {
                    if !endpoint_reachable(&host, port) {
                        p.status = "offline".to_string();
                    }
                }
            }
        }
        data
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000; // no console flash
        // One line per printer: "Name|WorkOffline|Default"
        let script = "Get-CimInstance Win32_Printer | ForEach-Object { \
                      \"$($_.Name)|$($_.WorkOffline)|$($_.Default)\" }";
        match Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            Ok(out) if out.status.success() => {
                parse_win_printers(&String::from_utf8_lossy(&out.stdout))
            }
            _ => PrintersData::Unavailable,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        PrintersData::Unavailable
    }
}

/// Map of printer name -> CUPS device URI, parsed from `lpstat -v`
/// (lines like "device for EPSON_ET3760: ipp://localhost:60000/ipp/print").
/// Returns an empty map on any failure.
#[cfg(target_os = "linux")]
fn device_uris() -> std::collections::HashMap<String, String> {
    use std::process::Command;
    let mut map = std::collections::HashMap::new();
    if let Ok(out) = Command::new("lpstat")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .arg("-v")
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(rest) = line.trim().strip_prefix("device for ") {
                if let Some((name, uri)) = rest.split_once(':') {
                    map.insert(name.trim().to_string(), uri.trim().to_string());
                }
            }
        }
    }
    map
}

/// Extract (host, port) from a *network* device URI (ipp/ipps/http/https/socket).
/// Returns None for local backends (usb://, hp:/, file://, dnssd://, ...) that
/// can't be TCP-probed. Pure/host-agnostic so it is unit-testable everywhere.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn network_endpoint(uri: &str) -> Option<(String, u16)> {
    let (scheme, rest) = uri.split_once("://")?;
    let default_port: u16 = match scheme {
        "ipp" | "http" => 631, // ipp-usb uses an explicit port anyway; 631 is the IPP default
        "ipps" | "https" => 443,
        "socket" => 9100,
        _ => return None, // usb, hp, file, dnssd, etc. -> not TCP-probeable
    };
    let authority = rest.split('/').next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority); // drop any user@ prefix
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().unwrap_or(default_port)),
        None => (authority, default_port),
    };
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port))
}

/// True if a TCP connection to host:port succeeds quickly. ipp-usb only listens on
/// its port while the printer is powered on, so this doubles as a presence check.
#[cfg(target_os = "linux")]
fn endpoint_reachable(host: &str, port: u16) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            addrs.any(|a| TcpStream::connect_timeout(&a, Duration::from_millis(1200)).is_ok())
        }
        Err(_) => false,
    }
}

/// Windows' built-in virtual "printers" that a normal user doesn't care about.
#[allow(dead_code)]
fn is_virtual_printer(name: &str) -> bool {
    let n = name.to_lowercase();
    n == "microsoft print to pdf"
        || n == "microsoft xps document writer"
        || n == "fax"
        || n.contains("onenote")
}

/// Parse the pipe-delimited "Name|WorkOffline|Default" lines from Win32_Printer,
/// skipping virtual printers. Offline -> "offline", otherwise "ready". Empty is still Ok.
#[allow(dead_code)]
fn parse_win_printers(text: &str) -> PrintersData {
    let mut printers = Vec::new();
    let mut default_name = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('|');
        let name = parts.next().unwrap_or("").trim().to_string();
        if name.is_empty() || is_virtual_printer(&name) {
            continue;
        }
        let offline = parts.next().unwrap_or("").trim().eq_ignore_ascii_case("true");
        let is_default = parts.next().unwrap_or("").trim().eq_ignore_ascii_case("true");
        if is_default {
            default_name = Some(name.clone());
        }
        printers.push(PrinterInfo {
            name,
            status: if offline { "offline" } else { "ready" }.to_string(),
        });
    }
    PrintersData::Ok {
        printers,
        default_name,
    }
}

/// Parse `lpstat -p -d` output into an Ok variant. An empty printer list is still
/// Ok (the frontend shows a friendly "No printers"). This never returns Unavailable
/// so it is easy to unit-test; `read()` is responsible for the missing-tool case.
pub fn parse_lpstat(text: &str) -> PrintersData {
    let mut printers: Vec<PrinterInfo> = Vec::new();
    let mut default_name: Option<String> = None;

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("printer ") {
            // rest looks like: "<name> is idle.  enabled since ..."
            let mut parts = rest.splitn(2, ' ');
            let name = parts.next().unwrap_or("").trim().to_string();
            if name.is_empty() {
                continue;
            }
            let tail = parts.next().unwrap_or("").to_lowercase();
            // CUPS often appends a reason after a state, e.g.
            // "disabled since ... - out of paper" or "... media-empty".
            let status = if tail.contains("paper") || tail.contains("media-empty") {
                "out_of_paper"
            } else if tail.contains("disabled") || tail.contains("offline") {
                "offline"
            } else if tail.contains("idle") || tail.contains("processing") {
                "ready"
            } else {
                "unknown"
            };
            printers.push(PrinterInfo {
                name,
                status: status.to_string(),
            });
        } else if let Some(rest) = line.strip_prefix("system default destination:") {
            let name = rest.trim();
            if !name.is_empty() {
                default_name = Some(name.to_string());
            }
        }
    }

    PrintersData::Ok {
        printers,
        default_name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_parts(d: &PrintersData) -> (&Vec<PrinterInfo>, &Option<String>) {
        match d {
            PrintersData::Ok {
                printers,
                default_name,
            } => (printers, default_name),
            PrintersData::Unavailable => panic!("expected Ok"),
        }
    }

    #[test]
    fn parses_two_printers_and_default() {
        let sample = "printer Office_LaserJet is idle.  enabled since Mon\n\
printer Photo is processing.  enabled since Tue\n\
system default destination: Office_LaserJet\n";
        let data = parse_lpstat(sample);
        let (printers, default_name) = ok_parts(&data);
        assert_eq!(printers.len(), 2);
        assert_eq!(printers[0].name, "Office_LaserJet");
        assert_eq!(printers[0].status, "ready");
        assert_eq!(printers[1].name, "Photo");
        assert_eq!(printers[1].status, "ready");
        assert_eq!(default_name.as_deref(), Some("Office_LaserJet"));
    }

    #[test]
    fn disabled_printer_is_offline() {
        let sample = "printer Old is disabled since Mon - out of toner\n";
        let data = parse_lpstat(sample);
        let (printers, _) = ok_parts(&data);
        assert_eq!(printers[0].status, "offline");
    }

    #[test]
    fn out_of_paper_detected() {
        let sample = "printer Photo is disabled since Mon - out of paper\n";
        let data = parse_lpstat(sample);
        let (printers, _) = ok_parts(&data);
        assert_eq!(printers[0].status, "out_of_paper");
    }

    #[test]
    fn empty_output_is_ok_with_no_printers() {
        let data = parse_lpstat("");
        let (printers, default_name) = ok_parts(&data);
        assert!(printers.is_empty());
        assert!(default_name.is_none());
    }

    #[test]
    fn network_endpoint_parses_probeable_uris_and_skips_local() {
        // ipp-usb bridge (the real case): explicit port must be honored.
        assert_eq!(
            network_endpoint("ipp://localhost:60000/ipp/print"),
            Some(("localhost".to_string(), 60000))
        );
        // scheme defaults when no port is given.
        assert_eq!(network_endpoint("ipp://host/ipp"), Some(("host".to_string(), 631)));
        assert_eq!(
            network_endpoint("ipps://printer.local/ipp/print"),
            Some(("printer.local".to_string(), 443))
        );
        assert_eq!(network_endpoint("socket://10.0.0.5"), Some(("10.0.0.5".to_string(), 9100)));
        // Local backends can't be TCP-probed -> None (status left as lpstat reported).
        assert_eq!(network_endpoint("usb://EPSON/ET-3760%20Series?serial=X"), None);
        assert_eq!(network_endpoint("hp:/usb/HP?serial=Y"), None);
        assert_eq!(network_endpoint("implicitclass://EPSON_ET_3760_Series_USB/"), None);
    }

    #[test]
    fn parses_windows_printers_skips_virtual_and_marks_default() {
        let sample = "Microsoft Print to PDF|False|False\n\
                      EPSON ET-3760 Series|True|True\n\
                      Office LaserJet|False|False\n";
        let data = super::parse_win_printers(sample);
        let (printers, default_name) = ok_parts(&data);
        assert_eq!(printers.len(), 2); // virtual "Print to PDF" filtered out
        assert_eq!(printers[0].name, "EPSON ET-3760 Series");
        assert_eq!(printers[0].status, "offline"); // WorkOffline = True
        assert_eq!(printers[1].name, "Office LaserJet");
        assert_eq!(printers[1].status, "ready");
        assert_eq!(default_name.as_deref(), Some("EPSON ET-3760 Series"));
    }
}
