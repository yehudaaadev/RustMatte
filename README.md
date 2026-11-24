# RustMatte

Offline, privacy-first AI background remover and photo enhancer with a Rust core.

Status: **pre-alpha, not usable yet**

Built with Rust. Not affiliated with or endorsed by the Rust Foundation.

## Overview
RustMatte is a standalone desktop and command-line utility for removing image backgrounds locally. Images never leave your device: processing occurs 100% offline using a local neural network on CPU (with optional hardware acceleration).

## CLI Usage (Phase 1C-1)

```bash
# Quality Tier (Default: BiRefNet Lite 512, ~3s on CPU)
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png

# Fast Tier (U-2-Netp, ~300ms on CPU)
cargo run --release -p rustmatte-cli -- product.jpg -o cutout.png --tier fast

# Custom model path and thread count
cargo run --release -p rustmatte-cli -- photo.jpg -o cutout.png --model-path ./models/birefnet_lite_512.onnx --threads 6 --overwrite
```

### Model Placement (Phase 1C-1 Note)
Models are not yet auto-downloaded in Phase 1C-1. Place `birefnet_lite_512.onnx` (for Quality tier) or `u2netp.onnx` (for Fast tier) in the `./models/` directory, or provide the path via `--model-path`. Automatic downloading with SHA-256 verification will be introduced in Phase 1C-2.

## Honest Capabilities and Known Limitations
- **Model Tiers**:
  - **Quality Tier (`birefnet_lite_512`)**: High-accuracy edge segmentation, preserves fine hair and intricate edges.
  - **Fast Tier (`u2netp`)**: Sub-second execution; best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.
- **Glass Transparency**: Transparent glass and translucent liquid containers are currently segmented as solid foreground objects (semi-transparent alpha matting is not supported in the raw model output).
- **Edge Post-Processing**: Phase 1C-1 performs basic bilinear mask upscaling and alpha blending. Advanced edge refinement (guided filter, defringing, feathering) will be introduced in Phase 1D.

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
