# RustMatte Architecture Map

Status: Implemented through Phase 1D-1 (Core Pipeline, Model Downloader, Batch Processing, and Release Workflow). Derived strictly from `docs/PRD.md` Sections 5, 6, 14, and Owner Decisions.

## Workspace Layout
```text
RustMatte/
├── Cargo.toml                 (workspace root, planned)
├── rust-toolchain.toml        (pinned toolchain, planned)
├── crates/
│   ├── core/                  (engine library, planned)
│   ├── cli/                   (command-line interface binary, planned)
│   └── app/                   (Tauri 2 desktop application, planned)
├── web/                       (WASM/PWA build for v2, planned)
├── tests/                     (integration tests and fixtures, planned)
├── docs/                      (architecture and task documentation)
├── .agents/                   (rules, skills, and workflows)
├── THIRD_PARTY_LICENSES.md    (license registry)
└── README.md                  (project overview, planned)
```

## Crate Responsibilities and Implemented Modules

### `crates/core` (Implemented through Phase 5A)
Core background removal engine library. Contains no UI code and performs zero terminal printing.
- `src/error.rs`: Strongly typed `CoreError` using `thiserror` (handling decode errors, dimension bounds, missing models, checksum mismatches, insufficient disk space, cancellation, ORT session errors, network/download errors, and I/O).
- `src/manifest.rs`: Model manifest schema and validation (`balanced` / `birefnet_lite_512` and `fast` / `u2netp` entries, tensor input specs, measured hardware data, dictionary profile tag codes, and minimum available RAM).
- `src/model.rs`: Defines `ModelTier` (`Quality`, `Fast`), `ModelSpec`, OS cache directory resolution via `dirs`, hierarchical lookup (`--model-path` > `RUSTMATTE_MODEL_DIR` > OS cache dir > `./models/`), and cache deletion (`delete_cached_model`).
- `src/sysinfo.rs`: Local available RAM and filesystem free-space probing via `sysinfo` (`system` and `disk` features only, no network access) with injectable test probes and `size + 10%` disk margin checks.
- `src/recommend.rs`: Pure, side-effect-free recommendation engine (`PRD_V2.md` §7.4 table + named `LOW_RAM_THRESHOLD_MB = 3072` / `MID_RAM_THRESHOLD_MB = 5120` thresholds).
- `src/download.rs`: Safe streaming HTTPS model downloader with localhost test overrides, disk-space preflight, HTTP `Range` resume (`206 Partial Content` / `200 OK` fallback) via `.part` + `.part.meta.json`, retry backoff, cancel preserving partial state, full-file SHA-256 verification, and atomic rename.
- `src/session.rs`: Handles ONNX Runtime session creation and hardware execution providers (`ExecutionProvider::Cpu`, `DirectML`, `CoreML`, `Auto`). Enforces D-015 memory allocator settings (`with_memory_pattern(false)` and `session.use_env_allocators = 1`), calculates default thread heuristic (half logical CPUs clamped to 2..=8), and implements multi-stage graceful CPU fallback during session creation and inference execution.
- `src/decode.rs`: Safe image decoding with dimension bounds (16384px limit, 512MB max allocation), EXIF orientation correction, and truncation/damage detection (`Warning::DamagedInput`).
- `src/pipeline.rs`: Preprocessing (RGB triangle resize and NCHW tensor normalization), postprocessing (raw logits sigmoid/min-max to 8-bit mask and bilinear upscaling), `refine_mask` hook for Phase 1D, and `apply_mask` alpha blending.
- `src/lib.rs`: Minimal public API (`Config`, `Output`, `StageTimings`, `Remover::new`, `Remover::process`, `download_model`, `recommend_model`, `manifest_entries`, `detect_available_memory_mb`, `check_disk_space`). Manages global ORT environment safely across instances via `std::sync::OnceLock`.

### `crates/cli` (Implemented through Phase 5A)
Lightweight command-line interface binary using `clap` (derive). Thin wrapper around `crates/core`.
- `src/main.rs`: Argument parsing (`rustmatte <INPUT> [-o <OUTPUT>] [--tier quality|fast] [--model-path <FILE>] [--threads N] [--overwrite] [--skip-existing] [--max-side N] [--recursive] [--yes] [--no-download] [--verbose]`), model management subcommands (`rustmatte models [list|download|delete|verify]`), interactive download disclosure/consent (`[y/N]`), atomic file creation via temp files, warning output to stderr, and `--verbose` per-stage execution timing breakdown. Returns exit codes: `0` (all ok), `1` (total failure / bad args), `2` (partial failure).
- `src/batch.rs`: Batch image discovery, recursive directory traversal with symlink traversal boundaries, batch limit warnings (>1,000 files), and deterministic collision resolution (conflicting files with identical stems in the same folder resolve to `<stem>_<ext>.png`).

### `crates/app` (Phase 3A, 3B, and 5B Implemented)
Desktop application using Tauri 2 (`=2.12.7`) and Svelte 5 (`5.59.1`) + TypeScript (`6.0.3`). Thin wrapper around `rustmatte-core`.
- `src-tauri/src/lib.rs`: Tauri command surface (`get_models_status`, `get_manifest_models`, `recommend_setup_model`, `expand_inputs`, `download_model`, `cancel_model_download`, `delete_partial_model_download`, `import_model_file`, `process_images`, `cancel_job`, `save_cutout_png`, `save_all_cutouts`, `open_output_folder`, `get_settings`, `set_settings`, `get_about_info`, `get_diagnostic_info`) and lifecycle cleanup (`RunEvent::Exit` cancels active jobs/downloads and wipes preview cache).
- `src-tauri/src/state.rs`: `SessionCache` keeping at most one loaded `Remover` per `ModelTier` across jobs (reloaded only on option changes), `RefineLevel` (`none`/`light`/`standard`), `JobCoordinator`, and `DownloadCoordinator` for single-active-job/download tracking and atomic cancellation.
- `src-tauri/src/preview.rs`: Bounded preview cache (`<= 1600 px` longest side, `128 MB` / `120` files cap, startup and exit cleanup) plus solid-background RGB compositing for PNG export. Full-resolution image data never crosses Tauri IPC; previews are written to `$APPCACHE/previews/` and loaded in the webview via `convertFileSrc` (`assetProtocol` scoped strictly to `$APPCACHE/previews/**`).
- `src-tauri/src/validation.rs`: Canonical input/output path validation, folder expansion, extension filtering, deterministic batch collision resolution (`<stem>_<ext>.png`), and overwrite protection against source files.
- `src-tauri/src/settings.rs`: Atomic JSON persistence (`settings.json` in OS `app_config_dir`) covering language, system/light/dark theme, tier, device, refine level, background color mode, output folder, `never_use_network`, `wizard_completed`, and `wizard_version` (wizard answers and detected RAM are never saved).
- `src/lib/{i18n.ts,queue.ts,errors.ts,wizard.ts,SetupWizard.svelte}` & `src/routes/+page.svelte`: Typed EN/ID dictionaries, pure queue and 5-step mandatory setup wizard reducers, friendly error mapper with technical details expander, windowed thumbnail queue, zoom/pan accessible comparison slider, 5-step guided setup wizard with measured model cards/offline import/resume controls, About & Licenses modal (from shipped `MODELS.md` / `THIRD_PARTY_LICENSES.md`), and keyboard shortcuts.
- Security posture (strict CSP, minimal `dialog:allow-open`/`dialog:allow-save` capabilities, no `shell` or `fs` plugins, release devtools disabled) is documented in `docs/SECURITY_MODEL.md`.

### `web/` (Planned for v2)
WebAssembly and PWA implementation for browser-based offline processing.

## Dependency Rule
- `crates/core` must contain NO UI code and NO printing to stdout/stderr. It returns strongly typed `Result<T, E>`.
- `crates/cli` and `crates/app` are thin consumers of `crates/core`.
- `crates/core` exposes a small, stable public API (e.g., `Remover::new(config)`, `remover.process(image)`, `remover.process_batch(paths, options, callback)`).

## Processing Pipeline
1. Decode image, apply EXIF orientation metadata (v1.0 supported: JPEG, PNG, WebP, BMP; unsupported formats return clear error, never crash).
2. Optional fast mode: downscale very large images early to save decode time and RAM (model input is fixed, so inference time is unchanged).
3. Resize to model input dimensions, preserving aspect ratio.
4. Normalize image pixel data to tensor shape and format expected by model.
5. Run ONNX inference using CPU as default (optional hardware execution providers DirectML on Windows, CoreML on macOS, with automatic CPU fallback).
6. Extract probability mask and upscale back to target dimensions.
7. Refine mask: Fast Guided Filter edge alignment ($r = 4, \epsilon = 10^{-4}$) using original grayscale luminance as guidance, selective transition zone feathering (locking solid interior foreground $\ge 0.95$ and definite background $\le 0.05$), and edge color cleanup (5x5 boundary defringing).
8. Apply mask as alpha channel with defringing, export transparent 32-bit PNG at specified resolution.

## Concurrency & Memory Guardrails
- Strictly sequential batch processing (D-020): Single shared ORT session (`Remover`) initialized once for the run.
- Zero parallel image queues: Exactly one full-resolution image decoded in RAM at a time. Each buffer is processed, written, and immediately dropped before decoding the next image.
- Memory profile remains flat and constant across arbitrarily large batches (empirically verified: ~313 MB flat on Fast tier, ~1,915 MB flat on Quality tier across 100 images).
- Early downscaling (`--max-side <N>`): Resizes input image immediately after decode if dimensions exceed N, bounding peak memory and cutting postprocessing time by up to 98% on 24MP images.
- Immediate deallocation: raw image buffers dropped immediately after saving to disk.

## Model, Settings, and Cache Storage
- Models are never committed to git and never bundled into the v1.0 installer.
- Settings (`settings.json`, atomic write) live in the OS app-config directory (`io.github.yehudaaadev.rustmatte`):
  - Windows: `%APPDATA%\io.github.yehudaaadev.rustmatte\settings.json`
  - macOS: `~/Library/Application Support/io.github.yehudaaadev.rustmatte/settings.json`
  - Linux: `~/.config/io.github.yehudaaadev.rustmatte/settings.json`
- Preview cache (`<= 1600 px` PNGs, wiped on startup and exit) lives in OS app-cache (`io.github.yehudaaadev.rustmatte/previews/`).
- Downloaded AI models (`.onnx`, SHA-256 verified) live in OS local data directory:
  - Windows: `%LOCALAPPDATA%\RustMatte\models\`
  - macOS: `~/Library/Application Support/RustMatte/models/`
  - Linux: `~/.local/share/RustMatte/models/`
- The application is 100% offline once the model file is cached (and blocks all downloads when `"Never use the network"` is enabled).

## Platform Strategy and Packaging Summary
- Windows (`x86_64-pc-windows-msvc`): Primary v1.0 target; NSIS installer (`RustMatte_<ver>_x64-setup.exe`) with WebView2 `embedBootstrapper` (`~1.8 MB`). ONNX Runtime (`ort 2.0.0-rc.15`) is statically linked into `rustmatte-app.exe` on Windows x64 (verified by PE import inspection and isolated clean-directory install + launch).
- macOS (`aarch64-apple-darwin`, experimental): `.dmg` bundle built unsigned on `macos-latest`; Intel `x86_64` Macs are out of scope for v0.1.
- Linux (`x86_64-unknown-linux-gnu`, experimental): `.AppImage` and `.deb` built on `ubuntu-24.04` (`glibc 2.39+` required by prebuilt `ort 2.0.0-rc.15` static archive) targeting system `libwebkit2gtk-4.1-0` and `libgtk-3-0`.
- Release workflow (`.github/workflows/release.yml`) builds CLI archives + desktop installers on `v*` tags and publishes a **draft prerelease** with consolidated `SHA256SUMS.txt` (release procedure in `docs/RELEASING.md`).
- Web Browser (WASM/PWA): Planned for v2.0; Android planned for v3.0.
