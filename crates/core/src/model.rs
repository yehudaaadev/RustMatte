//! Model tier specifications, manifest registry, cache directory resolution, and integrity verification.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::CoreError;

/// Supported background removal model tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModelTier {
    /// High-quality default model (`BiRefNet_lite_512`).
    /// Best quality on complex subjects, fine hair, and transparency.
    #[default]
    Quality,
    /// Fast preview model (`U-2-Netp`).
    /// Honest note: Best for people and products on clean backgrounds;
    /// may leave smears on fur or textured backgrounds.
    Fast,
    /// Maximum quality model (`BEN2 Base FP32`).
    /// Honest note: 1024x1024 input; ~13-15s/photo on CPU, ~2.6-2.8 GB RAM; faster with GPU.
    Maximum,
}

/// Internal classification of model output semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelKind {
    /// BiRefNet logits output requiring sigmoid activation.
    BiRefNet,
    /// U-2-Netp output requiring min-max normalization.
    U2Netp,
    /// BEN2 Base FP32 output requiring min-max normalization.
    Ben2,
}

/// Execution specifications and distribution manifest for an AI model.
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
    /// Expected file size in bytes.
    pub expected_size: u64,
    /// Official model license.
    pub license: &'static str,
    /// Short description of the source repository or export.
    pub source_description: &'static str,
    /// Ordered list of candidate download URLs.
    pub source_urls: &'static [&'static str],
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

// CITED PARAMETER SOURCES & MANIFEST:
//
// 1. BiRefNet_lite_512:
//    - Source export: https://huggingface.co/studioludens/birefnet-lite-512 (documented in docs/references/README.md)
//    - Verified in crates/core/examples/bench_matrix.rs lines 333-341
//    - Real byte length: 191,877,254 bytes
//    - SHA-256: 1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2
pub static SPEC_QUALITY: ModelSpec = ModelSpec {
    tier: ModelTier::Quality,
    id: "birefnet_lite_512",
    name: "BiRefNet Lite 512 (Quality Tier)",
    default_filename: "birefnet_lite_512.onnx",
    expected_sha256: "1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2",
    expected_size: 191_877_254,
    license: "MIT",
    source_description:
        "BiRefNet_lite 512 FP32 ONNX export from HuggingFace (studioludens/birefnet-lite-512)",
    source_urls: &[
        "https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/birefnet_lite_512.onnx",
        "https://huggingface.co/studioludens/birefnet-lite-512/resolve/main/onnx/model.onnx",
    ],
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
//    - Real byte length: 4,574,861 bytes
//    - SHA-256: 309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8
pub static SPEC_FAST: ModelSpec = ModelSpec {
    tier: ModelTier::Fast,
    id: "u2netp",
    name: "U-2-Netp (Fast Tier)",
    default_filename: "u2netp.onnx",
    expected_sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    expected_size: 4_574_861,
    license: "Apache-2.0",
    source_description: "U-2-Netp 320 FP32 ONNX export from HuggingFace (edgetools/u2netp)",
    source_urls: &[
        "https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/u2netp.onnx",
        "https://huggingface.co/edgetools/u2netp/resolve/main/u2netp.onnx",
    ],
    width: 320,
    height: 320,
    input_name: "input.1",
    output_name: "1959",
    kind: ModelKind::U2Netp,
    mean: [0.485, 0.456, 0.406],
    std: [0.229, 0.224, 0.225],
};

// 3. BEN2 Base FP32:
//    - Upstream weights: https://huggingface.co/PramaLLC/BEN2 (MIT, BEN2_Base)
//    - FP32 ONNX conversion: https://huggingface.co/Pezhgorski/BEN2-FP32-ONNX (Apache-2.0)
//    - Real byte length: 422,457,868 bytes
//    - SHA-256: 9a8145c1c7ab73fcb265cb5feaab16e0a8c1986265e1a301cf0edd95103b99c9
pub static SPEC_MAXIMUM: ModelSpec = ModelSpec {
    tier: ModelTier::Maximum,
    id: "ben2_fp32",
    name: "BEN2 Base FP32 (Maximum Tier)",
    default_filename: "ben2_fp32.onnx",
    expected_sha256: "9a8145c1c7ab73fcb265cb5feaab16e0a8c1986265e1a301cf0edd95103b99c9",
    expected_size: 422_457_868,
    license: "Apache-2.0 (FP32 ONNX) / MIT (BEN2 Base)",
    source_description:
        "BEN2 Base FP32 ONNX conversion from HuggingFace (Pezhgorski/BEN2-FP32-ONNX; upstream PramaLLC/BEN2)",
    source_urls: &[
        "https://huggingface.co/Pezhgorski/BEN2-FP32-ONNX/resolve/main/onnx/model.onnx",
    ],
    width: 1024,
    height: 1024,
    input_name: "pixel_values",
    output_name: "alphas",
    kind: ModelKind::Ben2,
    mean: [0.485, 0.456, 0.406],
    std: [0.229, 0.224, 0.225],
};

/// Returns the static specification for the chosen tier.
#[must_use]
pub fn spec_for_tier(tier: ModelTier) -> &'static ModelSpec {
    match tier {
        ModelTier::Quality => &SPEC_QUALITY,
        ModelTier::Fast => &SPEC_FAST,
        ModelTier::Maximum => &SPEC_MAXIMUM,
    }
}

/// Returns the default OS cache directory for RustMatte models.
///
/// Locations:
/// - Windows: `%LOCALAPPDATA%\rustmatte\models`
/// - macOS: `~/Library/Application Support/rustmatte/models`
/// - Linux: `$XDG_DATA_HOME/rustmatte/models` or `~/.local/share/rustmatte/models`
#[must_use]
pub fn default_cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|d| d.join("rustmatte").join("models"))
        .unwrap_or_else(|| PathBuf::from("models"))
}

/// Model presence and integrity status in local filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    /// Present in local storage and SHA-256 verified.
    Verified {
        /// Absolute or relative path to the verified model file.
        path: PathBuf,
    },
    /// Present in local storage but corrupted or modified.
    Corrupt {
        /// Path to corrupted file.
        path: PathBuf,
        /// Expected SHA-256 hash.
        expected: String,
        /// Actual computed SHA-256 hash.
        actual: String,
    },
    /// Absent from all lookup locations.
    Missing {
        /// Expected path in the OS cache directory.
        expected_cache_path: PathBuf,
    },
}

/// Computes the SHA-256 hash of a file on disk.
pub fn compute_file_sha256(path: &Path) -> Result<String, CoreError> {
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

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies that a local model file matches its expected SHA-256 checksum.
pub fn verify_model_checksum(path: &Path, expected_hex: &str) -> Result<(), CoreError> {
    let actual_hex = compute_file_sha256(path)?;
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

/// Checks the presence and integrity of a model tier across the hierarchical lookup chain.
///
/// Hierarchy:
/// 1. Custom path (if specified)
/// 2. `RUSTMATTE_MODEL_DIR` environment variable
/// 3. OS cache directory (`default_cache_dir()`)
/// 4. Local `./models/` directory
#[must_use]
pub fn check_model_status_with_custom(custom_path: Option<&Path>, tier: ModelTier) -> ModelStatus {
    let spec = spec_for_tier(tier);

    // 1. Explicit custom path
    if let Some(custom) = custom_path {
        if custom.exists() {
            return match compute_file_sha256(custom) {
                Ok(actual) => {
                    if actual.eq_ignore_ascii_case(spec.expected_sha256) {
                        ModelStatus::Verified {
                            path: custom.to_path_buf(),
                        }
                    } else {
                        ModelStatus::Corrupt {
                            path: custom.to_path_buf(),
                            expected: spec.expected_sha256.to_string(),
                            actual,
                        }
                    }
                }
                Err(_) => ModelStatus::Missing {
                    expected_cache_path: custom.to_path_buf(),
                },
            };
        }
        return ModelStatus::Missing {
            expected_cache_path: custom.to_path_buf(),
        };
    }

    // 2. Environment variable RUSTMATTE_MODEL_DIR
    if let Ok(env_dir) = std::env::var("RUSTMATTE_MODEL_DIR") {
        let p = PathBuf::from(env_dir).join(spec.default_filename);
        if p.exists() {
            return match compute_file_sha256(&p) {
                Ok(actual) => {
                    if actual.eq_ignore_ascii_case(spec.expected_sha256) {
                        ModelStatus::Verified { path: p }
                    } else {
                        ModelStatus::Corrupt {
                            path: p,
                            expected: spec.expected_sha256.to_string(),
                            actual,
                        }
                    }
                }
                Err(_) => ModelStatus::Missing {
                    expected_cache_path: p,
                },
            };
        }
    }

    // 3. OS cache directory
    let cache_path = default_cache_dir().join(spec.default_filename);
    if cache_path.exists() {
        return match compute_file_sha256(&cache_path) {
            Ok(actual) => {
                if actual.eq_ignore_ascii_case(spec.expected_sha256) {
                    ModelStatus::Verified { path: cache_path }
                } else {
                    ModelStatus::Corrupt {
                        path: cache_path,
                        expected: spec.expected_sha256.to_string(),
                        actual,
                    }
                }
            }
            Err(_) => ModelStatus::Missing {
                expected_cache_path: cache_path,
            },
        };
    }

    // 4. Local ./models/ directory
    let local_path = PathBuf::from("models").join(spec.default_filename);
    if local_path.exists() {
        return match compute_file_sha256(&local_path) {
            Ok(actual) => {
                if actual.eq_ignore_ascii_case(spec.expected_sha256) {
                    ModelStatus::Verified { path: local_path }
                } else {
                    ModelStatus::Corrupt {
                        path: local_path,
                        expected: spec.expected_sha256.to_string(),
                        actual,
                    }
                }
            }
            Err(_) => ModelStatus::Missing {
                expected_cache_path: cache_path,
            },
        };
    }

    ModelStatus::Missing {
        expected_cache_path: cache_path,
    }
}

/// Checks the presence and integrity of a model tier using standard lookup order.
#[must_use]
pub fn check_model_status(tier: ModelTier) -> ModelStatus {
    check_model_status_with_custom(None, tier)
}

/// Resolves the model path across the lookup chain and verifies its SHA-256 checksum.
///
/// Fails with `CoreError::ModelChecksumMismatch` if a file is found but corrupted.
/// Fails with `CoreError::ModelNotFound` if no file is found.
pub fn resolve_model_path(
    custom_path: Option<&Path>,
    tier: ModelTier,
) -> Result<PathBuf, CoreError> {
    match check_model_status_with_custom(custom_path, tier) {
        ModelStatus::Verified { path } => Ok(path),
        ModelStatus::Corrupt {
            path,
            expected,
            actual,
        } => Err(CoreError::ModelChecksumMismatch {
            path,
            expected,
            actual,
        }),
        ModelStatus::Missing {
            expected_cache_path,
        } => Err(CoreError::ModelNotFound {
            path: expected_cache_path,
            tier,
        }),
    }
}

/// Deletes a cached model (and any partial download files) inside `cache_dir`, returning `(deleted_path, freed_bytes)`.
pub fn delete_model_in_dir(
    spec: &ModelSpec,
    cache_dir: &Path,
) -> Result<Option<(PathBuf, u64)>, CoreError> {
    let target = cache_dir.join(spec.default_filename);
    let mut freed = 0u64;
    let mut removed_path = None;

    if target.exists() {
        let len = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        std::fs::remove_file(&target)?;
        freed += len;
        removed_path = Some(target);
    }

    let part = cache_dir.join(format!(".{}.part", spec.default_filename));
    if part.exists() {
        let len = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_file(&part);
        freed += len;
        if removed_path.is_none() {
            removed_path = Some(part);
        }
    }

    let meta = cache_dir.join(format!(".{}.part.meta.json", spec.default_filename));
    if meta.exists() {
        let _ = std::fs::remove_file(&meta);
    }

    Ok(removed_path.map(|p| (p, freed)))
}

/// Deletes a cached model (and partial download state) from `RUSTMATTE_MODEL_DIR` (if set) and `default_cache_dir()`.
pub fn delete_cached_model(tier: ModelTier) -> Result<Option<(PathBuf, u64)>, CoreError> {
    let spec = spec_for_tier(tier);

    if let Ok(env_dir) = std::env::var("RUSTMATTE_MODEL_DIR") {
        let dir = PathBuf::from(env_dir);
        if let Some(res) = delete_model_in_dir(spec, &dir)? {
            return Ok(Some(res));
        }
    }

    delete_model_in_dir(spec, &default_cache_dir())
}
