# Answer matching is normalized exact match, not fuzzy

An answer is correct when, after normalization, it equals one of the question's
Accepted Answers; on a question that asks for one answer, it may also join two
or more of their forms with no other words. Matching is deliberately not fuzzy
and not semantic, because a wrong accept is worse than a wrong reject: the
Applicant cannot tell it happened, and it silently corrupts Coverage.

## What normalization does

- Lowercase; hyphens and punctuation become spaces; whitespace collapses.
- Parenthetical words are read by the shape they take, the three named in
  `GLOSSARY.md`: a fragment is accepted present or dropped, an alternative
  spelling stands in place of the answer's own words, and a numeral form buys
  the digit form of the number before it. See Parenthetical shapes.
- Square brackets are annotations, not optional words, and are dropped. The one
  exception is Q120, where the bracket lists additional accepted answers.
- `or` and `/` split alternatives, but **only inside parentheses**. Splitting
  outside them breaks "work for local, state, or federal government".
- Ordinals are repaired: `19 th` extracted from the PDF becomes `19th`.

## Parenthetical shapes

M-1778 prints a parenthetical for three different jobs, and each job gets its
own rule.

**A fragment is accepted present or dropped, never alone.** Its words only mean
something beside the rest of the answer: `(U.S.) Constitution` accepts
`us constitution` and `constitution`, never `us`; so `(Example of) social
contract`, `(The basic) rights of Americans`, and `Citizens in their
(congressional) district`.

**An alternative spelling stands in the answer's place.** Its words spell the
answer another way, as a menu joined by `or` or `/`: `(U.S. or national)
legislature` accepts `U.S. legislature` and `national legislature`, and `Saved
(or preserved) the Union` accepts `Saved the Union` and `Preserved the Union`,
correcting the word before it rather than adding one.

**A numeral form buys the digit form of the number before it.** A parenthetical
that is nothing but digits, and that follows a spelled-out number, buys that
number's digit form: the answer also accepts the same words with the number
replaced by the parenthetical's digits and the parenthetical dropped.
`Twenty-seven (27)` accepts `27`; `Six (6) years` accepts `6 years`;
`Citizens eighteen (18) and older (can vote).` accepts `citizens 18 and older`.
The ten answers it covers, by question number:

| Q | Accepted Answer | Digit form |
|---|---:|---|
| 7 | `Twenty-seven (27)` | `27` |
| 21 | `One hundred (100)` | `100` |
| 22 | `Six (6) years` | `6 years` |
| 24 | `Four hundred thirty-five (435)` | `435` |
| 25 | `Two (2) years` | `2 years` |
| 27 | `Two (2)` | `2` |
| 36 | `Four (4) years` | `4 years` |
| 53 | `Nine (9)` | `9` |
| 54 | `Five (5)` | `5` |
| 63 | `Citizens eighteen (18) and older (can vote).` | `citizens 18 and older` |

A numeral parenthetical that follows no spelled-out number stays a fragment, and
its digits alone are never an answer. Q104 prints the only one: `The Great Crash
(1929)` accepts `the great crash 1929` and `the great crash`, never `1929`. The
crash is what the question asks for — its second Accepted Answer is `Stock
market crash of 1929` — so a bare year is not a shorter spelling of the answer.

Q28's other name is the one case no rule reads: `The Great Compromise
(Connecticut Compromise)` is the same compromise under its other name, but the
parenthetical is neither an `or`/`/` menu nor a numeral, so the rule makes it a
fragment and `connecticut compromise` alone is not an answer. Telling the two
apart would mean reading each parenthetical's prose for whether it renames the
answer — a judgment call per answer, which is what the overrides file is for
(ADR 0002), not a rule the notation marks. `Connecticut Compromise` is
therefore listed as an Accepted Answer of its own in
`crates/gen-questions/overrides.json`, with its reason beside it, and reaches
the committed records like any other published answer.

### The inventory

Every parenthetical in the committed records, classified — 108 bare
parentheticals across 64 questions, none left undefined. Eleven hold nothing but
digits, across 11 questions (Q7, Q21, Q22, Q24, Q25, Q27, Q36, Q53, Q54, Q63,
Q104); the other 97 hold words, across 54 questions.

| Shape | Parentheticals | Questions |
|---|---:|---:|
| fragment | 96 | 55 |
| numeral form | 10 | 10 |
| alternative spelling | 2 | 2 |
| **bare total** | **108** | **64** |

The digits are the ten numeral forms and Q104's fragment; the words are 95
fragments and the two alternative spellings, `(U.S. or national)` in Q18 and
`(or preserved)` in Q94. One further parenthetical sits in the records without
being bare: Q63's `(Women and men can vote.)` holds a whole sentence, and the
rule reads it as a fragment, present or dropped.

Applied by hand, the rule gives:

| Q | Published answer | Typed | Verdict |
|---|---|---:|---|
| 7 | `Twenty-seven (27)` | `27` | Correct, the numeral form |
| 22 | `Six (6) years` | `6 years` | Correct |
| 63 | `Citizens eighteen (18) and older (can vote).` | `citizens 18 and older` | Correct |
| 2 | `(U.S.) Constitution` | `us` | Wrong, a fragment's words alone |
| 28 | `Equal representation (for small states)` | `for small states` | Wrong |
| 104 | `The Great Crash (1929)` | `1929` | Wrong |

## A single-answer question joins forms

A question that asks for one answer (`answers_required` is 1) is correct when
the normalized typed answer equals one of the question's forms, or joins two or
more of those forms with no other words. A joining reads the typed words as
consecutive runs of whole words, each run a form of one Accepted Answer, with
every typed word inside a run.

The Applicant may say more than one correct answer in a line. "Where is the
Statue of Liberty?" lists five, and `liberty island new york` joins `Liberty
Island` with the `New York` form of `New York (Harbor)`, its parenthetical
dropped; either order passes. Both runs may also come from one answer, as
`27 twenty seven` does for the amendments question. The rule is mechanical
like the rest of the matcher and reads no negation: `not liberty island` fails
because `not` is no form's word, not because the word is understood.

The words must be the answers' own. `statue island` and `the statue` join
nothing, because no run of them is a form. A word that belongs to an answer but
makes no form of its own stays out: `the liberty island` is wrong, because
`the` alone is no form, although `on the Hudson (River)` holds it. Multi-item
questions are unchanged by this rule: they still count the distinct Accepted
Answers the typed answer carries.

## Considered Options

- **Fuzzy or semantic matching.** Rejected. It trades visible failures for
  invisible ones, and it cannot say why it accepted something.
- **A pick-list for multi-item questions.** Rejected as the default. It converts
  recall into recognition, a weaker skill than the oral test measures.
- **Accept any parenthetical's content as an answer.** Rejected: it reads Q104's
  `(1929)` as an answer, and a bare year does not say what started. The tool
  would accept a plausible wrong answer, the one failure matching exists to
  prevent. Only the two shapes that really hold an answer are read that way —
  the `or`/`/` menu and the numeral form — and each is named, not inferred.
- **Accept a form beside other words for a single-answer question.** Rejected:
  containing a form is not saying it. `not liberty island` contains `liberty
  island`, and the tool would accept a line an officer would not. A joining
  covers every typed word with the answers' own forms instead, so a word beside
  them refuses the line.

## Consequences

- For a single-answer question, accept the typed answer when it equals one of
  the question's forms or joins two or more of them with no other words. The
  runs cover the whole line, in any order: `liberty island new york` passes for
  Q120, `not liberty island` does not.
- For multi-item questions, count the distinct Accepted Answers present in the
  input and compare against the number the question demands ("Name two").
- Count distinct **bullets**, not distinct generated variants. Some bullets
  produce two variants that both match, which would otherwise double-count; a
  numeral form is another variant of its own bullet, never a second answer.
- Q16 and Q19 are shaped differently (one bullet holds a comma list), so those
  two need the bullet split before counting.
- The ten numeral answers accept their printed digits — `27`, `100`, `6 years`,
  `435`, `2 years`, `2`, `4 years`, `9`, `5`, and `citizens 18 and older` — and
  no other answer gains a form: `1929` alone stays wrong for Q104, `us` stays
  wrong for Q2, and a fragment's words alone stay wrong everywhere.
- Q28 carries three Accepted Answers: the two M-1778 prints and
  `Connecticut Compromise`, which comes from the overrides.
- 47 of 128 questions have exactly one Accepted Answer, so a valid but unlisted
  phrasing will be marked wrong. The **Override** exists for exactly this, which
  is why it must count as correct rather than merely record a synonym.
- The Scoring Guidelines state that an Applicant passes by providing "a correct
  answer **or an alternative phrasing of the correct answer**". The real test
  therefore accepts phrasings that no answer key can enumerate. Exact matching
  is a deliberate simplification, not a claim of equivalence. This is the
  policy basis for the Override.
