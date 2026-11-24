//! Safe streaming model downloader with HTTPS enforcement and checksum verification.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::CoreError;
use crate::model::ModelSpec;

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

/// Downloads a model specification to the specified directory with streaming verification.
///
/// Ensures:
/// - Target directory exists or is created.
/// - Strict HTTPS-only transport (except explicitly overridden localhost).
/// - Atomic writing via `.part` file.
/// - Stream size checking (aborts on truncation or oversized payload).
/// - SHA-256 verification before renaming to the final file name.
/// - Deletion of `.part` file on any failure.
pub fn download_model<F>(
    spec: &ModelSpec,
    target_dir: &Path,
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

    let target_path = target_dir.join(spec.default_filename);
    let pid = std::process::id();
    let temp_path = target_dir.join(format!(".{}.part.{}", spec.default_filename, pid));

    let mut last_error: Option<CoreError> = None;

    let override_url = std::env::var("RUSTMATTE_MODEL_SOURCE_OVERRIDE_URL").ok();
    let mut candidate_urls: Vec<&str> = Vec::new();
    if let Some(ref u) = override_url {
        candidate_urls.push(u.as_str());
    }
    candidate_urls.extend_from_slice(spec.source_urls);

    for url in candidate_urls {
        // Skip placeholders
        if url.starts_with("TODO") {
            continue;
        }

        if let Err(e) = validate_url_scheme(url) {
            last_error = Some(e);
            continue;
        }

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(15))
            .timeout_read(std::time::Duration::from_secs(30))
            .redirects(5)
            .user_agent(concat!("rustmatte/", env!("CARGO_PKG_VERSION")))
            .build();

        let response = match agent.get(url).call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(status, resp)) => {
                let status_text = resp.status_text().to_string();
                let suggested_fix = if status == 404 {
                    "The model file was not found on the remote server. Check model release URLs or mirror status."
                } else {
                    "The remote server returned an HTTP error. Check network availability and retry."
                };
                last_error = Some(CoreError::DownloadHttpStatus {
                    url: url.to_string(),
                    status,
                    status_text,
                    suggested_fix: suggested_fix.to_string(),
                });
                continue;
            }
            Err(ureq::Error::Transport(e)) => {
                last_error = Some(CoreError::DownloadNetwork {
                    url: url.to_string(),
                    reason: e.to_string(),
                });
                continue;
            }
        };

        if response.status() != 200 {
            last_error = Some(CoreError::DownloadHttpStatus {
                url: url.to_string(),
                status: response.status(),
                status_text: response.status_text().to_string(),
                suggested_fix: "Unexpected non-200 HTTP status.".to_string(),
            });
            continue;
        }

        if let Some(content_type) = response.header("content-type") {
            if content_type.to_ascii_lowercase().contains("text/html") {
                last_error = Some(CoreError::DownloadHtmlResponse {
                    url: url.to_string(),
                });
                continue;
            }
        }

        // Open temp part file
        let mut temp_file = match File::create(&temp_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(CoreError::CacheDirUnwritable {
                    path: target_dir.to_path_buf(),
                    reason: e.to_string(),
                });
            }
        };

        let mut reader = response.into_reader();
        let mut buffer = [0u8; 65536];
        let mut received_bytes: u64 = 0;
        let mut hasher = Sha256::new();
        let mut download_failed = false;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    hasher.update(&buffer[..count]);
                    if let Err(e) = temp_file.write_all(&buffer[..count]) {
                        download_failed = true;
                        last_error = Some(CoreError::Io(e));
                        break;
                    }
                    received_bytes += count as u64;
                    if received_bytes > spec.expected_size {
                        download_failed = true;
                        last_error = Some(CoreError::DownloadOversized {
                            url: url.to_string(),
                            expected: spec.expected_size,
                        });
                        break;
                    }
                    progress_callback(received_bytes, spec.expected_size);
                }
                Err(e) => {
                    download_failed = true;
                    last_error = Some(CoreError::DownloadNetwork {
                        url: url.to_string(),
                        reason: e.to_string(),
                    });
                    break;
                }
            }
        }

        let _ = temp_file.flush();
        drop(temp_file);

        if download_failed {
            let _ = std::fs::remove_file(&temp_path);
            continue;
        }

        if received_bytes != spec.expected_size {
            let _ = std::fs::remove_file(&temp_path);
            last_error = Some(CoreError::DownloadTruncated {
                url: url.to_string(),
                received: received_bytes,
                expected: spec.expected_size,
            });
            continue;
        }

        let actual_hex = format!("{:x}", hasher.finalize());
        if !actual_hex.eq_ignore_ascii_case(spec.expected_sha256) {
            let _ = std::fs::remove_file(&temp_path);
            last_error = Some(CoreError::ModelChecksumMismatch {
                path: temp_path.clone(),
                expected: spec.expected_sha256.to_ascii_lowercase(),
                actual: actual_hex,
            });
            continue;
        }

        // On Windows rename fails if destination exists
        if target_path.exists() {
            let _ = std::fs::remove_file(&target_path);
        }

        if let Err(e) = std::fs::rename(&temp_path, &target_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(CoreError::Io(e));
        }

        return Ok(target_path);
    }

    // Clean up temporary file if still present
    if temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }

    Err(last_error.unwrap_or_else(|| CoreError::DownloadNetwork {
        url: "none".to_string(),
        reason: "No valid download sources available".to_string(),
    }))
}
