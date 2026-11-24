# RustMatte

Offline, privacy-first AI background remover and photo enhancer with a Rust core.

Status: **pre-alpha, not usable yet**

Built with Rust. Not affiliated with or endorsed by the Rust Foundation.

## Overview
RustMatte is a standalone desktop and command-line utility for removing image backgrounds locally. Images never leave your device: processing occurs 100% offline using a local neural network on CPU (with optional hardware acceleration).

Model weights are downloaded separately on demand during first launch (with SHA-256 verification and local caching in the OS app data directory) and are not part of the source repository.

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
