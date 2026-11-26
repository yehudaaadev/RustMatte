# Project State: RustMatte

## Current Phase
Phase 5.0 Complete (`PRD_V2.md` accepted, `D-032`..`D-034` verified and committed locally) — ready to begin **Phase 5A** (Model manifest, resumable downloader + disk check, RAM detection, recommendation engine, CLI `models` subcommands) upon owner confirmation.

## Last Session Summary
Adopted `PRD_V2.md` (`Status: accepted`) and completed **Phase 5.0** (`D-032` close-out and carry-over verification):
- **PRD v2 Adoption & Scratch Cleanup (`D-033`)**: Marked `PRD_V2.md` `accepted`, added the v2 changelog entry in `docs/PRD.md`, logged `D-033`/`D-034` in `docs/DECISIONS.md`, registered Phases `5.0`–`5D` + `Release Readiness` in `docs/TASKS.md`, and deleted temporary `ben2_fp32.onnx` (`422,457,868` bytes) from conversation scratch (`BEN2` deferred to `v1.1` per `PRD_V2.md` Section 16).
- **Refinement Audit & Fixes (`crates/core/src/refine.rs`, `D-034`)**:
  - Aligned `refine_mask_guided` foreground early-exit (`m < 242` -> `m < 243`) to match `p_val >= 0.95` (`243/255 = 0.9529 >= 0.95`).
  - Added multi-band seam and full-frame parity unit tests (`test_color_guided_filter_multi_band_no_seam`, `test_color_guided_filter_early_exit_threshold_matches_float_boundary`, `test_color_guided_filter_multi_band_matches_full_frame`, max diff `<= 1` LSB).
  - Audited `q <= 0.06` / `0.06..0.14` haze suppression across `rambut1`, `rambut2`, `hewan1`, and `hewan2`: `0.00%` loss of `raw > 36` pixels (`0–42` pixels across `0.43M–1.57M` px images), `79%–94%` reduction in `1..15` smear haze, and verified preservation of fine cat whiskers and flyaway hair tips via direct strip inspection.
- **24 MP Fresh-Process Re-Measurement & Comparison Regeneration (Windows 11 Pro, i5-10400 6C/12T, 16 GB RAM, release CLI)**:
  - **Balanced (`birefnet_lite_512`, `--refine standard`)**: `2,191.39 MB` Peak Working Set (`<= 2.5 GB` target met), `2,571.77 MB` Peak Commit, `11.698 s` wall (`10,638.65 ms` internal total; `4,577.40 ms` inference, `2,706.83 ms` postprocess).
  - **Fast (`u2netp`, `--refine standard`)**: `498.96 MB` Peak Working Set (`<= 1.0 GB` target met), `745.76 MB` Peak Commit, `4.306 s` wall (`4,223.73 ms` internal total; `1,100.37 ms` inference, `2,680.80 ms` postprocess).
  - Regenerated clean 12-fixture outputs into `C:\Users\Admin\Downloads\Quality Model` (`80.85 s`) and `C:\Users\Admin\Downloads\fast model` (`8.67 s`) with `12/12` distinct SHA-256 pairs verified.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028`..`D-031`) + `models-v1` release + `v0.1.0-rc4` multi-OS CI/Release verification + `D-032` refinement/UI tracking + `PRD_V2.md` adoption & **Phase 5.0** (`D-033`..`D-034`).

## In Progress
- None in code; awaiting owner confirmation to start **Phase 5A**.

## Known Broken or Failing
- None in automated checks.
- Known model limits remain documented: transparent glass (`kaca1`) segments as solid; `birefnet_lite_512` can close small interior gaps (`baju1` collar) and clip partial fingertips (`produk1`/`produk2`).
- Carry-over owner/asset items remain open for Release Readiness (`PRD_V2.md` §3 & §14): `TODO(owner)` / portrait fixtures in `tests/fixtures/PROVENANCE.md`, custom brand icons replacing Tauri/Svelte placeholders, `docs/screenshots/` + README GIF, manual desktop UI verification, clean second-PC Windows installer test, and GitHub settings/publish.

## Next Step
- Begin **Phase 5A**: implement manifest schema + validation (`crates/core`), compare >= 2 RAM-detection crates (`license-check` first), implement resumable model downloader with `.part` + metadata and 10% disk-space check, pure recommendation engine + unit tests, and `rustmatte models list|download|delete|verify` CLI support.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / identifiable portrait fixtures in `tests/fixtures/PROVENANCE.md` before `v1.0`?
- Confirm readiness to start **Phase 5A** (and note candidate RAM-detection crate selection will be presented for approval per the Ask-First rule before adding to `Cargo.toml`).

## Last Updated
2026-11-26
