# Project State: RustMatte

## Current Phase
Phase 1D-1 (Housekeeping, Batch Mode, and Release Readiness) - Completed and verified. Awaiting owner confirmation before starting Phase 1D-2 (Advanced Post-Processing & Edge Refinement).

## Last Session Summary
Executed Phase 1D-1 end-to-end:
- **Step 0: Pre-Work Checks & Audits**:
  - Compiler verification: Confirmed active compiler is `rustc 1.98.1` / `cargo 1.98.1` strictly matching `rust-toolchain.toml`. Corrected report typo in `docs/BENCHMARKS.md`.
  - Real-world download verification: Verified live model download from network (4,697,620 bytes in 4,514 ms), confirmed fallback to HuggingFace because GitHub Release `models-v1` returned 404 (not yet uploaded by owner), and verified SHA-256 integrity.
  - License audit via `cargo-deny 0.22.0`: Created `deny.toml` with GPL-3.0-or-later compatible allow-list. Flagged `option-ext` (MPL-2.0, standard `dirs` dependency), `webpki-roots` (CDLA-Permissive-2.0, root CA bundle), and `icu_*` (Unicode-3.0). Ran `cargo deny check licenses` and `cargo deny check advisories` (both passed with 0 advisories). Added `cargo-deny-action@v2.1.3` to `.github/workflows/ci.yml`.
- **Step 1: Batch Mode & Early Downscaling**:
  - Implemented `pipeline::downscale_if_needed` and `Remover::process_with_options` supporting `--max-side <N>` in `crates/core`.
  - Built batch engine in `crates/cli/src/batch.rs`: directory discovery, `--recursive` hierarchy scanning, symlink containment checks, batch limit warnings (>1,000 files), and deterministic collision resolution (`<stem>_<ext>.png`).
  - Added CLI options: `--out <DIR>` (defaulting to `rustmatte-output` next to input), `--skip-existing`, `--overwrite`, and tri-state exit codes (`0` all ok, `1` total failure / invalid args, `2` partial failure).
  - Single shared ORT session initialized once; strictly sequential image processing with bounded single-image lifecycle in RAM.
- **Step 2: Batch Verification & Empirical Measurements**:
  - Benchmarked 100-image batch (`bench/fixtures_batch_100/` with 97 valid images, 1 corrupt JPEG, 1 zero-byte file, 1 fake text file):
    - **Fast Tier**: 98 processed, 0 skipped, 2 failed, 1 warning. Total time: 34.34s (avg 349.5 ms/image). RAM remained flat across all 100 images (~314 MB mid -> 313 MB end, peak 331.28 MB).
    - **Quality Tier**: 98 processed, 0 skipped, 2 failed, 1 warning. Total time: 299.02s (avg 3,049.9 ms/image). RAM remained flat across all 100 images (~1,958 MB mid -> 1,915 MB end, peak 2,022.59 MB).
  - High-resolution 24MP benchmark (`large_24mp.jpg`):
    - Without `--max-side`: 11,926 ms total (postprocess 2,883 ms), peak RAM 2,217.97 MB.
    - With `--max-side 1024`: 10,432 ms total (postprocess 43.65 ms, -98.5%), peak RAM 2,008.58 MB (-209.39 MB).
  - Ctrl+C interruption resilience: Terminated batch after 5 images; confirmed 0 `.tmp` partial files remained. Subsequent run with `--skip-existing` cleanly skipped all 5 existing images and completed remaining images.
- **Step 3: Release Readiness**:
  - Created `.github/workflows/release.yml` with multi-platform matrix (Windows zip with DirectML runtime DLL, macOS tar.gz, Ubuntu tar.gz), consolidated SHA256SUMS.txt, and automated release notes generation from CHANGELOG.md (marked UNVERIFIED).
  - Created `CHANGELOG.md` adhering to Keep a Changelog.
  - Updated `README.md` and `CONTRIBUTING.md` documenting batch mode, exit codes, and MSVC `-j 2` linker memory constraints.
- **Step 4: Architecture, Decisions, and Pitfalls Sync**:
  - Recorded D-020 (Batch processing architecture and memory bounding).
  - Recorded P-025 (MSVC link.exe LNK1102 memory limits) and P-026 (Child process pipe buffer deadlock avoidance).
  - Updated `docs/ARCHITECTURE.md` and `docs/TASKS.md`.

## Done
- Phase 0: Workspace setup, toolchain pinning, GPL-3.0 licensing, CI workflow, skeleton crates.
- Phase 1A: Inference smoke test with BiRefNet_lite on CPU via `ort` on 6 diverse real fixtures.
- Phase 1B: Model benchmark matrix (5 models across 12 fixtures, robustness tests, ORT sweeps).
- Phase 1B-FIX: Isolated per-model memory benchmarking, contact sheet regeneration, license audit, and documentation correction.
- Phase 1C-1: Core engine pipeline, decode safety, model checksum verification, and single-image CLI.
- Phase 1C-2: Automatic model downloader, OS local caching, consent prompt, CLI models subcommand, and per-stage timing visibility.
- Phase 1D-1: Housekeeping, transitive license audit, batch folder mode, early downscaling, flat memory verification, and release packaging workflow.

## In Progress
- Awaiting owner confirmation before starting Phase 1D-2 (Advanced Post-Processing & Edge Refinement).

## Known Broken or Failing
- None. All unit tests, integration tests, cargo clippy, and cargo deny checks pass cleanly with 0 errors.

## Next Step
- Phase 1D-2: Implement advanced post-processing in `crates/core::pipeline` (guided-filter edge refinement, transition zone feathering, and defringing to clean halo artifacts).

## Open Questions for the Owner
- Does the owner confirm Phase 1D-1 completion to proceed to Phase 1D-2?

## Last Updated
2026-11-24
