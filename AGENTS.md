# RustMatte Operating Rules

## Project
RustMatte is an offline, privacy-first AI background remover with a Rust core.
It processes single images and folders locally without accounts, cloud uploads, or telemetry.
Core inference uses ONNX Runtime (`ort`) on CPU by default, packaged for desktop via Tauri 2.

## Session Start Protocol
1. Read `PROJECT_STATE.md` to restore architectural context and current phase.
2. Read `docs/TASKS.md` for the active phase task list and acceptance criteria.
3. Read only specific documentation files directly relevant to the current task.

## Commands
- Build workspace: `TBD until Phase 0`
- Run test suite: `TBD until Phase 0`
- Run linter: `TBD until Phase 0`
- Check format: `TBD until Phase 0`
- Run CLI: `TBD until Phase 0`

## Hard Rules (Never Break)
- No `unwrap()` or `expect()` in library code (`crates/core`); return typed `Result`.
- No network calls anywhere except the documented, user-disclosed model download.
- No telemetry, analytics, tracking, or background pings.
- Never commit model weights (`.onnx`, etc.) or large files (>1MB) to git.
- Models download on demand on first run; never bundle models in v1.0 installer.
- Pin exact dependency versions in `Cargo.toml`; never use wildcards or unpinned ranges.
- Check licenses before adding dependencies or models using the `license-check` skill.
- Never add AGPL, GPL, or non-commercial licenses without asking the owner first.

## Ask-First List (Stop and Ask the Owner Before)
- Choosing or changing an AI model.
- Adding any major dependency or crate.
- Modifying architecture or crate boundaries.
- Deviating from requirements in `docs/PRD.md`.
- Using any dependency or asset with an unclear license.

## Working Style
- One phase per session; work on one task at a time.
- Plan first (short outline of files and risks), then write code.
- Small commits following Conventional Commits format (`feat:`, `fix:`, `docs:`, `chore:`).

## Definition of Done
A task is done only when:
1. `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass.
2. The feature was actually executed on real sample input with verified output.
3. Evidence (command output or verifiable artifact) is included in the report.

## Honesty Rule
Never claim success that was not directly observed by running the command.
If a command, test, or feature could not be executed or verified, state it explicitly.
Mark unmeasured performance or size claims as UNMEASURED.

## Session End Protocol
1. Update `PROJECT_STATE.md` with factual status.
2. Update `docs/TASKS.md` checkboxes.
3. Run the `phase-report` skill to generate the session summary report.

## Documentation Index
- `docs/PRD.md`: Source of truth for product requirements and scope.
- `docs/ARCHITECTURE.md`: System architecture, crate layout, and pipeline map.
- `docs/DECISIONS.md`: Log of accepted and pending architectural decisions.
- `docs/TASKS.md`: Task breakdown with verifiable acceptance criteria per phase.
- `docs/PITFALLS.md`: Known traps, edge cases, and prevention strategies.
- `docs/references/README.md`: Index of external authoritative references.
- `THIRD_PARTY_LICENSES.md`: Register of approved dependencies and weights.
- `tests/fixtures/README.md`: Test image set specification and guidelines.
