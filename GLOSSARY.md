# USCIS N-400 Civics Study

A single-user study tool for the civics portion of the U.S. naturalization
interview. It drills the applicant on the official 2025 civics test questions
and simulates the test itself.

## Sources

| File | Governs |
|---|---|
| `2025-Civics-Test-128-Questions-and-Answers-M-1778.pdf` | the 128 Civics Test Questions and their Accepted Answers |
| `USCIS-2025-Civics-Test-Study-Guide-M-1175.pdf` | Explanations, in chapter prose |
| `Test_Scoring_Guidelines.pdf` | how the Civics Test is administered, stopped and scored |
| `the-citizens-almanac-M-76.pdf` | rights, responsibilities, anthems, founding documents |
| `declarations-of-independence-and-constitution-M-654.pdf` | founding document text |
| `superseded-2008-learn-about-the-usa-M-638.pdf` | nothing. Superseded 2008 instrument, reference only |

## Language

**Applicant**:
The single person using this tool: someone preparing for the naturalization interview.
_Avoid_: User, student, player

**Civics Test**:
The oral test on U.S. government and history administered at the naturalization interview. The 2025 version asks up to 20 of 128 questions; 12 correct passes.
_Avoid_: Quiz

**Civics Test Question**:
One of the 128 official questions. Each has one or more Accepted Answers.
_Avoid_: Item, card

**Accepted Answer**:
One official answer to a Civics Test Question, as published by USCIS. Parenthetical words in an official answer are optional, not part of it.
_Avoid_: Correct answer, key

**Override**:
The Applicant's correction of an answer the tool marked wrong. Counts as correct for that attempt; it never changes the question's Accepted Answers.
_Avoid_: Dispute, appeal

**Verdict**:
What the tool decides about one typed answer: Correct or Wrong, judged before any Override.
_Avoid_: Result, score

**Explanation**:
Text that helps the Applicant understand a Civics Test Question. Sourced from the official study guide, and optional until that text exists.
_Avoid_: Answer, hint

**Test Update**:
A Civics Test Question whose Accepted Answer changes with elections or appointments. USCIS publishes current values at uscis.gov/citizenship/testupdates.
_Avoid_: Dynamic question

**Local Fact**:
An Accepted Answer that depends on where the Applicant lives: state senators, U.S. representative, governor, state capital.
_Avoid_: Dynamic question

**Section**:
One of the three top-level divisions of the 2025 civics test: American Government, American History, Symbols and Holidays.
_Avoid_: Category, domain, area

**Topic**:
A sub-division within a Section. The 2025 test has eight.
_Avoid_: Sub-section, chapter

**Exam**:
A scoring simulation of the Civics Test. Without the 65/20 Special Consideration, twenty questions are drawn at random from the 128, and the Exam ends the moment the Applicant has answered 12 correctly or 9 incorrectly. When the 65/20 Special Consideration applies, ten questions are drawn at random from the 20 starred questions, and the Exam ends at 6 correctly or 5 incorrectly. Either way it stops exactly as the officer stops the real test. Keeps its own record of outcome, score, and questions asked.
_Avoid_: Exam Run, Mock exam, Test

**Cleared**:
A Civics Test Question answered correctly three times in a row. A wrong answer makes it unclear again.
_Avoid_: Mastered, learned, known

**Coverage**:
The share of the Civics Test Questions that are Cleared (all 128, or only the 20 starred questions when the 65/20 Special Consideration applies). Falls as soon as a Cleared question is answered wrong.
_Avoid_: Proficiency, progress

**Answer Log**:
The append-only record of every answer the Applicant has given in study mode.
_Avoid_: History, results

**Profile**:
The Applicant's own facts: state, district, the two U.S. senators, the U.S. representative, the governor, the state capital, the four Test Updates, and whether the 65/20 Special Consideration applies.

**65/20 Special Consideration**:
Reduced scope for an Applicant aged 65 or older who has been a lawful permanent resident for 20 or more years: study only the 20 starred questions (the ones M-1778 marks with an asterisk), asked 10, 6 passes.

## Structure

Verbatim from M-1778, which numbers the questions contiguously in this order.

**American Government** (72 questions)

| Topic | Q | Range |
|---|---:|---|
| Principles of American Government | 15 | 1–15 |
| System of Government | 47 | 16–62 |
| Rights and Responsibilities | 10 | 63–72 |

**American History** (46 questions)

| Topic | Q | Range |
|---|---:|---|
| Colonial Period and Independence | 17 | 73–89 |
| 1800s | 10 | 90–99 |
| Recent American History and Other Important Historical Information | 19 | 100–118 |

**Symbols and Holidays** (10 questions)

| Topic | Q | Range |
|---|---:|---|
| Symbols | 6 | 119–124 |
| Holidays | 4 | 125–128 |

Topic names are USCIS's own. The last American History topic is long enough that
a display label may be needed in the UI; that is a presentation concern, not a
second name.

## Screens

| Screen | Purpose |
|---|---|
| Onboarding | First run only. Establishes the Profile, and with it the 65/20 flag and Coverage's denominator. |
| Main menu | Entry point. Shows Coverage. |
| Study | The endless question loop. No statistics. |
| Exam | One Exam, ending when the outcome is decided. No statistics. |
| Exam result | That Exam's outcome and score. |
| Stats | Global Coverage at the top, then the 3 Sections; opening a Section shows its Topics. Every figure carries a percentage and a count. Exam history below. |
| Profile | The same fields as onboarding, editable later, plus the JSON export and import of progress. |

## Relationships

- A **Section** contains one or more **Topics**
- A **Topic** contains one or more **Civics Test Questions**
- A **Civics Test Question** has one or more **Accepted Answers**
- An **Override** counts its answer as correct for that attempt and never changes the question's Accepted Answers
- A **Test Update** and a **Local Fact** take their value from the **Profile** or from USCIS, never from the published question list
- **Coverage** derives from the **Answer Log**, which records study answers only
- An **Exam** never appends to the **Answer Log**; it records its own outcome, score, and questions asked
- The **Profile** is established during onboarding, and its 65/20 flag narrows study selection, Coverage, and the Exam
- Each **Section** and **Topic** reports its own **Coverage**

## Rules

- Study mode is endless. There is no session, no fixed length, and no summary; the Applicant stops when they choose.
- Exam mode runs a single Exam at a time.
- The next question is chosen after every answer by taking the highest-weighted candidate. When the 65/20 Special Consideration applies, only the 20 starred questions are candidates. There is no batch planning and no fallback branch: relevance lives entirely in the weights.
- The weights favour questions asked least often, questions answered wrong, and Sections that have been under-served. They are tunable constants.
- A question asked recently is penalised, not blocked, so a wrong answer returns only after a gap of roughly 8–10 other questions. The penalty is tunable and never reaches zero, so a candidate always exists.
- **Cleared questions are never excluded.** They reappear whenever their weight wins, which is how the Applicant keeps seeing questions they got right.
- The one absolute exclusion: the same question is never asked twice in a row.
- After an answer the tool says only whether it was right. The Accepted Answers and the Explanation appear only when the Applicant asks, even when the answer was right. The reveal expands inline under the verdict; after a wrong verdict it also carries the Override control.
- When a Local Fact or Test Update value is missing from the Profile, its question stays in play and any answer is judged wrong; filling the value in the Profile fixes it.
- No statistics appear during a study or exam session. Coverage and exam history live in their own area, reached from the main menu.
- During an Exam the screen shows only the question: no counter and no verdict. The Exam stops the moment the outcome is fixed, so either would reveal the result before the result screen.
- The Exam is a practice simulation only. The real second examination (a retest 60 to 90 days after a failure, covering only the failed portions) is deliberately not modelled.

## Example dialogue

> **Dev:** "The Applicant answered a question right, wrong, right, right. Is it **Cleared**?"
> **Domain expert:** "No. The wrong answer reset it; it needs three in a row, so one more."

> **Dev:** "And right, wrong, right, right, right?"
> **Domain expert:** "Yes. It needs three in a row after the wrong answer, and those are three."

> **Dev:** "Is Holidays a **Section**?"
> **Domain expert:** "No, a **Topic**. It sits inside Symbols and Holidays."

## Flagged ambiguities

- "Proficiency" is resolved to a single measure: **Coverage**, the share of the Civics Test Questions currently Cleared. A separate rolling-accuracy number was considered and dropped: it would have tracked the same thing with a shorter window. Avoid the word "proficiency" in code and UI.
- "Dynamic question" was used for two unrelated things. Resolved: **Test Update** (changes with elections) versus **Local Fact** (depends on where the Applicant lives). They differ in what they depend on and in who can supply the value.
- The repository holds two test instruments: M-1778 (2025) and M-638 (2008, superseded). Resolved: questions and scoring rules come from the 2025 version **exclusively**. The 2008 material is never a source for a question, an Accepted Answer, or a rule.
- Exam answers deliberately never feed **Coverage** or **Cleared**. An **Exam** is scored and recorded on its own, so that a lucky exam cannot clear a question the Applicant has never studied.
- "Study Set" and "question bank" were used for the Applicant's scope. Resolved: neither is an entity; the questions are the 128 **Civics Test Questions**, and the 65/20 flag narrows them to the 20 starred ones. Study selection, Coverage, and the Exam all apply that single rule.
