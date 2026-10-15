# Answer matching is normalized exact match, not fuzzy

An answer is correct when, after normalization, it equals one of the question's
Accepted Answers. Matching is deliberately not fuzzy and not semantic, because
a wrong accept is worse than a wrong reject — the Applicant cannot tell it
happened, and it silently corrupts Coverage.

## What normalization does

- Lowercase; hyphens and punctuation become spaces; whitespace collapses.
- Parenthetical content is optional, so both forms are generated:
  `(U.S.) Constitution` yields `constitution` and `us constitution`.
- Square brackets are annotations, not optional words, and are dropped. The one
  exception is Q120, where the bracket lists additional accepted answers.
- `or` and `/` split alternatives, but **only inside parentheses**. Splitting
  outside them breaks "work for local, state, or federal government".
- Ordinals are repaired: `19 th` extracted from the PDF becomes `19th`.

## Considered Options

- **Fuzzy or semantic matching.** Rejected. It trades visible failures for
  invisible ones, and it cannot say why it accepted something.
- **A pick-list for multi-item questions.** Rejected as the default. It converts
  recall into recognition, a weaker skill than the oral test measures.

## Consequences

- For multi-item questions, count the distinct Accepted Answers present in the
  input and compare against the number the question demands ("Name two").
- Count distinct **bullets**, not distinct generated variants. Some bullets
  produce two variants that both match, which would otherwise double-count.
- Q16 and Q19 are shaped differently — one bullet holds a comma list — so those
  two need the bullet split before counting.
- 48 of 128 questions have exactly one Accepted Answer, so a valid but unlisted
  phrasing will be marked wrong. The **Override** exists for exactly this, which
  is why it must count as correct rather than merely record a synonym.
- The Scoring Guidelines state that an Applicant passes by providing "a correct
  answer **or an alternative phrasing of the correct answer**". The real test
  therefore accepts phrasings that no answer key can enumerate. Exact matching
  is a deliberate simplification, not a claim of equivalence — and this is the
  policy basis for the Override.
