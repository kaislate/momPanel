//! Printers collector. Uses `lpstat -p -d` (CUPS) on Linux and macOS — both ship the
//! same CUPS tooling and output format — and PowerShell/CIM (`Win32_Printer`) on
//! Windows. Any failure -> Unavailable.
//!
//! Serialized shape:
//!   { "state": "ok", "printers": [{ "name": ..., "status": ... }],
//!     "default_name": "..." | null,
//!     "inks": [{ "name": ..., "color": "#RRGGBB", "percent": 49, "low": false }] }
//! or { "state": "unavailable" }.
//!
//! `inks` (Linux/macOS, via `ipptool`) is omitted entirely when unavailable; the
//! frontend treats its absence as "no ink data".

use serde::Serialize;

#[derive(Serialize)]
pub struct PrinterInfo {
    pub name: String,
    pub status: String,
}

/// One ink/toner marker: display name, swatch color, level %, and whether it's low.
#[derive(Serialize, PartialEq, Debug)]
pub struct Ink {
    pub name: String,
    pub color: String,
    pub percent: i32,
    pub low: bool,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum PrintersData {
    Ok {
        printers: Vec<PrinterInfo>,
        default_name: Option<String>,
        /// Ink levels for the default/first printer; omitted when there's no data.
        #[serde(skip_serializing_if = "Option::is_none")]
        inks: Option<Vec<Ink>>,
    },
    Unavailable,
}

/// Dispatch: Linux/macOS use `lpstat` (CUPS), Windows uses PowerShell/CIM,
/// else Unavailable.
pub fn read() -> PrintersData {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Pin the locale: lpstat's status text is gettext-translated, and parse_lpstat
        // keys off the English words ("printer", "idle", "disabled", ...).
        let mut data = match crate::hostexec::host_command("lpstat")
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
        // Ink levels: one extra `ipptool` call per poll (30s), and only when there are
        // printers — we query the single default queue (or the first if none is default)
        // rather than every printer. ipptool absence / any failure -> inks stays None.
        if let PrintersData::Ok { printers, default_name, inks } = &mut data {
            if !printers.is_empty() {
                let queue =
                    default_name.clone().unwrap_or_else(|| printers[0].name.clone());
                *inks = query_inks(&queue);
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

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        PrintersData::Unavailable
    }
}

/// Map of printer name -> CUPS device URI, parsed from `lpstat -v`
/// (lines like "device for EPSON_ET3760: ipp://localhost:60000/ipp/print").
/// Returns an empty map on any failure.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn device_uris() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Ok(out) = crate::hostexec::host_command("lpstat")
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
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
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
#[cfg(any(target_os = "linux", target_os = "macos"))]
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
        inks: None, // Windows has no ipptool marker query
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
        inks: None, // populated by read() on Linux/macOS via ipptool
    }
}

/// Query one CUPS queue for ink/toner marker attributes via `ipptool`, returning parsed
/// inks or None. The queue name comes straight from lpstat (CUPS queue names have no
/// spaces, so no percent-encoding is needed). Missing `ipptool`, a spawn error, or an
/// unparseable response all resolve to None so the frontend shows "no data".
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn query_inks(queue: &str) -> Option<Vec<Ink>> {
    // LC_ALL=C pins ipptool's output; get-printer-attributes.test is the stock test file
    // that ships with CUPS and is resolved by name. -t = plain text, -v = show values.
    let out = crate::hostexec::host_command("ipptool")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .arg("-tv")
        .arg(format!("ipp://localhost/printers/{}", queue))
        .arg("get-printer-attributes.test")
        .output()
        .ok()?; // ipptool not installed / not on PATH -> no ink data
    // ipptool may exit non-zero (test "fails") while still printing the attributes we
    // want on stdout, so parse the output regardless of the exit status.
    parse_marker_attrs(&String::from_utf8_lossy(&out.stdout))
}

/// Parse CUPS `marker-*` attributes (as printed by `ipptool -tv`) into an ink list.
/// Returns None if there are no usable ink levels. Pure/host-agnostic -> unit-testable
/// on every platform. Only ink/toner markers are kept (waste tanks etc. are dropped),
/// and entries with an unknown level (outside 0..=100) are skipped.
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
pub fn parse_marker_attrs(text: &str) -> Option<Vec<Ink>> {
    let mut colors: Option<Vec<String>> = None;
    let mut levels: Option<Vec<i32>> = None;
    let mut low_levels: Option<Vec<i32>> = None;
    let mut names: Option<Vec<String>> = None;
    let mut types: Option<Vec<String>> = None;

    for raw in text.lines() {
        // Lines look like: `marker-levels (1setOf integer) = 49,77,80,80` (indented).
        let (key, value) = match split_attr(raw.trim()) {
            Some(kv) => kv,
            None => continue,
        };
        match key {
            "marker-colors" => colors = Some(split_csv_strings(value)),
            "marker-levels" => levels = Some(split_csv_ints(value)),
            "marker-low-levels" => low_levels = Some(split_csv_ints(value)),
            "marker-names" => names = Some(split_csv_strings(value)),
            "marker-types" => types = Some(split_csv_strings(value)),
            _ => {}
        }
    }

    // marker-levels is the essential column; without it there's nothing to show.
    let levels = levels?;
    if levels.is_empty() {
        return None;
    }
    let names = names.unwrap_or_default();
    let colors = colors.unwrap_or_default();
    let low_levels = low_levels.unwrap_or_default();
    let types = types.unwrap_or_default();

    let mut inks = Vec::new();
    for (i, &percent) in levels.iter().enumerate() {
        // Keep only real ink/toner markers; skip waste tanks and other marker types.
        // Missing marker-types -> assume "ink" (some drivers omit it).
        let mtype = types.get(i).map(String::as_str).unwrap_or("ink");
        if !matches!(mtype, "ink" | "ink-cartridge" | "toner") {
            continue;
        }
        // -1 (or anything outside 0..=100) means the printer reported no real level.
        if !(0..=100).contains(&percent) {
            continue;
        }
        // Default low threshold is 15 when the printer omits marker-low-levels (or
        // reports a nonsense negative value for this slot).
        let low_level = match low_levels.get(i).copied() {
            Some(v) if v >= 0 => v,
            _ => 15,
        };
        let name =
            names.get(i).cloned().unwrap_or_else(|| format!("Ink {}", i + 1));
        let color = colors.get(i).cloned().unwrap_or_default();
        inks.push(Ink { name, color, percent, low: percent <= low_level });
    }

    if inks.is_empty() {
        None
    } else {
        Some(inks)
    }
}

/// Split an ipptool attribute line into `(key, value)` at the ` = ` separator. The key
/// is the first whitespace token (e.g. `marker-levels` from `marker-levels (1setOf ...)`).
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
fn split_attr(line: &str) -> Option<(&str, &str)> {
    let eq = line.find(" = ")?;
    let key = line[..eq].split_whitespace().next()?;
    Some((key, line[eq + 3..].trim()))
}

/// Split a comma-separated value list into trimmed strings.
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
fn split_csv_strings(v: &str) -> Vec<String> {
    v.split(',').map(|s| s.trim().to_string()).collect()
}

/// Split a comma-separated integer list; unparseable entries become -1 ("unknown").
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
fn split_csv_ints(v: &str) -> Vec<i32> {
    v.split(',').map(|s| s.trim().parse::<i32>().unwrap_or(-1)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_parts(d: &PrintersData) -> (&Vec<PrinterInfo>, &Option<String>) {
        match d {
            PrintersData::Ok {
                printers,
                default_name,
                ..
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

    // Exact ipptool -tv output from the target Epson (verified real fixture).
    const INK_FIXTURE: &str = "\
        Get printer attributes using get-printer-attributes         [PASS]\n\
            marker-colors (1setOf nameWithoutLanguage) = #000000,#00FFFF,#FF00FF,#FFFF00\n\
            marker-levels (1setOf integer) = 49,77,80,80\n\
            marker-low-levels (1setOf integer) = 15,15,15,15\n\
            marker-names (1setOf nameWithoutLanguage) = Black ink,Cyan ink,Magenta ink,Yellow ink\n\
            marker-types (1setOf keyword) = ink,ink,ink,ink\n";

    #[test]
    fn parses_marker_attrs_from_real_fixture() {
        let inks = super::parse_marker_attrs(INK_FIXTURE).expect("some inks");
        assert_eq!(inks.len(), 4);
        assert_eq!(
            inks[0],
            Ink { name: "Black ink".into(), color: "#000000".into(), percent: 49, low: false }
        );
        assert_eq!(
            inks[3],
            Ink { name: "Yellow ink".into(), color: "#FFFF00".into(), percent: 80, low: false }
        );
        // 49 > low-level 15 -> not low; construct a fixture where one dips to the floor.
        assert!(inks.iter().all(|i| !i.low));
    }

    #[test]
    fn marker_level_at_or_below_low_threshold_is_low() {
        let fixture = "\
            marker-colors (1setOf nameWithoutLanguage) = #000000,#00FFFF\n\
            marker-levels (1setOf integer) = 15,8\n\
            marker-low-levels (1setOf integer) = 15,15\n\
            marker-names (1setOf nameWithoutLanguage) = Black ink,Cyan ink\n\
            marker-types (1setOf keyword) = ink,ink\n";
        let inks = super::parse_marker_attrs(fixture).expect("some inks");
        assert_eq!(inks.len(), 2);
        assert!(inks[0].low, "15 <= 15 should be low");
        assert!(inks[1].low, "8 <= 15 should be low");
    }

    #[test]
    fn unknown_levels_are_skipped() {
        // -1 and 200 are unknown/out-of-range: those entries are dropped, keeping only
        // the two real levels.
        let fixture = "\
            marker-colors (1setOf nameWithoutLanguage) = #000000,#00FFFF,#FF00FF,#FFFF00\n\
            marker-levels (1setOf integer) = -1,77,200,80\n\
            marker-low-levels (1setOf integer) = 15,15,15,15\n\
            marker-names (1setOf nameWithoutLanguage) = Black ink,Cyan ink,Magenta ink,Yellow ink\n\
            marker-types (1setOf keyword) = ink,ink,ink,ink\n";
        let inks = super::parse_marker_attrs(fixture).expect("some inks");
        assert_eq!(inks.len(), 2);
        assert_eq!(inks[0].name, "Cyan ink");
        assert_eq!(inks[0].percent, 77);
        assert_eq!(inks[1].name, "Yellow ink");
    }

    #[test]
    fn missing_low_levels_line_defaults_to_15() {
        // No marker-low-levels line at all: default threshold 15 -> 10 is low, 60 isn't.
        let fixture = "\
            marker-colors (1setOf nameWithoutLanguage) = #000000,#00FFFF\n\
            marker-levels (1setOf integer) = 10,60\n\
            marker-names (1setOf nameWithoutLanguage) = Black ink,Cyan ink\n\
            marker-types (1setOf keyword) = ink,ink\n";
        let inks = super::parse_marker_attrs(fixture).expect("some inks");
        assert_eq!(inks.len(), 2);
        assert!(inks[0].low, "10 <= default 15");
        assert!(!inks[1].low, "60 > default 15");
    }

    #[test]
    fn non_ink_markers_are_dropped() {
        // A waste tank (type "wasteToner") must not appear; toner is kept.
        let fixture = "\
            marker-colors (1setOf nameWithoutLanguage) = #000000,#888888\n\
            marker-levels (1setOf integer) = 42,90\n\
            marker-low-levels (1setOf integer) = 10,10\n\
            marker-names (1setOf nameWithoutLanguage) = Black Toner,Waste Tank\n\
            marker-types (1setOf keyword) = toner,wasteToner\n";
        let inks = super::parse_marker_attrs(fixture).expect("some inks");
        assert_eq!(inks.len(), 1);
        assert_eq!(inks[0].name, "Black Toner");
    }

    #[test]
    fn no_marker_levels_yields_none() {
        assert!(super::parse_marker_attrs("some unrelated ipptool output\n").is_none());
    }
}
