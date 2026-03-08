# Surface Current Session JSONL

To locate the JSONL file for the current Codex session:

1. Read `CODEX_THREAD_ID` from the environment.
2. Look up that id in `~/.codex/state_5.sqlite`, table `threads`.
3. Use the matching `rollout_path`.

Query shape:

```sql
select rollout_path from threads where id = ?;
```

## Fallback

If the database lookup fails, search `~/.codex/sessions` for the thread id with any file search tool.

## Notes

- `CODEX_THREAD_ID` identifies the current chat.
- `threads.rollout_path` is the absolute path to the session JSONL file.
- The first line of the JSONL file is session metadata and includes fields such as `cwd`.

