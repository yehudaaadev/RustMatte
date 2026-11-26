# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-11-26

### Added & Improved
- **Mandatory 5-Step Guided Setup Wizard (`D-036`)**: Automatic local RAM detection via `sysinfo` (`LOW_RAM_THRESHOLD_MB = 3072`, `MID_RAM_THRESHOLD_MB = 5120`), 2-question photo-mix and workflow questionnaire, intelligent model recommendation, measured Fast/Balanced cards, resumable downloader with HTTP `Range` and SHA-256 verification, and offline `.onnx` local import. Wizard answers and RAM are strictly in-memory and never persisted.
- **Dedicated Settings Screen & Model Management (`D-037`)**: Full model cache inspector (verify SHA-256, delete cached weights with freed disk space confirmation and SessionCache eviction, re-run wizard, download/import), allow-listed GitHub Issues feedback launcher, network lock, output defaults, and copyable diagnostic info without file paths.
- **Per-Photo Model Retry & Side-by-Side Comparison (`D-037`)**: Retry individual photos with another tier while preserving existing batch results, supporting instant toggle or side-by-side inspection between Fast and Balanced cutouts.
- **Visual Design System & WCAG AA Contrast Polish (`D-038`)**: Semantic tokens in `crates/app/src/app.css`, `<meta name="color-scheme" content="light dark" />`, WCAG AA verified contrast pairings (Light `--warning` `#945F0F` achieving 5.02:1 / 5.38:1, Dark `--accent-text` `#15181D` achieving 6.29:1), clean inline SVG empty dropzone, active-model indicator card, collapsed `<details class="advanced-panel">` on main screen, and non-color status glyphs.
- **Banded RGB Color Guided Filter & Wavefront Defringing (`D-032`, `D-034`)**: 3-channel guided filter in 128-row bands bounding 24 MP peak memory to `2,191.39 MB` Balanced / `498.96 MB` Fast, low-confidence haze suppression (`q <= 0.06`), and 6-pass wavefront foreground color decontamination eliminating edge halos.
- **Resumable HTTP Range Downloader (`D-035`)**: Atomic `.part` downloads with `.part.meta.json`, 10% disk safety margin check, cancel preservation, and dual mirror support (`github-release` with `huggingface.co` fallback).
- **CLI Model Management (`D-035`)**: Added `rustmatte models list`, `download`, `delete`, and `verify` subcommands.
- **Bilingual English and Indonesian (`D-008`, `D-029`)**: 100% synchronized dictionaries with code-based parity tests.
- Counted only saveable completed cutouts (`fullCutoutPath` present) in the `Save all (N)` button label and disabled state.
- Added compile-time `#[cfg(all(not(debug_assertions), dev, not(test)))]` guard preventing plain `cargo build --release -p rustmatte-app` from producing a binary pointed at `devUrl` (`http://localhost:1420`) instead of bundled frontend assets (`D-028`, `P-033`), plus `notify_frontend_ready` (`$APPCACHE/startup.log`) and a multi-OS `RUSTMATTE_BUILD_MODE=custom-protocol` check in `.github/workflows/release.yml`.
- Eliminated ~1.17 GB peak working set spike at 24 MP during edge refinement by processing the Guided Filter in 128-row horizontal bands and reading image pixels directly without full-frame `to_rgb8()`/`to_rgba8()` copies (Quality 24 MP: `3,352.68 MB` -> `2,185.15 MB`; Fast 24 MP: `1,660.08 MB` -> `492.68 MB`; postprocessing ~2x faster).
- Added explicit `--refine <none|light|standard>` CLI option (default `standard` per `D-021`, keeping `--no-refine` alias).
- Changed desktop bundle identifier from `com.rustmatte.app` to `io.github.yehudaaadev.rustmatte` (resolves macOS `.app` suffix collision warning).
- Resolved `svelte-check` accessibility warning on comparison slider viewport (`0` errors, `0` warnings) with `use:bindViewportEvents` + `role="region"` and keyboard zoom/pan support.
- Removed leftover unrelated `tests/fixtures/SHA256SUMS` file.

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
