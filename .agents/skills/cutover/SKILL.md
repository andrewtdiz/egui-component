---
name: cutover
description: Execute hard-cutover migrations for core functionality. Use when the user asks to replace, redesign, remove, or rewire behavior and wants a full break with no compatibility path.
---

# cutover

Since we are pre-release, we can make breaking changes to core engine behavior and APIs in order to improve the engine.
Use a hard cutover approach for engine migrations.
Hard cutover means remove old surface and behavior entirely (no shims/fallbacks), update callsites/tests/examples/docs in the same change, and keep docs greenfield-only with no legacy mentions.

## Workflow

1. Define the exact old contracts/behaviors to remove and the exact new contracts/behaviors to ship.
2. Apply a hard cutover only: remove old surface and behavior entirely in the same change.
3. Do not add compatibility paths: no shims, aliases, fallback paths, or runtime deprecation warnings.
4. Treat removed symbols and behavior as non-existent in runtime code, type surfaces, tests, examples, and docs.
5. Update all internal callsites, tests, templates, examples, and reference docs in the same change.
6. Keep docs greenfield and present-tense only; do not mention removed or deprecated APIs unless explicitly requested.
7. Run repo-wide searches for removed symbols/old behavior paths and require zero remaining matches outside intentional migration tooling.
8. Verify build and relevant static checks after migration.
9. If any requirement forces temporary compatibility or migration notes, stop and ask for explicit approval.

## Output Format

1. State what was removed and what replaced it (contracts, symbols, and behavior).
2. List changed files grouped by runtime, types, tests, templates, and docs.
3. Report verification results, including repo-wide old-surface/old-behavior search outcomes.
4. Explicitly call out any remaining migration risk or blockers.
