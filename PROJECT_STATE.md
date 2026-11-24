# Project State: RustMatte

## Current Phase
Phase 3 (Desktop UI with Tauri 2 & Svelte 5) - Completed and verified. Awaiting owner confirmation for Phase 4 (Packaging and Multi-OS Distribution).

## Last Session Summary
Executed Phase 3 end-to-end:
- **Tauri 2 Desktop Application Scaffolding (`crates/app`)**:
  - Pinned `tauri = "=2.12.7"` and `tauri-build = "=2.7.6"`.
  - Added `crates/app/src-tauri` as a first-class member of the Cargo workspace.
  - Configured window properties: 1080x720 initial dimension, 800x600 minimum, drag-and-drop support, CSP, and permissions.
  - Implemented typed Tauri 2 IPC commands in `crates/app/src-tauri/src/lib.rs`:
    - `get_model_cache_status`: returns availability of Quality and Fast tiers in OS cache.
    - `download_model_tier`: on-demand model weight downloading directly from UI.
    - `process_single_image`: processes raw image bytes using `rustmatte_core` pipeline (decoding, preprocessing, ORT inference, refinement) and returns Base64 PNG cutout and per-stage timings.
    - `save_cutout_png`: exports transparent cutout to local filesystem.
- **Frontend Architecture (`crates/app/src/routes/+page.svelte`)**:
  - Modern Svelte 5 with runes (`$state`, `$derived`).
  - Drag-and-drop zone supporting single files and batch queues.
  - Interactive Before/After split comparison slider with draggable dividing line and checkerboard transparency.
  - Background color preview picker (Transparent checkerboard, Solid White, Solid Black, Custom HEX color).
  - Configurable inference settings:
    - Tier: Quality (`BiRefNet`) vs Fast (`U-2-Netp`).
    - Device: CPU vs DirectML GPU.
    - Early Downscaling: `--max-side` slider (512px - 4096px, or original).
    - Edge Refinement: Guided Filter and defringing toggle.
  - Full bilingual support (English / Bahasa Indonesia) with instant reactive switching.
  - Dark / Light theme toggle with persistence.
  - Timing metrics panel displaying total processing duration and individual stage latencies.
- **Compilation & Quality Gate**:
  - SvelteKit static export compiled with 0 errors.
  - Release binary built: `target/release/rustmatte-app.exe` (34,575,360 bytes / 34.5 MB).
  - All 28 workspace unit and integration tests passed.
  - `cargo fmt --all -- --check` passed cleanly.
  - `cargo deny check licenses` and `cargo deny check advisories` verified with 0 issues.

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
- Phase 3: Desktop UI with Tauri 2 (Svelte 5 frontend, before/after slider, background preview, bilingual EN/ID, dark mode, `rustmatte-app.exe`).

## In Progress
- Completed Phase 3. Awaiting owner direction for Phase 4 (Packaging and Multi-OS Distribution).

## Known Broken or Failing
- None. All unit tests, integration tests, cargo clippy, and cargo deny checks pass cleanly with 0 errors.

## Next Step
- Phase 4: Release Packaging and Multi-OS Distribution (MSI installer generation, GitHub Actions release matrix for Windows, macOS, Linux).

## Open Questions for the Owner
- Does the owner want to test `rustmatte-app.exe` now, or proceed to Phase 4?

## Last Updated
2026-11-24
