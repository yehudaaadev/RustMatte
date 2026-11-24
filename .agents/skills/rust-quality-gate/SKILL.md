---
name: rust-quality-gate
description: Run mandatory Rust quality checks (format, clippy, tests, build) and verify execution on real input before marking any code task as done.
---

# Rust Quality Gate

Use this skill before reporting any code task as done.

## Procedure

1. Run format check:
   `cargo fmt --all -- --check`
2. Run strict clippy linting across all targets:
   `cargo clippy --workspace --all-targets -- -D warnings`
3. Run the complete test suite:
   `cargo test --workspace`
4. Run workspace build:
   `cargo build --workspace`
5. If any check fails, fix all reported errors and rerun from step 1. Never report done on a failing gate.
6. Execute the implemented feature or binary on real sample input.
7. Record the exact command run and the observed output or generated artifact.
