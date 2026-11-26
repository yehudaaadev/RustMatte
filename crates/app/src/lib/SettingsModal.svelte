<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { FriendlyError } from "$lib/errors";
  import type { Lang, Translations } from "$lib/i18n";
  import type { ManifestModelInfo, WizardModelId } from "$lib/wizard";

  type ThemeMode = "system" | "light" | "dark";
  type RefineLevel = "none" | "light" | "standard";
  type BgMode = "transparent" | "solid";

  interface Props {
    open: boolean;
    t: Translations;
    lang: Lang;
    themeMode: ThemeMode;
    selectedTier: "quality" | "fast";
    refineLevel: RefineLevel;
    maxSide: number | null;
    backgroundMode: BgMode;
    backgroundHex: string;
    outputFolder: string | null;
    displayOutputFolder: string;
    neverUseNetwork: boolean;
    manifestModels: ManifestModelInfo[];
    isBusy: boolean;
    diagnosticsCopied: boolean;
    onClose: () => void;
    onLanguageChange: (v: Lang) => void;
    onThemeChange: (v: ThemeMode) => void;
    onTierChange: (v: "quality" | "fast") => void;
    onRefineChange: (v: RefineLevel) => void;
    onMaxSideChange: (v: number | null) => void;
    onBgModeChange: (v: BgMode) => void;
    onBgHexChange: (v: string) => void;
    onChooseOutputFolder: () => void;
    onResetOutputFolder: () => void;
    onNeverUseNetworkChange: (v: boolean) => void;
    onRerunWizard: () => void;
    onDownloadOrImportModel: (id: WizardModelId, tier: "quality" | "fast") => void;
    onRefreshModels: () => Promise<void>;
    onOpenAbout: () => void;
    onCopyDiagnostics: () => Promise<void>;
    onError: (err: FriendlyError) => void;
  }

  let {
    open,
    t,
    lang,
    themeMode,
    selectedTier,
    refineLevel,
    maxSide,
    backgroundMode,
    backgroundHex,
    outputFolder,
    displayOutputFolder,
    neverUseNetwork,
    manifestModels,
    isBusy,
    diagnosticsCopied,
    onClose,
    onLanguageChange,
    onThemeChange,
    onTierChange,
    onRefineChange,
    onMaxSideChange,
    onBgModeChange,
    onBgHexChange,
    onChooseOutputFolder,
    onResetOutputFolder,
    onNeverUseNetworkChange,
    onRerunWizard,
    onDownloadOrImportModel,
    onRefreshModels,
    onOpenAbout,
    onCopyDiagnostics,
    onError,
  }: Props = $props();

  let confirmDeleteTier = $state<"quality" | "fast" | null>(null);
  let modelNotice = $state<{ tier: string; text: string; kind: "info" | "warning" | "error" } | null>(null);
  let modelActionBusy = $state(false);

  $effect(() => {
    if (open) {
      confirmDeleteTier = null;
      modelNotice = null;
    }
  });

  function formatMb(bytes: number): string {
    return (bytes / (1024 * 1024)).toFixed(1);
  }

  function modelTitle(m: ManifestModelInfo): string {
    return m.id === "fast" ? t.cardFastTitle : t.cardBalancedTitle;
  }

  async function verifyModel(m: ManifestModelInfo) {
    modelActionBusy = true;
    modelNotice = null;
    try {
      const res = await invoke<{ tier: string; status: string; size_bytes: number }>(
        "verify_cached_model",
        { tier: m.tier }
      );
      if (res.status === "verified") {
        modelNotice = {
          tier: m.tier,
          text: t.settingsModelVerifyOk.replace("{mb}", formatMb(res.size_bytes)),
          kind: "info",
        };
      } else if (res.status === "corrupt") {
        modelNotice = { tier: m.tier, text: t.settingsModelVerifyCorrupt, kind: "error" };
      } else {
        modelNotice = { tier: m.tier, text: t.settingsModelVerifyMissing, kind: "warning" };
      }
      await onRefreshModels();
    } catch (e) {
      modelNotice = { tier: m.tier, text: String(e), kind: "error" };
    } finally {
      modelActionBusy = false;
    }
  }

  async function deleteModel(m: ManifestModelInfo) {
    modelActionBusy = true;
    modelNotice = null;
    try {
      const res = await invoke<{ tier: string; deleted: boolean; freed_bytes: number }>(
        "delete_cached_model_cmd",
        { tier: m.tier }
      );
      confirmDeleteTier = null;
      if (res.deleted) {
        modelNotice = {
          tier: m.tier,
          text: t.settingsModelDeletedNotice
            .replace("{name}", modelTitle(m))
            .replace("{mb}", formatMb(res.freed_bytes)),
          kind: "info",
        };
      }
      await onRefreshModels();
    } catch (e) {
      modelNotice = { tier: m.tier, text: String(e), kind: "error" };
    } finally {
      modelActionBusy = false;
    }
  }

  async function sendFeedback() {
    try {
      await invoke("open_feedback_url");
    } catch (e) {
      onError({
        title: t.errGenericTitle,
        fix: t.errGenericFix,
        details: String(e),
        kind: "generic",
      });
    }
  }
</script>

{#if open}
  <div class="modal-backdrop" role="presentation">
    <div
      class="modal modal-lg settings-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
    >
      <div class="modal-header">
        <h2 id="settings-title">{t.settingsTitle}</h2>
        <button type="button" class="text-btn" onclick={onClose}>{t.closeBtn}</button>
      </div>

      <div class="settings-scroll">
        <section class="settings-section">
          <h3>{t.settingsGeneralSection}</h3>
          <div class="settings-grid">
            <div class="field">
              <label for="settings-lang">{t.languageLabel}</label>
              <select
                id="settings-lang"
                value={lang}
                onchange={(e) => onLanguageChange((e.currentTarget as HTMLSelectElement).value as Lang)}
              >
                <option value="en">English</option>
                <option value="id">Bahasa Indonesia</option>
              </select>
            </div>
            <div class="field">
              <label for="settings-theme">{t.themeLabel}</label>
              <select
                id="settings-theme"
                value={themeMode}
                onchange={(e) => onThemeChange((e.currentTarget as HTMLSelectElement).value as ThemeMode)}
              >
                <option value="system">{t.themeSystem}</option>
                <option value="light">{t.themeLight}</option>
                <option value="dark">{t.themeDark}</option>
              </select>
            </div>
          </div>
        </section>

        <section class="settings-section">
          <div class="section-row">
            <h3>{t.settingsModelsSection}</h3>
            <button type="button" class="secondary-btn sm" onclick={onRerunWizard} disabled={isBusy}>
              {t.settingsRerunWizardBtn}
            </button>
          </div>

          <div class="field">
            <label for="settings-active-tier">{t.settingsActiveModelLabel}</label>
            <select
              id="settings-active-tier"
              value={selectedTier}
              disabled={isBusy}
              onchange={(e) => onTierChange((e.currentTarget as HTMLSelectElement).value as "quality" | "fast")}
            >
              <option value="quality">{t.cardBalancedTitle}</option>
              <option value="fast">{t.cardFastTitle}</option>
            </select>
          </div>

          <div class="model-list">
            {#each manifestModels as m (m.id)}
              <div class="model-row">
                <div class="model-row-info">
                  <div class="model-row-title">
                    <strong>{modelTitle(m)}</strong>
                    <span class="badge-pill {m.cached ? 'ok' : ''}">
                      {m.cached ? t.wizardInstalledBadge : t.modelMissingBadge}
                    </span>
                    {#if selectedTier === m.tier}
                      <span class="badge-pill">{t.wizardSelectedBadge}</span>
                    {/if}
                  </div>
                  <span class="muted">
                    {m.cached
                      ? t.settingsModelSizeOnDisk.replace("{mb}", formatMb(m.size_on_disk_bytes || m.size_bytes))
                      : t.settingsModelNotOnDisk.replace("{mb}", m.expected_size_mb.toFixed(1))}
                  </span>
                </div>

                <div class="model-row-actions">
                  {#if m.cached}
                    <button
                      type="button"
                      class="secondary-btn sm"
                      disabled={modelActionBusy || isBusy}
                      onclick={() => verifyModel(m)}
                    >
                      {t.settingsModelVerifyBtn}
                    </button>
                    {#if confirmDeleteTier === m.tier}
                      <button
                        type="button"
                        class="danger-btn sm"
                        disabled={modelActionBusy || isBusy}
                        onclick={() => deleteModel(m)}
                      >
                        {t.settingsModelConfirmDeleteBtn}
                      </button>
                      <button
                        type="button"
                        class="text-btn"
                        onclick={() => (confirmDeleteTier = null)}
                      >
                        {t.cancelBtn}
                      </button>
                    {:else}
                      <button
                        type="button"
                        class="secondary-btn sm"
                        disabled={modelActionBusy || isBusy}
                        onclick={() => (confirmDeleteTier = m.tier)}
                      >
                        {t.settingsModelDeleteBtn}
                      </button>
                    {/if}
                  {:else}
                    <button
                      type="button"
                      class="primary-btn sm"
                      disabled={modelActionBusy || isBusy}
                      onclick={() => onDownloadOrImportModel(m.id, m.tier)}
                    >
                      {t.settingsModelDownloadBtn}
                    </button>
                  {/if}
                </div>
              </div>

              {#if confirmDeleteTier === m.tier}
                <p class="confirm-note">
                  {t.settingsModelDeleteConfirm
                    .replace("{name}", modelTitle(m))
                    .replace("{mb}", formatMb(m.size_on_disk_bytes || m.size_bytes))}
                </p>
              {/if}

              {#if modelNotice && modelNotice.tier === m.tier}
                <div class="banner {modelNotice.kind}" role="status">
                  <span>{modelNotice.text}</span>
                </div>
              {/if}
            {/each}
          </div>
        </section>

        <section class="settings-section">
          <h3>{t.settingsNetworkSection}</h3>
          <label class="checkbox-row">
            <input
              type="checkbox"
              checked={neverUseNetwork}
              onchange={(e) => onNeverUseNetworkChange((e.currentTarget as HTMLInputElement).checked)}
            />
            <span>{t.neverUseNetworkLabel}</span>
          </label>
          <p class="muted">{t.neverUseNetworkHint}</p>
        </section>

        <section class="settings-section">
          <h3>{t.settingsOutputSection}</h3>
          <div class="field">
            <span class="field-label">{t.outputFolderLabel}</span>
            <div class="folder-box">{outputFolder ? displayOutputFolder : t.outputFolderAuto}</div>
            <div class="inline-actions">
              <button type="button" class="secondary-btn sm" onclick={onChooseOutputFolder} disabled={isBusy}>
                {t.chooseFolder}
              </button>
              {#if outputFolder}
                <button type="button" class="secondary-btn sm" onclick={onResetOutputFolder} disabled={isBusy}>
                  {t.resetFolder}
                </button>
              {/if}
            </div>
          </div>

          <div class="settings-grid">
            <div class="field">
              <span class="field-label">{t.backgroundLabel}</span>
              <div class="inline-actions">
                <button
                  type="button"
                  class="secondary-btn sm {backgroundMode === 'transparent' ? 'active-choice' : ''}"
                  onclick={() => onBgModeChange("transparent")}
                >
                  {t.bgTransparent}
                </button>
                <button
                  type="button"
                  class="secondary-btn sm {backgroundMode === 'solid' ? 'active-choice' : ''}"
                  onclick={() => onBgModeChange("solid")}
                >
                  {t.bgSolid}
                </button>
                {#if backgroundMode === "solid"}
                  <input
                    type="color"
                    aria-label={t.bgColorPickerLabel}
                    value={backgroundHex}
                    onchange={(e) => onBgHexChange((e.currentTarget as HTMLInputElement).value)}
                  />
                {/if}
              </div>
            </div>

            <div class="field">
              <label for="settings-max-side">{t.maxSideLabel}</label>
              <select
                id="settings-max-side"
                value={maxSide === null ? "" : String(maxSide)}
                disabled={isBusy}
                onchange={(e) => {
                  const raw = (e.currentTarget as HTMLSelectElement).value;
                  onMaxSideChange(raw ? Number(raw) : null);
                }}
              >
                <option value="">{t.maxSideOriginal}</option>
                <option value="2048">{t.maxSide2048}</option>
                <option value="1024">{t.maxSide1024}</option>
                <option value="512">{t.maxSide512}</option>
              </select>
            </div>
          </div>

          <details class="advanced-details">
            <summary>{t.settingsAdvancedSummary}</summary>
            <div class="settings-grid">
              <div class="field">
                <label for="settings-refine">{t.refineLabel}</label>
                <select
                  id="settings-refine"
                  value={refineLevel}
                  disabled={isBusy}
                  onchange={(e) => onRefineChange((e.currentTarget as HTMLSelectElement).value as RefineLevel)}
                >
                  <option value="standard">{t.refineStandard}</option>
                  <option value="light">{t.refineLight}</option>
                  <option value="none">{t.refineNone}</option>
                </select>
              </div>
              <div class="field">
                <span class="field-label">{t.deviceLabel}</span>
                <div class="folder-box" role="note">{t.deviceReadOnlyLine}</div>
              </div>
            </div>
          </details>
        </section>

        <section class="settings-section">
          <h3>{t.settingsFeedbackSection}</h3>
          <div class="inline-actions">
            <button type="button" class="secondary-btn sm" onclick={sendFeedback}>
              {t.sendFeedbackBtn}
            </button>
            <button type="button" class="secondary-btn sm" onclick={onCopyDiagnostics}>
              {t.copyDiagnosticsBtn}
            </button>
            <button type="button" class="secondary-btn sm" onclick={onOpenAbout}>
              {t.aboutBtn}
            </button>
          </div>
          {#if diagnosticsCopied}
            <p class="ok-note">{t.diagnosticsCopied}</p>
          {/if}
          <p class="muted">{t.sendFeedbackHint}</p>
          <p class="muted">{t.sendFeedbackNoAccountNote}</p>
        </section>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    z-index: 50;
  }

  .settings-modal {
    width: min(740px, 94vw);
    max-height: 90vh;
    background: var(--bg-panel);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 18px;
  }

  .settings-scroll {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding-right: 4px;
  }

  .settings-section {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .settings-section h3 {
    margin: 0;
    font-size: 14px;
  }

  .section-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .settings-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field label,
  .field-label {
    font-size: 12px;
    font-weight: 600;
  }

  select {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 13px;
  }

  .model-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .model-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
    flex-wrap: wrap;
  }

  .model-row-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .model-row-title {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
    font-size: 13px;
  }

  .model-row-actions,
  .inline-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .badge-pill {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .badge-pill.ok {
    color: #2f7d5b;
    border-color: rgba(47, 125, 91, 0.45);
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }

  .folder-box {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .advanced-details summary {
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .muted {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .confirm-note {
    margin: 0;
    font-size: 12px;
    color: #b7791f;
    font-weight: 600;
  }

  .ok-note {
    margin: 0;
    font-size: 12px;
    color: #2f7d5b;
    font-weight: 600;
  }

  .banner {
    border-radius: 8px;
    padding: 8px 10px;
    font-size: 12px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
  }

  .banner.error {
    border-color: #b3412e;
    background: rgba(179, 65, 46, 0.12);
  }

  .banner.warning {
    border-color: #b7791f;
    background: rgba(183, 121, 31, 0.12);
  }

  .banner.info {
    border-color: var(--accent);
    background: rgba(63, 94, 168, 0.12);
  }

  button {
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .primary-btn {
    background: var(--accent);
    color: #ffffff;
  }

  .secondary-btn {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border-color: var(--border);
  }

  .secondary-btn.active-choice {
    border-color: var(--accent);
    color: var(--accent);
  }

  .danger-btn {
    background: #b3412e;
    color: #ffffff;
  }

  .text-btn {
    background: transparent;
    color: var(--accent);
    padding: 4px 6px;
  }

  @media (max-width: 600px) {
    .settings-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
