# Security Policy

## Supported Versions
Only the latest release on `main` (`0.1.x`) is supported with security updates.

## Reporting a Vulnerability Privately
Please **do not** open a public GitHub issue for security vulnerabilities, model tampering vectors, or Tauri capability/CSP bypasses.

Report vulnerabilities privately via GitHub Private Vulnerability Reporting:
- `https://github.com/yehudaaadev/RustMatte/security/advisories/new`

Include in your report:
- Affected component (`rustmatte-core`, `rustmatte-cli`, or `rustmatte-app` desktop UI)
- RustMatte version and OS/architecture
- Steps to reproduce or proof-of-concept
- Potential impact (for example, arbitrary file read/write, CSP bypass, checksum bypass, or crash on malformed input)

The maintainer will acknowledge receipt as soon as possible and coordinate a fix and advisory before public disclosure.

## Security Design Scope
RustMatte's desktop security boundary is documented in [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md):
- Zero telemetry, zero analytics, and zero background network requests.
- Downloaded AI model weights are verified against pinned SHA-256 digests (`crates/core/src/model.rs`) before use, and `"Never use the network"` blocks all downloads in Rust.
- Tauri 2 desktop UI runs under a strict CSP with no remote origins, no `unsafe-eval`, no `tauri-plugin-shell`, no `tauri-plugin-fs`, and `assetProtocol` scoped strictly to `$APPCACHE/previews/**`.
