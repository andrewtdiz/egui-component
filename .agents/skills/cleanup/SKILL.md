---
name: cleanup
description: Review an implementation for clarity and readability risks caused by legacy paths, dead-weight logic, and clearly unused code, then recommend safe cleanup focused on current behavior and near-term project goals. Surface only critical or high severity findings (8/10 or higher). Use when the user asks to clean up legacy implementations, remove unused functions or variables, or trim non-essential code that no longer serves active behavior.
---

# cleanup

## Workflow

1. Read the implementation end to end and identify active behavior required for current functionality and near-term project goals.
2. Trace call paths and references to separate active code from legacy branches, compatibility leftovers, and dead-weight logic.
3. Identify clearly unused functions, variables, imports, flags, and configuration surface with concrete evidence from call sites and repository search.
4. Evaluate whether each removable area improves readability and reduces maintenance burden without changing intended behavior.
5. Prioritize direct deletions or tight simplifications over broad rewrites when cleanup is sufficient.
6. Score each potential finding from 1 to 10 and keep only findings with severity >=8.

## Findings Format

1. List only cleanup findings with severity >=8/10, highest first.
2. Include file paths and precise locations.
3. Explain why the code is legacy, dead-weight, or clearly unused in current and near-term scope.
4. Cite evidence for unused status, such as missing call sites, unreachable branches, or orphaned configuration.
5. Suggest a minimal cleanup direction and validation step for each issue.
6. Do not include findings below 8/10.
7. State explicitly when no critical or high cleanup findings are present.
