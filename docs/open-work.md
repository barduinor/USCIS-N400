# Open Work

## Decided, not built

Each item below already has a decision behind it; none is waiting on a choice,
they are waiting on work.

### Structure

- **The module tree.** The scaffold (ticket #2) is in place: the Cargo
  workspace of three crates (`domain`, `gen-questions`, `app`) per ADR 0006
  (platform per ADR 0003, ADR 0005), the toolchain pin, the error pattern per
  crate, the thin entry points, and the quality-gate hook
  (`.githooks/pre-commit`, `scripts/install-hooks`). The remaining modules and
  screens land with their behavior and tests, in this decided layout:

  ```
  crates/domain/src/          lib.rs, error.rs, question.rs, matching.rs, answers.rs,
                              selection.rs, progress.rs, exam.rs, profile.rs
  crates/domain/tests/        tests_questions_data.rs: the data-file contract
  crates/gen-questions/src/   lib.rs, main.rs, cmd.rs, error.rs, generate.rs,
                              parse.rs, overrides.rs
  crates/gen-questions/       overrides.json
  crates/gen-questions/tests/ tests_regeneration.rs, tests_records.rs
  crates/app/                 Dioxus.toml, index.html, public/styles.css
  crates/app/src/             main.rs, routes.rs, error.rs, questions.rs, screens/,
                              components/, storage/
  ```

### Test gates

- Four gates, in order, gates-only (never mutate): `cargo fmt --check`;
  `cargo clippy --no-deps --all-targets -- -D warnings`; `cargo test` (the
  full workspace suite, including the data-file contract and the regeneration
  byte-guard); `cargo check -p uscis-n400 --target wasm32-unknown-unknown`
  (host checks do not prove the wasm ship target compiles). Hook installed via
  `scripts/install-hooks`; CI runs the same commands fresh on push and pull
  requests, and deploy waits on them. The `app` crate is expected to compile
  for host; if it is wasm-only, lint and test narrow to
  `-p domain -p gen-questions`.
- Inner loop: `cargo test -p domain` (native, fast). Not adopted: coverage
  gating, mutation testing, and `wasm-bindgen-test`/headless-browser tests in
  v1; the UI receives its real pass in the smoke and device checks.

### Content

- **Explanations for all 128 questions.** Source them from
  `docs/resources/USCIS-2025-Civics-Test-Study-Guide-M-1175.pdf`. Its chapters
  carry the prose but have no linkage to question numbers, so this is a mapping
  job (question to chapter excerpt), not a writing job. See ADR 0001. Until
  then the `Explanation` field stays empty and the reveal shows Accepted Answers
  only.

### Data

- **Test Update lookup.** `uscis.gov/citizenship/testupdates` serves the four
  national officeholders, and one fetch serves every user.
- **Local Fact sources.** State senators, representative and governor need a
  per-state source. Harder than the national four; deliberately deferred.
- **Automatic retrieval (investigate).** Whether the four Test Updates and the
  per-state Local Facts can be fetched from the web automatically, so blanks
  become rare: USCIS test-updates for the four; state legislature and
  congressional sources for the senators, representative, and governor. Until
  then, a missing value leaves its question in play and any answer is judged
  wrong.

### Build and deploy

- **PWA assets.** `crates/app/index.html` (the shell template, with `div#main`
  the app mounts into) and `public/styles.css` are in place: plain CSS, custom
  properties as the token set, phone-first. Tailwind rejected: a Node/binary
  build step in the hook and CI for seven screens of single-user UI; classes
  are strings, so adopting it later stays cheap. Still to come with the offline
  ticket: `public/manifest.json`, the hand-written `sw.js` (cache-first shell
  precache under a versioned cache name), and icons.

### Platform

- **PWA manifest and service worker.** Offline use on a phone, and durable
  storage on iOS. The installed web app is the intended mode: iOS shows no
  install prompt, and since iOS 26 home-screen items open as web apps by
  default, so first run shows a one-time "Add to Home Screen" hint before
  study starts, and the hint says install first: the installed app has its own
  storage context, separate from Safari-browsed pages, and progress does not
  carry over between the two. Home-screen web apps are exempt from Safari's
  7-day eviction; the JSON export stays the backstop. Acceptance includes a
  real-iPhone pass: install, airplane mode, reload, answer, export.
