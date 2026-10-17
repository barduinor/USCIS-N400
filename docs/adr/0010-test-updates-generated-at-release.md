# The Test Update values ship as generated data

The four national Test Update values — president, vice president, speaker of
the House, and chief justice — are generated when the app is released. A
host-side step in `gen-questions` fetches
`uscis.gov/citizenship/testupdates`, the page M-1778 points at, and writes the
values, the source URL, and the fetch date into
`crates/app/data/test-updates.json`. The application embeds that file, fills a
blank Profile field from it, and never fetches the page at runtime.

The Profile still stores only what the Applicant saved. A shipped value stands
in at display and judging time; a stored value always wins, including after the
data is regenerated. There is no scheduled refresh.

## Considered Options

- **Fetch at runtime.** Rejected. The app has no server (ADR 0003) and has to
  work offline on a phone; a fetch per Applicant would make the fields depend on
  a request that may not happen.
- **Hand-transcribe the four values into the Profile defaults.** Rejected. It
  cannot be re-checked against the source or regenerated when the officeholders
  change, which is the same reason the question records are generated (ADR 0002).
- **A scheduled refresh.** Rejected. It needs a server or a build trigger that
  runs without the release, and the shipped values are deliberately as fresh as
  the release.

## Consequences

- A release can ship stale values until it is rebuilt. The Profile says the
  values change with elections and appointments and names the USCIS page to
  check.
- The generation step fails rather than writing a partial set, and its committed
  output carries its source and fetch date so it can be checked offline.
- The state-level Local Facts remain a separate, deferred problem: they depend
  on where the Applicant lives, so one shipped set cannot serve every
  Applicant.
