# Project State: RustMatte

## Current Phase
Phase 2 (Batch Processing, Execution Providers, and Hardware Acceleration) - Completed and verified. Awaiting owner confirmation for Phase 3 (Tauri 2 Desktop UI).

## Last Session Summary
Executed Phase 2 end-to-end:
- **Cargo Feature Configuration**:
  - Added `directml = ["ort/directml"]` and `coreml = ["ort/coreml"]` feature flags to `crates/core` and `crates/cli`.
  - Upstream `ort-sys` links and bundles `DirectML.dll` on Windows targets without adding any third-party crate dependencies.
- **Execution Provider Selection & Safe Fallback (`crates/core/src/session.rs` & `lib.rs`)**:
  - Implemented `ExecutionProvider` enum (`Cpu`, `DirectML`, `CoreML`, `Auto`) and `ActiveExecutionProvider`.
  - Maintained pure CPU as default across all platforms per D-017.
  - Implemented multi-stage graceful fallback:
    1. Session init stage: attempts DirectML/CoreML initialization; falls back to CPU if hardware or drivers are unavailable.
    2. Runtime inference stage: catches GPU hardware errors (such as DirectX `8007000E` out-of-memory during fused shader compilation), immediately drops the failed hardware session to release all VRAM/RAM buffers, creates a clean CPU session, and transparently completes inference without crashing the user process.
- **CLI Device Flag & Logging (`crates/cli/src/main.rs`)**:
  - Added `--device <cpu|directml|coreml|auto>` (default: `cpu`).
  - Added active device reporting in single-file verbose output and batch summary (`Device: DirectML`).
- **Empirical Benchmarks & Verification (Windows 10, i5-10400, Intel UHD Graphics)**:
  - Fast Tier (`u2netp` on `tests/fixtures/baju1.jpg`):
    - DirectML GPU inference: **328.61 ms** (warm shader cache, 871 ms model load).
    - CPU inference: **464.33 ms** (207 ms model load).
    - GPU acceleration achieved **29.2% faster inference** on consumer hardware.
  - Quality Tier (`birefnet_lite_512` on `tests/fixtures/baju1.jpg`):
    - DirectML graph fusion exceeded GPU VRAM allocation (`8007000E`), triggering the automatic runtime fallback to CPU.
    - Successfully completed on CPU in 5.51s without user interruption or fatal error (exit code 0).
  - Batch Folder Execution (`tests/fixtures`, 15 images with DirectML Fast tier):
    - 14 valid/truncated images processed successfully on GPU DirectML.
    - 1 corrupt zero-byte file gracefully reported without aborting the batch.
    - Total batch time: 20.79s, average time: 1481.9 ms / image.
- **Quality Gates & Tests**:
  - All 28 tests passed (`cargo test -j 2 --workspace` and `cargo test -j 2 --workspace --features directml`).
  - `cargo clippy --workspace --all-targets -- -D warnings`: 0 warnings.
  - `cargo fmt --all -- --check`: 100% compliant.
  - `cargo deny check licenses` and `cargo deny check advisories`: passed with 0 issues.
- **Documentation & Decisions Sync**:
  - Recorded D-022 in `docs/DECISIONS.md`.
  - Marked Phase 2 tasks complete in `docs/TASKS.md`.
  - Updated `CHANGELOG.md` and `README.md`.

## Done
- Phase 0: Workspace setup, toolchain pinning, GPL-3.0 licensing, CI workflow, skeleton crates.
- Phase 1A: Inference smoke test with BiRefNet_lite on CPU via `ort` on 6 diverse real fixtures.
- Phase 1B: Model benchmark matrix (5 models across 12 fixtures, robustness tests, ORT sweeps).
- Phase 1B-FIX: Isolated per-model memory benchmarking, contact sheet regeneration, license audit, and documentation correction.
- Phase 1C-1: Core engine pipeline, decode safety, model checksum verification, and single-image CLI.
- Phase 1C-2: Automatic model downloader, OS local caching, consent prompt, CLI models subcommand, and per-stage timing visibility.
- Phase 1D-1: Housekeeping, transitive license audit, batch folder mode, early downscaling, flat memory verification, and release packaging workflow.
- Phase 1D-2: Advanced post-processing (Fast Guided Filter, selective transition feathering, color defringing, and `--no-refine` flag).
- Phase 2: Batch processing, formats, and performance (DirectML/CoreML execution providers, multi-stage CPU fallback, `--device` CLI flag).

## In Progress
- Awaiting owner confirmation before starting Phase 3 (Tauri 2 Desktop UI).

## Known Broken or Failing
- None. All unit tests, integration tests, cargo clippy, and cargo deny checks pass cleanly with 0 errors.

## Next Step
- Phase 3 (Tauri 2 Desktop UI with Svelte/TypeScript frontend, before/after comparison slider, drag-and-drop queue, and Windows installer).

## Open Questions for the Owner
- Does the owner approve Phase 2 completion to proceed to Phase 3 (Tauri 2 Desktop UI)?

## Last Updated
2026-11-24
