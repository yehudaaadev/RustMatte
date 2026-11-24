# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Tauri 2 Desktop Application (`crates/app`):
  - Native standalone executable `rustmatte-app.exe` (34.5 MB).
  - Modern Svelte 5 frontend with runes (`$state`, `$derived`).
  - Interactive Before/After split comparison slider with touch and mouse drag handles.
  - Background color preview picker (Transparent checkerboard, Solid White, Solid Black, Custom HEX color).
  - Drag-and-drop file queue supporting single and batch image additions.
  - Real-time output settings (Quality vs Fast tier, CPU vs DirectML GPU device, early downscaling slider, edge refinement toggle).
  - Model status detection and on-demand background model downloader in UI.
  - Bilingual localization (English and Bahasa Indonesia) with instant reactive switching.
  - Dark and Light theme toggle with localStorage persistence.
  - Per-stage latency breakdown display (Decode, Preprocess, Inference, Refinement).
- Hardware execution providers for ONNX Runtime:
  - Added `--device <cpu|directml|coreml|auto>` CLI argument (default: `cpu`).
  - Added `directml` cargo feature enabling DirectX 12 hardware acceleration on Windows.
  - Added `coreml` cargo feature for Apple Silicon / Neural Engine on macOS.
  - Multi-stage graceful fallback to CPU during session creation and runtime inference (e.g., when GPU memory is exhausted on large models).
  - Explicit hardware session deallocation before CPU fallback to prevent dual-session memory exhaustion.
  - Active execution provider reporting in verbose output and batch summary reports.
- Advanced edge refinement pipeline (`crates/core::refine`):
  - Fast Guided Filter with separable 2D box filtering ($O(N)$ runtime) using original grayscale luminance guidance.
  - Selective transition zone feathering: locks interior foreground ($\ge 0.95$) to full opacity and background ($\le 0.05$) to full transparency, preventing blurry boundaries while refining uncertain edges.
  - Boundary color defringing: $5 \times 5$ neighborhood decontamination extrapolating solid foreground RGB into semi-transparent edge pixels ($\alpha < 235$) to eliminate colored halos.
- `--no-refine` CLI flag to disable postprocessing edge refinement and export raw segmentation masks.
- Batch processing mode in `rustmatte` CLI: support directory inputs with automatic detection of supported image formats (`.jpg`, `.jpeg`, `.png`, `.webp`, `.bmp`).
- `--out <DIR>` output directory option (defaults to `rustmatte-output` adjacent to input directory).
- `--recursive` / `-r` flag for recursive directory scanning with symlink traversal guards.
- Deterministic collision handling: automatically formats conflicting output filenames as `<stem>_<ext>.png`.
- `--skip-existing` flag to resume interrupted batches without reprocessing existing outputs.
- `--max-side <N>` early downscaling to bound processing time and peak RAM on high-resolution images.
- Batch progress reporting to `stderr`: `[index/total] filename: status (elapsed)`.
- Batch summary report: processed count, skipped count, failure count, warnings, total execution time, and average per-image duration.
- Tri-state exit codes: `0` (all ok), `1` (total failure or invalid arguments), `2` (partial failure).
- Automated model downloader (`crates/core::download`) with one-time user consent prompt, progress indicator, SHA-256 verification, and atomic rename.
- Hierarchical model lookup chain: `--model-path` > `RUSTMATTE_MODEL_DIR` > OS cache directory > `./models/`.
- `models list` and `models download` CLI subcommands for offline model weight management.
- License and security auditing via `cargo-deny` (`deny.toml`) integrated into CI.
- Multi-platform GitHub Actions release workflow (`.github/workflows/release.yml`) for Windows, macOS, and Linux.

### Changed
- Standardized image decoding limits: `MAX_IMAGE_DIMENSION = 16384` and `MAX_ALLOCATION_BYTES = 512 MB`.
- EXIF orientation handling with automatic image rotation.
- Pinned ONNX Runtime session configuration adhering to Decision D-015.

### Fixed
- Fixed MSVC `link.exe` out-of-memory errors by restricting concurrent build jobs (`-j 2`).
- Fixed Linux/macOS build failures by gating Windows-specific `psapi` linkage in test examples behind `#[cfg(windows)]`.
