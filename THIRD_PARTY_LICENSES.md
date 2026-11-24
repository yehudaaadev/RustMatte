# Third-Party Licenses Register

Rule: Every external dependency (Rust crate, frontend package, tool) and every model weight file must be registered in this table before use. If an item has no entry here, it is NOT approved for use.

## Dependencies

| Name | Type | Version | License | Source URL | Approved by Owner | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `ort` | crate | `2.0.0-rc.15` | MIT OR Apache-2.0 | `https://crates.io/crates/ort` | Yes | Rust bindings for ONNX Runtime with `download-binaries` |
| `image` | crate | `0.25.10` | MIT OR Apache-2.0 | `https://crates.io/crates/image` | Yes | Image decoding and encoding (PNG, JPEG, WebP, BMP) |
| `ndarray` | crate | `0.17.2` | MIT OR Apache-2.0 | `https://crates.io/crates/ndarray` | Yes | N-dimensional tensor array manipulation |
| `thiserror` | crate | `2.0.21` | MIT OR Apache-2.0 | `https://crates.io/crates/thiserror` | Yes | Error handling in core library |
| `anyhow` | crate | `1.0.105` | MIT OR Apache-2.0 | `https://crates.io/crates/anyhow` | Yes | Error handling for binaries and examples |
| `sha2` | crate | `0.10.8` | MIT OR Apache-2.0 | `https://crates.io/crates/sha2` | Yes | SHA-256 verification of local model weights in core |
| `clap` | crate | `4.5.31` | MIT OR Apache-2.0 | `https://crates.io/crates/clap` | Yes | Command-line argument parser with derive in CLI binary |
| `ONNX Runtime` | binary | `1.20+` | MIT | `https://github.com/microsoft/onnxruntime` | Yes | Official Microsoft ONNX Runtime C++ library |

## Model Weights and Dataset Provenance Audit

| Model Candidate | Weight License | Dataset Provenance | Dataset License Terms | Risk Note (for Owner Evaluation) | Source URL |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`BiRefNet_lite` (512 & 1024)** | **MIT** (explicitly granted by author ZhengPeng7) | DIS5K (DIS-TR), DUTS (DUTS-TR), HRSOD, UHRSD, COD10K | Both DIS5K and DUTS specify academic/non-commercial research use | While authors released weights under MIT, underlying datasets carry non-commercial research terms. Whether model training constitutes non-infringing transformative fair use or inherits dataset restrictions is legally unsettled across jurisdictions. Flagged as moderate legal uncertainty. | Model: `https://github.com/ZhengPeng7/BiRefNet` ; Data: `https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf` |
| **`U-2-Netp` (320)** | **Apache-2.0** (explicitly granted by author xuebinqin) | DUTS (DUTS-TR, 10,553 images from ImageNet DET / SUN) | Academic / non-commercial research use | Identical dataset provenance risk to BiRefNet: author released weights and code under Apache-2.0, but training data (DUTS) carries academic research terms. Flagged as moderate legal uncertainty. | Model: `https://github.com/xuebinqin/U-2-Net` ; Data: `http://saliencydetection.net/duts/` |
| **`IS-Net` (`isnet_general_use`)** | **Ambiguous / Apache-2.0 code** | DIS5K (DIS-TR, 3,000 images) | Academic / non-commercial research only | High risk: author explicitly includes non-commercial dataset license alongside the model repository without separate commercial weight grant. Dropped from consideration per owner decision D-018. | Model: `https://github.com/xuebinqin/DIS` ; Data: `https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf` |

