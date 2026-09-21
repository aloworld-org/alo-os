# Updates, through the broker, as ADR 0053 decides

**Date:** 2026-09-20
**Workstream:** v0.5 — the broker and the disk
(`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, task 8)
**Contributor:** this development PC's lane
**Status:** ready for integration.

## What this is

The broker's two update verbs — `updates.apply` and `updates.roll-back` — have
answered `not-carried` since task 4 found they could not be built. The base's
own program changes the machine only for a process holding `CAP_SYS_ADMIN`; the
broker holds none, and giving it one would make *holds no capability* false for
the door, the record, the proxy writer and every other carrier in order to be
false for one. [ADR
0053](../../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
set out the four roads and recommended option B, and the owner accepted it on
2026-09-19: **an update is carried out by a unit the broker starts, never by the
broker.**

This is that, built.

## A user-readable change description

A person can now apply a version of alo OS they were offered, or go back to the
one their machine ran before, through the assistant or through Settings — and
their machine does the privileged part in a small program of its own that can do
nothing else. Applying checks, at the moment it acts rather than at the moment
they approved, that the machine is still running the version they were told it
would change from, that the version they approved is not already waiting, and
that the two versions named are different; going back checks that the machine is
still the one they were shown, including that no update has started waiting since
which going back would quietly set aside. Any of those failing changes nothing at
all. **Neither can restart the machine**: what they prepare happens at the next
restart the person makes themselves, and the programs that do the work are not
even permitted to reboot. On a company network, applying an update goes through
the proxy the machine is set to use, and a machine whose proxy setting cannot be
read fetches nothing rather than quietly going around it.

## What changed

### The broker's side

- **`crates/alo-brokerd/src/units.rs` (new)** — the two units the broker may
  start, as a **closed enum of two** (`TheUnit`), the trait that starts one
  (`StartingUnits`), and systemd on the system bus. Three methods and no other:
  `StartUnit` (mode `fail`), `GetUnit`, and `Get` for the properties it reads.
  It **waits**: while the unit has a job or is activating it asks again, then
  reads the service's `Result` and `ExecMainStatus` and answers from those.
- **`crates/alo-brokerd/src/updates.rs` (new)** — the carrier. For
  `updates.apply` it reads what the person handed over, refuses unless the bytes
  digest to the identity approved *and* read back as exactly two whole builds,
  writes those same bytes where only root can reach them, and starts the unit.
  For `updates.roll-back` it writes the approved identity there and starts the
  other. Both handed-over files are taken away whichever way the act went.
- **`crates/alo-brokerd/src/approved.rs` (new)** — `AnUpdate {from, to}` and
  `GoingBackApproved {from, to, sets_aside}`: the bytes both sides of the door
  digest, written one way, refused when they are written any other way.
- **`crates/alo-brokerd/src/for_the_unit.rs` (new)** — `/run/alo-broker/approved`,
  `0700` root's: the folder the broker hands a checked update on in, and the
  reading a unit believes it through.
- **`crates/alo-brokerd/src/carrying.rs`** — the two update arms now carry;
  `Carriers` gained a fourth type parameter and an additive `with_updates`,
  and `Carriers::of` keeps its published three-argument signature.
- **`crates/alo-brokerd/src/starting.rs`** — `Places` gained `approved`, and the
  broker makes that folder before its door opens, root's alone.
- **`crates/alo-brokerd/src/main.rs`** — the process supplies systemd.
- **`crates/alo-brokerd/Cargo.toml`**, **`Cargo.lock`** — `alo-keeping-up` and
  `alo-updating` (read, never decided here), `serde_json`, and `zbus` on Linux
  at the version and with the features the keyring, the portals, the network
  manager and the disk service already use, so it adds no crate to the
  workspace. The lock file changes with them; nothing else in it moved.

### The units, and their programs

- **`crates/alo-brokerd/alo-applying-an-update.service`**,
  **`crates/alo-brokerd/alo-going-back.service`** (new) — `Type=oneshot`, root,
  `ExecStart` with **no arguments**, capabilities named **one per line with the
  reason beside each**, no `CAP_SYS_BOOT`, no `Restart=`, no `Environment=`, and
  no install section, so nothing but the broker starts either.
- **`crates/alo-brokerd/src/staging_an_update.rs` (new)** — what the first unit
  decides: read what was handed over, read the machine now, `Staging::approved`,
  tell the base exactly those arguments, and tell a signature refusal apart from
  a download that stopped.
- **`crates/alo-brokerd/src/returning.rs` (new)** — what the second decides:
  decide `GoingBack` again for itself and refuse unless it digests to the
  identity approved, then `alo_updating::go_back` at the next restart.
- **`crates/alo-brokerd/src/the_machine_now.rs` (new)** — the base's status read
  **once** for both the builds on the disk and the repository the booted build
  came from.
- **`crates/alo-brokerd/src/the_road_out.rs` (new)** — the machine's own proxy
  on the one road that leaves the machine.
- **`crates/alo-brokerd/src/bin/alo-applying-an-update.rs`**,
  **`crates/alo-brokerd/src/bin/alo-going-back.rs`** (new) — the two thin
  programs.

### The asking side

- **`crates/alo-broker/src/asking.rs`** — `WAITING_FOR_AN_UPDATE` (an hour) and
  `waiting_for(verb)`, so the two update verbs are waited for longer than any
  other (ADR 0053, recommendation 2). `WAITING_FOR_THE_ANSWER` is unchanged and
  still public; `ask` keeps its signature.

### Tests and documents

- **`crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs` — deleted**,
  and replaced by **`tests/the_updates_are_carried_by_a_unit.rs`** and
  **`tests/only_the_update_approved_is_carried_out.rs`**, in the same commit that
  moves ADR 0053 to *accepted*.
- **`docs/contracts/machine-update-file.md` (new)** — the contract, beside
  `machine-proxy-file.md` as ADR 0053 asked.
- **`docs/contracts/agent-verbs.md`** — the update verbs described as carried
  out, additively.
- **`docs/quirks.md`** — what `bootc` 1.15.1 says about capabilities, and what
  it does not say.
- **`docs/decisions/0053-…md`** — *proposed* becomes *accepted*, and its
  consequences say which have happened and which the image lane still owes.
- The plan: task 8 marked done and what the image lane inherits written out;
  task 7's dependency on it cleared. `v0-5-the-machine-keeps-itself-plan.md`'s
  paragraph naming task 8 as blocked cleared with a dated sentence, per *A stale
  blocker is invisible work*.

## Decisions I made, and why

### Two binaries, not one

ADR 0053 says "a second binary" in one sentence and "two units, each with a
fixed `ExecStart` with no arguments" in the next. Those cannot both be literal:
two units with no arguments are two different programs. I took *no arguments*
as the binding half, because it is the half that is a law-2 promise — a single
program told which job to do would have an argument on the `ExecStart` line, and
an argument on that line is the one place on this road where something could
eventually be made to vary. So there are two programs, in this crate, as the ADR
places them.

### The broker's crate now depends on `alo-updating`, and the guard that mattered was kept

The deleted test asserted that `alo-brokerd` does not depend on the crate that
runs the base. That assertion could not survive the ADR's own instruction to put
the units' programs in this crate. What replaced it is narrower and is the thing
that was actually being protected: `tests/the_updates_are_carried_by_a_unit.rs`
reads every file in `src/` and refuses any that names the base, or
`alo_updating`, or starts a program, unless it is one of the three files that
are the units' own decisions — and it separately refuses `src/main.rs`, the
broker's own process, naming `alo_updating` at all. Linking a library is not a
capability; the unit file is where *holds no capability* is true or is prose, and
that is read here too.

The alternative was a fourth crate. I did not take it: ADR 0053 §1 places the
programs in `crates/alo-brokerd` explicitly, and a new crate has registrations
in this repository (collected words, the crate list, the image manifest) that a
worker gets wrong in ways nobody can see from inside the crate.

### The capability set is derived, and says so

Task 8's constraint says this task hands the image lane "the measured capability
set". ADR 0053's own consequences say the image lane measures which capabilities
`bootc switch` and `bootc rollback` need **on a booted machine**. Both cannot be
satisfied here, because no alo OS machine exists to measure on.

What I could measure I did, in the pinned base: `CAP_SYS_ADMIN` is the only
capability named anywhere in `bootc` 1.15.1's binary, and both strings it appears
in are about the check the program makes, not about the work it does. A unit
carrying only that would run as a root that cannot chown, cannot write a file it
does not own and cannot set a security label — which is a unit that would fail on
the first machine it met.

So each unit names `CAP_SYS_ADMIN` plus what writing an ostree deployment
touches — thirteen, one per line, each with its reason — and the unit file, the
test, `docs/quirks.md` and this report all say plainly that the list is
**derived** and wider than it will end up. What the derivation buys today is
real and is the part worth having: the units hold an **enumerated** set rather
than root's whole set, and `CAP_SYS_BOOT` is not in it.

**Both units carry the same list.** Going back writes less — it fetches nothing
— and a narrower list for it would have been a second guess with *a machine that
will not roll back* on the other side of it. Narrowing is one measurement, and
it belongs to the lane that can make it.

### Going back hands nothing over

ADR 0053 §B already says this, and building it made the reason sharper than the
sentence. Going back has no argument the machine did not already have, so the
broker hands over only the identity the person approved, and the unit decides
`GoingBack` again from the machine in front of it. The refusal ADR 0053 names by
hand — *an update has started waiting since, and going back would set it aside*
— then needs no code of its own: that machine produces a different
`{from, to, sets_aside}`, which digests differently, and nothing is set. One
check covers every way the machine can have stopped being the one the person was
shown.

### A second folder, root's, between the broker and the unit

`/run/alo-broker/wanted` is `0770` in the person's group, which is what lets them
hand anything over at all. Between the broker digesting a file there and a
privileged program reading it, anything running as that person can write it
again. So the broker copies the bytes it checked into `/run/alo-broker/approved`,
`0700` root's, and the unit reads only from there — and the unit checks again
that what it opened is a plain file of root's that nobody else can read, which is
the lock that still holds if somebody ever widens the folder.

### The update road takes the machine's proxy; going back does not

`alo-updating`'s own documentation puts the machine's proxy on the staging road
and leaves the caller to supply one; `machine-proxy-file.md` records that wiring
it "remains owed". On this road the caller is a unit in this crate, so it is
supplied here, through the same one reader `alo-looking-once` and `alo-software`
use, for `Road::FetchingAnUpdate`. A machine with no proxy file goes straight
out; a machine whose file is there and unreadable **fetches nothing**, because
reading an unreadable rule as no rule is how a company's traffic goes around its
own proxy with nobody told.

Going back reads no proxy, needs no credential and reaches no network, and its
unit says so. That asymmetry is deliberate and is a test.

### The asking side waits an hour; the broker gives up at fifty-five minutes

ADR 0053 asks only that the asker wait longer for these two verbs. The two
numbers are ordered on purpose and held by a test: the broker stops waiting
first, so a unit that has hung is answered *not carried* at the door rather than
abandoned by a caller that gave up on a change still being made.

### A machine reference is cut, not repaired

The base reports the booted deployment's image as this machine last wrote it,
which is a repository **with a build on it**. `alo_keeping_up::Source` refuses to
hold one, deliberately. So the build is cut off at the two places a registry puts
one — the `@`, or the tag's `:` in the last segment, which is why a host with a
port is not mistaken for a tag — and what is left goes through that crate's own
check. Anything else the base reports is refused rather than tidied into
something that would parse.

## What is not claimed

**No machine has been updated through this road.** Nothing here ran `bootc`,
staged a deployment or rolled one back. The units are held to their lines by
tests that read them; the programs' decisions are held by tests against a
stand-in base that records what it was told. A staged update and a return on a
booted machine — and the capability measurement that narrows both units — are the
image lane's, which ADR 0053's consequences already assign to it.

**`docs/features.md`'s v0.5 update lines are not ticked**, and this report is not
evidence that they should be.

**Nothing has walked through the door from a turn.** Like the network and printer
verbs before it, `updates.apply` is declared, carried out from an approved
authority, and not yet offered by a turn; and nothing yet writes
`/run/alo-broker/wanted/update.json`. That writer is a surface's, and the
contract is what it builds against.

**The credential line is owed image-side.** `the_road_out::THE_UNIT` names
`alo-applying-an-update.service`, which under ADR 0059 is the third unit file
that must carry a `LoadCredentialEncrypted=` line for a proxy that asks who this
machine is. `alo-agentd.service` and `alo-looking-once.service` are the other
two and do not carry it either. I did not add it: a `LoadCredentialEncrypted=`
naming a credential store entry that does not exist is a systemd behaviour I
could not measure here, and getting it wrong would fail the unit on every machine
that has no proxy password. Until it lands, a machine on a proxy that asks for a
name refuses the fetch, which is the same honest refusal the other two roads give.

## Verification

Platform: Windows Server 2022 host, Ubuntu 24.04 under WSL2, run in the
serialized Linux tree the gates use (`/root/alo-trees/this-machine`, built into
`/root/alo-builds/this-machine`) after an `rsync --checksum --no-times` of this
checkout, which is how `tools/kernel-loop/src/gates.rs` runs them. No Cargo
process was running when this began.

The product workspace is `.` throughout.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-brokerd -p alo-broker --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-brokerd` | ok — 42 + 6 + 5 + 5 + 8 + 5 + 6 + 11 + 4 + 9 passed, 0 failed |
| `cargo test -p alo-broker` | ok — 27 + 3 + 2 + 3 + 3 + 3 + 4 + 3 passed, 0 failed |
| `cargo test -p alo-citing` | ok — 31 passed, 0 failed (run because `docs/decisions/` and `docs/contracts/` changed) |
| `cargo test -p alo-changing-network -p alo-changing-printers` | ok — 50 passed, 0 failed (run because they build `Carriers`) |
| `cargo test` in `tools/kernel-loop` | ok — 152 passed, 0 failed (run because two plans changed) |
| `cargo doc -p alo-brokerd -p alo-broker --no-deps` | clean, zero warnings |

Not run, deliberately: the full workspace suite, and the BPF target's gates.
The supervisor runs all nine after this.

**The measurement in `docs/quirks.md`** was made with
`podman run --rm b035260f985f sh -c 'grep -a -o -E "CAP_[A-Z_]+" /usr/bin/bootc | sort | uniq -c'`
against the pinned base already on this machine, which answered `CAP_SYS_ADMIN`
and nothing else.

### Evidence, one line per acceptance criterion

| Criterion in the plan | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| the update verbs carry out exactly what `alo-keeping-up` decided and nothing it did not | `.` | `alo-brokerd` | `lib` | `staging_an_update::tests::the_build_approved_is_staged_with_the_instruction_the_crate_decided` |
| …and for going back, the one instruction `Returning` wrote | `.` | `alo-brokerd` | `lib` | `returning::tests::going_back_that_was_approved_is_set_and_the_base_is_told_one_word` |
| the rule that an update never interrupts is intact: no instruction the broker causes carries `--apply` (applying) | `.` | `alo-brokerd` | `lib` | `staging_an_update::tests::nothing_the_unit_tells_the_base_restarts_the_machine` |
| …and going back | `.` | `alo-brokerd` | `lib` | `returning::tests::nothing_the_unit_tells_the_base_restarts_the_machine` |
| the broker still holds no capability | `.` | `alo-brokerd` | `the_updates_are_carried_by_a_unit` | `the_brokers_own_unit_still_holds_no_capability` |
| what runs the base is a unit with a fixed command line | `.` | `alo-brokerd` | `the_updates_are_carried_by_a_unit` | `each_unit_runs_one_program_with_no_arguments` |
| its capabilities named line by line and held by a test | `.` | `alo-brokerd` | `the_updates_are_carried_by_a_unit` | `each_unit_names_its_capabilities_one_per_line` |
| …and neither can restart the machine | `.` | `alo-brokerd` | `the_updates_are_carried_by_a_unit` | `neither_unit_can_restart_the_machine_or_hold_what_it_has_no_business_with` |
| ADR 0053's refusal: the machine no longer runs the build the update was found against | `.` | `alo-brokerd` | `lib` | `staging_an_update::tests::a_machine_that_moved_on_since_the_approval_is_refused_before_the_base_is_told` |
| ADR 0053's refusal: the build is already waiting | `.` | `alo-brokerd` | `lib` | `staging_an_update::tests::a_build_already_waiting_is_not_staged_again` |
| ADR 0053's refusal: going back when the status no longer matches the offer, including an update that started waiting since | `.` | `alo-brokerd` | `lib` | `returning::tests::a_machine_with_an_update_waiting_since_the_offer_is_refused` |
| the build is named by what the person approved, and checked at that moment — the carried case at the real door | `.` | `alo-brokerd` | `only_the_update_approved_is_carried_out` | `the_update_approved_is_handed_to_the_unit_that_stages_it` |
| every refusal is written down before it is given, and starts no unit | `.` | `alo-brokerd` | `only_the_update_approved_is_carried_out` | `an_update_that_is_not_the_one_approved_starts_no_unit` |
| one approval is one execution | `.` | `alo-brokerd` | `only_the_update_approved_is_carried_out` | `one_approval_starts_the_unit_once` |
| where updates come from is the machine's, not the request's | `.` | `alo-brokerd` | `lib` | `the_machine_now::tests::the_builds_and_the_repository_are_read_from_one_answer` |
| the handed-over update reaches the unit as the bytes that were approved, and nothing is left in `/run` | `.` | `alo-brokerd` | `only_the_update_approved_is_carried_out` | `going_back_hands_the_unit_the_identity_that_was_approved` |
| `the_updates_wait_on_their_decision.rs` is replaced by the tests of what was built | `.` | `alo-brokerd` | `the_updates_are_carried_by_a_unit` | `the_base_is_named_only_in_the_units_own_programs` |
| ADR 0053 recommendation 2: the asking side waits longer for the two update verbs | `.` | `alo-broker` | `lib` | `asking::tests::the_two_update_verbs_are_the_ones_the_asker_waits_longer_for` |

Each was run on its own with `-- --exact <name>`, and each reported
`test result: ok. 1 passed; 0 failed`.

### A refusal measured failing

A guard nothing has seen fail is prose. In the Linux tree only, and put back
immediately afterwards, `Updates::applied` was changed to skip the
`AnUpdate::read` check and hand the handed-over bytes on unread — the smallest
way to let something that is not a pair of builds reach a privileged unit:

- `bytes_that_are_not_a_pair_of_builds_start_no_unit_even_under_their_own_digest`
  — `FAILED`, the unit having been started with `switch --apply
  ghcr.io/somebody/else` waiting for it.

It passed again once the file was restored, and the published tree is the
restored one.

## Limitations

- Everything under *What is not claimed*, above.
- **The unit is waited for by polling, not by a signal.** `StartUnit` answers
  when the job is enqueued, and systemd's `JobRemoved` signal is the canonical
  way to hear the end of it. The broker answers one request at a time and has no
  event loop, so it asks the unit's `Job` and `ActiveState` every half second
  instead, bounded and with a timeout. A signal would be tidier; a poll is the
  shape that fits a process with no loop to put a signal in, and it cannot hang.
- **`bootc` reports no distinct exit code for a policy refusal**, so
  `alo_updating::refused_for_its_signature` reads what it said. That is
  `docs/quirks.md`'s existing entry and this road inherits it.

## Proposed shared-document updates

For the integration owner; I have not edited any of the four.

**`CHANGELOG.md`**, under the unreleased heading:

> A version of alo OS a person approves can now actually be applied, and a
> machine can be put back to the version it ran before. The privileged part is
> done by a small program of its own that can do nothing else — the part of alo
> OS an assistant reaches still holds no special powers at all — and neither
> program is permitted to restart the machine, so what they prepare happens at
> the next restart the person makes themselves. Both check the machine as it is
> at the moment they act rather than as it was when the person approved, and
> refuse without changing anything when it has moved on, when the version
> approved is already waiting, or when going back would quietly undo an update
> the person had chosen. Applying an update goes through the proxy the machine
> is set to use; a machine whose proxy setting cannot be read fetches nothing
> rather than going around it.

**`ROADMAP.md`:** no change. *System verbs through the privileged broker* is not
complete until the image carries the broker and these units, and *atomic updates
with rollback* is not complete until a machine has done one.

**`docs/autonomy/QUEUE.md`:** the broker plan's task 8 is done; task 7 is ready
with nothing outstanding behind it; task 9 stays blocked on a certified machine.
The image lane inherits two units, two programs and the capability measurement
named under task 8's *What was handed to the image lane*.

**`docs/autonomy/STATE.md`:** reference this report.
