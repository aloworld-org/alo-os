# Every sentence, and the walk from a new printer to a recovered disk

**Date:** 2026-09-20.
**Workstream:** `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, task 7 —
★ *System verbs through the privileged broker* and *Full-disk encryption*.
**Machine:** `AGAI01`, the third PC's lane.
**Status:** ready for integration. Code and vocabulary evidence, run on this
machine; **nothing here is a claim about a printer, a network, a drive, a
machine that has been updated, or a disk on certified hardware.** What each of
those is owed is said under *What is not claimed* below.

## What this task is

Six tasks before this one each ended in sentences a person reads, and each crate
holds its own sentences one at a time. This task asks the two questions none of
them can:

1. **Is every sentence this workstream can say in the machine's one vocabulary,
   with a note a translator can work from — and does any of them name LUKS, a
   TPM, CUPS, NetworkManager, a socket or *root*?**
2. **What does a person actually meet, in order, walking from a printer they
   have just been given to a disk that will not open?**

The first is an audit. The second is a walk, and walking it found the thing an
audit could not: **two of the four verb families had nobody to say anything at
all.**

## What was found, and what was built because of it

The broker has eleven verbs across four families. Two of them — the printers and
the network — had a surface that turns the door's one-word answer into a
sentence a person reads: `alo-changing-printers` and `alo-changing-network`.
**Storage and updates had none.** `alo_brokerd::Storage` mounts and ejects a
drive and `alo-brokerd`'s two units apply an update and go back, and both of
them answer *carried*, *not kept* or *refused* into a process with no person in
front of it. A drive could be opened and finished with, and a person would read
nothing either way; an update could be refused at the door, and nothing anywhere
would tell them their machine was unchanged.

That is not a gap a vocabulary audit finds, because there is no sentence to
audit. It is what a walk finds, which is why the walk is the acceptance.

So this task built the two missing surfaces, as siblings of the two that exist:

- **`crates/alo-changing-drives`** — thirteen sentences, the choosing that goes
  before them, and the map from the broker's answer to what a person reads.
  A person opens a drive and finishes with it; the sentence they wait for before
  pulling it out is *"{drive} is finished with, and safe to unplug"*, and the one
  that must never be said carelessly is *"This machine could not finish what was
  asked of {drive}. Do not unplug it yet"*.
- **`crates/alo-changing-updates`** — **four** sentences, and the smallness is
  the point. Everything a person reads *about an update* was already written
  where updates are decided (`alo_keeping_up::words`), and this crate **says**
  those rather than writing them again. What was missing is the three things
  only the door can answer: an approval it would not take, a machine that has
  stopped writing changes down, and nothing there to make them.

Both are collected into `alo-saying` by the same road every other crate's words
take, and both are held by this task's two tests.

## Decisions I made, and why

Nobody was waiting to answer these, so they were decided the way the rest of the
workstream is built.

**Two crates rather than one, or none.** The alternative to a crate was putting
person-facing English in `alo-brokerd`, which runs as root and answers in one
word; a daemon with a vocabulary is a daemon deciding what a person reads, and
`alo-broker`'s dependency list is held by a test precisely so that a privileged
component stays auditable in an afternoon. The alternative to *two* crates was
one holding both, and drives and updates are not one subject: a person finishing
with a memory stick and a person whose machine will not update have nothing in
common but the door they went through. The two that already exist set the shape,
and these are the third and fourth of four.

**`alo-changing-drives`, not `alo-changing-storage`.** The verb family is called
`storage.*`, so the pattern-consistent name was the second one. It reads worse:
the crate is about drives a person plugs in, and it reads `alo-drives`, which is
the thing a person has. Crate names are not person-facing, but they are the first
thing a reader meets, and *storage* is a word about the machine.

**`alo-changing-updates` borrows rather than writes.** It could have had its own
*the update will apply the next time you restart*. A second copy of a sentence is
a second thing to translate and a second thing to get out of step, and a person
told *it could not be prepared* should read the same line whether the preparing
was refused before the door or behind it. So `NotChanged::CouldNotPrepare` says
`alo_keeping_up::words::NOT_PREPARED`, and carries which of the two changes it
was so that going back is not reported as an update.

**No new agent verb, and no second road to the door.** Task 4's constraint says
*USB drives that appear when plugged in* is the desktop's and mounting is the
broker's; task 8's says what the base may be told is `alo_keeping_up::Staging`
and `Returning` and nowhere else. Neither new crate declares a verb, opens a
socket, or assembles an instruction. They hold what a person reads and the
choosing that comes before it, and a surface hands its redeemed approval to the
road that already exists. Neither is registered in `alo-declared`, because
neither declares anything to register.

**The walk is the sentences, not the wire.** Each verb's road from an approval
to the broker's real door and out to the rented service is already held by the
test the task that built it wrote — `alo-changing-printers`' and
`alo-changing-network`'s end-to-end tests against a real door, and
`alo-brokerd`'s `only_the_drive_approved_is_mounted_or_ejected.rs` and
`only_the_update_approved_is_carried_out.rs`. Walking them again here would test
the wire twice and the sequence once. Every sentence in the walk comes out of the
machine's one assembled vocabulary through the value that really produces it — an
`alo_capability::Call`'s own sentence, a `changed_said`, a `NotChanged`, an
`alo_encrypting::TheDiskRefused` — and the walk asserts along the way that the
approval is spent once, that opening and finishing with a drive are different
identities, and that the recovery key's run is the one that opens the volume.

**Where the two tests live, and a guarantee that turned out to be stricter than
it looks.** They are the workstream's rather than any one crate's, so they were
written in `alo-brokerd` — the one crate that stands where all four verb
families meet. `cargo test -p alo-brokerd` then failed on a test written by task
4: `the_broker_can_make_no_grant`, which reads `alo-brokerd`'s **manifest as
text** and refuses the name `alo-capability` wherever it appears. The walk needs
`alo-capability`, because the sentence a person approves is an
`alo_capability::Call`'s own — and a dev-dependency is not linked into the
shipped daemon, so the check could fairly have been read as not applying.

It was not weakened. *The process that mounts a drive can make no grant* is a
promise about a file somebody audits, not about a link graph they would have to
compute, and a manifest with `alo-capability` in it is a manifest that makes an
auditor stop and work out which section it was in. So the two tests moved to
`alo-changing-drives` — which is, fittingly, the crate the walk found missing —
and `alo-brokerd`'s manifest now carries a comment where the dev-dependencies
would have gone, saying why they may not.

## The walk, sentence by sentence

One person's week, in the order they meet it. The sentences are what
`alo_saying::everything_this_machine_can_say()` really says, in English —
somebody reading a language nobody has written yet reads exactly this, which is
`alo-strings`' promise and the reason a missing line is survivable.

| Step | Moment | What a person reads |
|---|---|---|
| 1 | The agent proposes setting up a printer this machine found | set up the printer Brother HL-L2350DW series, so this machine can print on it |
| 2 | The person approves, and it is set up | Brother HL-L2350DW series is set up, and this machine prints on it |
| 3 | The agent proposes joining a network, and says what it does to this conversation | join the Wi-Fi network Café Central, and this conversation loses its connection until this machine is connected again |
| 4 | The person approves, and the machine joins | This machine is connected to Café Central |
| 5 | A drive is plugged in, and opened | Kingston-DataTraveler-1C1B is ready, and what is on it is yours to open |
| 6 | The person finishes with it | Kingston-DataTraveler-1C1B is finished with, and safe to unplug |
| 7 | They pull it out, and ask for it again | Nothing called Kingston-DataTraveler-1C1B is plugged into this machine, so nothing was opened. Plug it in again, then try once more |
| 8 | An update they approved is applied | The update will apply the next time you restart. Your files and settings stay as they are |
| 9 | The same approval is offered a second time | Nothing was changed, because the approval for it was not accepted: it was used already, came too late, or was not given for this. Ask again, and approve it when you are asked |
| 10 | The machine starts, and what they type does not open its disk | that does not open this machine — try again, or use the key you wrote down |
| 11 | The key they wrote down, typed with one character wrong | that does not open this machine — try again, or use the key you wrote down |

And then the key is typed correctly and the machine starts, **saying nothing** —
which is what a machine that opens does, and why step 10's sentence has to name
the key in the same breath as the refusal. There is no twelfth row because there
is no twelfth sentence, and inventing one would have been the easiest dishonest
thing in this task.

`crates/alo-changing-drives/tests/the_walk_from_a_new_printer_to_a_recovered_disk.rs`
parses this table out of this file rather than holding a copy of it, so a
sentence that changes without the table changing fails, and so does a table
edited to say something the machine does not. A later change that moves a
sentence publishes the table again in a follow-up report and points the test's
`THE_REPORT` at it; a published report is never rewritten.

### What the walk shows that no single crate could

- **Steps 1–4 read as one account.** A proposal, an approval, a change, a
  proposal, an approval, a change — and step 3 tells the person, in the sentence
  they are approving, that the conversation they are having will lose its
  connection, because `alo_changing_network::would` says it will.
- **Steps 10 and 11 are the same sentence, deliberately.** The machine does not
  say which secret was wrong, because which secret was wrong is a fact about
  somebody's own key. A person who mistyped their PIN and a person who mistyped
  their recovery key read one line, and it names the way out both times.
- **Step 9 is *one approval, one execution* as a person meets it.** The door has
  already spent it, and what they read says their machine is unchanged and what
  to do next.

## What is not claimed

- **No printer, no network, no drive and no updated machine.** Every sentence
  here was produced by the real value that produces it, from the real assembled
  vocabulary. None of it touched a printing service, a network manager, a disk
  service or a deployment, and none of it is a claim that a Brother HL-L2350DW
  was set up on anything.
- **Nothing about a chip, and nothing about a certified machine.** Step 10's
  refusal is `alo_encrypting::TheDiskRefused::WhatWasGivenDoesNotOpenIt`, which
  task 6 ran against a real LUKS2 volume under `podman`. The three promises that
  need a chip are task 9's and `docs/hardware.md` still lists no certified
  machine. `docs/features.md`'s v0.5 encryption line stays unticked.
- **Neither new crate has been reached from a turn**, exactly as
  `alo-changing-printers` and `alo-changing-network` have not. The surface that
  will ask for a drive is the desktop plan's; the daemon's wiring is owed where
  tasks 2, 3 and 8 already say.

## A finding for the desktop lane, recorded rather than fixed

A person reads the drive's name as `alo_drives::Drive::identifier()` — the
identifier the disk service keeps for it across plugging in and out, which for a
USB stick is its maker and model run together, `Kingston-DataTraveler-1C1B`. It
is the drive's own name and not machinery, and it is stable, which is what a
sentence naming a thing needs. It is also not quite what a person would write,
and `alo-drives` keeps nothing prettier: giving a drive a shown name is a change
to task 4's crate and to what the disk service is asked for, which is the
desktop lane's when it builds *USB drives that appear when plugged in*. Recorded
here rather than taken, because widening this task into `alo-drives` would have
put two lanes in one crate for a hyphen.

## A note improved in passing

`changing-network.wireless-on` and `changing-network.wireless-off` each carried a
note of six words — *"Said once Wi-Fi has been turned on."* Every other sentence
in the workstream tells a translator what the sentence is for, what must survive
translation and what it must not read as; these told them when it is shown. Both
were rewritten to the same standard, which is what *every sentence with a
translator's note* means when the check is a test rather than a box. Nothing
either sentence says changed.

## What changed

| Path | What it is |
|---|---|
| `crates/alo-changing-drives/` (new) | What a person reads when a drive they plugged in is opened or finished with: `words.rs`, `wanted.rs`, `choosing.rs`, `refusing.rs`, `lib.rs` |
| `crates/alo-changing-updates/` (new) | The three things only the door can answer about an update, and the map from the door's word to `alo-keeping-up`'s sentences |
| `crates/alo-changing-drives/tests/every_sentence_this_workstream_says.rs` (new) | The audit: every sentence collected, noted, and naming nothing of the machine's — and the four crates that deliberately say nothing, each with its reason |
| `crates/alo-changing-drives/tests/the_walk_from_a_new_printer_to_a_recovered_disk.rs` (new) | The walk, held to the table above |
| `crates/alo-changing-drives/Cargo.toml` | Dev-dependencies for those two tests |
| `crates/alo-brokerd/Cargo.toml` | A comment where the dev-dependencies would have gone, recording why they may not |
| `crates/alo-saying/Cargo.toml`, `crates/alo-saying/src/collecting.rs` | The two new crates collected into the one vocabulary |
| `crates/alo-changing-network/src/words.rs` | Two translator's notes rewritten; no sentence changed |
| `Cargo.toml`, `Cargo.lock` | The two new workspace members |
| `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` | Task 7 marked done; the header names the two crates it added |
| `tools/kernel-loop/src/who_owns.rs` | The plan's owned-crate list is read from that header, and the test that reads it |
| `docs/autonomy/v0-5-the-installer-plan.md` | Two lines putting `main`'s own plan gate back; see below |

### A user-readable change description

> alo OS now says something when you plug a drive in and when you finish with
> it, and when an update cannot be applied. Every sentence the machine says
> about its printers, its network, its drives, its updates and its disk has been
> checked against the one list it translates from: each has a note for whoever
> translates it, and none of them names the machinery underneath. The exact
> sequence a person meets — from the moment an assistant offers to set up a
> printer to the moment a disk asks for the key they wrote down — is written
> down and held by a test, so it cannot change by accident.

## Verification

Run on `AGAI01`: Windows checkout at `C:\dev\alo-os-2`, gated through WSL 2
Ubuntu against the serialized source copy at `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, which is the one build cache
this machine shares. `ring`, pulled in by `alo-broker` and so by everything in
this workstream, cannot be built on this Windows host — there is no `gcc.exe` —
and `cargo fmt --all` on Windows fails with *the filename or extension is too
long*, because the workspace has more members than a Windows command line holds.
Both are the machine rather than the work.

Every line below was run in the foreground and its exit code read.

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` then `cargo fmt --all --check`, in both workspaces | exit 0, clean |
| Clippy, warnings denied | `cargo clippy -p alo-changing-drives -p alo-changing-updates -p alo-brokerd -p alo-changing-network -p alo-changing-printers -p alo-saying -p alo-collected --all-targets -- -D warnings` | exit 0, no warning |
| Clippy, warnings denied | `cargo clippy --all-targets -- -D warnings` in `tools/kernel-loop` | exit 0, no warning |
| Tests | `cargo test -p alo-changing-drives -p alo-changing-updates` | exit 0 — 18 + 11 = **29 passed, 0 failed**, plus this task's 7 + 4 integration tests |
| Tests | `cargo test -p alo-brokerd` | exit 0 — 42 lib + 59 integration = **101 passed, 0 failed** |
| Tests | `cargo test -p alo-changing-network -p alo-changing-printers` | exit 0 — **50 passed, 0 failed** |
| Tests | `cargo test -p alo-saying -p alo-collected` | exit 0 — **87 passed, 0 failed** |
| Tests | `cargo test -p alo-citing -p alo-conforming -p alo-reconciling` | exit 0 — **75 passed, 0 failed** (the citation and plan checks a `docs/` change reaches) |
| Tests | `cargo test` in `tools/kernel-loop` | exit 0 — **152 passed, 0 failed** |
| Rustdoc, warnings denied | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-changing-drives -p alo-changing-updates --no-deps` | exit 0, no warning |

The exact counts are in the handoff's evidence block, one line per acceptance
criterion, each run on its own.

**The full workspace suite was not run here**, deliberately: it takes the better
part of an hour on this machine and the supervisor runs it on the committed tree
regardless. The crates touched and the crates that read them were run.

## Acceptance, against the plan

| The plan asks | Where it is |
|---|---|
| every sentence these crates can say is in the vocabulary with a translator's note | `alo-changing-drives`, `every_sentence_this_workstream_says`, `every_sentence_this_workstream_says_is_one_the_machine_says` and `every_sentence_carries_a_note_a_translator_can_work_from` |
| one walk … produces the exact sequence a person meets, recorded as a table and held by one test | `alo-changing-drives`, `the_walk_from_a_new_printer_to_a_recovered_disk`, `the_walk_from_a_new_printer_to_a_recovered_disk_reads_as_the_table` |
| the encryption sentences join the table — `alo-enrolling`'s `EVERY_WORD`, twenty-four of them | the same audit reads them beside the other four crates', and the walk's last two steps are two of them |
| no sentence names LUKS, TPM, CUPS, NetworkManager, a socket or *root* | `no_sentence_or_note_names_the_machinery`, over sentences **and** notes, and `no_sentence_on_the_walk_names_the_machinery` over the sequence |
| *nothing here re-decides what the sentences describe* (the constraint) | no verb, no argument, no road and no ADR was changed; the two new crates carry no decision of their own, and the one sentence pair that changed changed only its note |

## `main` was already red in the supervisor's own gate, and two lines fixed it

My change touches `tools/kernel-loop/`, so its three gates are in scope, and
`cargo test` there failed on **two faults already on `main`** — neither in
anything this task wrote, both in
`docs/autonomy/v0-5-the-installer-plan.md`:

1. `### 7. Replace Windows — the road with no way back` had been **deleted by
   accident** in `b38926f` (#84): a paragraph about a machine with no discrete
   graphics was inserted where the heading was, so the plan ran 1–6, 8–15 and
   `every_plan_this_repository_drives_holds_only_tasks` refused it. Task 7's
   body was still there; only its heading was gone. Put back, unchanged.
2. Task 11's status read `**done, 2026-09-21**` — lower case. `THE_DONE_MARK` is
   `**Done,`, so the plan reader saw a status saying done with no done mark,
   which is precisely the *a finished task the loop would take up again* case
   the check exists for. Written as `**Status:** **Done, 2026-09-21.**`, which
   is the spelling the check names in its own refusal.

The second was invisible until the first was fixed, because the numbering
assertion fired first. Both are one line, neither changes a word of anybody's
task, and both are named here rather than fixed quietly: they are another lane's
plan, and *a merge is finished when `main` still gates*
(`docs/autonomy/SHARED_MAIN.md`) puts them with the machine that merged them. I
could not gate my own work without them.

## Remaining limitations

- The two new surfaces are not reached from a turn, and the desktop surface that
  will ask for a drive does not exist yet.
- A drive is named to a person by the disk service's identifier; see the finding
  above.
- Nothing here has met a real printer, network, drive, update or certified
  machine.

## Proposed updates to the shared documents

For the integration owner; this report does not edit them.

**`CHANGELOG.md`**, under Unreleased:

> - alo OS says something when a drive you plugged in is opened and when it is
>   safe to unplug, and when an update or a return to the previous version
>   cannot be carried out. Every sentence the machine says about its printers,
>   network, drives, updates and disk is in one list with a note for whoever
>   translates it, and the exact sequence a person meets from a new printer to a
>   disk asking for the key they wrote down is written down and held by a test.

**`docs/autonomy/QUEUE.md`**: the broker plan's task 7 is done; tasks 8 and 6
were already done, and task 9 remains blocked on a certified machine existing.

**`ROADMAP.md`**: no v0.5 line ticks here. ★ *System verbs through the privileged
broker* now has a surface for all four of its verb families, which is a step
towards that line rather than the line itself — the machine evidence it needs is
still owed.

**`docs/autonomy/STATE.md`**: reference this report.
