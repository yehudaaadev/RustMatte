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

### P-013: ONNX Runtime BFCArena Allocation Failure on ASPP Deformable Convolution
- Date: 2026-11-24
- Symptom: Runtime crash during `session.run()` with error `BFCArena::AllocateRawInternal Failed to allocate memory for requested buffer of size 822083584` in `/decoder/decoder_block1/dec_att/aspp_deforms.2/atrous_conv/Sum`.
- Cause: ONNX Runtime's internal BFC (Best-Fit with Coalescing) arena allocator fails when trying to preallocate a contiguous 822 MB buffer for the unrolled deformable convolution operator in BiRefNet on CPU.
- Fix or avoidance: Configure the session builder with `.with_memory_pattern(false)` and `.with_config_entry("session.use_env_allocators", "1")`. This disables the internal arena pattern and delegates memory management to system malloc/heap allocators, allowing inference to complete without crash.

### P-014: High Working Set Memory during 1024x1024 BiRefNet Inference on CPU
- Date: 2026-11-24
- Symptom: Process memory consumption climbs to ~7.3 GB working set during 1024x1024 inference runs on CPU.
- Cause: Unrolling high-resolution 1024x1024 intermediate feature maps across ASPP deformable convolution blocks and multi-level decoders requires large intermediate tensor allocations.
- Fix or avoidance: Ensure sequential processing (never concurrent inference of multiple images on CPU), deallocate intermediate image buffers aggressively, and consider evaluating lightweight candidates or downscaled/quantized variants (or smaller resolution backbones like ISNet) for lower-RAM consumer machines.

### P-015: 1024x1024 Memory Exhaustion & Rust Heap Fast-Fail on Continuous Runs
- Date: 2026-11-24
- Symptom: Process crashes abruptly with Windows exit code `0xc0000409 (STATUS_STACK_BUFFER_OVERRUN)` and message `memory allocation of 3145728 bytes failed`.
- Cause: Consecutive multi-run inference on 1024x1024 BiRefNet pushes commit memory beyond operating system limits on 16GB machines. When ONNX Runtime memory pressure saturates the OS commit limit, standard Rust heap allocations fail, triggering Rust's global OOM panic handler which immediately aborts the process.
- Fix or avoidance: Do not use `BiRefNet_lite_1024` as the default model. Adopt `BiRefNet_lite_512` which drops peak commit memory by 66.4% to ~2.4 GB and runs crash-free across all fixtures.

### P-016: FP16 Emulation Overhead on x86 CPUs
- Date: 2026-11-24
- Symptom: FP16 quantized model executes slower than standard FP32 on CPU (4,220 ms vs 3,501 ms on 512x512 BiRefNet).
- Cause: x86 CPUs lacking native AVX-512 FP16 instructions must emulate 16-bit floating point calculations via software casting to 32-bit float and back, adding CPU cycle overhead.
- Fix or avoidance: On CPU execution provider, keep weights in native FP32. Reserve FP16 models for GPU execution providers (DirectML, CoreML, CUDA) with hardware tensor cores.

### P-017: Memory Pattern Disabled (D-015) Saves 1.33 GB RAM
- Date: 2026-11-24
- Symptom: Peak working set surges from ~3,995 MB to 5,331 MB (+33.4%) during inference if D-015 session allocator configuration is omitted.
- Cause: ORT's static memory pattern pre-allocates maximum upper-bound buffers for every node output in the computation graph.
- Fix or avoidance: Always configure `.with_memory_pattern(false)` and `.with_config_entry("session.use_env_allocators", "1")` for CPU inference.

### P-018: INT8 Dynamic Quantization Slower than FP32 on CPU
- Date: 2026-11-24
- Symptom: INT8 quantized ISNet model (`isnet_general_use_q8`) takes 3,359 ms inference vs 2,001 ms for FP32 model.
- Cause: Real-time dynamic quantization and dequantization overhead between layers on CPU exceeds the SIMD throughput advantage.
- Fix or avoidance: Benchmark quantized variants empirically on target hardware before assuming INT8 will yield speed improvements on CPU.

### P-019: Measuring Per-Model Memory Inside a Single Long-Lived Process
- Date: 2026-11-24
- Symptom: Consecutive models report identical peak memory numbers (e.g. 2,463.2 MB commit, 2,258.4 MB working set) across distinct model architectures.
- Cause: Operating system process memory counters (`GetProcessMemoryInfo`) report the peak high-water mark reached over the entire process lifetime. Once an earlier large model inflates the working set and pagefile commit, subsequent lighter models inherit that peak, concealing their true standalone footprints.
- Fix or avoidance: Never measure per-model memory inside one long-lived process. Always measure memory in dedicated, freshly spawned child processes per model so each process lifecycle starts clean with an isolated working set and pagefile counter.

### P-020: Treating Higher-Resolution Baseline Models as Ground Truth
- Date: 2026-11-24
- Symptom: Relying on IoU agreement with a 1024 baseline as a proxy for segmentation quality, ignoring defects present in the baseline.
- Cause: Baseline models are neural networks and can introduce artifacts (e.g., floating background fragments on complex scenes like `latarramai1.jpg`). Lighter candidates (like 512) may actually suppress such artifacts better than the baseline.
- Fix or avoidance: Baseline models can also make mistakes; do not treat any model as ground truth. Treat IoU strictly as mathematical mask agreement with a reference, never as ground-truth quality. Always perform visual review on diverse backgrounds (dark and light) to assess actual edge quality.

### P-021: Multiple ORT Environment Initializations in the Same Process
- Date: 2026-11-24
- Symptom: Runtime error when creating multiple `Remover` instances in the same process or running tests concurrently: `InferenceSession("Failed to initialize ORT environment: only one environment is allowed per process")`.
- Cause: ONNX Runtime C API enforces a strict single-environment per process constraint. Calling `ort::init().build()` multiple times within the same process lifecycle returns an error.
- Fix or avoidance: Store the initialized `ort::environment::Environment` in a process-wide `std::sync::OnceLock<Arc<Environment>>` singleton, allowing all remover sessions to share the single environment safely.

### P-022: Unguarded Platform-Specific FFI Linking Breaks Cross-Platform CI
- Date: 2026-11-24
- Symptom: GitHub Actions runner on Ubuntu and macOS fails during `cargo test --workspace` with linker errors: `rust-lld: error: unable to find library -lpsapi` and `ld: library 'psapi' not found`.
- Cause: Examples in `crates/core/examples/` included `#[link(name = "psapi")]` without `#[cfg(windows)]`. Cargo compiles examples during `cargo test --workspace`, causing Unix linkers to search for the Windows-specific DLL.
- Fix or avoidance: Always gate platform-specific FFI declarations and link directives behind `#[cfg(target_os = ...)]` or `#[cfg(windows)]`, and provide no-op fallbacks for other platforms.

### P-023: `ureq` Version 2.x API Differences
- Date: 2026-11-24
- Symptom: Compilation error `no method named max_redirects found for struct AgentBuilder`.
- Cause: `ureq` 3.x renamed several configuration methods, whereas pinned `ureq` 2.10.1 uses `.redirects(n)`.
- Fix or avoidance: Check documentation for the exact pinned crate version (`docs.rs/ureq/2.10.1`) rather than assuming modern 3.x method signatures.

### P-024: Cold-Start Model Session Initialization Latency
- Date: 2026-11-24
- Symptom: Single-image CLI invocation for `BiRefNet_lite_512` takes ~6.8 seconds total, despite inference taking only ~3.0 seconds in benchmarks.
- Cause: Cold-start ONNX model parsing, node allocation, and graph initialization in `ort::session::Session` takes ~3.68 seconds on a 183 MB model on CPU.
- Fix or avoidance: Verified via `--verbose` stage timing breakdown. This is normal for cold-start CLI commands. In Phase 2 (batch processing) and Phase 3 (GUI desktop app), the session is initialized once and reused across images, eliminating the 3.68s cold-start overhead on subsequent runs.

### P-025: MSVC `link.exe` LNK1102 Out of Memory Under High Test Concurrency
- Date: 2026-11-24
- Symptom: Debug test suite compilation fails on Windows with `fatal error LNK1102: out of memory`.
- Cause: When Cargo runs link steps across 12 logical cores simultaneously (`-j 12`), multiple instances of MSVC `link.exe` concurrently build debug test executables embedding symbols and object code from heavy dependencies (`ort`, `ring`, `icu_*`, `image`), exceeding available virtual memory.
- Fix or avoidance: Throttle debug test linker concurrency with `cargo test --workspace -j 2` or link binaries sequentially. Release builds optimize symbols and do not encounter this memory limit.

### P-026: Synchronous Child Process Pipe Buffer Deadlock
- Date: 2026-11-24
- Symptom: Child process hangs indefinitely after processing ~90 images during long-running batch CLI runs with redirected stdout/stderr.
- Cause: Operating system standard error/output pipes have a fixed 4KB buffer. If the child process writes verbose output that exceeds 4KB while the parent process synchronously polls `HasExited` without reading the stream asynchronously, the child blocks on `write()` / `eprintln!`.
- Fix or avoidance: Always redirect child process stdout/stderr to disk files (`cmd.exe /c ... > stdout.log 2> stderr.log`) or use asynchronous event handlers (`BeginOutputReadLine`, `BeginErrorReadLine`) when monitoring processes that generate large output streams.

### P-027: Base64 IPC Bloat & Webview Memory Duplication
- Date: 2026-11-25
- Symptom: Sending full-resolution original and cutout images as Base64 data URLs over Tauri IPC inflates JSON payloads by ~33% and duplicates multi-megabyte buffers across Rust and WebView2 JS heap.
- Cause: Tauri IPC serializes command payloads as JSON/strings; high-resolution PNGs (e.g., 24MP) create large allocations on both sides of the bridge.
- Fix or avoidance: Write bounded `<= 1600 px` preview PNGs into `$APPCACHE/previews/` and pass file paths to the frontend using `convertFileSrc` with `assetProtocol` scoped strictly to that directory, while writing full-resolution cutouts directly on disk.

### P-028: Tauri 2 Linux CI System Libraries & `svelte-check` TypeScript Peer Range
- Date: 2026-11-25
- Symptom: (1) `ubuntu-latest` CI fails compiling `gio-sys v0.18.1` (`Package 'gio-2.0' not found`) once `crates/app/src-tauri` joins the Cargo workspace; (2) upgrading `typescript` to `7.0.2` violates `svelte-check@4.7.8` peer dependency (`^5.0.0 || ^6.0.0`).
- Cause: Tauri 2 requires Linux GTK3/WebKitGTK dev headers via `apt-get`, and `svelte-check` 4.x targets TypeScript 5/6.
- Fix or avoidance: Install Tauri 2 Ubuntu system packages in `.github/workflows/ci.yml` when `matrix.os == 'ubuntu-latest'`, and pin `typescript` to `"6.0.3"` in `crates/app/package.json`.

### P-029: Mixing `node --test` TypeScript Files with SvelteKit `svelte-check`
- Date: 2026-11-25
- Symptom: Running `svelte-check` fails with `Cannot find name 'node:test'` and `An import path can only end with a '.ts' extension` when zero-dependency unit tests in `crates/app/tests/` use `node --test` with `.ts` imports.
- Cause: SvelteKit's generated `.svelte-kit/tsconfig.json` type-checks `tests/` without `@types/node` or `allowImportingTsExtensions`.
- Fix or avoidance: Enable `"allowImportingTsExtensions": true` and `"noEmit": true` in `crates/app/tsconfig.json` and provide a minimal `crates/app/tests/node.d.ts` declaration rather than adding external npm dev dependencies.

### P-030: PowerShell `--` Argument Parsing with `npm run tauri build`
- Date: 2026-11-25
- Symptom: Running `npm run tauri build -- --bundles nsis` in Windows PowerShell fails with `npm error Unknown cli flag: --bundles`.
- Cause: PowerShell strips or alters bare `--` tokens passed to `npm`, causing npm to interpret `--bundles` as an npm CLI flag instead of forwarding it to `tauri`.
- Fix or avoidance: Invoke `npx tauri build --bundles <target>` directly in both local PowerShell commands and `.github/workflows/release.yml`.

### P-031: Static vs Dynamic ONNX Runtime Linkage Verification
- Date: 2026-11-25
- Symptom: Checking `target\release\` on Windows shows no `onnxruntime.dll` (and an empty 0-byte `DirectML.dll` placeholder), raising concern that an installed app might fail on a clean system.
- Cause: `ort = "=2.0.0-rc.15"` (`download-binaries`) links ONNX Runtime statically into `rustmatte-app.exe` on Windows `x86_64-pc-windows-msvc` (verified via PE import table inspection and silent NSIS installation into an isolated directory). On macOS/Linux, dynamic linkage may still occur depending on upstream `ort` prebuilts.
- Fix or avoidance: Inspect `otool -L` (macOS) and `ldd` (Linux) inside `.github/workflows/release.yml` on the first tagged build before publishing the draft release.

### P-032: Full-Frame `Vec<f32>` Allocations in Guided Filter Cause +1.17 GB RAM Spike at 24 MP
- Date: 2026-11-25
- Symptom: Processing `large_24mp.jpg` (`6000x4000`) with edge refinement jumped from `2,277 MB` (`--refine none`) to `3,353 MB` on Quality tier and from `584 MB` to `1,660 MB` on Fast tier.
- Cause: `guided_filter` allocated 13 concurrent full-resolution `Vec<f32>` buffers (`96 MB` each = `1,248 MB`) alongside `orig.to_rgb8()` (`72 MB`) and `orig.to_rgba8()` (`96 MB`) copies.
- Fix or avoidance: Process `guided_filter` in 128-row horizontal bands with `4 * radius` halos, reuse band scratch buffers, and read pixels directly from `DynamicImage` without full-frame RGB/RGBA copies (`D-027`). Note that resetting the vertical sliding-window `f32` sum per band eliminates cross-image float drift (`~1.2e-5`), which can flip a few edge pixels by 1 LSB (changing PNG SHA-256 hashes while preserving visual equivalence).

### P-033: Plain Cargo Release Build Loads Tauri `devUrl` and Process Liveness Misses WebView2 `ERR_CONNECTION_REFUSED`
- Date: 2026-11-25
- Symptom: Launching `target\release\rustmatte-app.exe` built via plain `cargo build --release -p rustmatte-app` opens a window whose title reads `RustMatte - Offline AI Background Remover`, with `HasExited = False` and `Responding = True`, yet WebView2 shows `"Hmmm... can't reach this page. localhost refused to connect. ERR_CONNECTION_REFUSED"`.
- Cause: In pinned `tauri = "=2.12.7"`, `tauri::is_dev()` is `!cfg!(feature = "custom-protocol")`. `npx tauri build` passes `--features tauri/custom-protocol` (and runs `beforeBuildCommand`), whereas plain `cargo build --release -p rustmatte-app` compiles with `devUrl` (`http://localhost:1420`). Checking OS process liveness (`HasExited`/`Responding`) cannot detect a WebView2 error page inside an otherwise live native window.
- Fix or avoidance:
  1. Never add `custom-protocol` to `default` features (doing so turns off `tauri::is_dev()` during `npm run tauri dev`); declare optional `custom-protocol = ["tauri/custom-protocol"]` and enforce `#[cfg(all(not(debug_assertions), dev, not(test)))] compile_error!(...)` in `crates/app/src-tauri/src/lib.rs` (`D-028`).
  2. Emit a frontend-mounted readiness signal (`notify_frontend_ready`) to `$APPCACHE/startup.log` (`ui_ready mode=custom-protocol origin=http://tauri.localhost`) and verify port `1420` has zero connection activity during offline launch tests.
  3. Add a binary marker check in `.github/workflows/release.yml` across Windows, macOS, and Linux failing any build in `dev-url` mode.
