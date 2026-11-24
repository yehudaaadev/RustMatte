# RustMatte

Offline, privacy-first AI background remover and photo enhancer with a Rust core.

Status: **pre-alpha, not usable yet**

Built with Rust. Not affiliated with or endorsed by the Rust Foundation.

## Overview
RustMatte is a standalone desktop and command-line utility for removing image backgrounds locally. Images never leave your device: processing occurs 100% offline using a local neural network on CPU (with optional hardware acceleration).

## CLI Usage (Phase 1C-2)

```bash
# Quality Tier (Default: BiRefNet Lite 512, ~3s inference on CPU)
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png

# Fast Tier (U-2-Netp, ~250ms inference on CPU)
cargo run --release -p rustmatte-cli -- product.jpg -o cutout.png --tier fast

# Auto-confirm model download without interactive prompt
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png --yes

# Strictly offline execution (forbids network downloads)
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png --no-download

# Verbose pipeline timing breakdown (model load, decode, preprocess, inference, postprocess, write)
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png --verbose

# Manage model weights without processing images
cargo run --release -p rustmatte-cli -- models list
cargo run --release -p rustmatte-cli -- models download --tier quality --yes
```

## Network Use Policy

RustMatte operates 100% offline by design:
- The **only** network activity is an optional, one-time model download on first run when required model weights are not present locally.
- Before contacting the network, RustMatte discloses the file name, download size, source host, and license, requesting explicit confirmation (`[y/N]`).
- Network access can be unconditionally prohibited by passing `--no-download`.
- There are zero analytics, zero telemetry pings, and zero external tracking.

## Model Notice & Attribution

- **Quality Tier**: `BiRefNet_lite_512` (183 MB), released under the **MIT** license by author ZhengPeng7.
- **Fast Tier**: `U-2-Netp` (4.4 MB), released under the **Apache-2.0** license by author Xuebin Qin.
- **Training Data Notice**: Both models were trained on public academic datasets (DIS5K, DUTS) that contain non-commercial research clauses. Whether training dataset clauses extend to trained model weights is legally unsettled across jurisdictions. RustMatte provides this notice for transparency; it does not constitute legal advice. Full license texts and attribution are cataloged in [MODELS.md](file:///C:/DavidYehudaSurbakti/desktop-system/LocalBG/MODELS.md).

## Honest Capabilities and Known Limitations
- **Model Tiers**:
  - **Quality Tier (`birefnet_lite_512`)**: High-accuracy edge segmentation, preserves fine hair and intricate edges. Cold-start model session load takes ~3.6s on CPU; inference takes ~3.0s.
  - **Fast Tier (`u2netp`)**: Sub-second execution (~250ms inference); best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.
- **Glass Transparency**: Glass, windows, and translucent containers are segmented as solid foreground objects due to the nature of dichotomous salient object models. Edge refinement (Phase 1D) only improves boundary smoothing and suppresses color halos; it will not make solid glass transparent.
- **Cold-Start Latency**: The single-image CLI command parses and initializes the ONNX graph on every run. Batch folder processing (Phase 2) and the desktop GUI (Phase 3) reuse the session, eliminating this cold-start load on subsequent images.
- **Benchmark Hardware**: Tested and measured on Windows 11 Pro (build 26200), Intel Core i5-10400 (6 cores, 12 threads), 16 GB DDR4 RAM.

## Planned Features (v1.0)
- Single image background removal with transparent PNG output
- Parallel batch folder processing with bounded memory usage
- Interactive desktop GUI via Tauri 2 (drag-and-drop, before/after comparison slider)
- Pure CPU inference by default with optional DirectML/CoreML acceleration
- Support for standard raster formats (JPEG, PNG, WebP, BMP)
- Fast mode: early downscaling for high-resolution images to conserve decode time and RAM
- Bilingual interface (English and Bahasa Indonesia)
- Light and dark themes

## Building from Source

### Prerequisites
- Rust toolchain (refer to `rust-toolchain.toml`)

### Build Workspace
```bash
cargo build --workspace
```

### Run Tests and Quality Gate
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Run CLI Binary
```bash
cargo run -p rustmatte-cli -- --help
cargo run -p rustmatte-cli -- --version
```

## License
RustMatte is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See [LICENSE](LICENSE) for the full license text.
