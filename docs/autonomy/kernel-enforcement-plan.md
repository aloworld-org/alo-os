# Kernel enforcement — completion plan

- Owner: Claude Code, kernel-enforcement workstream
- Crates owned here: `alo-bounding-kernel`, `alo-bounding`, `alo-boundaryd`,
  and the tests and interfaces those need
- Not owned here: the native desktop compositor, and the four shared progress
  documents (`CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md`,
  `docs/autonomy/STATE.md`), which the integration worker consolidates
- First written: 2026-09-07, against `2f1ba51`

This file is the workstream's own plan. It is written from an audit of the code
and the tests as they are, not from the queue, and every "done" below names the
test that says so.

## The scoping question, answered first

**"All kernel tasks" means the current release's accepted requirements.** The
current release is v0.01, and `docs/features.md` is the only list of what was
promised. Read carefully, it puts almost all of this work in the *next* release:

- `[v0.5] ★ The grant is a boundary the kernel imposes, not a rule the daemon
  follows (ADR 0013)` — Landlock, seccomp, the eBPF programme on the turn's own
  cgroup.
- `[v0.5] ★ And the kernel is taught what a turn is (ADR 0015)`.
- `[v0.5] So the record stops being anybody's account of itself.`

And ADR 0013 and ADR 0015 both carry **"Status: accepted — direction, ordered
behind `alo-agentd` and the turn"**. They are accepted decisions about *how*
this will be done, deliberately written early so the turn was not built assuming
ambient authority; they are not v0.01 delivery commitments.

**One kernel-security requirement is named inside a v0.01 line**, in the
*On the machine* half of **Egress indicator, and no telemetry**:

> the indicator itself, which is a compositor surface; the daemon code that
> actually signs somebody in, fetches a model or checks for an update, none of
> which exists yet; and **the enforcement at the network boundary, without which
> all of this describes only the code that asked**

So: **network egress enforcement and attribution is the in-scope kernel-security
requirement for this release.** Everything else audited below is v0.5 work, some
of which has already been delivered early. That is stated here so that nothing
in this plan is mistaken for release scope it does not have, and so that
finishing the list is never read as finishing the release.

## Audit — what is implemented, verified, incomplete, or hardware-bound

Evidence means a test that runs against the **real loaded BPF LSM** on a running
kernel, not a mock and not compilation.

### Implemented and verified

| Requirement | Evidence |
|---|---|
| A turn may open only what its call named, and what is under it | `alo-bounding/tests/the_kernel_refuses.rs` — granted file opens, private key is `EACCES`, non-turn unaffected |
| Authority is gone when the turn ends, not revoked later | same file, `when_the_turn_is_over_the_authority_is_gone` |
| A turn may not rename out of or into what nobody granted | `the_kernel_refuses_a_rename.rs` — six cases including both legitimate directions |
| A turn may not delete outside its reach | `the_kernel_refuses_a_delete_or_a_link.rs` |
| A turn may not hard-link with either end outside its reach | same file |
| Data is preserved when any of the above refuses | assertions in each refusal test |
| Operations outside a turn are unaffected | a non-turn case in each of the three files |
| The LSM decides and forgets — no map, counter or `bpf_printk` beyond the two the loader fills | `the_boundary_decides_and_forgets.rs` |
| An `O_PATH` handle is invisible to the boundary and confers no reading | `what_an_o_path_handle_is.rs` |
| A hard link made *before* a turn is inside every boundary, so `alo-files` refuses to read one | `a_hard_link_is_inside_every_boundary.rs` |
| Loader: a leftover pin on any hook is refused over and not removed | `alo-boundaryd` `a_machine_that_already_has_a_boundary_keeps_it`, per hook |
| Loader: taking a boundary away leaves no hook attached | `taking_a_boundary_away_leaves_none_of_its_hooks_attached` |
| Loader: the boundary outlives the loader; a second loader refuses | `the_boundary_outlives_the_loader.rs` |
| The whole journey — a real approved read, archive and move inside a real boundary | `alo-agentd/tests/a_turn_is_bounded_by_the_kernel.rs` |

Four hooks exist: `file_open`, `inode_rename`, `inode_unlink`, `inode_link`.

### Incomplete — and which release owns it

| Gap | Release | Note |
|---|---|---|
| **Network egress enforcement and attribution** | **v0.01** | `alo-egress` is policy and indicator only. No socket or cgroup programme exists anywhere in the tree. This is the one in-scope item. |
| Filesystem: `inode_create`, `inode_mkdir`, `inode_rmdir`, `inode_symlink` | v0.5 | Documented in `deciding.rs`; none moves a byte of somebody's file past a grant |
| Filesystem: `inode_setattr`, `inode_setxattr` — attributes and ownership | v0.5 | Not hooked |
| Already-open descriptors, and access inherited across the start of a turn | v0.5 | A `file_open` hook decides at open time and says nothing afterwards; a descriptor opened before the turn began stays usable inside it. **Not addressed anywhere** |
| Landlock, seccomp, namespaces — ADR 0013's other three primitives | v0.5 | None built; the BPF LSM carries the whole boundary today |
| A snapshot at turn start, and exact undo | v0.5 / v1 | Not built |
| Kernel-sourced enforcement records | v0.5 | **Needs a decision, not code** — see below |
| Loader upgrade path | — | A machine carrying pins from an older build refuses a new loader and an operator removes them by hand. Correct and deliberate; no automated upgrade exists |

### Hardware-dependent, and not tickable here

Every measurement in this workstream is from Ubuntu on WSL2. `docs/hardware.md`
says that cannot certify a machine. **No *On the machine* box may be ticked from
anything in this plan**, and no task below claims otherwise.

### The one thing that needs a decision rather than an implementation

ADR 0015 promises that *the record stops being the daemon's account and becomes
what the kernel watched happen*. Its own discipline forbids exactly the mechanism
that would produce that: **the LSM decides and forgets** — no map of
observations, no counter, no ring buffer — and there is a test that fails if a
third map appears.

Those two cannot both be delivered as written. Resolving it is a decision about
how much a security module may remember, which is the most dangerous question in
this repository, and it belongs in an ADR. **This workstream will not build a
reporting path without one.** Recorded here; not scheduled.

## Tasks

Ordered. Each names its acceptance criteria and the evidence that closes it.
A task blocked does not block the ones after it that do not depend on it.

### 1. Establishing the network egress policy the kernel can actually enforce

**Status:** ready. **Depends on:** nothing.

Before choosing a hook, the policy has to be stated, because the obvious reading
does not survive contact with the kernel. `alo-egress`'s policy decides by
*provider* and *region*; a kernel hook sees a cgroup, a protocol and an address.
A provider is a name resolved by DNS and a region is a fact about a company, and
neither is visible where the enforcement would sit. **Enforcing the egress policy
literally in the kernel is not possible and this task must say so rather than
approximate it.**

What is enforceable, and what ADR 0013 actually names, is narrower and stronger:
*which sockets may be opened, and attribution of every one to the turn that
caused it.* The candidate policy is therefore **default-deny for turns**: a turn
opens no socket unless the daemon has written a departure for it — which is the
`Departing` that `Indicator::beginning` already produces and which cannot be
obtained without the policy having been asked and the person having been shown.
Errands and every other process on the machine are not turns and are unaffected.

- **Acceptance:** a written policy statement, in this file and in the crate that
  will carry it, naming what the kernel can decide, what it cannot, and which
  half of law 1 each layer keeps. An explicit statement that this does not make
  the kernel the policy engine.
- **Evidence:** the statement itself. The reproduction is task 2.
- **Approval needed if:** the policy turns out to require the daemon to write
  anything the current `Bounds` map cannot carry, or to change what `Departing`
  means. Stop and ask rather than widening either.

**Done, 2026-09-07.** The statement is `crates/alo-bounding/src/lib.rs`, under
*What this boundary can decide about the network, and what it cannot*, and it
settles the question the task was written to ask:

> A turn opens no socket unless the person has been shown that it is about to.

`alo-egress`'s policy — provider, region — **cannot** be enforced in the kernel,
and the statement says so plainly rather than approximating it: a provider is a
name resolved through DNS and a region is a fact about a company, and a
programme on a socket sees a control group, a protocol and an address. A
kernel-side guess at either would be a second policy that disagrees with the
first unpredictably. The kernel is not the policy engine.

What is enforceable is default-deny for turns, attributed by the control group
the boundary already reads, with the daemon writing a turn's permission to leave
exactly as it writes the places a turn may reach — and the programme still
writing nothing down. **No approval was needed:** no grant widens, no capability
is added, `Departing` keeps its meaning, and a turn can still open exactly the
sockets the person was shown.

### 2. Reproducing unrestricted network access from inside a bound turn

**Status:** ready. **Depends on:** 1 (for what to assert).

- **Acceptance:** a test in `alo-bounding` that binds a turn, has the child
  attempt an outbound connection to an address nothing granted, and records what
  the kernel does today.
- **Evidence:** the test failing in the direction that shows the gap, exactly as
  the rename and delete gaps were reproduced before being closed.
- **Shared-machine safety:** the connection must be to a listener this test owns
  on loopback, on a port chosen by the operating system. **No test in this
  workstream may reach a real network, change host-wide networking, or touch
  anything the compositor worker's machine shares.** If loopback cannot be used
  safely, stop and report rather than reaching outward.

### 3. Socket attribution and default-deny for a bound turn

**Status:** blocked on 1 and 2. **Depends on:** 1, 2.

The programme and the map entry that make task 1's policy true.

- **Acceptance:** a bound turn with no departure written is refused a connection
  by the kernel; a bound turn with one is allowed; a process that is not a turn
  is unaffected; the daemon can write and withdraw a departure; and authority is
  gone when the turn ends.
- **Evidence:** tests against the real loaded programme, in the shape the four
  filesystem hooks already use, plus every existing filesystem test still
  passing.
- **Constraint:** no new agent capability, no widened grant, and the LSM still
  writes nothing down.

### 4. Documenting the filesystem mutations that remain unwatched

**Status:** ready, independent. **Depends on:** nothing.

`deciding.rs` lists them; `docs/quirks.md` does not, and neither does a contract.
This is documentation of a known limit, not new enforcement.

- **Acceptance:** the list of unwatched mutations, why each is not a way to move
  a file's contents past a grant, and which release owns closing it, written
  where somebody auditing the boundary will find it.
- **Evidence:** the text, and `cargo doc` clean.

### 5. Descriptors opened before a turn began

**Status:** ready, independent. **Depends on:** nothing.

Audit and document only. A `file_open` hook cannot see a descriptor that already
exists, and `alo-agentd` runs a turn as a thread of a process that has its own
open files — its record, its socket, its vocabulary.

- **Acceptance:** a written account of what is inherited into a turn today and
  what that does and does not permit, with a test if one can be written honestly.
- **Evidence:** the account, and any test it produces.
- **Approval needed if:** closing it would need the turn to become a separate
  process, which is a change to how a turn works and belongs in an ADR.

## Rules this workstream holds itself to

- No `unsafe` outside `alo-bounding-kernel`'s one permitted file, no weakened
  test, no placeholder, no kernel patch.
- Every refusal path tested beside the legitimate path it must not break.
- Gaps reproduced with temporary fixtures before being closed.
- Kernel behaviour verified against the real loaded BPF LSM.
- Data preserved on refusal, and asserted.
- *Inside the boundary* is never equated with *authorised*: `alo-capability`
  decides, and the kernel is the floor under a verb with a bug in it.
- Shared kernel state is coordinated — per-process pin paths, the existing
  `one_at_a_time()` lock, and never removing another process's pins, unloading
  its programmes or restarting anything the compositor worker depends on.

## Completion

This workstream is complete when every in-scope requirement above has executable
evidence, and **not when the task list is exhausted**. When the list empties, the
honest report is *implementation complete; hardware acceptance pending* — never
release completion, and never "kernel complete" while the v0.5 items above sit
unbuilt with their release named.
