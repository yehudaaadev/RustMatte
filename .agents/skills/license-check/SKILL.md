---
name: license-check
description: Verify and record licenses for any new crate, model weight, or tool before introducing it to the project.
---

# License Check

Use this skill before adding any new crate dependency, AI model, or external tool.

## Procedure

1. Identify the license of the code repository or crate.
2. Identify the license of any model weights separately (weights often differ from code).
3. Check `THIRD_PARTY_LICENSES.md` for existing registration.
4. If the license is AGPL, GPL, non-commercial, proprietary, custom, or ambiguous, STOP and ask the owner for explicit approval. Never assume compatibility from memory.
5. If approved, add a complete entry to `THIRD_PARTY_LICENSES.md` with: Name, Type, Version, License, Source URL, and approval status.
