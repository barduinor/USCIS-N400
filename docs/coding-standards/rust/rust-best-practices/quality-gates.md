# Quality Gates

Before committing Rust code, the pre-commit hook at `.githooks/pre-commit` runs
three quality gates in order. All three must pass for the commit to proceed.

## Gate order

1. **Format check** — `cargo fmt --check`
2. **Lint** — `cargo clippy --no-deps -- -D warnings`
3. **Full test suite** — `cargo test`

The order is deliberate: format check runs first because it is the fastest gate
and catches the most common issue (forgot to run `cargo fmt`). Clippy runs next
because it subsumes `cargo check` (every clippy lint requires a successful
compilation). The full test suite runs last because it is the slowest gate and
subsumes `cargo build` (tests must compile to run).

## Why `--check` instead of auto-fix

The hook uses `cargo fmt --check`, not `cargo fmt`. The pre-commit hook must
**gate**, not **mutate**. Auto-formatting inside the hook would stage changes the
committer never reviewed. When `--check` fails, the agent sees the formatting
diff in the hook output, runs `cargo fmt` explicitly, reviews the changes, and
re-commits.

## Full test suite requirement

`cargo test` runs the **entire** test suite, not only tests for the current
issue. A change that passes issue-specific tests can still break unrelated tests.
The gate catches regressions before they reach the shared branch.

## Installation

The hook is **documented**, not automatically installed by cloning the
repository. Install it with the repository script:

```sh
scripts/install-hooks
```

The installer marks `.githooks/pre-commit` executable and sets
`core.hooksPath` to `.githooks`, so normal `git commit` attempts run the
repository hook.

## Pre-commit hook example

```sh
#!/bin/sh
set -e

echo "==> fmt check"
cargo fmt --check

echo "==> clippy"
cargo clippy --no-deps -- -D warnings

echo "==> test"
cargo test
```
