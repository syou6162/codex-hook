---
name: testing-codex-hook
description: E2E test the codex-hook CLI binary. Use when verifying hook condition evaluation, matcher logic, or action execution.
---

# Testing codex-hook

## Build

```bash
cd /home/ubuntu/repos/codex-hook
cargo build --release
# Binary at: target/release/codex-hook
```

## CLI Interface

```bash
echo '<JSON>' | ./target/release/codex-hook --event PreToolUse --config <path-to-yaml>
```

- `--event`: Hook event type (currently only `PreToolUse` is fully implemented)
- `--config`: Path to YAML config file defining hooks
- stdin: JSON input representing the hook event context

## YAML Config Format (Test Fixtures)

```yaml
PreToolUse:
  - matcher: "Bash"          # tool_name regex pattern
    conditions:               # optional; all must pass (AND)
      - type: file_exists
        value: "somefile.txt"
      - type: command_contains
        value: "rm"
    actions:
      - type: output
        message: "hook fired"
        exit_status: 0
```

## JSON Input Format (stdin)

```json
{
  "session_id": "s1",
  "transcript_path": "/tmp/t.log",
  "cwd": "/path/to/working/dir",
  "hook_event_name": "PreToolUse",
  "tool_name": "Bash",
  "tool_input": {"command": "echo hello"},
  "permission_mode": "default"
}
```

Key fields:
- `cwd`: Used by file/dir conditions and cwd conditions
- `tool_name`: Matched against `matcher` regex
- `tool_input`: Used by tool-specific conditions (command_contains, file_extension, url_starts_with)
- `permission_mode`: Optional, used by `permission_mode_is` condition

## Assertion Patterns

| Scenario | Expected stdout | Exit code |
|---|---|---|
| Conditions pass → action fires | `{"message":"..."}` | 0 |
| Conditions fail → action skipped | empty | 0 |
| No conditions → action fires | `{"message":"..."}` | 0 |
| Config parse error | empty stdout, stderr has error | 1 |

## Testing Tips

- Create test fixtures in `/tmp/codex-hook-e2e/` to avoid polluting the repo
- For `file_exists` / `dir_exists` conditions, create real files/dirs in the test dir
- Set `cwd` in the JSON to point to the test fixture directory
- The binary produces NO output when all matched hooks are skipped (conditions fail)
- Use `type: output` actions for easy stdout verification (avoid `type: command` in simple tests)
- `git_tracked_file_operation` condition requires an actual git repo with tracked files

## Unit Tests

```bash
cargo test           # Run all tests (includes condition_tests)
cargo test condition  # Run only condition-related tests
```

## Condition Types (18 for PreToolUse)

### Common (13)
- file_exists, file_not_exists, file_exists_recursive, file_not_exists_recursive
- dir_exists, dir_not_exists, dir_exists_recursive, dir_not_exists_recursive  
- cwd_is, cwd_is_not, cwd_contains, cwd_not_contains
- permission_mode_is

### Tool-specific (5)
- file_extension (checks tool_input.file_path extension)
- command_contains, command_starts_with (check tool_input.command)
- url_starts_with (checks tool_input.url)
- git_tracked_file_operation (checks if file in command is git-tracked)

## Devin Secrets Needed

None. This is a local CLI tool with no external service dependencies.
