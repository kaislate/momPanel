//! Storage collector: reports usage of the primary disk.
//!
//! Linux is the real implementation. On any platform we enumerate disks via
//! sysinfo; if the list is empty (or anything fails) we return `Unavailable`.
//! Selection: prefer the disk mounted at "/" (Linux root), otherwise the disk
//! with the largest total space.

use serde::Serialize;
use sysinfo::Disks;

const GIB: u64 = 1_073_741_824;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum StorageData {
    Ok {
        used_percent: f32,
        free_gb: u64,
        total_gb: u64,
    },
    Unavailable,
}

/// Used-space percentage (0-100). Returns 0.0 when total is 0 to avoid div-by-zero.
pub fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64 * 100.0) as f32
}

pub fn read() -> StorageData {
    let disks = Disks::new_with_refreshed_list();
    if disks.is_empty() {
        return StorageData::Unavailable;
    }

    // Prefer the root mount "/" (Linux); otherwise the largest disk by total space.
    let chosen = disks
        .iter()
        .find(|d| d.mount_point().to_str() == Some("/"))
        .or_else(|| disks.iter().max_by_key(|d| d.total_space()));

    let disk = match chosen {
        Some(d) => d,
        None => return StorageData::Unavailable,
    };

    let total = disk.total_space();
    if total == 0 {
        return StorageData::Unavailable;
    }
    let available = disk.available_space();
    let used = total.saturating_sub(available);

    StorageData::Ok {
        used_percent: percent(used, total),
        free_gb: available / GIB,
        total_gb: total / GIB,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_basic() {
        assert_eq!(percent(50, 100), 50.0);
        assert_eq!(percent(0, 100), 0.0);
        assert_eq!(percent(100, 100), 100.0);
    }

    #[test]
    fn percent_zero_total_is_zero() {
        assert_eq!(percent(10, 0), 0.0);
    }

    #[test]
    fn percent_realistic() {
        // ~63% used, like the mock fixture.
        let p = percent(308, 488);
        assert!((p - 63.11).abs() < 0.5, "got {p}");
    }
}
