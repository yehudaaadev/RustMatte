<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { FriendlyError } from "$lib/errors";
  import type { Lang, Translations } from "$lib/i18n";
  import {
    canAdvanceWizard,
    canCompleteWizard,
    createInitialWizardState,
    nextWizardStep,
    prevWizardStep,
    selectWizardModel,
    type ManifestModelInfo,
    type PhotoSubject,
    type RamAdvisory,
    type RecommendationReason,
    type SetupRecommendationResponse,
    type UsageFrequency,
    type WizardModelId,
    type WizardState,
    type WizardStep,
  } from "$lib/wizard";

  interface Props {
    open: boolean;
    lang: Lang;
    t: Translations;
    neverUseNetwork: boolean;
    manifestModels: ManifestModelInfo[];
    isDownloading: boolean;
    downloadProgress: number;
    downloadError: FriendlyError | null;
    downloadPaused: boolean;
    canDismiss: boolean;
    initialStep?: WizardStep;
    initialModelId?: WizardModelId | null;
    onLanguageChange: (nextLang: Lang) => void;
    onRefreshManifest: () => Promise<void>;
    onDownload: (tier: "quality" | "fast") => Promise<void>;
    onCancelDownload: () => Promise<void>;
    onDeletePartial: (tier: "quality" | "fast") => Promise<void>;
    onImportLocal: (tier: "quality" | "fast") => Promise<void>;
    onComplete: (chosenTier: "quality" | "fast") => void;
    onDismiss: () => void;
  }

  let {
    open,
    lang,
    t,
    neverUseNetwork,
    manifestModels,
    isDownloading,
    downloadProgress,
    downloadError,
    downloadPaused,
    canDismiss,
    initialStep = 1,
    initialModelId = null,
    onLanguageChange,
    onRefreshManifest,
    onDownload,
    onCancelDownload,
    onDeletePartial,
    onImportLocal,
    onComplete,
    onDismiss,
  }: Props = $props();

  let wState = $state<WizardState>(createInitialWizardState());
  let recResponse = $state<SetupRecommendationResponse | null>(null);
  let copiedKey = $state<"link" | "hash" | null>(null);

  $effect(() => {
    if (open) {
      wState = {
        ...createInitialWizardState(),
        step: initialStep,
        selectedModelId: initialModelId,
      };
      recResponse = null;
      copiedKey = null;
    }
  });

  let selectedModel = $derived(
    manifestModels.find((m) => m.id === wState.selectedModelId) ?? null
  );
  let selectedModelCached = $derived(selectedModel?.cached ?? false);
  let canAdvance = $derived(canAdvanceWizard(wState, selectedModelCached));
  let canFinish = $derived(canCompleteWizard(wState, selectedModelCached));
  let diskInsufficient = $derived(
    selectedModel?.disk_check !== null &&
      selectedModel?.disk_check !== undefined &&
      !selectedModel.disk_check.sufficient
  );

  let orderedCardIds = $derived.by((): WizardModelId[] => {
    if (recResponse) {
      return [
        recResponse.recommendation.primary_id,
        recResponse.recommendation.alternative_id,
      ];
    }
    return ["balanced", "fast"];
  });

  function formatMb(bytes: number): string {
    return (bytes / (1024 * 1024)).toFixed(1);
  }

  function stepLabel(step: number): string {
    return t.wizardStepIndicator.replace("{step}", String(step));
  }

  function translateTag(code: string): string {
    switch (code) {
      case "hair_fur":
        return t.cardTagHairFur;
      case "general":
        return t.cardTagGeneral;
      case "very_fast":
        return t.cardTagVeryFast;
      case "low_ram":
        return t.cardTagLowRam;
      case "tiny_download":
        return t.cardTagTinyDownload;
      case "closes_small_gaps":
        return t.cardTagClosesSmallGaps;
      case "larger_download":
        return t.cardTagLargerDownload;
      case "fur_smears":
        return t.cardTagFurSmears;
      case "fine_hair":
        return t.cardTagFineHair;
      default:
        return code;
    }
  }

  function translateReason(reason: RecommendationReason): string {
    switch (reason) {
      case "occasional_balanced":
        return t.wizardReasonOccasionalBalanced;
      case "batch_simple_fast":
        return t.wizardReasonBatchSimpleFast;
      case "batch_detail_balanced":
        return t.wizardReasonBatchDetailBalanced;
      case "low_ram_fast_override":
        return t.wizardReasonLowRamOverride;
      default:
        return t.wizardReasonDefaultBalanced;
    }
  }

  function ramMessage(advisory: RamAdvisory): string | null {
    switch (advisory) {
      case "low_ram_warning":
        return t.wizardRamLowWarning;
      case "mid_ram_note":
        return t.wizardRamMidNote;
      case "unknown_ram":
        return t.wizardRamUnknown;
      default:
        return null;
    }
  }

  async function handleContinue() {
    if (!canAdvance) return;
    if (wState.step === 2 && wState.usage && wState.photoType) {
      await onRefreshManifest();
      try {
        const res = await invoke<SetupRecommendationResponse>(
          "recommend_setup_model",
          {
            usage: wState.usage,
            photoType: wState.photoType,
          }
        );
        recResponse = res;
        wState = selectWizardModel(wState, res.recommendation.primary_id);
      } catch {
        wState = selectWizardModel(wState, "balanced");
      }
    }
    if (wState.step === 3) {
      await onRefreshManifest();
    }
    wState = nextWizardStep(wState, selectedModelCached);
  }

  function handleBack() {
    if (isDownloading) return;
    wState = prevWizardStep(wState);
  }

  function handleFinish() {
    if (!canFinish || !selectedModel) return;
    onComplete(selectedModel.tier);
  }

  async function copyText(kind: "link" | "hash", value: string) {
    try {
      await navigator.clipboard.writeText(value);
      copiedKey = kind;
    } catch {
      copiedKey = null;
    }
  }

  const usageOptions: { value: UsageFrequency; labelKey: keyof Translations }[] =
    [
      { value: "occasional", labelKey: "wizardQ1Occasional" },
      { value: "fairly_often", labelKey: "wizardQ1FairlyOften" },
      { value: "regular_work", labelKey: "wizardQ1RegularWork" },
      { value: "not_sure", labelKey: "wizardQ1NotSure" },
    ];

  const subjectOptions: { value: PhotoSubject; labelKey: keyof Translations }[] =
    [
      { value: "people_or_portraits", labelKey: "wizardQ2People" },
      { value: "products_plain_bg", labelKey: "wizardQ2Products" },
      { value: "animals_hair_or_fur", labelKey: "wizardQ2Animals" },
      { value: "mixed_or_complex", labelKey: "wizardQ2Mixed" },
    ];
</script>

{#if open}
  <div class="modal-backdrop" role="presentation">
    <div
      class="modal wizard-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="wizard-title"
    >
      <div class="wizard-top">
        <span class="badge-pill">{stepLabel(wState.step)}</span>
        <div class="step-dots" aria-hidden="true">
          {#each [1, 2, 3, 4, 5] as n}
            <span class="step-dot {wState.step === n ? 'current' : wState.step > n ? 'done' : ''}"></span>
          {/each}
        </div>
        {#if canDismiss && !isDownloading}
          <button type="button" class="text-btn" onclick={onDismiss}>{t.closeBtn}</button>
        {/if}
      </div>

      {#if wState.step === 1}
        <h2 id="wizard-title">{t.wizardStep1Title}</h2>
        <p class="muted">{t.wizardStep1Subtitle}</p>

        <div class="wizard-radio-grid" role="radiogroup" aria-label={t.wizardStep1Title}>
          <button
            type="button"
            role="radio"
            aria-checked={lang === "en"}
            class="wizard-choice-card {lang === 'en' ? 'selected' : ''}"
            onclick={() => onLanguageChange("en")}
          >
            <strong>English</strong>
            <span class="muted">Offline background remover</span>
          </button>
          <button
            type="button"
            role="radio"
            aria-checked={lang === "id"}
            class="wizard-choice-card {lang === 'id' ? 'selected' : ''}"
            onclick={() => onLanguageChange("id")}
          >
            <strong>Bahasa Indonesia</strong>
            <span class="muted">Penghapus latar belakang offline</span>
          </button>
        </div>
      {:else if wState.step === 2}
        <h2 id="wizard-title">{t.wizardBriefingTitle}</h2>
        <p class="wizard-briefing">{t.wizardBriefingBody}</p>

        <fieldset class="wizard-fieldset">
          <legend>{t.wizardQ1Label}</legend>
          <div class="wizard-options">
            {#each usageOptions as opt}
              <label class="wizard-radio-row {wState.usage === opt.value ? 'selected' : ''}">
                <input
                  type="radio"
                  name="wizard-usage"
                  value={opt.value}
                  checked={wState.usage === opt.value}
                  onchange={() => (wState = { ...wState, usage: opt.value })}
                />
                <span>{t[opt.labelKey]}</span>
              </label>
            {/each}
          </div>
        </fieldset>

        <fieldset class="wizard-fieldset">
          <legend>{t.wizardQ2Label}</legend>
          <div class="wizard-options">
            {#each subjectOptions as opt}
              <label class="wizard-radio-row {wState.photoType === opt.value ? 'selected' : ''}">
                <input
                  type="radio"
                  name="wizard-subject"
                  value={opt.value}
                  checked={wState.photoType === opt.value}
                  onchange={() => (wState = { ...wState, photoType: opt.value })}
                />
                <span>{t[opt.labelKey]}</span>
              </label>
            {/each}
          </div>
        </fieldset>
      {:else if wState.step === 3}
        <h2 id="wizard-title">{t.wizardStep3Title}</h2>

        {#if recResponse}
          {#if recResponse.available_ram_mb !== null}
            <p class="wizard-ram-line">
              {t.wizardRamDetected.replace("{mb}", String(recResponse.available_ram_mb))}
            </p>
          {/if}
          {#if ramMessage(recResponse.recommendation.ram_advisory)}
            <div
              class="banner {recResponse.recommendation.ram_advisory === 'low_ram_warning' ? 'warning' : 'info'}"
              role="status"
            >
              <div class="banner-body">
                <p>{ramMessage(recResponse.recommendation.ram_advisory)}</p>
              </div>
            </div>
          {/if}
        {/if}

        <div class="wizard-cards">
          {#each orderedCardIds as modelId, idx}
            {@const m = manifestModels.find((item) => item.id === modelId)}
            {#if m}
              {@const isRecommended = recResponse ? recResponse.recommendation.primary_id === m.id : idx === 0}
              {@const isSelected = wState.selectedModelId === m.id}
              <div class="model-card {isSelected ? 'selected' : ''}">
                <div class="model-card-header">
                  <div class="model-badges">
                    <span class="badge-pill {isRecommended ? 'accent-pill' : ''}">
                      {isRecommended ? t.wizardRecommendedBadge : t.wizardAlternativeBadge}
                    </span>
                    {#if m.cached}
                      <span class="badge-pill ok">{t.wizardInstalledBadge}</span>
                    {/if}
                  </div>
                  <button
                    type="button"
                    class="{isSelected ? 'primary-btn sm' : 'secondary-btn sm'}"
                    onclick={() => (wState = selectWizardModel(wState, m.id))}
                  >
                    {isSelected ? t.wizardSelectedBadge : t.wizardChooseModelBtn}
                  </button>
                </div>

                <h3>{m.id === "fast" ? t.cardFastTitle : t.cardBalancedTitle}</h3>
                <p class="model-summary">
                  {m.id === "fast" ? t.cardFastSummary : t.cardBalancedSummary}
                </p>

                {#if isRecommended && recResponse}
                  <p class="reason-line">{translateReason(recResponse.recommendation.reason_code)}</p>
                {/if}

                <div class="model-metrics">
                  <span>{m.expected_size_mb.toFixed(1)} MB</span>
                  <span>{m.id === "fast" ? t.cardFastSpeed : t.cardBalancedSpeed}</span>
                  <span>{m.id === "fast" ? t.cardFastRam : t.cardBalancedRam}</span>
                  <span>{t.firstRunLicenseLabel}: {m.license_weights}</span>
                </div>
                <p class="measured-note">{t.cardMeasuredNote}</p>

                <div class="model-lists">
                  <div>
                    <strong>{t.cardStrengthsLabel}</strong>
                    <ul>
                      {#each m.strengths as tag}
                        <li>{translateTag(tag)}</li>
                      {/each}
                    </ul>
                  </div>
                  <div>
                    <strong>{t.cardWeaknessesLabel}</strong>
                    <ul>
                      {#each m.weaknesses as tag}
                        <li>{translateTag(tag)}</li>
                      {/each}
                    </ul>
                  </div>
                </div>

                {#if wState.showAllModels}
                  <details open>
                    <summary>{t.cardDetailsSummary}</summary>
                    <dl class="meta-grid compact">
                      <dt>{t.cardEngineLabel}</dt>
                      <dd>{m.engine}</dd>
                      <dt>{t.cardSourceLabel}</dt>
                      <dd>{m.primary_host} ({t.firstRunFallbackLabel}: {m.fallback_host})</dd>
                      <dt>{t.cardSha256Label}</dt>
                      <dd><code>{m.sha256}</code></dd>
                    </dl>
                  </details>
                {:else}
                  <details>
                    <summary>{t.cardDetailsSummary}</summary>
                    <dl class="meta-grid compact">
                      <dt>{t.cardEngineLabel}</dt>
                      <dd>{m.engine}</dd>
                      <dt>{t.cardSourceLabel}</dt>
                      <dd>{m.primary_host} ({t.firstRunFallbackLabel}: {m.fallback_host})</dd>
                      <dt>{t.cardSha256Label}</dt>
                      <dd><code>{m.sha256}</code></dd>
                    </dl>
                  </details>
                {/if}
              </div>
            {/if}
          {/each}
        </div>

        <div class="wizard-sublink-row">
          <button
            type="button"
            class="text-btn"
            onclick={() => (wState = { ...wState, showAllModels: !wState.showAllModels })}
          >
            {wState.showAllModels ? t.wizardHideAllModels : t.wizardSeeAllModels}
          </button>
        </div>

        <p class="measured-note">{t.cardDatasetAllModelsNote}</p>
      {:else if wState.step === 4 && selectedModel}
        <h2 id="wizard-title">{t.wizardStep4Title}</h2>
        <p>{t.wizardStep4OfflineNote}</p>

        {#if selectedModel.cached}
          <div class="banner info" role="status">
            <div class="banner-body">
              <strong>{selectedModel.id === "fast" ? t.cardFastTitle : t.cardBalancedTitle}</strong>
              <p>{t.wizardStep4ReadyInstalled}</p>
            </div>
          </div>
        {:else}
          <dl class="meta-grid">
            <dt>{t.firstRunFileLabel}</dt>
            <dd>{selectedModel.filename}</dd>
            <dt>{t.firstRunSizeLabel}</dt>
            <dd>{selectedModel.expected_size_mb.toFixed(1)} MB</dd>
            <dt>{t.firstRunHostLabel}</dt>
            <dd>{selectedModel.primary_host} ({t.firstRunFallbackLabel}: {selectedModel.fallback_host})</dd>
            <dt>{t.firstRunLicenseLabel}</dt>
            <dd>{selectedModel.license_weights}</dd>
          </dl>

          {#if selectedModel.disk_check}
            {#if !selectedModel.disk_check.sufficient}
              <div class="banner error" role="alert">
                <div class="banner-body">
                  <p>
                    {t.wizardDiskInsufficient
                      .replace("{req}", formatMb(selectedModel.disk_check.required_bytes))
                      .replace("{avail}", formatMb(selectedModel.disk_check.available_bytes))}
                  </p>
                </div>
              </div>
            {:else}
              <p class="muted disk-note">
                {t.wizardDiskCheckLine
                  .replace("{avail}", formatMb(selectedModel.disk_check.available_bytes))
                  .replace("{req}", formatMb(selectedModel.disk_check.required_bytes))}
              </p>
            {/if}
          {/if}

          {#if selectedModel.partial_bytes > 0 && !isDownloading}
            <div class="banner info" role="status">
              <div class="banner-body">
                <p>
                  {t.wizardPartialResumeNote
                    .replace("{done}", formatMb(selectedModel.partial_bytes))
                    .replace("{total}", selectedModel.expected_size_mb.toFixed(1))}
                </p>
              </div>
              <button
                type="button"
                class="text-btn"
                onclick={() => onDeletePartial(selectedModel.tier)}
              >
                {t.wizardDeletePartialBtn}
              </button>
            </div>
          {/if}

          {#if neverUseNetwork}
            <p class="banner warning">{t.firstRunBlockedOffline}</p>
          {/if}

          {#if isDownloading}
            <div class="download-progress">
              <span>{t.progressDownloading} ({downloadProgress}%)</span>
              <progress max="100" value={downloadProgress}></progress>
            </div>
          {/if}

          {#if downloadPaused}
            <div class="banner warning" role="status">
              <div class="banner-body">
                <p>{t.wizardDownloadPaused}</p>
              </div>
            </div>
          {/if}

          {#if downloadError}
            <div class="banner error" role="alert">
              <div class="banner-body">
                <strong>{downloadError.title}</strong>
                <p>{downloadError.fix}</p>
                <details>
                  <summary>{t.errorDetailsLabel}</summary>
                  <code>{downloadError.details}</code>
                </details>
              </div>
            </div>
          {/if}

          <div class="wizard-download-actions">
            <button
              type="button"
              class="primary-btn"
              onclick={() => onDownload(selectedModel.tier)}
              disabled={isDownloading || neverUseNetwork || diskInsufficient}
            >
              {downloadError ? t.firstRunRetryBtn : t.firstRunDownloadBtn}
            </button>
            <button
              type="button"
              class="secondary-btn"
              onclick={() => onImportLocal(selectedModel.tier)}
              disabled={isDownloading}
            >
              {t.firstRunImportBtn}
            </button>
          </div>

          <details class="offline-box">
            <summary>{t.wizardOfflineRouteTitle}</summary>
            <p>{t.wizardOfflineRouteBody}</p>
            <div class="copy-row">
              <code>{selectedModel.primary_url}</code>
              <button
                type="button"
                class="secondary-btn sm"
                onclick={() => copyText("link", selectedModel.primary_url)}
              >
                {copiedKey === "link" ? t.wizardCopiedValue : t.wizardCopyLinkBtn}
              </button>
            </div>
            <div class="copy-row">
              <code>{selectedModel.sha256}</code>
              <button
                type="button"
                class="secondary-btn sm"
                onclick={() => copyText("hash", selectedModel.sha256)}
              >
                {copiedKey === "hash" ? t.wizardCopiedValue : t.wizardCopyHashBtn}
              </button>
            </div>
          </details>
        {/if}
      {:else if wState.step === 5 && selectedModel}
        <h2 id="wizard-title">{t.wizardStep5Title}</h2>
        <p class="wizard-ready-body">{t.wizardStep5Body}</p>
        <div class="banner info" role="status">
          <div class="banner-body">
            <strong>{t.wizardStep5ActiveModel}:</strong>
            <span>{selectedModel.id === "fast" ? t.cardFastTitle : t.cardBalancedTitle}</span>
          </div>
        </div>
      {/if}

      <div class="wizard-footer">
        <div class="wizard-footer-left">
          {#if wState.step > 1}
            <button
              type="button"
              class="secondary-btn"
              onclick={handleBack}
              disabled={isDownloading}
            >
              {t.wizardBackBtn}
            </button>
          {/if}
          {#if isDownloading}
            <button
              type="button"
              class="text-btn"
              onclick={onCancelDownload}
            >
              {t.firstRunCancelBtn}
            </button>
          {/if}
        </div>

        <div class="wizard-footer-right">
          {#if wState.step < 5}
            <button
              type="button"
              class="primary-btn"
              onclick={handleContinue}
              disabled={!canAdvance || isDownloading}
            >
              {t.wizardContinueBtn}
            </button>
          {:else}
            <button
              type="button"
              class="primary-btn"
              onclick={handleFinish}
              disabled={!canFinish}
            >
              {t.wizardFinishBtn}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .wizard-modal {
    width: min(680px, 94vw);
    max-height: 90vh;
    overflow-y: auto;
    background: var(--bg-panel);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .wizard-modal h2 {
    margin: 0;
    font-size: var(--text-title);
  }

  .download-progress {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
  }

  .download-progress progress {
    width: 100%;
    height: 8px;
  }

  .wizard-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .step-dots {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .step-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--border);
  }

  .step-dot.current {
    width: 18px;
    background: var(--accent);
  }

  .step-dot.done {
    background: var(--success);
  }

  .wizard-briefing,
  .wizard-ready-body {
    margin: 0;
    line-height: 1.5;
  }

  .wizard-radio-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .wizard-choice-card {
    padding: 14px;
    border-radius: var(--radius-card);
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text-primary);
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    cursor: pointer;
    min-height: 56px;
    transition: border-color var(--transition-fast);
  }

  .wizard-choice-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .wizard-fieldset {
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: var(--space-3);
    margin: 0;
  }

  .wizard-fieldset legend {
    font-weight: 600;
    font-size: var(--text-body);
    padding: 0 6px;
  }

  .wizard-options {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-2);
    margin-top: 6px;
  }

  .wizard-radio-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 10px;
    border-radius: var(--radius-control);
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    cursor: pointer;
    font-size: 13px;
  }

  .wizard-radio-row.selected {
    border-color: var(--accent);
  }

  .wizard-ram-line,
  .disk-note,
  .measured-note {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .wizard-cards {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .model-card {
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: var(--space-3);
    background: var(--bg-elevated);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .model-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .model-card h3 {
    margin: 2px 0 0;
    font-size: 15px;
  }

  .model-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
  }

  .model-badges {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .accent-pill {
    border-color: var(--accent);
    color: var(--accent);
  }

  .model-summary,
  .reason-line {
    margin: 0;
    font-size: 13px;
  }

  .reason-line {
    color: var(--accent);
    font-weight: 500;
  }

  .model-metrics {
    display: flex;
    gap: var(--space-3);
    flex-wrap: wrap;
    font-size: 12px;
    font-weight: 600;
  }

  .model-lists {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
    font-size: 12px;
  }

  .model-lists ul {
    margin: var(--space-1) 0 0;
    padding-left: var(--space-4);
  }

  .wizard-sublink-row {
    display: flex;
    justify-content: flex-start;
  }

  .wizard-download-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .offline-box {
    border: 1px solid var(--border);
    border-radius: var(--radius-control);
    padding: var(--space-2) 10px;
    font-size: 12px;
  }

  .offline-box summary {
    cursor: pointer;
    font-weight: 600;
  }

  .copy-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-top: 6px;
  }

  .copy-row code {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 11px;
  }

  .wizard-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: var(--space-2);
    border-top: 1px solid var(--border);
    margin-top: var(--space-1);
  }

  .wizard-footer-left,
  .wizard-footer-right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  @media (max-width: 600px) {
    .wizard-radio-grid,
    .wizard-options,
    .model-lists {
      grid-template-columns: 1fr;
    }
  }
</style>
