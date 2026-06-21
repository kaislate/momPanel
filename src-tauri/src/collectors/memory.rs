//! Memory collector. Reads total/used RAM via sysinfo (bytes) and reports the
//! usage percentage plus totals in MB. Real on all platforms sysinfo supports;
//! falls back to `Unavailable` when total memory reads as 0.

use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use sysinfo::System;

// Reuse one System across polls (every ~3s) instead of allocating a fresh one each
// time; we only ever refresh the memory subsystem.
fn shared_system() -> &'static Mutex<System> {
    static SYS: OnceLock<Mutex<System>> = OnceLock::new();
    SYS.get_or_init(|| Mutex::new(System::new()))
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum MemoryData {
    Ok {
        used_percent: f32,
        total_mb: u64,
        used_mb: u64,
    },
    Unavailable,
}

/// Usage percentage of `used` out of `total`. Returns 0.0 when `total` is 0.
fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64 * 100.0) as f32
}

pub fn read() -> MemoryData {
    // Recover from a poisoned lock so one panicked poll can't disable the tile forever.
    let mut sys = match shared_system().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    sys.refresh_memory();

    let total = sys.total_memory(); // bytes
    let used = sys.used_memory(); // bytes

    if total == 0 {
        return MemoryData::Unavailable;
    }

    MemoryData::Ok {
        used_percent: percent(used, total),
        total_mb: total / 1_048_576,
        used_mb: used / 1_048_576,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_half() {
        assert_eq!(percent(50, 100), 50.0);
    }

    #[test]
    fn percent_zero_total() {
        assert_eq!(percent(0, 0), 0.0);
    }

    #[test]
    fn percent_zero_used() {
        assert_eq!(percent(0, 100), 0.0);
    }
}
