# codex-hook

## Setup

### Pre-commit hooks

```bash
pip install pre-commit
pre-commit install
```

commit 前に `cargo fmt -- --check` と `cargo clippy -- -D warnings` が自動で実行されます。
