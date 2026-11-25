# Project State: RustMatte

## Current Phase
Pre-Release Stabilization Complete (`PRE-PUSH FINAL FIXES` committed locally on `main`, NOT pushed, NOT re-tagged) — ready for owner push and re-tag verification of `v0.1.0-rc2` plus manual owner checklist.

## Last Session Summary
Completed all remaining `PRE-PUSH FINAL FIXES` and verified the full workspace quality gate:
- **Task 1 (24-Pair Refine Parity & 128-Row Seam Check)**: Compared `207ef8d` vs `HEAD` release CLIs across `hewan1`, `orang1`, `baju1`, `rambut1`, `rambut2`, `large_24mp` (`quality`/`fast` x `light`/`standard`): `MaxA = 1`, `MeanA <= 0.0003`, `%>2 = 0.00%` across all 24 pairs (`MaxRGBA <= 2` on 23/24 pairs, `5` on `large_24mp` `quality/standard` affecting `0.00%` of pixels). On `large_24mp.jpg` (31 band seams at `128*k`), `HEAD` matched `207ef8d` identically (`SeamA = 0.0655` vs `AdjA = 0.0578` on Quality; `0.0874` vs `0.0733` on Fast). Generated `bench/out/refine-check/index.html` and removed temporary worktree `_rustmatte_207ef8d_wt`.
- **Desktop CPU Default & Read-Only Line (`D-030`, `D-031`)**: Migrated saved `"auto"` -> `"cpu"` on load in `crates/app/src-tauri/src/settings.rs` (+ test), hid `#device-select` in `crates/app/src/routes/+page.svelte`, and replaced it with localized read-only `"Processing: CPU"` / `"Perangkat proses: CPU"`.
- **Release `custom-protocol` Guard (`D-031`)**: Moved `compile_error!` from `lib.rs` to `crates/app/src-tauri/src/main.rs` so `cargo test --release -j 2 --workspace` passes (`45 passed, 2 ignored`), while `cargo build --release -p rustmatte-app` fails cleanly at `main.rs:5` (`EXIT=101`).
- **Model Source Primary (`D-031`)**: Made `huggingface.co` primary (`source_urls[0]`) in `crates/core/src/model.rs` (since `models-v1` release assets currently return `404`), keeping release assets as fallback mirror.
- **Release Workflow Fixes (`.github/workflows/release.yml`, `D-031`)**: Fixed macOS binary discovery (`*/Contents/MacOS/*` for `RustMatte.app/Contents/MacOS/RustMatte`) and switched Linux release builds from `ubuntu-22.04` to `ubuntu-24.04` (resolving `libort_sys` `__isoc23_strtoull` / GCC 13+ `libstdc++` undefined symbols), syncing `README.md`, `docs/RELEASING.md`, and `docs/ARCHITECTURE.md`.
- **Full Quality Gate & Offline Launch Verified**:
  - `release.yml` YAML valid, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace -j 2`, `cargo test --release --workspace -j 2`, `cargo deny check licenses`, `cargo deny check advisories` all passed.
  - `npm run license-check`, `npm run license-check:all`, `npm test` (6/6), `npm run check` (0 errors, 0 warnings), and `npx tauri build --bundles nsis` (`RustMatte_0.1.0_x64-setup.exe`, `9.69 MiB`) all passed.
  - Real offline launch of `target/release/rustmatte-app.exe` with `PORT_1420_LISTENERS=0` wrote `ui_ready mode=custom-protocol origin=http://tauri.localhost` in `1,606 ms`.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028` through `D-031`) verified locally on Windows.

## In Progress
- None (awaiting owner push to `origin/main` and re-tag `v0.1.0-rc2` to re-run `.github/workflows/release.yml` in cloud).

## Known Broken or Failing
- `https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/*.onnx` returns `404` (HuggingFace is now primary; release asset URLs remain secondary mirror).
- Cloud verification of `.github/workflows/release.yml` fixes (macOS `.app` binary lookup and Linux `ubuntu-24.04` build) is **UNVERIFIED in cloud** until owner pushes and re-tags `v0.1.0-rc2`.

## Next Step
- Owner pushes local `main` commits to `origin/main`, moves/re-pushes tag `v0.1.0-rc2` (when ready), verifies all 3 OS jobs + `Create Draft Prerelease` in GitHub Actions, and completes manual GUI/clean-machine/provenance checklist.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md` before pushing?
- Ready to push local commits to `origin/main` and re-tag `v0.1.0-rc2`?

## Last Updated
2026-11-25
