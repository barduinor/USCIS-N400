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
One official answer to a Civics Test Question, as published by USCIS. Parenthetical words in an official answer are read by their shape, not taken as part of it; see **Parenthetical**.
_Avoid_: Correct answer, key

**Alternative Answer**:
A reviewed phrasing that counts as one published Accepted Answer. It is not a second answer: it never changes the published list and does not increase a multi-item question's count.
_Avoid_: Synonym, user answer, override

**Parenthetical**:
A bracketed stretch inside an Accepted Answer, read by the shape it takes. A _fragment_ only means something beside the rest of the answer, so the answer is accepted with it and without it, never its words alone (`(U.S.) Constitution` accepts `us constitution` and `constitution`, never `us`; `The Great Crash (1929)` never accepts `1929`). An _alternative spelling_ holds words that stand in the answer's place, spelled as a menu with `or` or `/` (`(U.S. or national) legislature` accepts `U.S. legislature` and `national legislature`). A _numeral form_ follows a spelled-out number and buys that number's digits (`Twenty-seven (27)` accepts `27`). ADR 0004 owns the rule.
_Avoid_: Bracketed text, optional words

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
A Civics Test Question whose Accepted Answer changes with elections or appointments. USCIS publishes current values at uscis.gov/citizenship/testupdates. The four national values ship with the release as generated data (ADR 0010).
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
A Civics Test Question answered correctly for the first time while it has no wrong answer against it. After a wrong answer it takes three correct answers in a row. A wrong answer makes it unclear again.
_Avoid_: Mastered, learned, known

**Coverage**:
The share of the Civics Test Questions that are Cleared (all 128, or only the 20 starred questions when the 65/20 Special Consideration applies). Falls as soon as a Cleared question is answered wrong, and rises again when the question is Cleared anew.
_Avoid_: Proficiency, progress

**Answer Log**:
The append-only record of every answer the Applicant has given in study mode.
Only an explicit Reset, an explicit Clear, or a confirmed import removes
entries; a reload, a resume, or a failed read never does. A read that comes back
empty while the device's own count says the log held answers is retried, never
rendered as `0%`.
_Avoid_: History, results

**Profile**:
The Applicant's own facts: state, district, the two U.S. senators, the U.S. representative, the governor, the state capital, the four Test Updates, and whether the 65/20 Special Consideration applies.

**Clear**:
The Applicant's explicit emptying of one history: the Answer Log, or the Exam records. A Clear leaves the Profile and the other history in place, and never returns the app to onboarding. Each Clear has its own confirmation. Unlike a Reset, which empties every store.
_Avoid_: Delete, wipe, reset

**Import**:
Reading a bundle onto this device. With no local Answer Log or Exam records, it offers the replace confirmation. With either present, it asks the Applicant to keep the device's data, replace it with the bundle's, or merge. **Merge** keeps the device Profile, combines both histories sorted by time, and drops exact duplicate records. Keep writes nothing.
_Avoid_: Restore, load

**Reset**:
The Applicant's start over on this device: it empties all four stores — the Profile, the Answer Log, the Exam records, and the meta record — and returns the app to onboarding. It happens only after a confirmation, which can export a bundle first. Unlike an Import, a Reset leaves no state behind. The Accepted Answers, the app, the worker, and its cache stay.
_Avoid_: Wipe, factory reset, clear data

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
- A **Reset** empties every store on the device; an **import** replaces their contents with a bundle's
- Each **Section** and **Topic** reports its own **Coverage**

## Rules

- Study mode is endless. There is no session, no fixed length, and no summary; the Applicant stops when they choose.
- Exam mode runs a single Exam at a time.
- The next question is chosen after every answer by taking the highest-weighted candidate. When the 65/20 Special Consideration applies, only the 20 starred questions are candidates. There is no batch planning and no fallback branch: relevance lives entirely in the weights.
- The weights favour questions asked least often, questions answered wrong, and Sections that have been under-served. They are tunable constants.
- A question asked recently is penalised, not blocked, so a wrong answer returns only after a gap of roughly 8–10 other questions. The penalty is tunable and never reaches zero, so a candidate always exists.
- **Cleared questions are never excluded.** They reappear whenever their weight wins, which is how the Applicant keeps seeing questions they got right.
- The one absolute exclusion: the same question is never asked twice in a row.
- After an answer the tool says whether it was right, shows the answer the Applicant gave, and shows the Accepted Answers, so the two can be compared. The Accepted Answers appear after every verdict, correct or wrong. The Explanation appears only when the Applicant asks. After a wrong verdict the screen also offers the Override, below Next question and separated from it, so it is never the accidental tap.
- When a Local Fact value is missing from the Profile, its question stays in
  play and any answer is judged wrong; filling the value in the Profile fixes
  it. A Test Update value ships with the release: a blank Profile field presents
  the shipped value, and a value the Applicant stored wins.
- No global statistics appear during a study or exam session. Coverage and exam history live in their own area, reached from the main menu. The study screen carries the asked question's official number, which identifies it in the published list and is not a progress figure.
- During an Exam the screen carries the current Exam's own progress: the official question number, the exam position, the official numbers asked so far in this Exam, the right and wrong counts, and the verdict of the last answer. It waits for the Applicant after each answer: Next question while the Exam runs, or See result when the outcome is fixed.
- During an Exam the Accepted Answers are not shown. The result screen carries the outcome and score.
- The Exam is a practice simulation only. The real second examination (a retest 60 to 90 days after a failure, covering only the failed portions) is deliberately not modelled.
- A Reset happens only after its confirmation; cancelling writes nothing. It deletes the Profile, the Answer Log, and the Exam history from the device, never the Accepted Answers, the app, the worker, or its cache.
- A Clear happens only after its own confirmation. Clearing the study answers empties the Answer Log and leaves the Profile and the Exam history; clearing the Exam history empties the Exam records and leaves the Profile and the Answer Log. Neither returns the app to onboarding.
- An Import never clears local history without a choice. When the device holds answers or Exams, the Applicant chooses Keep, Replace, or Merge; when it holds neither, Replace is the only path. A local read that fails pauses the import.
- No reload, resume, failed read, or stale page snapshot clears or overwrites the Answer Log or the Exam records. An empty read is checked against the count the device recorded before any screen shows `0%`.
- The app asks the browser to keep this origin's storage, so the progress is not evicted under storage pressure; a refusal changes nothing.

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
