//! Model manifest registry, metadata specifications, and schema validation.

use serde::{Deserialize, Serialize};

use crate::model::{ModelKind, ModelTier};

/// Canonical ID for the Balanced profile (`BiRefNet Lite 512`).
pub const MODEL_ID_BALANCED_STR: &str = "balanced";
/// Canonical ID for the Fast profile (`U-2-Netp`).
pub const MODEL_ID_FAST_STR: &str = "fast";
/// Canonical ID for the Maximum profile (`BEN2 Base FP32`).
pub const MODEL_ID_MAXIMUM_STR: &str = "maximum";

/// Stability and visibility lifecycle status for a manifest entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatusKind {
    /// Production-ready default tier.
    Stable,
    /// Opt-in experimental entry.
    Experimental,
    /// Hidden internal or deprecated entry.
    Hidden,
}

/// Output tensor activation / normalization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationKind {
    /// Sigmoid logits (`1 / (1 + exp(-x))`).
    Sigmoid,
    /// Min-max linear scaling across output tensor values.
    MinMax,
}

/// Model tensor preprocessing and output contract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ModelInputSpec {
    /// Square input width/height in pixels.
    pub size: u32,
    /// RGB channel normalization means.
    pub mean: [f32; 3],
    /// RGB channel normalization standard deviations.
    pub std: [f32; 3],
    /// ONNX input tensor name.
    pub input_name: &'static str,
    /// ONNX output tensor name.
    pub output_name: &'static str,
    /// Output postprocessing activation rule.
    pub activation: ActivationKind,
}

/// Measured CPU runtime characteristics on reference hardware.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MeasuredSpec {
    /// Reference hardware label.
    pub hardware: &'static str,
    /// Measured median seconds per standard fixture photo.
    pub seconds_per_photo: f32,
    /// Measured peak working set RAM in MiB on standard fixture.
    pub peak_ram_mb: u64,
}

/// Dictionary-key tags for UI model cards (translated by UI dictionaries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProfileTags {
    /// Strength tag codes.
    pub strengths: &'static [&'static str],
    /// Known limitation / weakness tag codes.
    pub weaknesses: &'static [&'static str],
    /// Recommended workload tag codes.
    pub recommended_for: &'static [&'static str],
}

/// Ordered download mirror entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ModelSource {
    /// Source key (e.g., `"github-release"`, `"upstream-mirror"`).
    pub key: &'static str,
    /// Direct HTTPS download URL.
    pub url: &'static str,
}

/// Complete manifest entry describing a supported offline AI model (`PRD_V2.md` §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ModelManifestEntry {
    /// Canonical manifest ID (`"balanced"` or `"fast"`).
    pub id: &'static str,
    /// Associated core tier enum.
    pub tier: ModelTier,
    /// Engine / architecture identifier (`"birefnet_lite_512"` or `"u2netp"`).
    pub engine: &'static str,
    /// Output semantic kind.
    pub kind: ModelKind,
    /// Local filename inside model cache.
    pub file_name: &'static str,
    /// Exact expected file size in bytes.
    pub size_bytes: u64,
    /// Exact pinned SHA-256 digest in lowercase hex.
    pub sha256: &'static str,
    /// Ordered download sources (primary first, fallback mirror next).
    pub sources: &'static [ModelSource],
    /// Weight license identifier (SPDX).
    pub license_weights: &'static str,
    /// Training dataset provenance dictionary key.
    pub data_note: &'static str,
    /// Short upstream export description.
    pub source_description: &'static str,
    /// Release lifecycle status.
    pub status: ModelStatusKind,
    /// Input/output tensor contract.
    pub input: ModelInputSpec,
    /// Reference benchmark data.
    pub measured: MeasuredSpec,
    /// Card tag keys.
    pub profile: ProfileTags,
    /// Minimum recommended available RAM in MiB.
    pub min_available_ram_mb: u64,
}

/// Balanced profile (`BiRefNet Lite 512`, Quality tier).
pub static MANIFEST_BALANCED: ModelManifestEntry = ModelManifestEntry {
    id: "balanced",
    tier: ModelTier::Quality,
    engine: "birefnet_lite_512",
    kind: ModelKind::BiRefNet,
    file_name: "birefnet_lite_512.onnx",
    size_bytes: 191_877_254,
    sha256: "1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2",
    sources: &[
        ModelSource {
            key: "github-release",
            url: "https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/birefnet_lite_512.onnx",
        },
        ModelSource {
            key: "upstream-mirror",
            url: "https://huggingface.co/studioludens/birefnet-lite-512/resolve/main/onnx/model.onnx",
        },
    ],
    license_weights: "MIT",
    data_note: "academic-datasets",
    source_description:
        "BiRefNet_lite 512 FP32 ONNX export from HuggingFace (studioludens/birefnet-lite-512)",
    status: ModelStatusKind::Stable,
    input: ModelInputSpec {
        size: 512,
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
        input_name: "input_image",
        output_name: "output_image",
        activation: ActivationKind::Sigmoid,
    },
    measured: MeasuredSpec {
        hardware: "i5-10400, 6C/12T, 16 GB",
        seconds_per_photo: 3.1,
        peak_ram_mb: 2030,
    },
    profile: ProfileTags {
        strengths: &["hair_fur", "general"],
        weaknesses: &["closes_small_gaps", "larger_download"],
        recommended_for: &["occasional", "hair_fur", "mixed"],
    },
    min_available_ram_mb: 3072,
};

/// Fast profile (`U-2-Netp`, Fast tier).
pub static MANIFEST_FAST: ModelManifestEntry = ModelManifestEntry {
    id: "fast",
    tier: ModelTier::Fast,
    engine: "u2netp",
    kind: ModelKind::U2Netp,
    file_name: "u2netp.onnx",
    size_bytes: 4_574_861,
    sha256: "309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8",
    sources: &[
        ModelSource {
            key: "github-release",
            url: "https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/u2netp.onnx",
        },
        ModelSource {
            key: "upstream-mirror",
            url: "https://huggingface.co/edgetools/u2netp/resolve/main/u2netp.onnx",
        },
    ],
    license_weights: "Apache-2.0",
    data_note: "academic-datasets",
    source_description: "U-2-Netp 320 FP32 ONNX export from HuggingFace (edgetools/u2netp)",
    status: ModelStatusKind::Stable,
    input: ModelInputSpec {
        size: 320,
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
        input_name: "input.1",
        output_name: "1959",
        activation: ActivationKind::MinMax,
    },
    measured: MeasuredSpec {
        hardware: "i5-10400, 6C/12T, 16 GB",
        seconds_per_photo: 0.33,
        peak_ram_mb: 333,
    },
    profile: ProfileTags {
        strengths: &["very_fast", "low_ram", "tiny_download"],
        weaknesses: &["fur_smears", "fine_hair"],
        recommended_for: &["batch_people", "batch_products", "low_ram"],
    },
    min_available_ram_mb: 512,
};

/// Maximum profile (`BEN2 Base FP32`, Maximum tier).
pub static MANIFEST_MAXIMUM: ModelManifestEntry = ModelManifestEntry {
    id: "maximum",
    tier: ModelTier::Maximum,
    engine: "ben2_fp32",
    kind: ModelKind::Ben2,
    file_name: "ben2_fp32.onnx",
    size_bytes: 422_457_868,
    sha256: "9a8145c1c7ab73fcb265cb5feaab16e0a8c1986265e1a301cf0edd95103b99c9",
    sources: &[ModelSource {
        key: "upstream-mirror",
        url: "https://huggingface.co/Pezhgorski/BEN2-FP32-ONNX/resolve/main/onnx/model.onnx",
    }],
    license_weights: "Apache-2.0 / MIT",
    data_note: "dis5k-plus-proprietary-22k",
    source_description:
        "BEN2 Base FP32 ONNX conversion from HuggingFace (Pezhgorski/BEN2-FP32-ONNX; upstream PramaLLC/BEN2)",
    status: ModelStatusKind::Stable,
    input: ModelInputSpec {
        size: 1024,
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
        input_name: "pixel_values",
        output_name: "alphas",
        activation: ActivationKind::MinMax,
    },
    measured: MeasuredSpec {
        hardware: "i5-10400, 6C/12T, 16 GB",
        seconds_per_photo: 14.0,
        peak_ram_mb: 2760,
    },
    profile: ProfileTags {
        strengths: &["hair_fur", "general"],
        weaknesses: &["slow_cpu", "high_ram", "larger_download"],
        recommended_for: &["occasional", "hair_fur", "mixed"],
    },
    min_available_ram_mb: 4096,
};

static ALL_MANIFEST_ENTRIES: [&ModelManifestEntry; 3] =
    [&MANIFEST_BALANCED, &MANIFEST_FAST, &MANIFEST_MAXIMUM];

/// Returns all registered model manifest entries in stable presentation order.
#[must_use]
pub fn manifest_entries() -> &'static [&'static ModelManifestEntry] {
    &ALL_MANIFEST_ENTRIES
}

/// Looks up a manifest entry by tier.
#[must_use]
pub fn manifest_entry_for_tier(tier: ModelTier) -> &'static ModelManifestEntry {
    match tier {
        ModelTier::Quality => &MANIFEST_BALANCED,
        ModelTier::Fast => &MANIFEST_FAST,
        ModelTier::Maximum => &MANIFEST_MAXIMUM,
    }
}

/// Looks up a manifest entry by manifest ID, legacy tier name, or engine ID.
#[must_use]
pub fn find_manifest_entry(id_or_engine: &str) -> Option<&'static ModelManifestEntry> {
    let key = id_or_engine.trim().to_ascii_lowercase();
    match key.as_str() {
        "balanced" | "quality" | "birefnet_lite_512" => Some(&MANIFEST_BALANCED),
        "fast" | "u2netp" => Some(&MANIFEST_FAST),
        "maximum" | "ben2" | "ben2_fp32" | "ben2_base" => Some(&MANIFEST_MAXIMUM),
        _ => None,
    }
}

/// Validates a slice of manifest entries for integrity, uniqueness, and required fields.
pub fn validate_manifest(entries: &[&ModelManifestEntry]) -> Result<(), String> {
    if entries.is_empty() {
        return Err("Manifest must contain at least one model entry".to_string());
    }

    let mut seen_ids = std::collections::HashSet::new();
    let mut seen_engines = std::collections::HashSet::new();

    for entry in entries {
        if entry.id.trim().is_empty() {
            return Err("Manifest entry has empty id".to_string());
        }
        if !seen_ids.insert(entry.id) {
            return Err(format!("Duplicate manifest id '{}'", entry.id));
        }
        if entry.engine.trim().is_empty() {
            return Err(format!("Manifest '{}' has empty engine", entry.id));
        }
        if !seen_engines.insert(entry.engine) {
            return Err(format!("Duplicate manifest engine '{}'", entry.engine));
        }
        if entry.file_name.trim().is_empty() {
            return Err(format!("Manifest '{}' has empty file_name", entry.id));
        }
        if entry.size_bytes == 0 {
            return Err(format!("Manifest '{}' has zero size_bytes", entry.id));
        }
        if entry.sha256.len() != 64 || !entry.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "Manifest '{}' has invalid SHA-256 digest '{}'",
                entry.id, entry.sha256
            ));
        }
        if entry.sources.is_empty() {
            return Err(format!("Manifest '{}' has no download sources", entry.id));
        }
        for src in entry.sources {
            if src.key.trim().is_empty() || !src.url.starts_with("https://") {
                return Err(format!(
                    "Manifest '{}' has invalid source '{}' ({})",
                    entry.id, src.key, src.url
                ));
            }
        }
        if entry.license_weights.trim().is_empty() || entry.data_note.trim().is_empty() {
            return Err(format!(
                "Manifest '{}' missing license_weights or data_note",
                entry.id
            ));
        }
        if entry.input.size == 0
            || entry.input.input_name.trim().is_empty()
            || entry.input.output_name.trim().is_empty()
        {
            return Err(format!("Manifest '{}' has invalid input spec", entry.id));
        }
        if entry.measured.hardware.trim().is_empty()
            || entry.measured.seconds_per_photo <= 0.0
            || entry.measured.peak_ram_mb == 0
        {
            return Err(format!("Manifest '{}' has invalid measured spec", entry.id));
        }
        if entry.profile.strengths.is_empty()
            || entry.profile.weaknesses.is_empty()
            || entry.profile.recommended_for.is_empty()
        {
            return Err(format!("Manifest '{}' has empty profile tags", entry.id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_manifest_is_valid() {
        assert_eq!(validate_manifest(manifest_entries()), Ok(()));
    }

    #[test]
    fn test_manifest_lookup_aliases() {
        assert_eq!(
            find_manifest_entry("balanced").map(|e| e.engine),
            Some("birefnet_lite_512")
        );
        assert_eq!(
            find_manifest_entry("Quality").map(|e| e.id),
            Some("balanced")
        );
        assert_eq!(find_manifest_entry("u2netp").map(|e| e.id), Some("fast"));
        assert!(find_manifest_entry("unknown_model").is_none());
    }

    #[test]
    fn test_validate_manifest_rejects_bad_hash_and_zero_size() {
        let mut bad = MANIFEST_FAST;
        bad.sha256 = "not_a_valid_sha";
        assert!(validate_manifest(&[&bad]).is_err());

        let mut zero = MANIFEST_FAST;
        zero.size_bytes = 0;
        assert!(validate_manifest(&[&zero]).is_err());

        let dup = [&MANIFEST_FAST, &MANIFEST_FAST];
        assert!(validate_manifest(&dup).is_err());
    }
}
