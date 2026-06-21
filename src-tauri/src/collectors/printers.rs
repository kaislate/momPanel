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
        match Command::new("lpstat").arg("-p").arg("-d").output() {
            Ok(out) => parse_lpstat(&String::from_utf8_lossy(&out.stdout)),
            Err(_) => PrintersData::Unavailable,
        }
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
