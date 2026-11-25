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

### D-015: ORT Session Configuration for BiRefNet CPU Inference
- Date: 2026-11-24
- Decision: For `BiRefNet_lite` on CPU with `ort` 2.0.0-rc.15, disable the default memory pattern (`with_memory_pattern(false)`) and enable environment system allocators (`with_config_entry("session.use_env_allocators", "1")`).
- Reason: The default `BFCArena` allocator fails during the unrolled ASPP deformable convolution step with `BFCArena::AllocateRawInternal Failed to allocate memory for requested buffer of size 822083584` in `/decoder/decoder_block1/dec_att/aspp_deforms.2/atrous_conv/Sum`. Using OS environment allocators completely bypasses the arena allocation failure and allows stable CPU inference. Verified in Phase 1B ORT sweep: D-015 reduces peak Working Set by 1.33 GB (3,995 MB vs 5,331 MB).
- Status: accepted
- Alternatives considered: Using default arena allocator (fails with runtime crash), exporting custom ONNX without deformable convolution (untested, may reduce mask accuracy).

### D-016: v1.0 Scope, Image Formats, and Backlog Segregation
- Date: 2026-11-24
- Decision: Restrict v1.0 input formats to JPEG, PNG, WebP, and BMP. Move camera RAW and HEIC decoding to v1.1 (first step for RAW: extract embedded preview JPEG). Move standalone photo compression utility to v1.x backlog. Add fast mode option (early downscale of large images to reduce decode time and memory).
- Reason: HEIC requires complex cross-platform C library linkage, and RAW requires demosaicing and color profile management. Fast mode reduces decode latency and memory usage on large images without altering fixed model inference time.
- Status: accepted
- Alternatives considered: Full RAW/HEIC in v1.0 (rejected due to cross-platform compilation risk).

### D-017: Platform Tiering and Default Execution Provider
- Date: 2026-11-24
- Decision: Set CPU as the default execution provider across all platforms. Keep hardware execution providers (DirectML on Windows, CoreML on macOS) as optional Cargo-feature-gated settings with automatic CPU fallback (note: v0.1 release binaries compile CPU-only; see D-030/D-031). Designate Windows as the primary v1.0 target with full manual testing and WebView2 bootstrapper. Label macOS and Linux as experimental until physically tested, while building and testing in CI from Phase 0. Move Android to v3.
- Reason: Guarantees reliable out-of-the-box operation on standard PCs without discrete GPUs or specialized driver setups.
- Status: accepted
- Alternatives considered: DirectML as default (unstable across varied hardware without fallback), immediate mobile support.

### D-018: Default AI Model and Two-Tier Architecture Selection
- Date: 2026-11-24
- Decision: Adopt Two-Tier Model Architecture (Final):
  1. Default Quality Tier: `BiRefNet_lite_512` (183 MB ONNX, MIT license).
  2. Optional Fast Tier: `U-2-Netp` (4.36 MB ONNX, Apache-2.0 license). Mandatory honest caveat wherever described: "best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds."
  3. `BiRefNet_lite_1024`, `ISNet`, `fp16`, and `q8` variants are out of scope.
- Status: accepted
- Reason: Balances commercial licensing safety, sub-4s latency on consumer CPUs, and crash-free memory consumption.
- Alternatives considered: Single model BiRefNet 1024 (deferred due to memory exhaustion/crash), ISNet (dropped due to dataset non-commercial terms), U2Netp as only model (lacks micro-detail on hair).

### D-019: Model Downloader, Cache Strategy, and Integrity Verification
- Date: 2026-11-24
- Decision: Implement download-on-first-run with user consent (`[y/N]` interactive prompt or `--yes`). Models are checked against a pinned manifest (`expected_size` and lowercase SHA-256). Stored in the OS local application data directory (`dirs::data_local_dir()`). Dual-source hosting plan: primary placeholder for project GitHub Release asset (`TODO(owner)`), fallback to upstream HuggingFace export. Pure-Rustls blocking HTTP client (`ureq = "=2.10.1"`). Downloads write to temporary `.part` files, streaming size caps prevent decompression/oversize attacks, and files are atomically renamed only after SHA-256 verification passes. HTTP chunk resume is deferred to the backlog.
- Status: accepted
- Reason: Keeps the v1.0 installer small without bundling hundreds of megabytes of model weights, guarantees privacy (zero network access outside explicit model download), and prevents corruption or tampering via cryptographic checksum verification.
- Alternatives considered: Bundling models in installer (violates size goals), unverified dynamic URLs (security vulnerability), async runtime like Tokio/Reqwest (adds bloat and compile complexity for a one-time sequential task).

### D-020: Batch Processing Architecture, Deterministic Collision Handling, and Memory Bounding
- Date: 2026-11-24
- Decision: Batch folder processing runs strictly sequentially using a single shared ORT session (`Remover`) initialized once. Do not use Rayon or multi-image concurrent thread pools. Peak working set is strictly bounded to exactly 1 full-resolution image in RAM at any given time by immediately dropping and flushing image buffers after writing to disk. Output collision handling resolves deterministically using `<stem>_<ext>.png` when stems conflict within the same directory. Tri-state exit codes: `0` (all ok/skipped), `1` (total failure / bad args), `2` (partial failure).
- Status: accepted
- Reason: Prevents CPU cache thrashing and memory spikes that occur when running multiple high-resolution neural network inferences concurrently on consumer hardware. Guarantees flat, leak-free memory usage across 100+ images while ensuring deterministic, zero-data-loss output naming.
- Alternatives considered: Threadpool/Rayon parallel image queue (exhausts RAM on large images, high contention on ORT intra-op threads), overwriting on collision (data loss risk).

### D-021: Post-Processing Edge Refinement Pipeline (Fast Guided Filter, Selective Feathering, and Color Defringing)
- Date: 2026-11-24
- Decision: Implement an $O(N)$ separable 2D box-filtered Guided Filter ($r = 4, \epsilon = 10^{-4}$) using original grayscale luminance as guidance to align segmentation boundaries with fine strands of hair and subtle edges. Apply selective transition zone feathering locking definite foreground ($\ge 0.95$) to opaque ($255$) and definite background ($\le 0.05$) to transparent ($0$) to prevent interior softness. Implement a $5 \times 5$ boundary color defringing pass that extrapolates adjacent solid foreground RGB into semi-transparent boundary pixels ($\alpha < 235$) to decontaminate background color bleed and eliminate colored halos. Enable refinement by default in `Remover`, with an opt-out CLI flag `--no-refine`.
- Status: accepted
- Reason: Substantially elevates cutout quality on challenging hair, fur, and clothing silhouettes while adding negligible CPU runtime overhead (measured 25 ms on 1MP, 177 ms on 2MP; well below 300 ms budget).
- Owner Approval (2026-11-25): Default refine level `standard` approved by owner after inspecting three desktop app results (`rambut1` Quality, `baju1` Fast, `hewan2` Fast). **Caveat**: Refined `24 MP` outputs and saturated-color backgrounds have **not** yet been reviewed by the owner.
- Alternatives considered: Global Gaussian blur (causes interior softness and boundary bleeding), secondary neural matting network such as ViTMatte (adds hundreds of megabytes of model weight overhead and multi-second latency, violating lightweight offline desktop requirements).

### D-022: Hardware Execution Providers (DirectML/CoreML) and Multi-Stage Graceful CPU Fallback
- Date: 2026-11-24
- Decision: Add optional hardware execution providers (`directml` for Windows DirectX 12 GPUs, `coreml` for Apple Silicon/Neural Engine on macOS) gated behind cargo feature flags. Maintain pure CPU as the default provider (`--device cpu` per D-017). Prebuilt v0.1 release binaries compile CPU-only (`default = []`). Implement two-stage graceful fallback to CPU: (1) during session initialization if the EP cannot be registered or committed, and (2) during inference execution if the EP raises a runtime error (e.g., DirectX `8007000E` out-of-memory during fused graph compilation on low-VRAM GPUs). Before retrying inference on CPU, the failed hardware session is explicitly dropped to release all GPU/system memory buffers back to the OS.
- Status: accepted
- Reason: Unlocks hardware acceleration for models on compatible GPUs (measured 328 ms inference on Fast tier with DirectML vs 464 ms on CPU) while guaranteeing 100% crash-free stability across heterogeneous hardware configurations without fatal aborts.
- Alternatives considered: Crashing on GPU out-of-memory (poor UX), enforcing GPU by default (breaks headless/older machines).

### D-023: Desktop IPC Preview Cache via Scoped Asset Protocol & Per-Tier Session Caching
- Date: 2026-11-25
- Decision: Eliminate Base64 image payloads over Tauri IPC. Write bounded preview PNGs (`<= 1600 px` longest side) to `$APPCACHE/previews/` and serve them via Tauri's `assetProtocol` scoped strictly to `$APPCACHE/previews/**` under a strict CSP (no remote origins, no `unsafe-eval`, no `shell` or `fs` plugins). Maintain one cached `Remover` session per `ModelTier` in `AppState`, reloading only when tier options (`ExecutionProvider`, refine flag, threads) change. Persist user settings atomically to `settings.json` in `app_config_dir`.
- Status: accepted
- Reason: Sending full-resolution 24MP PNGs as Base64 strings over JSON IPC spikes memory and stalls WebView2 serialization, while reloading `BiRefNet` wastes ~3.7s per image. Scoped `asset:` URLs keep IPC messages tiny, enforce least-privilege filesystem access, and clean up automatically on exit.
- Alternatives considered: Base64 data URLs over IPC (excessive RAM/latency on large images), unrestricted `assetProtocol` scope `**/*` (overly broad filesystem exposure to the webview).

### D-024: Desktop UI/UX Architecture, Virtualized Queue, Zero-Dep Testing, and Backlog Scope
- Date: 2026-11-25
- Decision: Implement the PRD Section 8 single-screen layout using pure Svelte 5, system fonts, windowed thumbnail rendering for large queues, zero external i18n/UI libraries, and Node's built-in `node --test` TypeScript runner (`v26.13.0`) for frontend unit tests. Enforce `never_use_network` in Rust alongside SHA-256 verified manual `.onnx` import. Defer `crop to subject` and `WebP / additional export formats` to the post-v1.0 backlog (`docs/TASKS.md`) so Phase 3B exports PNG cleanly without extra scope.
- Status: accepted
- Reason: Keeps desktop dependencies minimal, ensures zero-freeze scrolling on 100+ photo queues, preserves offline guarantees, and avoids unrequested format/crop scope creep before v1.0 stabilization.
- Alternatives considered: Adding Vitest / i18next / Tailwind (unnecessary dependency bloat and license surface).

### D-025: Windows NSIS Installer (`embedBootstrapper`), Single-Source Versioning, and Draft Prereleases
- Date: 2026-11-25
- Decision: Standardize Windows desktop distribution on NSIS (`-setup.exe`, `currentUser` installMode) with `webviewInstallMode: "embedBootstrapper"` (`~1.8 MB`), omit `"version"` from `tauri.conf.json` so `Cargo.toml` `[workspace.package].version` is the single source of truth (enforced alongside `package.json` in CI), build macOS `aarch64` `.dmg` (Apple Silicon only; cheap Intel `x86_64-apple-darwin` cross-target build deferred unless requested) and Linux `ubuntu-22.04` `.AppImage`/`.deb`, upgrade GitHub Actions to Node-24-compatible major versions, and publish `v*` tag builds as **draft prereleases** with consolidated `SHA256SUMS.txt`. No code-signing keys or auto-updater.
- Status: accepted
- Reason: NSIS produces a compact `9.69 MiB` installer without WiX/VBSCRIPT constraints, handles missing WebView2 reliably via `embedBootstrapper` (avoiding ~127 MB offline WebView2 bloat), and draft prereleases prevent unverified tags from going public before checksum/OS inspection.
- Alternatives considered: MSI WiX installer (requires Windows-only WiX + VBSCRIPT feature, larger overhead for single-user desktop installs), `offlineInstaller` WebView2 (~127 MB size penalty), adding an auto-updater (adds unwanted background network checks).

### D-026: Public Community Governance, Dependabot Cadence, and Fixture Provenance Escalation
- Date: 2026-11-25
- Decision: Adopt verbatim Contributor Covenant v2.1 (`CODE_OF_CONDUCT.md`), private security reporting via GitHub Security Advisories (`SECURITY.md`), weekly Dependabot PRs for Cargo, npm (`/crates/app`), and GitHub Actions (`.github/dependabot.yml`), and flag all unverified/human-subject photographic fixtures in `tests/fixtures/PROVENANCE.md` (`TODO(owner)`) for explicit owner review rather than silently deleting or rewriting git history.
- Status: accepted
- Reason: Ensures transparent contributor terms (`GPL-3.0-or-later`), continuous security updates, and protects against accidental redistribution of unverified or identifiable-person photos without taking destructive `git filter-repo` actions without owner consent.
- Alternatives considered: Skipping provenance audit or rewriting git history automatically (risks data loss or force-push conflicts without owner approval).

### D-027: Banded Guided Filter (`GUIDED_BAND_HEIGHT = 128`) and Reverse-DNS Identifier (`io.github.yehudaaadev.rustmatte`)
- Date: 2026-11-25
- Decision: Refactor `crates/core/src/refine.rs` (`guided_filter` and `refine_mask_guided`) to process 128-row horizontal bands with `4 * radius` halos and direct `DynamicImage` pixel reads, eliminating full-frame `Vec<f32>` and `to_rgb8()`/`to_rgba8()` copies. Rename the Tauri bundle identifier from `com.rustmatte.app` to `io.github.yehudaaadev.rustmatte`.
- Status: accepted
- Reason: Cuts 24 MP peak working set by ~1.17 GB (Quality `3,352.68 MB` -> `2,185.15 MB`; Fast `1,660.08 MB` -> `492.68 MB`) and cuts postprocessing time ~2x while matching full-frame math within `1.2e-5` `f32` precision (`<= 1` LSB in 8-bit PNG). Reverse-DNS `io.github.yehudaaadev.rustmatte` resolves macOS `.app` bundle warnings; `dirs::data_local_dir()` model cache is unaffected.
- Alternatives considered: Auto-downscaling large images silently (changes output resolution without consent), `f16` CPU buffers (adds conversion overhead).

### D-028: Compile-Time Dev-URL Guard in Release Desktop Builds and Frontend Startup Verification Signal
- Date: 2026-11-25
- Decision: Keep `default = []` and declare optional `custom-protocol = ["tauri/custom-protocol"]` in `crates/app/src-tauri/Cargo.toml` (preserving `npm run tauri dev` live-reload behavior), add a compile-time `#[cfg(all(not(debug_assertions), dev, not(test)))] compile_error!(...)` guard plus binary `RUSTMATTE_BUILD_MODE` marker in `crates/app/src-tauri/src/lib.rs`, add `notify_frontend_ready` writing `ui_ready mode=... origin=...` to `$APPCACHE/startup.log` (no file paths or user data), and verify `RUSTMATTE_BUILD_MODE=custom-protocol` across all three OS targets in `.github/workflows/release.yml`.
- Status: accepted
- Reason: In pinned `tauri = "=2.12.7"`, `is_dev()` checks `!cfg!(feature = "custom-protocol")`, so plain `cargo build --release -p rustmatte-app` compiled `http://localhost:1420` into the binary and produced WebView2 `ERR_CONNECTION_REFUSED` while appearing alive via process-level checks. Putting `custom-protocol` in `default` would disable `is_dev()` during `npm run tauri dev`, so a compile-time release guard + CI marker check + real UI startup log is safer and unambiguous.
- Alternatives considered: Making `custom-protocol` a default Cargo feature (breaks `npm run tauri dev` `devUrl` loading because Tauri v2 CLI does not pass `--no-default-features` automatically on `tauri dev`), keeping undocumented `cargo build --release` hazard.

### D-029: Cutout-First Comparison Default, Display-Only Path Formatting, and Structured Cross-Layer Localization
- Date: 2026-11-25
- Decision: Default comparison preview to cutout-only (`sliderPos = 0`) whenever a photo finishes or is selected, keeping user-adjusted split position remembered within the session via a `"Compare with original"` toolbar toggle and `C` keyboard shortcut; strip Windows extended-length prefixes (`\\?\` / `\\?\UNC\`) for UI display only while retaining canonical `PathBuf` internally; replace Rust-formatted English banners/warnings/rejection reasons and dialog titles/filters with typed message codes rendered reactively in `en` and natural `id`; and count only saveable finished cutouts (`fullCutoutPath` present) in `Save all (N)`. Keep `device: "auto"` untouched pending explicit owner confirmation on whether to revert desktop default to `"cpu"` per `D-017`/`D-022`.
- Status: accepted
- Reason: Showing the finished cutout first matches user expectation immediately after processing, structured message codes allow live EN/ID switching on active banners without stale frozen strings, and display-only prefix stripping prevents confusing `\\?\C:\...` banners without breaking canonical path checks.
- Alternatives considered: Stripping `\\?\` internally (breaks long Windows path canonical equality checks), leaving `statusNotice` as a pre-formatted string (breaks live language switches).

### D-030: Align Desktop Default Execution Provider with D-017/D-022 (`cpu`)
- Date: 2026-11-25
- Decision: Set `AppSettings::default().device` and frontend initial `selectedDevice` to `"cpu"` (with `CPU only` listed first in the selector), matching `D-017`, `D-022`, `rustmatte-core`, and `rustmatte-cli`.
- Status: accepted
- Reason: Shipped CI/release binaries compile CPU-only (`default = []`, without `directml` or `coreml` features), and DirectML can encounter VRAM OOM (`8007000E`), minor floating-point divergence, or driver issues when enabled without explicit opt-in.
- Alternatives considered: Keeping `"auto"` default in the UI (misleads users when release binaries compile CPU-only and risks driver/VRAM surprises on custom GPU builds).

### D-031: HuggingFace Primary Model Source, Binary-Entrypoint Release Guard, and Ubuntu 24.04 Release Runner
- Date: 2026-11-25
- Decision: (1) Make `huggingface.co` primary (`source_urls[0]`) in `crates/core/src/model.rs` while keeping `models-v1` release assets as secondary mirror, and display only the read-only CPU processing line in the desktop UI (`settings.rs` migrates saved `"auto"` to `"cpu"`). (2) Place the `#[cfg(all(not(debug_assertions), dev, not(test)))] compile_error!(...)` guard in `crates/app/src-tauri/src/main.rs` instead of `lib.rs` so `cargo test --release --workspace` passes while `cargo build --release --workspace` / `npx tauri build` still enforces `custom-protocol`. (3) Update `.github/workflows/release.yml` to locate the bundled macOS executable via `*/Contents/MacOS/*` (`RustMatte`) and run Linux release builds on `ubuntu-24.04`.
- Status: accepted
- Reason: Both `models-v1` GitHub Release URLs currently return `HTTP 404`; putting `compile_error!` in `lib.rs` blocked `cargo test --release --workspace`; `RustMatte.app` names its executable after `productName` (`RustMatte`); and prebuilt `ort = "=2.0.0-rc.15"` static Linux binaries reference `__isoc23_strtoull` / GCC 13+ `libstdc++` symbols unavailable on `ubuntu-22.04` (`glibc 2.35`). Empirical 24-pair regression check (`207ef8d` vs `HEAD`) confirmed `MaxA = 1`, `%>2 = 0.00%`, and identical 128-row seam behavior on `large_24mp.jpg`.
- Alternatives considered: Keeping `models-v1` primary (fails on 404 before fallback), compiling ONNX Runtime from source on `ubuntu-22.04` (high CI build time and complexity).

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
