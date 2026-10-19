# Reviewed alternatives attach to a published Accepted Answer

An **Alternative Answer** is a reviewed phrasing that counts as one published
Accepted Answer. It is stored in the generated records as an attachment:
question number, the published answer it stands for, and the reviewed form. The
matcher reads the form beside the published answer's own forms, so the
published list remains USCIS's list and a multi-item question still counts one
answer once, however many forms of it the Applicant says.

Alternatives live in `crates/gen-questions/alternatives.json`, beside the data
corrections in `overrides.json`, and the generator emits them into
`crates/app/data/questions.json`. Profile-backed questions are excluded: their
accepted answer is resolved from the Profile at judging time, so a static form
cannot attach to it. A narrow fact-only short-form and typo rule covers those
eight questions instead.

## Considered Options

- **Append alternatives to `accepted_answers`.** Rejected. A paraphrase would
  become a second Accepted Answer, and a "Name two" question could count two
  forms of the same answer.
- **Put alternatives on the `AcceptedAnswer` value.** Rejected. The record is a
  transparent string; changing its shape rewrites every record and every
  construction site.
- **Fuzzy or semantic matching.** Rejected for the reason in ADR 0004: a wrong
  accept is invisible to the Applicant and silently corrupts Coverage.
  Alternatives are reviewed data, not a similarity threshold.
- **A dedicated Override store.** Rejected by ADR 0007. Alternatives are a
  matcher input, not progress.

## Consequences

- The question records carry a second list. The data contract test checks that
  every alternative's `answer` is published by its question, that no
  Profile-backed question carries one, and that no question repeats a form.
- The matcher gains a reviewed path; the exact `judge` remains for the
  published answers and the existing tests.
- The first batch comes from
  `docs/research/which-overrides-are-matching-false-negatives.md`.
