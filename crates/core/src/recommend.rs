//! Pure model recommendation engine (`PRD_V2.md` §7.4).
//!
//! No side effects, no network calls, and no persistent storage of wizard answers or RAM values.

use serde::{Deserialize, Serialize};

use crate::manifest::{MODEL_ID_BALANCED_STR, MODEL_ID_FAST_STR};

/// Named available-RAM threshold below which Balanced is overridden to Fast (`3 GiB`).
///
/// Validated against Phase 5.0 24 MP measurement (`2,191.39 MB` Peak Working Set /
/// `2,571.77 MB` Peak Commit for Balanced with standard refinement): below `3,072 MB`
/// available RAM, Balanced leaves minimal OS headroom and risks swapping.
pub const LOW_RAM_THRESHOLD_MB: u64 = 3072;

/// Named available-RAM threshold below which Balanced emits a memory-use note (`5 GiB`).
pub const MID_RAM_THRESHOLD_MB: u64 = 5120;

/// User's expected usage frequency from Wizard Question 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageFrequency {
    /// "Occasionally, a few photos"
    Occasional,
    /// "Fairly often, dozens at a time"
    FairlyOften,
    /// "Regular work, hundreds of photos"
    RegularWork,
    /// "Not sure yet"
    NotSure,
}

/// User's primary photo subject category from Wizard Question 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotoSubject {
    /// "People or portraits"
    PeopleOrPortraits,
    /// "Products on plain backgrounds"
    ProductsPlainBg,
    /// "Animals, hair, or fur"
    AnimalsHairOrFur,
    /// "Mixed or complex photos"
    MixedOrComplex,
}

/// Dictionary-key reason code for the primary model recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationReason {
    /// Occasional usage -> Balanced quality default.
    OccasionalBalanced,
    /// Frequent/regular batches of portraits or plain-background products -> Fast throughput.
    BatchSimpleFast,
    /// Frequent/regular batches with hair, fur, or complex scenes -> Balanced detail.
    BatchDetailBalanced,
    /// User unsure -> Balanced all-rounder.
    DefaultBalanced,
    /// Low available RAM (< 3 GiB) overrode Balanced recommendation to Fast.
    LowRamFastOverride,
}

/// Memory advisory level derived from local available RAM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RamAdvisory {
    /// Available RAM is >= `MID_RAM_THRESHOLD_MB` (5 GiB): no memory note needed.
    None,
    /// Available RAM is between `LOW_RAM_THRESHOLD_MB` (3 GiB) and `MID_RAM_THRESHOLD_MB` (5 GiB).
    MidRamNote,
    /// Available RAM is below `LOW_RAM_THRESHOLD_MB` (3 GiB): warn if user selects Balanced.
    LowRamWarning,
    /// RAM detection failed or unavailable: neutral note, no false warning.
    UnknownRam,
}

/// Pure output of the recommendation engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelRecommendation {
    /// Recommended primary manifest model ID (`"balanced"` or `"fast"`).
    pub primary_id: &'static str,
    /// Alternative manifest model ID.
    pub alternative_id: &'static str,
    /// True if `primary_id` is already installed locally.
    pub primary_installed: bool,
    /// True if `alternative_id` is already installed locally.
    pub alternative_installed: bool,
    /// Reason code translated by the UI dictionary.
    pub reason_code: RecommendationReason,
    /// Memory advisory state for UI display.
    pub ram_advisory: RamAdvisory,
    /// True if low RAM caused a Balanced table choice to be overridden to Fast.
    pub ram_overridden_to_fast: bool,
}

fn is_installed(installed_model_ids: &[&str], target_id: &str) -> bool {
    installed_model_ids
        .iter()
        .any(|id| id.trim().eq_ignore_ascii_case(target_id))
}

/// Computes the primary/alternative model recommendation from wizard answers, optional
/// available RAM (`MiB`), and currently installed model IDs (`PRD_V2.md` §7.4).
#[must_use]
pub fn recommend_model(
    usage: UsageFrequency,
    photo_type: PhotoSubject,
    available_ram_mb: Option<u64>,
    installed_model_ids: &[&str],
) -> ModelRecommendation {
    let (table_primary, table_alt, table_reason) = match (usage, photo_type) {
        (UsageFrequency::Occasional, _) => (
            MODEL_ID_BALANCED_STR,
            MODEL_ID_FAST_STR,
            RecommendationReason::OccasionalBalanced,
        ),
        (UsageFrequency::NotSure, _) => (
            MODEL_ID_BALANCED_STR,
            MODEL_ID_FAST_STR,
            RecommendationReason::DefaultBalanced,
        ),
        (
            UsageFrequency::FairlyOften | UsageFrequency::RegularWork,
            PhotoSubject::PeopleOrPortraits | PhotoSubject::ProductsPlainBg,
        ) => (
            MODEL_ID_FAST_STR,
            MODEL_ID_BALANCED_STR,
            RecommendationReason::BatchSimpleFast,
        ),
        (
            UsageFrequency::FairlyOften | UsageFrequency::RegularWork,
            PhotoSubject::AnimalsHairOrFur | PhotoSubject::MixedOrComplex,
        ) => (
            MODEL_ID_BALANCED_STR,
            MODEL_ID_FAST_STR,
            RecommendationReason::BatchDetailBalanced,
        ),
    };

    let (primary_id, alternative_id, reason_code, ram_advisory, ram_overridden_to_fast) =
        match available_ram_mb {
            Some(ram) if ram < LOW_RAM_THRESHOLD_MB => {
                if table_primary == MODEL_ID_BALANCED_STR {
                    (
                        MODEL_ID_FAST_STR,
                        MODEL_ID_BALANCED_STR,
                        RecommendationReason::LowRamFastOverride,
                        RamAdvisory::LowRamWarning,
                        true,
                    )
                } else {
                    (
                        table_primary,
                        table_alt,
                        table_reason,
                        RamAdvisory::LowRamWarning,
                        false,
                    )
                }
            }
            Some(ram) if ram < MID_RAM_THRESHOLD_MB => (
                table_primary,
                table_alt,
                table_reason,
                RamAdvisory::MidRamNote,
                false,
            ),
            Some(_) => (
                table_primary,
                table_alt,
                table_reason,
                RamAdvisory::None,
                false,
            ),
            None => (
                table_primary,
                table_alt,
                table_reason,
                RamAdvisory::UnknownRam,
                false,
            ),
        };

    ModelRecommendation {
        primary_id,
        alternative_id,
        primary_installed: is_installed(installed_model_ids, primary_id),
        alternative_installed: is_installed(installed_model_ids, alternative_id),
        reason_code,
        ram_advisory,
        ram_overridden_to_fast,
    }
}

/// Runs `recommend_model` using an injectable RAM-probe function (for CI tests).
pub fn recommend_model_with_probe<F>(
    usage: UsageFrequency,
    photo_type: PhotoSubject,
    installed_model_ids: &[&str],
    ram_probe: F,
) -> ModelRecommendation
where
    F: FnOnce() -> Option<u64>,
{
    recommend_model(usage, photo_type, ram_probe(), installed_model_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIGH_RAM: Option<u64> = Some(8192);

    #[test]
    fn test_recommendation_table_all_rows() {
        let subjects = [
            PhotoSubject::PeopleOrPortraits,
            PhotoSubject::ProductsPlainBg,
            PhotoSubject::AnimalsHairOrFur,
            PhotoSubject::MixedOrComplex,
        ];

        // Row 1: Occasionally, any -> Balanced / Fast
        for s in subjects {
            let rec = recommend_model(UsageFrequency::Occasional, s, HIGH_RAM, &[]);
            assert_eq!(rec.primary_id, "balanced");
            assert_eq!(rec.alternative_id, "fast");
            assert_eq!(rec.reason_code, RecommendationReason::OccasionalBalanced);
            assert_eq!(rec.ram_advisory, RamAdvisory::None);
        }

        // Rows 2-5: Fairly often
        let r2 = recommend_model(
            UsageFrequency::FairlyOften,
            PhotoSubject::PeopleOrPortraits,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r2.primary_id, r2.alternative_id), ("fast", "balanced"));
        assert_eq!(r2.reason_code, RecommendationReason::BatchSimpleFast);

        let r3 = recommend_model(
            UsageFrequency::FairlyOften,
            PhotoSubject::ProductsPlainBg,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r3.primary_id, r3.alternative_id), ("fast", "balanced"));

        let r4 = recommend_model(
            UsageFrequency::FairlyOften,
            PhotoSubject::AnimalsHairOrFur,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r4.primary_id, r4.alternative_id), ("balanced", "fast"));
        assert_eq!(r4.reason_code, RecommendationReason::BatchDetailBalanced);

        let r5 = recommend_model(
            UsageFrequency::FairlyOften,
            PhotoSubject::MixedOrComplex,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r5.primary_id, r5.alternative_id), ("balanced", "fast"));

        // Rows 6-9: Regular work
        let r6 = recommend_model(
            UsageFrequency::RegularWork,
            PhotoSubject::PeopleOrPortraits,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r6.primary_id, r6.alternative_id), ("fast", "balanced"));

        let r7 = recommend_model(
            UsageFrequency::RegularWork,
            PhotoSubject::ProductsPlainBg,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r7.primary_id, r7.alternative_id), ("fast", "balanced"));

        let r8 = recommend_model(
            UsageFrequency::RegularWork,
            PhotoSubject::AnimalsHairOrFur,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r8.primary_id, r8.alternative_id), ("balanced", "fast"));

        let r9 = recommend_model(
            UsageFrequency::RegularWork,
            PhotoSubject::MixedOrComplex,
            HIGH_RAM,
            &[],
        );
        assert_eq!((r9.primary_id, r9.alternative_id), ("balanced", "fast"));

        // Row 10: Not sure yet, any -> Balanced / Fast
        for s in subjects {
            let rec = recommend_model(UsageFrequency::NotSure, s, HIGH_RAM, &[]);
            assert_eq!(rec.primary_id, "balanced");
            assert_eq!(rec.alternative_id, "fast");
            assert_eq!(rec.reason_code, RecommendationReason::DefaultBalanced);
        }
    }

    #[test]
    fn test_ram_overrides_and_installed_flags() {
        // Low RAM (< 3 GB) overrides Balanced -> Fast
        let low_override = recommend_model(
            UsageFrequency::Occasional,
            PhotoSubject::AnimalsHairOrFur,
            Some(2500),
            &["fast"],
        );
        assert_eq!(low_override.primary_id, "fast");
        assert_eq!(low_override.alternative_id, "balanced");
        assert_eq!(
            low_override.reason_code,
            RecommendationReason::LowRamFastOverride
        );
        assert_eq!(low_override.ram_advisory, RamAdvisory::LowRamWarning);
        assert!(low_override.ram_overridden_to_fast);
        assert!(low_override.primary_installed);
        assert!(!low_override.alternative_installed);

        // Low RAM keeps Fast when table already picked Fast
        let low_fast = recommend_model(
            UsageFrequency::RegularWork,
            PhotoSubject::ProductsPlainBg,
            Some(2048),
            &[],
        );
        assert_eq!(low_fast.primary_id, "fast");
        assert_eq!(low_fast.ram_advisory, RamAdvisory::LowRamWarning);
        assert!(!low_fast.ram_overridden_to_fast);

        // Mid RAM (3..5 GB) keeps table choice with MidRamNote
        let mid = recommend_model(
            UsageFrequency::Occasional,
            PhotoSubject::MixedOrComplex,
            Some(4096),
            &["balanced", "fast"],
        );
        assert_eq!(mid.primary_id, "balanced");
        assert_eq!(mid.ram_advisory, RamAdvisory::MidRamNote);
        assert!(mid.primary_installed);
        assert!(mid.alternative_installed);

        // Unknown RAM emits UnknownRam with no false warning and preserves table choice
        let unknown = recommend_model_with_probe(
            UsageFrequency::Occasional,
            PhotoSubject::AnimalsHairOrFur,
            &[],
            || None,
        );
        assert_eq!(unknown.primary_id, "balanced");
        assert_eq!(unknown.ram_advisory, RamAdvisory::UnknownRam);
        assert!(!unknown.ram_overridden_to_fast);
    }
}
