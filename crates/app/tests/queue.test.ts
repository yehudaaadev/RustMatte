import assert from "node:assert/strict";
import test from "node:test";
import {
  applyProgressEvent,
  clearQueue,
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
} from "../src/lib/queue.ts";

test("queue state machine transitions cleanly across all lifecycle states", () => {
  let state = createInitialQueueState();
  assert.equal(state.phase, "empty");

  state = markModelMissing(state);
  assert.equal(state.phase, "model_missing");

  state = startModelDownload(state);
  assert.equal(state.phase, "downloading_model");
  state = setDownloadProgress(state, 45);
  assert.equal(state.downloadProgress, 45);

  state = failModelDownload(state, "Connection timed out");
  assert.equal(state.phase, "model_missing");
  assert.equal(state.lastError, "Connection timed out");

  state = enqueuePaths(
    state,
    ["/photos/a.jpg", "/photos/b.jpg", "/photos/a.jpg"],
    [
      {
        path: "/photos/note.txt",
        name: "note.txt",
        reason_code: "unsupported_format",
        reason: "Unsupported",
      },
    ]
  );
  assert.equal(state.items.length, 2);
  assert.equal(state.rejectedItems.length, 1);
  assert.equal(state.selectedIndex, 0);
  assert.equal(summarizeQueue(state).saveable, 0);

  state = startJob(state, true);
  assert.equal(state.phase, "loading_model");
  assert.equal(summarizeQueue(state).saveable, 0);

  state = setJobPhase(state, "processing");
  assert.equal(state.phase, "processing");

  state = applyProgressEvent(state, {
    index: 1,
    total: 2,
    file: "a.jpg",
    input_path: "/photos/a.jpg",
    status: "warning",
    timing_ms: 420,
    inference_ms: 300,
    warnings: [{ code: "damaged_input", detail: "truncated JPEG" }],
    oom_warning: true,
    error: null,
    original_preview_path: "/cache/orig_1.png",
    cutout_preview_path: "/cache/cut_1.png",
    full_cutout_path: "/cache/full_1.png",
    width: 6000,
    height: 4000,
    device: "CPU",
  });

  state = applyProgressEvent(state, {
    index: 2,
    total: 2,
    file: "b.jpg",
    input_path: "/photos/b.jpg",
    status: "failed",
    timing_ms: 15,
    inference_ms: 0,
    warnings: [],
    oom_warning: false,
    error: "Decode failed: corrupt header",
    width: 0,
    height: 0,
    device: "",
  });

  state = finishJob(state, {
    processed: 1,
    failed: 1,
    cancelled: false,
    total_ms: 435,
    output_dir: "/photos/rustmatte-output",
    display_output_dir: "/photos/rustmatte-output",
  });
  assert.equal(state.phase, "partial_failure");
  assert.equal(state.outputDir, "/photos/rustmatte-output");

  const summary = summarizeQueue(state);
  assert.equal(summary.total, 2);
  assert.equal(summary.warning, 1);
  assert.equal(summary.failed, 1);
  assert.equal(summary.saveable, 1);
  assert.equal(summary.oomWarnings, 1);

  state = selectAdjacent(state, 1);
  assert.equal(state.selectedIndex, 1);
  state = selectAdjacent(state, 1);
  assert.equal(state.selectedIndex, 1);
  state = selectAdjacent(state, -1);
  assert.equal(state.selectedIndex, 0);

  state = finishJob(state, {
    processed: 1,
    failed: 0,
    cancelled: true,
    total_ms: 200,
  });
  assert.equal(state.phase, "cancelled");

  state = clearQueue(state);
  assert.equal(state.phase, "empty");
  assert.equal(state.items.length, 0);
});
