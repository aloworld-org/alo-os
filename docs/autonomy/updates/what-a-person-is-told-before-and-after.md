# What a person is told, before and after

**Date:** 2026-09-19
**Workstream:** v0.5 the machine keeps itself — task 5 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (`ROADMAP.md`: *updates
that never interrupt*, *atomic updates with rollback*, ★ *undo what the agent
did*, and the decidable half of *recovery and rollback screen*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os-3`
**Machine:** Windows Server 2022 (VMware guest), gated in Ubuntu 24.04 under
WSL2. No hardware acceptance is claimed and none was needed: nothing in this
task touches the machine.
**Status:** ready for integration.

## What changed, for somebody outside this repository

Nothing a person reads changed. What changed is that the sentences they read
about updates are now **a sequence held by a test** rather than forty-five
strings in two crates that nobody had ever walked end to end. Somebody can now
read, in this report, exactly what their machine says to them from the moment
an update exists, through applying it, to going back — sixteen sentences in the
order they meet them — and if any one of those sentences is reworded without
that table being rewritten with it, the build stops. The same test holds three
other promises the plan made: that every sentence either crate can say is in
the machine's one vocabulary with a note a translator can work from **and is
reachable from something a person can actually do**; that *this cannot be
undone* always says **why**, fourteen different whys, no two alike; and that no
sentence anywhere on this road names the machinery underneath it.

## The walk

Sixteen sentences, in order, produced by driving the real types rather than by
reading a list back to itself: `Standing::between`, `THE_RULE`,
`WhenItApplies`, `Staging::of`, `Since::between` onto `Entry::updated`,
`GoingBack::offered`, `GoingBack::when_word`, `Returning::of`, and
`Since::RolledBack` onto `Entry::rolled_back`. Two of the sixteen are refusals,
because a person who chooses the same thing twice is an ordinary person rather
than an error. The two read *afterwards* come out of the machine's own history.

| What has just happened | Word | What the person reads |
|---|---|---|
| They check, and the version offered is the one running | `keeping-up.up-to-date` | This machine is up to date |
| They check again later, and a different version is offered | `keeping-up.ready` | An update is ready. It will apply when you choose, and nothing you are doing will be interrupted until then |
| Beside it, the first of the three promises | `keeping-up.never.restarts` | An update never restarts this machine. It applies when you restart |
| the second | `keeping-up.never.closes-an-application` | An update never closes an application you have open |
| the third | `keeping-up.never.interrupts` | An update never interrupts what you are doing |
| The first of the two choices they have | `keeping-up.when.at-the-next-restart` | Apply it the next time I restart |
| the other | `keeping-up.when.now` | Restart now and apply it, which closes the applications that are open |
| They choose the next restart, and the machine prepares it | `keeping-up.waiting-for-the-restart` | The update will apply the next time you restart. Your files and settings stay as they are |
| They choose the same update a second time | `keeping-up.already-waiting` | This update is already waiting for your next restart |
| They restart, and afterwards their machine's history says | `recounting.outcome.updated` | this machine started on an updated version of its system, after the person restarted it |
| Going back to the version before is offered | `keeping-up.going-back.offered` | This machine can go back to the version of its system it ran before its last update. Your files and your own settings stay as they are. Accounts, passwords and settings for the whole machine that changed since that update go back to how they were |
| The first of the two choices they have | `keeping-up.going-back.at-the-next-restart` | Go back the next time I restart |
| the other | `keeping-up.going-back.now` | Restart now and go back, which closes the applications that are open |
| They choose the next restart, and the machine sets it | `keeping-up.going-back.waiting-for-the-restart` | This machine will go back to the version it ran before the next time you restart. Your files stay as they are |
| They ask to go back a second time | `keeping-up.going-back.already-waiting` | This machine will already go back to the version it ran before the next time you restart |
| They restart, and afterwards their machine's history says | `recounting.outcome.rolled-back` | this machine went back to the version of its system it ran before, as the person asked, after they restarted it |

Held by `THE_WALK` in
`crates/alo-updating/tests/what_a_person_is_told.rs`, compared against the walk
row by row — the key **and** the exact English — by
`the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table`
and `the_table_and_the_vocabulary_say_the_same_thing`.

**Measured that it catches a change.** With
`keeping-up.up-to-date` altered from *This machine is up to date* to *This
machine is up-to-date* in the gated tree and nothing else touched, both tests
failed, each naming the row and both sentences; the change was reverted and
they passed again. A test that could not be made to fail is not evidence.

## The other road: every refusal on the same walk

The walk is the road where everything works twice over. These are the sentences
a person meets when it does not, each distinct, each checked to be reachable
from a real refusal value rather than looked up by key:

| When | Word | What the person reads |
|---|---|---|
| The answer about updates made no sense | `keeping-up.answer-not-understood` | The answer about whether there is an update could not be understood, so nothing on this machine has changed |
| Which version runs could not be read | `keeping-up.running-not-known` | Which version of its system this machine is running could not be read, so nothing was changed |
| The machine moved on between the update being found and chosen | `keeping-up.changed-since-it-was-found` | This machine changed after the update was found, so nothing was changed. Check for the update again |
| The base would not prepare it | `keeping-up.not-prepared` | The update could not be prepared, so nothing was changed. The next restart starts this machine as it is now |
| It updated and could not write that down | `keeping-up.not-written-down` | This machine could not write down whether it started on an updated version of its system. Nothing else was changed |
| There is nothing to go back to | `keeping-up.going-back.nothing-before` | This machine has no earlier version of its system to go back to |
| What it ran before is gone | `keeping-up.going-back.no-longer-kept` | The version of its system this machine ran before is no longer kept on it, so it cannot go back to it |
| The machine changed after going back was offered | `keeping-up.going-back.changed-since-it-was-offered` | This machine changed after going back was offered, so nothing was changed. Look again at what it can go back to |
| Going back could not be prepared | `keeping-up.going-back.not-prepared` | Going back could not be prepared, so nothing was changed. The next restart starts this machine as it is now |

The two other refusals on this road — *this update is already waiting* and
*this machine will already go back* — are in the walk table above, because a
person choosing the same thing twice is part of the ordinary road rather than
the broken one. The test covers all eleven.

Six of the nine are held to saying, in so many words, that the machine is as it
was: the four that follow something the person chose, plus the answer that made
no sense and the version that could not be read. The other three do not say it,
deliberately — two are said *instead of* an offer, so nothing was started for
them to be about, and the third is the machine's history failing rather than
the machine. `every_refusal_on_the_road_is_said_and_no_two_read_alike` also
holds all eleven against hedging — *probably*, *might*, *perhaps*, *possibly*,
*try again later*.

## Why *this cannot be undone* names why

`a_refusal_to_put_something_back_always_says_why` walks all fourteen
`NotUndoable` reasons. Each one renders from the machine's own vocabulary, is
at least eight words long — a reason is a clause, not a label — and no two of
the fourteen read the same. The ones the plan names by hand are checked by
hand: *This left your machine, and nothing here can call it back*, *A model was
told this, and nothing can untell it*, *This was printed on paper, which your
machine cannot take back*. A change verb nobody has classified — the sentence
every verb added after today falls into — is checked too, so the honest answer
survives the machine gaining verbs.

## What changed, in the repository

| File | What changed |
|---|---|
| `crates/alo-updating/tests/what_a_person_is_told.rs` | New. Six tests: the walk against the table, the table against the vocabulary, every sentence declared-and-reachable-and-noted, every *cannot be undone* naming why, no machinery anywhere, and every refusal on the road |
| `crates/alo-updating/Cargo.toml` | One dev-dependency, `alo-recounting`, with the reason beside it: the *after* half of this task is that crate's four clauses |
| `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` | Task 5 marked done; task 6, *Finding out there is an update*, written |
| `Cargo.lock` | The dev-dependency edge — **and four entries that were already missing**: `alo-keeping-up`'s dev-dependencies on `alo-capability` and `alo-declared`, and `alo-updating`'s on the same two, are in those crates' `Cargo.toml` files and were not in the lock. Cargo wrote them on the first resolve here. Nothing was chosen by this task; a lock that had fallen behind two `Cargo.toml` files was brought up to date with them |

**No `src/` file changed, in any crate.** That is the shape of this task rather
than an omission: its constraint is *nothing here re-decides what the sentences
describe*, and reading all forty-five found nothing that was false or that read
badly. Where a sentence was worth keeping as it is, the test now says so.

## Decisions taken, and why

- **The test lives in `alo-updating`, not `alo-keeping-up`.** *Before and
  after* needs both halves. The offer and the promise are `alo-keeping-up`'s;
  *this machine started on an updated version of its system* is
  `alo-recounting`'s, read off an entry `alo-record` keeps. `alo-updating` is
  the one crate that already has the record and the decisions beside each other
  — it is why `src/putting_back.rs` is there — so the walk could be written
  without inventing a new home for it. A walk that stopped at the moment the
  person approved something would be half the promise.
- **The table lives in the test, and this report copies it.** The alternative —
  a test that reads this report and holds the code to it, the way
  `alo-appearance` holds the palette to `docs/design/palette.toml` — was
  rejected. `docs/autonomy/SHARED_MAIN.md` says a published report is never
  edited by anybody but its author; binding the build to one would mean the
  next person to reword a sentence has to break that rule to make the tests
  pass. A living artefact and a dated record want different homes, and the
  living one is the code.
- **Reachability is measured, not asserted.** *Every sentence this plan's
  crates can say* would be an empty claim if the test simply iterated
  `EVERY_WORD`. So the test renders every sentence through the public `said()`
  that produces it — `Standing`, `NotADigest`, `THE_RULE`, `Staging`,
  `NotStaged`, `GoingBack`, `CannotGoBack`, `Returning`, `AnUndo`,
  `NotUndoable`, `WhatWasKept::forgetting`, `NotAWindow`, and `alo-updating`'s
  own `NotRead`, `NotApplied`, `NotRecorded` and `NotGoneBack` — collects what
  came out, and requires that every one of `alo-keeping-up`'s forty-one words
  is in that set. A string declared and reachable from nothing now fails.
- **The machinery list grew by three.** `alo-keeping-up`'s own check already
  forbade thirteen words; this adds `subvolume`, `btrfs` and `filesystem`,
  because ADR 0045's undo road is the next thing that could leak a rented name
  into a sentence, and applies the whole list to `alo-recounting`'s four
  clauses as well. They passed unchanged.
- **The undo entries are real entries.** The two clauses read back after an
  undo are produced from `Entry::undone` and `Entry::undo_failed` over a
  `move_file` that went through the machine's own verbs, grants and one
  redeemed approval — the same fixture shape
  `tests/putting_back_what_an_agent_did.rs` uses. A fixture verb of the test's
  own would let the table agree with a machine that does not exist.

## What this task did not do

- **It re-decided nothing.** Every sentence in both tables existed before this
  change. The constraint was that a sentence that is true and reads badly
  changes, and one that reads well and is not true changes the other way;
  neither case was found.
- **It did not touch the recovery screen.** Drawing any of this is the shell
  plan's, and this task's output is the decisions it hands over.
- **It did not walk the undo road end to end**, because there is no end to walk
  to: on every machine alo OS installs today `WhatWasKept::NothingOnThisMachine`
  is the true answer, and the four lanes that change it (the installer's
  filesystem, the accounts lane's home subvolume, lane A's bracket, the
  broker's privilege) are named in task 4 and are not this plan's. The sentence
  a person meets there — *This machine does not keep what your files were
  before the agent changed them, so it cannot put them back* — is in the
  refusal table and is held to saying why.

## Verification

Every command below was run in the foreground on 2026-09-19 and its exit code
read. The source was copied to `/root/alo-trees/this-machine` and built into
`/root/alo-builds/this-machine`, which is what `tools/kernel-loop`'s
`where_it_builds.rs` and `gates.rs` choose for this machine; no other Cargo run
held them. The Windows host cannot build this workspace at all — `ring`'s build
script needs a C compiler that is not installed there — which is why every line
is a WSL line.

```
wsl -d Ubuntu -u root -- bash -lc '
  export PATH="/root/.cargo/bin:$PATH"
  export CARGO_TARGET_DIR=/root/alo-builds/this-machine
  cd /root/alo-trees/this-machine
  <command>
'
```

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all --check` | exit 0 |
| Clippy | `cargo clippy -p alo-updating --all-targets -- -D warnings` | exit 0, no warnings |
| Tests | `cargo test -p alo-updating` | exit 0 — 51 passed across eight targets, 4 ignored (the virtual-machine tests, which are run by name and were not run here) |
| Rustdoc | `RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-updating --no-deps` | exit 0 |
| The plan checks | `cargo test` in `tools/kernel-loop` | exit 0 — 151 passed, including `every_plan_this_repository_drives_holds_only_tasks` over the edited plan |

The full workspace suite was **not** run here, per this worker's instructions:
the supervisor runs it on the combined tree.

### Acceptance, one line per criterion

Each was run on its own, `--exact`, before this report was written.

| Acceptance | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| every sentence this plan's crates can say is in the vocabulary with a translator's note | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `every_sentence_these_crates_can_say_is_in_the_vocabulary_with_a_note` |
| a walk from *an update exists* to *it is applied* to *it is rolled back* is the exact sequence a person meets, held by one test | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table` |
| …and the table's keys are the keys that say it | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `the_table_and_the_vocabulary_say_the_same_thing` |
| the sentence for *this cannot be undone* names why | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `a_refusal_to_put_something_back_always_says_why` |
| no sentence names the machinery | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `no_sentence_a_person_reads_names_the_machinery` |
| the refusal paths beside the legitimate ones | `.` | `alo-updating` | `tests/what_a_person_is_told.rs` | `every_refusal_on_the_road_is_said_and_no_two_read_alike` |

All six: `test result: ok. 1 passed; 0 failed; 5 filtered out`.

### The second gate run, 2026-09-19

The first handover of this task was refused, and **not for anything in it**.
The supervisor's gates build on `/root/alo-builds`, and that filesystem had
under 12 GiB free; a build started there does not fail as a build, it fails as
a linker that cannot open a file, which reads like a broken change and is not
one. Nothing was staged, committed or pushed, and nothing was discarded: the
files below are the ones the first worker left, unchanged but for this section
of this report. No line of code, test or plan was rewritten to pass a gate.

Room was made by whoever owns that filesystem. At the start of this run it was
79 GiB with 74 GiB free, against `where_it_builds.rs`'s 12 GiB reserve, and
`/root/alo-builds/this-machine` was empty — so everything below is a cold build
rather than a reused one, and it left 71 GiB free behind it. Nothing shared was
deleted to get there. Every command was run in the foreground, in the order
shown, and its exit code read:

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt -p <crate> -- --check` for all 94 workspace members | exit 0, nothing reformatted. `cargo fmt --all` itself cannot run on this host: Windows refuses the command line rustfmt is given for 94 crates (`os error 206`, *the filename or extension is too long*), so the same check was run per package, plus `--manifest-path tools/graphics-check/Cargo.toml` for the one member whose directory and package names differ |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 in 2 m 31 s, no warnings — the whole workspace, not only the crate this touches |
| Tests | `cargo test -p alo-updating` | exit 0 — 51 passed across eight targets, 4 ignored |
| Tests, neighbours | `cargo test -p alo-keeping-up -p alo-recounting -p alo-saying` | exit 0 — 14 targets, every one `ok` |
| Rustdoc | `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` | exit 0 in 1 m 16 s |
| The plan checks | `cargo test` in `tools/kernel-loop` | exit 0 — 151 passed |
| The six acceptance tests | `cargo test -p alo-updating --test what_a_person_is_told -- --exact <name>`, each on its own | exit 0 each, `1 passed; 0 failed; 5 filtered out` |

**The mutation was repeated rather than taken on trust.** With
`keeping-up.up-to-date` changed from *This machine is up to date* to *This
machine is up-to-date* in the gated tree and nothing else touched,
`the_walk_from_an_update_existing_to_going_back_is_the_sequence_in_the_table`
and `the_table_and_the_vocabulary_say_the_same_thing` both reported `FAILED`
and the other four passed — `test result: FAILED. 4 passed; 2 failed`. The file
was restored from its copy and compared byte for byte with the checkout's
before anything else ran.

The full workspace suite was again **not** run here, per this worker's
instructions; the supervisor runs it on the combined tree.

## Limitations

- **English only.** There are no translations in this repository yet, so the
  table is the source language and every `Said` in it came from
  `CameFrom::TheSource`. What the test proves about another language is that
  the sentence has a key and a note and that a translation is checked against
  the same gaps — the machinery `alo-strings` already provides, not something
  measured here.
- **Not on the certified machine, and nothing to measure there.** No code path
  in this change reaches hardware.
- **The walk is one walk.** It is the road a person takes when everything
  works and when they choose twice; the refusal table beside it is the rest of
  what `alo-keeping-up` and `alo-updating` can say, but a second walk — check,
  no route out, check again — belongs to task 6, which has to build the
  checking first.

## Proposed shared-document updates

These are for the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under the current release:

> Every sentence alo OS says about updating itself, going back, and putting
> back what the agent did is now one sequence held by a test: what a person
> reads from *an update is ready* through *it is applied* to *it is back the
> way it was*, in order, with the exact wording written down beside it. A
> sentence cannot be reworded without the written record of it being reworded
> too. Every reason the machine gives for something it cannot put back now has
> to say **why**, and no sentence about updates is allowed to name any of the
> software alo OS rents underneath.

**`docs/autonomy/QUEUE.md`**: task 5 of the machine-keeps-itself plan is done;
task 6, *Finding out there is an update*, is ready and depends on tasks 1 and 2.

**`docs/autonomy/STATE.md`**: reference this report.

**`ROADMAP.md`**: no line ticks. *Updates that never interrupt* still waits on
task 6 — nothing on this machine has ever checked for an update.
