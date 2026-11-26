//! Safe streaming model downloader with HTTPS enforcement, HTTP Range resume,
//! disk-space checks, exponential retries, and full-file SHA-256 verification.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::model::{verify_model_checksum, ModelSpec};
use crate::sysinfo::{detect_available_disk_bytes, required_space_with_margin};

const DEFAULT_RETRY_DELAYS: [Duration; 3] = [
    Duration::from_millis(500),
    Duration::from_millis(1500),
    Duration::from_millis(3000),
];

/// Persistent metadata stored alongside `.part` so HTTP Range resume only continues
/// when URL, expected size, and expected SHA-256 match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PartialDownloadMeta {
    url: String,
    expected_size: u64,
    expected_sha256: String,
}

/// Summary of an existing partial download on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialDownloadInfo {
    /// Path to the `.part` file.
    pub part_path: PathBuf,
    /// Number of bytes currently buffered in `.part`.
    pub bytes_downloaded: u64,
    /// Source URL recorded in metadata.
    pub url: String,
}

/// Type alias for injectable disk free-space probe (`target_dir -> Option<available_bytes>`).
pub type DiskProbe<'a> = &'a dyn Fn(&Path) -> Option<u64>;

/// Configuration options for resumable model downloads.
pub struct DownloadOptions<'a> {
    /// Optional cancellation flag; when true, `.part` and metadata are preserved for later resume.
    pub cancel: Option<&'a AtomicBool>,
    /// Retry backoff delays on transient transport/5xx errors.
    pub retry_delays: &'a [Duration],
    /// Whether to verify `model_size + 10%` free disk space before downloading.
    pub check_disk: bool,
    /// Optional disk probe override for deterministic tests.
    pub disk_probe: Option<DiskProbe<'a>>,
}

impl Default for DownloadOptions<'_> {
    fn default() -> Self {
        Self {
            cancel: None,
            retry_delays: &DEFAULT_RETRY_DELAYS,
            check_disk: true,
            disk_probe: None,
        }
    }
}

/// Extracts the host string from a URL for disclosure and confirmation prompts.
#[must_use]
pub fn extract_source_host(url: &str) -> String {
    if let Some(stripped) = url.strip_prefix("https://") {
        stripped.split('/').next().unwrap_or("unknown").to_string()
    } else if let Some(stripped) = url.strip_prefix("http://") {
        stripped.split('/').next().unwrap_or("unknown").to_string()
    } else {
        "unknown".to_string()
    }
}

fn part_file_path(spec: &ModelSpec, target_dir: &Path) -> PathBuf {
    target_dir.join(format!(".{}.part", spec.default_filename))
}

fn meta_file_path(spec: &ModelSpec, target_dir: &Path) -> PathBuf {
    target_dir.join(format!(".{}.part.meta.json", spec.default_filename))
}

fn read_partial_meta(meta_path: &Path) -> Option<PartialDownloadMeta> {
    let raw = std::fs::read_to_string(meta_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn write_partial_meta(meta_path: &Path, meta: &PartialDownloadMeta) -> Result<(), CoreError> {
    let json = serde_json::to_string(meta)
        .map_err(|e| CoreError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::write(meta_path, json)?;
    Ok(())
}

fn remove_partial_files(part_path: &Path, meta_path: &Path) {
    let _ = std::fs::remove_file(part_path);
    let _ = std::fs::remove_file(meta_path);
}

/// Returns information about an existing valid partial download in `target_dir`, if any.
#[must_use]
pub fn partial_download_info(spec: &ModelSpec, target_dir: &Path) -> Option<PartialDownloadInfo> {
    let part_path = part_file_path(spec, target_dir);
    let meta_path = meta_file_path(spec, target_dir);
    if !part_path.exists() || !meta_path.exists() {
        return None;
    }
    let meta = read_partial_meta(&meta_path)?;
    if meta.expected_size != spec.expected_size
        || !meta
            .expected_sha256
            .eq_ignore_ascii_case(spec.expected_sha256)
    {
        return None;
    }
    let len = std::fs::metadata(&part_path).ok()?.len();
    if len == 0 || len > spec.expected_size {
        return None;
    }
    Some(PartialDownloadInfo {
        part_path,
        bytes_downloaded: len,
        url: meta.url,
    })
}

/// Explicitly deletes any `.part` and metadata files for `spec` inside `target_dir`.
pub fn delete_partial_download(spec: &ModelSpec, target_dir: &Path) -> Result<bool, CoreError> {
    let part_path = part_file_path(spec, target_dir);
    let meta_path = meta_file_path(spec, target_dir);
    let mut existed = false;

    if part_path.exists() {
        std::fs::remove_file(&part_path)?;
        existed = true;
    }
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)?;
        existed = true;
    }
    Ok(existed)
}

/// Checks whether an HTTP URL is permissible under localhost test overrides.
fn validate_url_scheme(url: &str) -> Result<(), CoreError> {
    if url.starts_with("https://") {
        return Ok(());
    }

    if url.starts_with("http://") {
        let is_localhost = url.starts_with("http://localhost")
            || url.starts_with("http://127.0.0.1")
            || url.starts_with("http://[::1]");
        let override_set =
            std::env::var("RUSTMATTE_ALLOW_INSECURE_LOCALHOST").as_deref() == Ok("1");

        if is_localhost && override_set {
            return Ok(());
        }
    }

    Err(CoreError::InsecureHttpProhibited {
        url: url.to_string(),
    })
}

fn is_cancelled(cancel: Option<&AtomicBool>) -> bool {
    cancel.is_some_and(|flag| flag.load(Ordering::Relaxed))
}

fn parse_content_range_start(header: &str) -> Option<u64> {
    // Expected format: "bytes <start>-<end>/<total>" or "bytes <start>-<end>/*"
    let rest = header.trim().strip_prefix("bytes ")?;
    let (start_str, _) = rest.split_once('-')?;
    start_str.trim().parse::<u64>().ok()
}

fn verify_disk_space(
    spec: &ModelSpec,
    target_dir: &Path,
    existing_part_bytes: u64,
    options: &DownloadOptions<'_>,
) -> Result<(), CoreError> {
    if !options.check_disk {
        return Ok(());
    }

    let required_total = required_space_with_margin(spec.expected_size);
    let effective_needed = required_total.saturating_sub(existing_part_bytes);
    let available = match options.disk_probe {
        Some(probe) => probe(target_dir),
        None => detect_available_disk_bytes(target_dir),
    };

    if let Some(avail) = available {
        if avail < effective_needed {
            return Err(CoreError::InsufficientDiskSpace {
                path: target_dir.to_path_buf(),
                required_bytes: required_total,
                available_bytes: avail,
            });
        }
    }

    Ok(())
}

enum AttemptOutcome {
    Complete,
    Retryable(CoreError),
    NonRetryable(CoreError),
    Cancelled,
}

#[allow(clippy::too_many_arguments)]
fn attempt_url_download<F>(
    agent: &ureq::Agent,
    url: &str,
    spec: &ModelSpec,
    target_dir: &Path,
    part_path: &Path,
    meta_path: &Path,
    cancel: Option<&AtomicBool>,
    progress_callback: &mut F,
) -> AttemptOutcome
where
    F: FnMut(u64, u64),
{
    if is_cancelled(cancel) {
        return AttemptOutcome::Cancelled;
    }

    // Determine whether an existing .part file safely matches this URL and model metadata.
    let mut resume_offset: u64 = 0;
    if part_path.exists() {
        let valid_meta = read_partial_meta(meta_path).is_some_and(|m| {
            m.url == url
                && m.expected_size == spec.expected_size
                && m.expected_sha256.eq_ignore_ascii_case(spec.expected_sha256)
        });
        let part_len = std::fs::metadata(part_path).map(|m| m.len()).unwrap_or(0);
        if valid_meta && part_len > 0 && part_len < spec.expected_size {
            resume_offset = part_len;
        } else if valid_meta && part_len == spec.expected_size {
            if verify_model_checksum(part_path, spec.expected_sha256).is_ok() {
                return AttemptOutcome::Complete;
            }
            remove_partial_files(part_path, meta_path);
        } else {
            remove_partial_files(part_path, meta_path);
        }
    }

    let mut req = agent.get(url);
    if resume_offset > 0 {
        req = req.set("Range", &format!("bytes={resume_offset}-"));
    }

    let response = match req.call() {
        Ok(resp) => resp,
        Err(ureq::Error::Status(416, _)) if resume_offset > 0 => {
            remove_partial_files(part_path, meta_path);
            return AttemptOutcome::Retryable(CoreError::DownloadHttpStatus {
                url: url.to_string(),
                status: 416,
                status_text: "Range Not Satisfiable".to_string(),
                suggested_fix: "Restarting download cleanly from byte 0.".to_string(),
            });
        }
        Err(ureq::Error::Status(status, resp)) => {
            let status_text = resp.status_text().to_string();
            let suggested_fix = if status == 404 {
                "The model file was not found on the remote server. Check model release URLs or mirror status."
            } else {
                "The remote server returned an HTTP error. Check network availability and retry."
            };
            let err = CoreError::DownloadHttpStatus {
                url: url.to_string(),
                status,
                status_text,
                suggested_fix: suggested_fix.to_string(),
            };
            return if status >= 500 || status == 429 {
                AttemptOutcome::Retryable(err)
            } else {
                AttemptOutcome::NonRetryable(err)
            };
        }
        Err(ureq::Error::Transport(e)) => {
            return AttemptOutcome::Retryable(CoreError::DownloadNetwork {
                url: url.to_string(),
                reason: e.to_string(),
            });
        }
    };

    if let Some(content_type) = response.header("content-type") {
        if content_type.to_ascii_lowercase().contains("text/html") {
            remove_partial_files(part_path, meta_path);
            return AttemptOutcome::NonRetryable(CoreError::DownloadHtmlResponse {
                url: url.to_string(),
            });
        }
    }

    let status = response.status();
    let appending = if resume_offset > 0 && status == 206 {
        let range_ok = response
            .header("content-range")
            .and_then(parse_content_range_start)
            .is_some_and(|start| start == resume_offset);
        if !range_ok {
            remove_partial_files(part_path, meta_path);
            return AttemptOutcome::Retryable(CoreError::DownloadNetwork {
                url: url.to_string(),
                reason: "Server returned 206 with mismatched Content-Range; restarting cleanly."
                    .to_string(),
            });
        }
        true
    } else if status == 200 {
        false
    } else {
        return AttemptOutcome::NonRetryable(CoreError::DownloadHttpStatus {
            url: url.to_string(),
            status,
            status_text: response.status_text().to_string(),
            suggested_fix: "Unexpected HTTP status from download server.".to_string(),
        });
    };

    let meta = PartialDownloadMeta {
        url: url.to_string(),
        expected_size: spec.expected_size,
        expected_sha256: spec.expected_sha256.to_ascii_lowercase(),
    };
    if let Err(e) = write_partial_meta(meta_path, &meta) {
        return AttemptOutcome::NonRetryable(e);
    }

    let mut file = if appending {
        match OpenOptions::new().write(true).open(part_path) {
            Ok(mut f) => {
                if let Err(e) = f.seek(SeekFrom::Start(resume_offset)) {
                    return AttemptOutcome::NonRetryable(CoreError::Io(e));
                }
                f
            }
            Err(e) => {
                return AttemptOutcome::NonRetryable(CoreError::CacheDirUnwritable {
                    path: target_dir.to_path_buf(),
                    reason: e.to_string(),
                });
            }
        }
    } else {
        resume_offset = 0;
        match File::create(part_path) {
            Ok(f) => f,
            Err(e) => {
                return AttemptOutcome::NonRetryable(CoreError::CacheDirUnwritable {
                    path: target_dir.to_path_buf(),
                    reason: e.to_string(),
                });
            }
        }
    };

    let mut reader = response.into_reader();
    let mut buffer = [0u8; 65536];
    let mut received_bytes = resume_offset;

    if received_bytes > 0 {
        progress_callback(received_bytes, spec.expected_size);
    }

    loop {
        if is_cancelled(cancel) {
            let _ = file.flush();
            return AttemptOutcome::Cancelled;
        }

        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if let Err(e) = file.write_all(&buffer[..count]) {
                    return AttemptOutcome::NonRetryable(CoreError::Io(e));
                }
                received_bytes += count as u64;
                if received_bytes > spec.expected_size {
                    drop(file);
                    remove_partial_files(part_path, meta_path);
                    return AttemptOutcome::NonRetryable(CoreError::DownloadOversized {
                        url: url.to_string(),
                        expected: spec.expected_size,
                    });
                }
                progress_callback(received_bytes, spec.expected_size);
            }
            Err(e) => {
                let _ = file.flush();
                return AttemptOutcome::Retryable(CoreError::DownloadNetwork {
                    url: url.to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }

    if let Err(e) = file.flush() {
        return AttemptOutcome::NonRetryable(CoreError::Io(e));
    }
    drop(file);

    if received_bytes < spec.expected_size {
        return AttemptOutcome::Retryable(CoreError::DownloadTruncated {
            url: url.to_string(),
            received: received_bytes,
            expected: spec.expected_size,
        });
    }

    if let Err(e) = verify_model_checksum(part_path, spec.expected_sha256) {
        remove_partial_files(part_path, meta_path);
        return AttemptOutcome::NonRetryable(e);
    }

    AttemptOutcome::Complete
}

/// Downloads a model specification with disk check, HTTP Range resume, retries, and cancellation.
pub fn download_model_with_options<F>(
    spec: &ModelSpec,
    target_dir: &Path,
    options: DownloadOptions<'_>,
    mut progress_callback: F,
) -> Result<PathBuf, CoreError>
where
    F: FnMut(u64, u64),
{
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).map_err(|e| CoreError::CacheDirUnwritable {
            path: target_dir.to_path_buf(),
            reason: e.to_string(),
        })?;
    }

    let part_path = part_file_path(spec, target_dir);
    let meta_path = meta_file_path(spec, target_dir);
    let existing_part_bytes = partial_download_info(spec, target_dir)
        .map(|info| info.bytes_downloaded)
        .unwrap_or(0);
    verify_disk_space(spec, target_dir, existing_part_bytes, &options)?;

    let target_path = target_dir.join(spec.default_filename);
    let override_url = std::env::var("RUSTMATTE_MODEL_SOURCE_OVERRIDE_URL").ok();
    let mut candidate_urls: Vec<&str> = Vec::new();
    if let Some(ref u) = override_url {
        candidate_urls.push(u.as_str());
    }
    candidate_urls.extend_from_slice(spec.source_urls);

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(30))
        .redirects(5)
        .user_agent(concat!("rustmatte/", env!("CARGO_PKG_VERSION")))
        .build();

    let mut last_error: Option<CoreError> = None;

    for url in candidate_urls {
        if url.starts_with("TODO") {
            continue;
        }
        if let Err(e) = validate_url_scheme(url) {
            last_error = Some(e);
            continue;
        }

        let max_attempts = 1 + options.retry_delays.len();
        for attempt in 0..max_attempts {
            match attempt_url_download(
                &agent,
                url,
                spec,
                target_dir,
                &part_path,
                &meta_path,
                options.cancel,
                &mut progress_callback,
            ) {
                AttemptOutcome::Complete => {
                    if target_path.exists() {
                        let _ = std::fs::remove_file(&target_path);
                    }
                    if let Err(e) = std::fs::rename(&part_path, &target_path) {
                        remove_partial_files(&part_path, &meta_path);
                        return Err(CoreError::Io(e));
                    }
                    let _ = std::fs::remove_file(&meta_path);
                    return Ok(target_path);
                }
                AttemptOutcome::Cancelled => {
                    return Err(CoreError::DownloadCancelled {
                        filename: spec.default_filename.to_string(),
                    });
                }
                AttemptOutcome::NonRetryable(err) => {
                    last_error = Some(err);
                    break;
                }
                AttemptOutcome::Retryable(err) => {
                    last_error = Some(err);
                    if let Some(delay) = options.retry_delays.get(attempt) {
                        if is_cancelled(options.cancel) {
                            return Err(CoreError::DownloadCancelled {
                                filename: spec.default_filename.to_string(),
                            });
                        }
                        if !delay.is_zero() {
                            std::thread::sleep(*delay);
                        }
                    }
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| CoreError::DownloadNetwork {
        url: "none".to_string(),
        reason: "No valid download sources available".to_string(),
    }))
}

/// Downloads a model specification to `target_dir` using default retry and disk-space checks.
pub fn download_model<F>(
    spec: &ModelSpec,
    target_dir: &Path,
    progress_callback: F,
) -> Result<PathBuf, CoreError>
where
    F: FnMut(u64, u64),
{
    download_model_with_options(
        spec,
        target_dir,
        DownloadOptions::default(),
        progress_callback,
    )
}
