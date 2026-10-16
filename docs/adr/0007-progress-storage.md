# Progress storage: three collections plus meta, Override derived from the log

All progress (Profile, the Answer Log, Exam records) lives in one IndexedDB
database on the device, in three collections plus a `meta` record:

    profile      key 'profile' (single record): the Profile
    answer_log   key seq (autoIncrement):        { seq, at, question_id, answer,
                                                    verdict, override? }
    exams        key seq (autoIncrement):        { seq, at, track, questions
                                                    asked, correct_count, outcome }
    meta         key name:                       database version, last export

The Answer Log is append-only and the single source of truth for study
history: an Override stores its phrasing on the corrected entry, and the
question's extended Accepted Answers, Cleared, and Coverage are derived from
the log at read time; nothing duplicates them.

Exam records carry the track (`Standard` or `65/20`) they ran under, so a
later Profile change never makes an old Exam unreadable.

Export/import is one JSON bundle (`schema_version`, `exported_at`, Profile,
Answer Log, Exam records), required in v1 (ADR 0003). Import validates the
shape, refuses a newer `schema_version`, and replaces local state after
confirmation; it never merges. The IndexedDB structural version and the
export-schema version are separate mechanisms; both start at 1.

## Considered Options

- **A dedicated Override store.** Rejected. It would duplicate what the log
  already holds, and the two sources would drift.
- **Merge on import.** Rejected. Two logs cannot merge without a dedupe story;
  the real use cases (restore a backup, move to a new device) are
  replacements.

## Consequences

- Record shapes mirror the `domain` types; `app/src/storage/` is the only
  module that touches IndexedDB, and `domain` stays I/O-free.
- The Answer Log grows without bound by design; it is the Applicant's history.
- Any future schema change has two deliberately separate paths: an IndexedDB
  structural migration for local data, and an export-schema migration (or
  refusal) for bundles already in the wild.
