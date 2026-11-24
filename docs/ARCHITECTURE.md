# RustMatte Architecture Map

Status: Planned (No implementation code exists yet). Derived strictly from `docs/PRD.md` Sections 5, 6, 14, and Owner Corrections.

## Workspace Layout
```text
RustMatte/
├── Cargo.toml                 (workspace root, planned)
├── rust-toolchain.toml        (pinned toolchain, planned)
├── crates/
│   ├── core/                  (engine library, planned)
│   ├── cli/                   (command-line interface binary, planned)
│   └── app/                   (Tauri 2 desktop application, planned)
├── web/                       (WASM/PWA build for v2, planned)
├── tests/                     (integration tests and fixtures, planned)
├── docs/                      (architecture and task documentation)
├── .agents/                   (rules, skills, and workflows)
├── THIRD_PARTY_LICENSES.md    (license registry)
└── README.md                  (project overview, planned)
```

## Crate Responsibilities
- `crates/core` (planned): Model manager (first-run download, SHA-256 verification, caching), pre-processing (aspect-ratio resize, normalization), ONNX inference, post-processing (guided-filter edge refinement, defringing, alpha blending), and custom error types.
- `crates/cli` (planned): Command-line interface using `clap` (derive). Thin wrapper around `crates/core`.
- `crates/app` (planned): Desktop application using Tauri 2 with a web frontend (Svelte/TypeScript). Thin wrapper around `crates/core`.
- `web/` (planned v2): WebAssembly and PWA implementation for browser-based offline processing.

## Dependency Rule
- `crates/core` must contain NO UI code and NO printing to stdout/stderr. It returns strongly typed `Result<T, E>`.
- `crates/cli` and `crates/app` are thin consumers of `crates/core`.
- `crates/core` exposes a small, stable public API (e.g., `Remover::new(config)`, `remover.process(image)`, `remover.process_batch(paths, options, callback)`).

## Processing Pipeline
1. Decode image, apply EXIF orientation metadata (v1.0 supported: JPEG, PNG, WebP, BMP; unsupported formats return clear error, never crash).
2. Optional fast mode: downscale very large images early to save decode time and RAM (model input is fixed, so inference time is unchanged).
3. Resize to model input dimensions, preserving aspect ratio.
4. Normalize image pixel data to tensor shape and format expected by model.
5. Run ONNX inference using CPU as default (optional hardware execution providers DirectML on Windows, CoreML on macOS, with automatic CPU fallback).
6. Extract probability mask and upscale back to target dimensions.
7. Refine mask: guided-filter refinement against original photo, feathering only in transition zone, and edge color cleanup (defringing).
8. Apply mask as alpha channel, export transparent 32-bit PNG at specified resolution.

## Concurrency & Memory Guardrails
- Bounded batch queue: only 1-2 full-resolution images decoded simultaneously in RAM.
- Immediate deallocation: raw image buffers dropped immediately after saving to disk.

## Model Storage and Download Flow
- Models are never committed to git and never bundled into the v1.0 installer.
- On first launch, the required model downloads to the user's OS application data directory:
  - Windows: `%LOCALAPPDATA%\RustMatte\models\` (planned)
  - macOS: `~/Library/Application Support/RustMatte/models/` (planned)
  - Linux: `~/.local/share/RustMatte/models/` (planned)
- Every downloaded model file is verified with SHA-256 before use.
- The application is 100% offline once the model file is cached.

## Platform Strategy Summary
- Windows: Primary target for v1.0 desktop app via Tauri 2; full manual testing. Installer handles WebView2 bootstrapper.
- macOS and Linux: Experimental for v1.0; built and tested in CI from Phase 0.
- Web Browser (WASM/PWA): Planned for v2.0 to support mobile devices and tablets locally.
- Android: Planned for v3.0 after desktop application is complete and stable.
