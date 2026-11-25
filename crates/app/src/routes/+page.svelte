<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { dictionaries, type Lang } from "$lib/i18n";
  import { mapTechnicalError, type FriendlyError } from "$lib/errors";
  import {
    formatDisplayPath,
    renderRejectedReason,
    renderStatusBanner,
    renderWarningItem,
    type StatusBanner,
  } from "$lib/banners";
  import {
    createInitialSliderState,
    isComparingWithOriginal,
    resetSliderForNewPhoto,
    setSliderPosition,
    toggleCompareWithOriginal,
  } from "$lib/slider";
  import {
    applyProgressEvent,
    clearQueue as clearQueueState,
    createInitialQueueState,
    enqueuePaths,
    failModelDownload,
    finishJob,
    markModelMissing,
    selectAdjacent,
    setDownloadProgress,
    setJobPhase,
    startJob,
    startModelDownload,
    summarizeQueue,
    type FinishedPayload,
    type ProgressPayload,
  } from "$lib/queue";

  type ThemeMode = "system" | "light" | "dark";
  type RefineLevel = "none" | "light" | "standard";
  type BgMode = "transparent" | "solid";

  interface ModelTierInfo {
    tier: string;
    name: string;
    filename: string;
    cached: boolean;
    expected_size_mb: number;
    license: string;
    primary_host: string;
    fallback_host: string;
    caveat: string | null;
  }

  interface AboutInfo {
    version: string;
    license: string;
    models_markdown: string;
    third_party_licenses_markdown: string;
  }

  // Settings state
  let lang = $state<Lang>("en");
  let themeMode = $state<ThemeMode>("system");
  let systemPrefersDark = $state(true);
  let selectedTier = $state<"quality" | "fast">("quality");
  let selectedDevice = $state("cpu");
  let refineLevel = $state<RefineLevel>("standard");
  let maxSide = $state<number | null>(null);
  let backgroundMode = $state<BgMode>("transparent");
  let backgroundHex = $state("#ffffff");
  let outputFolder = $state<string | null>(null);
  let neverUseNetwork = $state(false);
  let settingsLoaded = $state(false);

  let t = $derived(dictionaries[lang]);
  let resolvedDark = $derived(
    themeMode === "system" ? systemPrefersDark : themeMode === "dark"
  );

  // Model & setup state
  let qualityModel = $state<ModelTierInfo | null>(null);
  let fastModel = $state<ModelTierInfo | null>(null);
  let showFirstRunModal = $state(false);
  let setupTier = $state<"quality" | "fast">("quality");
  let downloadError = $state<FriendlyError | null>(null);

  let activeModelInfo = $derived(
    selectedTier === "fast" ? fastModel : qualityModel
  );
  let setupModelInfo = $derived(
    setupTier === "fast" ? fastModel : qualityModel
  );
  let isActiveModelReady = $derived(activeModelInfo?.cached ?? false);

  // About & Diagnostics modal state
  let showAboutModal = $state(false);
  let aboutInfo = $state<AboutInfo | null>(null);
  let diagnosticsCopied = $state(false);

  // Queue & UI state
  let qState = $state(createInitialQueueState());
  let summary = $derived(summarizeQueue(qState));
  let selectedItem = $derived(
    qState.selectedIndex >= 0 && qState.selectedIndex < qState.items.length
      ? qState.items[qState.selectedIndex]
      : null
  );
  let isBusy = $derived(
    qState.phase === "loading_model" || qState.phase === "processing"
  );
  let isDownloading = $derived(qState.phase === "downloading_model");

  let isDragOver = $state(false);
  let activeError = $state<FriendlyError | null>(null);
  let statusBanner = $state<StatusBanner | null>(null);

  // Comparison slider, zoom & pan state (defaults to 100% cutout visible)
  let sliderState = $state(createInitialSliderState());
  let sliderPos = $derived(sliderState.sliderPos);
  let isComparing = $derived(isComparingWithOriginal(sliderState));
  let isDraggingSlider = $state(false);
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let isPanning = $state(false);
  let panStartX = 0;
  let panStartY = 0;

  // Virtualized thumbnail queue state (smooth for hundreds of items)
  const ROW_HEIGHT = 62;
  let queueScrollTop = $state(0);
  let queueViewportHeight = $state(420);
  let visibleStart = $derived(
    Math.max(0, Math.floor(queueScrollTop / ROW_HEIGHT) - 3)
  );
  let visibleEnd = $derived(
    Math.min(
      qState.items.length,
      Math.ceil((queueScrollTop + queueViewportHeight) / ROW_HEIGHT) + 4
    )
  );
  let visibleItems = $derived(
    qState.items.slice(visibleStart, visibleEnd).map((item, offset) => ({
      item,
      index: visibleStart + offset,
    }))
  );

  let originalPreviewUrl = $derived(
    selectedItem?.originalPreviewPath
      ? convertFileSrc(selectedItem.originalPreviewPath)
      : null
  );
  let cutoutPreviewUrl = $derived(
    selectedItem?.cutoutPreviewPath
      ? convertFileSrc(selectedItem.cutoutPreviewPath)
      : null
  );

  let unlistenFns: UnlistenFn[] = [];

  onMount(async () => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    systemPrefersDark = mq.matches;
    const onSchemeChange = (e: MediaQueryListEvent) => {
      systemPrefersDark = e.matches;
    };
    mq.addEventListener("change", onSchemeChange);
    unlistenFns.push(() => mq.removeEventListener("change", onSchemeChange));

    try {
      const saved: any = await invoke("get_settings");
      if (saved) {
        lang = saved.language === "id" ? "id" : "en";
        themeMode =
          saved.theme === "light" || saved.theme === "dark"
            ? saved.theme
            : "system";
        selectedTier = saved.tier === "fast" ? "fast" : "quality";
        selectedDevice =
          saved.device === "auto" || !saved.device ? "cpu" : saved.device;
        refineLevel =
          saved.refine_level === "none" || saved.refine_level === "light"
            ? saved.refine_level
            : "standard";
        maxSide = saved.max_side ?? null;
        backgroundMode =
          saved.background_mode === "solid" ? "solid" : "transparent";
        backgroundHex = saved.background_hex || "#ffffff";
        outputFolder = saved.output_folder ?? null;
        neverUseNetwork = Boolean(saved.never_use_network);
      }
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    } finally {
      settingsLoaded = true;
    }

    await refreshModelStatus();
    if (!isActiveModelReady) {
      setupTier = selectedTier;
      qState = markModelMissing(qState);
    }

    unlistenFns.push(
      await listen("download_progress", (event: any) => {
        if (event.payload) {
          qState = setDownloadProgress(qState, event.payload.percentage);
        }
      })
    );

    unlistenFns.push(
      await listen("job_status", (event: any) => {
        if (!event.payload) return;
        if (event.payload.phase === "loading_model") {
          qState = setJobPhase(qState, "loading_model");
        } else if (event.payload.phase === "processing") {
          qState = setJobPhase(qState, "processing");
        }
      })
    );

    unlistenFns.push(
      await listen("job_progress", (event: any) => {
        const p = event.payload as ProgressPayload | undefined;
        if (!p) return;
        qState = applyProgressEvent(qState, p);
        if (p.status === "failed" && p.error) {
          activeError = mapTechnicalError(p.error, t);
        }
        if (p.cutout_preview_path) {
          resetViewTransform();
        }
      })
    );

    unlistenFns.push(
      await listen("job_finished", (event: any) => {
        const p = event.payload as FinishedPayload | undefined;
        if (!p) return;
        qState = finishJob(qState, p);
        if (p.cancelled) {
          statusBanner = { code: "job_cancelled" };
        } else if (p.failed > 0) {
          statusBanner = {
            code: "job_partial",
            processed: p.processed,
            total: p.processed + p.failed,
            durationSec: p.total_ms / 1000,
          };
        } else {
          statusBanner = {
            code: "job_done",
            processed: p.processed,
            total: p.processed + p.failed,
            durationSec: p.total_ms / 1000,
          };
        }
      })
    );

    const webview = getCurrentWebview();
    unlistenFns.push(
      await webview.onDragDropEvent(async (event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          isDragOver = true;
        } else if (payload.type === "leave") {
          isDragOver = false;
        } else if (payload.type === "drop") {
          isDragOver = false;
          if (payload.paths && payload.paths.length > 0) {
            await addPathsToQueue(payload.paths);
          }
        }
      })
    );

    try {
      await invoke("notify_frontend_ready", {
        origin: window.location.origin,
      });
    } catch {
      // Ignore outside Tauri runtime.
    }
  });

  onDestroy(() => {
    for (const fn of unlistenFns) {
      fn();
    }
  });

  $effect(() => {
    if (!settingsLoaded) return;
    const payload = {
      language: lang,
      theme: themeMode,
      tier: selectedTier,
      device: selectedDevice,
      refine_level: refineLevel,
      output_folder: outputFolder,
      max_side: maxSide,
      background_mode: backgroundMode,
      background_hex: backgroundHex,
      never_use_network: neverUseNetwork,
    };
    invoke("set_settings", { newSettings: payload }).catch((e) => {
      console.error("Failed to save settings:", e);
    });
  });

  async function refreshModelStatus() {
    try {
      const res: any = await invoke("get_models_status");
      qualityModel = res.quality;
      fastModel = res.fast;
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function addPathsToQueue(rawPaths: string[]) {
    activeError = null;
    statusBanner = null;
    try {
      const expanded: {
        valid_paths: string[];
        rejected: { path: string; name: string; reason_code?: string; reason: string }[];
      } = await invoke("expand_inputs", { paths: rawPaths });
      qState = enqueuePaths(qState, expanded.valid_paths, expanded.rejected);
      if (expanded.valid_paths.length === 0 && expanded.rejected.length > 0) {
        activeError = mapTechnicalError(expanded.rejected[0].reason, t);
      }
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function browsePhotos() {
    const selected = await open({
      title: t.dialogBrowsePhotosTitle,
      multiple: true,
      directory: false,
      filters: [
        {
          name: t.filterSupportedImages,
          extensions: ["jpg", "jpeg", "png", "webp", "bmp"],
        },
      ],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length > 0) {
      await addPathsToQueue(paths);
    }
  }

  async function browseFolder() {
    const selected = await open({
      title: t.dialogBrowseFolderTitle,
      directory: true,
      multiple: false,
    });
    if (typeof selected === "string" && selected.trim().length > 0) {
      await addPathsToQueue([selected]);
    }
  }

  async function chooseOutputFolder() {
    const selected = await open({
      title: t.dialogOutputFolderTitle,
      directory: true,
      multiple: false,
    });
    if (typeof selected === "string" && selected.trim().length > 0) {
      outputFolder = selected;
    }
  }

  function openModelSetup(tier: "quality" | "fast") {
    setupTier = tier;
    downloadError = null;
    showFirstRunModal = true;
  }

  async function startDownloadFromModal() {
    if (neverUseNetwork) {
      downloadError = mapTechnicalError("Never use the network", t);
      return;
    }
    downloadError = null;
    qState = startModelDownload(qState);
    try {
      await invoke("download_model", { tier: setupTier, consent: true });
      await refreshModelStatus();
      showFirstRunModal = false;
      qState = {
        ...qState,
        phase: qState.items.length > 0 ? "done" : "empty",
      };
    } catch (e) {
      const msg = String(e);
      qState = failModelDownload(qState, msg);
      downloadError = mapTechnicalError(msg, t);
    }
  }

  async function importLocalModelFile() {
    const selected = await open({
      title: t.dialogImportModelTitle,
      multiple: false,
      directory: false,
      filters: [{ name: t.filterOnnxModel, extensions: ["onnx"] }],
    });
    if (typeof selected !== "string" || !selected.trim()) return;
    downloadError = null;
    try {
      await invoke("import_model_file", {
        tier: setupTier,
        filePath: selected,
      });
      await refreshModelStatus();
      showFirstRunModal = false;
      qState = {
        ...qState,
        phase: qState.items.length > 0 ? "done" : "empty",
      };
    } catch (e) {
      downloadError = mapTechnicalError(String(e), t);
    }
  }

  async function runRemoveBackground() {
    if (isBusy || qState.items.length === 0) return;
    await refreshModelStatus();
    if (!isActiveModelReady) {
      openModelSetup(selectedTier);
      return;
    }

    activeError = null;
    statusBanner = null;
    const paths = qState.items.map((item) => item.path);
    qState = startJob(qState, true);

    try {
      await invoke("process_images", {
        paths,
        options: {
          tier: selectedTier,
          device: selectedDevice,
          max_side: maxSide,
          refine_level: refineLevel,
          output_folder: outputFolder,
          background_hex: backgroundMode === "solid" ? backgroundHex : null,
        },
      });
    } catch (e) {
      const msg = String(e);
      activeError = mapTechnicalError(msg, t);
      qState = finishJob(qState, {
        processed: 0,
        failed: qState.items.length,
        cancelled: false,
        total_ms: 0,
      });
    }
  }

  async function cancelCurrentJob() {
    try {
      await invoke("cancel_job");
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function saveSelectedCutout() {
    if (!selectedItem?.fullCutoutPath) return;
    activeError = null;
    try {
      const stem = selectedItem.name.replace(/\.[^.]+$/, "");
      const dest = await save({
        title: t.dialogSavePngTitle,
        defaultPath: `${stem}_cutout.png`,
        filters: [{ name: t.filterPngImage, extensions: ["png"] }],
      });
      if (!dest) return;

      const saved: { path: string; display_path: string } = await invoke(
        "save_cutout_png",
        {
          sourceCutoutPath: selectedItem.fullCutoutPath,
          destinationPath: dest,
          inputPaths: [selectedItem.path],
          backgroundHex: backgroundMode === "solid" ? backgroundHex : null,
        }
      );
      statusBanner = {
        code: "saved_single",
        displayPath: saved.display_path,
      };
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function saveAllCutouts() {
    const readyItems = qState.items.filter(
      (i) => (i.status === "done" || i.status === "warning") && i.fullCutoutPath
    );
    if (readyItems.length === 0) return;

    let targetDir = outputFolder ?? qState.outputDir;
    if (!targetDir) {
      const picked = await open({
        title: t.dialogOutputFolderTitle,
        directory: true,
        multiple: false,
      });
      if (typeof picked !== "string" || !picked.trim()) return;
      targetDir = picked;
    }

    activeError = null;
    try {
      const res: {
        saved_count: number;
        output_dir: string;
        display_output_dir: string;
      } = await invoke("save_all_cutouts", {
        items: readyItems.map((i) => ({
          source_cutout_path: i.fullCutoutPath,
          input_path: i.path,
        })),
        outputDir: targetDir,
        backgroundHex: backgroundMode === "solid" ? backgroundHex : null,
      });
      qState = {
        ...qState,
        outputDir: res.output_dir,
        displayOutputDir: res.display_output_dir,
      };
      statusBanner = {
        code: "saved_all",
        count: res.saved_count,
        displayDir: res.display_output_dir,
      };
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function openOutputDir() {
    const dir = qState.outputDir ?? outputFolder;
    if (!dir) return;
    try {
      await invoke("open_output_folder", { path: dir });
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  async function openAbout() {
    showAboutModal = true;
    diagnosticsCopied = false;
    if (!aboutInfo) {
      try {
        aboutInfo = await invoke("get_about_info");
      } catch (e) {
        activeError = mapTechnicalError(String(e), t);
      }
    }
  }

  async function copyDiagnostics() {
    try {
      const info: {
        version: string;
        os: string;
        arch: string;
        cpu_cores: number;
        total_ram_mb: number | null;
        available_ram_mb: number | null;
        tier: string;
      } = await invoke("get_diagnostic_info", { tier: selectedTier });

      const text = [
        `RustMatte v${info.version}`,
        `OS: ${info.os} (${info.arch})`,
        `CPU cores: ${info.cpu_cores}`,
        `RAM: ${info.total_ram_mb ?? "unknown"} MB total / ${info.available_ram_mb ?? "unknown"} MB available`,
        `Selected tier: ${info.tier}`,
      ].join("\n");

      await navigator.clipboard.writeText(text);
      diagnosticsCopied = true;
    } catch (e) {
      activeError = mapTechnicalError(String(e), t);
    }
  }

  function statusLabel(status: string): string {
    switch (status) {
      case "waiting":
        return t.statusWaiting;
      case "processing":
        return t.statusProcessing;
      case "done":
        return t.statusDone;
      case "warning":
        return t.statusWarning;
      default:
        return t.statusFailed;
    }
  }

  function resetViewTransform() {
    sliderState = resetSliderForNewPhoto(sliderState);
    zoom = 1;
    panX = 0;
    panY = 0;
  }

  function adjustZoom(delta: number) {
    zoom = Math.max(0.5, Math.min(4, Number((zoom + delta).toFixed(2))));
    if (zoom <= 1) {
      panX = 0;
      panY = 0;
    }
  }

  function handleCanvasWheel(e: WheelEvent) {
    if (!cutoutPreviewUrl) return;
    e.preventDefault();
    adjustZoom(e.deltaY < 0 ? 0.2 : -0.2);
  }

  function handleCanvasPointerMove(e: MouseEvent) {
    if (isDraggingSlider) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
      sliderState = setSliderPosition(sliderState, (x / rect.width) * 100);
      return;
    }
    if (isPanning && zoom > 1) {
      panX += e.clientX - panStartX;
      panY += e.clientY - panStartY;
      panStartX = e.clientX;
      panStartY = e.clientY;
    }
  }

  function bindViewportEvents(node: HTMLElement) {
    const onWheel = (e: WheelEvent) => handleCanvasWheel(e);
    const onMouseDown = (e: MouseEvent) => {
      if (zoom > 1 && !isDraggingSlider) {
        isPanning = true;
        panStartX = e.clientX;
        panStartY = e.clientY;
      }
    };
    const onMouseMove = (e: MouseEvent) => handleCanvasPointerMove(e);
    const stopDrag = () => {
      isDraggingSlider = false;
      isPanning = false;
    };

    node.addEventListener("wheel", onWheel, { passive: false });
    node.addEventListener("mousedown", onMouseDown);
    node.addEventListener("mousemove", onMouseMove);
    node.addEventListener("mouseup", stopDrag);
    node.addEventListener("mouseleave", stopDrag);

    return {
      destroy() {
        node.removeEventListener("wheel", onWheel);
        node.removeEventListener("mousedown", onMouseDown);
        node.removeEventListener("mousemove", onMouseMove);
        node.removeEventListener("mouseup", stopDrag);
        node.removeEventListener("mouseleave", stopDrag);
      },
    };
  }

  function handleSliderKeyDown(e: KeyboardEvent) {
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      sliderState = setSliderPosition(sliderState, sliderPos - 5);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      sliderState = setSliderPosition(sliderState, sliderPos + 5);
    } else if (e.key === "Home") {
      e.preventDefault();
      sliderState = setSliderPosition(sliderState, 0);
    } else if (e.key === "End") {
      e.preventDefault();
      sliderState = setSliderPosition(sliderState, 100);
    } else if (e.key === "+" || e.key === "=") {
      e.preventDefault();
      adjustZoom(0.25);
    } else if (e.key === "-") {
      e.preventDefault();
      adjustZoom(-0.25);
    } else if (e.key === "0") {
      e.preventDefault();
      resetViewTransform();
    } else if (zoom > 1 && e.shiftKey) {
      const step = 24;
      if (e.key === "ArrowUp") {
        e.preventDefault();
        panY += step;
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        panY -= step;
      }
    }
  }

  function handleGlobalKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (showAboutModal) {
        showAboutModal = false;
        return;
      }
      if (showFirstRunModal && !isDownloading) {
        showFirstRunModal = false;
        return;
      }
      if (isBusy) {
        cancelCurrentJob();
        return;
      }
    }

    if (showAboutModal || showFirstRunModal) return;

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      browsePhotos();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runRemoveBackground();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "s") {
      e.preventDefault();
      saveAllCutouts();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
      e.preventDefault();
      saveSelectedCutout();
      return;
    }

    const activeEl = document.activeElement;
    const isInput =
      activeEl instanceof HTMLInputElement ||
      activeEl instanceof HTMLSelectElement ||
      activeEl instanceof HTMLTextAreaElement ||
      activeEl?.getAttribute("role") === "slider";

    if (!isInput && !e.ctrlKey && !e.metaKey && !e.altKey && e.key.toLowerCase() === "c" && cutoutPreviewUrl && originalPreviewUrl) {
      e.preventDefault();
      sliderState = toggleCompareWithOriginal(sliderState);
      return;
    }

    if (!isInput && qState.items.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        qState = selectAdjacent(qState, 1);
        resetViewTransform();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        qState = selectAdjacent(qState, -1);
        resetViewTransform();
      }
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeyDown} />

<main class={resolvedDark ? "theme-dark" : "theme-light"}>
  <header class="app-header">
    <div class="brand">
      <h1 class="brand-title">{t.appTitle}</h1>
      <span class="brand-subtitle">{t.appSubtitle}</span>
      <span class="badge-pill">{t.offlineBadge}</span>
      {#if isActiveModelReady}
        <span class="badge-pill ok">{t.modelReadyBadge}</span>
      {:else}
        <button class="badge-pill warn-btn" onclick={() => openModelSetup(selectedTier)}>
          {t.modelMissingBadge}: {t.setupModelBtn}
        </button>
      {/if}
    </div>

    <div class="header-controls">
      <label class="sr-only" for="lang-select">{t.languageLabel}</label>
      <select id="lang-select" bind:value={lang} aria-label={t.languageLabel}>
        <option value="en">EN</option>
        <option value="id">ID</option>
      </select>

      <label class="sr-only" for="theme-select">{t.themeLabel}</label>
      <select id="theme-select" bind:value={themeMode} aria-label={t.themeLabel}>
        <option value="system">{t.themeSystem}</option>
        <option value="light">{t.themeLight}</option>
        <option value="dark">{t.themeDark}</option>
      </select>

      <button class="secondary-btn sm" onclick={openAbout}>{t.aboutBtn}</button>
    </div>
  </header>

  <div class="app-body">
    {#if qState.items.length === 0}
      <!-- Empty Dropzone State -->
      <section class="empty-stage" aria-label={t.dropTitle}>
        <div class="dropzone-card {isDragOver ? 'drag-over' : ''}">
          <h2>{t.dropTitle}</h2>
          <p class="muted">{t.dropSubtitle}</p>
          <div class="drop-actions">
            <button class="primary-btn" onclick={browsePhotos}>{t.browsePhotos}</button>
            <button class="secondary-btn" onclick={browseFolder}>{t.browseFolder}</button>
          </div>
          <p class="shortcut-note">{t.shortcutsHint}</p>
        </div>

        {#if activeError}
          <div class="banner error" role="alert">
            <div class="banner-body">
              <strong>{activeError.title}</strong>
              <p>{activeError.fix}</p>
              <details>
                <summary>{t.errorDetailsLabel}</summary>
                <code>{activeError.details}</code>
              </details>
            </div>
            <button class="text-btn" onclick={() => (activeError = null)}>{t.dismissBtn}</button>
          </div>
        {/if}

        {#if qState.rejectedItems.length > 0}
          <div class="banner warning" role="status">
            <div class="banner-body">
              <strong>{t.rejectedFilesBanner} ({qState.rejectedItems.length})</strong>
              <ul class="compact-list">
                {#each qState.rejectedItems.slice(0, 5) as rej}
                  <li>{rej.name}: {renderRejectedReason(rej, t)}</li>
                {/each}
              </ul>
            </div>
          </div>
        {/if}
      </section>
    {:else}
      <!-- Left Thumbnail Queue (Windowed Rendering) -->
      <aside class="queue-sidebar" aria-label={t.queueTitle}>
        <div class="panel-header">
          <h2>{t.queueTitle} ({qState.items.length})</h2>
          <div class="panel-header-actions">
            <button class="text-btn" onclick={browsePhotos} disabled={isBusy}>{t.addMoreBtn}</button>
            <button
              class="text-btn"
              onclick={() => (qState = clearQueueState(qState))}
              disabled={isBusy}
            >
              {t.clearQueue}
            </button>
          </div>
        </div>

        <div
          class="queue-scroll"
          bind:clientHeight={queueViewportHeight}
          onscroll={(e) => (queueScrollTop = (e.currentTarget as HTMLElement).scrollTop)}
        >
          <div style:height={`${qState.items.length * ROW_HEIGHT}px`} class="queue-spacer">
            <div
              class="queue-window"
              style:transform={`translateY(${visibleStart * ROW_HEIGHT}px)`}
            >
              {#each visibleItems as { item, index } (item.path)}
                {@const thumbPath = item.cutoutPreviewPath ?? item.originalPreviewPath}
                <button
                  type="button"
                  class="queue-row {index === qState.selectedIndex ? 'selected' : ''}"
                  onclick={() => {
                    qState = { ...qState, selectedIndex: index };
                    resetViewTransform();
                  }}
                >
                  <div class="thumb-box">
                    {#if thumbPath}
                      <img src={convertFileSrc(thumbPath)} alt="" loading="lazy" />
                    {:else}
                      <span>{t.thumbPlaceholder}</span>
                    {/if}
                  </div>
                  <div class="row-meta">
                    <span class="row-name" title={item.name}>{item.name}</span>
                    <span class="status-tag {item.status}">
                      {statusLabel(item.status)}
                      {#if item.timingMs}
                        ({(item.timingMs / 1000).toFixed(2)}s)
                      {/if}
                    </span>
                  </div>
                </button>
              {/each}
            </div>
          </div>
        </div>
      </aside>

      <!-- Center Workspace -->
      <section class="center-stage {isDragOver ? 'drag-over' : ''}">
        {#if isBusy}
          <div class="progress-bar-box" role="status" aria-live="polite">
            <div class="progress-text">
              <span>
                {qState.phase === "loading_model"
                  ? t.progressLoadingModel
                  : `${t.progressProcessing} (${summary.completed + summary.failed}/${summary.total})`}
              </span>
              <button class="secondary-btn sm" onclick={cancelCurrentJob}>
                {t.cancelBtn}
              </button>
            </div>
            <progress
              max={Math.max(1, summary.total)}
              value={qState.phase === "loading_model" ? undefined : summary.completed + summary.failed}
            ></progress>
          </div>
        {/if}

        {#if statusBanner}
          <div class="banner info" role="status">
            <span>{renderStatusBanner(statusBanner, t)}</span>
            <button class="text-btn" onclick={() => (statusBanner = null)}>{t.dismissBtn}</button>
          </div>
        {/if}

        {#if summary.oomWarnings > 0}
          <div class="banner warning" role="status">
            <span>{t.oomWarningBanner}</span>
          </div>
        {/if}

        {#if selectedItem && selectedItem.warnings.length > 0}
          <div class="banner warning" role="status">
            <span>
              {t.damagedWarningPrefix}: {selectedItem.warnings
                .map((w) => renderWarningItem(w, t))
                .join("; ")}
            </span>
          </div>
        {/if}

        {#if qState.rejectedItems.length > 0}
          <div class="banner warning" role="status">
            <div class="banner-body">
              <strong>{t.rejectedFilesBanner} ({qState.rejectedItems.length})</strong>
              <ul class="compact-list">
                {#each qState.rejectedItems.slice(0, 4) as rej}
                  <li>{rej.name}: {renderRejectedReason(rej, t)}</li>
                {/each}
              </ul>
            </div>
          </div>
        {/if}

        {#if activeError}
          <div class="banner error" role="alert">
            <div class="banner-body">
              <strong>{activeError.title}</strong>
              <p>{activeError.fix}</p>
              <details>
                <summary>{t.errorDetailsLabel}</summary>
                <code>{activeError.details}</code>
              </details>
            </div>
            <button class="text-btn" onclick={() => (activeError = null)}>{t.dismissBtn}</button>
          </div>
        {/if}

        {#if qState.phase === "partial_failure"}
          {@const failedItems = qState.items.filter((i) => i.status === "failed")}
          {#if failedItems.length > 0}
            <div class="banner error" role="region" aria-label={t.statusFailed}>
              <div class="banner-body">
                <strong>{t.statePartialSummary}</strong>
                <ul class="compact-list">
                  {#each failedItems.slice(0, 6) as f}
                    <li>{f.name}: {mapTechnicalError(f.error ?? "", t).title}</li>
                  {/each}
                </ul>
              </div>
            </div>
          {/if}
        {/if}

        {#if cutoutPreviewUrl && originalPreviewUrl}
          <div class="viewer-toolbar">
            <div class="zoom-controls">
              <button class="secondary-btn sm" onclick={() => adjustZoom(-0.25)}>{t.zoomOut}</button>
              <span class="zoom-readout">{Math.round(zoom * 100)}%</span>
              <button class="secondary-btn sm" onclick={() => adjustZoom(0.25)}>{t.zoomIn}</button>
              <button class="secondary-btn sm" onclick={resetViewTransform}>{t.zoomReset}</button>
              <button
                type="button"
                class="secondary-btn sm {isComparing ? 'toggle-active' : ''}"
                aria-pressed={isComparing}
                title={t.compareToggleHint}
                onclick={() => (sliderState = toggleCompareWithOriginal(sliderState))}
              >
                {isComparing ? t.compareToggleActiveBtn : t.compareToggleBtn}
              </button>
            </div>
            {#if zoom > 1}
              <span class="muted small">{t.panHint}</span>
            {/if}
          </div>

          <div
            class="slider-viewport {backgroundMode === 'transparent' ? 'checkerboard' : ''}"
            style:background-color={backgroundMode === "solid" ? backgroundHex : undefined}
            role="region"
            aria-label={t.sliderAriaLabel}
            use:bindViewportEvents
          >
            <div
              class="zoom-stage"
              style:transform={`translate(${panX}px, ${panY}px) scale(${zoom})`}
            >
              <img src={cutoutPreviewUrl} alt={t.cutoutLabel} class="comparison-img" draggable="false" />
              <div
                class="clipped-overlay"
                style:clip-path={`polygon(0 0, ${sliderPos}% 0, ${sliderPos}% 100%, 0 100%)`}
              >
                <img src={originalPreviewUrl} alt={t.originalLabel} class="comparison-img" draggable="false" />
              </div>
            </div>

            <div
              class="slider-handle {sliderPos <= 0 ? 'at-start' : sliderPos >= 100 ? 'at-end' : ''}"
              style:left={`${sliderPos}%`}
              role="slider"
              tabindex="0"
              aria-label={t.sliderAriaLabel}
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={Math.round(sliderPos)}
              aria-valuetext={`${Math.round(sliderPos)}%`}
              onmousedown={(e) => {
                e.stopPropagation();
                isDraggingSlider = true;
              }}
              onkeydown={handleSliderKeyDown}
            >
              <div class="handle-line"></div>
              <div class="handle-knob" aria-hidden="true">||</div>
            </div>

            {#if sliderPos > 8}
              <span class="canvas-tag left">{t.originalLabel}</span>
            {/if}
            {#if sliderPos < 92}
              <span class="canvas-tag right">{t.cutoutLabel}</span>
            {/if}
          </div>

          {#if selectedItem}
            <div class="stats-bar">
              <span><strong>{t.activeDeviceLabel}:</strong> {selectedItem.device}</span>
              <span><strong>{t.dimensionsLabel}:</strong> {selectedItem.width} x {selectedItem.height}</span>
              <span><strong>{t.inferenceTimeLabel}:</strong> {(selectedItem.inferenceMs ?? 0).toFixed(0)} ms</span>
              <span><strong>{t.totalTimeLabel}:</strong> {(selectedItem.timingMs ?? 0).toFixed(0)} ms</span>
            </div>
          {/if}
        {:else}
          <div class="preview-placeholder">
            <p class="muted">
              {selectedItem
                ? `${selectedItem.name} (${statusLabel(selectedItem.status)})`
                : t.stateEmptyHint}
            </p>
            <p class="shortcut-note">{t.shortcutsHint}</p>
          </div>
        {/if}
      </section>

      <!-- Right Options & Actions Panel -->
      <aside class="options-sidebar" aria-label={t.optionsTitle}>
        <h2>{t.optionsTitle}</h2>

        <div class="field">
          <label for="tier-select">{t.tierLabel}</label>
          <select id="tier-select" bind:value={selectedTier} disabled={isBusy}>
            <option value="quality">{t.tierQualityName}</option>
            <option value="fast">{t.tierFastName}</option>
          </select>
          <p class="field-help">
            {selectedTier === "quality" ? t.tierQualityDesc : t.tierFastDesc}
          </p>
        </div>

        <div class="field">
          <label for="refine-select">{t.refineLabel}</label>
          <select id="refine-select" bind:value={refineLevel} disabled={isBusy}>
            <option value="standard">{t.refineStandard}</option>
            <option value="light">{t.refineLight}</option>
            <option value="none">{t.refineNone}</option>
          </select>
        </div>

        <div class="field">
          <div class="folder-box" role="note">{t.deviceReadOnlyLine}</div>
        </div>

        <div class="field">
          <label for="max-side-select">{t.maxSideLabel}</label>
          <select id="max-side-select" bind:value={maxSide} disabled={isBusy}>
            <option value={null}>{t.maxSideOriginal}</option>
            <option value={2048}>{t.maxSide2048}</option>
            <option value={1024}>{t.maxSide1024}</option>
            <option value={512}>{t.maxSide512}</option>
          </select>
        </div>

        <div class="field">
          <span class="field-label">{t.backgroundLabel}</span>
          <div class="segmented">
            <button
              type="button"
              class="seg-btn {backgroundMode === 'transparent' ? 'active' : ''}"
              onclick={() => (backgroundMode = "transparent")}
            >
              {t.bgTransparent}
            </button>
            <button
              type="button"
              class="seg-btn {backgroundMode === 'solid' ? 'active' : ''}"
              onclick={() => (backgroundMode = "solid")}
            >
              {t.bgSolid}
            </button>
          </div>
          {#if backgroundMode === "solid"}
            <div class="color-row">
              <label for="bg-color-picker">{t.bgColorPickerLabel}</label>
              <input id="bg-color-picker" type="color" bind:value={backgroundHex} />
            </div>
          {/if}
        </div>

        <div class="field">
          <span class="field-label">{t.outputFolderLabel}</span>
          <div class="folder-box" title={outputFolder ? formatDisplayPath(outputFolder) : t.outputFolderAuto}>
            {outputFolder ? formatDisplayPath(outputFolder) : t.outputFolderAuto}
          </div>
          <div class="inline-actions">
            <button class="secondary-btn sm" onclick={chooseOutputFolder} disabled={isBusy}>
              {t.chooseFolder}
            </button>
            {#if outputFolder}
              <button class="secondary-btn sm" onclick={() => (outputFolder = null)} disabled={isBusy}>
                {t.resetFolder}
              </button>
            {/if}
          </div>
        </div>

        <div class="field checkbox-field">
          <label>
            <input type="checkbox" bind:checked={neverUseNetwork} />
            <span>{t.neverUseNetworkLabel}</span>
          </label>
          <p class="field-help">{t.neverUseNetworkHint}</p>
        </div>

        <div class="primary-actions">
          <button
            class="primary-btn full"
            onclick={runRemoveBackground}
            disabled={isBusy || qState.items.length === 0}
          >
            {t.removeBackgroundBtn}
          </button>

          <div class="save-row">
            <button
              class="secondary-btn"
              onclick={saveSelectedCutout}
              disabled={!selectedItem?.fullCutoutPath || isBusy}
            >
              {t.saveBtn}
            </button>
            <button
              class="secondary-btn"
              onclick={saveAllCutouts}
              disabled={summary.saveable === 0 || isBusy}
            >
              {t.saveAllBtn} ({summary.saveable})
            </button>
          </div>

          {#if qState.outputDir || outputFolder}
            <button class="text-btn" onclick={openOutputDir}>
              {t.openOutputFolderBtn}
            </button>
          {/if}
        </div>
      </aside>
    {/if}
  </div>

  <!-- First-Run Model Setup Dialog -->
  {#if showFirstRunModal}
    <div class="modal-backdrop" role="presentation">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="first-run-title">
        <h2 id="first-run-title">{t.firstRunTitle}</h2>
        <p>{t.firstRunIntro}</p>

        {#if setupModelInfo}
          <dl class="meta-grid">
            <dt>{t.firstRunFileLabel}</dt>
            <dd>{setupModelInfo.filename}</dd>
            <dt>{t.firstRunSizeLabel}</dt>
            <dd>{setupModelInfo.expected_size_mb.toFixed(1)} MB</dd>
            <dt>{t.firstRunHostLabel}</dt>
            <dd>{setupModelInfo.primary_host} ({t.firstRunFallbackLabel}: {setupModelInfo.fallback_host})</dd>
            <dt>{t.firstRunLicenseLabel}</dt>
            <dd>{setupModelInfo.license}</dd>
          </dl>
        {/if}

        {#if neverUseNetwork}
          <p class="banner warning">{t.firstRunBlockedOffline}</p>
        {/if}

        {#if isDownloading}
          <div class="download-progress">
            <span>{t.progressDownloading} ({qState.downloadProgress}%)</span>
            <progress max="100" value={qState.downloadProgress}></progress>
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

        <div class="modal-actions">
          {#if downloadError}
            <button
              class="primary-btn"
              onclick={startDownloadFromModal}
              disabled={isDownloading || neverUseNetwork}
            >
              {t.firstRunRetryBtn}
            </button>
          {:else}
            <button
              class="primary-btn"
              onclick={startDownloadFromModal}
              disabled={isDownloading || neverUseNetwork}
            >
              {t.firstRunDownloadBtn}
            </button>
          {/if}
          <button
            class="secondary-btn"
            onclick={importLocalModelFile}
            disabled={isDownloading}
          >
            {t.firstRunImportBtn}
          </button>
          <button
            class="text-btn"
            onclick={() => (showFirstRunModal = false)}
            disabled={isDownloading}
          >
            {t.firstRunCancelBtn}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- About & Licenses Modal -->
  {#if showAboutModal}
    <div class="modal-backdrop" role="presentation">
      <div class="modal modal-lg" role="dialog" aria-modal="true" aria-labelledby="about-title">
        <div class="modal-header">
          <h2 id="about-title">{t.aboutTitle}</h2>
          <button class="text-btn" onclick={() => (showAboutModal = false)}>{t.closeBtn}</button>
        </div>

        {#if aboutInfo}
          <div class="about-scroll">
            <p>
              <strong>{t.aboutVersionLabel}:</strong> {aboutInfo.version} |
              <strong>{t.aboutLicenseLabel}:</strong> {aboutInfo.license}
            </p>

            <h3>{t.aboutNetworkTitle}</h3>
            <p>{t.aboutNetworkBody}</p>

            <h3>{t.aboutModelsTitle}</h3>
            <pre class="doc-pre">{aboutInfo.models_markdown}</pre>

            <h3>{t.aboutThirdPartyTitle}</h3>
            <pre class="doc-pre">{aboutInfo.third_party_licenses_markdown}</pre>
          </div>
        {/if}

        <div class="modal-actions">
          <button class="secondary-btn" onclick={copyDiagnostics}>{t.copyDiagnosticsBtn}</button>
          {#if diagnosticsCopied}
            <span class="muted small">{t.diagnosticsCopied}</span>
          {/if}
          <button class="primary-btn" onclick={() => (showAboutModal = false)}>{t.closeBtn}</button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    overflow: hidden;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-app);
    color: var(--text-primary);
  }

  .theme-dark {
    --bg-app: #121417;
    --bg-panel: #1a1d22;
    --bg-elevated: #23272e;
    --border: #2f3540;
    --text-primary: #f1f5f9;
    --text-secondary: #a1aab8;
    --accent: #2563eb;
    --accent-hover: #1d4ed8;
    --checker-a: #1e2228;
    --checker-b: #272c34;
  }

  .theme-light {
    --bg-app: #f6f7f9;
    --bg-panel: #ffffff;
    --bg-elevated: #eef1f5;
    --border: #d5dce4;
    --text-primary: #14181f;
    --text-secondary: #4e5968;
    --accent: #1d4ed8;
    --accent-hover: #1e40af;
    --checker-a: #e5e7eb;
    --checker-b: #ffffff;
  }

  button:focus-visible,
  select:focus-visible,
  input:focus-visible,
  [role="slider"]:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
    gap: 12px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .brand-title {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
  }

  .brand-subtitle {
    font-size: 13px;
    color: var(--text-secondary);
  }

  .badge-pill {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 999px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-secondary);
  }

  .badge-pill.ok {
    color: #16a34a;
    border-color: rgba(22, 163, 74, 0.4);
  }

  .warn-btn {
    color: #d97706;
    border-color: rgba(217, 119, 6, 0.5);
    cursor: pointer;
  }

  .header-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  select,
  input[type="color"] {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 8px;
    font-size: 13px;
  }

  .app-body {
    display: flex;
    flex: 1;
    min-height: 0;
  }

  .empty-stage {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    gap: 16px;
  }

  .dropzone-card {
    width: min(640px, 92%);
    padding: 48px 32px;
    border: 2px dashed var(--border);
    border-radius: 12px;
    background: var(--bg-panel);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .dropzone-card.drag-over,
  .center-stage.drag-over {
    border-color: var(--accent);
    background: rgba(37, 99, 235, 0.08);
  }

  .drop-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: center;
    margin-top: 6px;
  }

  .shortcut-note {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 4px 0 0;
  }

  .queue-sidebar {
    width: 260px;
    background: var(--bg-panel);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }

  .panel-header h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .panel-header-actions {
    display: flex;
    gap: 8px;
  }

  .queue-scroll {
    flex: 1;
    overflow-y: auto;
    position: relative;
  }

  .queue-spacer {
    position: relative;
    width: 100%;
  }

  .queue-window {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
  }

  .queue-row {
    width: 100%;
    height: 62px;
    padding: 6px 10px;
    border: none;
    border-bottom: 1px solid var(--border);
    background: transparent;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 10px;
    text-align: left;
    cursor: pointer;
  }

  .queue-row.selected {
    background: rgba(37, 99, 235, 0.14);
  }

  .thumb-box {
    width: 44px;
    height: 44px;
    border-radius: 6px;
    background: var(--bg-elevated);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .thumb-box img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .row-meta {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .row-name {
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-tag {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .status-tag.done {
    color: #16a34a;
  }

  .status-tag.warning {
    color: #d97706;
  }

  .status-tag.failed {
    color: #dc2626;
  }

  .center-stage {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    padding: 12px;
    gap: 10px;
  }

  .progress-bar-box {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-text {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
  }

  progress {
    width: 100%;
    height: 8px;
  }

  .banner {
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 13px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
  }

  .banner.error {
    border-color: #dc2626;
    background: rgba(220, 38, 38, 0.12);
  }

  .banner.warning {
    border-color: #d97706;
    background: rgba(217, 119, 6, 0.12);
  }

  .banner.info {
    border-color: var(--accent);
    background: rgba(37, 99, 235, 0.12);
  }

  .banner-body p {
    margin: 4px 0;
  }

  .compact-list {
    margin: 4px 0 0;
    padding-left: 18px;
    font-size: 12px;
  }

  .viewer-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .zoom-controls {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .zoom-readout {
    min-width: 48px;
    text-align: center;
    font-size: 12px;
    font-weight: 600;
  }

  .slider-viewport {
    position: relative;
    flex: 1;
    min-height: 240px;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    user-select: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .checkerboard {
    background-image:
      linear-gradient(45deg, var(--checker-a) 25%, transparent 25%),
      linear-gradient(-45deg, var(--checker-a) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, var(--checker-a) 75%),
      linear-gradient(-45deg, transparent 75%, var(--checker-a) 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0;
    background-color: var(--checker-b);
  }

  .zoom-stage {
    position: relative;
    width: 100%;
    height: 100%;
    transform-origin: center center;
  }

  .comparison-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: contain;
    pointer-events: none;
  }

  .clipped-overlay {
    position: absolute;
    inset: 0;
  }

  .slider-handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 28px;
    transform: translateX(-50%);
    cursor: ew-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 5;
  }

  .handle-line {
    width: 2px;
    height: 100%;
    background: #ffffff;
    box-shadow: 0 0 4px rgba(0, 0, 0, 0.7);
  }

  .handle-knob {
    position: absolute;
    padding: 4px 6px;
    border-radius: 999px;
    background: var(--accent);
    color: #ffffff;
    font-size: 11px;
    font-weight: 700;
  }

  .slider-handle.at-start .handle-knob {
    transform: translateX(12px);
  }

  .slider-handle.at-end .handle-knob {
    transform: translateX(-12px);
  }

  .toggle-active {
    border-color: var(--accent);
    color: var(--accent);
    font-weight: 600;
  }

  .canvas-tag {
    position: absolute;
    bottom: 10px;
    padding: 3px 8px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.65);
    color: #ffffff;
    font-size: 11px;
    z-index: 4;
  }

  .canvas-tag.left {
    left: 10px;
  }

  .canvas-tag.right {
    right: 10px;
  }

  .stats-bar {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
    font-size: 12px;
    color: var(--text-secondary);
    padding: 4px 2px;
  }

  .preview-placeholder {
    flex: 1;
    border: 1px dashed var(--border);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 24px;
    text-align: center;
  }

  .options-sidebar {
    width: 300px;
    background: var(--bg-panel);
    border-left: 1px solid var(--border);
    padding: 12px 14px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .options-sidebar h2 {
    margin: 0;
    font-size: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .field label,
  .field-label {
    font-size: 12px;
    font-weight: 600;
  }

  .field-help,
  .muted {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .small {
    font-size: 11px;
  }

  .segmented {
    display: flex;
    gap: 6px;
  }

  .seg-btn {
    flex: 1;
    padding: 6px 8px;
    font-size: 12px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    color: var(--text-primary);
    cursor: pointer;
  }

  .seg-btn.active {
    background: var(--accent);
    color: #ffffff;
    border-color: var(--accent);
  }

  .color-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 4px;
  }

  .folder-box {
    font-size: 11px;
    color: var(--text-secondary);
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .inline-actions {
    display: flex;
    gap: 6px;
  }

  .checkbox-field label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .primary-actions {
    margin-top: auto;
    padding-top: 8px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .save-row {
    display: flex;
    gap: 8px;
  }

  .save-row button {
    flex: 1;
  }

  .primary-btn,
  .secondary-btn,
  .text-btn {
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    padding: 8px 12px;
    border: 1px solid transparent;
  }

  .primary-btn {
    background: var(--accent);
    color: #ffffff;
  }

  .primary-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .secondary-btn {
    background: var(--bg-elevated);
    color: var(--text-primary);
    border-color: var(--border);
  }

  .text-btn {
    background: transparent;
    color: var(--accent);
    padding: 4px 6px;
  }

  .sm {
    padding: 5px 9px;
    font-size: 12px;
  }

  .full {
    width: 100%;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

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

  .modal {
    width: min(540px, 100%);
    max-height: 88vh;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .modal-lg {
    width: min(760px, 100%);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal h2 {
    margin: 0;
    font-size: 16px;
  }

  .meta-grid {
    display: grid;
    grid-template-columns: 120px 1fr;
    gap: 6px 12px;
    margin: 0;
    font-size: 13px;
  }

  .meta-grid dt {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .meta-grid dd {
    margin: 0;
  }

  .modal-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-end;
  }

  .about-scroll {
    overflow-y: auto;
    max-height: 60vh;
    padding-right: 4px;
    font-size: 13px;
  }

  .doc-pre {
    white-space: pre-wrap;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 11px;
    background: var(--bg-elevated);
    padding: 10px;
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  @media (prefers-reduced-motion: reduce) {
    * {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
