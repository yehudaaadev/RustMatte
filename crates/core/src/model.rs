//! Model tier specifications, parameters, and verification.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::CoreError;

/// Supported background removal model tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelTier {
    /// High-quality default model (`BiRefNet_lite_512`).
    /// Best quality on complex subjects, fine hair, and transparency.
    #[default]
    Quality,
    /// Fast preview model (`U-2-Netp`).
    /// Honest note: Best for people and products on clean backgrounds;
    /// may leave smears on fur or textured backgrounds.
    Fast,
}

/// Internal classification of model output semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    /// BiRefNet logits output requiring sigmoid activation.
    BiRefNet,
    /// U-2-Netp output requiring min-max normalization.
    U2Netp,
}

/// Execution specifications for a neural network model.
#[derive(Debug, Clone)]
pub struct ModelSpec {
    /// Model tier enum.
    pub tier: ModelTier,
    /// Model identifier.
    pub id: &'static str,
    /// Human-readable model title.
    pub name: &'static str,
    /// Default model file name in local storage.
    pub default_filename: &'static str,
    /// Expected SHA-256 hash in lowercase hex.
    pub expected_sha256: &'static str,
    /// Tensor input width in pixels.
    pub width: u32,
    /// Tensor input height in pixels.
    pub height: u32,
    /// ONNX input tensor name.
    pub input_name: &'static str,
    /// ONNX output tensor name.
    pub output_name: &'static str,
    /// Output processing kind.
    pub kind: ModelKind,
    /// Channel normalization means (RGB).
    pub mean: [f32; 3],
    /// Channel normalization standard deviations (RGB).
    pub std: [f32; 3],
}

// CITED PARAMETER SOURCES:
//
// 1. BiRefNet_lite_512:
//    - Source export: https://huggingface.co/studioludens/birefnet-lite-512 (documented in docs/references/README.md)
//    - Verified in crates/core/examples/bench_matrix.rs lines 333-341
//    - Input shape: 512x512 RGB (ImageNet mean [0.485, 0.456, 0.406], std [0.229, 0.224, 0.225])
//    - Input tensor: "input_image"
//    - Output tensor: "output_image" (raw logits requiring sigmoid activation)
//    - SHA-256: 1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2
pub static SPEC_QUALITY: ModelSpec = ModelSpec {
    tier: ModelTier::Quality,
    id: "birefnet_lite_512",
    name: "BiRefNet Lite 512 (Quality Tier)",
    default_filename: "birefnet_lite_512.onnx",
    expected_sha256: "1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2",
    width: 512,
    height: 512,
    input_name: "input_image",
    output_name: "output_image",
    kind: ModelKind::BiRefNet,
    mean: [0.485, 0.456, 0.406],
    std: [0.229, 0.224, 0.225],
};

// 2. U-2-Netp:
//    - Source export: https://huggingface.co/edgetools/u2netp (documented in docs/references/README.md)
//    - Verified in crates/core/examples/bench_matrix.rs lines 342-350
//    - Input shape: 320x320 RGB (ImageNet mean [0.485, 0.456, 0.406], std [0.229, 0.224, 0.225])
//    - Input tensor: "input.1"
//    - Output tensor: "1959" (logits requiring min-max normalization)
//    - SHA-256: 309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8
pub static SPEC_FAST: ModelSpec = ModelSpec {
    tier: ModelTier::Fast,
    id: "u2netp",
    name: "U-2-Netp (Fast Tier)",
    default_filename: "u2netp.onnx",
    expected_sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    width: 320,
    height: 320,
    input_name: "input.1",
    output_name: "1959",
    kind: ModelKind::U2Netp,
    mean: [0.485, 0.456, 0.406],
    std: [0.229, 0.224, 0.225],
};

/// Returns the static specification for the chosen tier.
#[must_use]
pub fn spec_for_tier(tier: ModelTier) -> &'static ModelSpec {
    match tier {
        ModelTier::Quality => &SPEC_QUALITY,
        ModelTier::Fast => &SPEC_FAST,
    }
}

/// Verifies that a local model file matches its expected SHA-256 checksum.
pub fn verify_model_checksum(path: &Path, expected_hex: &str) -> Result<(), CoreError> {
    let mut file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::ModelNotFound {
                path: path.to_path_buf(),
                tier: ModelTier::Quality,
            }
        } else {
            CoreError::Io(e)
        }
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let actual_hex = format!("{:x}", hasher.finalize());
    if actual_hex.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(CoreError::ModelChecksumMismatch {
            path: path.to_path_buf(),
            expected: expected_hex.to_ascii_lowercase(),
            actual: actual_hex,
        })
    }
}

/// Resolves the model path, checking the default `./models/` location if omitted.
pub fn resolve_model_path(
    custom_path: Option<&Path>,
    tier: ModelTier,
) -> Result<PathBuf, CoreError> {
    let spec = spec_for_tier(tier);
    let path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("models").join(spec.default_filename),
    };

    if !path.exists() {
        return Err(CoreError::ModelNotFound { path, tier });
    }

    Ok(path)
}
