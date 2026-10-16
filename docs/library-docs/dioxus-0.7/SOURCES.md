# Dioxus 0.7 Sources

This file records where these library docs came from and how agents should
retrieve more detail when needed.

## Version

- Project requirement: `dioxus = { version = "0.7" }`, the 0.7 patch line,
  per ADR 0005. Upstream 0.8 alphas exist; do not chase them.
- Latest 0.7 patch observed at the time of writing: `dioxus v0.7.10`.
- Documentation target: Dioxus 0.7 documentation.
- Notes adapted: 2026-10-16, from the project-agnostic Dioxus 0.7 pack
  maintained in the inkmetric project (original notes created 2026-05-08).

## Upstream Documentation

- Dioxus 0.7 docs website: `https://dioxuslabs.com/learn/0.7/`
- Source repository: `https://github.com/DioxusLabs/docsite`
- Versioned markdown source path: `docs-src/0.7/` in the upstream repository.

## Context Policy

The upstream docs are extensive. Agents should not paste or load the full
documentation into context by default.

Use `INDEX.md` to identify the relevant topic, then open only that upstream
section. If a local snapshot is added later, it should live in a clearly marked
generated folder and agents should still read it by targeted file, not
recursively.
