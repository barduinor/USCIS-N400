# USCIS-N400

A single-user, phone-first study tool for the civics portion of the U.S.
naturalization interview. It drills the 2025 civics test questions and simulates
the test itself. It runs entirely in the browser: no server, no account.

## Where to start

| Doc | What it holds |
|---|---|
| `GLOSSARY.md` | Domain language, the 3 Section / 8 Topic structure, screens, rules |
| `docs/adr/` | Decisions that are hard to reverse, and the alternatives rejected |
| `docs/open-work.md` | What is decided but unbuilt |
| `docs/resources/` | The USCIS source documents, and the authority each carries |
| `docs/coding-standards/` | Global, Rust, and Dioxus standards |
| `docs/library-docs/` | Curated library documentation, project-agnostic; each library has an `INDEX.md` |
| `docs/research/` | Cited research notes, one file per question |
| `AGENTS.md` | Agent entry: skills config pointers, the docs map, and the standing rules |

## Status

**Design phase complete. No code yet**: no `Cargo.toml`, no `src/`.

Settled and written down: answer matching and Override, Coverage, study
behaviour, Exam behaviour, the dynamic questions, the platform, the screens,
and the repo layout (three crates, ADR 0006). Nothing is open.

The v1 build is specified in issue #1 and sliced into tickets #2 through #15 on
the GitHub tracker, each labelled `ready-for-agent`. The first slice is the
scaffold: a compiling skeleton carrying one working vertical slice, expanded in
`docs/open-work.md` §Decided, not built.

Work lands on a branch per ticket, merged to `main` when the four quality gates
are green; the gates and the pre-commit hook are documented in
`docs/coding-standards/rust/rust-best-practices/quality-gates.md`.

## Setting up a machine

What a fresh machine needs, in order:

1. **Clone the repository.** Its agent skills are pinned inside it
   (`.agents/skills/`, `.claude/skills/`, `skills-lock.json`), so there is
   nothing to install globally: an agent harness that scans repo-local skill
   directories picks them up on its own.
2. **GitHub CLI, authenticated.** Agents read and write the issue tracker with
   `gh`. Run `gh auth login` once per machine; verify with `gh auth status`.
3. **Rust toolchain** (needed from the scaffold onward). Install `rustup`, then
   add the wasm target: `rustup target add wasm32-unknown-unknown`. The exact
   toolchain gets pinned by `rust-toolchain.toml` when the scaffold lands.
4. **Dioxus CLI** (needed to serve and build the app, from the app-shell ticket
   onward): `cargo install dioxus-cli --locked`, or a prebuilt binary.
5. **Hooks, once the scaffold lands.** `scripts/install-hooks` points git at
   the repository's pre-commit hook, so every commit runs the four quality
   gates locally.
6. **Optional: Node**, used only to update the pinned skills. Re-running the
   skills CLI inside the repo moves the pins forward; the diff shows what
   changed.

That is the whole list: git, an agent harness, `gh`, and the Rust toolchain.
There are no servers, databases, or credentials to arrange; the only account
involved is GitHub.
