#!/usr/bin/env bash
set -euo pipefail

output_path=dist/claude/response.md
max_wait_seconds=1800
prompt=""
prompt_file=""

usage() {
  cat <<'EOF'
Usage:
  run.sh --prompt "<text>" [--output <path>] [--timeout <seconds>]
  run.sh --prompt-file <path> [--output <path>] [--timeout <seconds>]

Options:
  --prompt <text>            Prompt text to send to Claude.
  --prompt-file <path>       File containing prompt text.
  --output <path>            Output file path. Default: dist/claude/response.md
  --timeout <seconds>        Max wait time. Default: 1800
  -h, --help                 Show usage.
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --prompt)
      shift
      prompt=${1:-}
      ;;
    --prompt-file)
      shift
      prompt_file=${1:-}
      ;;
    --output)
      shift
      output_path=${1:-}
      ;;
    --timeout)
      shift
      max_wait_seconds=${1:-}
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

if ! command -v claude >/dev/null 2>&1; then
  echo "claude CLI not found on PATH" >&2
  exit 127
fi

if [ -n "$prompt" ] && [ -n "$prompt_file" ]; then
  echo "provide either --prompt or --prompt-file, not both" >&2
  exit 2
fi

if [ -n "$prompt_file" ]; then
  if [ ! -f "$prompt_file" ]; then
    echo "prompt file not found: $prompt_file" >&2
    exit 2
  fi
  prompt=$(cat "$prompt_file")
fi

if [ -z "$prompt" ]; then
  echo "missing prompt: use --prompt or --prompt-file" >&2
  exit 2
fi

if ! [[ "$max_wait_seconds" =~ ^[0-9]+$ ]] || [ "$max_wait_seconds" -le 0 ]; then
  echo "timeout must be a positive integer: $max_wait_seconds" >&2
  exit 2
fi

mkdir -p "$(dirname "$output_path")"

cmd=(claude -p "$prompt" --permission-mode dontAsk)

set +e
timeout -k 30s "${max_wait_seconds}s" "${cmd[@]}" > "$output_path"
status=$?
set -e

if [ $status -eq 124 ] || [ $status -eq 137 ]; then
  echo "claude run timed out after ${max_wait_seconds}s: $output_path" >&2
  exit 124
fi

if [ $status -ne 0 ]; then
  echo "claude run failed with exit code $status" >&2
  exit $status
fi

if [ ! -s "$output_path" ]; then
  echo "claude run completed but output file is empty: $output_path" >&2
  exit 3
fi

echo "claude response written: $output_path"
