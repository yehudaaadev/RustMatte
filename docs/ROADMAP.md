# RustMatte Roadmap

This roadmap reflects planned milestones following the `v0.1.0` pre-release and `v1.0` stabilization. Scope focuses on offline-first, privacy-preserving image workflows without telemetry or cloud dependencies.

## v1.0 (Current Stabilization Target)
- Windows desktop installer (NSIS with WebView2 `embedBootstrapper`) verified on clean consumer hardware.
- Experimental macOS (`aarch64` `.dmg`) and Linux (`.AppImage`, `.deb`) builds verified via tagged CI runs and community testing.
- Complete documentation, provenance audit, and license compliance (`GPL-3.0-or-later`).

## v1.1 (Expanded Camera & Mobile Photo Input)
- **Apple HEIC / HEIF decoding** with cross-platform CI build validation.
- **Camera RAW preview extraction** (`.CR2`, `.NEF`, `.ARW`, `.DNG`): Phase 1 extracts embedded JPEG previews; full RAW demosaicing evaluated separately.

## v1.x (Workflow Polish & Hardware Coverage)
- **Crop-to-subject** export option in desktop UI.
- **Additional export formats** (WebP and configurable JPEG/solid backgrounds alongside PNG).
- **Resumable model downloads** (HTTP range resume for unstable connections).
- **Benchmarks on wider hardware** (Apple Silicon, Linux CPUs, low-RAM 8 GB laptops, DirectML/CoreML GPUs).
- Optional manual refine brush and preset marketplace/passport canvas dimensions.

## v2.0 (Upscaling & Browser Offline App)
- **Local AI Upscaler** (Real-ESRGAN ONNX with image tiling and strict memory cap).
- **Browser / PWA version** (`web/` using WASM / WebGPU) for offline processing on tablets and phones without installation.

## v3.0 (Native Mobile)
- **Android application** (after desktop and WASM pipelines are stable).
