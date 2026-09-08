# Descriptors opened before a turn began

- **Date:** 2026-09-08
- **Workstream:** kernel enforcement
- **Contributor:** Claude Code, in `C:\dev\alo-os-claude`
- **Task:** Descriptors opened before a turn began —
  `docs/autonomy/kernel-enforcement-plan.md`, task 6
- **Status:** ready for integration. Audit and documentation, with the whole of
  it reproduced. **Nothing was closed and no release gate is touched.**

## What this task was, and what it turned out to be

The plan asked for *a written account of what is inherited into a turn today and
what that does and does not permit, with a test if one can be written honestly*.
A test could be written honestly, so there are two of them, and the account they
hold up says something the audit did not expect.

**Every gap this workstream has documented so far keeps one promise. This one
does not.** Task 5's list of unwatched filesystem mutations was measured against
*no unwatched mutation moves a byte of somebody's file past a grant*, and each of
the seven keeps it — a symbolic link is a name, an empty file is empty, an
extended attribute needs bytes the turn cannot read. A descriptor opened before
the turn began is different in kind: the turn reads a file nobody granted through
it and writes what it read into the folder somebody did, from where an
`archive_folder` or a `move_file` carries it onwards and where the record names
only a granted path. That is the whole of what ADR 0013 says the boundary is for.

It is measured rather than argued:
`crates/alo-bounding/tests/what_a_turn_inherits.rs`, one turn, one thread, one
run — refused `open` on the private key with `EACCES`, every byte of the key read
through a descriptor opened a moment earlier, and those bytes written into the
granted folder.

## What is inherited, on this machine, today

A turn is **one thread of `alo-agentd`** and not a process of its own. That is
law 2's answer and `crates/alo-bounding/src/turns.rs` argues it: every shape that
spawns needs a program to spawn, and a program alo OS starts on an agent's behalf
is one review away from a program an agent named. The consequence is that a turn
shares the daemon's entire descriptor table for as long as it runs.

What is in that table:

- **the machine's record** — `alo_keeping::Writing` opens it with `append(true)`
  when the daemon starts and holds it for the life of the process;
- **the way out of a turn** — `Turns::back`, which is `home/cgroup.threads`,
  opened before the first turn ever ran;
- **the door and whoever is at it** — the `UnixListener` `alo-agentd` binds and
  the `UnixStream` it is answering a caller on;
- **standard output and error**, and whatever else started the service left open.

No verb hands a model a descriptor: the six verbs take paths and `alo-files`
opens what it opens from inside the boundary. So none of this is reachable
through the capability model, and all of it is reachable by a verb with a bug in
it — which is exactly the thing ADR 0013 says the kernel boundary is the floor
under.

## What it permits, and what it still does not

The account is the table in `docs/quirks.md` under *A descriptor opened before a
turn began is inside no boundary*. Five rows, each reproduced:

| What a turn inherits | Reproduced in |
|---|---|
| `a file open for reading` | `what_a_turn_inherits.rs` |
| `a file open for appending` | `what_a_turn_inherits.rs` |
| `a directory descriptor` | `what_a_turn_inherits.rs` |
| `a socket already connected` | `what_a_bound_turn_can_still_reach.rs` (task 7) |
| `the way out of a turn` | `what_a_turn_inherits.rs` |

The floor under the gap is measured beside every one of them, and it is real:

- **a descriptor cannot be reopened by name.** The record is appendable through
  the descriptor the daemon holds and the same turn is refused `open` on it, so a
  turn can add a line and cannot read, replace or truncate the record. `O_APPEND`
  puts every write at the end, so nothing already written can be altered either.
- **`/proc/self/fd/<n>` does not turn a descriptor back into an open.** The
  boundary's walk starts at the file the open really reached, so the private
  key's descriptor is refused there exactly as its name is. **The control for
  that is the granted file reopened the same way, which is allowed** — everything
  under `/proc` is outside the bound, so a boundary refusing `/proc/self/fd` on
  sight would produce an identical refusal and prove nothing about where the walk
  starts.
- **a folder handle is not a key to what is under it.** `openat` relative to an
  inherited directory is still an open; the hook is handed the file that was
  opened rather than the base, and the file is refused. The handle itself stays
  valid, which is asserted so that the refusal cannot be a stale descriptor.
- **no `cgroup.threads` can be opened by name**, the service's or the turn's own,
  so the way out has to be inherited and cannot be obtained from inside.

## The decision this needs, stated and not taken

**The plan's approval line was `closing it would need the turn to become a
separate process`. That line was not crossed: nothing here closes anything.**
What the audit adds is why neither available answer is a patch.

**The kernel's answer would be `file_permission`** — the hook that fires on every
read and every write — or `file_receive`, for a descriptor arriving from
somewhere else. Neither is in `crates/alo-bounding-kernel/src/kernel.rs`. Two
things are wrong with adding one:

1. it would put this boundary's walk on the hottest hook in the kernel, deciding
   about every read on the machine, which is the opposite direction from ADR
   0015's *the LSM decides and forgets*;
2. **it would break the turn.** Leaving a boundary is a write to
   `home/cgroup.threads` through a descriptor opened before the turn began,
   because opening that file from inside is an open the boundary correctly
   refuses. `the_way_out_of_a_turn_is_a_descriptor_the_boundary_would_refuse_to_open`
   measures both halves in one run: the open refused with `EACCES`, and the turn
   leaving anyway. A boundary that re-decided at the moment of *use* would refuse
   a turn its own way out.

**The other answer is to make a turn a process of its own**, with a descriptor
table it did not inherit. That is a change to what a turn is, it collides with
law 2's *nothing is started* — `turns.rs` chose the thread deliberately and for
that reason — and it belongs in an ADR rather than in a commit.

**This workstream has taken neither decision and written neither ADR.** Whoever
schedules v0.5 has to, and this report is where the question is written down.

## Files changed

| File | What changed |
|---|---|
| `crates/alo-bounding/tests/what_a_turn_inherits.rs` | New. Five tests through `Turns::doing`, against the real loaded programme |
| `crates/alo-bounding/tests/what_a_turn_inherits_is_written_down.rs` | New. Four tests holding the account to the programme and to its own reproductions |
| `docs/quirks.md` | New entry: *A descriptor opened before a turn began is inside no boundary*, with the table and the two answers |
| `crates/alo-bounding/src/lib.rs` | New section: *What a turn inherits, and why it is not on that list* |
| `crates/alo-bounding-kernel/src/deciding.rs` | The *what is inside a file already open* bullet now says what it means for the daemon and what would close it |
| `crates/alo-bounding/src/turns.rs` | The way-out descriptor now says it is the same property as the gap, used on purpose |
| `crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs` | Cross-reference: the socket row belongs to this account |
| `docs/autonomy/kernel-enforcement-plan.md` | Task 6 closed, with what the audit found; the audit table's row corrected |
| `docs/autonomy/updates/descriptors-opened-before-a-turn.md` | This report |

## A note on the shape of the tests

Its two siblings — `what_a_bound_turn_can_still_change.rs` and
`what_a_bound_turn_can_still_reach.rs` — put a **child process** in a control
group, which is the simplest way to bind something when what is being measured is
a hook's answer. That shape cannot measure this task's subject at all:
inheritance is exactly what a child process gets differently.

So every test here goes through `alo_bounding::Turns::doing`, on the thread the
assertions are made from, which is what `alo-agentd` really does. Nothing is
asserted from inside the boundary — `a_turn_is_this_thread.rs`'s rule, because a
failing assertion inside panics, a panic prints a backtrace, and a backtrace
opens `/proc/self/maps`, which is refused. Everything is gathered inside and
judged outside.

Every run also opens the file nobody granted, which must be refused, and the file
somebody did, which must be allowed. Neither is decoration: the first is what
says the boundary was in force at all, and the second is what says it was not
simply refusing everything.

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel started.
`CARGO_TARGET_DIR` is this checkout's own.

### Acceptance evidence — each run on its own

| Acceptance criterion | Test |
|---|---|
| The account exists, is true of the programme, and is where an auditor reads | `. alo-bounding what_a_turn_inherits_is_written_down what_a_turn_inherits_is_written_down_where_an_auditor_will_find_it` |
| …and the check would catch each way it can rot | `. alo-bounding what_a_turn_inherits_is_written_down the_check_catches_an_account_that_has_stopped_being_true` |
| What an inherited read descriptor permits: contents past a grant | `. alo-bounding what_a_turn_inherits a_file_opened_before_the_turn_began_is_still_readable_inside_it` |
| What an inherited append descriptor permits, and what it does not | `. alo-bounding what_a_turn_inherits a_record_opened_before_the_turn_began_is_still_appendable_inside_it` |
| What it does not permit: a descriptor is not a new open | `. alo-bounding what_a_turn_inherits an_inherited_descriptor_cannot_be_reopened_through_the_name_the_kernel_gives_it` |
| What it does not permit: a folder handle is not a key | `. alo-bounding what_a_turn_inherits a_directory_opened_before_the_turn_began_is_not_a_key_to_what_is_in_it` |
| Why closing it is a decision: the way out is itself inherited | `. alo-bounding what_a_turn_inherits the_way_out_of_a_turn_is_a_descriptor_the_boundary_would_refuse_to_open` |

### Gates

- `cargo fmt --all --check` — clean
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `cargo test --workspace` — passing
- `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` — clean
- the supervisor's own formatting, clippy and tests — clean
- the BPF target's formatting and clippy on the pinned nightly — clean

Results are in the loop's own journal for this iteration; the supervisor runs
every gate and then each piece of evidence on its own before anything is
published.

### Not run here

- **Windows gates.** `alo-bounding` compiles to nothing on Windows and there is
  no BPF target there. Nothing in this change is Windows-specific.
- **Certified hardware.** Every measurement above is WSL2.
  `docs/hardware.md` says that cannot certify a machine, so **no *On the machine*
  box is ticked by this report** and none may be.

## Limitations

- **Nothing is closed.** This is an audit that runs. The gap it documents is
  wider than any other this workstream has documented, and it stays open.
- **The inventory of inherited descriptors is this repository's**, read from
  `alo-keeping`, `alo-agentd` and `alo-bounding`. A descriptor opened by a crate
  added later would not appear in it, and nothing mechanical would notice — the
  table's rows are by *shape* rather than by owner for that reason, so a new
  file-open-for-reading is covered by the row that already exists.
- **The socket row is reproduced by task 7's file**, not by this one, and is
  cross-referenced rather than duplicated.
- **`file_receive` is named and not reproduced.** Nothing in this repository
  passes a descriptor over its socket, so there is no honest way to measure what
  would happen if something did; the hook is named as one of the two that would
  close this and the account says so rather than claiming a measurement.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **What a turn inherits is written down, and measured.** A turn runs as one
> thread of the agent service, so everything that service already had open — its
> record, its socket, its own way out of a turn — is open inside the turn too,
> and the kernel boundary is never asked about it. That is now documented with
> every case reproduced against a real kernel, including the one that matters
> most: a file opened before a turn begins can be read inside it and what is read
> can be written where the turn was allowed to write. Nothing about the boundary
> changed; what changed is that this is written down where somebody auditing it
> will find it, instead of being discovered.

### `docs/autonomy/QUEUE.md`

Task 6 of the kernel-enforcement plan is complete. It closes no gap and ticks no
release gate. It adds one question for whoever schedules v0.5: **whether a turn
becomes a process of its own, or whether this boundary starts deciding about
descriptors as well as about opens.** That needs an ADR and this workstream has
not written one.

### `docs/autonomy/STATE.md`

Reference `docs/autonomy/updates/descriptors-opened-before-a-turn.md`. The
kernel-enforcement plan's task list is now empty except for what its own
*Completion* section forbids reading as completion: the honest sentence is
*implementation complete for the in-scope v0.01 requirement; hardware acceptance
pending*, with the v0.5 items in the audit table still unbuilt and their release
named.

### `ROADMAP.md`

No change proposed. Nothing here is a v0.01 delivery commitment and nothing here
is measured on certified hardware.
