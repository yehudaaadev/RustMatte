# Project State: RustMatte

## Current Phase
Phase 1C-2 (Model Downloader, OS Cache, and CLI Prefetch) - Completed and verified. Awaiting owner confirmation before starting Phase 1D (Advanced Post-Processing & Edge Refinement).

## Last Session Summary
Executed Phase 1C-2 end-to-end:
- Added pinned dependencies in `crates/core`: `ureq = "=2.10.1"` (pure-Rustls + webpki-roots; zero OpenSSL) and `dirs = "=6.0.0"`.
- Pinned and committed `Cargo.lock`; removed `Cargo.lock` from `.gitignore`.
- Built model distribution manifest in `crates/core/src/model.rs`:
  - `ModelSpec` struct registering exact file names, sizes, SHA-256 hashes, licenses, descriptions, and candidate source URLs (project release placeholder + upstream fallback).
  - Defined OS cache location via `dirs::data_local_dir().join("rustmatte/models")`.
  - Implemented hierarchical lookup: `--model-path` > `RUSTMATTE_MODEL_DIR` > OS cache dir > `./models/`.
  - Enforced strict SHA-256 validation on any discovered local file.
- Built safe streaming HTTPS downloader in `crates/core/src/download.rs`:
  - Strict HTTPS enforcement (HTTP permitted only for localhost when `RUSTMATTE_ALLOW_INSECURE_LOCALHOST=1` is set).
  - Stream size caps (aborts on truncation or oversized payload), timeouts, redirects, and atomic write via temporary `.part` files.
  - Automatic cleanup of `.part` files on failure; files renamed to destination only upon passing SHA-256 validation.
- Extended CLI in `crates/cli/src/main.rs`:
  - Added `models [list|download]` subcommands while preserving full compatibility with `rustmatte <INPUT> -o <OUTPUT>`.
  - Interactive disclosure & consent prompt (`[y/N]`) before any network attempt; `--yes` for automated environments; `--no-download` to strictly forbid network access.
  - Real-time download progress reporting on stderr.
  - Added `--verbose` flag reporting per-stage timing breakdown (model load, decode, preprocess, inference, postprocess, write).
- Measured CLI performance & cold-start timing:
  - Quality tier model load cold-start: 3,680.64 ms; inference: 2,992.24 ms; total: 6,798.40 ms (confirms model load accounts for ~3.7s of cold-start).
  - Fast tier model load: 146.48 ms; inference: 243.22 ms; total: 458.91 ms.
  - Binary size impact: 24,456,704 bytes -> 26,535,936 bytes (+1.98 MB). Transitive dependencies: 98 -> 153 (+55 packages, zero openssl).
- Quality gates passed:
  - `cargo fmt --all -- --check` passed.
  - `cargo clippy --workspace --all-targets -- -D warnings` passed with 0 warnings.
  - `cargo test --workspace` passed 17 tests (9 download/mock tests + 8 core unit tests; 2 ignored model tests).
  - End-to-end download & cutout test passed with real models served via local HTTP server.
- Documentation synchronized:
  - Created `MODELS.md` cataloging official model licenses, attribution, and academic dataset provenance.
  - Updated `docs/ARCHITECTURE.md`, `docs/DECISIONS.md` (D-019), `docs/PITFALLS.md` (P-022 to P-024), `THIRD_PARTY_LICENSES.md`, and `README.md`.
  - Corrected Windows version in `docs/BENCHMARKS.md` and `README.md`: Microsoft Windows 11 Pro (build 26200).
  - Clarified in docs that glass is segmented as solid due to model architecture, and Phase 1D edge refinement only smooths boundaries.

## Done
- Phase 0: Workspace setup, toolchain pinning, GPL-3.0 licensing, CI workflow, skeleton crates.
- Phase 1A: Inference smoke test with BiRefNet_lite on CPU via `ort` on 6 diverse real fixtures.
- Phase 1B: Model benchmark matrix (5 models across 12 fixtures, robustness tests, ORT sweeps).
- Phase 1B-FIX: Isolated per-model memory benchmarking, contact sheet regeneration, license audit, and documentation correction.
- Phase 1C-1: Core engine pipeline, decode safety, model checksum verification, and single-image CLI.
- Phase 1C-2: Automatic model downloader, OS local caching, consent prompt, CLI models subcommand, and per-stage timing visibility.

## In Progress
- Awaiting owner confirmation before starting Phase 1D (Advanced Post-Processing & Edge Refinement).

## Known Broken or Failing
- None. All unit, integration, and CLI commands compile and pass cleanly without warnings.

## Next Step
- Phase 1D: Implement advanced post-processing in `crates/core::pipeline` (guided-filter edge refinement, transition zone feathering, and defringing to clean halo artifacts).

## Open Questions for the Owner
- Does the owner confirm Phase 1C-2 completion to proceed to Phase 1D?

## Last Updated
2026-11-24
