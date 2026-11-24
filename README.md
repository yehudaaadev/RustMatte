# RustMatte

Offline, privacy-first AI background remover and photo enhancer with a Rust core.

Status: **early alpha for developers and testers**

Built with Rust. Not affiliated with or endorsed by the Rust Foundation.

## Overview
RustMatte is a standalone desktop and command-line utility for removing image backgrounds locally. Images never leave your device: processing occurs 100% offline using a local neural network on CPU (with optional hardware acceleration).

## CLI Usage

### Single Image Processing
```bash
# Quality Tier (Default: BiRefNet Lite 512, ~1.8s inference on CPU)
rustmatte photo.jpg -o cutout.png

# Fast Tier (U-2-Netp, ~350ms inference on CPU)
rustmatte product.jpg -o cutout.png --tier fast

# Early downscaling for huge images (bounds peak RAM and decode time)
rustmatte large_photo.jpg -o cutout.png --max-side 1024

# Auto-confirm model download without interactive prompt
rustmatte photo.jpg -o cutout.png --yes

# Strictly offline execution (forbids network downloads)
rustmatte photo.jpg -o cutout.png --no-download

# Verbose pipeline timing breakdown (model load, decode, preprocess, inference, postprocess, write)
rustmatte photo.jpg -o cutout.png --verbose
```

### Batch Folder Processing
```bash
# Process all supported images in a folder (outputs to ./rustmatte-output by default)
rustmatte ./photos/

# Specify custom output directory
rustmatte ./photos/ --out ./photos-cutouts/

# Recursively process subdirectories
rustmatte ./photos/ --out ./photos-cutouts/ --recursive

# Skip already processed images (useful for resuming interrupted jobs)
rustmatte ./photos/ --out ./photos-cutouts/ --skip-existing

# Overwrite existing files
rustmatte ./photos/ --out ./photos-cutouts/ --overwrite

# Batch process with Fast tier and early downscaling
rustmatte ./photos/ --out ./photos-cutouts/ --tier fast --max-side 1024
```

### Model Weight Management
```bash
# List local model cache status, file sizes, licenses, and checksums
rustmatte models list

# Pre-fetch model weights to local OS cache
rustmatte models download --tier quality --yes
rustmatte models download --tier fast --yes
```

## Exit Codes
RustMatte returns deterministic exit codes suitable for scripting and automated pipelines:
- `0`: All operations succeeded (all images processed or skipped cleanly).
- `1`: Total failure or invalid usage (invalid CLI arguments, missing input, model load failure, zero valid images found, or 100% of images failed).
- `2`: Partial failure (in batch mode, at least one image failed while one or more images succeeded or were skipped).

## Network Use Policy
RustMatte operates 100% offline by design:
- The **only** network activity is an optional, one-time model download on first run when required model weights are not present locally.
- Before contacting the network, RustMatte discloses the file name, download size, source host, and license, requesting explicit confirmation (`[y/N]`).
- Network access can be unconditionally prohibited by passing `--no-download`.
- There are zero analytics, zero telemetry pings, and zero external tracking.

## Model Notice & Attribution
- **Quality Tier**: `BiRefNet_lite_512` (183 MB), released under the **MIT** license by author ZhengPeng7.
- **Fast Tier**: `U-2-Netp` (4.4 MB), released under the **Apache-2.0** license by author Xuebin Qin.
- **Training Data Notice**: Both models were trained on public academic datasets (DIS5K, DUTS) that contain non-commercial research clauses. Whether training dataset clauses extend to trained model weights is legally unsettled across jurisdictions. RustMatte provides this notice for transparency; it does not constitute legal advice. Full license texts and attribution are cataloged in [MODELS.md](MODELS.md).

## Honest Capabilities and Known Limitations
- **Model Tiers**:
  - **Quality Tier (`birefnet_lite_512`)**: High-accuracy edge segmentation, preserves fine hair and intricate edges. Model session load takes ~3.6s cold-start on CPU; inference takes ~1.8s.
  - **Fast Tier (`u2netp`)**: Lightweight execution (~350ms per image); best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.
- **Batch Processing**: Operates strictly sequentially with a single shared ORT session and bounded memory (exactly one full-resolution image in RAM at a time). Output filenames preserve stems, and collisions (e.g. `a.jpg` and `a.png`) resolve deterministically as `a_jpg.png` and `a_png.png`.
- **Glass Transparency**: Glass, windows, and translucent containers are segmented as solid foreground objects due to the nature of dichotomous salient object models.
- **Benchmark Hardware**: Tested and measured on Windows 11 Pro (build 26200), Intel Core i5-10400 (6 cores, 12 threads), 16 GB DDR4 RAM.

## Building from Source

### Prerequisites
- Rust toolchain (pinned in `rust-toolchain.toml`, version 1.98.1)
- On Windows with MSVC, use `-j 2` during multi-crate test linking to prevent linker memory limits.

### Build Workspace
```bash
cargo build --workspace
```

### Run Tests and Quality Gate
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 2
cargo deny check licenses
cargo deny check advisories
```

### Run CLI Binary
```bash
cargo run -p rustmatte-cli -- --help
cargo run -p rustmatte-cli -- --version
```

## License
RustMatte is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See [LICENSE](LICENSE) for the full license text.
