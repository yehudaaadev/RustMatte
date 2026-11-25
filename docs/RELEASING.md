# RustMatte Release Procedure

This document defines the end-to-end process to prepare, verify, and publish a RustMatte release (`CLI` + `Tauri 2` desktop installers).

## 1. Versioning (Single Source of Truth)
- `Cargo.toml` (`[workspace.package].version`) is the authoritative version for `rustmatte-core`, `rustmatte-cli`, and `rustmatte-app`.
- `crates/app/src-tauri/tauri.conf.json` intentionally omits `"version"` so Tauri 2 reads it directly from `crates/app/src-tauri/Cargo.toml` (`version.workspace = true`).
- `crates/app/package.json` `"version"` must match `Cargo.toml`. Both `.github/workflows/ci.yml` and `.github/workflows/release.yml` enforce version parity.
- When bumping a version:
  1. Update `version` in `Cargo.toml` and `crates/app/package.json` (run `npm install --package-lock-only` in `crates/app` and update `--excludePackages "cratesapp@<version>"` in `package.json`).
  2. Add a dated release section in `CHANGELOG.md` (and keep `## [Unreleased]` above it).
  3. Commit to `main` and confirm all jobs in `.github/workflows/ci.yml` pass.

## 2. Pre-Release Quality Gate
Run locally before tagging:
```bash
cd crates/app
npm ci
npm run license-check
npm run license-check:all
npm test
npm run check
npm run build
cd ../..
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 2
cargo deny check licenses
cargo deny check advisories
# Desktop release build (must use Tauri CLI so custom-protocol + npm run build run):
cd crates/app && npx tauri build --no-bundle && cd ../..
```
Note: Plain `cargo build --release -p rustmatte-app` is blocked at compile time because it omits `custom-protocol` (`P-033`). `.github/workflows/release.yml` builds via `npx tauri build` on all three OSes and runs a post-build binary check confirming `RUSTMATTE_BUILD_MODE=custom-protocol` (without `dev-url`).

## 3. Triggering a Tagged Draft Prerelease
`.github/workflows/release.yml` runs on any pushed `v*` tag using only the default `GITHUB_TOKEN`:
```bash
git tag v0.1.0-rc2
git push origin v0.1.0-rc2
```

Artifacts produced per platform:
- **Windows (`windows-latest`, `x86_64-pc-windows-msvc`)**: `rustmatte-cli-<tag>-x86_64-pc-windows-msvc.zip` + `RustMatte_<version>_x64-setup.exe` (NSIS with WebView2 `embedBootstrapper`)
- **macOS (`macos-latest`, `aarch64-apple-darwin`, Apple Silicon only)**: `rustmatte-cli-<tag>-aarch64-apple-darwin.tar.gz` + `RustMatte_<version>_aarch64.dmg` (unsigned; Intel `x86_64` Macs not covered)
- **Linux (`ubuntu-22.04`, `x86_64-unknown-linux-gnu`)**: `rustmatte-cli-<tag>-x86_64-unknown-linux-gnu.tar.gz` + `RustMatte_<version>_amd64.AppImage` + `RustMatte_<version>_amd64.deb`
- **Checksums**: consolidated `SHA256SUMS.txt`

## 4. Verification Before Publishing the Draft
1. Inspect GitHub Actions logs for `Build (aarch64-apple-darwin)` (`otool -L`) and `Build (x86_64-unknown-linux-gnu)` (`ldd`) to confirm ONNX Runtime linkage/bundling.
2. Download the installers and `SHA256SUMS.txt` from the GitHub Draft Prerelease and verify checksums (`Get-FileHash`, `shasum -a 256 -c`, `sha256sum -c`).
3. Test the Windows installer on a clean second Windows machine without Rust or Node installed, verify model download/import, disconnect from the internet, and confirm offline processing and uninstall behavior.
4. Paste the release notes template below into the GitHub Draft Release body, review, and click **Publish release**.

## 5. Release Notes Template (`v0.1.0`)

```markdown
## RustMatte v0.1.0 (Pre-Release)

Offline, privacy-first AI background remover for desktop and CLI (GPL-3.0-or-later).

### What Works
- **Desktop App (Windows x64 primary)**: Drop photos or folders, preview before/after cutouts with zoom/pan, choose Quality (`BiRefNet Lite 512`) or Fast (`U-2-Netp`) tiers, refine edges, and export transparent or solid-background PNGs (`Save` / `Save all`).
- **100% Offline After Setup**: Zero telemetry, analytics, or uploads. Models download once on consent (verified by SHA-256) or can be imported locally from `.onnx`, and `"Never use the network"` enforces strict offline mode.
- **CLI (`rustmatte`)**: Single-image and sequential folder batch mode (`--recursive`, `--skip-existing`, `--max-side`, `--device cpu|directml|coreml|auto`).

### What Does Not / Known Limitations
- **Experimental platforms**: macOS (`aarch64` `.dmg`, Apple Silicon only; no Intel build) and Linux (`.AppImage`, `.deb`) are built in CI and have not yet been manually tested on physical macOS/Linux machines.
- **Glass & translucent objects**: Dichotomous foreground segmentation treats transparent glass/bottles as solid objects.
- **Fast tier caveat**: `U-2-Netp` is fast (<1 s on measured CPU) and best for people/products on clean backgrounds, but may leave smears on fur or busy backgrounds.
- **Deferred features**: Crop-to-subject, WebP/JPEG export, RAW/HEIC decoding, resumable downloads, and upscaling are planned for future releases (`docs/ROADMAP.md`).

### Unsigned App Warnings
- **Windows (SmartScreen)**: Unsigned executable. Verify SHA-256 first, then click *More info -> Run anyway* if prompted.
- **macOS (Gatekeeper)**: Unsigned `.dmg`/`.app`. Verify SHA-256 first, then use *System Settings -> Privacy & Security -> Open Anyway* (or `xattr -dr com.apple.quarantine /Applications/RustMatte.app`).
- **Linux**: Requires system `libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`, `librsvg2-2` (plus `libfuse2` on some distros for AppImage).

### Verify Checksums
Compare downloaded files against `SHA256SUMS.txt`:
- Windows: `Get-FileHash .\RustMatte_0.1.0_x64-setup.exe -Algorithm SHA256`
- macOS: `shasum -a 256 -c SHA256SUMS.txt --ignore-missing`
- Linux: `sha256sum -c SHA256SUMS.txt --ignore-missing`

### Reporting Problems
Open a Bug Report at https://github.com/yehudaaadev/RustMatte/issues/new/choose and attach the output of **About & Licenses -> Copy diagnostic info** (contains no file paths). For security issues, report privately via https://github.com/yehudaaadev/RustMatte/security/advisories/new.
```
