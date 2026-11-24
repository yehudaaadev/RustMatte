---
name: verify-crate-api
description: Verify the exact API signatures and behavior of fast-changing crates against their pinned documentation before writing code.
---

# Verify Crate API

Use this skill before writing code that calls fast-evolving crates (e.g., `ort`, `tauri`).

## Procedure

1. Check `Cargo.toml` to confirm the exact pinned version of the crate.
2. Read the official documentation or source code for THAT exact version (e.g., on `docs.rs/<crate>/<pinned-version>`).
3. Never copy examples from tutorials or third-party guides targeting different versions.
4. Compile a minimal isolated test or example first to verify the API signature.
5. If an unexpected behavior or breaking change is discovered, log an entry in `docs/PITFALLS.md`.
