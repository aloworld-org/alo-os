# ADR 0045 — What undoing rewinds to

**Status:** accepted, 2026-09-16 — option **A**, with the six terms under *As the
owner accepted it*, which are part of the decision rather than commentary on it.
**Amended 2026-09-21** with a seventh term naming what removes an expired
snapshot: the first `btrfs` install measured that removing one needs
`CAP_SYS_ADMIN`, and nothing in this repository removed one at all.
Written by task 4 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (*Undo what the agent
did*), which cannot be built until it is answered. Nothing is built in the
change that adds this: the record has no entry for an undo, `alo-keeping-up`
has no answer about undoing, and
`crates/alo-keeping-up/tests/undoing_is_decided_before_it_is_built.rs` fails
the day either appears while this line still says *proposed*.
**Date:** 2026-09-15
**Proposed by:** the machine-keeps-itself workstream
**Context:** [ADR 0001](0001-the-capability-model.md) §5 and §7 (a change waits
for one approval; everything executed is a record),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (*undo what
the agent did this afternoon* — **atomic snapshots and rollback, the base's**),
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md) (*a snapshot at turn
start* is one of the four primitives, and *undo becomes exact*),
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md) (undo *exact, from a
snapshot taken when the turn began*, against *best-effort, reconstructed from
the record*), [ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(no lane edits another lane's crates), `docs/features.md` v0.5 ★ *Undo what the
agent did*, `docs/contracts/record-file.md` (additive only),
`crates/alo-installing`, `crates/alo-turn`, `crates/alo-files`,
`crates/alo-record`, `crates/alo-keeping-up`

## The question in one line

**When a person asks to undo what an agent did to their files, what does the
machine put back from?** The plan's task says *the base's own snapshot of that
subvolume is the road*. On the machine this repository installs there is no
subvolume and no snapshot, so the road has to be chosen before anybody builds
on it, and every choice is someone else's to make.

## What is true today, verified rather than remembered

Measured and read on 2026-09-15, on the tree this was written in.

- **The shipped machine's disk cannot take a snapshot.**
  `crates/alo-installing/src/writing.rs` installs with
  `bootc install to-disk --filesystem ext4`, and `docs/booting.md` builds the
  development disk the same way. ext4 has no subvolumes, no snapshots and no
  reflinked copies. The pinned base's `/usr/lib/bootc/install/` is empty, so
  the filesystem is whatever the installer names. Nothing else on the machine
  keeps an earlier state of a person's files.
- **The base *can* do it, unmodified.** In the pinned base image, measured with
  `podman run` in this lane's WSL box: `bootc 1.15.1`, whose `install to-disk
  --filesystem` accepts `xfs`, `ext4` and `btrfs`, and `btrfs-progs 6.19.1`.
  No `snapper`. So a snapshot is a configuration away and not a patch away.
- **What the base's rollback covers is not a person's files.** Task 3 of the
  plan measured it (`docs/autonomy/updates/back-to-yesterdays-machine.md`):
  going back to the build before swaps `/usr`, keeps `/var` as it is, and
  leaves `/etc` with the newer build. A person's home is under `/var/home`. The
  image rolling back does nothing for a file an agent renamed.
- **The kernel-enforcement plan says so too.** Its audit lists *a snapshot at
  turn start, and exact undo* as **not built**, v0.5 / v1.
- **What an agent can change today.** Nine change verbs exist:
  `rename_file`, `move_file` and `archive_folder` (`crates/alo-files`), which
  change names and create a file and never rewrite what is in one;
  `open_application`, `focus_application`, `close_application` and
  `arrange_application` (`crates/alo-applications`); `print_document`
  (`crates/alo-printing`); and `install_application` (`crates/alo-software`).
  Questions asked of a model are `answered-here` or `left` entries. Only the
  first three change anything a snapshot could hold.
- **An entry has no identity but its moment.** `alo_record::Entry` is a moment
  and what happened. The record file is appended to and pruned
  (`crates/alo-keeping/src/pruning.rs`), so a position in the file is not a
  name that lasts.
- **The plan's own constraint.** *No inverse is invented*: what can be undone is
  undone *through the mechanism that made it undoable, and never by a second
  implementation guessing at inverses*.

## Why this is a decision and not a task

Each road runs through something a worker may not choose alone:

- a snapshot needs a filesystem that has one, and the filesystem is chosen in
  `crates/alo-installing`, the installer plan's (ADR 0028). The disk plan
  (`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`) says in its opening
  lines that *nothing has decided the disk's* shape;
- taking the snapshot at the moment a turn begins is `crates/alo-turn`, lane
  A's, and making it needs a privilege `alo-agentd` does not hold, which is the
  broker plan's to grant or refuse;
- our own copy of the files instead contradicts ADR 0011's table, which puts
  undo on **the base**, and the plan's *never a second implementation*;
- rebuilding from the record is the inverse the plan forbids and the thing
  ADR 0015 calls *best-effort*;
- and shipping nothing that can be undone narrows the ★ line in
  `docs/features.md`, which only the owner may do in the open.

## What holds whichever option is taken

These are decided here, because none of them depends on the road, so the
worker who builds after the answer does not reopen them.

1. **What can never be undone, and why, in words a person reads.**

   | What the record says | Can it be undone | Why, as the person is told |
   |---|---|---|
   | `left` — a question put to a model elsewhere, anything sent | never | it left this machine |
   | `answered-here`, `answered-for-another-machine` | never | it was told to a model |
   | `ran` with a read (`list_folder`, `read_file`, …) | nothing to undo | it only looked, and nothing changed |
   | `ran` `print_document` | never | it was printed on paper |
   | `ran` `open_…`, `focus_…`, `arrange_…`, `close_application` | never | an application is not something the machine can put back, and a closed one may have let go of work |
   | `ran` `install_application` | never here | removing an application is the person's own act (ADR 0042), not an undo |
   | `ran` `rename_file`, `move_file`, `archive_folder` | **only through the road chosen below**, and only while what it left is still as it left it | — |
   | `stopped`, `turned-away`, `never-put-anywhere`, `held-back`, `not-bounded` | nothing to undo | nothing happened |
   | `updated`, `rolled-back`, `paired`, `workspace-opened`, `grants-not-read-again`, `left-on-its-own` | not an agent's | going back to the build before is its own act; undo is for what an agent did |

   A verb added later answers *never* until the change that adds it says
   otherwise, so a new verb cannot become undoable by being forgotten.
2. **An undo is the person's.** It is proposed from the record the person reads,
   with a sentence naming what will be put back and the moment it was changed,
   and it waits for one approval. There is **no agent verb** that undoes, and
   none that proposes an undo: an agent choosing which of a person's approvals
   to reverse is the thing the plan's constraint forbids. Adding one later is
   additive and needs its own decision.
3. **An undo never overwrites a later change.** When anything it would put back
   has changed since the verb left it — the person edited it, a later turn moved
   it — the undo is refused before it is offered, and the sentence says so.
4. **The record keeps the undo, naming what it undid by copy.** A new additive
   kind, `undone`, carrying the moment of the entry undone and its `What` — verb,
   sentence and arguments — copied in, not pointed at, so the record stays a
   complete account after the original is pruned. No agent field: the person
   did it. `format` stays `1`. An undo that failed at the moment it ran is
   recorded as failed, with the same copy.
5. **What an undo may keep is visible and forgettable.** Whatever the road keeps
   holds the person's files as they were, including files the person later
   deleted. How long that lasts is shown where the person can find it, and
   forgetting it is one act. On a managed machine the organisation may name
   how long; the machine ships the mechanism and a stated default, not a
   default that decides for them.

## The options

### Option A — the base's snapshot: btrfs, a subvolume per home, a snapshot either side of a changing turn *(recommended)*

The installer names `btrfs` instead of `ext4`; each person's home is its own
subvolume; a turn that runs a change verb is bracketed by two read-only
snapshots of that subvolume, one before the verb and one after, kept outside
every grant under `/var/lib/alo`. Undoing compares the two with the base's own
tools to learn exactly what the verb changed, refuses if the home no longer
matches the *after* snapshot on those paths, and puts them back from the
*before* snapshot by reflinked copy.

- **What it costs a person:** nothing they see per turn (a snapshot is
  constant-time and shares every block until something changes). What they
  do see is space: a file deleted after a turn is still held until the snapshot
  goes, which is what point 5 above exists for.
- **What it costs the disk:** the root filesystem changes for every new install,
  on the certified machine as well, and a machine already installed on ext4
  stays without undo until it is reinstalled — there is no conversion.
  `what_is_filling` has to count snapshots, or it lies.
- **What it costs another lane:** the installer plan's `crates/alo-installing`
  and `docs/booting.md` (one argument, and the virtual-machine tests measured
  again); the accounts lane (a home made as a subvolume); lane A's
  `crates/alo-turn` (the bracket); the broker plan (who holds the privilege to
  snapshot, as one fixed verb); the disk plan (btrfs under its encryption).
- **Why it is the recommendation:** it is the road ADR 0011, ADR 0013 and
  ADR 0015 already describe, done by the base, with nothing of ours guessing.
  Two snapshots make it exact about what one verb changed rather than what its
  arguments named, which also covers a folder archived whole.
- **Measured first, before building:** whether `bootc install to-disk
  --filesystem btrfs` makes any subvolume of its own; which capability creating
  a snapshot needs on the pinned kernel; and that a bootc update and a return
  to the build before leave the home subvolume and `/var/lib/alo` alone.

### Option B — our own copy of what a verb names, kept before it runs

The installer keeps `ext4`. Before a change verb runs, the turn copies every
path it names into a store outside every grant, and undo copies them back.

- **What it costs a person:** a turn that archives or, later, rewrites a large
  folder waits for a full copy first, and their disk holds it twice.
- **What it costs the disk:** double the space of everything an agent touched,
  for as long as undo is kept. On `xfs` the copies could be reflinked and cost
  nothing up front, which is a filesystem change again.
- **What it costs another lane:** lane A's `crates/alo-turn` (the copy, inside
  the privileged path), `crates/alo-files` (what each verb names), and an
  amendment to ADR 0011's table and the plan's constraint, both of which say
  undo is not ours to implement.
- **Against it:** a copy knows only the paths the arguments named, so it is
  exact for today's three verbs and quietly incomplete for the first verb that
  touches what it does not name. It is the second implementation the plan
  forbids, in the most privileged code we have.

### Option C — put things back from what the record says happened

No snapshot and no copy: `move_file` is undone by moving back, `rename_file`
by renaming back, `archive_folder` by removing the archive.

- **What it costs a person:** an undo that is wrong in the cases that matter —
  a name taken since, a file edited since, an archive the person already
  sent — and a machine that implied it could put something back.
- **What it costs the disk:** nothing.
- **What it costs another lane:** nothing, which is its only merit.
- **Against it:** it is the inverse the plan forbids by name, and the
  *best-effort, reconstructed from the record* ADR 0015 was written to get past.
  Rejected, and written down so that nobody builds it for being cheap.

### Option D — nothing can be undone yet, and the machine says so

Build points 1 to 5 above now, with every change verb answering *never, because
nothing kept what was there before*, and leave the road to v1.

- **What it costs a person:** the ★ promise — *undo everything the agent did
  this afternoon* — is not kept at v0.5, and they are told so honestly.
- **What it costs the disk:** nothing.
- **What it costs another lane:** nothing now; the same choice as A or B later.
- **Against it:** it narrows a starred line of `docs/features.md`, which is the
  owner's to do, in the open, with the reason beside it.

## The recommendation

**Option A**, with points 1 to 5 built by this lane as soon as it is accepted:
they hold under A, and the change verbs answer *not yet on this machine* until
the bracket exists on it, which is true rather than a stub. The installer's
argument, the home subvolume, the bracket and the broker's verb land in their
own lanes, each with its own virtual-machine measurement, and task 4's
virtual-machine test — rename a file under a grant, undo it, find it as it was
and one `undone` entry — is written last, when all of them exist.

If the owner cannot change the filesystem at v0.5, **D is the honest fallback
and B is not**: B builds the thing the plan forbids to avoid saying *not yet*.

## As the owner accepted it, 2026-09-16

Option A, and six terms. The owner's question was the right one — *if the system
takes snapshots every turn, will the machines not get fuller with them?* A
snapshot costs nothing when it is taken and everything a person later changes or
deletes, so unbounded snapshots are a disk that fills quietly. The development PC
filled three times in one night for a different reason, and the machine a person
owns gets rules rather than the same lesson.

1. **Snapshots expire.** Undo reaches back a bounded window — **the last seven
   days or the last fifty changing turns, whichever ends first** — and what falls
   outside it is removed by the machine. The window is one value in
   `alo-keeping-up`, read from the person's settings where they change it, and a
   test holds that nothing keeps a snapshot beyond it.
2. **Disk pressure wins over undo, and is said.** Below a named amount of free
   space the oldest snapshots go first, and the record says those turns can no
   longer be undone. **The machine never fills a disk to preserve an undo**, and
   what a person is told names the turns that lost it rather than a number.
3. **A machine too full to snapshot still runs the turn.** The bracket is
   attempted; when there is no room it is not taken, the turn proceeds, and the
   record says plainly that this one cannot be undone. A verb that refused to run
   for want of an undo would be a machine that stopped working to protect a
   feature.
4. **What is filling the disk counts snapshots, by name.** `alo-measuring`'s
   answer includes what undo is holding, as its own line, because an answer that
   hid it would send a person hunting for space the machine itself was keeping.
5. **Only a turn that changes files is bracketed.** Reading, searching, asking a
   question and every refused call take no snapshot, held by a test.
6. **The filesystem is chosen at install and cannot be converted**, so the
   installer's change to `btrfs` lands **before the certified laptop is
   installed** (ADR 0033). A machine installed on `ext4` keeps working and
   answers *not yet on this machine* for every undo, honestly, until it is
   reinstalled — and the laptop must not be the first machine in that position.

Until the bracket exists on a machine, points 1 to 5 of *What holds whichever
option is taken* are built and every change verb answers *not yet on this
machine*, which is true rather than a stub.


## Amended, 2026-09-21 — who removes an expired snapshot

Terms 1 and 2 above both require a snapshot to be *removed*, and neither says
by what. That was not noticed at acceptance because on `ext4` there was nothing
to remove; the first real `btrfs` install made it answerable, and the answer is
not the one the terms assume.

**What was measured** (installer task 11, on five KVM boots against pinned
0.0.5, 2026-09-20): taking a read-only snapshot **needs no capability** —
`capsh --drop=cap_sys_admin` still exits 0 — while **removing one needs
`CAP_SYS_ADMIN`**, and a read-only snapshot cannot be cleared with `rm -rf`
either. The cost `btrfs` carries is not the snapshot. It is the deletion.

**What was found in this repository at the same time:**
`crates/alo-keeping-up/src/how_far_back.rs` decides precisely which snapshots
fall outside seven days or fifty changing turns, and **nothing anywhere removes
one.** The window was built; the forgetting was not. An undo that is never
forgotten is a disk that fills quietly, which is the exact failure the owner's
question at acceptance was about — *if the system takes snapshots every turn,
will the machines not get fuller with them?* The terms answered it and the code
did not.

### 7. Expiry is done by a privileged unit on a timer, and no agent can ask for it

What falls outside the window, and what disk pressure takes oldest-first, is
removed by **a unit the machine runs as root on a timer** — the shape
[ADR 0053](0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md) already
uses for applying an update. It is not a verb.

- **`alo-turn` does not gain this, and cannot.** It runs as the person; the
  removal needs `CAP_SYS_ADMIN`; a capability granted to the thing that takes
  the brackets is a capability held all day for an act performed once a day.
- **The broker's fixed list does not gain a `undo.forget` either**, and the
  reason is stronger than surface area. **An agent that can forget an undo can
  erase the evidence of what it did.** Undo is the record's counterpart: ADR
  0001 §5 holds that a change waits and is written down, and a destructive verb
  over the written-down past is the one verb whose approval a person is least
  able to judge, because what it destroys is the thing they would judge it by.
  Nothing on the fixed list is reachable by an agent for this, now or later.
- **Expiry is therefore housekeeping, not a petition.** There is no request, no
  approval, no grant and no entry point: a person changes the window in their
  settings (term 1) and the unit obeys it. The only way to keep a snapshot
  longer is to widen the window, which is the person's setting and always was.

What does not change: the record still names the turns that can no longer be
undone (term 2), `alo-measuring` still counts what undo is holding by name
(term 4), and a machine too full to snapshot still runs the turn (term 3). A
person is told the same things by the same roads; only the hand that does the
deleting is named, and it is not an agent's.

**Why not simply drop the window and expire on disk pressure alone.** That was
weighed and refused. It is the shortcut: it leaves a roomy machine keeping
every snapshot forever, answers the owner's question with *eventually*, and
makes the first forgetting a person ever sees happen at the worst moment, when
the disk is already full.

### What this amendment changes

- `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` gains a task for the
  remover. Terms 1 and 2 are **not built** until it lands, and
  `how_far_back.rs` deciding a window is not the same as a machine obeying one.
- `crates/alo-broker/src/verbs.rs` gains nothing. `SystemVerb` is a closed
  enum and stays closed here; a test should hold that no verb's name begins
  `undo.`, so that this decision is walked by the compiler and the suite rather
  than remembered.
- `crates/alo-turn` gains no capability.
- `docs/quirks.md` keeps what task 11 wrote: a snapshot taken into an existing
  directory is made *inside* it and answers `Read-only file system`, which
  reads like a mount fault and is not one.
## Consequences if it is accepted

- `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` task 4 becomes ready,
  split into what this lane builds (points 1 to 5 and the undo itself) and
  what it waits on from the installer, accounts, broker and lane A.
- `docs/contracts/record-file.md` gains the `undone` kind; `format` stays `1`.
- `docs/hardware.md` gains nothing: btrfs needs no feature the certified
  machine lacks, and that is measured on it like everything else.
- `docs/quirks.md` gains whatever the first btrfs install measures.

## What the code waits on

**Nothing is built in the change that adds this ADR.** While this line says
*proposed*, `crates/alo-record/src/happened.rs` has no `Undone` kind and
`crates/alo-keeping-up/src/` has no file about undoing, and the test named in
the status line holds both. A worker who finds this file still saying
*proposed* and a task asking for an undo has found a plan that got ahead of its
decision, and should stop at the plan.
