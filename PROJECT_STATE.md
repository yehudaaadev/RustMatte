# RustMatte Project State

## Current Phase
v1.0.0 Published to Public Release — official v1.0.0 release published on GitHub with multi-OS installers (Windows NSIS setup, macOS DMG, Linux AppImage/deb, and CLI binaries); ready to start v1.1 scope (Phase 6).

## Last Session Summary
- Bumped workspace and app to `1.0.0`, verified full quality gate, and pushed commit `3bafc9d` to `origin/main`.
- Tagged `v1.0.0` and pushed to GitHub; GitHub Actions workflow `45756623551` successfully built Windows, macOS, and Linux installers and CLI archives.
- Published release `v1.0.0` publicly at https://github.com/yehudaaadev/RustMatte/releases/tag/v1.0.0.

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
- Transitioning to **v1.1 Scope** (`PRD_V2.md` §16 / `docs/TASKS.md` Phase 6):
  1. Camera RAW decoding support (`.CR2`, `.NEF`, `.ARW`, `.DNG`).
  2. Apple iPhone HEIC/HEIF decoding support.
  3. Model AI ke-3: **BEN2 (Maximum Quality)** with GPU DirectML/CoreML acceleration.

## Next Step
Owner decides priority for v1.1 kickoff:
1. RAW/HEIF image decoding codecs in `crates/core`, or
2. GPU DirectML / CoreML hardware acceleration & BEN2 model integration.

## Last Updated
2026-11-26
