# ADR 0131 — What happens to what is in a folder when it goes back

**Status:** **proposed, 2026-10-22, by the Mac lane** — built the same day under
the second clause of *work a task needs is part of that task*, because the
restore cannot be written without answering this and the answer is visible to a
person. **The owner may overrule it; what it costs them is written down below.**

**Context:** [ADR 0045](0045-what-undoing-rewinds-to.md) (what undoing rewinds
to — its third term is the one this is about),
[ADR 0127](0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
(a person's own commands are bracketed in a session they asked for — term 2
decided *a folder, now*, and term *what this does not decide* did not reach
this),
[ADR 0130](0130-what-takes-the-bracket-before-a-persons-own-command.md) (what
takes the bracket), `crates/alo-as-it-was/src/putting_back.rs`,
`docs/features.md` ★ *scrollback you can move along, not only read*.

## The question

ADR 0045's third accepted term is **an undo never overwrites a later change**,
and it says how that is enforced: an undo *compares the two sides of a turn and
refuses if the home no longer matches the after snapshot on those paths.*

ADR 0127 then decided that a person's own command takes **only the before side**
— nothing compares a person's command, and an after snapshot nothing reads would
be a second copy of their whole home per command — and said so in as many words:
*restoring reads a snapshot and compares nothing.*

So the mechanism that enforced the third term is not available on this road, and
the term still binds. **What happens to the work a person did after the snapshot
was taken?** Every answer is a decision about somebody's files.

## The options

### Option A — set it aside beside the folder, and say where *(chosen)*

The folder is replaced by the copy from the snapshot, and what was in it is
renamed to `<folder>.alo-was-here-<moment>` next to it. The sentence a person
approves says both halves: *this folder will be as it was at that moment, and
what is in it now will be at that path.* Nothing is deleted by alo, ever.

- **What it costs a person:** a directory in their home they have to delete
  themselves, and the disk it holds until they do. On `btrfs` the copy that goes
  *in* is reflinked and costs almost nothing; what costs is the old contents
  staying, which is the point.
- **What it buys:** ADR 0045's third term is met without a comparer, by a
  different road — a later change cannot be overwritten, because it is never
  overwritten, only moved. A person who went back by mistake can go back from
  that, with no mechanism at all: the directory is right there.
- **Against it:** it is clutter, and somebody who goes back ten times has ten
  directories. The honest reading is that this is a person's own filing rather
  than alo's to tidy, and the alternative is alo deleting their work.

### Option B — refuse unless the folder is untouched since the snapshot

The literal reading of ADR 0045's third term: compare, and refuse if anything
changed.

- **Against it, decisively:** *the folder changed since the snapshot* is the
  **only** case in which anybody wants this. A codemod mangled `src/`; of course
  it changed. A restore that refuses whenever it would do something is not a
  feature, and comparing is a privileged path ADR 0045 gave to another plan and
  which does not exist.

### Option C — replace it, and keep nothing

- **Against it:** it makes alo the thing that deleted somebody's afternoon, on
  one typed confirmation, with no record and no road back. Law 2's *each run is
  recorded and can be undone* is about running code; an undo that cannot itself
  be undone is the same gap one level up.

### Option D — put the old contents in the undo folder

- **Against it:** the undo folder has a privileged remover with an expiry on it
  (ADR 0045 term 5, `crates/alo-letting-go`). A person's work from *after* the
  snapshot would be swept on a timer meant for snapshots. Worse than C, because
  it looks safe.

## What this does not decide

- **Whether alo ever offers to tidy them up.** It does not today. A row that
  removed `*.alo-was-here-*` directories would be alo deleting a person's files
  on a setting, which is its own decision and a larger one.
- **The whole home.** ADR 0127 term 2 settled that: at the next sign-in, and
  never by a privileged helper. Nothing here changes it.
- **Anything about the comparer**, which is still another plan's and still does
  not exist.
