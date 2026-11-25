# Project State: RustMatte

## Current Phase
Pre-Release Complete (`PRE-PUSH FINAL FIXES` verified, pushed to `origin/main`, and tagged `v0.1.0-rc3`) — pending GitHub Actions verification & clean second-PC manual check.

## Last Session Summary
Completed all remaining `PRE-PUSH FINAL FIXES`, visual inspection, `models-v1` release publication, GitHub noreply commit authorship alignment, and full workspace quality gate:
- **Task 1 (24-Pair Parity, 128-Row Seam Check, & 100%-Zoom Visual Inspection)**: Compared `207ef8d` vs `HEAD` release CLIs across `hewan1`, `orang1`, `baju1`, `rambut1`, `rambut2`, `large_24mp` (`quality`/`fast` x `light`/`standard`): `MaxA = 1`, `MeanA <= 0.0003`, `%>2 = 0.00%` across all 24 pairs. Visually inspected 100%-zoom crops (`y=1280`, `y=1408`, `y=3968` on `large_24mp.jpg`) and saturated green (`#00c840`) / blue (`#145adc`) composites (`rambut1`, `hewan1`, `baju1`, `orang1`): zero horizontal 128-row band seams and zero green fringe halos on Quality tier. Generated `bench/out/refine-check/index.html` and removed temporary worktree `_rustmatte_207ef8d_wt`.
- **`models-v1` Release Published (`D-019`, `D-031`)**: Verified SHA-256 of cached `birefnet_lite_512.onnx` and `u2netp.onnx`, published GitHub Release `models-v1` (`--latest=false`), confirmed public unauthenticated `HTTP/1.1 200 OK` (`191877254` and `4574861` bytes), and restored `models-v1` as `source_urls[0]` with `huggingface.co` as fallback.
- **Desktop CPU Default & Read-Only Line (`D-030`, `D-031`)**: Migrated saved `"auto"` -> `"cpu"` on load in `crates/app/src-tauri/src/settings.rs`, hid `#device-select` in `+page.svelte`, and replaced it with localized read-only `"Processing: CPU"` / `"Perangkat proses: CPU"`.
- **Release `custom-protocol` Guard (`D-031`)**: Moved `compile_error!` to `crates/app/src-tauri/src/main.rs` so `cargo test --release -j 2 --workspace` passes (`45 passed, 2 ignored`), while `cargo build --release -p rustmatte-app` fails cleanly at `main.rs:5` (`EXIT=101`).
- **Release Workflow & README Fixes (`D-031`)**: Fixed `.github/workflows/release.yml` macOS binary lookup (`*/Contents/MacOS/*` for `RustMatte`) and Linux runner (`ubuntu-24.04`), synced `README.md`, `docs/RELEASING.md`, and `docs/ARCHITECTURE.md`, and removed unverified third-party numeric claims from `README.md`.
- **Git Identity & Push**: Configured local repo author `yehudaaadev <360074064+yehudaaadev@users.noreply.github.com>`, applied to unpushed local commits without rewriting older pushed commits, pushed `main` and tag `v0.1.0-rc3`.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028` through `D-031`) + `models-v1` asset release verified.

## In Progress
- GitHub Actions CI (`ci.yml`) and Release (`release.yml` on `v0.1.0-rc3`) verification.

## Known Broken or Failing
- Earlier `v0.1.0-rc2` release run (`45556804678`) failed on macOS (`otool -L` filename) and Linux (`ubuntu-22.04` glibc/libstdc++); fixed in `v0.1.0-rc3` (`release.yml`).

## Next Step
- Inspect GitHub Actions `ci.yml` (5 jobs) and `release.yml` (`v0.1.0-rc3` `otool -L` / `ldd` logs & draft release artifacts), then test `RustMatte_0.1.0_x64-setup.exe` on a clean second Windows PC.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md`?

## Last Updated
2026-11-25
