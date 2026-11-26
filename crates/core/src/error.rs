//! Strongly typed errors for the RustMatte core library.

use std::path::PathBuf;
use thiserror::Error;

use crate::model::ModelTier;

/// Errors that can occur within the RustMatte core library.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Failed to read or decode an input image.
    #[error("Failed to decode image '{path}': {reason}")]
    ImageDecode {
        /// File path that failed to decode.
        path: PathBuf,
        /// Underlying reason for failure.
        reason: String,
    },

    /// Image dimensions exceed safe processing limits.
    #[error("Image dimensions ({width}x{height}) exceed maximum allowed dimension of {max}px")]
    DimensionLimitExceeded {
        /// Image width in pixels.
        width: u32,
        /// Image height in pixels.
        height: u32,
        /// Maximum allowable dimension.
        max: u32,
    },

    /// Input image file is empty (zero bytes).
    #[error("Image file '{path}' is empty (0 bytes)")]
    ZeroByteInput {
        /// Empty file path.
        path: PathBuf,
    },

    /// Required ONNX model file was not found.
    #[error("Model file not found at '{path}' for tier {tier:?}")]
    ModelNotFound {
        /// Path where the model was looked up.
        path: PathBuf,
        /// Target model tier.
        tier: ModelTier,
    },

    /// Model file SHA-256 hash does not match expected checksum.
    #[error("Model integrity check failed for '{path}'. Expected SHA-256 '{expected}', found '{actual}'")]
    ModelChecksumMismatch {
        /// Corrupted model path.
        path: PathBuf,
        /// Expected SHA-256 hash string.
        expected: String,
        /// Actual computed SHA-256 hash string.
        actual: String,
    },

    /// Insecure plain HTTP URL was rejected.
    #[error("Insecure HTTP URL '{url}' is prohibited. Models must be fetched over HTTPS (or set RUSTMATTE_ALLOW_INSECURE_LOCALHOST=1 for local test servers).")]
    InsecureHttpProhibited {
        /// Rejected URL.
        url: String,
    },

    /// Download failed due to HTTP status.
    #[error("Failed to download model from '{url}': HTTP {status} {status_text}. {suggested_fix}")]
    DownloadHttpStatus {
        /// Target URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// HTTP status text.
        status_text: String,
        /// Actionable recommendation for the user.
        suggested_fix: String,
    },

    /// Download received HTML response instead of binary model data.
    #[error("Download from '{url}' returned an HTML page instead of binary model data. The file may have moved or the server requires authentication.")]
    DownloadHtmlResponse {
        /// Target URL.
        url: String,
    },

    /// Download stream was truncated before reaching expected size.
    #[error("Download truncated for '{url}'. Received {received} bytes, expected {expected} bytes. Check your network connection and retry.")]
    DownloadTruncated {
        /// Target URL.
        url: String,
        /// Actual received bytes.
        received: u64,
        /// Expected byte length.
        expected: u64,
    },

    /// Download stream exceeded the expected file size.
    #[error("Download exceeded expected size for '{url}'. Received more than {expected} bytes. Aborting download for safety.")]
    DownloadOversized {
        /// Target URL.
        url: String,
        /// Expected byte length.
        expected: u64,
    },

    /// Network error or connection dropped during download.
    #[error("Network connection error while connecting to '{url}': {reason}. Verify internet connectivity or place the model manually.")]
    DownloadNetwork {
        /// Target URL.
        url: String,
        /// Error description.
        reason: String,
    },

    /// Cache directory is not writable.
    #[error("Cache directory '{path}' is not writable: {reason}. Check directory permissions or configure RUSTMATTE_MODEL_DIR.")]
    CacheDirUnwritable {
        /// Target cache path.
        path: PathBuf,
        /// Error description.
        reason: String,
    },

    /// Insufficient free disk space for model download (`size + 10%`).
    #[error("Insufficient disk space in '{path}': need {required_bytes} bytes, only {available_bytes} bytes available.")]
    InsufficientDiskSpace {
        /// Target cache path.
        path: PathBuf,
        /// Required space in bytes (`model_size + 10%`).
        required_bytes: u64,
        /// Available space in bytes.
        available_bytes: u64,
    },

    /// Download was cancelled by user or caller (partial `.part` retained for resume).
    #[error(
        "Download cancelled for '{filename}'. Partial progress was kept and can be resumed later."
    )]
    DownloadCancelled {
        /// Model file name.
        filename: String,
    },

    /// Download was forbidden by configuration (--no-download).
    #[error("Model '{model_id}' is not found locally and network downloads are disabled by --no-download. Place '{filename}' in '{expected_path}' or run without --no-download.")]
    DownloadForbidden {
        /// Model identifier.
        model_id: &'static str,
        /// Expected filename.
        filename: &'static str,
        /// Suggested target directory or path.
        expected_path: PathBuf,
    },

    /// Download requires confirmation in non-interactive environment.
    #[error("Model '{model_id}' needs to be downloaded ({size_mb:.1} MB from {host}, license {license}). Run with '--yes' to confirm download, or run 'rustmatte models download', or place '{filename}' in '{expected_path}'.")]
    DownloadConsentRequired {
        /// Model identifier.
        model_id: &'static str,
        /// Expected filename.
        filename: &'static str,
        /// Size in MB.
        size_mb: f64,
        /// Source hostname.
        host: String,
        /// Model license.
        license: &'static str,
        /// Destination path.
        expected_path: PathBuf,
    },

    /// Failed to initialize or execute ONNX Runtime session.
    #[error("ONNX Runtime error: {0}")]
    InferenceSession(String),

    /// Output tensor extraction failed or produced unexpected shape.
    #[error("Tensor extraction failed: {0}")]
    TensorExtraction(String),

    /// Standard I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
