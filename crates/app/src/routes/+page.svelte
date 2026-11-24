<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  // Internationalization dictionaries
  const i18n = {
    en: {
      title: "RustMatte",
      subtitle: "Offline AI Background Remover",
      offlineBadge: "100% Offline & Private",
      dropTitle: "Drag and drop an image here",
      dropSubtitle: "Supports JPG, JPEG, PNG, WebP, BMP",
      chooseFile: "Browse File",
      tierLabel: "Model Tier",
      tierQuality: "Quality (BiRefNet Lite 512)",
      tierFast: "Fast (U-2-Netp Preview)",
      tierFastCaveat: "Best for people and products on clean backgrounds.",
      deviceLabel: "Execution Device",
      maxSideLabel: "Output Resolution",
      maxSideOriginal: "Original Size (Full Resolution)",
      refineLabel: "Edge Refinement & Defringing",
      processBtn: "Remove Background",
      processing: "Processing neural inference...",
      saveBtn: "Save Transparent PNG",
      newImageBtn: "Process Another Image",
      original: "Original",
      cutout: "Removed Background",
      statsTitle: "Execution Metrics",
      device: "Active Device",
      dimensions: "Dimensions",
      totalTime: "Total Time",
      inferenceTime: "Inference Time",
      modelNotCached: "Model not downloaded yet",
      downloadModel: "Download Model",
      downloading: "Downloading model...",
      previewBg: "Preview Background",
      bgTransparent: "Transparent",
      bgWhite: "White",
      bgBlack: "Black",
      bgCustom: "Custom",
      queueTitle: "Batch Queue",
      clearQueue: "Clear Queue",
    },
    id: {
      title: "RustMatte",
      subtitle: "Penghapus Latar Belakang AI Offline",
      offlineBadge: "100% Offline & Privasi Terjaga",
      dropTitle: "Tarik dan letakkan gambar di sini",
      dropSubtitle: "Mendukung JPG, JPEG, PNG, WebP, BMP",
      chooseFile: "Pilih Berkas",
      tierLabel: "Tingkat Model",
      tierQuality: "Quality (BiRefNet Lite 512)",
      tierFast: "Fast (U-2-Netp Pratinjau)",
      tierFastCaveat: "Terbaik untuk objek dan orang dengan latar belakang bersih.",
      deviceLabel: "Perangkat Inferensi",
      maxSideLabel: "Resolusi Output",
      maxSideOriginal: "Ukuran Asli (Resolusi Penuh)",
      refineLabel: "Penghalus Tepi & Defringing",
      processBtn: "Hapus Latar Belakang",
      processing: "Memproses inferensi neural...",
      saveBtn: "Simpan PNG Transparan",
      newImageBtn: "Proses Gambar Lain",
      original: "Asli",
      cutout: "Tanpa Latar Belakang",
      statsTitle: "Metrik Eksekusi",
      device: "Perangkat Aktif",
      dimensions: "Dimensi",
      totalTime: "Waktu Total",
      inferenceTime: "Waktu Inferensi",
      modelNotCached: "Model belum terunduh",
      downloadModel: "Unduh Model",
      downloading: "Mengunduh model...",
      previewBg: "Latar Belakang Pratinjau",
      bgTransparent: "Transparan",
      bgWhite: "Putih",
      bgBlack: "Hitam",
      bgCustom: "Kustom",
      queueTitle: "Antrean Batch",
      clearQueue: "Hapus Antrean",
    },
  };

  type Lang = "en" | "id";
  let lang = $state<Lang>("en");
  let t = $derived(i18n[lang]);
  let isDark = $state(true);

  // Settings
  let selectedTier = $state("quality");
  let selectedDevice = $state("auto");
  let maxSide = $state<number | null>(null);
  let enableRefine = $state(true);
  let previewBgColor = $state("transparent");
  let customBgHex = $state("#ffffff");

  // Model cache state
  let qualityCached = $state(false);
  let fastCached = $state(false);
  let isDownloading = $state(false);
  let downloadProgress = $state(0);

  // Active processing state
  let currentFilePath = $state<string | null>(null);
  let isProcessing = $state(false);
  let errorMessage = $state<string | null>(null);

  // Result state
  let originalUrl = $state<string | null>(null);
  let cutoutUrl = $state<string | null>(null);
  let imgWidth = $state(0);
  let imgHeight = $state(0);
  let activeDevice = $state("");
  let totalDurationMs = $state(0);
  let inferenceDurationMs = $state(0);
  let warningMessages = $state<string[]>([]);

  // Slider state
  let sliderPos = $state(50); // percentage 0-100
  let isDraggingSlider = $state(false);

  // Queue state
  interface QueueItem {
    id: string;
    path: string;
    name: string;
    status: "pending" | "processing" | "done" | "error";
    durationMs?: number;
    errorMsg?: string;
  }
  let queue = $state<QueueItem[]>([]);

  onMount(async () => {
    await checkModelStatus();

    await listen("download_progress", (event: any) => {
      if (event.payload) {
        downloadProgress = event.payload.percentage;
      }
    });
  });

  async function checkModelStatus() {
    try {
      const res: any = await invoke("get_model_cache_status");
      qualityCached = res.quality_cached;
      fastCached = res.fast_cached;
    } catch (e: any) {
      console.error("Failed to check model status:", e);
    }
  }

  async function handleDownloadModel(tier: string) {
    isDownloading = true;
    downloadProgress = 0;
    errorMessage = null;
    try {
      await invoke("download_model_tier", { tier });
      await checkModelStatus();
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      isDownloading = false;
    }
  }

  async function onFileSelected(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      const files = Array.from(target.files);
      if (files.length === 1) {
        const file = files[0];
        // @ts-ignore
        const fullPath = file.path || file.name;
        currentFilePath = fullPath;
        await processSingle(fullPath);
      } else {
        // Multi-file batch queue
        for (const file of files) {
          // @ts-ignore
          const fullPath = file.path || file.name;
          queue.push({
            id: Math.random().toString(36).substring(2, 9),
            path: fullPath,
            name: file.name,
            status: "pending",
          });
        }
      }
    }
  }

  async function processSingle(path: string) {
    isProcessing = true;
    errorMessage = null;
    originalUrl = null;
    cutoutUrl = null;

    try {
      const res: any = await invoke("process_single_image", {
        req: {
          input_path: path,
          tier: selectedTier,
          device: selectedDevice,
          max_side: maxSide,
          no_refine: !enableRefine,
        },
      });

      originalUrl = res.original_data_url;
      cutoutUrl = res.cutout_data_url;
      imgWidth = res.width;
      imgHeight = res.height;
      activeDevice = res.device;
      totalDurationMs = res.total_ms;
      inferenceDurationMs = res.inference_ms;
      warningMessages = res.warnings || [];
      sliderPos = 50;
    } catch (e: any) {
      errorMessage = String(e);
    } finally {
      isProcessing = false;
    }
  }

  async function saveCutout() {
    if (!cutoutUrl) return;
    try {
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
      const defaultDest = `rustmatte_${timestamp}.png`;
      await invoke("save_cutout_png", {
        dataUrl: cutoutUrl,
        destinationPath: defaultDest,
      });
      alert(`Cutout saved to: ${defaultDest}`);
    } catch (e: any) {
      alert(`Save failed: ${e}`);
    }
  }

  function handleSliderMouseMove(e: MouseEvent) {
    if (!isDraggingSlider) return;
    const container = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - container.left, container.width));
    sliderPos = (x / container.width) * 100;
  }
</script>

<main class={isDark ? "theme-dark" : "theme-light"}>
  <!-- App Header -->
  <header class="app-header">
    <div class="brand">
      <h1 class="brand-title">{t.title}</h1>
      <span class="brand-subtitle">{t.subtitle}</span>
    </div>

    <div class="header-controls">
      <span class="privacy-badge">{t.offlineBadge}</span>

      <button
        class="icon-btn"
        onclick={() => (lang = lang === "en" ? "id" : "en")}
        aria-label="Toggle Language"
      >
        {lang.toUpperCase()}
      </button>

      <button
        class="icon-btn"
        onclick={() => (isDark = !isDark)}
        aria-label="Toggle Theme"
      >
        {isDark ? "Light" : "Dark"}
      </button>
    </div>
  </header>

  <div class="app-body">
    <!-- Sidebar Controls -->
    <aside class="sidebar">
      <div class="control-group">
        <label for="tier-select">{t.tierLabel}</label>
        <select id="tier-select" bind:value={selectedTier}>
          <option value="quality">{t.tierQuality}</option>
          <option value="fast">{t.tierFast}</option>
        </select>
        {#if selectedTier === "fast"}
          <span class="caveat-text">{t.tierFastCaveat}</span>
        {/if}

        <!-- Cache check & download -->
        {#if selectedTier === "quality" && !qualityCached}
          <div class="model-status-card">
            <span>{t.modelNotCached} (183 MB)</span>
            <button
              class="primary-btn sm"
              disabled={isDownloading}
              onclick={() => handleDownloadModel("quality")}
            >
              {isDownloading ? `${t.downloading} ${downloadProgress}%` : t.downloadModel}
            </button>
          </div>
        {:else if selectedTier === "fast" && !fastCached}
          <div class="model-status-card">
            <span>{t.modelNotCached} (4.4 MB)</span>
            <button
              class="primary-btn sm"
              disabled={isDownloading}
              onclick={() => handleDownloadModel("fast")}
            >
              {isDownloading ? `${t.downloading} ${downloadProgress}%` : t.downloadModel}
            </button>
          </div>
        {/if}
      </div>

      <div class="control-group">
        <label for="device-select">{t.deviceLabel}</label>
        <select id="device-select" bind:value={selectedDevice}>
          <option value="auto">Auto (GPU fallback CPU)</option>
          <option value="directml">DirectML (DirectX 12 GPU)</option>
          <option value="cpu">CPU (Multi-threaded)</option>
        </select>
      </div>

      <div class="control-group">
        <label for="max-side-select">{t.maxSideLabel}</label>
        <select id="max-side-select" bind:value={maxSide}>
          <option value={null}>{t.maxSideOriginal}</option>
          <option value={2048}>Max 2048px</option>
          <option value={1024}>Max 1024px (Fast Mode)</option>
          <option value={512}>Max 512px</option>
        </select>
      </div>

      <div class="control-group checkbox-group">
        <label>
          <input type="checkbox" bind:checked={enableRefine} />
          {t.refineLabel}
        </label>
      </div>

      <div class="control-group">
        <span class="group-title">{t.previewBg}</span>
        <div class="bg-options">
          <button
            class="chip {previewBgColor === 'transparent' ? 'active' : ''}"
            onclick={() => (previewBgColor = 'transparent')}
          >
            {t.bgTransparent}
          </button>
          <button
            class="chip {previewBgColor === 'white' ? 'active' : ''}"
            onclick={() => (previewBgColor = 'white')}
          >
            {t.bgWhite}
          </button>
          <button
            class="chip {previewBgColor === 'black' ? 'active' : ''}"
            onclick={() => (previewBgColor = 'black')}
          >
            {t.bgBlack}
          </button>
          <button
            class="chip {previewBgColor === 'custom' ? 'active' : ''}"
            onclick={() => (previewBgColor = 'custom')}
          >
            {t.bgCustom}
          </button>
        </div>
        {#if previewBgColor === 'custom'}
          <input type="color" bind:value={customBgHex} class="color-picker" />
        {/if}
      </div>

      {#if cutoutUrl}
        <div class="stats-panel">
          <h3>{t.statsTitle}</h3>
          <div class="stat-row">
            <span>{t.device}:</span>
            <strong>{activeDevice}</strong>
          </div>
          <div class="stat-row">
            <span>{t.dimensions}:</span>
            <span>{imgWidth} x {imgHeight} px</span>
          </div>
          <div class="stat-row">
            <span>{t.inferenceTime}:</span>
            <span>{inferenceDurationMs.toFixed(1)} ms</span>
          </div>
          <div class="stat-row">
            <span>{t.totalTime}:</span>
            <span>{totalDurationMs.toFixed(1)} ms</span>
          </div>
        </div>
      {/if}
    </aside>

    <!-- Main Workspace Area -->
    <section class="workspace">
      {#if errorMessage}
        <div class="alert-banner error">
          <span>{errorMessage}</span>
          <button onclick={() => (errorMessage = null)}>Dismiss</button>
        </div>
      {/if}

      {#if warningMessages.length > 0}
        <div class="alert-banner warning">
          {#each warningMessages as w}
            <span>{w}</span>
          {/each}
        </div>
      {/if}

      {#if isProcessing}
        <div class="loading-container">
          <div class="spinner"></div>
          <p>{t.processing}</p>
        </div>
      {:else if cutoutUrl && originalUrl}
        <!-- Before / After Slider -->
        <div
          class="slider-viewport"
          role="region"
          aria-label="Before and after comparison slider"
          onmousemove={handleSliderMouseMove}
          onmouseup={() => (isDraggingSlider = false)}
          onmouseleave={() => (isDraggingSlider = false)}
        >
          <!-- Background layer for transparent cutout -->
          <div
            class="preview-canvas"
            style:background-color={previewBgColor === 'white'
              ? '#ffffff'
              : previewBgColor === 'black'
              ? '#000000'
              : previewBgColor === 'custom'
              ? customBgHex
              : 'transparent'}
            class:checkerboard={previewBgColor === 'transparent'}
          >
            <!-- Cutout Image (Base) -->
            <img src={cutoutUrl} alt="Background Removed" class="comparison-img" />

            <!-- Original Image (Clipped Overlay) -->
            <div
              class="clipped-overlay"
              style:clip-path={`polygon(0 0, ${sliderPos}% 0, ${sliderPos}% 100%, 0 100%)`}
            >
              <img src={originalUrl} alt="Original" class="comparison-img" />
            </div>

            <!-- Divider Line and Handle -->
            <div
              class="slider-handle"
              style:left={`${sliderPos}%`}
              onmousedown={() => (isDraggingSlider = true)}
              role="slider"
              tabindex="0"
              aria-valuenow={sliderPos}
            >
              <div class="handle-line"></div>
              <div class="handle-button">
                <span>◀ ▶</span>
              </div>
            </div>

            <!-- Labels -->
            <span class="view-tag left">{t.original}</span>
            <span class="view-tag right">{t.cutout}</span>
          </div>
        </div>

        <!-- Action bar -->
        <div class="action-bar">
          <button class="primary-btn" onclick={saveCutout}>
            {t.saveBtn}
          </button>
          <button
            class="secondary-btn"
            onclick={() => {
              cutoutUrl = null;
              originalUrl = null;
              currentFilePath = null;
            }}
          >
            {t.newImageBtn}
          </button>
        </div>
      {:else}
        <!-- Dropzone View -->
        <div class="dropzone-container">
          <label class="dropzone">
            <input
              type="file"
              accept=".jpg,.jpeg,.png,.webp,.bmp"
              multiple
              onchange={onFileSelected}
              class="hidden-file-input"
            />
            <div class="dropzone-content">
              <div class="dropzone-icon">⤓</div>
              <h2>{t.dropTitle}</h2>
              <p class="sub-text">{t.dropSubtitle}</p>
              <span class="file-browse-btn">{t.chooseFile}</span>
            </div>
          </label>
        </div>
      {/if}

      <!-- Batch queue if multiple files selected -->
      {#if queue.length > 0}
        <div class="queue-panel">
          <div class="queue-header">
            <h3>{t.queueTitle} ({queue.length})</h3>
            <button class="text-btn" onclick={() => (queue = [])}>{t.clearQueue}</button>
          </div>
          <div class="queue-list">
            {#each queue as item}
              <div class="queue-item {item.status}">
                <span class="name">{item.name}</span>
                <span class="status-pill {item.status}">{item.status}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </section>
  </div>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    font-size: 14px;
    user-select: none;
  }

  /* Theme variables */
  .theme-dark {
    --bg-primary: #121214;
    --bg-secondary: #1a1a1e;
    --bg-tertiary: #26262b;
    --border-color: #333338;
    --text-primary: #f0f0f2;
    --text-secondary: #9e9ea6;
    --accent: #2563eb;
    --accent-hover: #1d4ed8;
    --card-bg: #1f1f24;
  }

  .theme-light {
    --bg-primary: #f8f9fa;
    --bg-secondary: #ffffff;
    --bg-tertiary: #e9ecef;
    --border-color: #dee2e6;
    --text-primary: #212529;
    --text-secondary: #6c757d;
    --accent: #2563eb;
    --accent-hover: #1d4ed8;
    --card-bg: #ffffff;
  }

  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .brand-title {
    font-size: 18px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.3px;
  }

  .brand-subtitle {
    font-size: 12px;
    color: var(--text-secondary);
    margin-left: 8px;
  }

  .header-controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .privacy-badge {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 12px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .icon-btn {
    padding: 5px 10px;
    border-radius: 6px;
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 290px;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .control-group label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  select {
    padding: 8px 10px;
    border-radius: 6px;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 13px;
    outline: none;
  }

  .caveat-text {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .model-status-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    border-radius: 6px;
    background-color: var(--card-bg);
    border: 1px dashed var(--border-color);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .bg-options {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .chip {
    padding: 4px 8px;
    border-radius: 4px;
    border: 1px solid var(--border-color);
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 11px;
  }

  .chip.active {
    background-color: var(--accent);
    color: #ffffff;
    border-color: var(--accent);
  }

  .color-picker {
    margin-top: 4px;
    width: 100%;
    height: 30px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .stats-panel {
    margin-top: auto;
    padding: 12px;
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .stats-panel h3 {
    font-size: 12px;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .stat-row strong, .stat-row span:last-child {
    color: var(--text-primary);
  }

  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background-color: var(--bg-primary);
    padding: 20px;
    position: relative;
    overflow: hidden;
  }

  .dropzone-container {
    width: 100%;
    max-width: 600px;
    height: 380px;
  }

  .dropzone {
    width: 100%;
    height: 100%;
    border: 2px dashed var(--border-color);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.2s, background-color 0.2s;
  }

  .dropzone:hover {
    border-color: var(--accent);
    background-color: var(--bg-secondary);
  }

  .hidden-file-input {
    display: none;
  }

  .dropzone-content {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .dropzone-icon {
    font-size: 48px;
    color: var(--accent);
    line-height: 1;
  }

  .dropzone-content h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .sub-text {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .file-browse-btn {
    margin-top: 10px;
    padding: 8px 16px;
    background-color: var(--accent);
    color: #ffffff;
    border-radius: 6px;
    font-weight: 500;
  }

  .slider-viewport {
    flex: 1;
    width: 100%;
    max-width: 900px;
    max-height: calc(100vh - 160px);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .preview-canvas {
    position: relative;
    max-width: 100%;
    max-height: 100%;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }

  .checkerboard {
    background-image: linear-gradient(45deg, #ccc 25%, transparent 25%),
      linear-gradient(-45deg, #ccc 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #ccc 75%),
      linear-gradient(-45deg, transparent 75%, #ccc 75%);
    background-size: 16px 16px;
    background-position: 0 0, 0 8px, 8px -8px, -8px 0px;
  }

  .comparison-img {
    display: block;
    max-width: 100%;
    max-height: calc(100vh - 180px);
    object-fit: contain;
    user-select: none;
    pointer-events: none;
  }

  .clipped-overlay {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .slider-handle {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 2px;
    transform: translateX(-50%);
    cursor: ew-resize;
    z-index: 10;
  }

  .handle-line {
    width: 2px;
    height: 100%;
    background-color: #ffffff;
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.5);
  }

  .handle-button {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background-color: #ffffff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    color: #1a1a1e;
    font-weight: 700;
  }

  .view-tag {
    position: absolute;
    bottom: 12px;
    padding: 4px 10px;
    border-radius: 4px;
    background-color: rgba(0, 0, 0, 0.7);
    color: #ffffff;
    font-size: 11px;
    font-weight: 600;
  }

  .view-tag.left {
    left: 12px;
  }

  .view-tag.right {
    right: 12px;
  }

  .action-bar {
    display: flex;
    gap: 12px;
    margin-top: 14px;
  }

  .primary-btn {
    padding: 9px 18px;
    border-radius: 6px;
    background-color: var(--accent);
    color: #ffffff;
    border: none;
    cursor: pointer;
    font-weight: 600;
  }

  .primary-btn.sm {
    padding: 6px 12px;
    font-size: 12px;
  }

  .secondary-btn {
    padding: 9px 18px;
    border-radius: 6px;
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    cursor: pointer;
    font-weight: 600;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .alert-banner {
    position: absolute;
    top: 12px;
    left: 20px;
    right: 20px;
    padding: 10px 14px;
    border-radius: 6px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    z-index: 20;
  }

  .alert-banner.error {
    background-color: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    color: #ef4444;
  }

  .alert-banner.warning {
    background-color: rgba(245, 158, 11, 0.15);
    border: 1px solid #f59e0b;
    color: #f59e0b;
  }
</style>
