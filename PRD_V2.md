# RustMatte PRD v2: Guided Setup, Model Profiles, UI Polish, and Release

Status: accepted (2026-11-26). Continues docs/PRD.md (v1). Written in English for the coding agent. All in-app strings are bilingual (Indonesian and English).

## 1. Purpose and relationship to PRD v1

This document continues docs/PRD.md. Where the two conflict, this document wins for: model strategy and download UX, the first-run experience, settings, UI design, and release scope. Everything else in PRD v1 and AGENTS.md still applies: one phase per session, phase gates, the honesty rule, the license-check and verify-crate-api skills, release-mode measurements, and the ask-first list.

The agent must record the changes below in docs/DECISIONS.md (new decision IDs) and add a changelog entry to docs/PRD.md that points to this document.

## 2. Current state (verified work to date)

- Phases 0 to 4 are complete: Cargo workspace, core engine, CLI with batch mode, edge refinement, Tauri 2 + Svelte desktop app, NSIS installer, CI on Windows, macOS and Linux, release workflow, community files.
- Models in use: **Balanced** = BiRefNet Lite 512 (MIT weights, about 183 MB) and **Fast** = U-2-Netp (Apache-2.0 weights, about 4.4 MB). Rejected earlier: RMBG-2.0 (CC BY-NC), ISNet (dataset terms), BiRefNet 1024 (memory), fp16 and int8 (slower on CPU).
- Measured on i5-10400 (6 cores, 12 threads), 16 GB RAM, Windows 11, release builds: Balanced about 3.0 s per photo and about 2.0 GB peak; Fast about 0.3 s per photo and about 0.3 to 0.6 GB peak; 24 MP photo with refine standard: Balanced about 2.2 GB and Fast about 0.5 GB after the banded filter (D-027). All of these must be re-measured after the D-032 refine rewrite.
- Security posture: compile-time guard against release builds that load the dev URL, strict CSP, minimal Tauri capabilities, no telemetry, network used only for the consented model download.
- Known limits (must stay documented): glass and other transparent objects are segmented as solid; Fast can leave smears on fur and textured backgrounds; Balanced sometimes closes small gaps (for example a collar) and can clip fingertips on some photos.

## 3. Carry-over that must be closed before v1.0

- **D-032 close-out:** commit the pending refine rewrite; re-measure memory and time at 24 MP; check band seams; review the haze-suppression rule (q at or below 0.06 becomes 0) for lost hair and whisker detail; regenerate clean comparison sets from final code.
- **Fixture provenance:** resolve every TODO(owner) in tests/fixtures/PROVENANCE.md; replace or remove portraits of identifiable people without clear rights.
- **Brand assets:** replace placeholder Tauri and Svelte icons.
- **Docs and screenshots:** capture screenshots and a short GIF for the README.
- **Verification gates owned by the owner:** manual desktop review, installer test on a clean second Windows PC, GitHub repository settings, and publishing the draft release.
- **Documentation honesty:** the Linux build needs glibc 2.39 or newer (prebuilt ONNX Runtime); macOS builds are Apple Silicon only and unsigned; Windows installer is unsigned (SmartScreen warning).

## 4. Owner decisions (binding for this PRD)

1. The first-run wizard is **mandatory**: no skip button. It has Back and a step indicator.
2. Wizard order: language, short briefing, two questions, recommendation, download consent, ready.
3. Profiles in v1.0: **Cepat / Fast** and **Seimbang / Balanced**. Profile names are final.
4. A per-photo button, **Retry with another model**, shows old and new results side by side.
5. Automatic RAM detection through one small dependency; resumable downloads; disk-space check before download; model management in Settings.
6. Language and theme can be changed at any time in Settings; Settings can also re-run the wizard.
7. Feedback link opens the GitHub Issues page (the channel most people expect for open source). Details in section 7.8.
8. BEN2 (“Maximum”) is **not** in v1.0. It is a v1.1 candidate with conditions (section 16).
9. UI is modern, calm, and uncluttered, with an easy-to-find primary action on every screen.
10. Privacy: wizard answers are never sent anywhere and are not saved. Only language, theme, chosen model, and a wizard-completed flag are stored.
11. The RAM thresholds in section 7.4 and the palette in section 8 are set by this document and are validated by measurement and by owner review of screenshots.

## 5. Product principles

- **Offline and private by default.** The only network use is the one-time, consented model download.
- **Honest.** Every number in docs and UI is measured and labeled with the hardware it was measured on. Known weaknesses are shown to users.
- **Simple.** A first-time user always knows the next click. Technical names live under a Details section.
- **Safe to extend.** Adding a model is a data change plus a checklist, not a rewrite.

## 6. Scope

**In scope for v1.0:** everything in section 7; the carry-over in section 3; Windows as the fully tested platform; macOS (Apple Silicon) and Linux (x86\_64, glibc 2.39+) as experimental builds.

**Out of scope for v1.0 (backlog):** BEN2 or any third model; importing arbitrary ONNX files; GPU execution providers; HEIC and RAW (v1.1); crop to subject, WebP and JPEG export, manual refine brush, presets (v1.x); upscaler and browser version (v2); mobile (v3); telemetry; auto-update.

## 7. Functional requirements

### 7.1 Model manifest

Each model is one data entry. Fields: id, engine, file name, size in bytes, SHA-256, ordered download sources, weight license, dataset-provenance note key, status (stable, experimental, hidden), input spec (size, mean, std, input and output tensor names, activation), measured data (hardware label, seconds per photo, peak RAM), profile tags (strength codes, weakness codes, recommended-for codes), and minimum available RAM. Card text comes from the language dictionaries through the codes, never from strings embedded in logic.

```toml
[[model]]
id = "balanced"
engine = "birefnet_lite_512"
file_name = "birefnet_lite_512.onnx"
size_bytes = 191877254
sha256 = "<pinned>"
sources = ["github-release", "upstream-mirror"]
license_weights = "MIT"
data_note = "academic-datasets"
status = "stable"

[model.input]
size = 512
mean = [0.485, 0.456, 0.406]
std = [0.229, 0.224, 0.225]
activation = "sigmoid"

[model.measured]
hardware = "i5-10400, 6C/12T, 16 GB"
seconds_per_photo = 3.1
peak_ram_mb = 2030

[model.profile]
strengths = ["hair_fur", "general"]
weaknesses = ["closes_small_gaps", "larger_download"]
recommended_for = ["occasional", "hair_fur", "mixed"]
```

**Checklist required for every model (also for contributors):** license of weights and training-data provenance from primary sources; hash and size; verified origin of the ONNX export (prefer exporting from official weights ourselves); one-process-per-model benchmark of time and memory; visual results on the fixture set reviewed by the owner; bilingual card text with measured claims only. Only manifest models with verified hashes may be loaded. Arbitrary ONNX files are not supported, because a model file is executed as code.

### 7.2 Model manager and download

- **Consent first:** show file name, size, source host, and weight license before any network request. The “Never use the network” setting blocks all downloads; import stays available.
- **Sources:** GitHub Release first, upstream mirror as fallback, both checked against the same pinned SHA-256. HTTPS only (plain HTTP only for localhost in tests).
- **Disk check:** required space is model size plus 10 percent. If free space is lower, show required and available amounts and block the download button.
- **Resume:** keep a .part file plus a small metadata file (URL, expected size, expected hash). Resume with HTTP Range only when the server answers 206 and the metadata matches; otherwise restart cleanly. Always verify SHA-256 of the **complete** file before the atomic rename. Retry automatically with increasing delays, then show a Retry button. Cancel keeps the partial download, with a separate Delete partial download option.
- **Import:** choose a file the user already has; verify SHA-256 against the manifest. Wrong hash gives a clear error and the file is not used.
- **Offline first run:** the wizard explains the import route (download on another computer, then import) and shows the link and hash to copy.
- **Settings > Models:** list downloaded models with size on disk, active model, Delete (confirm, states the space freed), Verify again, and Download another model.
- **Errors (friendly sentence, suggestion, technical details):** no internet, server error, checksum mismatch, truncated download, disk full, unwritable cache folder.

### 7.3 RAM detection

Use one small dependency that reads **available** memory (not only total). Requirements: license compatible with GPL-3.0-or-later (cargo-deny), actively maintained, few transitive dependencies, works on Windows, macOS and Linux, no network access. The agent must compare at least two candidates and justify the choice. The value is used locally for the recommendation and warnings, never stored or sent. If detection fails, show a neutral note and no false warning.

### 7.4 Recommendation engine

A pure function (no side effects): inputs are usage answer, photo-type answer, available RAM (optional), and the set of already installed models. Outputs are primary model, alternative model, and reason codes translated by the UI.

| Usage | Photo type | Primary | Alternative |
| --- | --- | --- | --- |
| Occasionally, a few photos | any | Balanced | Fast |
| Fairly often, dozens at a time | People or portraits | Fast | Balanced |
| Fairly often, dozens at a time | Products on plain background | Fast | Balanced |
| Fairly often, dozens at a time | Animals, hair or fur | Balanced | Fast |
| Fairly often, dozens at a time | Mixed or complex | Balanced | Fast |
| Regular work, hundreds of photos | People or portraits | Fast | Balanced |
| Regular work, hundreds of photos | Products on plain background | Fast | Balanced |
| Regular work, hundreds of photos | Animals, hair or fur | Balanced | Fast |
| Regular work, hundreds of photos | Mixed or complex | Balanced | Fast |
| Not sure yet | any | Balanced | Fast |

**Overrides.** Available RAM below 3 GB: primary becomes Fast, with a warning if the user picks Balanced. Between 3 and 5 GB: keep the table result and add a note on memory use. 5 GB or more: no note. Unknown RAM: no warning. Model already installed: show Installed and skip its download. These thresholds are named constants. The agent re-validates them in Phase 5A by measuring Balanced at 24 MP with refine standard, and reports if the constants should change.

**Tests:** one test per table row, plus low RAM, mid RAM, unknown RAM, and already-installed cases. They run in CI without model files.

### 7.5 First-run wizard

Shown when the wizard-completed flag is absent. The flag survives reinstalls that keep settings. Steps: 1 of 5 (language), 2 of 5 (briefing and questions), 3 of 5 (recommendation), 4 of 5 (download), 5 of 5 (ready). Closing mid-way restarts the wizard from step 1 next time. The wizard completes only when one verified model is available. It is fully keyboard operable and uses the chosen language from step 1 on.

| Screen | English | Indonesian |
| --- | --- | --- |
| Language | Choose your language | Pilih bahasa |
| Briefing | Two quick questions before we start. RustMatte has several AI models, each with strengths and weaknesses. Your answers help us suggest the best fit, so you don’t download files you don’t need. Your answers are never sent anywhere and are not saved. | Sebelum mulai, dua pertanyaan singkat. RustMatte punya beberapa model AI dengan kelebihan dan kekurangan masing-masing. Jawabanmu membantu kami menyarankan yang paling cocok, supaya kamu tidak mengunduh file yang tidak perlu. Jawabanmu tidak dikirim ke mana pun dan tidak disimpan. |
| Question 1 | How often and how much will you use RustMatte? Occasionally, a few photos / Fairly often, dozens at a time / Regular work, hundreds of photos / Not sure yet | Seberapa sering dan banyak kamu akan memakai RustMatte? Sesekali, beberapa foto / Cukup sering, puluhan foto sekali proses / Untuk pekerjaan rutin, ratusan foto / Belum tahu |
| Question 2 | What kind of photos do you edit most? People or portraits / Products on plain backgrounds / Animals, hair, or fur / Mixed or complex photos | Foto apa yang paling sering kamu olah? Orang atau potret / Produk di latar polos / Hewan, rambut, atau bulu / Campuran atau foto rumit |
| Download | This is the only internet connection RustMatte makes. After it finishes, everything runs offline. | Ini satu-satunya hubungan internet yang dilakukan RustMatte. Setelah selesai, semua berjalan offline. |
| Ready | All set. Drop photos here, or click to choose. | Semua siap. Seret foto ke sini, atau klik untuk memilih. |

The recommendation screen shows one **Recommended for you** card with a one-sentence reason (from reason codes), one **Alternative** card, and a **See all models** link. Choosing a card moves to the download step. There are exactly two questions; adding a third requires owner approval.

### 7.6 Model cards (v1.0 content)

Each card shows: profile name, one-sentence summary, strengths, weaknesses, download size, estimated time per photo, RAM need, weight license, and a Details section (technical name, source, SHA-256, dataset note). Time estimates carry the label “measured on a mid-range PC (i5-10400, 6 cores)”, and after the first photo the app shows the real time on the user’s machine.

- **Fast (U-2-Netp):** very fast and light; best for many photos with simple backgrounds. Strengths: about 0.3 to 0.5 s per photo, 4.4 MB download, low RAM (about 0.3 to 0.6 GB). Weaknesses: may leave faint smears around fur and textured backgrounds; less tidy on fine hair. License: Apache-2.0.
- **Balanced (BiRefNet Lite 512):** cleaner hair and fur at a reasonable speed. Strengths: tidier edges on hair and fur; about 3 s per photo. Weaknesses: about 183 MB download; about 2 GB RAM; in our tests it sometimes closed small gaps (for example a collar area) and clipped fingertips on some photos. License: MIT.
- **All models:** transparent objects such as glass are cut out as solid objects. The weights were trained on academic datasets with non-commercial terms; whether those terms extend to model weights is legally unsettled (details in MODELS.md; this is not legal advice).

Only claims backed by a measurement or a cited source may appear on a card.

### 7.7 Settings

A Settings screen reachable from the main screen at all times:

- **Language:** Indonesian or English; applies instantly everywhere.
- **Theme:** Follow system, Light, Dark.
- **Models:** section 7.2 list; active model; re-run setup wizard.
- **Network:** “Never use the network” switch with a plain explanation.
- **Output defaults:** output folder, background (transparent or solid color), maximum output size (advanced).
- **Advanced (collapsed by default):** edge refinement level (none, light, standard), processing device shown read-only as CPU in this release.
- **Feedback and About:** Send feedback (section 7.8), About and Licenses, Copy diagnostic info (app version, OS, CPU cores, RAM, active model; no file names or paths).

### 7.8 Feedback channel

Settings shows **Send feedback**, which opens the repository’s GitHub Issues page (the new-issue chooser with the bug and feature templates) in the system browser. Implementation rules: a single hard-coded, allow-listed URL; opened through the operating system’s default handler without adding a shell or opener plugin; no request made by the app itself; the Copy diagnostic info button sits next to it so users can paste details into the issue. The issue templates already ask for diagnostic information. GitHub Discussions may be added later; users who do not have a GitHub account are told they can also try the project page on the web.

### 7.9 Retry with another model

On the result screen, a secondary button next to the comparison control offers **Retry with another model**. It lists installed models; a model that is not installed offers the download flow with consent. The new result for the same photo is shown beside the old one, with a toggle between them and the existing original comparison. Results per model are kept only for the current session, and the preview cache keeps its size cap. When the user ran Fast on a photo with fur, a gentle suggestion offers Balanced.

### 7.10 Main screen polish (no new features)

- Advanced options are grouped under a collapsible Advanced section; the visible controls are the active model (with a link to change it), the primary action, and Save.
- A friendly empty state: simple inline SVG icon plus one sentence.
- The comparison slider defaults to showing the full cutout, with a Compare with original toggle (already implemented; keep it).
- Banners are short, dismissible, and carry a relevant action (Open folder, Save).
- All text comes from the dictionaries; Rust sends message codes plus data.

## 8. UI and UX design system

**Principles:** one primary action per screen; consistent button positions (Continue bottom right, Back bottom left, Cancel as a text button near Back); calm palette; status never conveyed by color alone (icon plus text); system fonts only (no external fonts or CDNs); card-based layout with light separators; no heavy borders.

**Tokens.** Spacing 4, 8, 12, 16, 24, 32 px. Corner radius 10 to 12 px. Minimum button height 40 px. Body text 14 to 15 px, screen titles 22 to 26 px. Transitions at most 150 to 200 ms and disabled when reduced motion is requested.

| Role | Light | Dark |
| --- | --- | --- |
| Background | #F6F7F9 | #15181D |
| Surface (cards) | #FFFFFF | #1C2027 |
| Subtle border | #E3E6EB | #2A303A |
| Primary text | #1F2430 | #E6E9EF |
| Secondary text | #5B6475 | #9AA3B2 |
| Accent (primary button) | #3F5EA8 | #7C98E0 |
| Success | #2F7D5B | #5DB58C |
| Warning | #B7791F | #D9A441 |
| Error | #B3412E | #E07A68 |

These are starting values. The agent computes WCAG AA contrast (4.5:1 for text) for every text and background pair in both themes and adjusts if needed, then the owner reviews screenshots.

**Accessibility:** full keyboard operation including the wizard, visible focus ring, labels for every control, accessible names for the comparison slider and cards, no information by color alone, reduced-motion support.

**Internationalization:** typed Indonesian and English dictionaries with identical keys (tested), natural Indonesian wording rather than literal translation, localized native dialog titles and filters, and a structure that lets the community add languages later.

## 9. Non-functional requirements

- **Offline:** after a model is installed, no network access is possible or needed; verified by capability audit and a launch test with the network disabled.
- **Memory (release, measured):** Balanced at 24 MP with refine standard at or below about 2.5 GB peak; Fast at or below about 1 GB. If exceeded, name the buffers and fix them without changing results, or report measured options.
- **Responsiveness:** the UI never freezes; large queues stay smooth; preview cache capped.
- **Startup:** report time to UI-ready measured in release mode (the verified signal is the startup log line, not process liveness).
- **Security:** keep the strict CSP, minimal capabilities, scoped asset protocol, path validation, atomic writes, and the compile-time guard. Do not add shell, filesystem, or HTTP plugins. Any new capability needs owner approval and an entry in docs/SECURITY\_MODEL.md.
- **Licenses:** every new Rust or npm dependency passes the license-check skill and cargo-deny or the npm license check, and is recorded in THIRD\_PARTY\_LICENSES.md.
- **Cross-platform:** CI green on three systems; Windows manually tested; macOS (Apple Silicon) and Linux experimental until tested on real hardware.

## 10. Architecture changes

- **crates/core:** new modules for manifest (data and validation), model manager (resume, disk check, verification, sources), recommendation (pure function), and system info (RAM). The core still prints nothing and has no UI code.
- **crates/cli:** `rustmatte models list|download|delete|verify` extended to use the manifest; CLI keeps working exactly as before for existing flags.
- **crates/app:** thin command layer for the new operations; wizard and settings screens in Svelte; state machine for wizard steps; per-session result store for the retry feature.
- **Settings file (JSON, atomic writes, tolerant of corruption):** language, theme, active model id, wizard-completed flag with wizard version, network switch, output defaults, refine level. Wizard answers and detected RAM are **not** stored.

## 11. Test plan

**Automatic (must pass in CI without model files):** one test per recommendation row plus RAM cases; wizard state machine (forward, back, close, complete); manifest validation (hash and size present, reason codes translated); resume download against a local test server (206, 200 fallback, truncated, oversized, wrong hash); disk-space check with injected values; RAM detection through an injectable function; i18n key parity and non-empty values; settings load and save with corrupt and missing files; path validation; feedback URL allow-list.

**Manual (owner):** wizard in light and dark, Indonesian and English, keyboard only, with the network disabled, with the download interrupted midway, and with a narrow window; settings changes; model delete and re-download; retry with another model; 20-photo batch with cancel; installer on a clean second PC. The agent cannot judge how the UI looks and must not claim visual quality; it provides screenshots if its tooling allows, or a numbered checklist.

## 12. Phase plan

Each phase ends with the quality gate (fmt, clippy -D warnings, tests with `-j 2`, cargo-deny, frontend check and build, supported release builds), updated docs, a phase report with complete Known issues, Decisions made, and Proposed next step, and a stop for owner confirmation.

| Phase | Scope | Acceptance |
| --- | --- | --- |
| **5.0** | D-032 close-out and carry-over checks (section 3): commits, memory re-measure, seam check, haze-risk review, clean comparison sets | Targets in section 9 met or options reported; clean tree; CI green |
| **5A** | Manifest, model manager with resume and disk check, RAM detection, recommendation function, CLI models subcommands | All automatic tests pass; real downloads of both models verified against both sources; RAM thresholds re-validated |
| **5B** | Wizard UI (7.5), bilingual dictionaries, cards (7.6), wizard state machine | Screenshots or manual checklist done by owner; wizard cannot be skipped and completes only with a verified model |
| **5C** | Settings (7.7), model management, feedback link (7.8), retry with another model (7.9) | Owner manual test of every settings action and the retry flow |
| **5D** | Visual polish (section 8, 7.10): palette, spacing, empty and error states, advanced options collapse | Owner approves screenshots in both themes and languages |
| **Release** | Provenance, icons, screenshots, GitHub settings, release candidate tag, clean second-PC install test, publish | Section 14 checklist complete |

Rough effort at 2 to 3 hours per day: 5.0 about 1 to 2 days; 5A 2 to 4 days; 5B 2 to 4 days; 5C 2 to 4 days; 5D 2 to 5 days. Resume downloads and model management are the likeliest to expand.

## 13. Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Too many wizard steps annoy users | Exactly two questions; Back button; short text |
| Recommendation feels wrong | Show the reason, always show the alternative and See all models, allow changing the model any time |
| Large downloads fail on poor connections | Resume, retries, import route, clear errors |
| Low-RAM computers freeze with Balanced | RAM detection, warning, Fast as the safe default |
| Memory regression after refine changes | Measure per phase, fixed targets, hash comparisons |
| Model licenses or provenance | Checklist, primary sources, honest notes in MODELS.md |
| UI looks inconsistent or dated | Tokens in one place, owner screenshot review in 5D |
| Scope creep before release | Out-of-scope list; third model and extra questions need owner approval |
| Unverified cross-OS builds | Experimental labels, CI logs, volunteers for real hardware |

## 14. Release readiness checklist (v1.0)

- CI green (all five jobs) on the release commit; release workflow run on a release candidate tag; artifacts and SHA256SUMS reviewed in the draft release.
- Windows installer tested on a clean second PC, including offline use after the model is installed and a clean uninstall.
- Provenance file resolved; placeholder icons replaced; README screenshots and GIF added; comparison claims about other projects sourced.
- README states: Linux needs glibc 2.39 or newer; macOS is Apple Silicon only and unsigned; Windows installer is unsigned; models download once with consent; measured speeds are from one machine; glass is cut out as solid.
- GitHub settings applied: description, topics, labels, branch protection with exact job names as required checks, Dependabot alerts, private vulnerability reporting.
- Release notes cover what works, what does not, known limitations, checksums, and how to report problems.

## 15. Agent working rules (continuing AGENTS.md)

1. Read AGENTS.md, PROJECT\_STATE.md, docs/TASKS.md, and this document at the start of every session; work on one phase only.
2. Plan first, then code; ask before choosing a dependency, changing architecture, changing a default, or deviating from this document.
3. Verify by running. Never claim success you did not observe. Mark everything you could not verify.
4. Every number is measured in release mode, one fresh process per measurement where memory matters, and labeled with the hardware.
5. Never use model files or crates with unclear licenses; run the license-check skill first.
6. Defaults and visual quality are owner decisions. Do not change a default silently and do not describe visual quality as good without owner review.
7. Keep documentation current in the same phase (ARCHITECTURE, DECISIONS, PITFALLS, TASKS, PROJECT\_STATE, CHANGELOG, README).
8. At most two attempts at any failing step, then report the exact error.
9. Do not push, tag, or publish; the owner does that.

## 16. Backlog and conditions

**BEN2 “Maximum” tier (v1.1 candidate).** Conditions: (a) confirm from the model card that the weights are BEN2 Base (open, MIT) and not the commercial full model; (b) verify how the ONNX “FP32” file was produced (the official ONNX is reported to be fp16, so a third-party FP32 file may be a conversion) and prefer exporting it ourselves from official weights; (c) record the training-data note (it mentions DIS5K plus a proprietary dataset) with the same standard as other models; (d) measure time and memory with the final refine code (earlier measurements: about 14 to 19 s per photo and about 2.6 to 2.8 GB RAM on the reference PC); (e) owner reviews fair side-by-side results against Balanced; (f) it appears as a third model card with honest time and RAM warnings, through the manifest only.

**Other backlog:** HEIC and RAW (v1.1), crop to subject, more output formats, manual refine brush, presets, resume improvements, upscaler (v2), browser version (v2), mobile (v3), installers with bundled models, code signing, auto-update (only if a privacy-preserving design is approved).

## 17. Definition of done for v1.0

A new user can install the app, choose a language, answer two questions, accept one consented download, process photos offline, switch models, change language and theme in Settings, and send feedback, with no step requiring documentation. All automatic checks pass, the owner has reviewed the UI visually and tested a clean second PC, and the release checklist in section 14 is complete.
