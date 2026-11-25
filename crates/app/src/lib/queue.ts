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

export interface QueueItem {
  path: string;
  name: string;
  status: QueueItemStatus;
  timingMs?: number;
  inferenceMs?: number;
  warnings: string[];
  oomWarning: boolean;
  error?: string;
  originalPreviewPath?: string;
  cutoutPreviewPath?: string;
  fullCutoutPath?: string;
  width?: number;
  height?: number;
  device?: string;
}

export interface RejectedItem {
  path: string;
  name: string;
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
}

export interface ProgressPayload {
  index: number;
  total: number;
  file: string;
  input_path: string;
  status: string;
  timing_ms: number;
  inference_ms: number;
  warnings: string[];
  oom_warning: boolean;
  error?: string | null;
  original_preview_path?: string | null;
  cutout_preview_path?: string | null;
  full_cutout_path?: string | null;
  width: number;
  height: number;
  device: string;
}

export interface FinishedPayload {
  processed: number;
  failed: number;
  cancelled: boolean;
  total_ms: number;
  output_dir?: string | null;
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
    items: state.items.map((item) => ({
      ...item,
      status: "waiting",
      warnings: [],
      oomWarning: false,
      error: undefined,
    })),
  };
}

export function setJobPhase(
  state: QueueState,
  phase: "loading_model" | "processing"
): QueueState {
  return { ...state, phase };
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
    const mappedStatus: QueueItemStatus =
      payload.status === "failed"
        ? "failed"
        : payload.status === "warning"
        ? "warning"
        : "done";
    return {
      ...item,
      status: mappedStatus,
      timingMs: payload.timing_ms,
      inferenceMs: payload.inference_ms,
      warnings: payload.warnings ?? [],
      oomWarning: Boolean(payload.oom_warning),
      error: payload.error ?? undefined,
      originalPreviewPath: payload.original_preview_path ?? item.originalPreviewPath,
      cutoutPreviewPath: payload.cutout_preview_path ?? item.cutoutPreviewPath,
      fullCutoutPath: payload.full_cutout_path ?? item.fullCutoutPath,
      width: payload.width || item.width,
      height: payload.height || item.height,
      device: payload.device || item.device,
    };
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

  for (const item of state.items) {
    if (item.oomWarning) oomWarnings += 1;
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
    oomWarnings,
  };
}
