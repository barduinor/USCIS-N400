# Coding Standards

This is the canonical entry point for project-owned coding standards.

These standards apply to all implementation work unless a more specific curated
standard in this directory says otherwise.

## Test Integrity

- Do not weaken, delete, skip, or rewrite tests only to make a failing change
  pass.
- Preserve tests that capture established behavior. If behavior must change,
  update tests to describe the new expected behavior and explain the reason in
  the implementation issue or commit.
- Prefer tests that exercise user-visible behavior, persistence boundaries, or
  domain rules over tests that only mirror implementation details.
- Treat failing verification as a blocker until the cause is understood. Do not
  close an issue while required tests or checks are failing.
- When a requested change is documentation-only or prompt-only, verification can
  be limited to inspecting the changed files and confirming no application
  behavior changed.
- Every issue must have at least one **behavioral** acceptance criterion —
  something that can only be verified by running the system, not by running
  a unit test. Bad: `validate_config() returns Err`. Good: `Running the CLI
  with a missing config file exits with code 1 and prints the error before
  doing any work`. A behavioral AC describes what the **system does**,
  not what a function returns.

## Issue Standards

Behavioral scenarios (Given/When/Then) are the backbone of every issue — these
standards **complement** BDD, not replace it. A complete issue gives the
implementer consequence awareness: WHY the change matters, what invariants must
hold, and what must NOT be touched.

- **User Story**: Every issue should include an "As a \<role\>, I want
  \<feature\>, so that \<benefit\>" statement. The actor, motivation, and
  desired outcome must be traceable. Inherits from the parent PRD when
  available.
- **Current Behavior (bugs)**: Bug issues must capture what happens now with
  evidence — CLI output, log excerpts, or screenshots. Enhancement issues
  describe the baseline being built upon.
- **Root Cause (bugs)**: Bug issues should include a root cause analysis
  explaining WHY the behavior is broken, not just WHAT is broken. Prevents
  surface-level fixes.
- **Business Rules**: The issue should explicitly list invariants that must hold
  after the change and things that must NOT change. These are guardrails for the
  implementer. Use universal claims ("Every X must Y"), not instance-specific
  assertions.
- **Non-Functional Requirements** (complex changes): Document performance
  targets, compatibility guarantees, determinism requirements, and security
  constraints when they apply.
- **Technical Context** (complex changes): Architecture decisions, schema
  changes, interface contracts, and ADR references. Describe types and
  interfaces, not file paths — file paths go stale.

Not every issue needs all sections. User Story and Business Rules are encouraged
for most issues. Current Behavior, Root Cause, NFRs, and Technical Context are
for bugs and complex changes.

## Naming Intent

- Name code after the intent it serves, not after incidental UI placement or
  storage mechanics.
- Use the established domain vocabulary of the project consistently across
  user-facing text, issue language, tests, modules, and comments.
- Do not replace canonical domain terms with local synonyms. If a concept has no
  settled name, surface that gap instead of inventing durable vocabulary
  silently.

## File And Entry-Point Boundaries

- Keep each source file focused on one coherent topic. A file can contain
  multiple functions or types when they serve the same responsibility, but it
  should not mix unrelated application behavior, domain workflow, UI rendering,
  persistence, request handling, and test support.
- Keep executable entry points thin. Main files, binary targets, launchers,
  command entry points, and server bootstraps should wire startup dependencies
  and delegate to named modules.
- Do not let entry-point files accumulate durable application behavior. Once
  code has a stable topic, state, workflow, rendering responsibility, or
  testable behavior, move it behind a module named for that responsibility.
- Extract before adding more behavior when an entry point already contains more
  than startup orchestration, or when any source file mixes two named
  responsibilities.
- Treat a growing entry point as an architectural boundary issue, not only a
  formatting issue. Organizing a large entry-point file internally is not a
  substitute for extracting cohesive responsibilities into modules.

## Standards Routing

Detailed Rust standards now exist at:

- `docs/coding-standards/rust/README.md`

Use the Rust entry point for Rust application code, backend code, tests, Cargo
metadata, and Rust examples. Continue to apply the global test integrity and
naming rules in this file before any language-specific detail.

UI framework guidance is not curated yet. When a UI target is chosen, add an
entry point at `docs/coding-standards/<framework>/README.md` covering component
boundaries, target-specific concerns, and UI async state, and route that work
there. Do not put framework-specific rules in the Rust entry point.

Expected future routing:

- `docs/coding-standards/<framework>/` for UI framework standards, once a UI
  target is chosen.
- `docs/library-docs/` for curated, project-agnostic library documentation.
