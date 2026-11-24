# RustMatte Tasks and Acceptance Criteria

Rule: Only the owner approves moving to the next phase. Never mark a phase done without running verification commands.

## Phase 0: Workspace Setup and Foundation
- [x] Initialize Cargo workspace root `Cargo.toml` with `crates/core` and `crates/cli` members.
  - Acceptance criterion: `cargo check --workspace` passes without errors. Verified.
- [x] Create `rust-toolchain.toml` pinning Rust stable channel (1.98.1).
  - Acceptance criterion: Toolchain adheres to pinned version. Verified.
- [x] Create `LICENSE` file with official GPL-3.0 text fetched from gnu.org.
  - Acceptance criterion: File exists and matches decision D-014. Verified.
- [x] Create skeleton `README.md` with project description, build instructions, and trademark disclaimer.
  - Acceptance criterion: README documents build and test procedures without unmeasured performance claims. Verified.
- [x] Setup GitHub Actions CI workflow (`.github/workflows/ci.yml`) for Windows, macOS, and Linux.
  - Acceptance criterion: Workflow YAML is valid (verified via python yaml parser) and defines build, test, clippy, and fmt checks across all three OS. UNVERIFIED in cloud until pushed.
- [x] Record project license decision (GPL-3.0-or-later) and decisions in `docs/DECISIONS.md`.
  - Acceptance criterion: Entry exists and is approved by owner. Verified.
- [x] Verify clean workspace build.
  - Acceptance criterion: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --all -- --check` all exit with code 0. Verified.

## Phase 1: Core Engine and CLI (v0.1)
- [x] Phase 1A: Inference smoke test for `BiRefNet_lite` on CPU via `ort`.
  - Acceptance criterion: Model loads and executes on real CPU through `ort`, handles input/output tensor shapes and EXIF orientation, produces transparent PNG cutouts on at least 5 fixture images, reports median timings and peak memory. Verified on 6 fixtures (`rambut1`, `hewan1`, `produk1`, `kaca1`, `latarramai1`, `orang1`).
- [x] Define error types in `crates/core::error` using `thiserror`.
  - Acceptance criterion: Unit tests verify error formatting without panics. Verified.
- [x] Phase 1C-1: Local model resolution and SHA-256 integrity verification in `crates/core::model`.
  - Acceptance criterion: Verifies SHA-256 checksum against pinned reference hash, rejects corrupt hash, and locates model in `./models/`. Verified.
- [x] Phase 1C-2: Implement automatic model downloader in `crates/core::model` and `crates/core::download` (download, SHA-256 verification, local caching in OS app data, consent prompt, CLI `models` subcommand).
  - Acceptance criterion: Test verifies cache hit for existing valid model, rejects corrupt hash, cleans up `.part` files on failure, enforces HTTPS (with localhost override for testing), and executes end-to-end mock and local server downloads; no model bundling in repository. Verified.
- [x] Implement image pre-processing in `crates/core::pipeline` (aspect-ratio resize, normalization to tensor for standard formats: JPEG, PNG, WebP, BMP).
  - Acceptance criterion: Test validates normalized tensor output format; unsupported formats return typed error. Verified.
- [x] Phase 1C: Implement image decoding with damage/truncation warning flag in `crates/core::decode`.
  - Acceptance criterion: Damaged or truncated files decode with a warning flag so CLI/UI can report "file is damaged or incomplete" while still producing output when possible; valid files decode without warning flag. Verified on `corrupt_truncated.jpg`.
- [x] Implement ONNX inference session in `crates/core::session` using pinned `ort` on CPU by default with D-015 allocator.
  - Acceptance criterion: Inference runs against real model on CPU and produces raw probability mask output. Verified.
- [x] Benchmark candidate models (`BiRefNet_lite` vs `ISNet` vs `U-2-Netp`) on CPU using `tests/fixtures` (Phase 1B).
  - Acceptance criterion: Real CPU timings, memory consumption, supply-chain verification (source URL, license, SHA-256, operator compatibility in `ort`), robustness testing (zero-byte, truncated, 24MP), threading sweep, and visual quality review package (`bench/out/compare/` and `bench/out/index.html`) generated and reported in `docs/BENCHMARKS.md` for owner decision. Two-tier architecture accepted in D-018. Verified.
- [x] Implement basic post-processing in `crates/core::pipeline` (upscale mask, bilinear interpolation, alpha blending).
  - Acceptance criterion: Generates valid 32-bit RGBA image matching source dimensions. Verified.
- [x] Phase 1D-1: Housekeeping, batch processing, and release readiness.
  - [x] Step 0: Toolchain verification (Rust 1.98.1), transitive license audit via `cargo-deny`, `deny.toml` creation, and CI cargo-deny integration.
  - [x] Step 1: Batch folder processing in `rustmatte` CLI with `--recursive`, `--out <DIR>`, deterministic collision resolution (`<stem>_<ext>.png`), `--skip-existing`, `--overwrite`, and tri-state exit codes (`0`, `1`, `2`).
  - [x] Step 1: Early downscaling `--max-side <N>` in core pipeline and CLI, bounding decode and postprocessing overhead on 24MP images.
  - [x] Step 2: Batch verification on 100 images (97 valid + 1 corrupt truncated + 1 zero-byte + 1 fake text): verified flat memory profile (~313 MB flat on Fast tier, ~1.91 GB flat on Quality tier, zero memory leaks).
  - [x] Step 2: Benchmarked `--max-side 1024` on 24MP image: reduced peak RAM by 209 MB and cut postprocessing time from 2883ms to 43ms (-98.5%).
  - [x] Step 2: Ctrl+C interruption and `--skip-existing` recovery: verified 0 orphaned temp files and clean resumption of skipped files.
  - [x] Step 3: Multi-platform release packaging workflow `.github/workflows/release.yml` with SHA-256 checksums and automated release notes.
  - [x] Step 4: Documentation synchronization (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`, `docs/PITFALLS.md`).
- [x] Phase 1D-2: Implement advanced post-processing in `crates/core::pipeline` (guided-filter refinement, feathering in transition zone, defringing).
  - Acceptance criterion: Refinement filter smooths transition zones and defringes semi-transparent edge halos. Verified on `rambut1.jpg` and `rambut2.jpg` (overhead: 25ms - 177ms, edge halos suppressed, interior opacity preserved).
- [x] Implement CLI single-image command in `crates/cli` (`rustmatte input.jpg -o output.png`).
  - Acceptance criterion: Running CLI on sample image creates valid transparent PNG with network disconnected after model cache. Verified.
- [x] Test CLI across varied sample images.
  - Acceptance criterion: Diverse sample images processed with acceptable transparent cutout quality (verified across cat, person, shirt, 24MP with bit-for-bit regression parity against Phase 1B). Verified.

## Phase 2: Batch Processing, Formats, and Performance (v0.2)
- [ ] Implement folder processing with `rayon` and bounded concurrency (1-2 full-resolution images in RAM).
  - Acceptance criterion: Folder of sample images processes without crash; memory deallocated immediately per image.
- [ ] Implement per-file error handling in batch processing.
  - Acceptance criterion: Corrupt file in batch logs friendly error and does not stop remaining files.
- [ ] Implement optional execution provider selection (DirectML on Windows, CoreML on macOS) with automatic CPU fallback.
  - Acceptance criterion: Logs confirm execution provider selection; fallback to CPU executes gracefully if GPU unavailable.
- [ ] Implement fast mode option (early downscaling of very large images to conserve decode time and RAM).
  - Acceptance criterion: Large image processes with reduced decode overhead; document that model inference time is unchanged.

## Phase 3: Desktop UI with Tauri 2 (v0.3)
- [ ] Scaffold Tauri 2 application in `crates/app` with frontend (Svelte + TypeScript).
  - Acceptance criterion: Application window launches with drag-and-drop zone and queue.
- [ ] Implement before/after comparison slider with zoom and pan.
  - Acceptance criterion: Slider interactively reveals original vs transparent output.
- [ ] Implement output settings (background color picker, format, fast mode toggle, export directory).
  - Acceptance criterion: Saving exports file according to selected settings.
- [ ] Implement bilingual support (English and Indonesian) and light/dark theme.
  - Acceptance criterion: Toggling language and theme updates all UI text instantly.
- [ ] Configure Windows installer with WebView2 bootstrapper.
  - Acceptance criterion: Installer builds and runs cleanly on Windows.

## Phase 4: Release Packaging and Multi-OS Distribution (v1.0)
- [ ] Configure GitHub Actions release matrix for Windows (.msi, .exe), macOS (.dmg), and Linux (.AppImage, .deb).
  - Acceptance criterion: CI builds release artifacts across all three platforms on git tag; macOS and Linux labeled experimental.
- [ ] Complete documentation: README, CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue templates.
  - Acceptance criterion: All documentation files present and reviewed.
- [ ] Perform full license audit in `THIRD_PARTY_LICENSES.md`.
  - Acceptance criterion: Every dependency and model weight file documented and verified.
- [ ] Test release installers on clean Windows machines without developer tools.
  - Acceptance criterion: Windows installer runs correctly offline.

---

## Backlog (v1.1, v1.x, v2.0, v3.0)

### v1.1 Scope
- [ ] Camera RAW format support (.CR2, .NEF, .ARW, .DNG): Phase 1 extracts embedded JPEG preview; Phase 2 implements full RAW demosaic.
- [ ] Apple HEIC/HEIF decoding support with cross-platform build validation.

### v1.x Scope
- [ ] Standalone photo compression utility.
- [ ] Manual refine brush (restore/erase on mask).
- [ ] Preset dimensions for marketplace and passport photos.

### v2.0 Scope
- [ ] Real-ESRGAN ONNX upscaler with image tiling and memory cap.
- [ ] Web browser version (WASM / PWA) for local offline mobile and tablet processing.

### v3.0 Scope
- [ ] Android version (after desktop application is complete and stabilized).
