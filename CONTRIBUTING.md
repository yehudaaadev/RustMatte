# Contributing to RustMatte

Thank you for your interest in contributing to RustMatte.

## License and Contribution Terms
All contributions to this project are accepted under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.
By submitting a pull request, you confirm that you have the right to license your work and agree that your contributions will be distributed under `GPL-3.0-or-later`.

## Prerequisites
- Rust stable toolchain (pinned in `rust-toolchain.toml`, currently `1.98.1`)
- Node.js `22+` and npm (for `crates/app` desktop frontend)
- `cargo-deny` (`cargo install cargo-deny --locked`) for license and advisory audits
- On Ubuntu/Debian Linux (for `crates/app`): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`, `libssl-dev`, `pkg-config`

## Building and Running Tests
```bash
# Rust workspace
cargo build --workspace

# Desktop frontend (from crates/app)
cd crates/app
npm ci
npm test
npm run check
npm run build
cd ../..
```

### Windows MSVC Build Note
On Windows systems using MSVC, running `cargo test --workspace` concurrently with high thread counts can cause `link.exe` to exceed virtual memory limits (`fatal error LNK1102: out of memory`). Always throttle test linking using `-j 2` (see Pitfall `P-025` in `docs/PITFALLS.md`).

## Mandatory Quality Gate
Before opening a pull request, every check below must pass without warnings or errors:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 2
cargo deny check licenses
cargo deny check advisories
cd crates/app && npm run license-check && npm run license-check:all && npm test && npm run check && npm run build
```

## Adding Dependencies or AI Models Safely
1. **Do not commit model weights (`.onnx`) or files >1 MB to git.**
2. **Check licenses before introducing any crate, npm package, or AI model weight:**
   - Run `cargo deny check licenses` (Rust) and `npm run license-check` / `npm run license-check:all` (npm).
   - Never introduce AGPL, non-commercial, or proprietary dependencies/weights.
   - For AI models, verify **both** the weight license and training-dataset terms separately, verify exact ONNX Runtime (`ort`) operator compatibility, and record pinned SHA-256 checksums, source URLs, and risk notes in `THIRD_PARTY_LICENSES.md` and `MODELS.md`.
3. **Pin exact dependency versions** (`=x.y.z` in `Cargo.toml`, exact versions in `crates/app/package.json`) and commit updated lockfiles.
4. **No telemetry or unconsented network calls**: Any network code outside the consented model downloader is rejected.

## Commit Conventions
Use Conventional Commits:
- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation updates
- `chore:` maintenance, dependencies, or CI
- `test:` unit or integration tests

Keep commits focused on one logical change.
