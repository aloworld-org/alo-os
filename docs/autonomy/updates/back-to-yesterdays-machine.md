# Back to yesterday's machine

**Date:** 2026-09-15
**Workstream:** v0.5 the machine keeps itself — task 3 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (`ROADMAP.md`: *atomic
updates with rollback*, and the decidable half of *recovery and rollback screen*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os`
**Status:** ready for integration. Measured in an emulated virtual machine on the
pinned base; not on the certified machine. Drawing it on the recovery screen is
the shell plan's, and calling `after_a_restart` at boot is still the image lane's
(see limitations).

## What changed, for a person

After an update, the machine can now go back to the version of its system it ran
before. It can say which version that was, when it was replaced, and whether it
still has a copy. If it can't go back — it has never updated, or the earlier
version is no longer kept — it says so instead of offering. Nothing starts and
then fails halfway. Before a person agrees, they are told what going back
changes. Their files and their own settings stay as they are. Accounts,
passwords and settings for the whole machine that changed since the update go
back to how they were. If an update is waiting, they are told that it will not
apply. Going back happens at the next restart, or now if they ask for that. The
machine's history then says it went back, from which version to which, and when.

## What changed, in the repository

**`crates/alo-keeping-up`** (decisions; still no clock, thread, socket, file or
process, and still four dependencies):

| File | What it holds |
|---|---|
| `src/before.rs` | `Changed` (what the record last says changed, digests only) and `Before::of(deployments, last_changed)`: the build before, from the base's kept deployment or else the record's last change *to the running build*; `is_kept`, `replaced_as_the_record_says` |
| `src/going_back.rs` | `GoingBack::offered` and `CannotGoBack` (`NotRunningABuild`, `NothingBefore`, `NoLongerKept`, `AlreadyGoingBack`, `ChangedSinceItWasOffered`); the sentence the person approves, with a separate sentence when an update waiting is set aside; `when_word` |
| `src/returning.rs` | `Returning::of(offer, deployments_now, when)`: `rollback`, plus `--apply` only for `NowBecauseThePersonAsked`; refuses a machine that is not what the offer described, or already going back |
| `src/deployments.rs` | Reads `status.rollbackQueued` (`is_going_back`, `going_back_at_the_next_restart`) |
| `src/since.rs` | `Since::between(last_known, going_back_to, deployments)` and `Since::RolledBack { from, to }` |
| `src/words.rs` | Ten sentences with notes; the machinery test now also forbids *rollback*, *roll back*, *rolled back*, *snapshot* |

**`crates/alo-updating`** (the doing):

| File | What it holds |
|---|---|
| `src/yesterday.rs` | `yesterday(base, record_at)` → `Yesterday`: running build, `Before`, `replaced_at` (the moment on the record's matching entry), `going_back()` offer or refusal, `record_not_read` |
| `src/going_back.rs` | `go_back(base, offer, kept, when)`: status now → `Returning` → note the build → run the base |
| `src/across_restarts.rs` | `AcrossRestarts`: `/var/lib/alo/last-known-build` and `/var/lib/alo/going-back-to` as one value |
| `src/one_build.rs` | One digest in one file, written whole or not at all, read strictly, cleared; `last_known.rs` now uses it |
| `src/restarted.rs` | `after_a_restart(base, kept, record, at)` writes `updated` or `rolled-back`, then the last known build, then clears the note |
| `src/refusing.rs` | `NotGoneBack`; `NotRecorded::GoingBackNotRead` / `GoingBackNotCleared` |
| `tests/going_back_to_yesterdays_machine.rs` | Legitimate and refusal paths against a stand-in base that writes down every question |
| `tests/back_to_yesterdays_machine.rs` | The virtual-machine test |
| `tests/applying_an_update.rs`, `tests/an_update_keeps_the_persons_things.rs` | Call sites moved to `AcrossRestarts`, nothing else |

**Other crates' surfaces, additive:** `alo-record` `Happened::RolledBack` and
`Entry::rolled_back` (JSON `"rolled-back":{"from":…,"to":…}`, no agent, not
egress), added to every accessor. `alo-recounting` `Outcome::RolledBack` and
`recounting.outcome.rolled-back`, in `EVERY_WORD` and `EVERY_OUTCOME`.
`docs/contracts/record-file.md` documents the tag; entries without an agent are
now six. `docs/quirks.md` has *Going back to the build before does not bring
back `/etc` as it is now*.

## Decisions

- **A return is told apart from an update by what the person chose, not
  guessed.** Both look the same afterwards: a different build booted, with the
  other one kept. So `go_back` notes the chosen build **before** it tells the base.
  At the next start, that build booting is `rolled-back`, and anything else is
  `updated`. Digests have no order, so "older" could not decide it. This also
  closes the limitation task 2 left, where a rollback would have read as
  `updated`.
- **The note comes first, and a note left by a refusing base is not removed.**
  Without the note, a return would be recorded as an update. So if the note
  can't be written, the base is told nothing (`NotNoted`). If the base refuses
  after the note is written, the note stays. If the base actually set the return
  before failing, the next start is recorded correctly. If it didn't, the machine
  starts on the same build (`Unchanged`, and a note naming another build is
  kept). The next change then clears the note and is recorded as the update it
  is. A note naming the running build with nothing changed is the tail of a
  return already written, and is cleared.
- **Order at start: entry, last known, clear the note.** A crash between steps
  duplicates the entry, as task 2 already accepts. It never loses the entry and
  never mislabels it.
- **Agreement with the offer is checked, not assumed.** `bootc rollback` takes
  no target: it reorders what the base has. If anything changes after the offer
  (the running build, the kept build, an update that started waiting), the same
  command would do something other than what the person approved. So that is
  `ChangedSinceItWasOffered`. Going back that is already set is refused twice
  over (`GoingBack::offered` and `Returning::of`), because a second `rollback`
  turns the machine round again. The virtual machine confirmed a refused second
  request left `rollbackQueued` set.
- **An update waiting is set aside by the base, and the person is told.** I
  offer the return rather than refuse it: the recovery screen is most needed when
  something is wrong. But it uses its own sentence (*the update waiting for your
  next restart will not apply*), because the return undoes a choice the person
  made.
- **"What it was" is the build by digest, and "when it was replaced" is the
  record's moment.** The base's status has no deployment time. The `updated` or
  `rolled-back` entry is exactly the first start on the replacing build. It is
  used only when that entry ends at the build running now. A kept build the
  record says nothing about is still named, with no moment invented. A build the
  record names but the base no longer keeps is named, and `NoLongerKept`. The
  image's version label is not read: nothing is decided by it, and the shell can
  ask for it additively.
- **A record that can't be read leaves only *when* unknown.** A damaged record
  should not block a return the base can make. `record_not_read` says why the
  moment is missing.
- **The time lives in `alo-updating`.** `alo-keeping-up`'s test forbids naming
  `SystemTime`, and loosening it would narrow task 1's promise. So `Before` and
  `Changed` hold digests, and `Yesterday` carries the moment beside them.
- **`after_a_restart` now takes `&AcrossRestarts`, not a path.** A caller that
  keeps one of the two facts has to name where the other is. The only callers are
  this crate's tests. Task 2's proposed boot unit uses
  `AcrossRestarts::on_this_machine()`.
- **`/etc` reverting is said, not hidden or patched.** Measured and in
  `docs/quirks.md`. ADR 0011 forbids patching the base. The honest sentence is
  the deliverable, and the test fails if a later base changes the behaviour, so
  the sentence has to change with it.
- **Editing `alo-record` and `alo-recounting` again**, for the reason task 2
  gave: the acceptance asks for a record entry, no existing kind fits, and the
  contract allows additive kinds. Both edits are minimal.
- **The virtual-machine test copies its host machinery from task 2's test**
  instead of sharing a module. Moving task 2's test onto a shared module would
  change a measured test that I couldn't re-run within this task's time (it takes
  about 16 minutes on top of this one's 27). Proposed follow-up below.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The previous deployment is named and readable: what, when replaced, still on the disk | `alo-updating` `going_back_to_yesterdays_machine::going_back_after_an_update_is_named_set_with_one_instruction_and_recorded_once`; `alo-keeping-up` lib `before::tests::a_build_no_longer_on_the_disk_is_named_from_the_record_and_not_kept` |
| A rollback is one act, tested in a VM: update, roll back, earlier digest running, files untouched | `alo-updating` `back_to_yesterdays_machine::going_back_in_a_virtual_machine_runs_the_earlier_build_and_leaves_the_persons_things` |
| A rollback that cannot be done says so before it is offered, in the vocabulary | `going_back_to_yesterdays_machine::a_return_that_cannot_be_done_is_said_before_it_is_offered`; `alo-keeping-up` lib `going_back::tests::a_build_before_that_is_no_longer_kept_is_refused_before_it_is_offered` |
| One approval, one execution; a changed machine is refused and the base told nothing | `going_back_to_yesterdays_machine::going_back_already_set_is_refused_and_the_base_is_told_once`, `…a_machine_that_changed_since_the_offer_is_refused_and_nothing_is_noted`, `…going_back_that_cannot_be_noted_tells_the_base_nothing`, `…a_base_that_will_not_go_back_is_said_as_nothing_changed_and_the_note_does_no_harm` |
| The record says the machine rolled back, to which digest and when | `alo-record` lib `entry::tests::going_back_is_recorded_from_one_build_to_the_one_before_with_nobody_behind_it`; `alo-keeping-up` lib `since::tests::the_build_chosen_to_go_back_to_booting_is_a_return_and_any_other_is_an_update`; `going_back_to_yesterdays_machine::a_note_that_cannot_be_read_writes_nothing_down` |
| What a person is shown is decisions handed to the shell, not drawn here | `Yesterday`, `GoingBack::said`, `CannotGoBack::said`, `GoingBack::when_word`, `Returning::said`; `alo-keeping-up` lib `words::tests::nothing_here_names_the_machinery_hedges_or_calls_an_update_urgent` |

## Verification

Run on 2026-09-15 in WSL Ubuntu 24.04 on the Windows Server gate machine, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p alo-updating -p alo-keeping-up -p alo-record -p alo-recounting --all-targets -- -D warnings` | exit 0 |
| `cargo test -p alo-keeping-up -p alo-updating -p alo-record -p alo-recounting -p alo-saying -p alo-collected` | exit 0: keeping-up 47 unit + 10; updating 9 unit + 10 + 9 + 1 (2 ignored) + 1 (2 ignored); record 74 + 2; recounting 55 + 16; saying 63 + 4 + 1; collected 8 + 11; doctests pass |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-updating -p alo-keeping-up -p alo-record -p alo-recounting --no-deps` | clean |
| `cargo test -p alo-updating --test back_to_yesterdays_machine -- --exact --ignored going_back_in_a_virtual_machine_runs_the_earlier_build_and_leaves_the_persons_things` | **passed, 1586.92 s** |

**The virtual machine run.** QEMU 8.2.2, `-accel tcg`, q35, OVMF, 3 CPUs, 3 GB;
bootc 1.15.1 from the pinned base. The console, in order:

- first start (233 s): running
  `sha256:add6d582ab6522fcb0c45d0b7e1cfa68454e3c52a5e80c3d443d4efe5dffa76f`;
  *nothing to go back to, said before anything was offered*; the update staged;
  passed at 673 s;
- second start: the update recorded; *the build before is sha256:add6d582…,
  kept, replaced when the record says*; a letter written in the person's folder
  and a file under `/etc/alo/`; *going back is set for the next restart*; a
  second request refused as already going back, with `rollbackQueued` still set;
  passed at 387 s after boot;
- third start: the first build running with its own system files and the second
  build kept; every named thing, the letter written after the update, and the
  record byte for byte; the `/etc/alo/` file written after the update absent;
  one `rolled-back` entry from the second build to the first, with no agent;
  none added on a second start; the second build now named as the one before,
  kept, and going back offered again. Powered off.

The third start's console lines aren't quoted: the test deletes its working
folder when it passes. The assertions behind each line above are in the test.

Not run: the full workspace suite, which the supervisor runs. Task 2's
virtual-machine test was not re-run. Its only change is that `after_a_restart`
takes `&AcrossRestarts::on_this_machine()`, and this run exercised the same
update path end to end. Nothing was measured on the certified laptop.

## Remaining limitations

- **Nothing calls these from the shell or at boot yet.** The recovery screen
  (shell plan) draws `yesterday()` and calls `go_back`. The boot unit proposed by
  task 2 (image lane) calls `after_a_restart` with
  `AcrossRestarts::on_this_machine()`. Until that unit exists, neither `updated`
  nor `rolled-back` is written on the shipped image.
- **`/etc` on going back** is the base's behaviour, and is said rather than
  changed. If alo OS ever needs whole-machine configuration to survive a return,
  that is an ADR, not a patch.
- **Choosing an update after setting a return** replaces the return. That matches
  task 2's rule that a later choice replaces a waiting one, and the start is
  recorded as `updated`. Nothing warns about it yet. The shell can check
  `Deployments::is_going_back` before offering an update.
- **The base's `bootc-fetch-apply-updates.timer`** would undo a return, and an
  update the person didn't choose would break `THE_RULE`. It reported *disabled*
  in a container of the pinned base. Whether it is masked on the booted shipped
  image is for the image lane to confirm.
- **Shared VM test machinery:** the two virtual-machine tests duplicate their
  host half (images, registry, QEMU, console). Proposed follow-up: move both onto
  `tests/a_virtual_machine/mod.rs` and re-run both.
- Measured in an emulated VM, not on the certified laptop.

## Proposed shared-document updates

- **CHANGELOG.md:** "After an update, alo OS can go back to the version it ran
  before. It says which version that was, when it was replaced, and whether it
  still has a copy. If it can't go back, it says so before offering. Before you
  agree, it tells you that your files and settings stay as they are, and that
  accounts, passwords and whole-machine settings changed since the update go back
  with it. The machine's history records the return. Measured in a virtual
  machine."
- **ROADMAP.md:** v0.5 *atomic updates with rollback*: going back, its refusals
  and its record entry are in (`crates/alo-keeping-up`, `crates/alo-updating`).
  The boot unit, the signature policy in the image, and the recovery screen
  remain.
- **QUEUE.md / STATE.md:** machine-keeps-itself plan task 3 done; task 4 *Undo
  what the agent did* next.
