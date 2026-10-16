# The workspace is three crates: domain, gen-questions, app

The repository is a Cargo workspace with three members. `domain` holds the
Civics Test model and the rules — answer matching, selection weights, Cleared
and Coverage, Exam scoring — with no I/O and no wasm, so it tests natively.
`gen-questions` parses M-1778, applies the hand overrides, and writes the
committed question records. `app` is the Dioxus web application: screens,
components, routing, and the IndexedDB boundary.

Dependencies point one way: `app -> domain` and `gen-questions -> domain`;
nothing depends on `gen-questions`. The parser's PDF dependencies can therefore
never reach the WASM build.

## Considered Options

- **A single crate with the parser feature-gated.** Rejected. One forgotten
  gate drags host-only dependencies into every WASM build, and the rules could
  not be tested without the UI toolchain.
- **Two crates, with `domain` as a module inside `app`.** Rejected. Fewer
  members, but every rule test would compile the full UI stack, and the
  domain/UI boundary would hold only by convention.

## Consequences

- Rule behavior is tested with `cargo test -p domain`, natively and fast; the
  behavior worth testing lives there rather than in components.
- The question records are committed at `data/questions.json` and owned by the
  `domain` types: `gen-questions` serializes them, and `app` embeds the file
  at compile time (`include_str!`). Hand corrections live in
  `gen-questions/overrides.json`, never in the data file (ADR 0002).
- A contract test guards the data file: it parses into the domain types and
  holds exactly 128 questions, 20 starred. A bad regeneration fails CI, not
  the browser.
