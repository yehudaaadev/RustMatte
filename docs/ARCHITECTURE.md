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

### `crates/core` (Implemented in Phase 1C-1 and Phase 1C-2)
Core background removal engine library. Contains no UI code and performs zero terminal printing.
- `src/error.rs`: Strongly typed `CoreError` using `thiserror` (handling decode errors, dimension bounds, missing models, checksum mismatches, ORT session errors, network/download errors, and I/O).
- `src/model.rs`: Defines `ModelTier` (`Quality`, `Fast`), `ModelSpec` manifest (metadata, sizes, SHA-256 hashes, licenses, source URLs), OS cache directory resolution via `dirs`, and hierarchical lookup (`--model-path` > `RUSTMATTE_MODEL_DIR` > OS cache dir > `./models/`).
- `src/download.rs`: Safe streaming HTTPS model downloader with localhost test overrides, `.part` atomic rename, redirect limits, timeouts, progress callbacks, and SHA-256 verification.
- `src/session.rs`: Handles ONNX Runtime session creation. Enforces D-015 memory allocator settings (`with_memory_pattern(false)` and `session.use_env_allocators = 1`) and calculates default thread heuristic (half logical CPUs clamped to 2..=8).
- `src/decode.rs`: Safe image decoding with dimension bounds (16384px limit, 512MB max allocation), EXIF orientation correction, and truncation/damage detection (`Warning::DamagedInput`).
- `src/pipeline.rs`: Preprocessing (RGB triangle resize and NCHW tensor normalization), postprocessing (raw logits sigmoid/min-max to 8-bit mask and bilinear upscaling), `refine_mask` hook for Phase 1D, and `apply_mask` alpha blending.
- `src/lib.rs`: Minimal public API (`Config`, `Output`, `StageTimings`, `Remover::new`, `Remover::process`, `download_model`). Manages global ORT environment safely across instances via `std::sync::OnceLock`.

### `crates/cli` (Implemented through Phase 1D-1)
Lightweight command-line interface binary using `clap` (derive). Thin wrapper around `crates/core`.
- `src/main.rs`: Argument parsing (`rustmatte <INPUT> [-o <OUTPUT>] [--tier quality|fast] [--model-path <FILE>] [--threads N] [--overwrite] [--skip-existing] [--max-side N] [--recursive] [--yes] [--no-download] [--verbose]`), prefetch subcommand (`rustmatte models [list|download]`), interactive download disclosure/consent (`[y/N]`), atomic file creation via temp files, warning output to stderr, and `--verbose` per-stage execution timing breakdown. Returns exit codes: `0` (all ok), `1` (total failure / bad args), `2` (partial failure).
- `src/batch.rs`: Batch image discovery, recursive directory traversal with symlink traversal boundaries, batch limit warnings (>1,000 files), and deterministic collision resolution (conflicting files with identical stems in the same folder resolve to `<stem>_<ext>.png`).

### `crates/app` (Planned for Phase 3)
Desktop application using Tauri 2 with a web frontend (Svelte/TypeScript). Thin wrapper around `crates/core`.

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

## Model Storage and Download Flow
- Models are never committed to git and never bundled into the v1.0 installer.
- On first launch, the required model downloads to the user's OS application data directory:
  - Windows: `%LOCALAPPDATA%\RustMatte\models\` (planned)
  - macOS: `~/Library/Application Support/RustMatte/models/` (planned)
  - Linux: `~/.local/share/RustMatte/models/` (planned)
- Every downloaded model file is verified with SHA-256 before use.
- The application is 100% offline once the model file is cached.

## Platform Strategy Summary
- Windows: Primary target for v1.0 desktop app via Tauri 2; full manual testing. Installer handles WebView2 bootstrapper.
- macOS and Linux: Experimental for v1.0; built and tested in CI from Phase 0.
- Web Browser (WASM/PWA): Planned for v2.0 to support mobile devices and tablets locally.
- Android: Planned for v3.0 after desktop application is complete and stable.
