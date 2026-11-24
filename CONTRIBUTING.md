# Contributing to RustMatte

Thank you for your interest in contributing to RustMatte.

## License & Contribution Terms
All contributions to this project are accepted under the terms of the **GNU General Public License v3.0 or later (GPL-3.0-or-later)**.
By submitting a pull request, you agree that your contributions will be licensed under this license.

## Prerequisites
- Rust stable toolchain (pinned in `rust-toolchain.toml`)
- Cargo

## Building the Workspace
```bash
cargo build --workspace
```

## Running the Quality Gate
Before submitting any pull request or declaring a task complete, all checks must pass without warnings or errors:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 2
cargo deny check licenses
cargo deny check advisories
cargo build --workspace
```

### Windows MSVC Build Note
On Windows systems using MSVC, running `cargo test --workspace` concurrently with high thread counts can cause `link.exe` to exceed virtual memory limits (`fatal error LNK1102: out of memory`). Always throttle test linking using `-j 2` (as documented in Pitfall P-025).

## Commit Conventions
Commit messages must follow the Conventional Commits specification:
- `feat:` for new features or capabilities
- `fix:` for bug fixes
- `docs:` for documentation updates
- `chore:` for toolchain, CI, or workspace maintenance
- `test:` for test additions or fixtures

One logical change per commit. Never commit model weights or large files (>1MB) to git.
