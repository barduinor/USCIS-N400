# Which overridden answers are matching false negatives?

**Question.** Which answers that the tool judged Wrong and the Applicant
overrode are substantively correct, and what should the matcher do about them?

**Answer.** All 51 overridden entries are substantively correct. The 22 wrong
verdicts the Applicant did not override contain 11 more substantively correct
answers and 11 genuinely wrong ones. The exact matcher's observed error in this
log is therefore **62 false negatives out of 73 wrong verdicts (85%)**, not 51.
Filtering only on `overridden` misses 11 of the 62.

This note records the counts, the reviewed list, the judgment behind the
labels, and a staged proposal. It is a research note, not an ADR; the matching
policy remains ADR 0004 until a replacement is written.

## Method

The source is the export bundle supplied as `local://paste-1.md`, exported
2026-10-17T16:15:35Z. It is schema version 1 and holds 151 Answer Log entries
(`crates/app/src/storage/bundle.rs:35-44`). The bundle's Answer Log is the
record the study screen appends to; an Override marks the last entry and never
rewrites its typed answer (`crates/app/src/storage/answer_log.rs`).

The accepted answers were joined from `crates/app/data/questions.json`. For the
eight Profile-backed questions, the accepted answers are resolved from the
Profile at export time, as the app does at judging time
(`crates/domain/src/answers.rs:31-39`). The Profile values used:

| Q | Fact | Profile value |
|---:|---|---|
| 23 | State senators | Jon Ossoff; Raphael Warnock |
| 29 | U.S. representative | Hank Johnson |
| 30 | Speaker of the House | Mike Johnson |
| 38 | President | Donald Trump |
| 39 | Vice President | JD Vance |
| 57 | Chief Justice | John G. Roberts Jr |
| 61 | Governor | Brian Kemp |
| 62 | State capital | Atlanta |

The matching policy under review is normalized exact match: lowercase, drop
periods, read other punctuation and hyphens as spaces, collapse whitespace,
repair a stray ordinal space, and read the parenthetical shapes defined in
`GLOSSARY.md` and ADR 0004. It is deliberately not fuzzy and not semantic
(`crates/domain/src/matching.rs`, ADR 0004).

The review standard for **substantively correct** was: the typed answer names
the same fact as one Accepted Answer, allowing a typo, a missing or extra
article, a preposition or determiner change, word order, tense, a shortened
name, a fragment that is the answer's core, or a paraphrase that preserves the
fact. A wrong fact, a placeholder, or a vague answer that does not name a
listed reason was not counted as correct. This standard is a judgment, not a
USCIS adjudication.

## Stats

**Verifiable from the bundle:**

| Measure | Count |
|---|---:|
| Answers logged | 151 |
| Correct verdicts | 78 |
| Wrong verdicts | 73 |
| Overridden | 51 |
| Wrong, not overridden | 22 |
| Distinct questions overridden | 51 |
| Profile-backed overrides | 3 (Q23, Q38, Q57) |

Override rate: 51 of 73 wrong verdicts (70%), or 51 of 151 answers (34%). Every
overridden question was overridden once; no question has two Override entries.

**Reviewed outcome (judgment):**

| Measure | Count | Share of wrong verdicts |
|---|---:|---:|
| Overrides that are substantively correct | 51 | 70% |
| Non-overridden wrongs that are substantively correct | 11 | 15% |
| Total false negatives | 62 | 85% |
| True negatives (genuinely wrong) | 11 | 15% |

The 78 correct verdicts were **not** reviewed for false accepts. The figures
above measure false negatives only. A matcher change that ignores false accepts
would be incomplete; see Limitations.

## Judgment

The overrides are not a random sample of mistakes. The Applicant overrode
answers in one class: those that were substantively right but did not match an
Accepted Answer exactly. The 51 cases cluster into these surface classes:

- **Typo:** `Jon ossof` / `Jon Ossoff`, `Donal trump` / `Donald Trump`,
  `Tonstop` / `To stop`, `Afyyer` / `After`, `Duplomat` / `U.S. diplomat`,
  `distric` / `district`, `asisgned` / `assigned`.
- **Article, determiner, preposition:** `Citizens of a state` / `Citizens of
  their state`, `Citizens of their state` / `Citizens from their state`,
  `Civil war` / `The Civil War`, `The ussr` / `USSR`.
- **Word form, tense, word order:** `Self governance` / `Self-government`,
  `Fight` / `Fought`, `It help` / `They helped`, `Naturalization process` /
  `Naturalize`, `Freedom of religion` / `Religious freedom`, `1929 stock market
  crash` / `Stock market crash of 1929`.
- **Fragment or short form:** `Executive` / `Executive branch`, `First
  president` / `First president of the United States`, `Federalist papers` /
  `One of the writers of the Federalist Papers`, `Declaration of independence`
  / `Writer of the Declaration of Independence`, `4` / `Four (4) years`.
- **Paraphrase:** `The us was attacked by terrorists` / `Terrorists attacked
  the United States`, `It is the law` / `Required by law`, `The longest
  economic depression in recent history` / `Longest economic recession in
  modern history`, `To support the allied countries` / `To support the Allied
  Powers`.
- **Date or ordinal:** `July 4th 1776` / `July 4, 1776`.

Two conclusions follow.

First, the **Override flag is not ground truth for the matcher**. The 22
non-overridden wrongs contain 11 substantively correct answers that were left
uncorrected; the Applicant did not press Override on them, often because the
same question already had a later corrected attempt. The flag also contains no
information about false accepts: a wrong accept cannot be overridden, because
the tool never shows the Override after a correct verdict.

Second, the **dominant failure is surface variation, not semantic distance**.
Most of the 62 cases differ from an Accepted Answer by one or two surface
operations. The paraphrase class needs more than surface normalization, but the
observed paraphrases can be covered by reviewed alternatives; semantic matching
is not required for this corpus.

## Sorted list: overridden entries that are substantively correct (51)

Each row is one Answer Log entry. `Accepted answers` lists the forms the
question accepted for this Applicant at export time. Q23, Q38, and Q57 are
Profile-backed; their accepted forms are the Profile values above.

| Q | Question | Typed answer | Accepted answers | Difference |
|---:|---|---|---|---|
| 4 | The U.S. Constitution starts with the words “We the People.” What does “We the People” mean? | Self governance | Self-government<br>Popular sovereignty<br>Consent of the governed<br>People should govern themselves<br>(Example of) social contract | word form: governance / government |
| 6 | What does the Bill of Rights protect? | Basic rights of americans | (The basic) rights of Americans<br>(The basic) rights of people living in the United States | missing leading article |
| 8 | Why is the Declaration of Independence important? | Because it represnts free of the british control | It says America is free from British control.<br>It says all people are created equal.<br>It identifies inherent rights.<br>It identifies individual freedoms. | typo + paraphrase |
| 17 | The President of the United States is in charge of which branch of government? | Executive | Executive branch | fragment of the accepted answer |
| 23 | Who is one of your state’s U.S. senators now? | Jon ossof | Jon Ossoff<br>Raphael Warnock | typo |
| 26 | Why do U.S. representatives serve shorter terms than U.S. senators? | To closely monitor popular opinion | To more closely follow public opinion | synonyms: monitor/follow, popular/public |
| 28 | Why does each state have two senators? | Tonhave equalnrepresentation if the states | Equal representation (for small states)<br>The Great Compromise (Connecticut Compromise)<br>Connecticut Compromise | typos + missing words |
| 31 | Who does a U.S. senator represent? | Citizens of a state | Citizens of their state<br>People of their state | determiner: a / their |
| 32 | Who elects U.S. senators? | Citizens of their state | Citizens from their state | preposition: of / from |
| 33 | Who does a member of the House of Representatives represent? | Citizens from their congressional distric | Citizens in their (congressional) district<br>Citizens in their district<br>People from their (congressional) district<br>People in their district | preposition + typo |
| 34 | Who elects members of the House of Representatives? | Citizens from their congressional distric | Citizens from their (congressional) district | typo |
| 35 | Some states have more representatives than other states. Why? | Because some states have more population | (Because of) the state’s population<br>(Because) they have more people<br>(Because) some states have more people | synonym: population / people |
| 36 | The President of the United States is elected for how many years? | 4 | Four (4) years | short form; digit-only |
| 37 | The President of the United States can serve only two terms. Why? | So it does not become too powerful | (Because of) the 22nd Amendment<br>To keep the president from becoming too powerful | paraphrase |
| 38 | What is the name of the President of the United States now? | Donal trump | Donald Trump | typo |
| 49 | Why is the Electoral College important? | To determine who was elected for president | It decides who is elected president.<br>It provides a compromise between the popular election of the president and congressional selection. | paraphrase |
| 57 | Who is the Chief Justice of the United States now? | John roberts | John G. Roberts Jr | shortened name; missing middle initial and suffix |
| 58 | Name one power that is only for the federal government. | Foreign policy | Print paper money<br>Mint coins<br>Declare war<br>Create an army<br>Make treaties<br>Set foreign policy | short form; missing verb |
| 60 | What is the purpose of the 10th Amendment? | Powers not given to the federal government are assigned to the states | (It states that the) powers not given to the federal government belong to the states or to the people. | synonym: assigned / belong; missing “or to the people” |
| 63 | There are four amendments to the U.S. Constitution about who can vote. Describe one of them. | All citizens 18 years or older can vote | Citizens eighteen (18) and older (can vote).<br>You don’t have to pay (a poll tax) to vote.<br>Any citizen can vote. (Women and men can vote.)<br>A male citizen of any race (can vote). | paraphrase; digit form |
| 65 | What are three rights of everyone living in the United States? | Freedom of speech, freedom of assembly, religious freedom | Freedom of expression<br>Freedom of speech<br>Freedom of assembly<br>Freedom to petition the government<br>Freedom of religion<br>The right to bear arms | word order: religious freedom / freedom of religion; third item |
| 67 | Name two promises that new citizens make in the Oath of Allegiance. | Protect the constitution,  loyalty to the us | Give up loyalty to other countries<br>Defend the (U.S.) Constitution<br>Obey the laws of the United States<br>Serve in the military (if needed)<br>Serve (help, do important work for) the nation (if needed)<br>Be loyal to the United States | synonyms: protect/defend, loyalty/be loyal |
| 68 | How can people become United States citizens? | Naturalization process | Be born in the United States, under the conditions set by the 14th Amendment<br>Naturalize<br>Derive citizenship (under conditions set by Congress) | word form: naturalization / naturalize |
| 71 | Why is it important to pay federal taxes? | It is the law | Required by law<br>All people pay to fund the federal government<br>Required by the (U.S.) Constitution (16th Amendment)<br>Civic duty | paraphrase |
| 73 | The colonists came to America for many reasons. Name one. | Freedom of religion | Freedom<br>Political liberty<br>Religious freedom<br>Economic opportunity<br>Escape persecution | word order: freedom of religion / religious freedom |
| 79 | When was the Declaration of Independence adopted? | July  4th 1776 | July 4, 1776 | ordinal suffix: 4th / 4 |
| 84 | Why were the Federalist Papers important? | It help the people understand the constitution | They helped people understand the (U.S.) Constitution.<br>They supported passing the (U.S.) Constitution. | tense and grammar |
| 85 | Benjamin Franklin is famous for many things. Name one. | Duplomat | Founded the first free public libraries<br>First Postmaster General of the United States<br>Helped write the Declaration of Independence<br>Inventor<br>U.S. diplomat | typo + missing “U.S.” |
| 86 | George Washington is famous for many things. Name one. | First president | “Father of Our Country”<br>First president of the United States<br>General of the Continental Army<br>President of the Constitutional Convention | fragment of the accepted answer |
| 87 | Thomas Jefferson is famous for many things. Name one. | Declaration of independence | Writer of the Declaration of Independence<br>Third president of the United States<br>Doubled the size of the United States (Louisiana Purchase)<br>First Secretary of State<br>Founded the University of Virginia<br>Writer of the Virginia Statute on Religious Freedom | fragment; missing the role |
| 88 | James Madison is famous for many things. Name one. | Federalist papers | “Father of the Constitution”<br>Fourth president of the United States<br>President during the War of 1812<br>One of the writers of the Federalist Papers | fragment of the accepted answer |
| 89 | Alexander Hamilton is famous for many things. Name one. | Federalist papers | First Secretary of the Treasury<br>One of the writers of the Federalist Papers<br>Helped establish the First Bank of the United States<br>Aide to General George Washington<br>Member of the Continental Congress | fragment of the accepted answer |
| 92 | Name the U.S. war between the North and the South. | Civil war | The Civil War | missing leading article |
| 94 | Abraham Lincoln is famous for many things. Name one. | Emancipation proclamation | Freed the slaves (Emancipation Proclamation)<br>Saved (or preserved) the Union<br>Led the United States during the Civil War<br>16th president of the United States<br>Delivered the Gettysburg Address | short form |
| 96 | What U.S. war ended slavery? | Civil war | The Civil War | missing leading article |
| 98 | When did all men get the right to vote? | Afyyer the civil war | After the Civil War<br>During Reconstruction<br>(With the) 15th Amendment<br>1870 | typo |
| 101 | Why did the United States enter World War I? | To support the allied countries | Because Germany attacked U.S. (civilian) ships<br>To support the Allied Powers (England, France, Italy, and Russia)<br>To oppose the Central Powers (Germany, Austria-Hungary, the Ottoman Empire, and Bulgaria) | synonym: countries / Powers |
| 103 | What was the Great Depression? | The longest economic depression in recent history | Longest economic recession in modern history | synonyms: depression/recession, recent/modern |
| 104 | When did the Great Depression start? | 1929 stock market crash | The Great Crash (1929)<br>Stock market crash of 1929 | word order |
| 108 | Who was the United States’ main rival during the Cold War? | The ussr | Soviet Union<br>USSR<br>Russia | extra leading article |
| 110 | Why did the United States enter the Korean War? | Tonstop the spread of communism | To stop the spread of communism | typo |
| 112 | What did the civil rights movement do? | Fight to end racial discrimination | Fought to end racial discrimination | tense: fight / fought |
| 114 | Why did the United States enter the Persian Gulf War? | To drive the iraqui army out of Kuwait | To force the Iraqi military from Kuwait | paraphrase + typo |
| 115 | What major event happened on September 11, 2001 in the United States? | The us was attacked by terrorists | Terrorists attacked the United States<br>Terrorists took over two planes and crashed them into the World Trade Center in New York City<br>Terrorists took over a plane and crashed into the Pentagon in Arlington, Virginia<br>Terrorists took over a plane originally aimed at Washington, D.C., and crashed in a field in Pennsylvania | passive / active voice |
| 116 | Name one U.S. military conflict after the September 11, 2001 attacks. | The iraqui war | (Global) War on Terror<br>War in Afghanistan<br>War in Iraq | paraphrase + typo |
| 118 | Name one example of an American innovation. | The integrated circuit | Light bulb<br>Automobile (cars, internal combustion engine)<br>Skyscrapers<br>Airplane<br>Assembly line<br>Landing on the moon<br>Integrated circuit (IC) | extra leading article |
| 121 | Why does the flag have 13 stripes? | Represents the 13 original colonies | (Because there were) 13 original colonies<br>(Because the stripes) represent the original colonies | paraphrase; extra number |
| 122 | Why does the flag have 50 stars? | Representsbthe 50 states | (Because there is) one star for each state<br>(Because) each star represents a state<br>(Because there are) 50 states | typo + paraphrase |
| 125 | What is Independence Day? | A holiday to celebrate the us independence | A holiday to celebrate U.S. independence (from Britain)<br>The country’s birthday | extra internal article |
| 127 | What is Memorial Day? | A holiday to honor soldiers who died serving in the military | A holiday to honor soldiers who died in military service | paraphrase |
| 128 | What is Veterans Day? | Holiday to honer who served in the military | A holiday to honor people in the (U.S.) military<br>A holiday to honor people who have served (in the U.S. military) | typo + missing words |

## Sorted list: wrong but not overridden, substantively correct (11)

These entries were judged Wrong and left uncorrected. They are the evidence
that an Override-only sample undercounts the false negatives.

| Q | Question | Typed answer | Accepted answers | Difference |
|---:|---|---|---|---|
| 1 | What is the form of government of the United States? | Constitutional Republic | Republic<br>Constitution-based federal republic<br>Representative democracy | synonym |
| 5 | How are changes made to the U.S. Constitution? | Amendement process | Amendments<br>The amendment process | typo |
| 34 | Who elects members of the House of Representatives? | Citizens | Citizens from their (congressional) district | fragment of the accepted answer |
| 41 | Name one power of the president. | Approves and signs bills | Signs bills into law<br>Vetoes bills<br>Enforces laws<br>Commander in Chief (of the military)<br>Chief diplomat<br>Appoints federal judges | paraphrase of “Signs bills into law” |
| 41 | Name one power of the president. | Foreign policy | Signs bills into law<br>Vetoes bills<br>Enforces laws<br>Commander in Chief (of the military)<br>Chief diplomat<br>Appoints federal judges | short form of “Chief diplomat” |
| 60 | What is the purpose of the 10th Amendment? | Gives states the powers not asisgned to the federal goverment | (It states that the) powers not given to the federal government belong to the states or to the people. | typo + paraphrase |
| 103 | What was the Great Depression? | The biggest economic depression in recent history | Longest economic recession in modern history | synonyms: biggest/longest, depression/recession |
| 112 | What did the civil rights movement do? | Fought for equal rights | Fought to end racial discrimination | paraphrase |
| 113 | Martin Luther King, Jr. is famous for many things. Name one. | Fight to end racial discrimination | Fought for civil rights<br>Worked for equality for all Americans<br>Worked to ensure that people would “not be judged by the color of their skin, but by the content of their character” | paraphrase + tense |
| 125 | What is Independence Day? | Holiday to celebrate country birthday | A holiday to celebrate U.S. independence (from Britain)<br>The country’s birthday | paraphrase of “The country’s birthday” |
| 127 | What is Memorial Day? | Holiday to honor people who died serving in the military | A holiday to honor soldiers who died in military service | paraphrase |

## Not substantively correct (11)

These are the wrong verdicts that should stay wrong. They are the control set
for any matcher change. Repeated rows are separate attempts.

| Q | Question | Typed answer | Accepted answers | Why it stays wrong |
|---:|---|---|---|---|
| 29 | Name your U.S. representative. | A | Hank Johnson | placeholder, not an answer |
| 30 | What is the name of the Speaker of the House of Representatives now? | A | Mike Johnson | placeholder, not an answer |
| 30 | What is the name of the Speaker of the House of Representatives now? | A | Mike Johnson | placeholder, not an answer |
| 30 | What is the name of the Speaker of the House of Representatives now? | A | Mike Johnson | placeholder, not an answer |
| 57 | Who is the Chief Justice of the United States now? | A | John G. Roberts Jr | placeholder, not an answer |
| 57 | Who is the Chief Justice of the United States now? | A | John G. Roberts Jr | placeholder, not an answer |
| 70 | What is one way Americans can serve their country? | A | Vote<br>Pay taxes<br>Obey the law<br>Serve in the military<br>Run for office<br>Work for local, state, or federal government | placeholder, not an answer |
| 77 | Name one reason why the Americans declared independence from Britain. | Freedom | High taxes<br>Taxation without representation<br>British soldiers stayed in Americans’ houses (boarding, quartering)<br>They did not have self-government<br>Boston Massacre<br>Boston Tea Party (Tea Act)<br>Stamp Act<br>Sugar Act<br>Townshend Acts<br>Intolerable (Coercive) Acts | vague; not a listed reason |
| 83 | The Federalist Papers supported the passage of the U.S. Constitution. Name one of the writers. | A | (James) Madison<br>(Alexander) Hamilton<br>(John) Jay<br>Publius | placeholder, not an answer |
| 87 | Thomas Jefferson is famous for many things. Name one. | Constitution | Writer of the Declaration of Independence<br>Third president of the United States<br>Doubled the size of the United States (Louisiana Purchase)<br>First Secretary of State<br>Founded the University of Virginia<br>Writer of the Virginia Statute on Religious Freedom | wrong for Jefferson |
| 99 | Name one leader of the women’s rights movement in the 1800s. | A | Susan B. Anthony<br>Elizabeth Cady Stanton<br>Sojourner Truth<br>Harriet Tubman<br>Lucretia Mott<br>Lucy Stone | placeholder, not an answer |

## Proposal to fix this

The proposal is staged by risk. The first stage changes capture and tests the
policy; the second accepts reviewed alternatives; the third adds only
normalization rules that cannot produce a false accept on the control set.
Semantic matching is deliberately last, because ADR 0004 rejects it for a
reason that has not changed: a wrong accept is invisible to the Applicant and
silently corrupts Coverage.

### 1. Capture every Wrong verdict, not only the Overrides

Extract all `verdict == "wrong"` entries and keep `overridden` as a column:
the 11 non-overridden false negatives are the proof that the Override filter
under-counts the error by 18% (11 of 62). The Answer Log already holds every
entry, so no new store is needed (ADR 0007). The export remains the transport
(ADR 0003).

Add the accepted set to the entry at append time. `AnswerLogEntry` gains an
optional snapshot:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub accepted_answers: Option<Vec<AcceptedAnswer>>,
```

The study screen already resolves the accepted set before judging and before
`append` (`crates/app/src/screens/study.rs:170-200`); the snapshot is local to
that call. Old records read as `None`; new records carry the exact forms the
verdict used, including the empty set when a Local Fact is missing. This makes
the sample reproducible when the Profile or `questions.json` changes. It is a
record-shape change, not a new IndexedDB store; check the export-schema rule in
ADR 0007 before deciding whether `schema_version` moves.

A compact extractor is enough:

```sh
jq '[.answer_log[] | select(.verdict == "wrong")
     | {at, question, answer, overridden, accepted_answers}]' bundle.json
```

### 2. Review the 78 correct verdicts before loosening anything

The current corpus measures only false negatives. The failure ADR 0004 is
designed to prevent is a false accept, and a false accept is invisible: the
Applicant never sees an Override offer for it. Review the 78 correct verdicts,
especially short answers and the eight Profile-backed questions, and add any
wrong accept to the fixture as `false_positive`. Do this before changing the
matcher, so the change is measured on both error directions.

### 3. Build a reviewed regression fixture

Put the 73 wrong verdicts, their accepted sets, and their labels in a data
fixture, for example `crates/domain/tests/fixtures/wrong_verdict_review.json`.
The labels are:

- `false_negative`: the typed answer is substantively correct;
- `true_negative`: the typed answer is genuinely wrong;
- `false_positive`: a correct verdict whose typed answer is genuinely wrong
  (added after step 2).

A test in `crates/domain/tests/` replays `matching::judge` over the fixture and
asserts that every `false_negative` is accepted and every `true_negative`
stays wrong. The existing test suite is the regression guard for the 78 correct
verdicts. The fixture is a policy test, not a claim that USCIS would accept
every listed phrasing.

### 4. Accept reviewed alternatives before adding fuzzy rules

The safest fix is to grow the accepted set, not to loosen the comparison. The
Scoring Guidelines already state that the real test accepts "a correct answer
or an alternative phrasing of the correct answer" (quoted in ADR 0004). A
reviewed alternative list per question keeps matching exact and makes every
new accept explainable.

Add a reviewed `alternatives` field to the question records, fed by a separate
`crates/gen-questions/alternatives.json`; do not reuse the data-correction
`overrides.json`, whose entries are published-answer corrections, not an
Applicant's phrasings. Fill it from the false-negative rows. Examples worth
accepting as-is:

- Q71: `It is the law`
- Q101: `To support the Allied countries`
- Q115: `The US was attacked by terrorists`
- Q127: `A holiday to honor soldiers who died serving in the military`
- Q128: `A holiday to honor people who served in the military`
- Q73: `Freedom of religion`
- Q65: `Freedom of religion` as a third item
- Q36: `4` (the question asks for a number; the accepted form already buys
  `4 years`)
- Q41: `Approves and signs bills`, `Foreign policy`
- Q1: `Constitutional Republic`

Do not add raw misspellings. Add the intended phrasing once; the typo cases
stay with the Override or get a narrow rule below. For fragments, add the short
form only where it is unambiguous (`First president`, `Federalist papers`,
`Executive`). Do not add a generic subset rule: `Civil war` is a complete
answer to Q92 and Q96, but an incomplete answer to Q94, so the same fragment
cannot be accepted everywhere.

### 5. Add narrow normalization rules only where the control set proves them safe

Candidate rules, in order:

1. **Numbers both ways.** Accept a spelled-out number and its digits in either
   direction, not only the numeral-parenthetical form. Q36 (`4` /
   `Four (4) years`) and Q63 (`18` / `eighteen`) are the observed cases.
2. **Ordinals.** Accept `4th` and `4` when the accepted answer has a date or a
   number. Q79 is the observed case.
3. **Leading articles.** Ignore a leading `the`, `a`, or `an` when the rest
   matches exactly. Q92, Q96, Q108, and Q118 are the observed cases. Do not
   ignore internal determiners such as `a` / `their` without a reviewed
   alternative.
4. **Abbreviations.** `U.S.` and `US` already normalize to `us`; keep that and
   do not add a general synonym table.

Reject for now: generic edit distance, bag-of-words matching, and stemming.
They are the rules most likely to accept a different proper noun or a different
fact. A one-edit rule would need a denylist and a minimum length; even then,
`Iran` / `Iraq` shows why a substitution cannot be treated as a typo. The typo
cases are 15 entries: 13 in the overrides and 2 in the non-overridden set. They
are the least valuable to automate because the Applicant can see and correct
them with one Override tap.

### 6. Do not reopen ADR 0004 for semantic matching yet

The paraphrase class is real: `It is the law` / `Required by law`, `The us was
attacked by terrorists` / `Terrorists attacked the United States`, `The longest
economic depression in recent history` / `Longest economic recession in modern
history`. Curated alternatives cover those cases without a model. Semantic
matching would add an offline/WASM model, cannot explain an accept, and
directly contradicts ADR 0004. If the curated approach plateaus, write a new
ADR with the false-accept measurement and a budget; do not slip semantic
matching in as an undocumented matcher branch.

### 7. Re-export and compare

After the changes, run the same extractor over a fresh export and compare:

- false negatives remaining among the reviewed 62;
- any new false accepts among the 78 correct verdicts;
- any new false accepts introduced by the new alternatives.

The goal is not zero false negatives. The goal is to remove the surface
variation without accepting a wrong fact. The Override remains the escape hatch
for the long tail, which is exactly its purpose in the domain model
(`GLOSSARY.md`, ADR 0004).

## Limitations

- **One Applicant, one export.** The 151 entries are not a random sample. They
  are this Applicant's study history, and the "substantively correct" labels
  are this review's judgment.
- **False accepts are unmeasured.** The 78 correct verdicts were not reviewed.
  The corpus says nothing about whether the exact matcher already accepted a
  wrong answer.
- **Accepted answers are joined now, not at judgment time.** For ordinary
  questions the committed records are stable enough; for the eight
  Profile-backed questions, the Profile can change. Q57's accepted value, for
  example, is the Profile's `John G. Roberts Jr`, not the USCIS placeholder in
  `questions.json`. A snapshot in the entry (proposal step 1) removes this
  limitation.
- **The 11 "not substantively correct" labels are judgment.** `Freedom` for
  Q77 is the closest call; it is not a listed reason, but an oral examiner
  might ask for a specific one. `Constitution` for Q87 is wrong for Jefferson.
  The placeholders are unambiguous.
- **The wording of the accepted answers is the record's, not USCIS's
  adjudication.** The records are generated from M-1778 and corrected by hand;
  the matcher's policy is the subject here, not the official scoring of any
  particular answer.

## Sources

- `local://paste-1.md` — the supplied export bundle, schema version 1,
  exported 2026-10-17T16:15:35Z, 151 Answer Log entries. The bundle itself is
  not committed; the tables above reproduce every reviewed entry and its
  accepted forms.
- `crates/app/data/questions.json` — the committed question records and their
  Accepted Answers.
- `crates/domain/src/answers.rs:31-39` — the eight Profile-backed questions and
  their facts.
- `crates/domain/src/matching.rs` — normalized exact match and the
  parenthetical rules.
- `crates/domain/src/progress.rs:25-51` — `AnswerLogEntry` and
  `counts_correct`.
- `crates/app/src/storage/answer_log.rs` — `append`, `override_last`, and the
  record encoding.
- `crates/app/src/storage/bundle.rs:35-44` — the export bundle carries the
  Answer Log.
- `crates/app/src/screens/study.rs:170-200` — the accepted set is resolved
  before judging and appending.
- `docs/adr/0003-client-side-only-no-server.md` — no server.
- `docs/adr/0004-answer-matching-normalized-exact.md` — exact matching, and
  the Scoring Guidelines' alternative-phrasing policy.
- `docs/adr/0007-progress-storage.md` — the Answer Log as the single source,
  and the rejection of a separate Override store.
- `GLOSSARY.md` — Accepted Answer, Override, Verdict.
