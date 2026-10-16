# Dioxus Coding Standards

These standards apply when changing Dioxus UI code in the `app` crate.
Consult `docs/library-docs/dioxus-0.7/` for Dioxus 0.7 API documentation, and
apply the global and Rust standards in `docs/coding-standards/` first. The app
is web-only (ADR 0005); none of these rules reach outside `app`.

## Component Boundaries

- Keep executable entry points thin: `main.rs` launches and wires the router,
  `routes.rs` defines the routes. Screen behavior lives in `screens/`, one
  module per screen.
- Extract a component when it has its own state, async resource, or testable
  rendering behavior; share such pieces through `components/`.
- Screens render decisions, they never compute them. Verdicts, Cleared,
  Coverage, question selection, and Exam scoring come from `domain`; the app
  never re-implements a rule.
- Keep component names, visible text, tests, and comments on `GLOSSARY.md`
  language (Applicant, Civics Test Question, Accepted Answer, Override,
  Coverage, Cleared, Exam) and away from its avoided words (quiz, card,
  user, score).
- Preserve stable list keys (a question's number) when rendering lists.

## State, Storage, And Async Boundaries

- `storage/` is the only module that touches IndexedDB. Screens and
  components call storage functions with domain types; they never open a
  store. Durable state is IndexedDB only: never localStorage or
  sessionStorage.
- Keep record-to-domain conversion functions pure and browser-free so they
  unit-test on the host; keep the IndexedDB calls separate from them.
- UI signals hold transient view state. Persisted state (Profile, Answer Log
  entries, Exam records) changes only through `storage/`.
- Async UI (resources and futures) exists for storage work and the Test
  Update fetch. Render loading, empty, error, and success states distinctly;
  never hide a storage or export/import failure behind a generic string.

## Targets And Assets

- Web target only. Do not add desktop or mobile code paths or `cfg` branches
  for other targets, and do not copy desktop-oriented examples from the docs
  pack.
- The service worker, manifest, styles, and icons live in `public/`;
  precache lists are versioned.
- Styling is plain CSS (`public/styles.css`), linked from `index.html`.

## Formatting

- `cargo fmt` does not format `rsx!` bodies: run `dx fmt` after editing RSX.
  It is a writing convention, not a quality gate; there is no check mode.

## Tests

- The UI layer stays thin because the behavior worth testing lives in
  `domain` (`cargo test -p domain`).
- Host unit tests cover pure storage conversions; keep them browser-free.
- No `wasm-bindgen-test` or headless-browser tests in v1. A UI change is
  verified by running it (`dx serve`) and using the actual screen; the
  device-level pass is the PWA acceptance in `docs/open-work.md` §Platform.
- Prefer assertions on user-visible state or domain rules over incidental
  Dioxus implementation details.
- Treat repeated confusion around a Dioxus pattern as a signal to improve
  these standards or the library docs.
