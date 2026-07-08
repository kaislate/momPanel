//! Storage collector: reports usage of the primary disk.
//!
//! Linux is the real implementation. On any platform we enumerate disks via
//! sysinfo; if the list is empty (or anything fails) we return `Unavailable`.
//! Selection: prefer the disk mounted at "/" (Linux root), otherwise the disk
//! with the largest total space.

use serde::Serialize;
use sysinfo::Disks;

// Decimal GB (1e9), not GiB: this matches the capacity GNOME Files/Disks (and drive
// labels) report, so the tile's numbers line up with what the user sees elsewhere.
const GB: u64 = 1_000_000_000;

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

/// Choose which disk the "Storage" tile should report. The tile is meant to show the
/// drive that holds the user's OS and files (C:\ on Windows, / or /home on Linux), NOT
/// merely the largest attached disk. We pick the mount point that is the longest path
/// prefix of the user's home directory; failing that, "/" (Linux); failing that, the
/// largest disk by total size.
fn choose_disk(
    mounts: &[std::path::PathBuf],
    home: Option<&std::path::Path>,
    totals: &[u64],
) -> Option<usize> {
    if let Some(h) = home {
        if let Some((i, _)) = mounts
            .iter()
            .enumerate()
            .filter(|(_, m)| h.starts_with(m))
            .max_by_key(|(_, m)| m.as_os_str().len())
        {
            return Some(i);
        }
    }
    if let Some(i) = mounts
        .iter()
        .position(|m| m == std::path::Path::new("/"))
    {
        return Some(i);
    }
    totals
        .iter()
        .enumerate()
        .max_by_key(|(_, t)| **t)
        .map(|(i, _)| i)
}

pub fn read() -> StorageData {
    let disks = Disks::new_with_refreshed_list();
    if disks.is_empty() {
        return StorageData::Unavailable;
    }

    let mounts: Vec<std::path::PathBuf> =
        disks.iter().map(|d| d.mount_point().to_path_buf()).collect();
    let totals: Vec<u64> = disks.iter().map(|d| d.total_space()).collect();
    let idx = match choose_disk(&mounts, dirs::home_dir().as_deref(), &totals) {
        Some(i) => i,
        None => return StorageData::Unavailable,
    };
    let disk = &disks[idx];

    let total = disk.total_space();
    if total == 0 {
        return StorageData::Unavailable;
    }
    let available = disk.available_space();
    let used = total.saturating_sub(available);

    StorageData::Ok {
        used_percent: percent(used, total),
        free_gb: available / GB,
        total_gb: total / GB,
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

    use std::path::{Path, PathBuf};

    #[test]
    fn picks_home_owning_mount() {
        let mounts = vec![PathBuf::from("/"), PathBuf::from("/home")];
        let totals = vec![500u64, 200u64];
        // /home is the longer prefix of /home/user, so it wins even though / is bigger.
        assert_eq!(choose_disk(&mounts, Some(Path::new("/home/user")), &totals), Some(1));
    }

    #[test]
    fn falls_back_to_root_without_home() {
        let mounts = vec![PathBuf::from("/mnt/big"), PathBuf::from("/")];
        let totals = vec![9000u64, 100u64];
        assert_eq!(choose_disk(&mounts, None, &totals), Some(1));
    }

    #[test]
    fn falls_back_to_largest_without_home_or_root() {
        let mounts = vec![PathBuf::from("/mnt/a"), PathBuf::from("/mnt/b")];
        let totals = vec![100u64, 900u64];
        assert_eq!(choose_disk(&mounts, None, &totals), Some(1));
    }

    #[cfg(windows)]
    #[test]
    fn picks_system_drive_over_largest_on_windows() {
        let mounts = vec![PathBuf::from("C:\\"), PathBuf::from("R:\\")];
        let totals = vec![1810u64, 5589u64];
        // C:\ holds the user's home, so it wins over the bigger R:\ drive.
        assert_eq!(choose_disk(&mounts, Some(Path::new("C:\\Users\\x")), &totals), Some(0));
    }
}
