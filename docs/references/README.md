# External Technical References

This directory indexes official, authoritative external documentation required for development.
Rule: Do not fabricate content or write summaries from memory. When a reference is needed, fetch it from the official source, verify it against pinned versions, and link or save it here.

## References Collected for Phase 1A

### 1. Pinned `ort` Crate Documentation (v2.0.0-rc.15)
- What it is: Official documentation and API reference for `ort = "=2.0.0-rc.15"`.
- Why it matters: Pinned version ensures API stability across session setup, execution providers, and tensor views.
- Source URL: `https://docs.rs/ort/2.0.0-rc.15`
- Linking Strategy: `features = ["download-binaries", "copy-dylibs"]`. The build requires initial internet access to fetch the official precompiled Microsoft ONNX Runtime C++ dynamic libraries (`onnxruntime.dll`). At runtime, `copy-dylibs` places `onnxruntime.dll` directly into `target/debug/` adjacent to the executable.
- Status: Collected and verified

### 2. Candidate Model: `BiRefNet_lite` ONNX Export (1024 FP32 - Baseline Reference)
- What it is: Lightweight Bilateral Reference model with Swin-T backbone for high-resolution background removal.
- Repository: `https://github.com/ZhengPeng7/BiRefNet`
- Export Source URL: `https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model.onnx`
- Weight License: MIT (explicitly granted by author ZhengPeng7 in model card and repo).
- Dataset Provenance: DIS5K (DIS-TR, 3,000 images), DUTS (DUTS-TR, 10,553 images), HRSOD, COD10K, and UHRSD.
- Dataset License Terms: Both DIS5K and DUTS datasets specify academic/non-commercial research use (`https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf`, `http://saliencydetection.net/duts/`).
- Dataset Provenance Risk Note: Author explicitly released model weights under MIT. However, underlying training datasets contain non-commercial research restrictions. Whether model weights inherit dataset restrictions or constitute non-infringing transformative fair use is legally unsettled across jurisdictions. Flagged as moderate legal uncertainty for owner evaluation.
- File size: 224,005,088 bytes (approx. 213.63 MB)
- SHA-256: `5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333`
- Preprocessing and Output Specifications:
  - Input resolution: 1024x1024 RGB (`config.py: self.size = (1024, 1024)`)
  - Image normalization: ImageNet standard mean `[0.485, 0.456, 0.406]`, std `[0.229, 0.224, 0.225]`
  - Output format: Raw logits; sigmoid activation required (`prob = 1.0 / (1.0 + exp(-logit))`)
- Status: Verified. Deferred (not default) due to memory footprint.

### 3. Candidate Model: `BiRefNet_lite` ONNX Export (512 FP32)
- What it is: Downscaled 512x512 resolution export of BiRefNet_lite for reduced memory and faster CPU latency.
- Export Source URL: `https://huggingface.co/studioludens/birefnet-lite-512/resolve/main/onnx/model.onnx`
- Weight License: MIT
- Dataset Provenance: Same as 1024 baseline (DIS5K, DUTS, HRSOD, COD10K, UHRSD).
- Dataset License Terms: Academic/non-commercial research use in training sets.
- Dataset Provenance Risk Note: Author released weights under MIT; dataset provenance carries moderate legal uncertainty regarding non-commercial training data clauses.
- File size: 191,877,254 bytes (approx. 182.99 MB)
- SHA-256: `1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2`
- Preprocessing: 512x512 RGB, ImageNet mean/std normalization. Output requires sigmoid.
- Status: Verified. Default Quality Tier.

### 4. Candidate Model: `BiRefNet_lite` ONNX Export (512 FP16)
- What it is: Half-precision floating-point export of BiRefNet_lite 512.
- Export Source URL: `https://huggingface.co/studioludens/birefnet-lite-512/resolve/main/onnx/model_fp16.onnx`
- License: MIT
- File size: 98,484,532 bytes (approx. 93.92 MB)
- SHA-256: `eff9216bb2f9d3f023d9c2b7196845a7485739ab1f231593633e4d2344ffc516`
- Preprocessing: 512x512 RGB, ImageNet mean/std. Note: ONNX Runtime CPU constant folding emits missing CPU kernel warnings on FP16 nodes.
- Status: Verified

### 5. Candidate Model: `IS-Net` General Use (1024 FP32)
- What it is: Highly Accurate Dichotomous Image Segmentation (ECCV 2022).
- Repository: `https://github.com/xuebinqin/DIS` (Code Apache-2.0)
- Export Source URL: `https://huggingface.co/SacredNoir/isnet-general-use-onnx/resolve/main/isnet-general-use.onnx`
- Weight License: Ambiguous. Checkpoints released with repository without distinct commercial weight grant.
- Dataset Provenance: DIS5K (`DIS-TR`, 3,000 images).
- Dataset License Terms: Strict academic and non-commercial research use only (`https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf`).
- Dataset Provenance Risk Note: High commercial risk. Upstream dataset terms restrict use to non-commercial academic research. Dropped from consideration per owner decision D-018 until license question is resolved.
- File size: 176,213,804 bytes (approx. 168.05 MB)
- SHA-256: `4c56bbc21588459dda11efba5a4a8ee163969da109ae170fb1988c1c2ea4a90a`
- Preprocessing: 1024x1024 RGB, normalized with mean `[0.5, 0.5, 0.5]` and std `[1.0, 1.0, 1.0]` (`(px / 255.0 - 0.5)`). Output contains embedded Sigmoid activation.
- Status: Dropped from consideration (D-018).

### 6. Candidate Model: `IS-Net` General Use (1024 INT8)
- What it is: Quantized INT8 representation of IS-Net.
- Export Source URL: `https://huggingface.co/SacredNoir/isnet-general-use-onnx/resolve/main/isnet-general-use-q8.onnx`
- Status: Dropped (slower on CPU, D-018).

### 7. Candidate Model: `U-2-Netp` (320 FP32)
- What it is: Extremely lightweight salient object detection network (~4.4 MB).
- Repository: `https://github.com/xuebinqin/U-2-Net`
- Export Source URL: `https://huggingface.co/edgetools/u2netp/resolve/main/u2netp.onnx`
- Weight License: Apache-2.0 (explicitly granted by author Xuebin Qin in repository).
- Dataset Provenance: DUTS (`DUTS-TR`, 10,553 images, augmented to 21,106 images).
- Dataset License Terms: Academic/non-commercial research use (`http://saliencydetection.net/duts/`).
- Dataset Provenance Risk Note: Author explicitly released code and weights under Apache-2.0. However, the underlying DUTS dataset was provided for academic research. Shares the same moderate legal uncertainty as BiRefNet regarding non-commercial training data clauses. Flagged for owner evaluation.
- File size: 4,574,861 bytes (approx. 4.36 MB)
- SHA-256: `309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8`
- Preprocessing: 320x320 RGB, ImageNet mean/std. Output requires min-max normalization: `(val - min) / (max - min)`.
- Status: Verified. Fast Preview Tier (D-018).

### 8. Tauri 2 Desktop Documentation
- What it is: Authoritative guides for Tauri 2 IPC commands, window event lifecycle, file drag-and-drop APIs, NSIS bundling, and WebView2 bootstrapper.
- Why it matters: Ensures desktop integration follows current Tauri 2 patterns rather than obsolete Tauri 1.x patterns.
- Source URL: `https://v2.tauri.app/`
- Status: not collected yet (Phase 3)
