---
name: ask-claude
description: Runs an external Claude CLI prompt for a task and writes the full markdown/text response to a file. Use ONLY when the user explicitly tells you to ask Claude something.
---

# ask-claude

Use this skill to run `claude` with a user-provided prompt and save its response.

## Inputs

- Prompt text (required) or prompt file path (required)
- Output path (default: `dist/claude/response.md`)
- Max wait seconds for long-running reviews (default: `1800`)

## Workflow

1. Confirm Claude CLI exists.
- Run `which claude` and `claude --version`.

2. Run the script with a prompt.
- `bash .agents/skills/ask-claude/scripts/run.sh --prompt "..." --output <output_md> --timeout <seconds>`
- Or: `bash .agents/skills/ask-claude/scripts/run.sh --prompt-file <file> --output <output_md> --timeout <seconds>`

3. Poll long-running execution.
- If command execution returns an active session id, poll every 5 seconds.
- Keep polling until the process exits.
- Report progress while polling so the user knows the run is still active.

4. Verify output file.
- Confirm output file exists and is non-empty.
- Print a short preview (`head`/`sed`) and line count.

## Polling Rules

- Use non-interactive runs with `dontAsk` permission mode.
- Do not use `--dangerously-skip-permissions` when running as root.
- If run exceeds `max_wait_seconds`, terminate with timeout and allow a short kill grace period.
- Treat timed out runs as partial and report that output may be incomplete.

## Deliverable

- Output file at the requested path containing Claude's full response.
