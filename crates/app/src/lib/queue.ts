export type AppPhase =
  | "empty"
  | "model_missing"
  | "downloading_model"
  | "loading_model"
  | "processing"
  | "done"
  | "partial_failure"
  | "cancelled";

export type QueueItemStatus =
  | "waiting"
  | "processing"
  | "done"
  | "warning"
  | "failed";

export interface ItemWarning {
  code: string;
  detail: string;
}

export type ModelTierId = "quality" | "fast" | "maximum";

export interface TierCutoutResult {
  tier: ModelTierId;
  cutoutPreviewPath: string;
  fullCutoutPath: string;
  timingMs: number;
  inferenceMs: number;
  width: number;
  height: number;
  device: string;
  refineLevel?: "none" | "light" | "standard";
  maxSide?: number | null;
}

export interface QueueItem {
  path: string;
  name: string;
  status: QueueItemStatus;
  timingMs?: number;
  inferenceMs?: number;
  warnings: Array<ItemWarning | string>;
  oomWarning: boolean;
  error?: string;
  originalPreviewPath?: string;
  cutoutPreviewPath?: string;
  fullCutoutPath?: string;
  width?: number;
  height?: number;
  device?: string;
  processedTier?: ModelTierId;
  processedRefine?: "none" | "light" | "standard";
  processedMaxSide?: number | null;
  resultsByTier?: Partial<Record<ModelTierId, TierCutoutResult>>;
}

export interface RejectedItem {
  path: string;
  name: string;
  reason_code?: string;
  reason: string;
}

export interface QueueState {
  phase: AppPhase;
  items: QueueItem[];
  selectedIndex: number;
  downloadProgress: number;
  rejectedItems: RejectedItem[];
  lastError: string | null;
  outputDir: string | null;
  displayOutputDir: string | null;
}

export interface ProgressPayload {
  index: number;
  total: number;
  file: string;
  input_path: string;
  status: string;
  timing_ms: number;
  inference_ms: number;
  warnings: Array<ItemWarning | string>;
  oom_warning: boolean;
  error?: string | null;
  original_preview_path?: string | null;
  cutout_preview_path?: string | null;
  full_cutout_path?: string | null;
  width: number;
  height: number;
  device: string;
  tier?: string;
  refine_level?: string;
  max_side?: number | null;
}

export interface FinishedPayload {
  processed: number;
  failed: number;
  cancelled: boolean;
  total_ms: number;
  output_dir?: string | null;
  display_output_dir?: string | null;
}

export function createInitialQueueState(): QueueState {
  return {
    phase: "empty",
    items: [],
    selectedIndex: -1,
    downloadProgress: 0,
    rejectedItems: [],
    lastError: null,
    outputDir: null,
    displayOutputDir: null,
  };
}

function baseName(p: string): string {
  const parts = p.split(/[/\\]/);
  return parts[parts.length - 1] || p;
}

export function enqueuePaths(
  state: QueueState,
  paths: string[],
  rejected: RejectedItem[] = []
): QueueState {
  const existing = new Set(state.items.map((i) => i.path));
  const appended: QueueItem[] = [];
  for (const p of paths) {
    if (!existing.has(p)) {
      existing.add(p);
      appended.push({
        path: p,
        name: baseName(p),
        status: "waiting",
        warnings: [],
        oomWarning: false,
      });
    }
  }

  const nextItems = [...state.items, ...appended];
  const nextSelected =
    nextItems.length === 0
      ? -1
      : state.selectedIndex >= 0
      ? state.selectedIndex
      : 0;

  return {
    ...state,
    phase: nextItems.length === 0 ? "empty" : state.phase === "empty" ? "done" : state.phase,
    items: nextItems,
    selectedIndex: nextSelected,
    rejectedItems: [...state.rejectedItems, ...rejected],
    lastError: null,
  };
}

export function markModelMissing(state: QueueState): QueueState {
  return { ...state, phase: "model_missing", downloadProgress: 0 };
}

export function startModelDownload(state: QueueState): QueueState {
  return {
    ...state,
    phase: "downloading_model",
    downloadProgress: 0,
    lastError: null,
  };
}

export function setDownloadProgress(state: QueueState, pct: number): QueueState {
  return {
    ...state,
    downloadProgress: Math.max(0, Math.min(100, pct)),
  };
}

export function failModelDownload(state: QueueState, error: string): QueueState {
  return {
    ...state,
    phase: "model_missing",
    lastError: error,
  };
}

export function startJob(state: QueueState, loadingModelFirst: boolean): QueueState {
  return {
    ...state,
    phase: loadingModelFirst ? "loading_model" : "processing",
    lastError: null,
    outputDir: null,
    displayOutputDir: null,
    items: state.items.map((item) => ({
      ...item,
      status: "waiting",
      timingMs: undefined,
      inferenceMs: undefined,
      warnings: [],
      oomWarning: false,
      error: undefined,
      originalPreviewPath: undefined,
      cutoutPreviewPath: undefined,
      fullCutoutPath: undefined,
      width: undefined,
      height: undefined,
      device: undefined,
      processedTier: undefined,
      processedRefine: undefined,
      processedMaxSide: undefined,
      resultsByTier: undefined,
    })),
  };
}

export function setJobPhase(
  state: QueueState,
  phase: "loading_model" | "processing"
): QueueState {
  return { ...state, phase };
}

function normalizeProcessedTier(raw?: string): ModelTierId | undefined {
  if (raw === "fast") return "fast";
  if (raw === "quality") return "quality";
  if (raw === "maximum") return "maximum";
  return undefined;
}

function normalizeProcessedRefine(
  raw?: string
): "none" | "light" | "standard" | undefined {
  if (raw === "none" || raw === "light" || raw === "standard") return raw;
  return undefined;
}

function buildUpdatedItemFromPayload(
  item: QueueItem,
  payload: ProgressPayload
): QueueItem {
  const mappedStatus: QueueItemStatus =
    payload.status === "failed"
      ? "failed"
      : payload.status === "warning"
      ? "warning"
      : "done";
  const tier = normalizeProcessedTier(payload.tier);
  const refineLevel = normalizeProcessedRefine(payload.refine_level);
  const maxSide =
    payload.max_side === undefined ? undefined : payload.max_side;

  let nextResultsByTier = item.resultsByTier;
  if (
    tier &&
    (mappedStatus === "done" || mappedStatus === "warning") &&
    payload.cutout_preview_path &&
    payload.full_cutout_path
  ) {
    nextResultsByTier = {
      ...(item.resultsByTier ?? {}),
      [tier]: {
        tier,
        cutoutPreviewPath: payload.cutout_preview_path,
        fullCutoutPath: payload.full_cutout_path,
        timingMs: payload.timing_ms,
        inferenceMs: payload.inference_ms,
        width: payload.width,
        height: payload.height,
        device: payload.device,
        refineLevel,
        maxSide,
      },
    };
  }

  return {
    ...item,
    status: mappedStatus,
    timingMs: payload.timing_ms,
    inferenceMs: payload.inference_ms,
    warnings: payload.warnings ?? [],
    oomWarning: Boolean(payload.oom_warning),
    error: payload.error ?? undefined,
    originalPreviewPath:
      payload.original_preview_path ?? item.originalPreviewPath,
    cutoutPreviewPath: payload.cutout_preview_path ?? undefined,
    fullCutoutPath: payload.full_cutout_path ?? undefined,
    width: payload.width || undefined,
    height: payload.height || undefined,
    device: payload.device || undefined,
    processedTier: tier,
    processedRefine: refineLevel,
    processedMaxSide: maxSide,
    resultsByTier: nextResultsByTier,
  };
}

export function applyProgressEvent(
  state: QueueState,
  payload: ProgressPayload
): QueueState {
  let matchedIdx = -1;
  const nextItems = state.items.map((item, idx) => {
    if (item.path !== payload.input_path) {
      return item;
    }
    matchedIdx = idx;
    return buildUpdatedItemFromPayload(item, payload);
  });

  const nextSelected =
    matchedIdx >= 0 && payload.cutout_preview_path
      ? matchedIdx
      : state.selectedIndex;

  return {
    ...state,
    phase: "processing",
    items: nextItems,
    selectedIndex: nextSelected,
    lastError:
      payload.status === "failed" && payload.error
        ? `${payload.file}: ${payload.error}`
        : state.lastError,
  };
}

export function applySingleRetryResult(
  state: QueueState,
  payload: ProgressPayload
): QueueState {
  const nextItems = state.items.map((item) => {
    if (item.path !== payload.input_path) {
      return item;
    }
    return buildUpdatedItemFromPayload(item, payload);
  });
  return {
    ...state,
    items: nextItems,
    lastError:
      payload.status === "failed" && payload.error
        ? `${payload.file}: ${payload.error}`
        : state.lastError,
  };
}

export function selectItemModelResult(
  state: QueueState,
  itemPath: string,
  tier: ModelTierId
): QueueState {
  const nextItems = state.items.map((item) => {
    if (item.path !== itemPath) {
      return item;
    }
    const chosen = item.resultsByTier?.[tier];
    if (!chosen) {
      return item;
    }
    return {
      ...item,
      cutoutPreviewPath: chosen.cutoutPreviewPath,
      fullCutoutPath: chosen.fullCutoutPath,
      timingMs: chosen.timingMs,
      inferenceMs: chosen.inferenceMs,
      width: chosen.width,
      height: chosen.height,
      device: chosen.device,
      processedTier: chosen.tier,
      processedRefine: chosen.refineLevel,
      processedMaxSide: chosen.maxSide,
    };
  });
  return { ...state, items: nextItems };
}

export function finishJob(
  state: QueueState,
  payload: FinishedPayload
): QueueState {
  const phase: AppPhase = payload.cancelled
    ? "cancelled"
    : payload.failed > 0
    ? "partial_failure"
    : state.items.length === 0
    ? "empty"
    : "done";

  return {
    ...state,
    phase,
    outputDir: payload.output_dir ?? state.outputDir,
    displayOutputDir:
      payload.display_output_dir ?? payload.output_dir ?? state.displayOutputDir,
  };
}

export function selectAdjacent(state: QueueState, delta: -1 | 1): QueueState {
  if (state.items.length === 0) return state;
  const current = state.selectedIndex < 0 ? 0 : state.selectedIndex;
  const next = Math.max(0, Math.min(state.items.length - 1, current + delta));
  return { ...state, selectedIndex: next };
}

export function clearQueue(state: QueueState): QueueState {
  return {
    ...createInitialQueueState(),
    phase: state.phase === "model_missing" ? "model_missing" : "empty",
  };
}

export function summarizeQueue(state: QueueState) {
  let waiting = 0;
  let processing = 0;
  let done = 0;
  let warning = 0;
  let failed = 0;
  let oomWarnings = 0;
  let saveable = 0;

  for (const item of state.items) {
    if (item.oomWarning) oomWarnings += 1;
    if ((item.status === "done" || item.status === "warning") && Boolean(item.fullCutoutPath)) {
      saveable += 1;
    }
    switch (item.status) {
      case "waiting":
        waiting += 1;
        break;
      case "processing":
        processing += 1;
        break;
      case "done":
        done += 1;
        break;
      case "warning":
        warning += 1;
        break;
      case "failed":
        failed += 1;
        break;
    }
  }

  return {
    total: state.items.length,
    waiting,
    processing,
    done,
    warning,
    failed,
    completed: done + warning,
    saveable,
    oomWarnings,
  };
}

export function hasStaleProcessedItems(
  state: QueueState,
  currentTier: ModelTierId,
  currentRefine: "none" | "light" | "standard",
  currentMaxSide: number | null
): boolean {
  return state.items.some((item) => {
    if ((item.status !== "done" && item.status !== "warning") || !item.fullCutoutPath) {
      return false;
    }
    if (item.processedTier !== undefined && item.processedTier !== currentTier) {
      return true;
    }
    if (item.processedRefine !== undefined && item.processedRefine !== currentRefine) {
      return true;
    }
    if (
      item.processedMaxSide !== undefined &&
      item.processedMaxSide !== currentMaxSide
    ) {
      return true;
    }
    return false;
  });
}
