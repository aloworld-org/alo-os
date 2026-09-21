# ADR 0053 — An update is carried out by a unit the broker starts, never by the broker

**Status:** **accepted, option B, by the owner on 2026-09-19** — and built by
task 8 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` on 2026-09-20,
which is the change that moved this line. Written by task 4 of the same plan
(*Updates and storage, through the broker*). The storage half of that task was
built then; the update half could not be, without deciding something a worker
may not decide: widening a privileged component's privilege, or adding a
privileged component. Until this line moved, the update verbs were answered
`not-carried`, held by `crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs`,
which was written to fail the day it stopped saying *proposed*. It did, and the
change that moved it replaced that test with the tests of what was built —
`crates/alo-brokerd/tests/the_updates_are_carried_by_a_unit.rs` and
`crates/alo-brokerd/tests/only_the_update_approved_is_carried_out.rs`. What
remains owed by the image lane is under *Consequences* below.
**Date:** 2026-09-17
**Proposed by:** the broker-and-the-disk workstream
**Context:** [ADR 0001](0001-the-capability-model.md) (§2: what needs privilege
sits behind a broker small enough to audit in an afternoon; §5: a change waits for
one approval); [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(the base and `bootc` are rented and never patched);
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md) (no
lane edits another lane's crates);
[ADR 0036](0036-the-image-is-signed-by-a-key-a-person-holds.md) (a build is staged
only under a signature policy); [ADR 0049](0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
(§3: what the door cannot carry is handed over as a file and asked for by its
digest); `docs/features.md` v0.5 *Updates that never interrupt* and *Atomic
updates with rollback*; `crates/alo-broker`, `crates/alo-brokerd`,
`crates/alo-keeping-up`, `crates/alo-updating`, `crates/alo-egress`.

## The question in one line

**How is an update a person approved — the build they were told is ready, or the
build before — carried out on a machine whose broker holds no capability, when the
base's own program will change nothing for a process without one?**

## What is true today, measured rather than remembered

Measured on 2026-09-17 in the pinned base image (`quay.io/fedora/fedora-bootc`,
local image `b035260f985f`), `bootc 1.15.1`, `rpm-ostree 2025.12`, `udisks2
2.10.91`:

1. **`bootc` refuses to change the machine without `CAP_SYS_ADMIN`.** Its binary
   carries *This command requires full root privileges (CAP_SYS_ADMIN)* and
   *Verified uid 0 with CAP_SYS_ADMIN*. The broker runs as root with **both
   capability lines of its unit empty**, and
   `crates/alo-brokerd/tests/the_unit_is_the_process.rs` holds it to that. So
   `alo_updating::apply` and `alo_updating::go_back`, run inside the broker, would
   fail on every machine. (A container is not a booted host, and `bootc` refuses
   one for that reason first, so the refusal itself was read from the program
   rather than provoked; a booted virtual machine is where it is confirmed.)
2. **The decision to stage an update cannot cross into another process.**
   `alo_keeping_up::Staging::of` takes a `Ready`, and a `Ready` is made only by
   `Standing::between` from an `Offered`, which is heard only during a check for an
   update that is on the indicator. That is deliberate: no answer about updates
   exists that nobody was shown being fetched. But it also means the broker, told
   thirty-two bytes, can never hold one, so it cannot decide *exactly what
   `alo-keeping-up` decided*. Going back has no such gap: `GoingBack::offered` is
   made from the base's status and the record, which a privileged process can read
   for itself.
3. **Staging a build is minutes of download.** The side that asks the broker waits
   180 seconds for an answer (`alo_broker::asking::WAITING_FOR_THE_ANSWER`), and
   the broker answers one request at a time. Staging a system image over an
   ordinary connection is longer than that. What the download is also matters: it
   is this machine reaching a registry with no agent behind it, and nothing puts it
   on the indicator today (`alo_egress::Errand` has *checking for an update* and no
   *fetching one*).
4. **`bootc` has a two-step road, but not for the instruction we use.** `bootc
   upgrade --download-only` downloads and stages a build that will not be applied,
   and `--from-downloaded` releases it. `bootc switch` — which `alo-keeping-up`
   decided, because it stages a build *by digest* — has neither.

## Why this is a decision and not a task

Every road that carries an update out with the code as it stands runs through one
of three things. Giving the broker `CAP_SYS_ADMIN` widens a privileged component.
A second process that holds it is a new privileged component. Changing
`alo-keeping-up`, so that a decided update can cross a process boundary, is
another lane's crate (ADR 0028), and the plan says this one reads it and never
edits it. None of the three is a worker's to choose.

## What holds whichever option is taken

- **The broker never restarts the machine.** No instruction it causes carries
  `--apply`. *Restart now and apply it* is the person's restart through the
  session's own power menu, after the change is carried. `THE_RULE` then holds by
  construction: nothing this road starts can cause a restart.
- **The build is named by what the person approved, and checked against the
  machine at that moment.** `updates.apply` is refused when the machine no longer
  runs the build the update was found against, or already has it waiting.
  `updates.roll-back` is refused when the base's status no longer matches what the
  offer said, including an update that has started waiting since and would be set
  aside.
- **Where updates come from is the machine's, not the request's.** The source is
  the repository the booted build came from, read from the base's status. It is
  never handed over.
- **Every refusal and every answer is written down before it is given**, as every
  broker answer already is.

## The options

### A. Give the broker `CAP_SYS_ADMIN`

The update carrier calls `alo-updating` directly, as the network carrier calls the
network manager.

- **For:** the least code. One unit line.
- **Against:** `CAP_SYS_ADMIN` is most of root. The door, the record, the proxy
  writer and every carrier would run holding it, so *holds no capability*, the
  sentence that makes an afternoon's audit possible, stops being true for all of
  them in order to be false for one. The network and the disk already show the
  broker needs no capability when a rented daemon does the privileged work.
  **Rejected.**

### B. One unit per update verb, started by the broker, holding what `bootc` needs

Two `Type=oneshot` system units in the image, for example
`alo-applying-an-update.service` and `alo-going-back.service`. Each has **a fixed
`ExecStart` with no arguments**, a small Rust program, and exactly the capabilities
`bootc` needs, measured on a booted machine. The broker carries a verb out by
asking systemd, over the system bus, to start the unit (`StartUnit`, mode `fail`).
It waits for the job's result and answers `carried` or `not-carried` from it.

- What the unit acts on is handed to it the way ADR 0049 §3 hands a proxy to the
  broker. **For applying**, the person's side writes `{from, to}` to
  `/run/alo-broker/wanted/update.json`. The approved identity is the digest of
  those bytes. The broker checks it as it checks a proxy, then writes the same
  bytes to a root-only folder the unit reads. **For going back**, nothing is
  handed over: the identity is the digest of the offer's `{from, to, sets_aside}`,
  and the unit decides `GoingBack::offered` again from the status and the record,
  and refuses unless it digests the same.
- The unit decides through `alo-keeping-up` and runs through `alo-updating`, so
  the argument list is still the one `Staging` or `Returning` wrote, and nothing is
  written twice.
- **For:** the broker keeps no capability. The privileged program has one job,
  fixed arguments and nothing to read but the file the broker checked. systemd
  gives it its own journal, its own limits and a result the broker can read. The
  long download happens in the unit, not at the door's answer.
- **Against:** a fourth privileged component to audit, though a closed and tiny
  one; the image carries two units; and it **needs a change to `alo-keeping-up` by
  its owner**: a way to decide a `Staging` from an approved `{from, to}` checked
  against the base's status now, standing in for a `Ready` across a process
  boundary. The check itself stays on the person's side, on the indicator.

### C. Ask `rpm-ostree`'s daemon, a rented service with a bus interface

The broker calls `org.projectatomic.rpmostree1` to rebase and roll back, as it calls
NetworkManager and udisks2. The daemon holds the privilege, and the broker stays as
it is.

- **For:** the same shape as the network and the disk, and no new component of
  ours. A transaction model with progress, and cancellation.
- **Against:** `alo-keeping-up` decided `bootc`'s instruction — a switch by digest
  under `--enforce-container-sigpolicy` — and the machine-keeps-itself plan
  measured that instruction in a virtual machine. A rebase through another tool
  takes a different road to the signature policy. It leaves two tools writing one
  deployment's origin, which `bootc`'s own documentation advises against on a
  `bootc` host. And it re-decides a settled question in another lane's plan.
  **Rejected** unless that plan's owner reopens it.

### D. No update verbs through the broker in v0.5

- **Against:** *atomic updates with rollback* and *updates that never interrupt*
  are v0.5 lines in `docs/features.md`, and a person could then update only from a
  root shell. That narrows a promise. **Rejected.**

## The recommendation, and what was accepted

**B**, recommended 2026-09-17 and accepted by the owner on 2026-09-19 without
amendment. It is the only option that keeps the broker holding nothing, keeps
`alo-keeping-up`'s instruction exactly, and puts the long, privileged, networked
part of an update where systemd can bound it and report on it. With it:

1. **The unit's program and its units live in `crates/alo-brokerd`**, as a second
   binary. Its source and its unit files are held by the same kind of test as the
   broker's unit: fixed `ExecStart`, the capability set named line by line, no
   `Restart=`, no `Environment=`.
2. **The broker's door waits for the unit**, and the side that asks waits longer
   for the two update verbs than for any other. The door answering nothing else
   during a download is accepted for v0.5: an update is a person's rare,
   deliberate act. A door that must stay open during one is a later decision.
3. **The download is on the indicator.** An additive `alo_egress::Errand` for
   fetching an update, shown by the person's side from the moment it asks the
   broker until the answer, is owed by `alo-egress`'s owner.
4. **Owed by `alo-keeping-up`'s owner**, before any of this is built: a
   constructor deciding a `Staging` from an approved `{from, to}` and the base's
   status now, whose refusals are `Staging::of`'s own.

## Consequences, and which of them have happened

- **Done, 2026-09-20 (task 8).** The task that builds it adds the units' own
  programs as this crate's second and third binaries, the two units, the
  broker's update carrier (start a unit, wait for its result), the handed-over
  update file and its contract beside `machine-proxy-file.md`
  (`machine-update-file.md`), and replaces `the_updates_wait_on_their_decision.rs`
  with tests of each refusal above. Two binaries rather than one, because *a
  fixed `ExecStart` with no arguments* and two units means two programs; the
  broker never spells a unit's name out of anything a request carried, since
  `alo_brokerd::TheUnit` is a closed enum of two.
- **Owed by the image lane.** It installs both units and the broker. It
  measures which capabilities `bootc switch` and `bootc rollback` need on a
  booted machine, and whether systemd lets root holding no capability start a
  unit. Both go into `docs/quirks.md` whichever way they answer. Until that
  measurement, each unit carries the list derived in task 8 — what the base's
  own program states it requires, plus what writing an ostree deployment
  touches — enumerated line by line in the unit with its reason, and held by
  `crates/alo-brokerd/tests/the_updates_are_carried_by_a_unit.rs`. It is wider
  than it will end up; what it is not is inherited.
- **Done.** `alo-keeping-up` and `alo-egress` each gained one additive item,
  from their owners: `Staging::approved` (the machine-keeps-itself plan's task
  9) and `Errand::FetchingAnUpdate`.

## What the code waited on

All three are closed: this decision, accepted 2026-09-19; the `alo-keeping-up`
constructor, landed 2026-09-19; the fetching errand, landed the same day.

**What is still not claimed.** No real machine has been updated through this
road. The units are held to their lines by tests and the programs' decisions
are held by tests against a stand-in base; a staged update and a return on a
booted machine are the virtual-machine acceptance the image lane owns.
