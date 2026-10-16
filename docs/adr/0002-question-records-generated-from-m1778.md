# The question records are generated from M-1778, then corrected by hand

The per-question records are produced by a parser that reads the 2025 question
list (M-1778) and emits a committed data file. Hand corrections are applied as a
separate overrides step, so the generated file is never edited directly.

## Considered Options

- **Hand-transcribe all 128.** Rejected. There are 410 answer strings to copy,
  with no way to re-check the result against the source.
- **Parse at build time.** Rejected. The source PDFs are 58 MB, and the hand
  corrections would still live outside the pipeline, so "re-derivable at build
  time" would not actually be true.

## Consequences

- The parser stays in the repo, so the records can be audited against M-1778 and
  regenerated when the source is reissued.
- **Never hand-edit the generated file.** Corrections belong in the overrides
  step, or the next regeneration silently discards them.
- Known corrections to make: the 5 square-bracket annotations that are not
  optional words, Q16 and Q19 (one bullet holding a comma list), and 2
  superscript artifacts where `19th` extracted as `19 th`.
- Explanation text is an empty field until the content problem is solved.
