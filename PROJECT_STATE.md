# Project State: RustMatte

## Current Phase
Diagnostic & Pre-Release Hardening Complete — release `custom-protocol` compile-time guard, `startup.log` UI readiness verification, CI binary check, and clean offline NSIS/exe verification completed locally on `main` (not pushed, not tagged).

## Last Session Summary
Executed Pre-Release Fixes + Desktop Dev-URL Diagnostic & Fix (`D-027`, `D-028`, `P-032`, `P-033`):
- **Tauri `devUrl` / `ERR_CONNECTION_REFUSED` Root Cause & Fix (`D-028`, `P-033`)**:
  - Diagnosed against pinned `tauri = "=2.12.7"`: `tauri::is_dev()` evaluates `!cfg!(feature = "custom-protocol")`, so plain `cargo build --release -p rustmatte-app` compiled `http://localhost:1420` into the release binary while skipping `beforeBuildCommand`. Reproduced `PROBE_1420_HITS = [('GET', '/', 'localhost:1420'), ...]` even while Windows reported `HasExited=False Responding=True`.
  - Kept `default = []` + added optional `custom-protocol = ["tauri/custom-protocol"]` in `crates/app/src-tauri/Cargo.toml` (so `npm run tauri dev` retains HMR), added compile-time guard `#[cfg(all(not(debug_assertions), dev, not(test)))] compile_error!(...)`, added `RUSTMATTE_BUILD_MODE` marker checked on all 3 OS in `.github/workflows/release.yml`, and added privacy-safe `notify_frontend_ready` writing `$APPCACHE/startup.log`.
  - Re-measured release offline startup (`port 1420` closed, `Port1420Hits=0`):
    - `npx tauri build` `rustmatte-app.exe`: `2,770 ms` UI ready, `ui_ready mode=custom-protocol origin=http://tauri.localhost`, `28.28 MB` host + `322.06 MB` (6 WebView2 processes) = `350.34 MB` idle RSS.
    - Installed NSIS `RustMatte.exe`: `2,209 ms` UI ready, `ui_ready mode=custom-protocol origin=http://tauri.localhost`, `42.02 MB` host + `320.98 MB` (6 WebView2 processes) = `363.00 MB` idle RSS.
    - Earlier Phase 3A/3C idle RAM (`117.23 MB` / `94.77 MB`) and `204 ms` launch metrics marked **UNRELIABLE**.
    - Confirmed `npm run tauri dev` boots to `ui_ready mode=dev-url origin=http://localhost:1420`.
- **Removed leftover `tests/fixtures/SHA256SUMS`** (`git rm`, owner-approved).
- **24 MP Memory & Speed Regression Fix (`crates/core/src/refine.rs`, `D-027`, `P-032`)**:
  - Replaced full-frame allocations with 128-row banded `guided_filter` (`4 * radius` halos, reused `BandBuffers`) and direct `DynamicImage` slice reads.
  - Measured results (release CLI, fresh process per run, `large_24mp.jpg`):
    - Quality `--refine standard`: `3,352.68 MB` -> **`2,185.15 MB`** (`<= 2.5 GB` target), postprocess `3,590.40 ms` -> `1,748.98 ms`.
    - Fast `--refine standard`: `1,660.08 MB` -> **`492.68 MB`** (`<= 1.0 GB` target), postprocess `3,603.99 ms` -> `1,579.88 ms`.
  - Added `--refine <none|light|standard>` CLI flag (default `standard` per `D-021`).
- **Identifier, Accessibility, Docs & Audits**:
  - Updated Tauri identifier `com.rustmatte.app` -> `io.github.yehudaaadev.rustmatte` across `tauri.conf.json`, `README.md`, `docs/ARCHITECTURE.md` (`dirs::data_local_dir()` model cache unaffected).
  - Fixed `.slider-viewport` accessibility warning (`svelte-check`: `0` errors, `0` warnings).
  - Audited `README.md` comparison table against upstream sources (`rembg` MIT, `@imgly/background-removal` AGPL-3.0, `remove.bg` proprietary credit SaaS; checked `2026-11-25`).

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + Desktop `custom-protocol` / UI Verification Hardenings complete locally.

## In Progress
- Awaiting owner action on `tests/fixtures/PROVENANCE.md` (`TODO(owner)` fixtures), `models-v1` asset upload, pushing to `origin/main`, and `v0.1.0-rc2` draft release review.

## Known Broken or Failing
- `https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/*.onnx` returns `404` (HuggingFace fallback works).
- Multi-OS `release.yml` cloud build (`v0.1.0-rc2`), macOS/Linux runtime tests, visual inspection of the desktop UI, and clean second-PC installer verification remain unverified locally.

## Next Step
- Owner reviews `tests/fixtures/PROVENANCE.md` (`TODO(owner)` / person fixtures) and visual UI rendering, then confirms pushing `main` and tagging `v0.1.0-rc2`.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md` before pushing?
- Ready to push local commits to `origin/main`?

## Last Updated
2026-11-25
