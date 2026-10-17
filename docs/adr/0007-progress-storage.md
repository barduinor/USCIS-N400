# Progress storage: three collections plus meta, Override on the log

All progress (Profile, the Answer Log, Exam records) lives in one IndexedDB
database on the device, in three collections plus a `meta` record:

    profile      key 'profile' (single record): the Profile
    answer_log   key seq (autoIncrement):        { seq, at, question_id, answer,
                                                    verdict, override? }
    exams        key seq (autoIncrement):        { seq, at, track, questions
                                                    asked, correct_count, outcome }
    meta         key name:                       database version, last export

The Answer Log is append-only and the single source of truth for study
history: an Override marks the corrected entry as counting correct, and
Cleared and Coverage are derived from the log at read time, so nothing is
duplicated.

The `meta` store also records the Answer Log and Exam counts. Each append,
Override, clear, and import writes the count in the same transaction as the
records it counts. A read that comes back empty is checked against that count:
when the count says records existed, the app retries the read and, if it stays
empty, shows a recovery message instead of a false `0%`. The read path never
writes or clears anything. The app asks for persistent storage so the origin is
not evicted under storage pressure.

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

- Record shapes mirror the `domain` types; `crates/app/src/storage/` is the only
  module that touches IndexedDB, and `domain` stays I/O-free.
- The Answer Log grows without bound by design; it is the Applicant's history.
- Any future schema change has two deliberately separate paths: an IndexedDB
  structural migration for local data, and an export-schema migration (or
  refusal) for bundles already in the wild.
