## Summary
<!-- Describe what this PR changes and why. -->

## Type of Change
- [ ] Bug fix (`fix:`)
- [ ] New feature (`feat:`)
- [ ] Documentation (`docs:`)
- [ ] CI / dependencies / housekeeping (`chore:`)

## Quality & Security Checklist
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace -j 2` passes
- [ ] `cargo deny check licenses` and `cargo deny check advisories` pass
- [ ] Frontend checks pass (`cd crates/app && npm test && npm run license-check && npm run license-check:all && npm run check && npm run build`)
- [ ] No model weights (`.onnx`) or files >1 MB added to git
- [ ] Zero telemetry, analytics, or unconsented network calls introduced
- [ ] Any new crate, npm package, or model weight is pinned and recorded in `THIRD_PARTY_LICENSES.md`
- [ ] I agree that my contributions are licensed under `GPL-3.0-or-later` (see `CONTRIBUTING.md`)
