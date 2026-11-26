# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Improved & Fixed
- Implemented Phase 5D visual polish and design system (`D-038`, `PRD_V2.md` §8 & §7.10): central CSS design tokens in `crates/app/src/app.css` (semantic spacing, radii, typography, transitions, `@media (prefers-reduced-motion)` and `@media (forced-colors)`), WCAG AA contrast tuning for Light/Dark themes (`Light --warning` `#945F0F` and `Dark --accent-text` `#15181D`), inline SVG + single-sentence empty dropzone, active-model card with Settings shortcut, collapsed `<details class="advanced-panel">` on the main screen, and non-color status glyphs.
- Implemented Phase 5C Settings modal, model management, allow-listed feedback link, and per-photo model retry (`D-037`): dedicated `SettingsModal.svelte` (language, theme, cached model verify/delete with freed-space confirmation/download/re-run wizard, network lock, output defaults, collapsed Advanced, copy diagnostics), `open_feedback_url` (`validate_feedback_url` allow-listed to repository GitHub Issues via OS handler without opener plugins), and `retry_single_with_model` + `resultsByTier` side-by-side/toggle comparison on the result viewer.
- Implemented Phase 5B mandatory 5-step bilingual setup wizard (`D-036`): `wizard.ts` state machine, `SetupWizard.svelte` UI (`language`, `briefing + 2 questions`, `recommendation` with `See all models` and measured `Fast`/`Balanced` cards, `download/resume/cancel/delete-partial/offline import`, `ready`), `wizard_completed`/`wizard_version` settings persistence (without saving wizard answers or RAM), and Tauri commands (`get_manifest_models`, `recommend_setup_model`, `cancel_model_download`, `delete_partial_model_download`).
- Implemented Phase 5A core & CLI infrastructure (`D-035`): typed model manifest + validation (`crates/core/src/manifest.rs`), pure recommendation engine (`crates/core/src/recommend.rs`), local available RAM & `size + 10%` disk probing via `sysinfo` (`crates/core/src/sysinfo.rs`), HTTP `Range` resumable model downloader with `.part` + metadata + retry backoff + cancel preservation (`crates/core/src/download.rs`), and `rustmatte models list|download|delete|verify` CLI subcommands (`crates/cli/src/main.rs`).
- Adopted `PRD_V2.md` (`Status: accepted`, `D-033`) locking `v1.0` profiles to `Fast` (`u2netp`) and `Balanced` (`birefnet_lite_512`) while deferring `BEN2` to `v1.1`, and completed Phase 5.0 close-out (`D-034`: aligned `refine_mask_guided` foreground early-exit to `m < 243`, added multi-band full-frame parity tests, audited `q <= 0.06` hair/whisker haze retention, and re-verified 24 MP peak RAM at `2,191.39 MB` Balanced / `498.96 MB` Fast).
- Upgraded edge refinement (`crates/core/src/refine.rs`) to a banded 3-channel RGB Color Guided Filter with low-confidence haze suppression and 6-pass wavefront foreground color decontamination, eliminating color halos on isoluminant and high-contrast backgrounds (`D-032`).
- Added processed model/refinement tracking (`JobProgressEvent`, `QueueItem`, `stats-bar`), stale-result reset on `startJob`, and localized EN/ID warnings when tier/refine/resolution settings change after processing (`D-032`).
- Published pinned ONNX weights (`birefnet_lite_512.onnx` and `u2netp.onnx`) to GitHub Release `models-v1` (verified public `200 OK` and SHA-256) with `huggingface.co` retained as automatic fallback (`D-031`).
- Migrated saved `"auto"` device settings to `"cpu"` on load and replaced `#device-select` with a read-only localized CPU processing status line (`D-030`, `D-031`).
- Moved the release `custom-protocol` `compile_error!` guard from `lib.rs` to `crates/app/src-tauri/src/main.rs` so `cargo test --release --workspace` succeeds while `cargo build --release` still blocks `devUrl` release binaries (`D-031`).
- Fixed `.github/workflows/release.yml` macOS binary lookup (`*/Contents/MacOS/*` for `RustMatte.app/Contents/MacOS/RustMatte`) and updated Linux release builds to `ubuntu-24.04` (`glibc 2.39+` required by `ort` prebuilt static archives; `D-031`).
- Aligned desktop default hardware execution provider with `D-017`/`D-022` (`cpu` default in `settings.rs` and UI options, keeping `auto`/`directml`/`coreml` opt-in; `D-030`).
- Defaulted the comparison preview slider to cutout-only (`0%`) when a photo finishes or is selected, added a `"Compare with original"` toolbar toggle (`C` keyboard shortcut) with per-session split memory, and reset new photos back to cutout-only (`D-029`).
- Stripped Windows extended-length path prefixes (`\\?\` drive paths and `\\?\UNC\`) for UI display only while preserving canonical paths internally (`P-034`).
- Moved all active status banners, decode warnings, file rejection reasons, dialog titles/filters, and About network descriptions to code-based `en` and natural `id` dictionaries (`D-029`).
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
