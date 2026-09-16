# Undo what the agent did — the road it needs is a decision, and it is written

**Date:** 2026-09-15
**Workstream:** v0.5 — the machine keeps itself
(`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`), task 4
**Responsible contributor:** the machine-keeps-itself worker, in `C:\dev\alo-os`
**Status:** ready for integration **as a decision**. Task 4 itself is **not
done** and is marked *blocked* in the plan: its code waits on the owner's
answer to ADR 0045.

## What changed

| Path | What |
|---|---|
| `docs/decisions/0045-what-undoing-rewinds-to.md` | New, **proposed**. The roads to undo, what each costs, a recommendation, and what holds under every road |
| `crates/alo-keeping-up/tests/undoing_is_decided_before_it_is_built.rs` | New. Holds the ADR to existing once, standing, being named by the plan and setting out four costed roads — and holds the code to waiting while it is proposed |
| `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` | Task 4's status is *blocked — waits on the owner's answer to ADR 0045*, with what was found and what was decided |
| `docs/autonomy/updates/undo-what-the-agent-did-waits-on-a-snapshot-road.md` | This report |

No product code changed. No record kind, no answer about undoing and no sentence
was added, on purpose — see *Why the code waits*.

**User-readable change description.** *Undoing what an agent did to a person's
files needs the machine to keep an earlier copy of them, and the machine alo OS
installs today cannot: its disk is formatted in a way that has no snapshots. A
decision record now sets out how that can change — the recommendation is to
format with btrfs and take a snapshot either side of every change an agent
makes — and what undo will and will not be able to do, whichever way it is
decided. Nothing a person uses has changed yet.*

## What was found

The plan's task says what can be undone is undone *through the mechanism that
made it undoable — a file written inside a granted folder, where the base's own
snapshot of that subvolume is the road*. On the machine this repository
installs, **that road does not exist**:

- `crates/alo-installing/src/writing.rs` installs with
  `bootc install to-disk --filesystem ext4`, and `docs/booting.md` builds the
  development disk the same way. ext4 has no subvolume, no snapshot and no
  reflink.
- Task 3 measured that the base's rollback swaps `/usr`, keeps `/var` and leaves
  `/etc` with the newer build. A person's home is `/var/home`; rolling the image
  back does nothing for a file an agent renamed.
- The kernel-enforcement plan's audit already lists *a snapshot at turn start,
  and exact undo* as **not built**.

**Measured, 2026-09-15**, in this lane's WSL box (Ubuntu 24.04, podman 4.9.3),
running the pinned base as `localhost/alo-rollback-test:one`:

```
podman run --rm localhost/alo-rollback-test:one bash -c \
  'bootc install to-disk --help; ls /usr/lib/bootc/install/; rpm -q btrfs-progs; bootc --version'
```

`bootc 1.15.1`; `--filesystem` accepts `xfs`, `ext4`, `btrfs`;
`/usr/lib/bootc/install/` is empty, so the installer's argument decides;
`btrfs-progs-6.19.1-1.fc42` is in the base; no `snapper`. So a snapshot is a
configuration of the rented base, not a patch to it.

What an agent can change today, read off the tree: `rename_file`, `move_file`,
`archive_folder` (`alo-files`); `open_`, `focus_`, `close_`,
`arrange_application` (`alo-applications`); `print_document` (`alo-printing`);
`install_application` (`alo-software`). Only the first three change anything a
snapshot can hold.

## Why this is a decision and not code

Every way forward runs through something a worker may not choose alone:

1. **A snapshot needs a different filesystem**, chosen in
   `crates/alo-installing` — the installer plan's crate (ADR 0028: no lane
   edits another's). The disk plan says *nothing has decided the disk's* shape.
2. **The snapshot at turn start** belongs in `crates/alo-turn` (lane A's), and
   making one needs a privilege `alo-agentd` does not hold — the broker plan's
   to grant as a fixed verb, or refuse.
3. **Our own copy of the files** on ext4 contradicts ADR 0011's table (*undo …
   atomic snapshots and rollback — the base*) and this task's constraint (*never
   a second implementation*).
4. **Putting things back from the record** is the inverse the constraint forbids
   by name, and what ADR 0015 calls *best-effort*.
5. **Shipping undo as *never* for everything** narrows the ★ line in
   `docs/features.md`, which only the owner does.

The standing instructions for this lane say that when the only way forward runs
through contradicting an ADR, narrowing a promise or another lane's partition,
the decision itself is the work. ADR 0045 is that work.

## Decisions

**Recommended in ADR 0045 (for the owner):** option A — install with `btrfs`,
make each person's home its own subvolume, and bracket every changing turn with
two read-only snapshots kept outside every grant; undo learns what changed from
the two snapshots with the base's tools, refuses if the home no longer matches
the *after* snapshot on those paths, and restores from *before*. Option D
(nothing can be undone yet, said honestly) is the fallback; B (our copy) and C
(inverses) are set out and argued against.

**Settled in ADR 0045 under every option, so the next worker does not reopen
them:**

- which entries can never be undone and the reason a person is told — *it left
  this machine*, *it was told to a model*, *it was printed*, *an application is
  not something the machine can put back*; reads and refusals have nothing to
  undo; updates, returns, pairings and errands are not an agent's;
- a verb added later answers *never* until its own change says otherwise;
- an undo is the **person's**: a sentence, one approval, and **no agent verb**
  that undoes or proposes undoing;
- an undo never overwrites a later change — refused before it is offered;
- the record gains an additive `undone` kind carrying the undone entry's moment
  and its `What` **by copy**, so the account survives pruning; no agent field;
  `format` stays `1`;
- what undo keeps (including files the person later deleted) is visible,
  forgettable in one act, and its lifetime is the organisation's to name on a
  managed machine.

**Why the code waits rather than building the half that holds everywhere.** The
*never, and why* answers are true under every option, and could be built today.
They were not, because they are half of task 4's acceptance, and handing over
half a task as though it were the task is the one thing this lane may not do.
ADR 0045's recommendation says they are built first once it is accepted.

**Why the plan says *blocked* and not *Done*.** The loop steps over a blocked
task and re-selects an unmarked one; marking it done would claim an undo that
does not exist. Task 5 depends on 4 and waits with it, which is correct: its
sentences include *this can be undone* and *this cannot*.

## Acceptance criteria — where each stands

| Plan's criterion | State |
|---|---|
| For a verb the record says ran, `alo-keeping-up` answers whether it can be undone, and says plainly when it cannot | **Decided** (ADR 0045, point 1), **not built** |
| What can be undone is undone through the mechanism that made it undoable, never by guessing inverses | **Not buildable on the shipped machine** — the decision is ADR 0045 |
| Undoing is recorded, naming the entry it undid | **Decided** (point 4), **not built** |
| Undoing requires the person's approval | **Decided** (point 2), **not built** |
| *(this handover)* the decision exists, is named by the plan, sets out costed roads, and the code waits on it | **Built and tested** |

## Verification

Executed on 2026-09-15 against the checkout `C:\dev\alo-os`, from Ubuntu 24.04
in WSL2 (kernel `6.18.33.2`), which is where this machine can build at all —
every crate reaches `ring` and Windows has no C compiler. Build directory
`/root/alo-builds/alo-os-88e6ebddb0cab76e`, this checkout's own.

- `cargo fmt --all` (and `cargo fmt --all -- --check`) — clean.
- `cargo clippy -p alo-keeping-up --all-targets -- -D warnings` — clean, exit 0.
- `cargo test -p alo-keeping-up` — 47 + 6 + 10 tests pass, exit 0. The six are
  `undoing_is_decided_before_it_is_built`, one of which hands each check the
  thing it exists to catch (a second file with the number, a withdrawn status,
  a status line naming no status, a plan without the pointer, a road without a
  cost, an `Undone` entry and an `undoing.rs` beside a proposed decision) and
  requires it refused.
- `cargo test -p alo-citing` — 31 tests pass: every citation of ADR 0045
  resolves, and the decision is cited the way the convention requires.
- `cargo test plan::` in `tools/kernel-loop` (its own manifest, build directory
  `/root/alo-builds/kernel-loop-gates`) — 12 pass, including
  `every_plan_this_repository_drives_holds_only_tasks`: the edited plan still
  reads as numbered tasks and task 4 reads as blocked rather than done.

Not executed: the full workspace suite (the supervisor runs it); any virtual
machine — nothing here touches the machine.

## Remaining limitations

- The measurements option A needs before anything is built are listed in
  ADR 0045 and not taken: whether `bootc install to-disk --filesystem btrfs`
  makes subvolumes of its own, which capability a snapshot needs on the pinned
  kernel, and whether an update and a return leave a home subvolume alone.
  Taking them means loop-mounting a disk in the shared WSL box, which
  `SHARED_MAIN.md` asks to coordinate.
- A machine already installed on ext4 has no conversion under option A.

## Proposed updates to the shared documents

- **CHANGELOG.md:** the user-readable description above, under *decided, not
  yet built*.
- **ROADMAP.md:** under ★ *Undo what the agent did*, a sub-line: *the road is a
  decision — ADR 0045, proposed 2026-09-15; the shipped disk (ext4) has no
  snapshot*. Nothing ticked.
- **QUEUE.md / STATE.md:** machine-keeps-itself plan task 4 **blocked on
  ADR 0045**, not done; task 5 waits with it. Owner's question: accept option A
  (and which lanes carry the installer argument, the home subvolume, the
  bracket and the broker's verb), or D.
