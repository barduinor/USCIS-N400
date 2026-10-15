# The 2025 civics test is the only source of questions and rules

The repository holds two test instruments: M-1778 (2025) and M-638 (2008,
superseded). Questions and scoring rules come from the 2025 version
**exclusively**. The 2008 material is never a source for a question, an
Accepted Answer, or a rule.

## Considered Options

- **Merge both sets.** Rejected. The two instruments ask different questions,
  number them differently, and contradict each other on accepted answers.
  M-638 accepts "the War between the States" for a question the 2025 list
  accepts only as "The Civil War". Its Cabinet list predates the 2025 revision
  — it omits six positions the current list accepts, and names a department
  that has since been renamed. A merged bank would teach answers the officer
  rejects.
- **Use M-638 as the explanation source.** Rejected as a source of answers. Its
  per-question explanations cover only the 56 of 128 current questions with a
  strong or close 2008 counterpart, and its accepted answers diverge even on
  questions worded identically.
- **Adopt the study guide's 12-chapter grouping for Section and Topic.**
  Rejected. M-1175's chapters have no linkage to question numbers — there is
  not a single "question N" reference in the chapter prose — so the mapping
  would be hand-authored. M-1778's own headers yield 3 Sections and 8 Topics
  directly from the question list.

## Consequences

- M-638 stays in `docs/resources/` as reference only, marked superseded in its
  filename. Nothing reads it programmatically.
- The 2008 scoring rules — 100 questions, 10 asked, 6 to pass — must never
  appear in code, constants, or UI.
- Explanation coverage for all 128 questions remains an open problem. It cannot
  be solved by copying M-638.
- Section and Topic reporting follows M-1778: 3 Sections, 8 Topics, with
  contiguous question ranges.
