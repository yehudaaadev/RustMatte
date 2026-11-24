# RustMatte Known Pitfalls and Lessons Learned

Rule: Append a new entry here whenever an issue fails in a non-obvious way, or when a critical architectural trap is identified.

Format:
- ID: P-001, P-002, ...
- Date: YYYY-MM-DD
- Symptom: What went wrong or failed
- Cause: Underlying technical explanation
- Fix or avoidance: Concrete preventive action

---

### P-001: Fast-Changing `ort` (ONNX Runtime) Crate API
- Date: 2026-11-24
- Symptom: Compile errors on session initialization, tensor extraction, or execution provider setup.
- Cause: The `ort` crate undergoes breaking API changes across versions (e.g., 1.x to 2.x). Online tutorials and outdated docs frequently demonstrate incompatible signatures.
- Fix or avoidance: Always pin the exact `ort` version in `Cargo.toml`. Never copy examples from tutorials; read the docs for the exact pinned version.

### P-002: Model Weight Licensing Mismatches and Non-Commercial Conflicts
- Date: 2026-11-24
- Symptom: Legal conflict where commercial end-users (sellers, photographers) violate model weight license terms.
- Cause: Popular models (such as RMBG-2.0 under CC BY-NC 4.0) forbid commercial use. Model weights often carry licenses distinct from their repository code.
- Fix or avoidance: Verify weight licenses from primary sources independently from code licenses. Avoid non-commercial weights for RustMatte. Record licenses in `THIRD_PARTY_LICENSES.md` before writing inference code.

### P-003: Poor Edge Quality and Color Bleed on Hair and Fur
- Date: 2026-11-24
- Symptom: Cutouts look artificial with jagged edges or colored fringes (halo effect) around hair and fur.
- Cause: Raw model mask output has soft transition zones containing background color bleeding into semi-transparent pixels.
- Fix or avoidance: Implement dedicated mask refinement (guided filter on original photo, feathering in transition zone) and alpha defringing post-processing in `crates/core::image_ops`.

### P-004: Memory Spikes on High-Resolution Images in Batch Mode
- Date: 2026-11-24
- Symptom: Out-Of-Memory (OOM) crashes or system freezing when processing large folders.
- Cause: Decoding multiple 24MP+ images into uncompressed 32-bit RGBA buffers simultaneously consumes gigabytes of RAM.
- Fix or avoidance: Use a strictly bounded worker pool in `rayon` (1-2 full-resolution images in RAM max) and immediately deallocate image buffers after saving to disk.

### P-005: Divergence of Hardware Execution Providers Across Operating Systems
- Date: 2026-11-24
- Symptom: Application crashes at startup or fails to create ONNX sessions on machines without specific GPUs.
- Cause: Execution providers (DirectML on Windows, CoreML on macOS, CUDA on Linux) depend on specific system drivers and hardware.
- Fix or avoidance: Always use CPU as default execution provider. Keep DirectML/CoreML as optional acceleration with automatic CPU fallback. Never make discrete GPU a hard requirement.

### P-006: Dynamic Linking of ONNX Runtime Shared Libraries
- Date: 2026-11-24
- Symptom: Executable fails to launch on clean machines with "missing DLL / so / dylib" error.
- Cause: ONNX Runtime relies on underlying C++ binaries (`onnxruntime.dll`, etc.) that must be present in the executable path.
- Fix or avoidance: Use the `download-binaries` feature in the `ort` crate and configure Tauri bundling scripts to package the runtime shared libraries with the application installer.

### P-007: Assuming Model Size or License from Different Versions or Memory
- Date: 2026-11-24
- Symptom: Incorrect architectural assumptions (e.g. assuming RMBG-2.0 is 50-90 MB when actual ONNX files are 234 MB to 1 GB).
- Cause: Relying on unverified claims or confusing quantized weights of older versions (e.g., u2netp or RMBG-1.4 INT8) with newer architectures.
- Fix or avoidance: Always inspect the exact `.onnx` file metadata, byte size, and primary source model card before documenting or committing to a model.

### P-008: Non-Commercial Model Weights Conflicting with Target Users
- Date: 2026-11-24
- Symptom: Users processing product photos for online stores commit copyright infringement.
- Cause: Non-commercial licenses (CC BY-NC) forbid any business-related use, even by individuals selling items online.
- Fix or avoidance: Reject non-commercial model weights. Only select models with permissive licenses (such as MIT or Apache-2.0) or commercial-use clearance.

### P-009: Misunderstanding Copyleft vs Commercial Freedom
- Date: 2026-11-24
- Symptom: Believing GPL or AGPL licenses forbid selling software.
- Cause: Conflating copyleft (which requires making source code available when distributing modifications) with non-commercial restrictions.
- Fix or avoidance: Remember that GPL-3.0 and AGPL-3.0 explicitly allow selling software and commercial usage. Only non-commercial licenses (e.g. PolyForm Noncommercial, CC-BY-NC) forbid commercial use, and they are not open source.

### P-010: Untrusted or Incompatible Third-Party ONNX Exports
- Date: 2026-11-24
- Symptom: ONNX Runtime fails to run inference on CPU or produces nonsensical outputs due to missing operator kernels.
- Cause: Random third-party ONNX conversions may use unsupported operators (e.g., deformable convolution in certain BiRefNet exports) that fail on CPU execution providers.
- Fix or avoidance: Never trust unverified third-party exports. Prefer official exports or export from official weights directly. Verify the model runs correctly in `ort` on CPU with sample tensors before adopting.

### P-011: Stating Unmeasured Performance Numbers as Facts
- Date: 2026-11-24
- Symptom: Misleading documentation or unmet product expectations (e.g., claiming "under 3 seconds on mid-range CPU" before testing).
- Cause: Guessing latency, throughput, or memory usage without empirical benchmarks on real hardware.
- Fix or avoidance: Mark all unverified performance claims as UNMEASURED. Record performance numbers only after running benchmarks on documented hardware in `tests/fixtures`.

### P-012: Rust Trademark Policy Compliance
- Date: 2026-11-24
- Symptom: Potential trademark dispute with the Rust Foundation.
- Cause: Incorporating "Rust" into a software project name without following the Rust Foundation Trademark Policy.
- Fix or avoidance: Using "rust" in repository or crate names is permissible when referring to software written in or for the Rust language, but the project must not imply official endorsement, use the official Rust logo, or register trademarks/domains that infringe. Explicitly include a disclaimer in README: "Built with Rust. Not affiliated with or endorsed by the Rust Foundation."
