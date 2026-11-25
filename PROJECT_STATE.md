# Project State: RustMatte

## Current Phase
UI Polish & CPU Default Alignment Complete — ready for owner manual UI verification, fixture provenance confirmation, and confirmation to push/tag `v0.1.0-rc2`.

## Last Session Summary
Executed UI Polish Fixes + Desktop Hardware Default Alignment (`D-029`, `D-030`, `P-034`) on top of pre-release hardening commits:
- **Hardware Default Alignment (`D-030`)**: Changed desktop default execution provider from `"auto"` to `"cpu"` in `crates/app/src-tauri/src/settings.rs` and `crates/app/src/routes/+page.svelte` (`cpu` option listed first), matching `D-017`/`D-022` and CPU-only compiled release binaries.
- **Comparison Slider Default**: Changed initial slider state to cutout-only (`0%`) with `"Compare with original"` toggle button (`C` shortcut), remembered split ratio per session, reset to cutout-only on each newly processed/selected photo, kept ARIA slider keyboard navigation intact, and added `crates/app/tests/slider.test.ts`.
- **Windows Path Display Prefix**: Added cross-platform `format_display_path` in Rust (`crates/app/src-tauri/src/validation.rs`) and TypeScript (`crates/app/src/lib/banners.ts`) stripping `\\?\` on drive paths and normalizing `\\?\UNC\` -> `\\` for display only while retaining canonical `PathBuf` internally.
- **Full Localization Audit**: Converted active banners (`job_done`, `job_partial`, `job_cancelled`, `saved_single`, `saved_all`), decode warnings (`damaged_input`), rejected input reasons (`inaccessible`, `read_dir_failed`, `not_file_or_dir`, `unsupported_format`), file dialog titles/filters, and About network description into typed message codes + complete EN and natural Indonesian translations (`crates/app/tests/i18n.test.ts`).
- **Save All Count**: Added `summary.saveable` in `crates/app/src/lib/queue.ts` so `Save all (N)` counts only completed/warning items with `fullCutoutPath` and disables when `0`.

Earlier Hardenings Retained Intact (`D-027`, `D-028`, `P-032`, `P-033`):
- `custom-protocol` compile-time release guard, `$APPCACHE/startup.log` offline readiness verification (`ui_ready mode=custom-protocol origin=http://tauri.localhost`, zero port `1420` connections), and `release.yml` binary check.
- Banded 128-row Guided Filter (`2,185.15 MB` Quality / `492.68 MB` Fast at 24 MP), `io.github.yehudaaadev.rustmatte` bundle identifier, comparison table verification, and removal of stale `tests/fixtures/SHA256SUMS`.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + Desktop `custom-protocol` Guard + UI Polish & CPU Default Alignment complete locally.

## In Progress
- Awaiting owner manual UI verification checklist, `tests/fixtures/PROVENANCE.md` (`TODO(owner)` fixtures) review, `models-v1` asset upload, and confirmation to push `main` / tag `v0.1.0-rc2`.

## Known Broken or Failing
- `https://github.com/yehudaaadev/RustMatte/releases/download/models-v1/*.onnx` returns `404` (HuggingFace fallback works).
- Multi-OS `release.yml` cloud build (`v0.1.0-rc2`), macOS/Linux runtime tests, visual inspection of the updated desktop UI, and clean second-PC installer verification remain unverified locally.

## Next Step
- Owner verifies the updated UI with the manual checklist, reviews `tests/fixtures/PROVENANCE.md`, and confirms pushing `main` and tagging `v0.1.0-rc2`.

## Open Questions for the Owner
- How would you like to handle the `TODO(owner)` / person fixtures in `tests/fixtures/PROVENANCE.md` before pushing?
- Ready to push local commits to `origin/main` and create tag `v0.1.0-rc2`?

## Last Updated
2026-11-25
