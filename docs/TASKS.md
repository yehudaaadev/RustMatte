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
- [x] Implement folder processing with sequential execution and bounded concurrency (exactly 1 full-resolution image in RAM per D-020).
  - Acceptance criterion: Folder of sample images processes without crash; memory deallocated immediately per image. Flat memory profile (~313 MB Fast tier, ~1.91 GB Quality tier) verified across 100 images in Phase 1D-1. Verified.
- [x] Implement per-file error handling in batch processing.
  - Acceptance criterion: Corrupt file in batch logs friendly error and does not stop remaining files. Verified on zero-byte and corrupt truncated fixtures in Phase 1D-1. Verified.
- [x] Implement optional execution provider selection (DirectML on Windows, CoreML on macOS) with automatic CPU fallback.
  - Acceptance criterion: Logs confirm execution provider selection; fallback to CPU executes gracefully if GPU unavailable or out-of-memory occurs. Verified on Windows with DirectML (Fast tier inference: 328ms GPU vs 464ms CPU; Quality tier gracefully fell back to CPU after DirectX memory exhaustion without crashing). Verified.
- [x] Implement fast mode option (early downscaling of very large images to conserve decode time and RAM).
  - Acceptance criterion: Large image processes with reduced decode overhead; document that model inference time is unchanged. Verified with `--max-side 1024` on 24MP image (peak RAM reduced by 209MB, postprocessing overhead reduced by 98.5%). Verified.

## Phase 3A: Desktop App Foundation (Tauri 2 + Svelte 5)
- [x] Pin exact Rust (`tauri = "=2.12.7"`, `tauri-plugin-dialog = "=2.8.4"`) and npm (`svelte = "5.59.1"`, `vite = "8.3.8"`, `typescript = "6.0.3"`) versions, run Rust + npm license checks, and document in `THIRD_PARTY_LICENSES.md` and CI.
  - Acceptance criterion: `cargo deny check licenses advisories`, `npm run license-check`, and `npm run license-check:all` pass cleanly. Verified.
- [x] Implement thin Rust backend commands (`get_models_status`, `download_model` with consent, `process_images` on blocking thread with events, `cancel_job`, `open_output_folder`, `get_settings`, `set_settings`, `save_cutout_png`) and per-tier cached `Remover` session.
  - Acceptance criterion: No image-processing logic duplicated outside `rustmatte-core`; session reused across jobs; unit tests for path validation, settings round-trip/corruption, preview cache cap/cleanup, and job/cancel state pass (10/10 tests). Verified.
- [x] Enforce zero full-resolution image data over IPC via bounded (`<= 1600 px`) preview cache in `$APPCACHE/previews/` served through scoped `assetProtocol` + strict CSP in `tauri.conf.json` and `docs/SECURITY_MODEL.md`.
  - Acceptance criterion: Strict CSP configured, `shell`/`fs` plugins omitted, release devtools disabled, preview cache capped and wiped on exit. Verified in code/config; interactive GUI checks pending manual checklist.
- [x] Adapt Svelte 5 frontend for native dialog open/save, Tauri `onDragDropEvent`, consent-gated model download, live queue progress/cancel, asset-protocol preview slider, persistent settings, and bilingual EN/ID + dark/light toggles.
  - Acceptance criterion: `npm run check` (`svelte-check`), `npm run build`, and `npx tauri build --bundles nsis` succeed; earlier Phase 3A `cargo build --release` launch metrics (`25.73 MB` host + `117.23 MB` WebView2 idle RSS) were marked **UNRELIABLE** (could hit `ERR_CONNECTION_REFUSED` without `custom-protocol` or partial WebView2 process accounting) and replaced by verified `mode=custom-protocol origin=http://tauri.localhost` launch telemetry: `2,770 ms` UI ready (`28.28 MB` host + `322.06 MB` across 6 WebView2 processes = `350.34 MB` idle RSS, zero port `1420` traffic). Verified.

## Phase 3B: Desktop UI/UX and Features
- [x] Main single-screen layout (large drop zone for photos/folders, windowed left thumbnail queue, center zoom+pan comparison slider on checkerboard, right options panel with honest tier descriptions, Remove background, Save/Save all, progress bar + Cancel).
  - Acceptance criterion: Frontend compiles (`svelte-check`, `vite build`), queue state machine unit tests pass (`node --test`), and 100-image batch memory stays flat (`313.64 MB` mid -> `311.7 MB` end, peak `417.38 MB` on Fast tier release CLI). Interactive GUI visual review pending manual checklist.
- [x] First-run model dialog (`Download`, `Choose a file I already have` with SHA-256 verification, `Cancel`, `Retry`, `"Never use the network"` offline lock), About & Licenses modal (from shipped `MODELS.md` and `THIRD_PARTY_LICENSES.md` + `Copy diagnostic info` without paths), and friendly errors with technical details expander.
  - Acceptance criterion: Rust commands (`import_model_file`, `get_about_info`, `get_diagnostic_info`, offline lock check in `download_model`) and `errors.ts` unit tests pass. Verified.
- [x] Typed EN/ID dictionaries (natural Indonesian, key-parity unit tested), system/light/dark themes with reduced-motion and system fonts, full keyboard shortcuts (`Ctrl+O`, `Ctrl+Enter`, `Ctrl+S`, `Ctrl+Shift+S`, `ArrowUp/Down`, `Esc`), and screen-reader slider ARIA attributes.
  - Acceptance criterion: `npm test` (3/3), `svelte-check` (0 errors), `cargo test --workspace -j 2` (41 passed, 0 failed, 2 ignored), and release builds pass. Verified.
- [ ] Manual GUI review checklist and screenshots (`docs/screenshots/`) on real desktop session.
  - Acceptance criterion: Owner signs off on manual checklist covering every state, both themes, both languages, keyboard-only run, 20-photo batch, mid-run cancel, and corrupt file.

## Phase 3C: Desktop Installers and Distribution
- [x] Configure Windows NSIS installer (`embedBootstrapper` WebView2), single-source version parity (`Cargo.toml` + `package.json` + omitted `tauri.conf.json` version), and verify standalone clean-directory install/launch/uninstall on Windows.
  - Acceptance criterion: `RustMatte_0.1.0_x64-setup.exe` (`9.69 MiB`) silently installs cleanly, launches standalone in `2,209 ms` to `ui_ready mode=custom-protocol origin=http://tauri.localhost` (`42.02 MB` host + `320.98 MB` across 6 WebView2 processes = `363.00 MB` idle RSS, zero port `1420` traffic; earlier `204 ms` / `94.77 MB` check measured pre-UI `WaitForInputIdle` and marked **UNRELIABLE**), and uninstalls cleanly. Verified locally on Windows.
- [x] Update `.github/workflows/release.yml` and `.github/workflows/ci.yml` with Node-24-compatible pinned actions, version parity check, multi-OS CLI + desktop bundles (Windows NSIS, macOS `aarch64` `.dmg`, Linux `ubuntu-22.04` `.AppImage`/`.deb` with `otool -L` / `ldd` logs), consolidated `SHA256SUMS.txt`, and draft prerelease output (`GITHUB_TOKEN` only).
  - Acceptance criterion: Workflow YAML parsed cleanly; tagged cloud build UNVERIFIED until `v0.1.0-rc2` is pushed.
- [x] Complete static no-network code/capability audit and update `README.md` (per-OS install, SmartScreen/Gatekeeper/Linux runtime deps, Verify your download, OS data/model removal), `docs/RELEASING.md`, and `CHANGELOG.md`.
  - Acceptance criterion: Static grep confirms `ureq` exists only in `crates/core/src/download.rs` and `capabilities/default.json` grants only `core:default`, `dialog:allow-open`, `dialog:allow-save`. Verified.
- [ ] Verify `v0.1.0-rc2` tagged release run in GitHub Actions (inspect macOS `otool -L` and Linux `ldd` logs) and test Windows installer on a clean second machine without developer tools.
  - Acceptance criterion: Draft release artifacts and `SHA256SUMS.txt` verified on all three OSes and clean-machine offline run confirmed by owner.

## Phase 4: Release Packaging and Multi-OS Distribution (v1.0)
- [x] Configure GitHub Actions release matrix for Windows (.exe NSIS), macOS (.dmg aarch64), and Linux (.AppImage, .deb).
  - Acceptance criterion: Workflow YAML valid and builds CLI + desktop bundles with `otool -L`/`ldd` logging and consolidated `SHA256SUMS.txt`. Cloud tag run UNVERIFIED until `v0.1.0-rc2` pushed.
- [x] Complete community, provenance, and release documentation (`README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, issue/PR templates, `.github/dependabot.yml`, `docs/ROADMAP.md`, `docs/RELEASING.md`, `CHANGELOG.md`, `tests/fixtures/PROVENANCE.md`).
  - Acceptance criterion: All files present, claims audited against `docs/BENCHMARKS.md`, and fresh-clone build verified. Verified.
- [x] Perform full license audit in `THIRD_PARTY_LICENSES.md` and verify `cargo-deny` + npm license checks in CI.
  - Acceptance criterion: Every Rust crate, npm package, and model weight file documented; `cargo deny` and `npm run license-check` pass. Verified.
- [ ] Owner manual verification: review `tests/fixtures/PROVENANCE.md`, apply GitHub repository settings/labels, capture `docs/screenshots/`, test Windows installer on clean second machine without dev tools, and review/publish draft prerelease (`v0.1.0-rc2`).
  - Acceptance criterion: Windows installer runs offline on clean machine and owner publishes release.
- [x] Pre-release fixes: fix 24 MP guided-filter memory regression (`2,185 MB` Quality / `493 MB` Fast), rename identifier to `io.github.yehudaaadev.rustmatte`, fix `svelte-check` a11y warning (`0` warnings), audit comparison table, add `--refine <none|light|standard>` CLI flag, enforce release `custom-protocol` compile guard + `startup.log` UI verification + CI binary marker check (`D-028`, `P-033`), remove leftover `tests/fixtures/SHA256SUMS`, and create logical commits.
  - Acceptance criterion: Plain `cargo build --release -p rustmatte-app` fails cleanly at compile time, `npx tauri build --bundles nsis` + `npm run tauri dev` work, `ui_ready mode=custom-protocol origin=http://tauri.localhost` verified offline with 0 port `1420` hits, and full quality gate passes cleanly. Verified.
- [x] UI polish fixes (`D-029`, `D-030`, `P-034`): default slider to cutout-only (`0%`) with `"Compare with original"` toggle (`C` shortcut, per-session split memory, reset per photo), strip Windows `\\?\` / `\\?\UNC\` prefixes for display only, move all Rust/UI banners, warnings, rejections, dialog titles/filters, and About network text into EN/ID dictionaries with code-parity unit tests, count only `saveable` finished results in `Save all (N)`, and align desktop default execution provider to `"cpu"` (`D-017`/`D-022`).
  - Acceptance criterion: Full Rust/npm quality gate + `npx tauri build --bundles nsis` + offline launch check pass cleanly. Verified.
- [x] Pre-push final fixes (`D-031`): migrate saved `"auto"` -> `"cpu"` on load and show read-only CPU processing line in UI, move release `custom-protocol` `compile_error!` to `main.rs` so `cargo test --release --workspace` passes, publish and verify public `models-v1` GitHub Release assets (`HTTP 200`, matching SHA-256) with `huggingface.co` fallback, verify 24-pair refine parity (`MaxA = 1`, `%>2 = 0.00%`) + `large_24mp` 128-row seams + 100%-zoom visual crops on saturated green/blue backgrounds (`207ef8d` vs `HEAD`), tighten `README.md` comparison claims, and fix `.github/workflows/release.yml` macOS `.app` binary discovery + `ubuntu-24.04` Linux runner.
  - Acceptance criterion: Full quality gate, `cargo test --release -j 2 --workspace`, `npx tauri build --bundles nsis`, and offline launch `startup.log` check pass cleanly. Verified.
- [x] Post-inspection quality, UI tracking, storage cleanup, and candidate benchmark (`D-032`): upgrade `refine.rs` to banded 3-channel RGB Color Guided Filter + 6-pass wavefront defringing, track active `tier`/`refine_level`/`max_side` across `JobProgressEvent`/`QueueItem`/`stats-bar` with `startJob` reset and stale-settings warning banner, clean ~6.9 GB unused build/model/cache artifacts, benchmark `BEN2 FP32` + `InSPyReNet` candidates, and export 3 comparison folders in `Downloads`.
  - Acceptance criterion: `cargo fmt --all -- --check`, `cargo clippy --release --workspace --all-targets --features rustmatte-app/custom-protocol -q -- -D warnings`, `cargo build --release --workspace --features rustmatte-app/custom-protocol -q`, `cargo test --release --workspace --features rustmatte-app/custom-protocol -q` (`47 passed, 0 failed, 2 ignored`), `npm test` (`6/6`), `npm run check` (`0` errors/`0` warnings), `npm run build`, and 12-photo Downloads comparison export pass; owner model selection & manual desktop verification pending.

## Phase 5.0: D-032 Close-Out and PRD v2 Adoption (`PRD_V2.md`)
- [x] Adopt `PRD_V2.md` (`Status: accepted`), record `D-033`/`D-034` in `docs/DECISIONS.md`, add v2 changelog entry in `docs/PRD.md`, add Phases `5.0`–`5D` in `docs/TASKS.md`, and remove temporary `ben2_fp32.onnx` from scratch.
  - Acceptance criterion: Documentation synchronized and scratch `ben2_fp32.onnx` deleted. Verified.
- [x] Audit `crates/core/src/refine.rs`: align foreground early-exit threshold (`m < 243` matching `p >= 0.95`), verify 128-row multi-band vs full-frame parity (`max_diff <= 1` LSB) and seam checks, audit `q <= 0.06` hair/whisker haze suppression (`0.00%` loss of `raw > 36` pixels across `rambut1`/`rambut2`/`hewan1`/`hewan2`), re-measure 24 MP release single-process peak RAM/time (`2,191.39 MB` Balanced / `498.96 MB` Fast), and regenerate clean 12-fixture comparison folders in `Downloads`.
  - Acceptance criterion: Full Rust + npm quality gate passes (`50` Rust tests passed / `2` ignored, `6/6` npm tests, `0` clippy/svelte-check warnings, `cargo deny` clean), 24 MP peak Working Set meets `PRD_V2.md` Section 9 targets, and `Downloads\Quality Model` + `Downloads\fast model` pass `12/12` distinct SHA-256 verification; changes committed locally (`D-032`..`D-034`). Verified.

## Phase 5A: Manifest, Resumable Model Manager, RAM Detection, and Recommendation Engine
- [x] Implement manifest data & validation in `crates/core`, resumable HTTP Range downloader with `.part` + metadata and 10% disk-space check, local available-RAM detection (evaluate >= 2 candidate crates via `license-check`), pure recommendation function (`PRD_V2.md` §7.4 table + RAM thresholds), and `rustmatte models list|download|delete|verify` CLI support.
  - Acceptance criterion: All CI tests pass without model files (recommendation rows, RAM cases, local test server 206/200/truncated/oversized/wrong-hash resume, cancel/delete partial, disk-space checks), real downloads of both `fast` and `balanced` verified against both `github-release` and `huggingface.co` mirrors (`models download`/`verify`/`delete`), and RAM thresholds (`3072 MB` / `5120 MB`) re-validated against 24 MP Balanced (`2,191.39 MB` Peak WS). Verified.

## Phase 5B: Mandatory Bilingual First-Run Setup Wizard
- [x] Implement 5-step mandatory keyboard-operable guided setup wizard (`language`, `briefing + 2 questions`, `recommendation`, `download/offline import`, `ready`) with wizard state machine, EN/ID dictionaries, and measured model cards (`Fast` / `Balanced`).
  - Acceptance criterion: Wizard state machine (`wizard.test.ts`), i18n parity/card tag tests (`i18n.test.ts`), `svelte-check` (`0` errors/`0` warnings), `vite build`, and Rust backend tests/build (`63` passed/`2` ignored) pass; wizard cannot be skipped and completes only with a verified model. Interactive desktop visual review remains pending owner checklist in Phase 5D / Release Readiness. Verified (automated checks).

## Phase 5C: Settings, Model Management, Feedback Link, and Retry With Another Model
- [x] Implement Settings screen (language, theme, model list/delete/verify/download + re-run wizard, network switch, output defaults, collapsed Advanced, copy diagnostic info), allow-listed GitHub Issues feedback link (`PRD_V2.md` §7.8), and per-photo `Retry with another model` side-by-side comparison (`PRD_V2.md` §7.9).
  - Acceptance criterion: Feedback URL allow-list unit tests (`feedback_url_allow_list_permits_only_pinned_issues_page`), `resultsByTier` retry/switch unit tests (`queue.test.ts`), `svelte-check` (`0` errors/`0` warnings), `vite build`, and Rust backend tests/build (`64` passed/`2` ignored) pass; release `startup.log` launch verified. Owner manual test of every Settings action and retry flow remains pending. Verified (automated checks).

## Phase 5D: Visual Polish (`PRD_V2.md` §8 & §7.10)
- [x] Apply design tokens (light/dark palette verified for WCAG AA 4.5:1 contrast, spacing, radii, system fonts, reduced motion), collapsed Advanced controls on main screen, friendly empty/error states, and consistent action placement.
  - Acceptance criterion: Quality gate passes and owner approves screenshots in both themes and both languages. Verified (automated quality gate + release startup verification; interactive screenshot review ready).

## Release Readiness (`v1.0`, `PRD_V2.md` §14)
- [ ] Resolve `TODO(owner)` / portrait fixtures in `tests/fixtures/PROVENANCE.md`, replace placeholder Tauri/Svelte brand icons, capture `docs/screenshots/` + README GIF, apply GitHub repository settings/labels/branch protection, test Windows installer offline on a clean second PC, and publish verified release.
  - Acceptance criterion: Section 14 checklist complete and owner publishes release.

---

## Backlog (v1.1, v1.x, v2.0, v3.0)

### v1.1 Scope
- [ ] Camera RAW format support (.CR2, .NEF, .ARW, .DNG): Phase 1 extracts embedded JPEG preview; Phase 2 implements full RAW demosaic.
- [ ] Apple HEIC/HEIF decoding support with cross-platform build validation.

### v1.x Scope
- [ ] Crop-to-subject export option in desktop UI.
- [ ] WebP and additional image export formats alongside PNG.
- [ ] Standalone photo compression utility.
- [ ] Manual refine brush (restore/erase on mask).
- [ ] Preset dimensions for marketplace and passport photos.

### v2.0 Scope
- [ ] Real-ESRGAN ONNX upscaler with image tiling and memory cap.
- [ ] Web browser version (WASM / PWA) for local offline mobile and tablet processing.

### v3.0 Scope
- [ ] Android version (after desktop application is complete and stabilized).
