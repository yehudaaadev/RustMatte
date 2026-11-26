# RustMatte Project State

## Current Phase
Phase 5C Complete (`D-037`: dedicated Settings modal, model verify/delete/download management, allow-listed GitHub Issues feedback launcher, and session-scoped per-photo Retry with another model verified in automated checks and release launch) — ready to begin **Phase 5D** (Visual polish: `PRD_V2.md` §8 palette/WCAG AA contrast/tokens/spacing, collapsed Advanced on main screen, friendly empty/error states) upon owner confirmation.

## Last Session Summary
Implemented and verified **Phase 5C** (`PRD_V2.md` §7.7–7.9, §10–12, `D-037`):
- **Backend Settings, Model Management, Feedback URL & Retry (`crates/app/src-tauri/src/`)**:
  - Added `SessionCache::evict` (`state.rs`), `size_on_disk_bytes` in `get_manifest_models`, `verify_cached_model`, `delete_cached_model_cmd` (cancelling active download and evicting loaded session before deleting `.onnx`), `open_feedback_url` + `validate_feedback_url` allow-listed to `https://github.com/yehudaaadev/RustMatte/issues/new/choose` (via OS default handler, zero new plugins/capabilities), and `retry_single_with_model` (`lib.rs`).
- **Frontend Settings Modal & Model Retry Comparison (`crates/app/src/lib/`, `crates/app/tests/`)**:
  - Added `SettingsModal.svelte` covering language, theme, active model, re-run setup wizard, per-model size on disk / verify again / confirmed delete / download-import, `"Never use the network"`, output defaults, collapsed Advanced (`refine` + read-only `CPU`), Send feedback + Copy diagnostic info + About & Licenses.
  - Added `resultsByTier`, `applySingleRetryResult`, and `selectItemModelResult` (`queue.ts`, unit tested in `queue.test.ts`), plus `Retry with <model>`, side-by-side `Fast` vs `Balanced` comparison view, active-model result toggle for `Save`, and gentle Balanced suggestion banner after `Fast` runs (`+page.svelte`, EN/ID in `i18n.ts`).
- **Verification**: Full frontend + Rust quality gate (`9/9` npm tests, `svelte-check` `0` errors/`0` warnings, `vite build`, `cargo fmt`, `cargo deny`, `cargo clippy`, `64` Rust tests passed / `2` ignored, release workspace build) and release `rustmatte-app.exe` `startup.log` check passed.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028`..`D-031`) + `models-v1` release + `v0.1.0-rc4` multi-OS CI/Release verification + `D-032` refinement/UI tracking + `PRD_V2.md` adoption & **Phase 5.0** (`D-033`..`D-034`) + **Phase 5A** (`D-035`) + **Phase 5B** (`D-036`) + **Phase 5C** (`D-037`).

## In Progress
- None in code; awaiting owner confirmation to start **Phase 5D**.

## Known Broken or Failing
- None in automated checks.
- Manual owner verification of Settings actions, feedback browser launch, retry side-by-side flow, and wizard walkthrough remains pending (`PRD_V2.md` §11 & §12).
- Carry-over owner/asset items remain open for Release Readiness (`PRD_V2.md` §3 & §14): `TODO(owner)` / portrait fixtures in `tests/fixtures/PROVENANCE.md`, custom brand icons replacing Tauri/Svelte placeholders, `docs/screenshots/` + README GIF, clean second-PC Windows installer test, and GitHub settings/publish.

## Next Step
- Begin **Phase 5D**: unify light/dark palette tokens from `PRD_V2.md` §8 (verified for WCAG AA 4.5:1 text contrast), spacing/corner-radius/40px-button tokens, collapsed Advanced options on the main screen (§7.10), friendly empty state with inline SVG + one sentence, dismissible banners with relevant actions, and consistent dialog action layout.

## Open Questions for the Owner
- Confirm readiness to start **Phase 5D**.
- How would you like to handle the `TODO(owner)` / identifiable portrait fixtures in `tests/fixtures/PROVENANCE.md` before `v1.0`?

## Last Updated
2026-11-26
