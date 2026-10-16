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

### Structure

- Settled: Rust 2024 and Dioxus compiled to WASM, client-side only, IndexedDB,
  static hosting on GitHub Pages — ADR 0002, ADR 0003. The screens are listed in
  `CONTEXT.md` §Screens.
- Open: crate and module layout. **This is the first coding step.**

## Decided, not built

Each item below already has a decision behind it — none is waiting on a choice,
they are waiting on work.

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

### Platform

- **PWA manifest and service worker.** Offline use on a phone, and durable
  storage on iOS.
- **Export and import progress as JSON.** The only backup and the only way to
  move between devices. Required in v1, not later.

### Tuning

These are constants to adjust against real sessions, not decisions to revisit.

- **Bias weights** — least-asked, wrong-answer, section-balance.
- **Recent-ask penalty** — currently sized to leave a gap of roughly 8–10
  questions before a wrong answer returns.
