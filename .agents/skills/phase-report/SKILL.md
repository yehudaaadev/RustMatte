---
name: phase-report
description: Generate the standardized phase and session completion report after updating project state and task lists.
---

# Phase Report

Use this skill at the end of every phase or session.

## Procedure

1. Ensure `PROJECT_STATE.md` is updated with current facts.
2. Ensure `docs/TASKS.md` has completed task checkboxes marked.
3. Format the final output using these exact headings:
   - **What was built**: Concise summary of completed deliverables.
   - **Files changed**: List of created, modified, or deleted files.
   - **Commands run and their real results**: Exact terminal commands and outputs.
   - **Test and benchmark results**: Pass/fail counts and metrics.
   - **Known issues and limitations**: Documented gaps or deferred items.
   - **Decisions made**: Architectural choices logged (highlight pending owner approvals).
   - **Proposed next step**: The immediate next action for the upcoming session.
