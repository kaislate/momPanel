//! Tiny append-only trace files in <config>/momPanel/, for field-diagnosing the
//! Linux reports that are otherwise invisible (failures returned to the webview get
//! swallowed there). Best-effort; each file starts over past 20 KB so it can never
//! grow unbounded. Read them over SSH: ~/.config/momPanel/<name>.log
use std::io::Write;

pub fn trace(name: &str, line: &str) {
    let mut p = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push("momPanel");
    let _ = std::fs::create_dir_all(&p);
    p.push(format!("{name}.log"));
    let fresh = std::fs::metadata(&p).map(|m| m.len() > 20_000).unwrap_or(false);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = format!("{ts} {line}\n");
    let _ = if fresh {
        std::fs::write(&p, entry)
    } else {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&p)
            .and_then(|mut f| f.write_all(entry.as_bytes()))
    };
}
