export type WizardStep = 1 | 2 | 3 | 4 | 5;

export type UsageFrequency =
  | "occasional"
  | "fairly_often"
  | "regular_work"
  | "not_sure";

export type PhotoSubject =
  | "people_or_portraits"
  | "products_plain_bg"
  | "animals_hair_or_fur"
  | "mixed_or_complex";

export type WizardModelId = "balanced" | "fast";

export type RecommendationReason =
  | "occasional_balanced"
  | "batch_simple_fast"
  | "batch_detail_balanced"
  | "default_balanced"
  | "low_ram_fast_override";

export type RamAdvisory =
  | "none"
  | "mid_ram_note"
  | "low_ram_warning"
  | "unknown_ram";

export interface DiskSpaceCheck {
  required_bytes: number;
  available_bytes: number;
  sufficient: boolean;
}

export interface ManifestModelInfo {
  id: WizardModelId;
  tier: "quality" | "fast";
  engine: string;
  filename: string;
  size_bytes: number;
  expected_size_mb: number;
  sha256: string;
  license_weights: string;
  data_note: string;
  primary_host: string;
  primary_url: string;
  fallback_host: string;
  fallback_url: string;
  measured_hardware: string;
  measured_seconds: number;
  measured_peak_ram_mb: number;
  strengths: string[];
  weaknesses: string[];
  recommended_for: string[];
  min_available_ram_mb: number;
  cached: boolean;
  partial_bytes: number;
  disk_check: DiskSpaceCheck | null;
}

export interface ModelRecommendation {
  primary_id: WizardModelId;
  alternative_id: WizardModelId;
  primary_installed: boolean;
  alternative_installed: boolean;
  reason_code: RecommendationReason;
  ram_advisory: RamAdvisory;
  ram_overridden_to_fast: boolean;
}

export interface SetupRecommendationResponse {
  recommendation: ModelRecommendation;
  available_ram_mb: number | null;
}

export interface WizardState {
  step: WizardStep;
  usage: UsageFrequency | null;
  photoType: PhotoSubject | null;
  selectedModelId: WizardModelId | null;
  showAllModels: boolean;
}

export function createInitialWizardState(): WizardState {
  return {
    step: 1,
    usage: null,
    photoType: null,
    selectedModelId: null,
    showAllModels: false,
  };
}

export function canAdvanceWizard(
  state: WizardState,
  selectedModelCached: boolean
): boolean {
  switch (state.step) {
    case 1:
      return true;
    case 2:
      return state.usage !== null && state.photoType !== null;
    case 3:
      return state.selectedModelId !== null;
    case 4:
      return state.selectedModelId !== null && selectedModelCached;
    case 5:
      return state.selectedModelId !== null && selectedModelCached;
  }
}

export function nextWizardStep(
  state: WizardState,
  selectedModelCached: boolean
): WizardState {
  if (!canAdvanceWizard(state, selectedModelCached) || state.step >= 5) {
    return state;
  }
  return {
    ...state,
    step: (state.step + 1) as WizardStep,
  };
}

export function prevWizardStep(state: WizardState): WizardState {
  if (state.step <= 1) {
    return state;
  }
  return {
    ...state,
    step: (state.step - 1) as WizardStep,
  };
}

export function resetWizard(): WizardState {
  return createInitialWizardState();
}

export function canCompleteWizard(
  state: WizardState,
  selectedModelCached: boolean
): boolean {
  return (
    state.step === 5 &&
    state.selectedModelId !== null &&
    selectedModelCached
  );
}

export function selectWizardModel(
  state: WizardState,
  modelId: WizardModelId
): WizardState {
  return {
    ...state,
    selectedModelId: modelId,
  };
}
