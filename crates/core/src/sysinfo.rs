//! Local hardware memory and disk-space probing (`PRD_V2.md` §7.2 & §7.3).
//!
//! Uses `sysinfo` with `system` and `disk` features only (no network access).
//! Detected RAM is transient for local recommendations and is never persisted or transmitted.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sysinfo::{Disks, MemoryRefreshKind, RefreshKind, System};

/// Snapshot of system memory in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Available RAM in bytes.
    pub available_bytes: u64,
    /// Total physical RAM in bytes.
    pub total_bytes: u64,
}

impl MemoryInfo {
    /// Returns available RAM in whole mebibytes (`MiB`).
    #[must_use]
    pub fn available_mb(&self) -> u64 {
        self.available_bytes / (1024 * 1024)
    }

    /// Returns total RAM in whole mebibytes (`MiB`).
    #[must_use]
    pub fn total_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }
}

/// Result of checking free disk space against `model_size + 10%`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskSpaceCheck {
    /// Required free bytes (`model_size_bytes + 10%`, rounded up).
    pub required_bytes: u64,
    /// Available free bytes on the target filesystem.
    pub available_bytes: u64,
    /// True when `available_bytes >= required_bytes`.
    pub sufficient: bool,
}

/// Computes required disk space in bytes as `model_size_bytes + 10%` (rounded up).
#[must_use]
pub fn required_space_with_margin(model_size_bytes: u64) -> u64 {
    ((u128::from(model_size_bytes) * 11).div_ceil(10)) as u64
}

/// Probes current system available and total memory locally.
///
/// Returns `None` if the OS reports zero memory or initialization fails.
#[must_use]
pub fn detect_memory_info() -> Option<MemoryInfo> {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
    )
    .ok()?;
    let available_bytes = sys.available_memory();
    let total_bytes = sys.total_memory();
    if available_bytes == 0 && total_bytes == 0 {
        return None;
    }
    Some(MemoryInfo {
        available_bytes,
        total_bytes,
    })
}

/// Probes current system available RAM in whole MiB (`None` if detection fails).
#[must_use]
pub fn detect_available_memory_mb() -> Option<u64> {
    detect_memory_info().and_then(|info| {
        let mb = info.available_mb();
        if mb == 0 && info.total_bytes == 0 {
            None
        } else {
            Some(mb)
        }
    })
}

fn normalize_for_mount_match(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let stripped = if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        raw.into_owned()
    };

    #[cfg(windows)]
    {
        PathBuf::from(stripped.to_ascii_lowercase())
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(stripped)
    }
}

fn resolve_existing_anchor(path: &Path) -> PathBuf {
    let mut cur = path.to_path_buf();
    loop {
        if cur.exists() {
            return std::fs::canonicalize(&cur).unwrap_or(cur);
        }
        if let Some(parent) = cur.parent() {
            if parent.as_os_str().is_empty() {
                break;
            }
            cur = parent.to_path_buf();
        } else {
            break;
        }
    }
    std::env::current_dir()
        .and_then(std::fs::canonicalize)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Probes available disk space (in bytes) on the filesystem hosting `target_dir`.
///
/// Walks up to the nearest existing ancestor directory when `target_dir` does not yet exist.
#[must_use]
pub fn detect_available_disk_bytes(target_dir: &Path) -> Option<u64> {
    let anchor = resolve_existing_anchor(target_dir);
    let norm_anchor = normalize_for_mount_match(&anchor);
    let disks = Disks::new_with_refreshed_list().ok()?;

    let mut best: Option<(usize, u64)> = None;
    for disk in &disks {
        let norm_mount = normalize_for_mount_match(disk.mount_point());
        if norm_anchor.starts_with(&norm_mount) {
            let depth = norm_mount.as_os_str().len();
            if best.is_none_or(|(best_depth, _)| depth > best_depth) {
                best = Some((depth, disk.available_space()));
            }
        }
    }

    best.map(|(_, avail)| avail)
}

/// Checks disk space using an injectable probe function (for deterministic CI tests).
pub fn check_disk_space_with<F>(
    target_dir: &Path,
    model_size_bytes: u64,
    probe: F,
) -> Option<DiskSpaceCheck>
where
    F: FnOnce(&Path) -> Option<u64>,
{
    let required_bytes = required_space_with_margin(model_size_bytes);
    let available_bytes = probe(target_dir)?;
    Some(DiskSpaceCheck {
        required_bytes,
        available_bytes,
        sufficient: available_bytes >= required_bytes,
    })
}

/// Checks whether the filesystem containing `target_dir` has `model_size_bytes + 10%` free.
#[must_use]
pub fn check_disk_space(target_dir: &Path, model_size_bytes: u64) -> Option<DiskSpaceCheck> {
    check_disk_space_with(target_dir, model_size_bytes, detect_available_disk_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_space_with_margin_rounds_up_ten_percent() {
        assert_eq!(required_space_with_margin(0), 0);
        assert_eq!(required_space_with_margin(100), 110);
        assert_eq!(required_space_with_margin(101), 112);
        // 191,877,254 * 1.1 = 211,064,979.4 -> 211,064,980
        assert_eq!(required_space_with_margin(191_877_254), 211_064_980);
    }

    #[test]
    fn test_check_disk_space_with_injected_values() {
        let dir = Path::new("/tmp/models");

        let exact = check_disk_space_with(dir, 1_000, |_| Some(1_100)).unwrap();
        assert_eq!(exact.required_bytes, 1_100);
        assert_eq!(exact.available_bytes, 1_100);
        assert!(exact.sufficient);

        let short = check_disk_space_with(dir, 1_000, |_| Some(1_099)).unwrap();
        assert!(!short.sufficient);

        let unknown = check_disk_space_with(dir, 1_000, |_| None);
        assert!(unknown.is_none());
    }

    #[test]
    fn test_detect_memory_and_disk_smoke() {
        let info = detect_memory_info();
        assert!(info.is_some());
        let mem = info.unwrap();
        assert!(mem.total_bytes > 0);
        assert!(detect_available_memory_mb().is_some());

        let temp_dir = std::env::temp_dir().join("rustmatte_nonexistent_disk_probe_subdir");
        let avail = detect_available_disk_bytes(&temp_dir);
        assert!(avail.is_some());
        assert!(avail.unwrap() > 0);
    }
}
