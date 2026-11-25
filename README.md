# RustMatte

[![CI](https://github.com/yehudaaadev/RustMatte/actions/workflows/ci.yml/badge.svg)](https://github.com/yehudaaadev/RustMatte/actions/workflows/ci.yml)

Offline, privacy-first AI background remover for desktop and CLI, built with a Rust core and ONNX Runtime.

Status: **v0.1.0 pre-release** (Windows desktop app & CLI tested locally; macOS and Linux builds are experimental and CI-verified only).

Built with Rust. Not affiliated with or endorsed by the Rust Foundation.

## Overview
RustMatte removes image backgrounds locally on your device. No accounts, no cloud uploads, no analytics, and no telemetry. Once the AI model weights are downloaded (once, with explicit confirmation) or imported from a local `.onnx` file, RustMatte works 100% offline.

## Screenshots and Demo (Pending Owner Capture)
Placeholder slots for `docs/screenshots/` (not yet captured interactively):
- `docs/screenshots/main-dropzone.png` — Empty drop zone (`Drop photos or a folder here, or click to browse`)
- `docs/screenshots/comparison-slider.png` — Before/after comparison slider with zoom/pan on checkerboard background and right options panel
- `docs/screenshots/first-run-dialog.png` — One-time AI model setup dialog (`Download` / `Choose a file I already have` / `Cancel`)
- `docs/screenshots/demo-batch.gif` — End-to-end 5–10s clip: drop photos, remove background, inspect cutout, save PNG

## Desktop Application (GUI)

RustMatte includes a native desktop application built with **Tauri 2** and **Svelte 5**:
- **Single-Screen Workflow**: Drop photos or a folder (or browse), inspect a windowed thumbnail queue, pan/zoom the Before/After comparison slider on checkerboard or solid color preview, and export transparent or solid-background PNGs (`Save` or `Save all`).
- **First-Run & Offline Guarantee**: Explains the one-time model download (file name, size, source host, license) with **Download**, **Choose a file I already have** (SHA-256 verified local `.onnx` import), **Cancel**, **Retry**, and a **Never use the network** lock in Options.
- **Transparency & Accessibility**: **About & Licenses** dialog reading shipped [MODELS.md](MODELS.md) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) plus **Copy diagnostic info** (zero file paths); English and Indonesian; System / Light / Dark themes; keyboard shortcuts (`Ctrl+O`, `Ctrl+Enter`, `Ctrl+S`, `Ctrl+Shift+S`, `ArrowUp/Down`, `Esc`).

## Installation (Pre-Release)

Installers are published on [GitHub Releases](https://github.com/yehudaaadev/RustMatte/releases) (no Rust or Node.js required on end-user machines). Model weights are not bundled in installers.

### Windows (`x86_64`)
- Download `RustMatte_<version>_x64-setup.exe` (NSIS installer, `9.69 MiB` measured for `0.1.0`).
- **WebView2**: Bundles Microsoft's WebView2 bootstrapper (`embedBootstrapper`); if Edge WebView2 Runtime is missing, the installer fetches it from Microsoft during setup.
- **SmartScreen Warning**: Because executables are unsigned (`v0.1.0`), Windows SmartScreen may show *"Windows protected your PC"*. Verify the SHA-256 checksum first, then click **More info -> Run anyway**.

### macOS (`aarch64` Apple Silicon only — Experimental)
- Download `RustMatte_<version>_aarch64.dmg`, open it, and drag **RustMatte.app** to **Applications**.
- **Architecture Notice**: Builds currently target **Apple Silicon (`aarch64-apple-darwin`) only**. Intel (`x86_64`) Macs are not built in `v0.1.0`.
- **Gatekeeper Warning**: Because the app is unsigned and not notarized, macOS will block opening it (*"cannot be opened because the developer cannot be verified"*). Verify the `.dmg` checksum first, then open **System Settings -> Privacy & Security** -> **Open Anyway**, or run `xattr -dr com.apple.quarantine /Applications/RustMatte.app`.

### Linux (`x86_64` — Experimental)
- **AppImage**: Download `RustMatte_<version>_amd64.AppImage`, run `chmod +x RustMatte_*.AppImage`, and launch. On Ubuntu 22.04+/24.04+, `libfuse2` (`sudo apt install libfuse2`) may be required.
- **Debian / Ubuntu (`.deb`)**: Download `RustMatte_<version>_amd64.deb` and install with `sudo apt install ./RustMatte_*.deb`.
- **Required Runtime Packages**: System WebKitGTK/GTK packages (`libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`, `librsvg2-2`).

## Verify Your Download

Always verify downloaded installers or CLI archives against `SHA256SUMS.txt` from the release page:
```powershell
# Windows (PowerShell)
Get-FileHash .\RustMatte_0.1.0_x64-setup.exe -Algorithm SHA256
```
```bash
# macOS
shasum -a 256 -c SHA256SUMS.txt --ignore-missing

# Linux
sha256sum -c SHA256SUMS.txt --ignore-missing
```

## Quick CLI Usage

```bash
# Single image (Quality tier default: BiRefNet Lite 512, ~3.0s median total on measured i5-10400 CPU)
rustmatte photo.jpg -o cutout.png

# Fast tier (U-2-Netp, ~0.33s median total across 12 fixtures on measured i5-10400 CPU)
rustmatte product.jpg -o cutout.png --tier fast

# Folder batch mode (outputs to ./rustmatte-output by default)
rustmatte ./photos/ --out ./photos-cutouts/ --recursive --skip-existing

# Early downscaling for very large images (saves RAM and post-processing time)
rustmatte large_photo.jpg -o cutout.png --max-side 1024

# Strictly offline execution (forbids network downloads)
rustmatte photo.jpg -o cutout.png --no-download

# Manage cached model weights
rustmatte models list
rustmatte models download --tier quality --yes
```

### CLI Exit Codes
- `0`: All operations succeeded (processed or skipped cleanly).
- `1`: Total failure or invalid arguments.
- `2`: Partial failure (some images failed in batch mode while others succeeded or were skipped).

## Data Storage and Uninstall

Uninstalling RustMatte removes only the application binary and never touches your photos or exported PNG folders.

| Data | Windows | macOS | Linux |
| :--- | :--- | :--- | :--- |
| **Settings** (`settings.json`) | `%APPDATA%\io.github.yehudaaadev.rustmatte\` | `~/Library/Application Support/io.github.yehudaaadev.rustmatte/` | `~/.config/io.github.yehudaaadev.rustmatte/` |
| **Preview cache** (wiped on exit) | `%LOCALAPPDATA%\io.github.yehudaaadev.rustmatte\previews\` | `~/Library/Caches/io.github.yehudaaadev.rustmatte/previews/` | `~/.cache/io.github.yehudaaadev.rustmatte/previews/` |
| **AI Model cache** (`.onnx`) | `%LOCALAPPDATA%\RustMatte\models\` | `~/Library/Application Support/RustMatte/models/` | `~/.local/share/RustMatte/models/` |

- **Windows**: Uninstall via **Settings -> Apps -> Installed apps -> RustMatte -> Uninstall** (or `uninstall.exe`).
- **macOS**: Move `/Applications/RustMatte.app` to Trash.
- **Linux**: Delete the `.AppImage` or run `sudo apt remove rustmatte`.

## FAQ

- **Does RustMatte work offline?** Yes. Once the `.onnx` model file is cached or imported, RustMatte runs 100% locally. You can also turn on **"Never use the network"** in the desktop Options panel or pass `--no-download` in the CLI.
- **Why does the first run ask to download a model?** Bundling 183 MB+ of AI weights inside every installer bloats downloads and updates. Instead, RustMatte keeps the installer small (`9.69 MiB` on Windows) and asks permission before downloading weights once, verified by SHA-256.
- **What licenses apply to the code and AI models?** The RustMatte application is licensed under **GPL-3.0-or-later** ([LICENSE](LICENSE)). Quality tier (`BiRefNet_lite_512`) weights are **MIT**, and Fast tier (`U-2-Netp`) weights are **Apache-2.0**. Both were trained on academic datasets (DIS5K / DUTS) with non-commercial dataset terms; see [MODELS.md](MODELS.md) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for full provenance notes.
- **Why isn't output always identical to paid cloud APIs?** Cloud services often run huge proprietary multi-stage models on datacenter GPUs. RustMatte runs compact open-weight models (`512x512` / `320x320` input) on local consumer CPUs/GPUs so your photos never leave your computer.
- **Why are glass bottles or windows segmented as solid?** Dichotomous foreground-segmentation models predict a foreground object mask rather than background refraction/transmittance, so transparent glass is treated as a solid subject.

## Comparison with Alternatives

Verified against upstream repositories and pricing pages on **2026-11-25**:

| Tool | Runtime & Install | Privacy / Offline | Licensing Model | Typical Trade-off |
| :--- | :--- | :--- | :--- | :--- |
| **RustMatte** | Native Rust + Tauri 2 desktop app & single CLI binary (`9.69 MiB` Windows setup) | 100% local after one-time consented model fetch or `.onnx` import | App: GPL-3.0-or-later; Weights: MIT / Apache-2.0 ([MODELS.md](MODELS.md)) | Runs on local CPU/GPU (`~3.1 s` Quality, `~0.33 s` Fast measured on desktop CPU); glass segmented solid |
| **[rembg](https://github.com/danielgatis/rembg)** | Python CLI, library, HTTP server, or Docker container | Runs locally on your machine/server (fetches weights on first run) | MIT License ([LICENSE.txt](https://github.com/danielgatis/rembg/blob/main/LICENSE.txt)) | Requires Python environment; supports multiple models (`U^2-Net`, `ISNet`, `BiRefNet`, etc.) |
| **[@imgly/background-removal](https://github.com/imgly/background-removal-js)** | JavaScript / WASM / WebGPU in browser or Node.js | Runs client-side in browser/Node | AGPL-3.0 ([LICENSE.md](https://github.com/imgly/background-removal-js/blob/main/packages/web/LICENSE.md)) or commercial license | Convenient for web apps; AGPL copyleft obligations or commercial licensing apply |
| **[remove.bg](https://www.remove.bg/pricing)** | Cloud SaaS / REST API | Uploads images to third-party servers (files discarded after 60 min per upload notice) | Proprietary commercial service (free previews up to `0.25 MP`; paid credits up to `50 MP`) | Zero local hardware requirements; cloud upload and per-credit cost at full resolution |

## Honest Capabilities and Known Limitations
- **Measured CPU Performance** (Windows 11 Pro, Intel Core i5-10400, 16 GB DDR4, release build; see [docs/BENCHMARKS.md](docs/BENCHMARKS.md)):
  - **Quality Tier (`birefnet_lite_512`, 183 MB)**: `2,984.9 ms` median inference / `3,070.9 ms` median total across 12 fixtures (`~3.68 s` cold-start model load); `2,029.9 MB` peak Working Set. Preserves fine hair and intricate silhouettes.
  - **Fast Tier (`u2netp`, 4.36 MB)**: `266.9 ms` median inference / `332.3 ms` median total across 12 fixtures (`349.5 ms`–`454.6 ms` average in 100-image batch); `333.3 MB` peak Working Set. Best for people and products on clean backgrounds; may leave smears on fur or textured backgrounds.
- **Glass Transparency**: Glass, windows, and translucent containers are segmented as solid foreground objects.
- **Supported Formats**: JPEG, PNG, WebP, and BMP. RAW and HEIC support are on the [Roadmap](docs/ROADMAP.md) (`v1.1`).
- **Platform Status**: Windows is tested locally; macOS (`aarch64`) and Linux builds are experimental until verified on physical hardware.

## Privacy and Network Use Statement
RustMatte makes **zero** background network connections, contains **zero** telemetry/analytics/crash reporting, and never uploads images. The only network code (`ureq` in `crates/core/src/download.rs`) executes solely on explicit user confirmation to fetch pinned AI weights over HTTPS and verify SHA-256 checksums. See [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) and [SECURITY.md](SECURITY.md).

## Building from Source

### Prerequisites
- Rust toolchain (pinned in `rust-toolchain.toml`, version `1.98.1`)
- Node.js `22+` and npm (for `crates/app` frontend)
- On Ubuntu/Debian Linux: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`, `libssl-dev`, `pkg-config`

### Build and Test Steps
```bash
git clone https://github.com/yehudaaadev/RustMatte.git
cd RustMatte

# 1. Build frontend static assets (required before compiling crates/app/src-tauri)
cd crates/app
npm ci
npm test
npm run check
npm run build
cd ../..

# 2. Build and test Rust workspace (use -j 2 on Windows MSVC to avoid LNK1102 OOM)
cargo build --workspace
cargo test --workspace -j 2
```

### Quality Gate
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 2
cargo deny check licenses
cargo deny check advisories
cd crates/app && npm run license-check && npm run license-check:all && npm test && npm run check && npm run build
```

## Documentation Index
- [MODELS.md](MODELS.md) — Model weights, SHA-256 digests, and dataset provenance
- [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) — Full crate, npm, and weight license register
- [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) — Tauri CSP, scoped asset protocol, and IPC boundary
- [docs/BENCHMARKS.md](docs/BENCHMARKS.md) — Measured CPU/memory benchmarks
- [docs/ROADMAP.md](docs/ROADMAP.md) — Post-`v1.0` planned milestones
- [docs/RELEASING.md](docs/RELEASING.md) — Release checklist and draft prerelease notes
- [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## License
RustMatte is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**. See [LICENSE](LICENSE) for the full license text.
