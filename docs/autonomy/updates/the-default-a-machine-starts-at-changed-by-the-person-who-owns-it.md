# The default a machine starts at, changed by the person who owns it

**Date:** 2026-09-22
**Workstream:** `docs/autonomy/v0-5-the-installer-plan.md`, task 17 — the road a
person's choice travels to reach the loader's own file, and the surface in
Settings in front of it.
**Contributor:** the third PC (`AGAI01`), one working tree, two workers — the
second took up the first's code untouched and answered the ownership refusal it
was handed (*What the first attempt was refused for*, below).
**Decisions it rests on:**
[ADR 0066](../../decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
(which system a machine starts by default is changed by a verb, and kept in one
place both systems can reach), which is what this builds;
[ADR 0062](../../decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
term 3 (one copy of the last choice, in the loader's own file);
[ADR 0001](../../decisions/0001-the-capability-model.md) §1–2 (a privileged act
is an enumerated verb on the broker's closed list);
[ADR 0011](../../decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)
(the base's loader is configured, never patched).
**Status:** ready for integration. **Nothing in it is ticked on a machine**,
which is what this task's own constraint asks.

## What a person gets out of this

A computer with both alo OS and the Windows it came with says, in Settings,
*This computer starts alo OS when nobody chooses* — and lets the person change
it. Choosing the other system asks them to approve one sentence, and from then
on that is what the machine starts on its own after the five-second countdown.
Nothing else about the machine changes: *Restart into Windows* still means once,
and this still means always. If it could not be done — because there is no
Windows on the computer, or because the file that remembers could not be
written — the person is told which, in a sentence that says nothing was changed
and what to do instead, and the computer behaves exactly as it did before.

## The part a. of the task asked about, and why no ADR is written here

Task 17 says the road *probably needs a decision*, and that if the answer is a
thirteenth member of `alo_broker::SystemVerb` then the ADR is written first and
handed over as the task. **The ADR already exists.** ADR 0066 was accepted and
merged (`d62e1bb`, PR #116) before this work began; it weighs the alternatives
task 17a lists — a `Switch` argument, a person's act with no agent verb, a saved
default only the loader may write — and decides for the verb, with the argument
being *the identity of a system the menu already offers*. So the decision half
of this task was already done, and what was left was b. (the road) and c. (the
surface), which is what is here.

The two options the ADR set aside are worth restating, because a reader will
ask. A **`Switch`** would fit the arithmetic — there are exactly two systems —
and reads as *on* and *off* at the door and in the record, where what a person
approved was *start Windows* or *start alo OS*; an identity says which system in
the shape every other verb already uses. **Settings offering nothing but *choose
at the menu*** would need no verb at all, and would leave a machine whose owner
must reboot and catch a five-second countdown to change a setting — which is not
a setting.

## What changed

### `alo_broker::SystemVerb` gains its thirteenth member

`starting.default`, taking an identity, in `crates/alo-broker/src/verbs.rs`. Its
argument is the identity of **a system the machine's own menu offers**: the
SHA-256 of `alo-starting system 1`, a zero byte, and the word the generated menu
knows that system by. So nothing on this road names a file, a path, a partition
or a loader — at the door or behind it — which is what ADR 0001 §2's *no
free-form parameters* asks and what ADR 0066 §2 spells out for this verb.

`crates/alo-letting-go/tests/nothing_here_is_a_verb.rs` moved its count of the
broker's list from twelve to thirteen, **in this change**, with ADR 0066 named
beside it and the reason written out. That count is a tripwire and it did its
job again.

### `crates/alo-starting` gains the road, in five new files

| file | what it is |
|---|---|
| `offered.rs` | The identity of a system the menu offers, made the one way both sides of the door make it — `the_identity_of`, `the_system_named`, `System::offered_as` |
| `loader.rs` | `TheLoader`: the loader's two files as whoever changes them reaches them, in three methods over **bytes**, with `NotRead` and `NotWritten` |
| `on_this_machine.rs` | `TheLoadersFiles`: that trait over the two files — the menu under `/boot`, the environment block on the EFI system partition — reading the menu to answer *what does this machine offer* and writing the block **in place** |
| `by_default.rs` | `start_by_default`: the whole act, in order, each step refusing before the next, with `NotSet` naming every way it can refuse |
| `by_hand.rs` | `to_start_by_default(System)`: a person's own choice in Settings becoming the broker's own verb |

and four existing ones changed: `wanted.rs` (`Change` gains `StartByDefault`, and
says why the next start and the default are two changes), `refusing.rs`
(`NotChanged::NotRemembered`, and `from_the_brokers` now takes the change as
well as the answer), `words.rs` (three new sentences, and two more in the
groups), `menu.rs` (`Menu::offers_windows`, which reads the generated file for
the identifier the menu itself writes).

### Carried out behind the broker's door

`alo_brokerd::ByDefault` (`crates/alo-brokerd/src/by_default.rs`), supplied
additively through `Carriers::with_by_default(…)`; the published
`Carriers::of(network, proxy, storage)` keeps its signature and refuses the verb
by name until the loader's files are supplied.
`crates/alo-brokerd/src/main.rs` supplies this machine's own. Nothing in the
broker knows the shape of those files: it hands the verb's argument to
`alo_starting::start_by_default` and turns what comes back into the door's one
word.

### Everything the new member of a closed enum touched

`crates/alo-changing-drives`, `-printers` and `-updates` each gained the new arm
in their exhaustive match, and so did `crates/alo-brokerd/src/printers.rs`.
`crates/alo-saying` collects the new vocabulary with no change of its own — its
counts are computed rather than written down.

### Documentation

`docs/contracts/agent-verbs.md` gains the verb additively: the row in the table,
the list's count, what a system's identity is made of, how it is carried out and
each way it is refused. It also **corrects a sentence that ADR 0066 overtook** —
the contract said there was no verb for the menu's default and that the answer
was read and written by `alo_starting::TheStartingChoice` and nowhere else. The
second half is still true; the first is not, and a contract that says the
opposite of an accepted decision is how a rule stops being one.
`docs/booting.md` §3 now says what Settings shows, that it does not write that
file itself, and that the Windows side of the same setting is the installer's
program writing the same file — one setting, one place, two sides.

### The owners' releases, recorded where the mechanism reads them

Four of the files above are in crates this plan does not own, and both of the
plans that do own them still have unfinished tasks — so the supervisor's
ownership check refuses them, correctly, unless the owning plan's own header
records that the owner released them. `docs/autonomy/LOOP.md` calls this an
owner-release block, and the first attempt at this task was refused for want of
one; the refusal is quoted under *Verification* below, with what it cost.

So this change carries two of them, each in the header of the plan that owns
the files, each naming ADR 0066 as the owner decision it records:

- `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` releases
  `crates/alo-broker/src/lib.rs`, `crates/alo-broker/src/verbs.rs`,
  `crates/alo-changing-drives/src/wanted.rs` and
  `crates/alo-changing-updates/src/wanted.rs` to task 17 of this plan. It is
  the same shape as the release that plan already made for task 16's verb, and
  the prose beside it says what a closed enum costs: every carrier is obliged
  to say what it does with a new member, which here is one line apiece saying
  *not mine*.
- `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` releases
  `crates/alo-letting-go/tests/nothing_here_is_a_verb.rs`, the count of the
  broker's list, and says what that count is for and why this verb does not
  reach it: ADR 0045's seventh term is about **undo**, and `starting.default`
  is a verb over which system starts, not over the written-down past.

**`alo-changing-printers` is deliberately not in either record.** It gains the
same one line, and no plan's header claims it — so no plan has the standing to
release it, and neither ownership check refuses it. A release naming a file its
own plan does not own would be a record asserting something it cannot; the
broker plan's task-16 record leaves it out for the same reason, and the line
saying so is in that plan beside the block.

Neither release transfers a crate, finishes anybody's task, or widens this
plan's header. They are records of a decision the owner already made, which is
what LOOP.md says they are and the only thing they may be.

## Where the last choice is kept, and how that was settled

The first version of this change kept the last choice at `/boot/grub2/grubenv`,
which is where task 16 put it under ADR 0062. ADR 0066 term 1 moved it: *the
saved default is kept in the loader's environment on the EFI system partition —
the one filesystem both systems can read and write.* Left under `/boot`, term 3
of the same ADR — the Windows program writing the same file — is not merely
unbuilt but impossible, and the test that says *one answer, and both sides see
it* would have gone on passing while the promise behind it was false. The owner
called that the load-bearing term, and asked for the path to be **measured**
rather than assumed, because some Fedora/UEFI layouts already keep that file on
the ESP through a symlink and inventing a second location would be a patch to a
rented engine in spirit if not in bytes.

**It was measured, on the base `image/Containerfile` pins, at its digest.** The
base's own `bootc install to-disk` wrote a disk; the disk was read; and the disk
was booted under OVMF in QEMU with no KVM, which this machine has none of.
`docs/quirks.md` has the whole of it. In short: this base keeps `grubenv` as a
**plain file on `/boot`**, not a symlink; the ESP has **no block at all**; the
block the base installs holds **nothing**; and **nothing mounts the ESP**. So
there was no file of the base's to adopt, and none left behind keeping a second
answer — the owner's first branch did not apply, and the second did.

**What the loader can actually do there was proved, not believed.** FAT and
`save_env` have a reputation rather than a specification, so: the base's own
`grubx64.efi`, under real firmware, read a value from a block on the ESP, saved
a new one, re-read its own write, and left the new value on the partition with
the machine powered off. `$cmdpath` turned out to be **empty** in this chain, so
the ESP is found the way the Windows entry beside it is found — `search --file`,
by the block itself, the same answer computed at every start rather than a disk
identifier written down once.

**And it was proved on the artifact rather than on a description of it.** The
drop-in was taken from the generator itself, not retyped, copied onto an
installed disk unchanged, and booted: the machine drew its menu with the Windows
entry preselected from the ESP's block, started it, and the entry's own
`save_env` line left the new value there — with `/boot`'s block still holding
nothing. One copy, on the partition both systems share, read and written by the
loader.

What this costs is a mount the base does not make. That is written down in this
plan and in `docs/quirks.md`, and owed by the lane that owns the installer.

## Decisions taken, and why

Nobody was waiting to answer these, so they were decided the way a senior
engineer would and are written down here.

**1. The identity is over what the *menu* offers, never over what the firmware
reports.** `Entry`'s identity (task 16's) is over bytes a firmware reported and
differs from machine to machine; this one is over a word this crate chooses, and
is the same on every machine. That is deliberate: what the verb names is *which
of the two systems*, not *which entry on this computer*. Whether this machine
has that system at all is a separate question, asked of the machine where the
change is made — so an approval made in Settings is still refused if the machine
turns out not to offer it.

**2. Whether this machine offers Windows is read off the generated menu file.**
Not a setting beside it, not a value the installer wrote down once. A record of
*there is a Windows here* kept anywhere else would be a second copy of a fact
about somebody's disk, of exactly the kind ADR 0062's third term refuses, and it
would be wrong the first time somebody removed one of the two systems. The
firmware would have been the other candidate — it is what `NextStart` asks — and
it was not chosen because ADR 0066 §2's own words are *a system the menu already
offers*, and because a machine can have a Windows partition with no firmware
entry and the menu finds it by searching for its loader at every start.

**3. A machine with no menu of ours offers alo OS, and that is an answer rather
than an error.** It is what a machine alo OS *replaced* Windows on is: there is
nothing to choose between, so no menu was ever generated. Every other reason the
file could not be read is a refusal — the difference between *there is no
Windows here* and *this could not be read* is the difference between a person
who can stop looking and one who should try again.

**4. The environment block is written in place, and never replaced.** Opened for
writing, its bytes overwritten where they are: not truncated, not created if
absent, and never written to a new file and renamed over. The loader saves into
that file from inside the loader, with no filesystem driver that could grow one
or follow it somewhere new, so each of those would hand it a file it may no
longer be able to save into — which is the last choice silently stopping being
kept. `EnvironmentBlock::written` refusing to produce anything but the length it
was read at is what makes that safe to say, and the test reads the file's size
back afterwards.

**5. The change is carried out even when the file already says so.** An
execution that quietly did nothing because the answer was already that would
make what the record shows depend on a state nobody can see. Writing is
idempotent — one setting, whatever it is written — and the test that makes the
same choice three times shows the file holding one answer.

**6. `NotChanged::from_the_brokers` now takes the change as well as the
answer.** The door has one word for *it was not carried out*, and it has to
become two different sentences: a person whose machine would not be told to
start Windows once can switch it off and on and choose Windows at the menu, and
a person whose machine would not keep which system it starts has a computer that
behaves exactly as it did. One sentence for both would tell one of them
something untrue about their own machine. This changed a function task 16
published; it is a day old, `alo-starting` is its only caller, and an internal
Rust signature is not one of the public surfaces `CLAUDE.md` holds to additive
change.

**7. The asking road is deliberately not built here.** A surface issuing a token
at the broker's door under `alo_broker::BY_HAND` exists for printers and the
network (`alo-changing-printers/src/carrying_out.rs`) and does **not** exist for
updates, storage or `starting.windows-next`. This crate follows the second
shape: it produces the verb and the sentences, and whatever runs Settings asks.
Building a second asking road here would have been this task inventing a
component neither the plan nor ADR 0066 asks for, in a crate that deliberately
depends on no socket.

**8. No agent verb was declared — and task 16's reason for that no longer
holds, so here is the real one.** Task 16 said an `alo_capability::Verb` would
need a promise in `docs/features.md` with a tier for `alo-by-hand` to answer
against (ADR 0009), and that adding one is the owner's. **That promise now
exists**: the change that accepted ADR 0066 (`d62e1bb`) added a `[v0.5]` line to
`docs/features.md` — *Which system the machine starts by default … alo OS
changes it through a verb on the broker's list, never by a person writing under
`/boot`* — and ADR 0066 §2 says in as many words that *an agent may ask for it
under a grant*. So the promise is no longer what stands in the way.

What stands in the way is **ownership**, and it is written into this plan's own
first page: *Nothing in `crates/alo-shell`, nothing in `alo-nearby`,
`alo-asking`, `alo-record`, `alo-capability`, `alo-turn`, `alo-egress` (lane
A's)*. An agent verb is a member of `alo_capability`'s list, dispatched in
`alo-agentd`, answered by `alo-by-hand`, and declared in the agent half of
`docs/contracts/agent-verbs.md` — all of it outside what this plan may edit, and
two lanes in one crate is the collision the lane table exists to prevent. So it
is **written down and passed across** rather than done here: the promise, the
decision and the broker verb are all in place, and declaring the agent verb is a
small change in lane A's crates whenever that lane takes it. The broker verb is
the road in the meantime, which is the shape `updates.apply` and `storage.mount`
already have.

## The refusal paths, and where each is tested

Every one of these leaves the loader's file exactly as it was, and every one of
them is in the record.

| refused when | where it is refused | the test |
|---|---|---|
| the identity names neither system | `start_by_default`, step 1 | `an_identity_that_names_no_system_changes_nothing` |
| this machine's menu does not offer it | step 2 | `a_machine_with_no_windows_is_not_set_to_start_one` |
| the menu could not be read at all | step 2 | `a_menu_that_could_not_be_read_is_not_read_as_no_windows` |
| the file could not be read | step 3 | `a_file_that_could_not_be_read_is_not_written_from_nothing` |
| what is there is not an environment block | step 3 | `a_file_that_is_not_this_machines_is_not_written_over` |
| it will not hold another setting | step 4 | `a_file_that_will_not_hold_the_answer_changes_nothing` |
| it would not be written | step 5 | `a_file_that_would_not_be_written_is_a_refusal` |
| the broker was built with no loader | the carriers | `a_broker_with_no_loader_refuses_the_verb_by_name` |
| the approval was for the other verb | the door | `an_approval_for_the_next_start_does_not_change_the_default` |
| the approval was already spent | the door | `one_approval_sets_the_default_exactly_once` |

## Acceptance, criterion by criterion

| what task 17 asks | what shows it |
|---|---|
| a person's change reaches that one file **and nothing else** | `alo-starting`, `the_last_choice_has_one_place_and_no_copy`, `the_road_a_persons_change_travels_ends_in_that_one_file` — the whole road over real files, checking the menu is unchanged and that nothing was written beside the loader's two files |
| **under one approval** | `alo-brokerd`, `only_the_default_approved_is_set`, `one_approval_sets_the_default_exactly_once` |
| **recorded either way** | the same target, `every_answer_is_in_the_record` |
| the test that finds no second copy still passes, and now covers the new road | `nothing_outside_this_crate_keeps_a_second_copy_of_the_last_choice`, in a file this change adds two tests of the new road to |
| every refusal is a sentence with a translator's note | `alo-starting` `lib`, `words::tests::every_refusal_says_nothing_changed_and_what_to_do`, over a list of ten that now includes the new one; and `refusing::tests::not_carried_reads_as_the_road_it_was_on` |
| the ADR is in `docs/decisions/` and the count moved in the same change | `alo-letting-go`, `nothing_here_is_a_verb`, `the_brokers_list_is_exactly_as_long_as_it_was` |

## What this does not do, and who owes it

- **The loader's half has now run on a machine; alo OS's half has not.** The
  configuration this generates was booted — real UEFI firmware, the base's own
  signed loader, an installed disk — and what that proved is in the section
  above and in `docs/quirks.md`. What has *not* run on a machine is the Rust:
  `TheLoadersFiles` has been run against files in a directory and never against
  a mounted ESP, `alo-brokerd` has never carried this verb on a real machine,
  and no person has ever changed this setting in Settings.
- **What a `bootupd` update does to the block is not measured, and it now
  matters more than it did.** The block is in `EFI/fedora/`, which is the
  directory `bootupd` owns. What can be said is that `bootupd` 0.2.31 records
  the files it installed — `bootupd-state.json` names seven under `EFI/`, and
  the block is not one of them — which is a reason to expect it is left alone,
  and not a measurement of an update. Measuring it needs two versions of the
  base, and belongs with task 4's walk. If it turns out `bootupd` does clear the
  directory, the answer is not to move the block back to `/boot` — that breaks
  ADR 0066 term 1 — but to put it back after an update, and the finding belongs
  in `docs/quirks.md` beside the rest.
- **Who writes `custom.cfg` and the environment block onto a machine at install
  time is still task 4's**, as this task's constraint says, and
  `crates/alo-installer` and `crates/alo-installing` were not edited.
- **The Windows side of the same setting is task 4's** (ADR 0066 §3). This
  change makes the alo OS side read and write the one file; the test that the
  two sides see each other's change needs both, and the Windows half does not
  exist yet.
- **The agent verb is owed by lane A**, for the reason under decision 8: the
  promise in `docs/features.md` and ADR 0066 §2 both have an agent asking for
  this under a grant, and `crates/alo-capability`, `alo-agentd` and
  `alo-by-hand` are not this plan's to edit. Everything it would need on this
  side exists.
- **`docs/quirks.md` gains one entry**, because reality and the specification
  did disagree: the base's loader keeps its saved default on `/boot` as a plain
  file rather than on the ESP through a symlink, and mounts the ESP nowhere at
  all. The entry carries the versions, the four things measured, the two things
  proved about the loader, and what the installer lane is owed as a result.

## What the first attempt was refused for, and what fixed it

This task was handed over once before, on 2026-09-22, and the supervisor's
ownership check refused it without gating anything. Verbatim:

> this task changes a crate that is not `docs/autonomy/v0-5-the-installer-plan.md`'s
> to change right now: `alo-broker` belongs to
> `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, which still has an
> unfinished task; `alo-changing-drives` … `alo-changing-updates` … ;
> `alo-letting-go` belongs to
> `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, which still has an
> unfinished task.

**Nothing was wrong with the code, and nothing about it was changed to fix
this.** The two owner-release records described above were written three
minutes *after* the handoff was read, and they were not among the files the
handoff named — so the check ran against a tree that did not have them, and
would have kept doing so however many times it ran, because the loop stages
only what a task names. The whole fix is that the two owning plans are now part
of this change and are listed in the handoff, plus the one correction to the
broker plan's record (dropping `alo-changing-printers`, which that plan does
not own).

It is worth writing down for the next lane that meets it: a release block is
part of the change that uses it, exactly as LOOP.md says — *record the release
with the receiving task's gated publication* — and a release block sitting in a
working tree that no handoff names is invisible to the check it exists for.

That the two checks now pass was verified against the supervisor's own code
rather than argued: `owner_releases::permits` and `who_owns::refusal` were run
over the real plan files and this change's real file list, in a scratch copy of
`tools/kernel-loop` that was restored afterwards and is not part of this
change. Five files come back released — the four from the broker plan and the
one from the machine-keeps-itself plan — and both refusals come back `None`.

## Verification

Run from the checkout, in WSL Ubuntu, against a copy of the tree at
`/root/alo-trees/this-machine`, building in `/root/alo-builds/this-machine`.
Toolchain `cargo 1.98.1 (797e8a9bc 2026-08-05)`. The table below was run again
in full on 2026-09-22 after the release records were corrected; every row is a
result seen, not a result remembered.

| what | result |
|---|---|
| `cargo fmt --all --check` (workspace) | clean, exit 0 |
| `cargo fmt -p <crate>` for each of the seven crates touched | nothing to change |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, zero warnings, exit 0 |
| `cargo test -p alo-broker` | pass — 27 + 3 + 2 + 3 + 3 + 3 + 4 + 3 |
| `cargo test -p alo-starting` | pass — 82 unit, 6 + 5 + 3 integration |
| `cargo test -p alo-brokerd` | pass — 53 unit and fifteen integration targets, the new `only_the_default_approved_is_set` among them at 10 |
| `cargo test -p alo-changing-drives` | pass — 18 + 7 + 4 |
| `cargo test -p alo-changing-printers` | pass — 11 + 9 |
| `cargo test -p alo-changing-updates` | pass — 11 |
| `cargo test -p alo-letting-go` | pass — 69 unit and twelve integration targets (4 ignored, all pre-existing) |
| `cargo test -p alo-citing` | pass — 21 + 10, run because this change edits four documents |
| `cargo test -p alo-saying`, `-p alo-collected` | pass — the vocabulary's five new sentences are collected |
| `cargo test` in `tools/kernel-loop` | pass (152), run because this change edits three plans |
| each of the seventeen evidence tests, alone, with `--exact` | one passing each, exit 0 each |

`cargo doc --workspace --no-deps` with `RUSTDOCFLAGS="-D warnings"` was clean on
the first attempt's tree and was **not** re-run for this one, because nothing
between them is Rust: the second attempt changes three Markdown files and no
line of code. Said rather than implied, so nobody reads the table as a claim it
is not.

**The whole-workspace suite was not run here**, by instruction: it takes the
better part of an hour on this machine and the supervisor runs it after this.
What is above is every gate this change can reach, plus the supervisor's own
tests, which its plan edits reach.

**WSL is development evidence and never certified-hardware acceptance.** Nothing
in this report may be read as the latter.

## Proposed shared-document changes

For the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under the unreleased heading:

> A computer with both alo OS and Windows on it can now be told, in Settings,
> which of them to start when nobody chooses — and it remembers in the one place
> both systems read, so the menu and the setting can never disagree. The change
> is the person's own approval, recorded like every other, and it is refused
> with a sentence they can read when there is no second system to start or the
> computer would not keep the answer. Restarting into Windows for one start is
> unchanged and untouched by it.

**`docs/autonomy/QUEUE.md`:** task 17 of the installer plan is done; the next
free work in that plan is task 19 (*Killed at every step, Windows' own partition
byte for byte*) and task 20 (*A download that stops arriving*), both the
development PC's.

**`docs/autonomy/STATE.md`:** reference this report's path. Task 17 is marked
done in the plan by this change; tasks 18 and after already existed, so no new
task was written.
