# Project State: RustMatte

## Current Phase
Phase 1D-2 (Advanced Post-Processing & Edge Refinement) - Completed and verified. Awaiting owner confirmation for Phase 2 / Phase 3.

## Last Session Summary
Executed Phase 1D-2 end-to-end:
- **Fast Guided Filter Implementation (`crates/core/src/refine.rs`)**:
  - Implemented $O(N)$ separable 2D box filtering using sliding window accumulators with border clamping.
  - Applied guided filter using original image grayscale luminance as guidance image $I$ to transfer high-contrast edge details into the soft segmentation mask $p$.
- **Selective Transition Zone Feathering**:
  - Enforced solid interior protection: pixels with initial mask $\ge 0.95$ are locked to $1.0$ ($255$), and pixels with initial mask $\le 0.05$ are locked to $0.0$ ($0$).
  - Soft boundaries ($0.05 < p < 0.95$) smoothly blend with guided filter output to preserve sub-pixel hair and fur strands without softening garment interiors.
- **Color Defringing & Halo Decontamination**:
  - Implemented $5 \times 5$ neighborhood color extrapolation for semi-transparent boundary pixels ($\alpha < 235$).
  - Searches for adjacent solid foreground pixels ($\alpha \ge 235$) and decontaminates background color bleed proportional to transparency, eliminating green-screen or white background halo rims.
- **Engine & CLI Integration**:
  - Extended `Config` and `Remover` in `crates/core` with `RefineConfig`.
  - Added `--no-refine` CLI option to allow exporting raw masks for empirical A/B comparison.
- **Empirical Benchmarks & Verification**:
  - Evaluated on challenging real-world hair fixtures (`rambut1.jpg` and `rambut2.jpg`):
    - `rambut1.jpg`: Raw postprocess 76.17 ms -> Refined postprocess 101.98 ms (**Overhead: 25.81 ms**, well below 300 ms budget).
    - `rambut2.jpg`: Raw postprocess 208.48 ms -> Refined postprocess 386.20 ms (**Overhead: 177.72 ms**).
    - Output PNG size dropped from 1.04 MB to 976 KB on `rambut1`, and 2.63 MB to 1.91 MB on `rambut2`, demonstrating that edge noise was successfully eliminated and background regions flattened to pure alpha.
  - All 4 unit tests in `refine::tests` passed (`test_box_filter_1d_flat`, `test_box_filter_2d_flat`, `test_guided_filter_edge_preservation`, `test_defringing_decontamination`).
- **Documentation & Decisions Sync**:
  - Recorded D-021 in `docs/DECISIONS.md`.
  - Marked Phase 1D-2 completed in `docs/TASKS.md`.
  - Updated `CHANGELOG.md`, `README.md`, and `docs/ARCHITECTURE.md`.

## Done
- Phase 0: Workspace setup, toolchain pinning, GPL-3.0 licensing, CI workflow, skeleton crates.
- Phase 1A: Inference smoke test with BiRefNet_lite on CPU via `ort` on 6 diverse real fixtures.
- Phase 1B: Model benchmark matrix (5 models across 12 fixtures, robustness tests, ORT sweeps).
- Phase 1B-FIX: Isolated per-model memory benchmarking, contact sheet regeneration, license audit, and documentation correction.
- Phase 1C-1: Core engine pipeline, decode safety, model checksum verification, and single-image CLI.
- Phase 1C-2: Automatic model downloader, OS local caching, consent prompt, CLI models subcommand, and per-stage timing visibility.
- Phase 1D-1: Housekeeping, transitive license audit, batch folder mode, early downscaling, flat memory verification, and release packaging workflow.
- Phase 1D-2: Advanced post-processing (Fast Guided Filter, selective transition feathering, color defringing, and `--no-refine` flag).

## In Progress
- Awaiting owner confirmation before starting Phase 2 (Hardware Acceleration & DirectML/CoreML Provider Selection) or Phase 3 (Tauri 2 Desktop UI).

## Known Broken or Failing
- None. All unit tests, integration tests, cargo clippy, and cargo deny checks pass cleanly with 0 errors.

## Next Step
- Phase 2 (DirectML execution provider selection on Windows, CoreML on macOS with CPU fallback) or Phase 3 (Tauri 2 desktop GUI).

## Open Questions for the Owner
- Does the owner confirm Phase 1D-2 completion to proceed to Phase 2 (Hardware Acceleration) or Phase 3 (Desktop UI)?

## Last Updated
2026-11-24
