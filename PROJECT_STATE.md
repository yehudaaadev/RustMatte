# RustMatte Project State

## Current Phase
Phase 5B Complete (`D-036`: mandatory 5-step bilingual setup wizard, measured model cards, resume/cancel/delete-partial/offline import UI, and `wizard_completed`/`wizard_version` persistence verified in automated checks and release launch) — ready to begin **Phase 5C** (Settings screen, model management, allow-listed GitHub Issues feedback link, and per-photo retry with another model) upon owner confirmation.

## Last Session Summary
Implemented and verified **Phase 5B** (`PRD_V2.md` §4, §7.2, §7.5–7.6, §10–12, `D-036`):
- **Backend Setup Commands & Settings (`crates/app/src-tauri/src/`)**:
  - Added `wizard_completed` (`false` default) and `wizard_version` (`0` -> `1`) to `AppSettings` (`settings.rs`); wizard answers and detected RAM are never persisted.
  - Added `DownloadCoordinator` (`state.rs`) and `get_manifest_models`, `recommend_setup_model`, `cancel_model_download`, and `delete_partial_model_download` commands (`lib.rs`), upgraded `download_model` to use `download_model_with_options` (disk check + HTTP Range resume + cancel keeping `.part`), and delegated `system_ram_mb()` to `rustmatte_core::detect_memory_info()`.
- **Wizard State Machine & Bilingual UI (`crates/app/src/lib/`, `crates/app/tests/`)**:
  - Built pure `wizard.ts` state machine + Node unit tests (`wizard.test.ts`) enforcing mandatory questions, Back/reset behavior, and verified-model completion gate.
  - Added EN and natural Indonesian translations (`i18n.ts`, verified in `i18n.test.ts`) covering §7.5 copy, §7.6 measured model cards, recommendation reason codes, RAM advisories, disk-space errors, and copyable offline URL/SHA-256 import guidance.
  - Built `SetupWizard.svelte` and replaced the old `firstRunModal` in `+page.svelte` (auto-opening when `!wizard_completed`, with no skip button).
- **Verification**: Full frontend + Rust quality gate (`9/9` npm tests, `svelte-check` `0` errors/`0` warnings, `vite build`, `cargo fmt`, `cargo deny`, `cargo clippy`, `63` Rust tests passed / `2` ignored, release workspace build) and release `rustmatte-app.exe` `startup.log` check passed.

## Done
- Phase 0 through Phase 4 + Pre-Release Fixes + UI Polish + Pre-Push Final Fixes (`D-028`..`D-031`) + `models-v1` release + `v0.1.0-rc4` multi-OS CI/Release verification + `D-032` refinement/UI tracking + `PRD_V2.md` adoption & **Phase 5.0** (`D-033`..`D-034`) + **Phase 5A** (`D-035`) + **Phase 5B** (`D-036`).

## In Progress
- None in code; awaiting owner confirmation to start **Phase 5C**.

## Known Broken or Failing
- None in automated checks.
- Interactive desktop GUI verification of the 5-step wizard (light/dark, EN/ID, keyboard-only, narrow window, offline/interrupted download) remains pending manual owner walkthrough (`PRD_V2.md` §11 & §12).
- Carry-over owner/asset items remain open for Release Readiness (`PRD_V2.md` §3 & §14): `TODO(owner)` / portrait fixtures in `tests/fixtures/PROVENANCE.md`, custom brand icons replacing Tauri/Svelte placeholders, `docs/screenshots/` + README GIF, clean second-PC Windows installer test, and GitHub settings/publish.

## Next Step
- Begin **Phase 5C**: implement the dedicated Settings screen (language, theme, model list/delete/verify/download + re-run wizard, network lock, output defaults, collapsed Advanced, copy diagnostic info), allow-listed GitHub Issues feedback link (`PRD_V2.md` §7.8), and per-photo `Retry with another model` side-by-side comparison (`PRD_V2.md` §7.9).

## Open Questions for the Owner
- Confirm readiness to start **Phase 5C** (and note feedback URL default: `https://github.com/yehudaaadev/RustMatte/issues/new/choose` via OS default handler without shell/opener plugins per `PRD_V2.md` §7.8).
- How would you like to handle the `TODO(owner)` / identifiable portrait fixtures in `tests/fixtures/PROVENANCE.md` before `v1.0`?

## Last Updated
2026-11-26
