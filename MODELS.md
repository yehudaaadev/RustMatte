# RustMatte Model Attribution and Licenses

RustMatte uses offline neural network weights for image matting and background removal.
Weights are downloaded on demand on first run with user consent and cached locally.

## Quality Tier: BiRefNet Lite 512

- Model Architecture: BiRefNet Lite (Bilateral Reference Network)
- Export Variant: 512x512 FP32 ONNX
- Author / Repository: ZhengPeng7 (https://github.com/ZhengPeng7/BiRefNet)
- Export Source: studioludens/birefnet-lite-512 on HuggingFace
- Expected File Size: 191,877,254 bytes
- Pinned SHA-256: `1cb0fb360dadd15af77c639085d77a9df67db0c64315560c3de005f676345ac2`
- Weight License: MIT License (Copyright (c) 2024 ZhengPeng7)
- Dataset Provenance: DIS5K, DUTS, HRSOD, COD10K, and UHRSD.
- Dataset Terms Notice: Training datasets (such as DIS5K and DUTS) include academic/research non-commercial clauses. Model weights were published under the MIT license by the author. Whether training data terms restrict downstream model weights is legally unsettled across jurisdictions; this notice is informational and does not constitute legal advice.

## Fast Tier: U-2-Netp

- Model Architecture: U-2-Netp (Lightweight Salient Object Detection)
- Export Variant: 320x320 FP32 ONNX
- Author / Repository: Xuebin Qin (https://github.com/xuebinqin/U-2-Net)
- Export Source: edgetools/u2netp on HuggingFace
- Expected File Size: 4,574,861 bytes
- Pinned SHA-256: `309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8`
- Weight License: Apache License 2.0 (Copyright (c) 2020 Xuebin Qin)
- Dataset Provenance: DUTS (DUTS-TR).
- Dataset Terms Notice: DUTS training dataset specifies non-commercial academic research use. Model code and weights were published under Apache-2.0 by the author. Subject to the same legal uncertainty noted above.
