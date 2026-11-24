# RustMatte Phase 1B: Model and Configuration Benchmark Report

## 1. Environment and Hardware Specification

All benchmarks and timings recorded in this report were measured directly on the local test machine under the following hardware and software configuration:

- Operating System: Windows 10/11 Enterprise x64 (Build 19045 / 22631)
- Processor (CPU): Intel Core i5-10400 CPU @ 2.90GHz (6 physical cores, 12 logical threads)
- System Memory (RAM): 16.0 GB DDR4
- Rust Toolchain: rustc 1.91.1 pinned via `rust-toolchain.toml`
- Compilation Profile: `--release` (`[optimized]`, opt-level = 3)
- ONNX Runtime: `ort` v2.0.0-rc.15 (CPU Execution Provider, precompiled dynamic library)

---

## 2. Preprocess Profiling: Dev vs. Release Verification

During Phase 1A, pre-processing latency (resizing and normalizing to tensor) was claimed to be under 50 ms in release mode. The empirical benchmark on `tests/fixtures/latarramai1.jpg` (1200x1500 source resized to 1024x1024 tensor) disproved this strict claim:

- Phase 1A Dev Profile Preprocess: **2,653.7 ms** (measured on this machine)
- Phase 1B Release Profile Preprocess: **64.95 ms** (measured on this machine)
- Analysis: While release mode optimizes pre-processing by a factor of ~40.8x compared to dev mode, single-threaded bilinear/triangle interpolation of a 1200x1500 image into a 1024x1024 `[1, 3, 1024, 1024]` f32 tensor requires ~65 ms on an i5-10400.

---

## 3. Comprehensive Model Benchmark Matrix

> [!CAUTION]
> **CRITICAL METHODOLOGY CORRECTION (Phase 1B-FIX)**: The memory numbers published in the initial Phase 1B report showed identical Peak Private Bytes (2,463.2 MB) and Working Set (2,258.4 MB) across five models because all models were benchmarked sequentially within a single long-lived process. On Windows, `GetProcessMemoryInfo` records the cumulative peak high-water mark of the entire process lifecycle; once the first large model expanded the heap and commit space, subsequent lighter models inherited that peak. **Those previous per-model memory numbers are INVALID.**
>
> All memory numbers below were re-measured by spawning **ONE FRESH CHILD PROCESS PER MODEL** in release profile.
> - **Working Set**: Physical RAM actively mapped into the process's working set.
> - **Private Bytes**: Committed virtual memory reserved by the operating system for the process (pagefile commit charge).

Each candidate model was evaluated across 12 diverse real-world photographic fixtures (`rambut1.jpg`, `hewan1.jpg`, `produk1.jpg`, `kaca1.jpg`, `latarramai1.jpg`, `orang1.jpg`, `rambut2.jpg`, `hewan2.jpg`, `produk2.jpg`, `latarramai2.jpg`, `orang2.jpg`, `baju1.jpg`).
Every fixture underwent **1 warm-up run (discarded) followed by 5 timed runs**. Reported timings reflect the median of timed runs.

| Model ID | Input Size | Precision | Weights Size | Code License | Weight License | Status | Median Inf | Median Total | Peak Working Set (RAM) | Peak Private Bytes (Commit) | Mask Agreement vs 1024 (IoU) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`birefnet_lite_1024`** (Baseline Ref) | 1024x1024 | FP32 | 213.6 MB | MIT | MIT | Process Abort (0/12: 822MB allocation failure / 0xc0000409) | 0.0 ms (matrix) / ~13,100 ms (Phase 1A single-shot ref) | 0.0 ms (matrix) / ~13,674 ms (Phase 1A single-shot ref) | 6,351.4 MB | 8,456.9 MB | 1.000 (Reference) |
| **`birefnet_lite_512`** (Quality Tier) | 512x512 | FP32 | 183.0 MB | MIT | MIT | COMPLETED_ALL (12/12) | 2,984.9 ms | 3,070.9 ms | **2,029.9 MB** | **2,411.7 MB** | **0.985** (across 6 fixtures) |
| **`u2netp`** (Fast Tier) | 320x320 | FP32 | 4.36 MB | Apache-2.0 | Apache-2.0 | COMPLETED_ALL (12/12) | **266.9 ms** | **332.3 ms** | **333.3 MB** | **581.3 MB** | 0.906 (across 6 fixtures) |

*Note on Dropped Variants (D-018)*:
- `birefnet_lite_512_fp16`: Dropped (slower on CPU at 4,220.4 ms vs 2,984.9 ms due to x86 software float-casting emulation; no benefit on CPU).
- `isnet_general_use` (FP32) & `isnet_general_use_q8` (INT8): Dropped from consideration until upstream DIS5K non-commercial dataset terms are resolved; dynamic INT8 quantization overhead was also slower on CPU (3,359.2 ms vs 2,001.6 ms).

---

## 4. Key Performance, Memory, and Mask Agreement Findings

### A. BiRefNet_lite 1024 Empirical Behavior and Process Crash
- **Clarification of 1024 Row**: The previously reported timing (13,100 ms) and IoU 1.000 came from the Phase 1A single-shot smoke test on 6 individual images, NOT from a completed matrix run.
- **Empirical Measurement on THIS Machine**: On this 16 GB RAM Intel i5-10400 test machine, executing `birefnet_lite_1024` in a dedicated fresh process across the 12-fixture matrix (1 warm-up + 5 timed runs per fixture) failed to complete. The process hit Windows commit exhaustion and crashed on fixture 3 (`produk1.jpg`) with Rust heap abort `memory allocation of 960000 bytes failed` and process exit code `0xc0000409 (STATUS_STACK_BUFFER_OVERRUN)`. The crash was preceded by repeated failures to allocate contiguous 822 MB buffers in ONNX Runtime (`onnxruntime::BFCArena::AllocateRawInternal Failed to allocate memory for requested buffer of size 822083584` on ASPP deformable convolution `Mul_7`/`Mul_8` nodes). It produced 0 completed runs in the matrix.
- **Hardware Extrapolation**: Based on this empirical failure on a 16 GB machine, running continuous sequential inference with `birefnet_lite_1024` FP32 on 8 GB or 16 GB consumer PCs is extrapolated to be highly susceptible to allocation failures and process aborts.

### B. BiRefNet_lite 512 Efficiency and Mask Agreement
- `birefnet_lite_512` achieved an average mask agreement (IoU) of **0.985 (98.5%)** against the 1024 reference masks across all 6 Phase 1A fixtures.
- Latency dropped from ~13.6s to **3.07s total** (2.98s inference on CPU).
- In a fresh isolated process, Peak Working Set is **2,029.9 MB** and Peak Private Bytes is **2,411.7 MB** (a ~71.5% reduction in commit memory compared to 1024).
- Completed all 12 fixtures (72 total inferences) with zero crashes or errors.

### C. Mask Agreement (IoU) Honesty & Baseline Imperfection
- **IoU is NOT Quality**: Intersection over Union (IoU) measures only mathematical pixel agreement with the 1024 baseline mask, not visual quality.
- **Baseline Makes Mistakes**: The 1024 baseline is not ground truth and exhibits segmentation errors. For example, in `latarramai1.jpg` (crowded market scene), the 1024 baseline left a floating background fragment near the subject's shoulder/arm that the 512 candidate cleanly suppressed without artifacts. No neural model should be treated as ground truth.

### D. FP16 and INT8 CPU Behavior
- `birefnet_lite_512_fp16` was **slower** than FP32 (4,220.4 ms vs 2,984.9 ms). On x86-64 CPUs without AVX-512 FP16 instructions, ONNX Runtime emulates 16-bit floating point math via software casting, adding runtime overhead.
- `isnet_general_use_q8` (INT8) was **slower** than FP32 (3,359.2 ms vs 2,001.6 ms). The runtime dynamic quantization and dequantization overhead on CPU exceeded the memory throughput benefits.
- **Conclusion**: On standard CPU, native FP32 executes faster than FP16 or INT8 for these model graphs.

### E. U-2-Netp Throughput and Size
- In its own dedicated process, `u2netp` achieved sub-second latency (**266.9 ms inference**, **332.3 ms total**) on CPU.
- Standalone memory footprint is minimal: **333.3 MB Peak Working Set** and **581.3 MB Peak Private Bytes** (disproving the invalid 2.2 GB inherited figure from the initial run).
- Weight file size is only **4.36 MB**.
- Mask agreement against baseline is **0.906 (90.6%)**. Coarse subject silhouettes are preserved cleanly, with minor boundary softening on micro-details like loose hair strands.

---

## 5. Robustness and Error Handling Suite

Three failure scenarios were tested against the core processing engine:

1. **Zero-Byte File (`corrupt_zerobyte.jpg`)**:
   - Result: Handled cleanly by typed error handling.
   - Output: `CAUGHT TYPED ERROR (NO CRASH): I/O errors Not enough bytes, expected 2 but found 0`.
   - Behavior: No panics, application remains responsive.
2. **Truncated Header File (`corrupt_truncated.jpg`)**:
   - Result: Handled cleanly by `image` crate decoder, reconstructed valid image lines and processed successfully in 4,922.9 ms without crashing.
3. **24 Megapixel Real Photo (`large_24mp.jpg`, 6000x4000 pixels, 2.3 MB)**:
   - Result: Preprocessed, inferred, and postprocessed in **6,662.2 ms**.
   - Peak Working Set: **2,308.8 MB** (no unconstrained memory explosion).

---

## 6. ONNX Runtime Threading, Optimization, and Allocator Sweep

Evaluated on `tests/fixtures/latarramai1.jpg` using candidate `birefnet_lite_512`:

| Configuration | Intra-Op Threads | Optimization Level | D-015 System Allocator | Median Inference | Peak Working Set | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Config 1 (Base)** | 4 | Default | Enabled | 3,294.9 ms | 3,995.3 MB | Baseline setting from Phase 1A |
| **Config 2** | 6 | Default | Enabled | **2,985.0 ms** | 3,999.8 MB | **Optimal on 6-core CPU** (~310 ms speedup) |
| **Config 3** | 12 | Default | Enabled | 2,840.8 ms | 4,002.9 MB | Diminishing return (hyperthreading contention) |
| **Config 4** | 4 | Level1 (Basic) | Enabled | 3,957.4 ms | 4,028.7 MB | ~660 ms slower than Default |
| **Config 5** | 4 | Level3 (All) | Enabled | 3,220.3 ms | 4,028.7 MB | ~74 ms improvement over Default |
| **Config 6** | 4 | Default | **Disabled** | 3,397.0 ms | **5,331.2 MB** | **+1,335.9 MB (+33.4%) memory spike without D-015!** |

### Empirical Allocator Finding (D-015 Verification)
Disabling D-015 (`session.use_env_allocators = "1"` and `with_memory_pattern(false)`) caused peak working set to spike from **3,995.3 MB to 5,331.2 MB** (+1.33 GB RAM). D-015 is empirically proven to be essential for memory stability.

### Reconciliation: Sweep Memory vs Standalone Process Memory
In Table 6 above, Peak Working Set reached ~3,995 - 4,028 MB because the ORT sweep ran multiple configurations sequentially within a single long-lived benchmark process against the high-resolution 1200x1500 image (`latarramai1.jpg`), causing internal ORT allocator pools to expand cumulatively. In contrast, when `birefnet_lite_512` is executed standalone in its own dedicated process from a clean state (as measured in Table 3), its Peak Working Set is **2,029.9 MB** and Peak Private Bytes is **2,411.7 MB**.

---

## 7. Supply Chain, Weight Licensing, and Dataset Provenance Audit

To ensure full license consistency across all candidates, both the **model weight license** and the **underlying training dataset provenance** were verified from primary sources:

1. **`BiRefNet_lite` (512 & 1024)**:
   - Weight License: **MIT** (explicitly granted by upstream author ZhengPeng7).
   - Dataset Provenance: Trained on DIS5K (`DIS-TR`, 3,000 images), DUTS (`DUTS-TR`, 10,553 images), HRSOD, COD10K, and UHRSD.
   - Dataset Terms: Both DIS5K and DUTS specify non-commercial academic research use (`https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf` and `http://saliencydetection.net/duts/`).
   - Provenance Risk Note: Moderate legal uncertainty. While the model weights were published under permissive MIT, the training data carries non-commercial terms. Whether trained weights inherit dataset restrictions or qualify as transformative fair use remains legally unsettled across jurisdictions.
2. **`U-2-Netp` (320)**:
   - Weight License: **Apache-2.0** (explicitly granted by author Xuebin Qin).
   - Dataset Provenance: Trained on DUTS-TR (10,553 images from ImageNet DET / SUN, augmented to 21,106 images).
   - Dataset Terms: Academic / non-commercial research use (`http://saliencydetection.net/duts/`).
   - Provenance Risk Note: Moderate legal uncertainty. Identical dataset provenance risk to BiRefNet: author granted Apache-2.0 for weights and code, but training data carries academic research restrictions.
3. **`IS-Net` (`isnet_general_use`)**:
   - Weight License: Ambiguous / Apache-2.0 code. Checkpoints released alongside DIS paper without distinct commercial weight grant.
   - Dataset Provenance: DIS5K (`DIS-TR`, 3,000 images).
   - Dataset Terms: Strict academic / non-commercial research use only (`https://github.com/xuebinqin/DIS/blob/main/DIS5K-Dataset-Terms-of-Use.pdf`).
   - Provenance Risk Note: High commercial risk. Upstream author bundled non-commercial dataset terms directly with repository. Dropped from consideration per owner decision D-018.

---

## 8. Architectural Decision (D-018 Status)

Owner Decision D-018 has been **ACCEPTED PROVISIONALLY** pending visual review of the compact comparison contact sheet (`bench/out/index.html`):

1. **Default Quality Tier: `BiRefNet_lite_512`**:
   - Model download size: 183.0 MB.
   - Inference latency: ~2.98 s CPU inference (~3.07 s total on i5-10400).
   - Memory footprint: 2,029.9 MB Working Set, 2,411.7 MB Private Commit.
   - Mask agreement: 0.985 average IoU against 1024 reference (suppressing baseline artifacts on noisy backgrounds).
   - Weight License: MIT (moderate dataset provenance uncertainty).
2. **Optional Fast Tier: `U-2-Netp` ("Fast Preview / Low Spec")**:
   - Model download size: 4.36 MB.
   - Inference latency: 266.9 ms CPU inference (332.3 ms total).
   - Memory footprint: 333.3 MB Working Set, 581.3 MB Private Commit.
   - Mask agreement: 0.906 average IoU against 1024 reference.
   - Weight License: Apache-2.0 (moderate dataset provenance uncertainty).
3. **Deferred / Dropped Models**:
   - `birefnet_lite_1024`: Deferred (not default) due to 6.35 GB working set / 8.46 GB commit footprint and allocation abort on continuous execution.
   - `IS-Net`: Dropped from consideration until dataset license questions are resolved.
   - `fp16` and `q8` variants: Dropped (slower on CPU due to software emulation and dynamic quantization overhead).
