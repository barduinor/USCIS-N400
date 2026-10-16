# Quality Gates

Before committing Rust code, the pre-commit hook at `.githooks/pre-commit` runs
four quality gates in order. All four must pass for the commit to proceed. CI
runs the same four gates on a fresh clone; the hook is the local mirror.

## Gate order

1. **Format check** — `cargo fmt --check`
2. **Lint** — `cargo clippy --no-deps --all-targets -- -D warnings`
3. **Full test suite** — `cargo test`
4. **Ship-target check** — `cargo check -p uscis-n400 --target wasm32-unknown-unknown`

The order is deliberate: format check runs first because it is the fastest gate
and catches the most common issue (forgot to run `cargo fmt`). Clippy runs next
because it subsumes `cargo check` (every clippy lint requires a successful
compilation), and `--all-targets` extends the lint to test code. The full test
suite runs last of the run-and-compile gates because it is the slowest gate and
subsumes `cargo build` (tests must compile to run). The ship-target check
closes the sequence because host checks do not prove the wasm target compiles —
cfg differences, wasm-only dependencies, and wasm-bindgen glue break where the
host build cannot see.

## Workspace and the app crate

The repository is a virtual Cargo workspace (ADR 0006), so `cargo fmt`,
`cargo clippy`, and `cargo test` at the root cover every member: `domain`,
`gen-questions`, and the `app` crate.

The `app` crate is expected to compile for the host target, which lets the lint
and test gates cover it. If it turns out to be wasm-only, the lint and test
gates narrow to `-p domain -p gen-questions`, and the ship-target check remains
the `app` crate's compile gate.

## Why `--check` instead of auto-fix

The hook uses `cargo fmt --check`, not `cargo fmt`. The pre-commit hook must
**gate**, not **mutate**. Auto-formatting inside the hook would stage changes the
committer never reviewed. When `--check` fails, the agent sees the formatting
diff in the hook output, runs `cargo fmt` explicitly, reviews the changes, and
re-commits.

## Full test suite requirement

`cargo test` runs the **entire** test suite, not only tests for the current
issue. A change that passes issue-specific tests can still break unrelated tests.
The gate catches regressions before they reach the shared branch. The suite
includes the committed-data guards — `domain`'s contract test on
`data/questions.json` and `gen-questions`' regeneration byte-guard — so a stale
or hand-edited data file fails the gate even when no Rust code changed.

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
cargo clippy --no-deps --all-targets -- -D warnings

echo "==> test"
cargo test

echo "==> wasm check"
cargo check -p uscis-n400 --target wasm32-unknown-unknown
```
