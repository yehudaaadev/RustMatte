# RustMatte Project State

## Current Phase
Phase 5A Complete (`D-035`: manifest, RAM/disk probing, pure recommendation engine, resumable downloader, and CLI `models list|download|delete|verify` verified and committed locally) — ready to begin **Phase 5B** (Mandatory 5-step bilingual first-run setup wizard) upon owner confirmation.

## Last Session Summary
Implemented and verified **Phase 5A** (`PRD_V2.md` §7.1–7.4, §10–12, `D-035`):
- **Model Manifest (`crates/core/src/manifest.rs`)**: Defined `ModelManifestEntry` schema (`balanced` / `birefnet_lite_512` and `fast` / `u2netp`), tensor input/output specs, measured reference benchmarks, dictionary tag codes, and `validate_manifest` with unit tests.
- **RAM & Disk Probing (`crates/core/src/sysinfo.rs`)**: Added `sysinfo = "=0.40.4"` (`system`, `disk` features only; `MIT` license registered in `THIRD_PARTY_LICENSES.md` and `cargo deny` clean) to probe `available_memory()` and target filesystem `available_space()`, plus `required_space_with_margin` (`model_size + 10%`) and injectable CI test probes.
- **Pure Recommendation Engine (`crates/core/src/recommend.rs`)**: Implemented all 10 table combinations from `PRD_V2.md` §7.4, named `LOW_RAM_THRESHOLD_MB = 3072` / `MID_RAM_THRESHOLD_MB = 5120` constants (re-validated against `2,191.39 MB` Balanced 24 MP Peak WS), installed-model detection, and injectable RAM probe tests.
- **Resumable Downloader & Cache Management (`crates/core/src/download.rs`, `model.rs`, `error.rs`, `crates/cli/src/main.rs`)**: Added `size + 10%` disk preflight (`InsufficientDiskSpace`), HTTP `Range` resume (`206 Partial Content`, `200 OK` fallback, `.part` + `.part.meta.json`, retry backoff, cancel preserving `.part`, `delete_partial_download`), full-file SHA-256 verification before atomic rename, `delete_cached_model`, and `rustmatte models list|download|delete|verify`.
- **Real Download & Quality Verification**: Verified `rustmatte models download --tier <fast|balanced> -y`, `verify`, and `delete -y` end-to-end against both `github-release` (`models-v1`) and `huggingface.co` mirrors (`u2netp.onnx` `4.36 MB`, `birefnet_lite_512.onnx` `182.99 MB`), restored original cache, and passed the full Rust + npm quality gate.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028`..`D-031`) + `models-v1` release + `v0.1.0-rc4` multi-OS CI/Release verification + `D-032` refinement/UI tracking + `PRD_V2.md` adoption & **Phase 5.0** (`D-033`..`D-034`) + **Phase 5A** (`D-035`).

## In Progress
- None in code; awaiting owner confirmation to start **Phase 5B**.

## Known Broken or Failing
- None in automated checks.
- Known model limits remain documented: transparent glass (`kaca1`) segments as solid; `birefnet_lite_512` can close small interior gaps (`baju1`) and clip partial fingertips (`produk1`/`produk2`).
- Carry-over owner/asset items remain open for Release Readiness (`PRD_V2.md` §3 & §14): `TODO(owner)` / portrait fixtures in `tests/fixtures/PROVENANCE.md`, custom brand icons replacing Tauri/Svelte placeholders, `docs/screenshots/` + README GIF, manual desktop UI verification, clean second-PC Windows installer test, and GitHub settings/publish.

## Next Step
- Begin **Phase 5B**: implement the mandatory 5-step bilingual guided setup wizard (`language`, `briefing + 2 questions`, `recommendation`, `download/offline import`, `ready`), wizard state machine + unit tests, measured EN/ID model cards (`Fast` / `Balanced`), and `wizard_completed` / `wizard_version` persistence in `crates/app`.

## Open Questions for the Owner
- Confirm readiness to start **Phase 5B**.
- How would you like to handle the `TODO(owner)` / identifiable portrait fixtures in `tests/fixtures/PROVENANCE.md` before `v1.0`?

## Last Updated
2026-11-26
