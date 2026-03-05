---
name: simplify
description: Review an implementation for bloat, unnecessary complexity, and unclear flow, then narrow it to core functionality with concise, newcomer-friendly structure. Surface only critical or high severity issues (>=8/10). Use when the user asks to simplify code, reduce over-engineering, remove indirection, trim non-essential logic, or make an implementation easier to understand for someone unfamiliar with the codebase.
---

# simplify

## Workflow

1. Read the implementation end to end and identify the minimum required behavior.
2. Identify non-essential branches, abstractions, and indirection that do not serve the core behavior.
3. Identify duplicated logic, speculative flexibility, and configuration surface that can be removed.
4. Evaluate whether control flow and naming are immediately understandable to a new contributor.
5. Prioritize simplifications that preserve behavior while reducing moving parts and code volume.
6. Score each potential finding from 1 to 10 and keep only findings with severity >=8.

## Findings Format

1. List only simplification findings with severity >=8/10, highest first.
2. Include file paths and precise locations.
3. Explain why each area adds bloat or confusion.
4. Suggest a direct, minimal rewrite direction for each issue.
5. Do not include findings below 8/10.
6. State explicitly when no critical or high simplification findings are present.
