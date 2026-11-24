# Project State: RustMatte

## Current Phase
Phase 1C-1 (Core Pipeline and Single-Image CLI) - Completed and verified. Awaiting owner confirmation before starting Phase 1C-2 (Automatic Model Downloader).

## Last Session Summary
Executed Phase 1C-1 end-to-end:
- Reused verified logic from `crates/core/examples/bench_matrix.rs` and modularized `crates/core`:
  - `error.rs`: Strongly typed `CoreError` using `thiserror` (handling decode, limits, models, hashes, ORT, and I/O).
  - `model.rs`: `ModelTier` (`Quality`, `Fast`), `ModelSpec` with parameters cited from benchmarks, and SHA-256 integrity verification via `sha2` crate.
  - `session.rs`: Centralized ORT session builder applying D-015 memory allocator configuration and thread heuristic (half logical CPUs clamped to 2..=8).
  - `decode.rs`: Safe decoding with limits (16384px dimension cap, 512MB max allocation), EXIF orientation handling, and truncation detection returning `Warning::DamagedInput`.
  - `pipeline.rs`: NCHW normalization, logits to probability mask (sigmoid / min-max), `refine_mask` hook for Phase 1D, and alpha blending.
  - `lib.rs`: Minimal public API (`Config`, `Output`, `Remover::new`, `Remover::process`) with process-wide ORT environment singleton via `std::sync::OnceLock`.
- Implemented single-image CLI in `crates/cli` (`rustmatte <INPUT> -o <OUTPUT> [--tier quality|fast] [--model-path <FILE>] [--threads N] [--overwrite]`) using pinned `clap` v4.5.31. Employs atomic write (temp file in target directory + rename) and prints warnings to stderr.
- Regression verification: Ran both tiers across 3 fixtures (cat `hewan1.jpg`, person `orang1.jpg`, shirt `baju1.jpg`) into `bench/out/cli/` and verified 100% bit-for-bit SHA-256 match against Phase 1B bench outputs (IoU = 1.000, 0 pixel difference).
- Measured CLI performance in release mode with one fresh process per tier on this machine:
  - Quality tier on `orang1.jpg` (683x1024): 6.75s total, 2,011.42 MB Peak Working Set (confirmed ~2 GB).
  - Quality tier on `large_24mp.jpg` (6000x4000): 10.47s total, 2,238.61 MB Peak Working Set (bounded RAM).
  - Fast tier on `orang1.jpg`: 608.96 ms total, 317.96 MB Peak Working Set.
  - Fast tier on `large_24mp.jpg`: 2.15s total, 582.11 MB Peak Working Set.
- Quality gates passed: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and ignored tests (`cargo test -p rustmatte-core -- --ignored`).
- Documented D-018 final acceptance in `docs/DECISIONS.md`, updated `docs/TASKS.md`, `docs/ARCHITECTURE.md`, `THIRD_PARTY_LICENSES.md`, `README.md`, and appended P-021 in `docs/PITFALLS.md`.

## Done
- Phase 0: Workspace setup, toolchain pinning, GPL-3.0 licensing, CI workflow, skeleton crates.
- Phase 1A: Inference smoke test with BiRefNet_lite on CPU via `ort` on 6 diverse real fixtures.
- Phase 1B: Model benchmark matrix (5 models across 12 fixtures, robustness tests, ORT sweeps).
- Phase 1B-FIX: Isolated per-model memory benchmarking, contact sheet regeneration, license audit, and documentation correction.
- Phase 1C-1: Core engine pipeline, decode safety, model checksum verification, and single-image CLI.

## In Progress
- Awaiting owner confirmation before starting Phase 1C-2 (Automatic Model Downloader).

## Known Broken or Failing
- None. Core pipeline and CLI operate cleanly with zero panics or regressions.

## Next Step
- Phase 1C-2: Implement model downloader in `crates/core::model` (downloading on demand with progress reporting, resume on failure, local caching in OS app data, and SHA-256 verification).

## Open Questions for the Owner
- Does the owner confirm Phase 1C-1 completion to proceed to Phase 1C-2?

## Last Updated
2026-11-24
