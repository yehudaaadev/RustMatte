# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Eliminated ~1.17 GB peak working set spike at 24 MP during edge refinement by processing the Guided Filter in 128-row horizontal bands and reading image pixels directly without full-frame `to_rgb8()`/`to_rgba8()` copies (Quality 24 MP: `3,352.68 MB` -> `2,185.15 MB`; Fast 24 MP: `1,660.08 MB` -> `492.68 MB`; postprocessing ~2x faster).
- Added explicit `--refine <none|light|standard>` CLI option (default `standard` per `D-021`, keeping `--no-refine` alias).
- Changed desktop bundle identifier from `com.rustmatte.app` to `io.github.yehudaaadev.rustmatte` (resolves macOS `.app` suffix collision warning).
- Resolved `svelte-check` accessibility warning on comparison slider viewport (`0` errors, `0` warnings) with `use:bindViewportEvents` + `role="region"` and keyboard zoom/pan support.

## [0.1.0] - 2026-11-25 (Pre-Release)

### Added
- **Core Background Removal Engine (`crates/core`)**:
  - Offline ONNX Runtime (`ort = "=2.0.0-rc.15"`) CPU inference pipeline with optional `directml` (Windows) and `coreml` (macOS) execution providers and two-stage graceful CPU fallback.
  - Two-tier model architecture (`D-018`): Quality tier (`BiRefNet_lite_512`, 183 MB, MIT) and Fast tier (`U-2-Netp`, 4.36 MB, Apache-2.0).
  - Post-processing edge refinement (`crates/core::refine`): $O(N)$ separable Fast Guided Filter ($r = 4, \epsilon = 10^{-4}$), selective transition zone feathering, and $5 \times 5$ boundary color defringing.
  - Safe image decoding (`MAX_IMAGE_DIMENSION = 16384`, `MAX_ALLOCATION_BYTES = 512 MB`), EXIF orientation normalization, and truncated-input warning detection (`Warning::DamagedInput`).
  - Consented one-time model downloader (`crates/core::download`) using pure-Rustls `ureq`, `.part` atomic rename, strict SHA-256 verification, and hierarchical model lookup (`--model-path` > `RUSTMATTE_MODEL_DIR` > OS cache > `./models/`).
- **Command-Line Interface (`crates/cli`)**:
  - Single-image and folder batch processing (`--out`, `--recursive`, `--skip-existing`, `--overwrite`, `--max-side`, `--no-refine`, `--device`, `--yes`, `--no-download`, `--verbose`) with deterministic collision naming (`<stem>_<ext>.png`) and tri-state exit codes (`0`, `1`, `2`).
  - `rustmatte models list` and `rustmatte models download` subcommands.
- **Desktop Application (`crates/app`, Tauri 2 + Svelte 5)**:
  - Single-screen UI with photo/folder drop zone, windowed thumbnail queue, zoom/pan Before/After comparison slider on checkerboard or solid preview, transparent or solid-color PNG export (`Save` and `Save all`), progress bar, and `Cancel`.
  - Scoped `assetProtocol` preview cache (`<= 1600 px` in `$APPCACHE/previews/`, cleaned on exit) — zero full-resolution image data over IPC.
  - First-run model dialog with explicit size/host/license disclosure, SHA-256 verified local `.onnx` import (`import_model_file`), `Retry`, and `"Never use the network"` offline lock.
  - About & Licenses modal reading shipped `MODELS.md` and `THIRD_PARTY_LICENSES.md`, plus `"Copy diagnostic info"` (no file paths).
  - Bilingual English and Indonesian dictionaries, `system`/`light`/`dark` themes, reduced-motion support, and keyboard shortcuts (`Ctrl+O`, `Ctrl+Enter`, `Ctrl+S`, `Ctrl+Shift+S`, `ArrowUp/Down`, `Esc`).
  - Strict CSP, minimal `dialog:allow-open`/`dialog:allow-save` capabilities, and atomic JSON settings persistence (`docs/SECURITY_MODEL.md`).
- **Packaging, CI, and Community Infrastructure**:
  - Windows NSIS installer (`RustMatte_0.1.0_x64-setup.exe`, `9.69 MiB`) with WebView2 `embedBootstrapper`.
  - Multi-OS release workflow (`.github/workflows/release.yml`) for Windows NSIS, macOS `aarch64` `.dmg`, Linux `.AppImage`/`.deb`, CLI archives, and consolidated `SHA256SUMS.txt`.
  - CI checks (`fmt`, `clippy -D warnings`, workspace tests, `cargo-deny`, npm license check, `node --test`, `svelte-check`, `vite build`, version parity), Dependabot, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue/PR templates, `docs/ROADMAP.md`, and `docs/RELEASING.md`.
