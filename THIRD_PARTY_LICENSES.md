# Third-Party Licenses Register

Rule: Every external dependency (Rust crate, frontend package, tool) and every model weight file must be registered in this table before use. If an item has no entry here, it is NOT approved for use.

## Rust Dependencies (Core, CLI, and Desktop Backend)

| Name | Type | Version | License | Source URL | Approved by Owner | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `ort` | crate | `2.0.0-rc.15` | MIT OR Apache-2.0 | `https://crates.io/crates/ort` | Yes | Rust bindings for ONNX Runtime with `download-binaries` |
| `image` | crate | `0.25.10` | MIT OR Apache-2.0 | `https://crates.io/crates/image` | Yes | Image decoding and encoding (PNG, JPEG, WebP, BMP) |
| `heic-rs` | crate | `0.1.5` | MIT OR Apache-2.0 | `https://crates.io/crates/heic-rs` | Yes | Pure-Rust HEIC/HEIF still image decoder (`no_std`-friendly, no `unsafe`) |
| `ndarray` | crate | `0.17.2` | MIT OR Apache-2.0 | `https://crates.io/crates/ndarray` | Yes | N-dimensional tensor array manipulation |
| `thiserror` | crate | `2.0.21` | MIT OR Apache-2.0 | `https://crates.io/crates/thiserror` | Yes | Error handling in core library |
| `anyhow` | crate | `1.0.105` | MIT OR Apache-2.0 | `https://crates.io/crates/anyhow` | Yes | Error handling for binaries and examples |
| `sha2` | crate | `0.10.8` | MIT OR Apache-2.0 | `https://crates.io/crates/sha2` | Yes | SHA-256 verification of local model weights in core |
| `clap` | crate | `4.5.31` | MIT OR Apache-2.0 | `https://crates.io/crates/clap` | Yes | Command-line argument parser with derive in CLI binary |
| `ureq` | crate | `2.10.1` | MIT OR Apache-2.0 | `https://crates.io/crates/ureq` | Yes | Pure-Rustls blocking HTTP client for model downloading |
| `dirs` | crate | `6.0.0` | MIT OR Apache-2.0 | `https://crates.io/crates/dirs` | Yes | Standard OS application data directory lookup |
| `sysinfo` | crate | `0.40.4` | MIT | `https://crates.io/crates/sysinfo` | Yes | Local available RAM and disk space detection (`system`, `disk` features only; no network) |
| `tauri` | crate | `2.12.7` | Apache-2.0 OR MIT | `https://crates.io/crates/tauri` | Yes | Tauri 2 desktop application framework (`protocol-asset` enabled) |
| `tauri-build` | crate | `2.7.6` | Apache-2.0 OR MIT | `https://crates.io/crates/tauri-build` | Yes | Tauri 2 build-time codegen and CSP asset hashing |
| `tauri-plugin-dialog` | crate | `2.8.4` | Apache-2.0 OR MIT | `https://crates.io/crates/tauri-plugin-dialog` | Yes | Native OS open/save file dialogs for Tauri 2 |
| `serde` | crate | `1.0.229` | MIT OR Apache-2.0 | `https://crates.io/crates/serde` | Yes | Serialization/deserialization for IPC payloads and settings |
| `serde_json` | crate | `1.0.154` | MIT OR Apache-2.0 | `https://crates.io/crates/serde_json` | Yes | JSON settings persistence |
| `ONNX Runtime` | binary | `1.20+` | MIT | `https://github.com/microsoft/onnxruntime` | Yes | Official Microsoft ONNX Runtime C++ library |

## Frontend Dependencies (`crates/app`)

| Name | Type | Version | License | Source URL | Approved by Owner | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `@tauri-apps/api` | npm (prod) | `2.12.5` | MIT OR Apache-2.0 | `https://www.npmjs.com/package/@tauri-apps/api` | Yes | Tauri 2 IPC, events, webview drag-drop, and `convertFileSrc` |
| `@tauri-apps/plugin-dialog` | npm (prod) | `2.8.4` | Apache-2.0 OR MIT | `https://www.npmjs.com/package/@tauri-apps/plugin-dialog` | Yes | JS bindings for native open/save dialogs |
| `svelte` | npm (dev/compile) | `5.59.1` | MIT | `https://www.npmjs.com/package/svelte` | Yes | Compiled reactive UI framework (zero runtime bloat) |
| `@sveltejs/kit` | npm (dev/compile) | `2.70.4` | MIT | `https://www.npmjs.com/package/@sveltejs/kit` | Yes | Static app bundler with `@sveltejs/adapter-static` `3.0.11` |
| `@sveltejs/vite-plugin-svelte` | npm (dev) | `7.3.2` | MIT | `https://www.npmjs.com/package/@sveltejs/vite-plugin-svelte` | Yes | Vite integration for Svelte 5 |
| `vite` | npm (dev) | `8.3.8` | MIT | `https://www.npmjs.com/package/vite` | Yes | Frontend build bundler |
| `typescript` | npm (dev) | `6.0.3` | Apache-2.0 | `https://www.npmjs.com/package/typescript` | Yes | TypeScript compiler (pinned for `svelte-check` compatibility) |
| `svelte-check` | npm (dev) | `4.7.8` | MIT | `https://www.npmjs.com/package/svelte-check` | Yes | Svelte/TS type checker |
| `@tauri-apps/cli` | npm (dev) | `2.12.6` | Apache-2.0 OR MIT | `https://www.npmjs.com/package/@tauri-apps/cli` | Yes | Tauri desktop CLI tooling |
| `license-checker-rseidelsohn` | npm (dev) | `5.0.1` | BSD-3-Clause | `https://www.npmjs.com/package/license-checker-rseidelsohn` | Yes | Automated npm license audit tool (dev/CI only) |

Note on transitive npm dev-dependencies: `license-checker-rseidelsohn --summary` reports `BlueOak-1.0.0` (12 build tooling crates by `isaacs`: `glob`, `tar`, `lru-cache`, etc.), `MPL-2.0` (`lightningcss` build tool), and `CC-BY-3.0` / `(MIT AND CC-BY-3.0)` (`spdx-exceptions`, `spdx-ranges` inside the license checker itself). Production runtime npm dependencies (`--production`) consist solely of `@tauri-apps/api` (`MIT OR Apache-2.0`) and `@tauri-apps/plugin-dialog` (`Apache-2.0 OR MIT`).

## Model Weights and Dataset Provenance Audit

| Model Candidate | Weight License | Dataset Provenance | Dataset License Terms | Risk Note (for Owner Evaluation) | Source URL |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`BiRefNet_lite` (512 & 1024)** | **MIT** (explicitly granted by author ZhengPeng7) | DIS5K (DIS-TR), DUTS (DUTS-TR), HRSOD, UHRSD, COD10K | Both DIS5K and DUTS specify academic/non-commercial research use | While authors released weights under MIT, underlying datasets carry non-commercial research terms. Whether model training constitutes non-infringing transformative fair use or inherits dataset restrictions is legally unsettled across jurisdictions. Flagged as moderate legal uncertainty. | Model: `https://github.com/ZhengPeng7/BiRefNet` ; Data: `https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf` |
| **`U-2-Netp` (320)** | **Apache-2.0** (explicitly granted by author xuebinqin) | DUTS (DUTS-TR, 10,553 images from ImageNet DET / SUN) | Academic / non-commercial research use | Identical dataset provenance risk to BiRefNet: author released weights and code under Apache-2.0, but training data (DUTS) carries academic research terms. Flagged as moderate legal uncertainty. | Model: `https://github.com/xuebinqin/U-2-Net` ; Data: `http://saliencydetection.net/duts/` |
| **`IS-Net` (`isnet_general_use`)** | **Ambiguous / Apache-2.0 code** | DIS5K (DIS-TR, 3,000 images) | Academic / non-commercial research only | High risk: author explicitly includes non-commercial dataset license alongside the model repository without separate commercial weight grant. Dropped from consideration per owner decision D-018. | Model: `https://github.com/xuebinqin/DIS` ; Data: `https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf` |
| **`BEN2 Base` (`ben2_fp32`)** | **MIT** (upstream `PramaLLC/BEN2` Base) / **Apache-2.0** (FP32 ONNX `Pezhgorski/BEN2-FP32-ONNX`) | DIS5K + 22K proprietary segmentation dataset | DIS5K: academic/research terms; proprietary 22K: author-owned training set | Open-source BEN2 Base weights (MIT); FP32 ONNX converted from `onnx-community/BEN2-ONNX`. Users should be aware of DIS5K academic dataset terms. | Model: `https://huggingface.co/PramaLLC/BEN2` ; ONNX: `https://huggingface.co/Pezhgorski/BEN2-FP32-ONNX` |
