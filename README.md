# USCIS-N400

A single-user, phone-first study tool for the civics portion of the U.S.
naturalization interview. It drills the 2025 civics test questions and simulates
the test itself. It runs entirely in the browser — no server, no account.

## Where to start

| Doc | What it holds |
|---|---|
| `CONTEXT.md` | Domain language, the 3 Section / 8 Topic structure, screens, rules |
| `docs/adr/` | Decisions that are hard to reverse, and the alternatives rejected |
| `docs/open-work.md` | What is decided but unbuilt, and what is still undecided |
| `docs/resources/` | The USCIS source documents, and the authority each carries |
| `docs/coding-standards/` | Stack-level standards, deliberately project-agnostic |

## Status

**Design phase. No code yet** — no `Cargo.toml`, no `src/`.

Settled and written down: answer matching and Override, Coverage, study
behaviour, Exam behaviour, the dynamic questions, the platform, the screens,
and the repo layout — three crates, ADR 0006.

Still open: the study screen's reveal and Override presentation, and the
onboarding field list. Each is expanded, with what is already settled, in
`docs/open-work.md` §Open decisions.

The first coding step is the scaffold: a compiling skeleton carrying one
working vertical slice, expanded in `docs/open-work.md` §Decided, not built.
