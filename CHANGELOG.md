# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
