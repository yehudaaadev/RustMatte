# Project State: RustMatte

## Current Phase
Pre-Release Fixes Complete — all uncommitted Phase 3A, 3B, 3C, 4, and pre-release fix work committed logically on local `main` (not pushed, not tagged). Awaiting owner review and confirmation.

## Last Session Summary
Executed Pre-Release Fixes:
- **24 MP Memory & Speed Regression Fix (`crates/core/src/refine.rs`, `D-027`, `P-032`)**:
  - Identified root cause: `guided_filter` allocated 13 concurrent full-frame `Vec<f32>` buffers (`96 MB` each at 24 MP = `1,248 MB`) plus `orig.to_rgb8()` (`72 MB`) and `orig.to_rgba8()` (`96 MB`).
  - Replaced full-frame allocations with 128-row banded `guided_filter` (`4 * radius` halos, reused `BandBuffers`) and direct `DynamicImage` slice reads.
  - Measured results (release CLI, fresh process per run, `large_24mp.jpg`):
    - Quality `--refine standard`: `3,352.68 MB` -> **`2,185.15 MB`** (`<= 2.5 GB` target), postprocess `3,590.40 ms` -> `1,748.98 ms`.
    - Fast `--refine standard`: `1,660.08 MB` -> **`492.68 MB`** (`<= 1.0 GB` target), postprocess `3,603.99 ms` -> `1,579.88 ms`.
  - Added `--refine <none|light|standard>` CLI flag (default `standard` per `D-021`).
- **Identifier, Accessibility, Docs & Audits**:
  - Updated Tauri identifier `com.rustmatte.app` -> `io.github.yehudaaadev.rustmatte` across `tauri.conf.json`, `README.md`, `docs/ARCHITECTURE.md` (zero old refs remaining; `dirs::data_local_dir()` model cache unaffected).
  - Fixed `.slider-viewport` accessibility warning (`svelte-check`: `0` errors, `0` warnings) using `use:bindViewportEvents` + `role="region"` and keyboard zoom/pan.
  - Audited `README.md` comparison table against upstream sources (`rembg` MIT, `@imgly/background-removal` AGPL-3.0, `remove.bg` proprietary credit SaaS; checked `2026-11-25`).
  - Checked `models-v1` GitHub Release URLs: both returned `HTTP/1.1 404 Not Found`.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes committed cleanly on local `main`.

## In Progress
- Awaiting owner action on `tests/fixtures/PROVENANCE.md` / `tests/fixtures/SHA256SUMS`, `models-v1` asset upload, pushing to `origin/main`, and `v0.1.0-rc2` draft release review.

## Known Broken or Failing
- `https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/*.onnx` returns `404` (HuggingFace fallback works).
- `tests/fixtures/SHA256SUMS` contains leftover `opencode-hop-*` hashes from commit `8a53caf`.
- Multi-OS `release.yml` cloud build (`v0.1.0-rc2`), macOS/Linux runtime tests, and clean second-PC installer verification remain unverified until pushed/tested.

## Next Step
- Owner reviews commits and `tests/fixtures/PROVENANCE.md`, then confirms pushing `main` and tagging `v0.1.0-rc2`.

## Open Questions for the Owner
- What would you like to do with `tests/fixtures/SHA256SUMS` and the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md` before pushing?
- Ready to push local commits to `origin/main`?

## Last Updated
2026-11-25
