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
    #[error("Model file not found at '{path}' for tier {tier:?}. Automatic downloading will be available in a later version.")]
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
