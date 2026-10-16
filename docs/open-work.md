# Open Work

## Open decisions

Not yet decided. Each needs a conversation, not effort. What is **already
settled** is referenced, so the thread can be picked up without the original
discussion.

### Study screen detail

- Settled: the verdict appears immediately; Accepted Answers and Explanation sit
  behind a tap; Override appears only on a wrong verdict; Next is manual; no
  counter and no statistics in-session — `CONTEXT.md` §Rules, §Screens.
- Open: whether the reveal opens inline or as an overlay, and where the
  Override control sits.

### Onboarding

- Settled: it runs before the first question, because it fixes the 65/20 flag and
  therefore Coverage's denominator; it collects state, district, the three
  local names, the four Test Updates, and whether the 65/20 Special
  Consideration applies — `CONTEXT.md` §Rules, §Language.
- Open: the exact field list, the order they are asked in, and what happens when
  a field is left blank.

## Decided, not built

Each item below already has a decision behind it — none is waiting on a choice,
they are waiting on work.

### Structure

- **The scaffold.** A Cargo workspace of three crates — `domain`,
  `gen-questions`, `app` — per ADR 0006 (platform per ADR 0003, ADR 0005).
  The module tree:

  ```
  domain/src/          lib.rs, error.rs, question.rs, matching.rs, answers.rs,
                       selection.rs, progress.rs, exam.rs, profile.rs
  domain/tests/        tests_questions_data.rs — the data-file contract
  gen-questions/src/   lib.rs, main.rs, cmd.rs, error.rs, generate.rs,
                       parse.rs, overrides.rs
  gen-questions/       overrides.json
  gen-questions/tests/ tests_regeneration.rs
  app/src/             main.rs, routes.rs, error.rs, questions.rs, screens/,
                       components/, storage/
  ```

  Ships a compiling skeleton, not empty stubs: workspace and member manifests,
  the quality-gate hook (`.githooks/pre-commit`, `scripts/install-hooks`), the
  error pattern per crate, thin entry points, and one working vertical slice —
  `gen-questions` producing `data/questions.json`, the `domain` types and the
  contract test, and the app shell rendering the main menu. Every other module
  and screen lands with its behavior and tests. **This is the first coding
  step.**

### Test gates

- Four gates, in order, gates-only (never mutate): `cargo fmt --check`;
  `cargo clippy --no-deps --all-targets -- -D warnings`; `cargo test` (the
  full workspace suite, including the data-file contract and the regeneration
  byte-guard); `cargo check -p uscis-n400 --target wasm32-unknown-unknown`
  (host checks do not prove the wasm ship target compiles). Hook installed via
  `scripts/install-hooks`; CI runs the same commands fresh on push and pull
  requests, and deploy waits on them. The `app` crate is expected to compile
  for host — if it is wasm-only, lint and test narrow to
  `-p domain -p gen-questions`.
- Inner loop: `cargo test -p domain` (native, fast). Not adopted: coverage
  gating, mutation testing, and `wasm-bindgen-test`/headless-browser tests in
  v1 — the UI receives its real pass in the smoke and device checks.

### Storage

- **Storage layer and export/import.** One IndexedDB database; the record
  shapes, the derived-Override rule, and the replace-on-import contract are in
  ADR 0007. Export/import is required in v1 (ADR 0003). Exact Profile fields
  wait on the onboarding decision; the Profile is stored as one record, so
  that decision does not change the shape.

### Question records

- **The parser pipeline.** `gen-questions` extracts text from the committed
  M-1778 PDF with `pdf-extract` (pure Rust, no system dependencies), parses
  Sections, Topics, questions, bullets, the 20 asterisks, and the
  required-answer counts from the "Name <u>two</u>" underlines, then applies
  `overrides.json` and writes `data/questions.json`. The generated file is
  clean: display and matching both consume it, and overrides are the only
  correction path (ADR 0002). Known corrections: the five bracket annotations
  (Q23, Q29, Q61, Q62 dropped; Q120 adds Accepted Answers), Q16/Q19
  comma-lists split, `19 th` → `19th`.
- **Tests.** Parsing units on small text fixtures, plus the regeneration guard:
  the full pipeline against the PDF, byte-compared with the committed records.
  Extractor upgrades that change output surface as guard diffs — deliberate.

### Content

- **Explanations for all 128 questions.** Source them from
  `docs/resources/USCIS-2025-Civics-Test-Study-Guide-M-1175.pdf`. Its chapters
  carry the prose but have no linkage to question numbers, so this is a mapping
  job — question to chapter excerpt — not a writing job. See ADR 0001. Until
  then the `Explanation` field stays empty and the reveal shows Accepted Answers
  only.

### Profile

- **Onboarding.** Collects state, district, the three local names, the four Test
  Updates, and whether the 65/20 Special Consideration applies. Must run before
  the first question, because it fixes the 65/20 flag and therefore Coverage's
  denominator.

### Data

- **Test Update lookup.** `uscis.gov/citizenship/testupdates` serves the four
  national officeholders, and one fetch serves every user.
- **Local Fact sources.** State senators, representative and governor need a
  per-state source. Harder than the national four; deliberately deferred.

### Build and deploy

- **Styling.** Plain CSS: one `styles.css` linked from the index template,
  CSS custom properties as the token set. Tailwind rejected — a Node/binary
  build step in the hook and CI for seven screens of single-user UI; classes
  are strings, so adopting it later stays cheap.
- **App shell.** A custom `index.html` (required to load the service worker
  and manifest; must contain `div#main`) plus `public/`: `manifest.json`,
  `sw.js` (hand-written, cache-first shell precache under a versioned cache
  name), `styles.css`, icons.
- **Deploy.** GitHub Actions on push to `main`: the three gates, then pinned
  `dx` and `wasm32-unknown-unknown`, `dx build --release`, the Pages artifact,
  `actions/deploy-pages`. Pages source is "GitHub Actions"; the site lives
  under `/USCIS-N400/`, so the base path must match — it also feeds the
  manifest scope and the service worker registration. `rust-toolchain.toml`
  pins the toolchain.

### Platform

- **PWA manifest and service worker.** Offline use on a phone, and durable
  storage on iOS. The installed web app is the intended mode: iOS shows no
  install prompt, and since iOS 26 home-screen items open as web apps by
  default — so first run shows a one-time "Add to Home Screen" hint before
  study starts, and the hint says install first: the installed app has its own
  storage context, separate from Safari-browsed pages, and progress does not
  carry over between the two. Home-screen web apps are exempt from Safari's
  7-day eviction; the JSON export stays the backstop. Acceptance includes a
  real-iPhone pass: install, airplane mode, reload, answer, export.
- **Export and import progress as JSON.** The only backup and the only way to
  move between devices. Required in v1, not later.

### Tuning

These are constants to adjust against real sessions, not decisions to revisit.

- **Bias weights** — least-asked, wrong-answer, section-balance.
- **Recent-ask penalty** — currently sized to leave a gap of roughly 8–10
  questions before a wrong answer returns.
