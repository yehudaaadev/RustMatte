# RustMatte: PRD and Agent Playbook

Offline AI background remover and photo enhancer. Rust core, open source, cross-platform. Version 1.1 of this document.

## 1. Meta Prompt (paste this first to the coding agent)

```
You are a senior Rust engineer and product-minded open-source maintainer.
You are building RustMatte, an offline, privacy-first AI background remover
(and later photo upscaler) with a Rust core.

Read this entire document before writing any code. Follow Section 12
(Agent Rules) strictly. Work ONE phase at a time (Section 13). After each
phase: run build, tests, cargo clippy, cargo fmt, then report what you did,
what you verified by actually running it, and what is still uncertain.
Do not start the next phase until the user confirms.

Never claim something works unless you ran it. If a crate API is uncertain,
read the current docs for the pinned version instead of guessing. If a
license is unclear, stop and ask. Prefer simple, readable, well-tested code
over clever code.
```

## 2. Project Summary and Analysis

**One-liner:** Drag a photo in, get a transparent PNG out. Fully offline, no upload, no account, no limits.

**Problem.** Online background removers (remove.bg and similar) limit free usage, reduce resolution on free tiers, require accounts, and upload private photos to someone else's server. Local alternatives exist (for example rembg) but are Python tools that ordinary users cannot install or run.

**Opportunity.** A single small installer where the user drags photos in and gets results, with batch processing for sellers who have hundreds of product photos.

**Competitor analysis (verify current status before finalizing the README).**

| Tool | Strength | Weakness we exploit |
| --- | --- | --- |
| remove.bg and online tools | Easy, good quality | Quotas, paywall, uploads photos |
| rembg (Python) | Free, popular | Needs Python and terminal, no friendly UI |
| Upscayl (upscaler only) | Good UI, popular | Does not remove backgrounds |
| Photopea and web editors | Powerful | Heavy, general purpose, upload or account friction |

**Differentiation.**

- Zero setup: one installer, no Python, Java, or Docker
- Privacy: images never leave the device (only the one-time model download touches the network, clearly disclosed)
- Batch-first workflow for sellers and photographers
- Light and fast thanks to Rust
- Later: background removal and upscaling in one lightweight app

**Honest risks.** Model quality (hair and fur edges) decides whether people stay. The AI models are not our invention, so our value is packaging, UX, speed, edge-quality postprocessing, and trust.

## 3. Goals, Non-goals, Success Metrics

**Goals (v1.0)**

- Remove background from a single image with good edge quality
- Batch process a folder with progress and per-file error reporting
- Desktop app with drag and drop and before/after preview
- Fully offline after the first model download
- Installers for Windows (primary target, fully tested) and CI verification for macOS and Linux (experimental)

**Non-goals (v1.0)**

- Full photo editor (no brushes, layers, filters)
- Accounts, cloud sync, telemetry, ads
- Native Android and iOS apps (Android planned for v3)
- Video background removal
- Upscaler (planned for v2)
- Advanced camera RAW and HEIC decoding (planned for v1.1; v1.0 rejects unsupported formats gracefully)
- Standalone image compression tool (planned for v1.x backlog)

**Success metrics**

- 30-image test set (people, hair, fur, products, glass) with acceptable results on at least 80 percent
- 1080p image processed on CPU within acceptable interactive latency (target: UNMEASURED, measure and report honestly on real hardware)
- Release binaries run on Windows without extra installs; macOS and Linux built and tested in CI
- Within 3 months of v1.0: 100+ GitHub stars, first external contributor

## 4. Target Users

- Online sellers and product photographers (batch, consistent output, commercial usage)
- Students and job seekers (ID and document photos, privacy matters)
- Content creators and designers (thumbnails, stickers)
- Developers (CLI and Rust library)

## 5. Tech Stack

| Layer | Choice | Notes |
| --- | --- | --- |
| Language | Rust (stable, 2021 edition or newer) | Pin toolchain in rust-toolchain.toml |
| AI inference | ort (ONNX Runtime bindings) | Pin exact version. The 2.x API changes often. Read docs for the pinned version |
| Image I/O | image crate | PNG, JPEG, WebP, BMP at minimum |
| Arrays | ndarray | Tensor preprocessing |
| Parallelism | rayon | Batch mode with bounded concurrency (1-2 full-res images in RAM) |
| CLI | clap (derive) |  |
| Errors | thiserror in core, anyhow in binaries | No unwrap in library code |
| Desktop app | Tauri 2 | Web frontend, small installer |
| Frontend | Svelte or React with TypeScript, plain CSS or Tailwind | Keep dependencies few |
| Logging | tracing | Quiet by default |
| Download | reqwest (rustls) | Model download only, with checksum (sha2) |
| CI | GitHub Actions | build, test, clippy, fmt, release artifacts across Windows, macOS, Linux |
| Model Candidates | BiRefNet_lite (MIT, Swin-T) or ISNet (verify license) | Decided in Phase 1 via CPU benchmark. RMBG-2.0 rejected (non-commercial license violates seller/photographer use) |
| Upscaler (v2) | Real-ESRGAN ONNX | Verify weight license. Needs tiling |

## 6. Architecture

```
RustMatte/
├── Cargo.toml                 (workspace)
├── rust-toolchain.toml
├── crates/
│   ├── core/    model manager, preprocess, inference, postprocess, errors
│   ├── cli/     clap-based command line
│   └── app/     Tauri 2 desktop app (phase 3)
├── web/         WASM and PWA build (v2, later)
├── tests/       fixtures and integration tests (small images only)
├── docs/
├── .github/workflows/
├── LICENSE
├── README.md
├── CONTRIBUTING.md
└── CHANGELOG.md
```

**Rules of the architecture**

- core contains NO UI code and NO printing to stdout. It returns Results.
- cli and app are thin wrappers around core.
- core exposes a small, stable API, for example: `Remover::new(config)`, `remover.process(image) -> Result<RgbaImage>`, `remover.process_batch(paths, options, progress_callback)`.
- Models are never committed to git and never bundled into the v1.0 installer. They download on first run to the OS app data folder, verified by SHA-256.

**Processing pipeline**

1. Decode image, apply EXIF orientation
2. Optional fast mode: downscale very large images early to save decode time and RAM (model input is fixed, so inference time is unchanged)
3. Resize to model input size (keep aspect ratio handling documented)
4. Normalize to tensor as the model expects
5. Run inference with CPU as default (optional hardware execution providers DirectML on Windows, CoreML on macOS with automatic CPU fallback)
6. Take mask output, resize back to target resolution
7. Refine mask: guided-filter style refinement against original photo, feathering only in transition zones, and edge color cleanup (defringing)
8. Apply mask as alpha, export PNG at specified resolution

## 7. Features

**Must have (v1.0)**

- Single image background removal, transparent PNG at original resolution
- Batch folder processing, parallel with bounded queue, progress, and per-file errors (one bad file never stops the batch)
- First-run model download with progress, resume on failure, and checksum verification (cached in OS app data, never bundled)
- Output options: transparent PNG, solid color background, crop to subject, WebP output
- Fast mode option: downscale very large images early to conserve decode time and RAM
- Execution: CPU default with optional hardware acceleration (DirectML/CoreML) and automatic CPU fallback
- CLI: `RustMatte input.jpg -o output.png` and `RustMatte ./folder --out ./result`
- Desktop app: drag and drop, before/after slider, batch queue, save and export
- Supported formats: JPEG, PNG, WebP, BMP (unsupported formats report friendly error, never crash)
- Languages: English and Indonesian
- Light and dark mode

**Should have (v1.1 / v1.x)**

- v1.1: Camera RAW support (first step: extract embedded preview JPEG; full RAW decoding later)
- v1.1: Apple HEIC/HEIF decoding
- v1.x: Separate image compression utility
- v1.x: Manual refine: simple restore/erase brush on the mask
- v1.x: Presets: marketplace white background, passport photo sizes and colors
- v1.x: Model selector (fast vs high quality)
- v1.x: Drag results out of the app into other apps

**Later (v2+)**

- Upscaler with tiling for large images (v2.0)
- Browser version (WASM/PWA) for phones and tablets (v2.0)
- Android version (v3.0, after desktop is stable and complete)
- iOS version (v3.0+, if real demand exists)

## 8. UI and UX Specification

**Principles:** zero learning curve, no clutter, never block the UI during processing, always show progress, always explain errors in plain language.

**Main window (single screen)**

- Large drop zone in the center: 'Drop photos or folder here, or click to browse'
- After dropping: left side is a thumbnail queue (status: waiting, processing, done, failed), center is a before/after comparison slider with zoom and pan on a checkerboard background
- Right or bottom panel: output options (format, background color picker with transparent default, crop to subject, fast mode toggle, output folder), a primary 'Remove background' button, and 'Save' and 'Save all' buttons
- Top bar: settings (language, theme, model, hardware acceleration, output folder), about and licenses

**First run:** a short, honest dialog: 'RustMatte needs to download its AI model once. After that it works fully offline.' With progress bar, retry, and a manual-import option for users who prefer to supply the model file.

**States to design:** empty, loading model, processing (progress per file), done, partial failure with error list, out of memory warning, unsupported file.

**Accessibility:** keyboard shortcuts for open, process, save, next/previous image. Sufficient contrast. Labels on all controls.

**Tone of the UI text:** short, friendly, no jargon.

## 9. Final Output (what 'done' looks like)

- GitHub repository with the workspace above, building cleanly from a fresh clone with documented commands
- Release artifacts: Windows installer (.msi or .exe with WebView2 bootstrapper); macOS .dmg and Linux AppImage/.deb verified in CI
- A CLI binary for each platform
- README with demo, install instructions, usage, model and license notes, FAQ, and roadmap
- CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue and PR templates
- Automated tests and CI passing on Windows, macOS, and Linux
- THIRD_PARTY_LICENSES file covering code dependencies and model weights
- Browser version, mobile apps, and bundled model installers are NOT part of v1.0

## 10. Expectations (quality bar)

- Works offline after setup, verified by running with the network disabled
- Results are visibly good on a mixed test set, not just on easy images
- Startup under acceptable interactive latency without model load, UI never freezes (UNMEASURED until hardware benchmarks)
- Memory stays bounded on very large images (bounded batch queue, early downscale option, drop buffers immediately)
- No panics on corrupt, huge, or unusual files. Clear error messages
- Code is idiomatic, documented where non-obvious, and readable by new contributors
- Installer is small because models are downloaded on first run, not bundled
- Honest documentation: known limitations are listed in the README

## 11. What to Avoid

- Adding telemetry, analytics, ads, accounts, or any hidden network calls
- Bundling model weights in git or in the v1.0 installer
- Using model weights or crates with non-commercial or unclear licenses (conflicts with seller and photographer commercial use)
- Scope creep: no editor features, no RAW/HEIC in v1.0, no cloud
- Copying code from tutorials for old ort versions. Always check the pinned version's docs
- unwrap() and expect() in library code, panics on user input
- Giant files and god modules. Keep modules small and focused
- Premature optimization and unnecessary dependencies
- Claiming performance or quality numbers without measuring them (mark unmeasured estimates as UNMEASURED)
- Over-engineering abstractions before the first working pipeline exists
- Starting the next phase without user confirmation
- Rewriting working code unprompted

## 12. Agent Rules

1. **Read first.** Read this whole document and the current repo state before changing anything.
2. **One phase at a time.** Only work on the phase the user approved. Do not touch later phases.
3. **Plan, then code.** At the start of each phase, write a short plan (files to create or change, risks) and wait for a quick confirmation if anything is ambiguous.
4. **Verify by running.** Build, run tests, run the actual CLI or app on real sample images. Report real output, not assumptions. If you could not run something, say so explicitly.
5. **Quality gates before reporting done:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` must pass.
6. **Pin and check versions.** Pin dependency versions. For ort, Tauri, and other fast-moving crates, read the docs for the exact version used. Never guess API signatures.
7. **License discipline.** Before adding any dependency or model, record its license in THIRD_PARTY_LICENSES. If unclear, copyleft (AGPL/GPL), non-commercial, or custom, stop and ask the user.
8. **No silent decisions.** Ask before choosing a model, adding a major dependency, changing architecture, or deviating from this document. Record decisions in docs/DECISIONS.md.
9. **Small commits, clear messages.** Conventional commit style. One logical change per commit.
10. **Error handling.** Library code returns typed errors. No panics on user input. Messages must be understandable by non-developers when shown in the UI.
11. **Tests.** Add unit tests for preprocessing and postprocessing math, and integration tests with small fixture images. Do not commit large files.
12. **Security and privacy.** No network access except the documented model download. Validate file paths and sizes. Never log image contents or full user paths unnecessarily.
13. **Performance honesty.** Benchmark with fixed images and report numbers with hardware details. Do not claim performance numbers until measured.
14. **Keep the docs alive.** Update README, CHANGELOG, and PROJECT_STATE.md at the end of every phase so a new session can resume without context.
15. **Be candid.** Report failures, bad results, and uncertainty plainly. Do not hide problems or claim success you did not verify.
16. **Stay in scope and in language.** Code, comments, and docs in English. Talk to the user in the language they use (Indonesian is fine).

**End-of-phase report format**

```
PHASE X REPORT
- What was built
- Files changed
- Commands I ran and their real results
- Test and benchmark results
- Known issues and limitations
- Decisions made (and any that need user approval)
- Proposed next step
```

## 13. Phase Plan with Acceptance Criteria

**Phase 0: Setup**

- Create Cargo workspace, rust-toolchain.toml, README skeleton, CI for build, test, clippy, fmt across Windows, macOS, Linux
- Project license decision recorded as PENDING in docs/DECISIONS.md (no LICENSE file created until decided)
- Accept when: fresh clone builds and CI is green across all platforms

**Phase 1: Core and CLI (v0.1)**

- Model manager (download, checksum, cache), preprocess, inference, postprocess, PNG export
- CPU benchmark on `tests/fixtures` comparing candidate models (`BiRefNet_lite` vs `ISNet`) for latency, memory, and visual quality; owner selects default model
- Verify supply chain: source URL, license, SHA-256, and ONNX operator compatibility in `ort` on CPU
- CLI for a single image
- Accept when: `RustMatte photo.jpg -o out.png` produces a correct transparent PNG on test images, benchmark documented, tests pass, and it works with network off after model is cached

**Phase 2: Batch and speed (v0.2)**

- Folder processing with rayon, bounded worker queue, progress, per-file error report
- Execution providers (DirectML on Windows, CoreML on macOS) with automatic CPU fallback
- Edge refinement improvements (guided filter, feathering in transition zone, defringing)
- Fast mode option for early downscaling of large images
- Accept when: a folder of images processes without crashing, memory remains bounded, failures are reported individually, and benchmarks are documented

**Phase 3: Desktop app (v0.3)**

- Tauri 2 app wrapping core: drop zone, queue, before/after slider, options, save
- Windows installer with WebView2 bootstrapper
- English and Indonesian, light and dark
- Accept when: a non-technical person can install, drop a photo, and save the result without reading docs

**Phase 4: Release (v1.0)**

- GitHub Actions release pipeline for Windows (primary), macOS, and Linux
- README with demo, CONTRIBUTING, templates, license audit
- Accept when: tagging a release publishes working installers tested on Windows and built via CI

**Phase 5: Upscaler and web (v2.0)**

- Real-ESRGAN ONNX with tiling and a memory cap
- Research and prototype the browser/WASM version (separate PRD)

## 14. Platform Strategy

| Platform | Plan |
| --- | --- |
| Windows | Primary target for v1.0. Full manual testing. Installer handles WebView2 bootstrapper. |
| macOS | Experimental in v1.0. Built and tested in CI; CoreML optional execution provider. |
| Linux | Experimental in v1.0. Built and tested in CI (.AppImage and .deb). |
| Browser (WASM/PWA) | Planned for v2.0, runs locally in browser for privacy. |
| Android | Planned for v3.0, after desktop is complete and stabilized. |
| iPhone | Planned for v3.0+, PWA or native depending on demand. |

Do not chase all platforms at once. A rock-solid Windows desktop v1.0 beats five half-working ports.

## 15. Licensing

- Project license: PENDING owner decision.
  - PolyForm Noncommercial is REJECTED because it is not open source and forbids commercial use by our target users (sellers, photographers).
  - Evaluated options: MIT or Apache-2.0 (maximum adoption, permits commercial use/sale); GPL-3.0 or AGPL-3.0 (open source copyleft, forces forks to share source; does NOT forbid selling); non-commercial licenses (not open source).
- Every dependency and every model weight file must be compatible with commercial use by end users.
- RMBG-2.0 rejected due to CC BY-NC 4.0 license restricting commercial users.
- Maintain THIRD_PARTY_LICENSES and show it in the app's About screen.

## 16. Risks and Mitigations

| Risk | Mitigation |
| --- | --- |
| Poor edges on hair and fur | Invest in postprocessing (guided filter, defringing), build hard test set, benchmark candidates |
| ort API changes | Pin version, read matching docs, wrap ort usage in one module |
| Model license problems | Verify weight license from primary source; avoid non-commercial weights; record in THIRD_PARTY_LICENSES |
| Third-party ONNX export flaws | Export from official weights or verify operator support (e.g., deformable convolution) on CPU in ort |
| Large model download | Download on demand, resume support, manual import option |
| GPU provider differences | CPU default, DirectML/CoreML optional, always fall back to CPU, test on real hardware |
| Memory spikes on huge images | Fast mode early downscale, bounded batch parallelism (1-2 images in RAM), free buffers immediately |
| Scope creep | Move HEIC/RAW to v1.1, compression tool to v1.x, Android to v3 |
| Crowded space | Win on zero-setup, privacy, offline batch workflow, and Indonesian-friendly UX |

## 17. Open Questions for the Owner

1. Project license: Decision PENDING (MIT, Apache-2.0, GPL-3.0, AGPL-3.0)?
2. Default model: To be chosen after Phase 1 benchmark between BiRefNet_lite and ISNet.
3. Frontend framework preference for the Tauri UI (Svelte or React)?
4. Final project name: Confirm if "RustMatte" is final or a placeholder.

## 18. Changelog of this PRD

- **2026-11-24 (v1.1)**:
  - Model: Rejected RMBG-2.0 due to CC BY-NC 4.0 license restricting commercial use by target users (sellers/photographers). Removed incorrect size claims (actual: ~234MB q4f16 to ~1GB fp32). Established `BiRefNet_lite` (MIT) and `ISNet` as primary candidates to be benchmarked on CPU in Phase 1 before owner selection. Added supply-chain verification for ONNX exports.
  - License: Rejected PolyForm Noncommercial (not open source, forbids commercial use). Set project license decision to PENDING without generating a LICENSE file.
  - Distribution: Reaffirmed first-run model download; model is never bundled in v1.0 installer and never committed to git. Bundled installer deferred to later task.
  - Scope: Restricted v1.0 input formats to JPEG, PNG, WebP, BMP with graceful error handling. Moved HEIC and RAW to v1.1 (embedded preview JPEG first). Added fast mode (early downscale for large images to save decode time and RAM). Moved separate compression utility to v1.x backlog.
  - Platforms: Designated Windows as primary target with WebView2 bootstrapper; labeled macOS and Linux as experimental pending real-machine tests. Moved Android to v3.
  - Execution: Set CPU as default execution provider; DirectML and CoreML maintained as optional settings with automatic CPU fallback.
  - Quality & Memory: Codified bounded batch queue (1-2 full-res images in RAM) and mask postprocessing (guided filter, feathering, defringing).
  - Honesty: Marked all unmeasured performance and latency targets as UNMEASURED.
