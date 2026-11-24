# Third-Party Licenses Register

Rule: Every external dependency (Rust crate, frontend package, tool) and every model weight file must be registered in this table before use. If an item has no entry here, it is NOT approved for use.

| Name | Type | Version | License | Source URL | Approved by Owner | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |

## Candidate Items Awaiting Verification & Approval (Phase 0 / Phase 1)
- `BiRefNet_lite` (model weights): MIT license candidate (Swin-T backbone). Awaiting export verification and Phase 1 CPU benchmark.
- `ISNet` (model weights): License verification from primary source pending.
- `ort` (Rust crate): Pin exact version in Phase 0 (MIT/Apache-2.0).
- `image` (Rust crate): Standard image decoding (MIT/Apache-2.0).
- `tauri` (Rust crate & frontend): Desktop integration (MIT/Apache-2.0).
