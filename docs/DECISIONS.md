# Architectural Decisions Log

Rule: Every non-trivial decision made by the agent must be appended to this log. Decisions marked `pending` require explicit owner approval before implementation. Conflicting past decisions must be marked `superseded` with a pointer to the superseding decision.

Format:
- ID: Unique sequential identifier (D-001, D-002, ...)
- Date: YYYY-MM-DD
- Decision: Concise statement of what was decided
- Reason: Why this choice was made
- Status: accepted | pending | superseded
- Alternatives considered: Other options evaluated and why they were rejected

---

## Accepted Decisions

### D-001: Rust Core
- Date: 2026-11-24
- Decision: Use Rust (stable, 2021 edition or newer) as the core language.
- Reason: High performance, deterministic memory safety, small binary footprint, and cross-platform compilation without runtime bloat.
- Status: accepted
- Alternatives considered: Python (too heavy for desktop installers, requires Python runtime), C++ (memory safety risks).

### D-002: Cargo Workspace Architecture
- Date: 2026-11-24
- Decision: Structure the project as a Cargo workspace with distinct crates: `rustmatte-core` (library), `rustmatte-cli` (binary), and `crates/app` (Tauri 2, in Phase 3).
- Reason: Enforces strict separation of concerns; keeps core logic free of UI and CLI dependencies.
- Status: accepted
- Alternatives considered: Monolithic single-crate repository (leads to entangled UI and business logic).

### D-003: ONNX Inference via `ort` Crate
- Date: 2026-11-24
- Decision: Use the `ort` crate for ONNX Runtime bindings, pinning the exact version.
- Reason: Industry standard for running vision models across varied hardware execution providers.
- Status: accepted
- Alternatives considered: Tract (pure Rust, but lacks full operator support for modern segmentation models), Burn.

### D-004: Tauri 2 for Desktop Application
- Date: 2026-11-24
- Decision: Use Tauri 2 for the desktop GUI.
- Reason: Lightweight installers, low RAM usage, uses OS webview, strong cross-platform support.
- Status: accepted
- Alternatives considered: Electron (heavy installer bloat, high RAM usage), Iced/Slint (less flexible web ecosystem for sliders and previews).

### D-005: Model Download on Demand (Not Committed to Git or Bundled in v1.0)
- Date: 2026-11-24
- Decision: Models download on first run to the OS app data directory, verified by SHA-256; never commit weights to git and never bundle in v1.0 installer.
- Reason: Keeps installer and git repository sizes minimal; avoids redistributing third-party weights directly.
- Status: accepted
- Alternatives considered: Bundling model inside v1.0 installer (deferred to a separate optional task post-v1.0), Git LFS.

### D-006: Fully Offline Operation
- Date: 2026-11-24
- Decision: Application operates completely offline after the initial model download.
- Reason: User privacy; images never leave the local machine.
- Status: accepted
- Alternatives considered: Cloud API inference (violates privacy premise, requires subscriptions and servers).

### D-007: One Phase Per Session Protocol
- Date: 2026-11-24
- Decision: Enforce one phase per session, verified by running real tests and commands.
- Reason: Prevents agent hallucination, uncontrolled scope expansion, and broken state progression.
- Status: accepted
- Alternatives considered: Unconstrained multi-phase autonomous execution (high risk of cumulative errors).

### D-008: English and Indonesian UI Localization
- Date: 2026-11-24
- Decision: Support both English and Indonesian language interfaces in v1.0.
- Reason: Core target demographic includes Indonesian online sellers, content creators, and students.
- Status: accepted
- Alternatives considered: English-only initial release.

### D-011: Frontend UI Framework
- Date: 2026-11-24
- Decision: Adopt Svelte with TypeScript for the Tauri 2 desktop UI.
- Reason: Minimal runtime overhead, fast DOM reactivity, small bundle size compared to React, and simple component architecture.
- Status: accepted
- Alternatives considered: React (larger runtime footprint, more complex re-render cycles).

### D-012: Project Name (RustMatte)
- Date: 2026-11-24
- Decision: Rename project from placeholder LocalBG to **RustMatte** (crate and repository slug: `rustmatte`).
- Reason: Retired LocalBG because it is too common. The name reflects the technical focus (alpha matting / background removal in Rust).
- Trademark note: Using "rust" in a repo/crate name is permissible under the Rust Foundation trademark policy when referring to the Rust language, but the name must not be used as a standalone brand, logo, or commercial domain that incorporates the Rust trademark without checking the Rust Foundation trademark policy. Added disclaimer to README.
- Status: accepted
- Alternatives considered: LocalBG (retired placeholder).

### D-013: Model Selection Strategy and RMBG-2.0 Rejection
- Date: 2026-11-24
- Decision: Reject RMBG-2.0 as default model. Establish BiRefNet_lite (MIT, Swin-T backbone) as default candidate and ISNet as secondary candidate. Final selection will be decided in Phase 1 via CPU benchmark on `tests/fixtures`.
- Reason: RMBG-2.0 weights are licensed under CC BY-NC 4.0, which strictly prohibits commercial use. Our core users (online sellers, photographers) process images commercially, making RMBG-2.0 legally hazardous. Previous size claims were incorrect (actual ONNX size is ~234MB to ~1GB). Additionally, third-party ONNX exports must be verified for operator compatibility (e.g. deformable convolution) on CPU.
- Status: accepted
- Alternatives considered: RMBG-2.0 (rejected due to non-commercial license and large size). Supersedes D-009.

### D-014: Project License (GPL-3.0-or-later)
- Date: 2026-11-24
- Decision: Adopt the **GNU General Public License v3.0 or later (GPL-3.0-or-later)** for the project source code.
- Reason: Genuine open source that guarantees software freedom; forces commercial proprietary forks to publish their source code; fully compatible with MIT/Apache-2.0 dependencies and MIT model weights. Note: GPL does NOT forbid selling copies.
- Status: accepted
- Alternatives considered: PolyForm Noncommercial (rejected because it is not open source and forbids business use by sellers/photographers), MIT/Apache-2.0 (allows proprietary closed-source relicensing and selling without sharing improvements). Supersedes D-010.

### D-015: v1.0 Scope, Image Formats, and Backlog Segregation
- Date: 2026-11-24
- Decision: Restrict v1.0 input formats to JPEG, PNG, WebP, and BMP. Move camera RAW and HEIC decoding to v1.1 (first step for RAW: extract embedded preview JPEG). Move standalone photo compression utility to v1.x backlog. Add fast mode option (early downscale of large images to reduce decode time and memory).
- Reason: HEIC requires complex cross-platform C library linkage, and RAW requires demosaicing and color profile management. Fast mode reduces decode latency and memory usage on large images without altering fixed model inference time.
- Status: accepted
- Alternatives considered: Full RAW/HEIC in v1.0 (rejected due to cross-platform compilation risk).

### D-016: Platform Tiering and Default Execution Provider
- Date: 2026-11-24
- Decision: Set CPU as the default execution provider across all platforms. Keep hardware execution providers (DirectML on Windows, CoreML on macOS) as optional settings with automatic CPU fallback. Designate Windows as the primary v1.0 target with full manual testing and WebView2 bootstrapper. Label macOS and Linux as experimental until physically tested, while building and testing in CI from Phase 0. Move Android to v3.
- Reason: Guarantees reliable out-of-the-box operation on standard PCs without discrete GPUs or specialized driver setups.
- Status: accepted
- Alternatives considered: DirectML as default (unstable across varied hardware without fallback), immediate mobile support.

---

## Superseded Decisions

### D-009: Default AI Model Selection (RMBG-2.0)
- Date: 2026-11-24
- Decision: Select RMBG-2.0 as default model.
- Status: superseded by D-013.
- Reason: Superseded because CC BY-NC 4.0 license forbids commercial use by target users, and ONNX sizes were significantly underestimated.

### D-010: Project License (PolyForm Noncommercial)
- Date: 2026-11-24
- Decision: Adopt PolyForm Noncommercial License 1.0.0.
- Status: superseded by D-014.
- Reason: Superseded because PolyForm is not open source and forbids business use by sellers and photographers.
