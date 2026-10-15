# USCIS N-400 Civics Study

A single-user study tool for the civics portion of the U.S. naturalization
interview. It drills the applicant on the official 2025 civics test questions
and simulates the test itself.

## Language

**Applicant**:
The single person using this tool — someone preparing for the naturalization interview.
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
The Applicant's correction of a question the tool marked wrong. Counts as correct, and adds the typed phrasing to that question's Accepted Answers.
_Avoid_: Dispute, appeal

**Explanation**:
Text that helps the Applicant understand a Civics Test Question. Sourced from the official study guide, and optional until that text exists.
_Avoid_: Answer, hint

**Test Update**:
A Civics Test Question whose Accepted Answer changes with elections or appointments. USCIS publishes current values at uscis.gov/citizenship/testupdates.
_Avoid_: Dynamic question

**Local Fact**:
An Accepted Answer that depends on where the Applicant lives — state senators, U.S. representative, governor, state capital.
_Avoid_: Dynamic question

**Section**:
One of the three top-level divisions of the 2025 civics test: American Government, American History, Symbols and Holidays.
_Avoid_: Category, domain, area

**Topic**:
A sub-division within a Section. The 2025 test has eight.
_Avoid_: Sub-section, chapter

**Exam**:
A scoring simulation of the Civics Test: questions drawn without weighting, ending as soon as the outcome is decided. Keeps its own record of outcome, score, and questions asked.
_Avoid_: Exam Run, Mock exam, Test

**Cleared**:
A Civics Test Question answered correctly three times in a row. A wrong answer makes it unclear again.
_Avoid_: Mastered, learned, known

**Study Set**:
The questions the Applicant is accountable for: all 128, or only the 20 starred questions when the 65/20 Special Consideration applies. Fixed during onboarding.
_Avoid_: Question bank, deck

**Coverage**:
The share of the **Study Set** that is Cleared. Falls as soon as a Cleared question is answered wrong.
_Avoid_: Proficiency, progress

**Answer Log**:
The append-only record of every answer the Applicant has given in study mode.
_Avoid_: History, results

**Profile**:
The Applicant's own facts: state, district, current officeholders, and whether the 65/20 Special Consideration applies.

**65/20 Special Consideration**:
Reduced scope for an Applicant aged 65 or older who has been a lawful permanent resident for 20 or more years: study only the 20 starred questions, asked 10, 6 passes.

## Structure

Verbatim from M-1778, which numbers the questions contiguously in this order.

**American Government** — 72 questions

| Topic | Q | Range |
|---|---:|---|
| Principles of American Government | 15 | 1–15 |
| System of Government | 47 | 16–62 |
| Rights and Responsibilities | 10 | 63–72 |

**American History** — 46 questions

| Topic | Q | Range |
|---|---:|---|
| Colonial Period and Independence | 17 | 73–89 |
| 1800s | 10 | 90–99 |
| Recent American History and Other Important Historical Information | 19 | 100–118 |

**Symbols and Holidays** — 10 questions

| Topic | Q | Range |
|---|---:|---|
| Symbols | 6 | 119–124 |
| Holidays | 4 | 125–128 |

Topic names are USCIS's own. The last American History topic is long enough that
a display label may be needed in the UI; that is a presentation concern, not a
second name.

## Relationships

- A **Section** contains one or more **Topics**
- A **Topic** contains one or more **Civics Test Questions**
- A **Civics Test Question** has one or more **Accepted Answers**
- An **Override** adds an **Accepted Answer** for that question and never counts as a wrong answer
- A **Test Update** and a **Local Fact** take their value from the **Profile** or from USCIS, never from the published question list
- **Coverage** derives from the **Answer Log**, which records study answers only
- An **Exam** never appends to the **Answer Log**; it records its own outcome, score, and questions asked
- The **Profile** is established during onboarding, and it fixes the **Study Set**
- Each **Section** and **Topic** reports its own **Coverage**

## Rules

- Study mode is endless. There is no session, no fixed length, and no summary; the Applicant stops when they choose.
- Exam mode runs a single Exam at a time.
- The next question is chosen after every answer by taking the highest-weighted candidate. There is no batch planning and no fallback branch — relevance lives entirely in the weights.
- The weights favour questions asked least often, questions answered wrong, and Sections that have been under-served. They are tunable constants.
- A question asked recently is penalised, not blocked, so a wrong answer returns only after a gap of roughly 8–10 other questions. The penalty is tunable and never reaches zero, so a candidate always exists.
- **Cleared questions are never excluded.** They reappear whenever their weight wins, which is how the Applicant keeps seeing questions they got right.
- The one absolute exclusion: the same question is never asked twice in a row.
- After an answer the tool says only whether it was right. The Accepted Answers and the Explanation appear only when the Applicant asks — even when the answer was right.

## Example dialogue

> **Dev:** "The Applicant answered a question right, wrong, right, right, right. Is it **Cleared**?"
> **Domain expert:** "No. The wrong answer reset it — it needs three in a row, so one more."

> **Dev:** "Is Holidays a **Section**?"
> **Domain expert:** "No, a **Topic**. It sits inside Symbols and Holidays."

## Flagged ambiguities

- "Proficiency" is resolved to a single measure: **Coverage**, the share of the study set currently Cleared. A separate rolling-accuracy number was considered and dropped — it would have tracked the same thing with a shorter window. Avoid the word "proficiency" in code and UI.
- "Dynamic question" was used for two unrelated things. Resolved: **Test Update** (changes with elections) versus **Local Fact** (depends on where the Applicant lives). They differ in what they depend on and in who can supply the value.
- The repository holds two test instruments: M-1778 (2025) and M-638 (2008, superseded). Resolved: questions and scoring rules come from the 2025 version **exclusively**. The 2008 material is never a source for a question, an Accepted Answer, or a rule.
- Exam answers deliberately never feed **Coverage** or **Cleared**. An **Exam** is scored and recorded on its own, so that a lucky exam cannot clear a question the Applicant has never studied.
