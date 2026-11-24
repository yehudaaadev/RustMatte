# WORKFLOW: end-session

Description: Protocol to conclude a development session cleanly, update state, and generate the final report.

## Steps

1. Run the `rust-quality-gate` skill (skip if no application code exists yet).
2. Update `PROJECT_STATE.md` with factual status, current phase, and next steps.
3. Update `docs/TASKS.md` checkboxes for completed tasks.
4. Append any newly discovered pitfalls to `docs/PITFALLS.md` and decisions to `docs/DECISIONS.md`.
5. Run the `phase-report` skill to generate and present the session summary.
