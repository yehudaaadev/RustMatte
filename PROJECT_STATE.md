# RustMatte Project State

## Current Phase
v1.1.0 Ready for Release — Stage 1 (RAW `.CR2`/`.NEF`/`.ARW`/`.DNG` preview & HEIC/HEIF), Stage 2A (GPU DirectML/CoreML with CPU fallback & UI device selector), and Stage 2B (BEN2 Base FP32 `Maximum` tier) implemented, reviewed side-by-side, and verified against quality/license gates; version bumped to `1.1.0`.

## Last Session Summary
- Published v1.0.0 publicly at https://github.com/yehudaaadev/RustMatte/releases/tag/v1.0.0 (`3bafc9d`, workflow `45756623551`).
- Implemented v1.1 Stage 1: internal TIFF-IFD embedded JPEG preview extractor (`crates/core/src/raw.rs`) and pure-Rust HEIC/HEIF decoder (`heic-rs = "=0.1.5"`, `MIT OR Apache-2.0`) in `crates/core/src/decode.rs`, synced across `crates/cli`, `crates/app/src-tauri`, and Svelte UI (`crates/app/src/`).
- Verified `crates/app/` frontend build (`npm ci`, `svelte-check`: 0 errors/0 warnings, `vite build`).
- Implemented v1.1 Stage 2A (GPU acceleration):
  - Enabled target-specific `ort` `directml` (Windows) and `coreml` (macOS) in `crates/core/Cargo.toml`.
  - Refactored `crates/core/src/session.rs` (`try_directml_session`, `try_coreml_session`) and fixed `ExecutionProvider::Auto` to try CoreML on macOS as well as DirectML on Windows with automatic CPU fallback.
  - Enabled `device` selector (`cpu`, `auto`, `directml`, `coreml`) in `crates/app/src/lib/SettingsModal.svelte` and `crates/app/src/routes/+page.svelte`, and preserved saved `"auto"` in `crates/app/src-tauri/src/settings.rs`.
  - Verified release CLI DirectML execution on `tests/fixtures/rambut1.jpg` (`Fast --device auto`: DirectML `278.41 ms` inference; `Quality --device directml`: DirectML `3343.15 ms` inference).
- Implemented v1.1 Stage 2B (BEN2 Base FP32 `Maximum` tier):
  - Added `ModelTier::Maximum` (`ben2_fp32.onnx`, `422,457,868` bytes, SHA-256 `9a8145c1c7ab73fcb265cb5feaab16e0a8c1986265e1a301cf0edd95103b99c9`) in `crates/core/src/model.rs`, `manifest.rs`, `pipeline.rs`, `lib.rs`, `crates/cli`, `crates/app/src-tauri`, and Svelte UI (`SetupWizard.svelte`, `SettingsModal.svelte`, `+page.svelte`, `i18n.ts`).
  - Documented BEN2 Base MIT / Apache-2.0 license and DIS5K + 22K proprietary dataset provenance in `THIRD_PARTY_LICENSES.md` and `MODELS.md`.
  - Downloaded & verified `ben2_fp32.onnx` via `rustmatte models download --tier maximum --yes` and benchmarked on `tests/fixtures/rambut1.jpg` (`CPU`: `16,241.64 ms` inference / `20,412.94 ms` total; `DirectML`: `24,068.57 ms` inference / `72,794.10 ms` total).

## Done
- **v1.0.0 Officially Released**:
  - Full end-to-end background removal engine with offline ONNX Runtime CPU inference.
  - Two verified tiers: Fast (`U-2-Netp`, 4.36 MB) and Balanced (`BiRefNet Lite 512`, 183 MB).
  - Banded RGB Color Guided Filter and 6-pass wavefront defringing bounding 24 MP peak memory to ~2.19 GB / ~499 MB.
  - Resumable HTTP Range downloader with SHA-256 verification and atomic rename.
  - Mandatory 5-step guided setup wizard with RAM detection and questionnaire recommendation.
  - Dedicated Settings modal with model cache management and per-photo model retry comparison.
  - Visual design tokens strictly tuned for WCAG AA (>= 4.5:1) contrast in Light and Dark themes.
  - Bilingual English and Indonesian interface with 100% dictionary synchronization.
  - Multi-OS GitHub Release published (`RustMatte_1.0.0_x64-setup.exe`, `.dmg`, `.AppImage`, `.deb`, CLI archives).

## In Progress
- **v1.1.0 Release Prep**:
  1. Camera RAW & Apple HEIC/HEIF decoding — implemented, quality-gated, and verified by owner.
  2. GPU DirectML (Windows) / CoreML (macOS) with CPU fallback — Windows DirectML verified; CoreML macOS pending CI run.
  3. Model AI ke-3: **BEN2 Base FP32 (`Maximum`)** — implemented, quality-gated, and side-by-side review vs Balanced approved by owner.

## Next Step
1. Build & verify Windows NSIS installer (`v1.1.0`), then commit & tag `v1.1.0` to trigger multi-OS CI/Release builds (including macOS CoreML & Linux).

## Last Updated
2026-11-27
