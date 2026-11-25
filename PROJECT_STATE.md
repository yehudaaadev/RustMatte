# Project State: RustMatte

## Current Phase
Pre-Release Verified in Cloud (`CI` 5/5 green on `main`, `Release` `v0.1.0-rc4` 4/4 green with all 8 draft prerelease assets + `models-v1` public) — awaiting owner second-PC clean install test, GUI screenshots, and `PROVENANCE.md` review.

## Last Session Summary
Completed all pre-release fixes, visual verification, model asset publication, GitHub noreply commit alignment, and full local + cloud verification:
- **Task 1 (24-Pair Parity, 128-Row Seam Check, & 100%-Zoom Visual Inspection)**: Compared `207ef8d` vs `HEAD` release CLIs across `hewan1`, `orang1`, `baju1`, `rambut1`, `rambut2`, `large_24mp` (`quality`/`fast` x `light`/`standard`): `MaxA = 1`, `MeanA <= 0.0003`, `%>2 = 0.00%` across all 24 pairs. Visually inspected 100%-zoom crops (`y=1280`, `y=1408`, `y=3968` on `large_24mp.jpg`) and saturated green (`#00c840`) / blue (`#145adc`) composites (`rambut1`, `hewan1`, `baju1`, `orang1`): zero horizontal 128-row band seams and zero green fringe halos on Quality tier. Generated `bench/out/refine-check/index.html` and removed temporary worktree `_rustmatte_207ef8d_wt`.
- **`models-v1` Release Published (`D-019`, `D-031`)**: Verified SHA-256 of cached `birefnet_lite_512.onnx` and `u2netp.onnx`, published GitHub Release `models-v1` (`--latest=false`), confirmed public unauthenticated `HTTP/1.1 200 OK` (`191877254` and `4574861` bytes), and restored `models-v1` as `source_urls[0]` with `huggingface.co` as fallback.
- **Desktop CPU Default, Release Guard, & README (`D-030`, `D-031`)**: Migrated saved `"auto"` -> `"cpu"` on load, replaced `#device-select` with localized read-only `"Processing: CPU"` / `"Perangkat proses: CPU"`, moved `compile_error!` guard to `crates/app/src-tauri/src/main.rs` (`cargo test --release --workspace` passes `45 passed, 2 ignored`), and removed unverified third-party numbers from `README.md`.
- **Cloud CI & Release Verification (`v0.1.0-rc4`)**:
  - `CI` run `45570151591` passed all 5 jobs (`ubuntu-latest`, `windows-latest`, `macos-latest`, `License and Security Audit`, `Frontend Check, Build, and License Audit`).
  - `Release` run `45570157479` (`v0.1.0-rc4`) passed all 4 jobs (`x86_64-unknown-linux-gnu` on `ubuntu-24.04`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `Create Draft Prerelease`), producing 7 binary/installer packages + `SHA256SUMS.txt`.
  - Inspected `otool -L` (macOS) and `ldd` (Linux) CI logs: ONNX Runtime (`ort 2.0.0-rc.15`) links statically on all 3 OSes (`libonnxruntime` absent from dynamic deps; only system Apple frameworks / GTK+WebKitGTK libs linked).

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028` through `D-031`) + `models-v1` release + `v0.1.0-rc4` multi-OS CI/Release draft verification.

## In Progress
- Owner final manual checklist (second clean Windows PC offline test, `docs/screenshots/`, `tests/fixtures/PROVENANCE.md` review, and publishing `v0.1.0-rc4` draft release).

## Known Broken or Failing
- None in CI/Release (`v0.1.0-rc4` draft prerelease and `models-v1` are live; earlier failed runs `v0.1.0-rc2` and `v0.1.0-rc3` superseded by `v0.1.0-rc4`).

## Next Step
- Owner downloads `RustMatte_0.1.0_x64-setup.exe` + `SHA256SUMS.txt` from the `v0.1.0-rc4` draft release, verifies checksum, tests on a clean second Windows PC (online first-run `models-v1` download + offline processing), captures `docs/screenshots/`, reviews `tests/fixtures/PROVENANCE.md`, and publishes the release.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md`?

## Last Updated
2026-11-25
