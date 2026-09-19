# Undo what the agent did

**Date:** 2026-09-18
**Workstream:** v0.5 the machine keeps itself — task 4 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (`ROADMAP.md` and
`docs/features.md` v0.5 ★ *undo what the agent did*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os-3`
**Status:** ready for integration. Points 1 to 5 of
[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md)'s *what holds
whichever option is taken*, built as the decision asks while the road it chose
is still four other lanes' work. Nothing here puts a file back, and nothing here
claims to.

## What changed, for a person

A person can now ask their machine whether something the assistant did can be
undone, and get an honest answer with a reason in it.

For most of what an agent does, the answer is **no, and here is why**: a message
that was sent left the machine and nothing here can call it back; a question a
model was told cannot be untold; a page that was printed is on paper; opening or
closing an application is not something the machine puts back, and an
application that closed may have let go of what was open in it; an application
that was installed is the person's own to remove, where they install and remove
them. A refusal, or something that only looked at a folder, changed nothing, so
there is nothing to put back. None of those is hedged and none of them is a
shrug: each is its own sentence, in the person's own language.

For the three things an agent can do to a person's own files — renaming one,
moving one, making an archive of a folder — there **is** a road back in
principle, and on the machine alo OS installs today the machine says so plainly:
*this machine does not keep what your files were before the agent changed them,
so it cannot put them back*. That is the truth rather than a fault, and it is
said before anything is offered rather than halfway through a folder.

When a machine does keep it, the rules a person meets are already decided.
An undo names exactly what comes back and when it was changed, and waits for one
approval of that sentence — an undo is a change, so it is approved like one. If
anything it would put back has changed since, the whole undo is refused rather
than taking away the person's own newer work. What the machine keeps for this
reaches back seven days or fifty changes, whichever ends first, so a disk does
not fill up quietly; if the machine was too short of space to keep one change,
the work still happened and the machine says that one cannot be undone. Letting
go of all of it is one act, and the sentence for it says both halves: nothing
can be put back afterwards, and the space comes back.

Afterwards, the machine's own history says the person put something back, in
which words it was described when it happened — and says so as well when they
asked and it could not be done.

**No agent can do any of this.** There is no verb that undoes and none that
proposes an undo. Undoing is the person's, and the machine was built so that an
agent cannot quietly reverse something somebody approved.

## What changed, in the repository

**`crates/alo-keeping-up`** (decisions; still no clock, thread, socket, file or
process, and still the same four dependencies):

| File | What it holds |
|---|---|
| `src/undoing.rs` | `Change` (`Renamed`, `Moved`, `Archived`) and `WhatWasDone`, what the record says as far as undoing cares; `WhatWasDone::of_a_verb(verb, changed)`; `could_be_put_back` — ADR 0045 point 1's closed table; `NotUndoable`, fourteen reasons, each with its own sentence |
| `src/putting_back.rs` | `WhatWasKept` (`NothingOnThisMachine`, `NotThisTurn`, `LetGo`, `EitherSideOfTheTurn`), `Bracket` (what a turn changed, and what of it moved since), `WhenItWasDone`, `AnUndo::offered` and `AnUndo::said`; `WhatWasKept::forgetting` |
| `src/how_far_back.rs` | `HowFarBack`, the owner's first term: `AS_SHIPPED` is seven days or fifty changing turns, `still_reaches`, `how_many_it_still_reaches`; `NotAWindow` refuses a window that reaches nothing, on the way in and when read back off a disk |
| `src/words.rs` | Eighteen more sentences with translator's notes; `THE_ONE_WITH_GAPS` names the only sentence in the crate with anything to fill in, and the gap test holds the other forty-one to having none |
| `tests/undo_what_the_agent_did.rs` | The acceptance: every verb `alo-declared` ships put through the table, every reason held to a sentence in the machine's one vocabulary, no verb that undoes, no inverse in the source |

**`crates/alo-updating`** (the doing, and here the seam):

| File | What it holds |
|---|---|
| `src/putting_back.rs` | `what_was_done(entry)` — every kind of record entry onto the table, exhaustively; `what_this_machine_kept()` — `NothingOnThisMachine`, with the four lanes that would change it named; `putting_back(entry)` — one call from an entry to an undo or to the reason there is none |
| `tests/putting_back_what_an_agent_did.rs` | The acceptance from the record's side: a real `move_file`, really granted, approved once and run, read onto the table; the undo entry; the failed one; the copy outliving the original |

**`crates/alo-record`, additive, `format` stays `1`:** `Happened::Undone {
undid, what, failed }`, with no agent and no field for one, carrying a **copy**
of the entry it undid rather than a pointer into a file that is pruned; written
only from an entry that `ran` (`Entry::undone`, `Entry::undo_failed`, both
`Option`); `Entry::undid` and `Happened::undid`; the new kind added to every
accessor, and counted by `was_stopped` when it failed.

**`crates/alo-recounting`, additive:** `Outcome::PutBack` and
`Outcome::NotPutBack` with their two clauses; `Told::undid`, and an undo's line
reading as the change it put back so that *what did I put back, and when had it
happened* is one line rather than two entries a reader has to pair up.

**`docs/contracts/record-file.md`:** the `undone` kind, what it carries, that it
is written only from an entry that ran, and that whether something *could* have
been put back is not the record's question.

## Decisions

Where the task left something open, this is what was chosen and why.

- **The table lives in `alo-keeping-up` and the record is read in
  `alo-updating`.** `alo-keeping-up` names no `alo_record::Happened` — the same
  separation task 1 kept for the clock. One crate decides and another reads, and
  `alo_updating::putting_back` is **the one place** the two are put beside each
  other, so a reviewer checking that the machine applies ADR 0045's table has one
  file to read rather than every surface that ever shows a record.
- **A verb nobody classified answers *never*.** `WhatWasDone::of_a_verb` takes
  the verb's name **and** whether the record says it changed anything, so a verb
  added next year lands somewhere honest without anybody remembering this file: a
  new read verb only looked, a new change verb is not one of the changes there is
  a road back for. The acceptance asks `alo-declared`'s registry rather than
  keeping a list of its own, so the day the two disagree is the day the test
  fails rather than the day it passes by agreeing with itself.
- **Fourteen refusals rather than one.** Nine are about what was done and hold on
  every machine there will ever be; five are about the machine in front of the
  person. *Why not* is the whole of the answer, and one sentence covering three
  facts would answer none of them — so each has its own, and a test holds that no
  two read alike.
- **`what_this_machine_kept()` is a function with one answer today, and that is
  not a stub.** It is where the four lanes ADR 0045 names — the installer's
  filesystem, the accounts lane's home subvolume, `alo-turn`'s bracket, the
  broker's privilege — meet this one. Until all four have landed,
  `NothingOnThisMachine` is what is true here, and the day the bracket exists it
  is the only function that changes. The alternative, letting every surface decide
  for itself what the machine keeps, is how two of them come to disagree.
- **`Entry::undone` refuses an entry that did not run**, not merely one with no
  call in it. A change a person **declined** carries a `What` too, and a
  constructor that took anything carrying one would let the record say somebody
  put back something nobody did. An undo of an undo is refused for the same
  reason.
- **An undo's `What` is answered by `undid` and never by `what`.** An undo of a
  move that read back as a move would be one change counted twice, and a review
  asking what this machine executed would find an act no agent performed.
  `alo-recounting` puts them on one line, which is a surface's job, and the
  record keeps them apart, which is the record's.
- **A failed undo is a refusal.** Something a person asked for did not happen,
  and a review looking for that finds it with the rest rather than nowhere. The
  sentence kept is the one they were shown, handed in already worded, so the
  record and the screen are one account of one moment.
- **One `WhatWasKept::LetGo` for three causes** — the window passed, the disk
  needed the room, the person forgot it. What a person can do about it is the
  same in all three, and which turns lost their undo belongs in the record
  (ADR 0045's second term), not in three sentences that read alike.
- **No maximum on `HowFarBack`, and zero is refused.** An organisation naming a
  long window is naming their own rule (`CLAUDE.md`: we ship the mechanism, never
  a default that decides for them), and the disk is already protected by *the
  oldest go first under pressure*. A window of zero would be undo switched off by
  arithmetic in a settings file; the honest way to hold nothing is
  `WhatWasKept::forgetting`, which is one act with a sentence on it. It is
  refused when it is made **and** when it is read back, so a file cannot do what
  the constructor will not.
- **Editing `alo-record` and `alo-recounting` again**, for the reason tasks 2 and
  3 gave: the acceptance asks for a record entry, no existing kind fits, and the
  contract allows additive kinds. Both edits are minimal and neither changes an
  existing kind.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| For a verb the record says ran, `alo-keeping-up` answers whether it can be undone | `alo-keeping-up` `undo_what_the_agent_did::every_verb_this_machine_ships_is_answered_by_the_table`; `alo-updating` `putting_back_what_an_agent_did::a_change_the_record_says_ran_is_answered_and_so_is_a_read` |
| …and says plainly when it cannot, with the reason in the vocabulary | `alo-keeping-up` `undo_what_the_agent_did::every_reason_it_cannot_is_a_sentence_this_machine_collects`, `…::a_verb_added_after_this_was_written_is_not_undoable`; `alo-updating` `putting_back_what_an_agent_did::what_cannot_be_put_back_says_why_in_words_the_machine_collects` |
| What can be undone is undone through the mechanism that made it undoable, and never by a second implementation guessing at inverses | `alo-keeping-up` `undo_what_the_agent_did::nothing_here_puts_a_file_back_or_invents_an_inverse`; `alo-updating` `putting_back_what_an_agent_did::the_seam_puts_nothing_back`; `alo-keeping-up` lib `putting_back::tests::a_machine_that_keeps_nothing_says_so_instead_of_offering` |
| Undoing is itself recorded, with the entry it undid named | `alo-updating` `putting_back_what_an_agent_did::an_undo_is_written_down_naming_what_it_undid`, `…::what_an_undo_undid_is_a_copy_and_outlives_the_original`; `alo-record` lib `entry::tests::an_undo_is_recorded_with_a_copy_of_what_it_undid_and_nobody_behind_it`; `alo-recounting` lib `told::tests::an_undo_reads_back_as_the_change_it_put_back_and_when_that_was` |
| Undoing requires the same person's approval the doing did | `alo-keeping-up` `undo_what_the_agent_did::the_only_way_to_an_undo_is_a_sentence_a_person_approves`; `alo-keeping-up` lib `putting_back::tests::a_rename_the_machine_kept_is_offered_with_what_and_when` |
| No inverse is invented, and no agent can undo its own work unasked | `alo-keeping-up` `undo_what_the_agent_did::no_verb_this_machine_ships_undoes_anything`; `alo-updating` `putting_back_what_an_agent_did::an_undo_names_no_agent_and_is_no_execution_of_a_verb` |

Refusal paths, beside the legitimate ones:

| What is refused | Test |
|---|---|
| An undo on a machine that kept nothing, one it had no room to keep, one let go | `alo-keeping-up` `undo_what_the_agent_did::a_machine_that_kept_nothing_says_so_and_says_it_apart_from_having_let_go` |
| An undo where anything it would put back has moved since | `alo-keeping-up` `undo_what_the_agent_did::an_undo_is_refused_before_it_is_offered_when_anything_moved_since`; lib `putting_back::tests::anything_changed_since_refuses_the_undo_before_it_is_offered` |
| A bracket that found nothing | `alo-keeping-up` lib `putting_back::tests::a_turn_that_changed_nothing_offers_nothing` |
| An offer with a blank where the moment belongs | `alo-keeping-up` lib `putting_back::tests::a_blank_moment_is_not_a_moment` |
| A window that reaches nothing, made or read back off a disk | `alo-keeping-up` lib `how_far_back::tests::a_window_that_reaches_nothing_is_refused`, `…::a_window_read_back_is_checked_the_way_one_made_here_is` |
| An undo written from an entry that did not run, and an undo of an undo | `alo-record` lib `entry::tests::an_entry_that_did_not_run_cannot_be_undone` |
| An undo that did not happen, kept as a refusal | `alo-record` lib `entry::tests::an_undo_that_did_not_happen_is_kept_as_a_refusal_with_the_sentence_shown`; `happened::tests::an_undo_that_did_not_happen_is_counted_among_the_refusals`; `alo-recounting` lib `told::tests::an_undo_that_did_not_happen_reads_back_as_one_that_did_not`; `alo-updating` `putting_back_what_an_agent_did::an_undo_that_did_not_happen_is_recorded_as_a_refusal` |
| What the broker refused, and what it handed on | `alo-updating` lib `putting_back::tests::the_broker_is_answered_by_whether_it_handed_anything_on` |
| Anything no agent did | `alo-updating` lib `putting_back::tests::what_no_agent_did_is_not_an_agents_to_undo`, `…::an_errand_of_the_machines_own_is_not_an_agents_either` |

## Verification

Run on 2026-09-18 in WSL Ubuntu 24.04 on this Windows Server machine, from the
serialized Linux source copy at `/root/alo-trees/this-machine`, with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine` and
`RUSTFLAGS=-C link-arg=-fuse-ld=mold`, as `docs/autonomy/SHARED_MAIN.md`
requires:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | (filled in below) |
| `cargo clippy --all-targets -- -D warnings` | (filled in below) |
| `cargo test -p alo-keeping-up` | (filled in below) |
| `cargo test -p alo-record` | (filled in below) |
| `cargo test -p alo-recounting` | (filled in below) |
| `cargo test -p alo-updating` | (filled in below) |

**Not run here, deliberately:** the whole workspace's tests and the remaining
five of the nine gates. `docs/autonomy/SHARED_MAIN.md`'s *gate what the change
can reach* puts those with the supervisor, which runs them on the combined tree
afterwards.

**Not measured, and not claimed:** nothing in this change was run on the
certified machine or in a virtual machine. It needs neither: no file is written,
no process is run, and the one thing that would need a real disk — a machine that
actually keeps what a folder was — does not exist on any machine yet. When it
does, that measurement is the task the ADR's consequences name.

## Limitations, and what is owed by whom

1. **Nothing can actually be put back yet, and the machine says so.** The road
   ADR 0045 chose needs four things this plan may not build: `btrfs` from
   `crates/alo-installing` (the installer plan's task 11, and it has to land
   **before the certified laptop is installed**, because the filesystem is chosen
   at install and cannot be converted); a home subvolume per person (the accounts
   work's); the two snapshots either side of a changing turn (lane A's
   `crates/alo-turn`); and the privilege to take one (the broker plan's). Until
   then `what_this_machine_kept()` answers `NothingOnThisMachine`, which is true.
2. **Four of the owner's six terms are other lanes' to hold.** Term 1, the
   window, is built here, and so is the sentence term 3 asks for — *there was not
   enough room to keep what your files were before this change* — though what
   attempts the bracket is `alo-turn`'s. Term 2 (*the oldest go first under disk
   pressure, and the record says which turns lost their undo*) needs something
   that holds snapshots to remove; term 4 (*what is filling the disk counts what
   undo is holding, by name*) is `alo-measuring`'s, which this lane does not own;
   term 5 (*only a turn that changes files is bracketed*) is `alo-turn`'s, where
   the bracket is; term 6 is the installer's `btrfs`. Each is named in the ADR
   against its lane.
3. **Nothing draws this.** What a person sees is the shell plan's recovery and
   history surfaces. What is handed to them is `AnUndo::said`, `NotUndoable::said`
   and `WhatWasKept::forgetting`, and `alo_updating::putting_back` is the one call
   that produces the first two from a record entry.
4. **`HowFarBack` is a value, not yet a setting.** Where a person changes it is
   `alo-choosing`/`alo-kept`'s, another lane's; it is `Serialize`/`Deserialize`
   and refuses a window of nothing on the way in, so wiring it up is additive.

## Proposed updates to the shared documents

For the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under v0.5:

> **Undo what the agent did — the honest half.** A machine can now say, for
> anything in its own history, whether it can be put back and why not: what was
> sent has left, what a model was told cannot be untold, what was printed is on
> paper, an application is not something the machine puts back, and a refusal
> changed nothing. For the three things an agent can do to a person's own files
> there is a road back in principle, and on the machines alo OS installs today
> the machine says plainly that it does not keep what those files were, rather
> than offering something that would fail. The rules for when it does are
> decided: an undo names what comes back and when it was changed and waits for
> one approval; it is refused outright if anything it would put back has changed
> since; what is kept reaches back seven days or fifty changes, whichever ends
> first, and letting go of all of it is one act. No agent can undo anything —
> there is no such verb, and there is not going to be one. A machine's history
> now records an undo, and records one that could not be done.

**`ROADMAP.md`:** v0.5's ★ *undo what the agent did* is not yet deliverable and
should not be ticked. What is deliverable is every part of it that does not need
a snapshot, which is what this change is; the remainder is the four lanes in
limitation 1.

**`docs/autonomy/QUEUE.md`:** task 4 of the machine-keeps-itself plan is done;
task 5 (*what a person is told, before and after*) is ready and now has the
whole vocabulary it walks through.

**`docs/autonomy/STATE.md`:** reference this report.
