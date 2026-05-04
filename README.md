# codex-hook

A [cchook](https://github.com/syou6162/cchook)-compatible hook tool for [Codex CLI](https://github.com/openai/codex), written in Rust.

codex-hook replaces complex JSON hook configurations with clean YAML syntax, template-based data access, and conditional logic.

## Installation

```bash
cargo install --git https://github.com/syou6162/codex-hook
```

## Quick Start

### 1. Enable Hooks and Configure Codex CLI

Codex CLI hooks require a feature flag. Add the following to `~/.codex/config.toml`:

```toml
[features]
codex_hooks = true
```

Then register codex-hook as a command hook. You can use either **inline TOML** or a separate **hooks.json** file.

#### Option A: Inline in `config.toml`

```toml
[features]
codex_hooks = true

[[hooks.PreToolUse]]
matcher = ""

[[hooks.PreToolUse.hooks]]
type = "command"
command = "codex-hook --event PreToolUse"

[[hooks.PostToolUse]]
matcher = ""

[[hooks.PostToolUse.hooks]]
type = "command"
command = "codex-hook --event PostToolUse"
```

#### Option B: Separate `hooks.json`

Create `~/.codex/hooks.json` (keep the feature flag in `config.toml`):

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "codex-hook --event PreToolUse"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "codex-hook --event PostToolUse"
          }
        ]
      }
    ]
  }
}
```

> **Note**: Do not use both `hooks.json` and inline `[hooks]` in the same config layer — Codex will merge them but emit a warning.

You can also place project-scoped hooks in `.codex/config.toml` or `.codex/hooks.json` at the project root.

### 2. Create Configuration File

Create `~/.config/codex-hook/config.yaml` with your hooks:

```yaml
# Block write operations to .env files
PreToolUse:
  - matcher: "Write|Edit"
    conditions:
      - type: file_extension
        value: ".env"
    actions:
      - type: output
        message: ".env files should not be modified directly"
        exit_status: 2

# Auto-format Go files after Write/Edit
PostToolUse:
  - matcher: "Write|Edit"
    conditions:
      - type: file_extension
        value: ".go"
    actions:
      - type: command
        command: "gofmt -w {.tool_input.file_path}"
```

### 3. Run

codex-hook is invoked automatically by Codex CLI via the hooks configuration above.
Codex passes JSON on stdin and codex-hook returns JSON on stdout.

## CLI Options

```
codex-hook --event <EventType> [--config <path>]
```

| Flag | Description |
|---|---|
| `--event` (required) | Event type (`PreToolUse`, `PostToolUse`, etc.) |
| `--config` | Path to YAML config file (default: `~/.config/codex-hook/config.yaml`) |

If `--config` is omitted, codex-hook looks for `$XDG_CONFIG_HOME/codex-hook/config.yaml`, falling back to `~/.config/codex-hook/config.yaml`.

## Configuration

The configuration file uses YAML format, compatible with [cchook](https://github.com/syou6162/cchook).

Top-level keys are PascalCase event types. Each event contains a list of hooks with `matcher`, `conditions`, and `actions`.

```yaml
PreToolUse:
  - matcher: "Write|Edit"       # regex pattern to match tool_name
    conditions:                  # all conditions must be true (AND)
      - type: file_extension
        value: ".rs"
    actions:
      - type: output
        message: "Rust file detected"
```

### Matcher

The `matcher` field is a regular expression that filters hooks by tool name. It is automatically anchored (`^(?:...)$`) to match the full name. Use `|` for alternation (e.g. `"Write|Edit"`). An empty matcher matches everything.

### Conditions

All conditions in a hook must evaluate to `true` (AND semantics) for actions to execute. Each condition has a `type` and a `value`.

#### Filesystem Conditions

| Type | Description |
|---|---|
| `file_exists` | File exists in cwd |
| `file_not_exists` | File does not exist in cwd |
| `file_exists_recursive` | File exists anywhere under cwd (recursive) |
| `file_not_exists_recursive` | File does not exist anywhere under cwd (recursive) |
| `dir_exists` | Directory exists in cwd |
| `dir_not_exists` | Directory does not exist in cwd |
| `dir_exists_recursive` | Directory exists anywhere under cwd (recursive) |
| `dir_not_exists_recursive` | Directory does not exist anywhere under cwd (recursive) |

#### CWD Conditions

| Type | Description |
|---|---|
| `cwd_is` | cwd equals value exactly |
| `cwd_is_not` | cwd does not equal value |
| `cwd_contains` | cwd contains value as substring |
| `cwd_not_contains` | cwd does not contain value as substring |

#### Permission Condition

| Type | Description |
|---|---|
| `permission_mode_is` | Codex's permission mode matches value |

#### Tool-Specific Conditions

| Type | Description |
|---|---|
| `file_extension` | `tool_input.file_path` ends with value |
| `command_contains` | `tool_input.command` contains value |
| `command_starts_with` | `tool_input.command` starts with value |
| `url_starts_with` | `tool_input.url` starts with value |
| `git_tracked_file_operation` | Command operates on git-tracked files (value is pipe-separated command list, e.g. `"rm\|mv"`) |

#### Prompt/Session Conditions

| Type | Description |
|---|---|
| `prompt_regex` | User prompt matches regex (UserPromptSubmit) |
| `every_n_prompts` | Fires every N prompts (UserPromptSubmit) |
| `reason_is` | Session end reason matches value (SessionEnd) |

### Actions

Each hook can have multiple actions. Two types are supported:

#### `command`

Executes a shell command via `sh -c`. The command's stdout is suppressed to avoid interfering with the Codex JSON protocol. stderr is inherited.

```yaml
actions:
  - type: command
    command: "echo {.tool_name} >> /tmp/hook.log"
```

#### `output`

Outputs a JSON message (`{"message": "..."}`) to stdout. Optionally sets an exit status (non-zero takes precedence across multiple output actions).

```yaml
actions:
  - type: output
    message: "Operation blocked"
    exit_status: 2
```

### Template Syntax

Action `command` and `message` fields support `{<jq_query>}` template syntax. The content inside braces is evaluated as a jq query against the JSON input from Codex.

Examples:

| Template | Result |
|---|---|
| `{.tool_name}` | Tool name (e.g. `Write`) |
| `{.tool_input.file_path}` | File path from tool input |
| `{.tool_input.command}` | Command string from tool input |
| `{.cwd}` | Current working directory |

Empty braces `{}` are not treated as templates.

## Supported Event Types

| Event | Matcher target | Status |
|---|---|---|
| `PreToolUse` | `tool_name` | Implemented |
| `PostToolUse` | `tool_name` | Config parsing only |
| `PermissionRequest` | `tool_name` | Config parsing only |
| `Notification` | optional matcher | Config parsing only |
| `Stop` | — | Config parsing only |
| `SubagentStop` | optional matcher | Config parsing only |
| `SubagentStart` | `tool_name` | Config parsing only |
| `PreCompact` | `tool_name` | Config parsing only |
| `SessionStart` | `tool_name` | Config parsing only |
| `SessionEnd` | optional matcher | Config parsing only |
| `UserPromptSubmit` | — | Config parsing only |

## Configuration Examples

### Block dangerous shell commands

```yaml
PreToolUse:
  - matcher: "Bash"
    conditions:
      - type: command_starts_with
        value: "rm -rf"
    actions:
      - type: output
        message: "rm -rf is not allowed"
        exit_status: 2
```

### Run pre-commit hooks on file changes

```yaml
PostToolUse:
  - matcher: "Write|Edit"
    conditions:
      - type: file_exists
        value: ".pre-commit-config.yaml"
    actions:
      - type: command
        command: "pre-commit run --files {.tool_input.file_path}"
```

### Project-specific hooks

```yaml
PreToolUse:
  - matcher: "Write|Edit"
    conditions:
      - type: cwd_contains
        value: "/my-project"
      - type: file_extension
        value: ".py"
    actions:
      - type: output
        message: "Python file in my-project — remember to run tests"
```

## License

MIT
