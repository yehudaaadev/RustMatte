# Project State: RustMatte

## Current Phase
UI Polish Fixes Complete — ready for owner manual UI verification and confirmation on hardware provider default (`cpu` vs `auto`) before pushing/tagging.

## Last Session Summary
Executed UI Polish Fixes (`D-029`, `P-034`) on top of the pre-release hardening commits:
- **Comparison Slider Default**: Changed initial slider state to cutout-only (`0%`) with `"Compare with original"` toggle button (`C` shortcut), remembered split ratio per session, reset to cutout-only on each newly processed/selected photo, kept ARIA slider keyboard navigation intact, and added `crates/app/tests/slider.test.ts`.
- **Windows Path Display Prefix**: Added cross-platform `format_display_path` in Rust (`crates/app/src-tauri/src/validation.rs`) and TypeScript (`crates/app/src/lib/banners.ts`) stripping `\\?\` on drive paths and normalizing `\\?\UNC\` -> `\\` for display only while retaining canonical `PathBuf` internally.
- **Full Localization Audit**: Converted active banners (`job_done`, `job_partial`, `job_cancelled`, `saved_single`, `saved_all`), decode warnings (`damaged_input`), rejected input reasons (`inaccessible`, `read_dir_failed`, `not_file_or_dir`, `unsupported_format`), file dialog titles/filters, and About network description into typed message codes + complete EN and natural Indonesian translations (`crates/app/tests/i18n.test.ts`).
- **Save All Count**: Added `summary.saveable` in `crates/app/src/lib/queue.ts` so `Save all (N)` counts only completed/warning items with `fullCutoutPath` and disables when `0`.
- **Hardware Default Investigation (Unchanged Pending Owner Approval)**: Documented discrepancy between `D-017`/`D-022` (`cpu` default) and `crates/app/src-tauri/src/settings.rs` / `+page.svelte` (`"auto"` default), plus evidence that CI/release builds compile CPU-only (`default = []` without `directml` or `coreml` features).

Earlier Hardenings Retained Intact (`D-027`, `D-028`, `P-032`, `P-033`):
- `custom-protocol` compile-time release guard, `$APPCACHE/startup.log` offline readiness verification (`ui_ready mode=custom-protocol origin=http://tauri.localhost`, zero port `1420` connections), and `release.yml` binary check.
- Banded 128-row Guided Filter (`2,185.15 MB` Quality / `492.68 MB` Fast at 24 MP), `io.github.yehudaaadev.rustmatte` bundle identifier, comparison table verification, and removal of stale `tests/fixtures/SHA256SUMS`.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + Desktop `custom-protocol` Guard + UI Polish Fixes complete locally.

## In Progress
- Awaiting owner decision on desktop default execution provider (`"cpu"` vs `"auto"`), owner manual UI verification checklist, `tests/fixtures/PROVENANCE.md` (`TODO(owner)` fixtures), `models-v1` asset upload, and confirmation to push/tag `v0.1.0-rc2`.

## Known Broken or Failing
- Desktop `settings.rs` and `+page.svelte` default `device` to `"auto"` instead of `"cpu"` (`D-017`/`D-022`); left unchanged intentionally pending owner confirmation.
- `https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/*.onnx` returns `404` (HuggingFace fallback works).
- Multi-OS `release.yml` cloud build (`v0.1.0-rc2`), macOS/Linux runtime tests, visual inspection of the updated desktop UI, and clean second-PC installer verification remain unverified locally.

## Next Step
- Owner verifies the updated UI with the manual checklist and confirms whether to change the desktop default execution provider from `"auto"` to `"cpu"` before pushing `main` and tagging `v0.1.0-rc2`.

## Open Questions for the Owner
- Should desktop `device` default change from `"auto"` to `"cpu"` (`D-017`/`D-022`), matching the CPU-only compiled release binaries and avoiding GPU driver/VRAM risks?
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md` before pushing?
- Ready to push local commits to `origin/main`?

## Last Updated
2026-11-25
