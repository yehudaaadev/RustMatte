# External Technical References

This directory indexes official, authoritative external documentation required for development.
Rule: Do not fabricate content or write summaries from memory. When a reference is needed, fetch it from the official source, verify it against pinned versions, and link or save it here.

## References to Collect During Phase 0

### 1. Pinned `ort` Crate Documentation
- What it is: Official documentation and API reference for the pinned version of the `ort` Rust crate.
- Why it matters: Prevents compilation failures and API guesswork during ONNX session setup and tensor creation.
- Source URL: `https://docs.rs/ort`
- Status: not collected yet

### 2. Candidate Model Specifications (BiRefNet_lite and ISNet)
- What it is: Official repositories, licenses, input/output tensor specifications, and documented ONNX exports for candidate models.
  - Candidate A: `BiRefNet_lite` (MIT license, Swin-T backbone; official repo: `https://github.com/ZhengPeng7/BiRefNet`).
  - Candidate B: `ISNet` (Dichotomous Image Segmentation; verify primary source weight license: `https://github.com/xuebinqin/DIS`).
- Why it matters: Required to evaluate ONNX operator support (e.g. checking for unsupported operators like deformable convolution on CPU) and license validity before Phase 1 benchmark.
- Source URL: `TODO(owner): Confirm verified ONNX export URLs and SHA-256 hashes for BiRefNet_lite and ISNet`
- Status: not collected yet

### 3. Tauri 2 Desktop Documentation
- What it is: Authoritative guides for Tauri 2 IPC commands, window event lifecycle, file drag-and-drop APIs, NSIS bundling, and WebView2 bootstrapper.
- Why it matters: Ensures desktop integration follows current Tauri 2 patterns rather than obsolete Tauri 1.x patterns.
- Source URL: `https://v2.tauri.app/`
- Status: not collected yet
