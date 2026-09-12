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

## Audit — five sections, and each one is a different kind of claim

Reconciled with the code and the published reports on 2026-09-08. What is built
and proved, what this release still owes, what a later release owes, what only a
physical machine can settle, and what nobody has decided. A row that moved
between them says which report moved it.

### 1. Implemented and verified

Evidence means a test that runs against the **real loaded BPF LSM** on a running
kernel, not a mock and not compilation.

**Seven hooks exist:** `file_open`, `file_permission`, `inode_rename`,
`inode_unlink`, `inode_link`, `socket_connect`, `socket_sendmsg`. The programme
has exactly two maps and writes nothing down.

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
| A bound turn cannot start a program — `execve` opens the file it runs | `what_a_bound_turn_can_still_change.rs` |
| **A bound turn is refused a destination nobody showed it** | `the_kernel_refuses_a_departure.rs` — seven cases; second destination refused while the first is permitted, withdrawal, both address families, loopback exempt, non-turn unaffected |
| **Both halves of law 1 in one turn** — a file nobody granted and a destination nobody showed, refused together | `what_a_turn_can_reach_on_the_network.rs` |
| **A provider request runs inside the boundary and answers** | `alo-agentd/tests/a_question_is_bounded_by_the_kernel.rs` — production path, real socket, answer and `authorization:` header asserted |
| **The production path reaches a destination the kernel rules on** | same file, at this machine's own non-loopback address |
| **An unregistered destination is refused while the registered one works** | same file, `EACCES` |
| **A question the rule refuses reaches no socket** | same file; indicator quiet, server saw nothing |
| **A failed request leaves no permission for the next one** | same file, asked of the kernel rather than assumed |
| **A question is bounded, and only to what it resolved** | `alo-turn` `a_file_verb_and_a_question_are_both_carried_out_inside_a_boundary` |
| **A socket joined before the turn began is refused the moment the turn writes on it** — and one joined to a shown destination carries on | `what_a_bound_turn_can_still_reach.rs`, `a_connection_made_before_the_boundary_is_refused_inside_it` and its sibling |
| **A datagram sent without connecting is refused** — and one to a shown destination goes, one to loopback goes unshown, and a process that is not a turn sends it | same file, `an_unconnected_datagram_is_refused_inside_a_bound_turn` and two siblings |
| The message hook, outside a turn, leaves no trace | `the_boundary_decides_and_forgets.rs`, datagrams beside the opens |
| A refused message and a refused connection are one sentence in the record | `alo-asking` `a_message_the_kernel_refused_is_the_sentence_a_refused_connection_is` |
| **A descriptor opened before the turn began is refused at the first byte** — read, write, append and listing, with nothing moved and the file undisturbed | `what_a_turn_inherits.rs`, six refusals through `Turns::doing` on the asserting thread |
| **A descriptor to a file inside the grant is untouched** — read through, written through, listed through; and a Unix socket is left to the message hook | same file, `a_descriptor_to_a_file_inside_the_grant_is_untouched` and two siblings |
| **A turn cannot write itself out of its boundary, and ends anyway** — a thread of the service that was never in it brings it home | same file, `the_way_out_of_a_turn_is_refused_to_the_turn_and_the_turn_ends_anyway` |
| The read-and-write hook, outside a turn, leaves no trace | `the_boundary_decides_and_forgets.rs`, reads and writes beside the opens |
| A refused read and a refused open are one sentence in the record | `alo-files` `a_read_the_kernel_refused_is_the_sentence_a_refused_open_is` |
| The account of what a turn inherits is held to the programme | `what_a_turn_inherits_is_written_down.rs` |
| Loader: a leftover pin on any hook is refused over and not removed | `alo-boundaryd` `a_machine_that_already_has_a_boundary_keeps_it`, per hook |
| Loader: taking a boundary away leaves no hook attached | `taking_a_boundary_away_leaves_none_of_its_hooks_attached` |
| Loader: the boundary outlives the loader; a second loader refuses | `the_boundary_outlives_the_loader.rs` |
| The whole journey — a real approved read, archive and move inside a real boundary | `alo-agentd/tests/a_turn_is_bounded_by_the_kernel.rs` |
| Publication fails closed on every path, including a person's | `tools/kernel-loop`, 23 tests |

**The stale row this table used to carry** said *no socket or cgroup programme
exists anywhere in the tree*. It was written before task 3 and left standing
through task 4. It is gone; the eight rows above replace it.

### 2. Remaining current-release (v0.01) gaps

The in-scope kernel-security requirement for this release is the *On the machine*
half of **Egress indicator, and no telemetry**, and the sovereignty claim it
serves is `docs/features.md`'s *every network egress an agent causes, visible at
the moment it happens*.

| Gap | State | What it needs |
|---|---|---|
| **A proxy on loopback carries a question off the machine, unshown** | **Open, production-reachable, reproduced** — `what_a_bound_turn_can_still_reach.rs` for the kernel half, and the door is `Answers::Service`, not `Answers::Provider`, which already refuses a loopback address | **A decision, not code.** [ADR 0021](../decisions/0021-what-a-service-on-this-machine-vouches-for.md) is **proposed** and recommends trusting a service the person started *with the promise reworded to say so*. Nothing is built and no promise is narrowed until the owner answers |

**Nothing else in this workstream is v0.01.** Everything below is later-release
hardening with its release named, and the remaining clauses of the *Egress
indicator* roadmap line — the compositor surface, and the daemon code that signs
somebody in and fetches a model — belong to other workstreams.

### 3. Later-release hardening (v0.5 and beyond)

Each is documented, most are reproduced, and none is scheduled here.

| Gap | Release | State |
|---|---|---|
| ~~A descriptor opened before the turn began~~ | v0.5 | **Closed, task 12, 2026-09-12** — `file_permission` decides on every read and write, asked of the using thread's cgroup; the turn is brought home by a thread that was never in it. Moved to section 1 |
| **A mapping of a file opened before the turn began** | v0.5 | `mmap_file` is not hooked: a file mapped into memory is read by the processor, so a mapping of an inherited descriptor made inside the turn reaches its contents past `file_permission`. **Not reproduced** — there is no safe `mmap` in Rust and `unsafe` is forbidden outside the kernel package's one file; the same rule that keeps `truncate(2)` out of the suite. `docs/quirks.md` names it beside what closed |
| ~~A socket already open or inherited~~ | v0.5 | **Closed, task 13, 2026-09-12** — `socket_sendmsg` decides on every message, asked of the sending thread's cgroup. Moved to section 1 |
| ~~A datagram sent without connecting~~ | v0.5 | **Closed, task 13, 2026-09-12** — the same hook reads the address a message names. Moved to section 1 |
| ~~A connection reused after its destination is withdrawn~~ | v0.5 | **Closed, task 13, 2026-09-12** — the message hook reads the map on every message, so a withdrawn destination is refused on the next write. ADR 0020's per-request client had already closed it on the production path |
| Filesystem: `inode_create`, `inode_mknod`, `inode_mkdir`, `inode_rmdir`, `inode_symlink` | v0.5 | Documented and reproduced by task 5; none moves a byte past a grant |
| Filesystem: `inode_setattr`, `inode_setxattr` — attributes, ownership **and size** | v0.5 | Not hooked. `truncate(2)` reaches `inode_setattr` without an open, so a bound turn can **empty** a file nobody granted. No contents leave a grant and contents are destroyed where they are — the one row that does more than litter, and the one to close first |
| Landlock, seccomp, namespaces — ADR 0013's other three primitives | v0.5 | None built; the BPF LSM carries the whole boundary today |
| A snapshot at turn start, and exact undo | v0.5 / v1 | Not built |
| Kernel-sourced enforcement records | v0.5 | **Needs a decision, not code** — see below |
| Loader upgrade path | — | A machine carrying pins from an older build refuses a new loader and an operator removes them by hand. Correct and deliberate; no automated upgrade exists |

### 4. Physical hardware acceptance

**Every measurement in this workstream is Ubuntu on WSL2**, kernel 6.18.33.2.
`docs/hardware.md` says that cannot certify a machine, and ADR 0007 requires
**two** certified machines — an ordinary business laptop first, then a GPU
workstation with 24 GB VRAM or more.

- **No *On the machine* box may be ticked from anything in this plan**, and no
  task in it claims otherwise.
- The image has been booted in QEMU with KVM and answered all five kernel
  questions; a virtual machine is not a certified one, and `ROADMAP.md`'s image
  line keeps its machine box empty for that reason.
- Nothing here is certified. The honest phrase when the task list empties is
  *implementation complete for the in-scope requirement; hardware acceptance
  pending* — never release completion.

### 5. Decisions awaiting an answer

Three, and this workstream builds none of them without one.

1. **What a service on this machine vouches for.**
   [ADR 0021](../decisions/0021-what-a-service-on-this-machine-vouches-for.md),
   proposed 2026-09-08 and revised twice — for a third option, and to align with
   the owner's model-choice clarification. It separates who provides the model,
   who manages the runtime, where processing occurs, what alo can verify, and
   what has been permitted; **brand is evidence of none of them**. Recommends
   **C1** (truthful processing-location labels), rejects A, and leaves the
   `ThisMachineOnly` rule as **four stated options with consequences** rather
   than taking one. A future local-only guarantee would be qualified by
   **supervision, not ownership** — a third-party runtime under alo's
   supervision qualifies and alo's own outside it does not — and is blocked on a
   mechanism that does not exist. **The gap is open under every option.**
2. **Inherited file descriptors.** Options in
   `docs/autonomy/updates/network-boundary-decisions-proposed.md`. The
   question is not which hook: every hook runs into the same exemption, because
   the way out of a turn is itself an inherited descriptor. **The socket half
   of that decision was taken by task 13 on 2026-09-12 and needed no
   exemption**: a message has a destination where a read has none, the way out
   of a turn is a file and not a socket, and the daemon's socket to the person
   is a Unix socket the hook does not call egress. What remains open is the
   file half, and only the file half.
3. **Kernel-sourced enforcement records.** ADR 0015 promises the record becomes
   what the kernel watched; *the LSM decides and forgets* forbids the mechanism
   that would produce it, and a test fails if a third map appears. Those two
   cannot both be delivered as written. Recorded, not scheduled.

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

**Done, 2026-09-07.** `crates/alo-bounding/tests/what_a_turn_can_reach_on_the_network.rs`.
One turn bound to one folder, on a running kernel with the real programme
loaded: the same child is **refused a file nobody granted** with `EACCES`, and
then **opens a socket, which nothing stops**.

The file refusal is the control and it is not decoration — *a bound turn opened
a socket* is also what a turn with no boundary at all would do, so a fixture
that quietly failed to bind anything would have reported the gap it was written
to find. The address is a listener the test owns on loopback, on a port the
operating system chose; nothing resolves a name and nothing leaves the machine.

The test asserts what this machine does **today**, so the day task 3 lands it
fails and says where to come and what to change.

### 3. Socket attribution and default-deny for a bound turn

**Status:** ready. **Depends on:** 1, 2 — both done.

The programme and the map entry that make task 1's policy true.

#### The policy, corrected

Task 1 said a turn with a departure written may open sockets. **That is wrong
and would have shipped a hole**: one displayed departure would have become
permission for every destination the turn cared to reach, and a person shown
*asking alo, in Frankfurt* would have authorised a connection to anywhere. The
policy is per-destination, and these are its four parts.

- **Destination binding.** A departure permits **one destination** — an address
  and a port — and nothing else. The bound carries the list of destinations the
  person was actually shown. An address the person was not shown is refused even
  while another departure is open.
- **Lifetime.** A destination is permitted for the **turn**, not for a
  connection and not for a period. It arrives in the same map entry as the
  places a turn may reach, is written when the departure is shown, and is gone
  when the turn ends — which is the existing revocation, unchanged, and already
  tested.
- **Withdrawal.** The daemon rewrites the entry without that destination. It
  takes effect on the next `connect`, immediately, because the programme reads
  the map on every call and holds nothing between them.
- **Connection reuse.** `socket_connect` fires when a connection is *made*. A
  socket already established is **not** re-checked, so a destination withdrawn
  while a connection to it is open stays reachable over that connection until it
  closes. This is a gap, it is stated here rather than discovered later, and it
  is the same shape as the already-open-descriptor gap in task 5.

#### What is covered, and what is a named gap

| | |
|---|---|
| TCP over IPv4 | enforced |
| TCP over IPv6 | enforced — the bound holds 128-bit addresses |
| UDP with `connect` | enforced, because it reaches the same hook |
| **UDP with `sendto`** | **not enforced.** An unconnected datagram never reaches `socket_connect`; it would need `socket_sendmsg`, which is a hook on every message rather than every connection. Named, not built |
| **A socket already open** | **not enforced**, as above |
| **A socket inherited into a turn** | **not enforced** — the same class as task 5 |
| A destination reached through a proxy the person was shown | enforced against the *proxy's* address, which is what the machine can see and what the indicator showed |

#### One map, and the guard is not touched

Destinations go in the **existing** `BOUNDS` entry, beside the places. A third
map would change what `the_program_has_nowhere_to_write_what_it_sees` asserts —
it names `["BOUNDS", "FIELDS"]` exactly, because *a program that had somewhere
to write would have to have somewhere, and this is the list of everywhere it
has*. Extending a value the daemon already writes keeps that guard true as
written and keeps the rule it stands for intact.

**Cgroup attribution is not an audit record and this task does not claim it is.**
The programme still writes nothing down; what a person is shown is what
`alo-egress` shows them. Kernel-sourced recording remains the separate decision
recorded above, unscheduled and unbuilt.

- **Acceptance:** a bound turn is refused a connection to a destination nobody
  showed; permitted the one that was; refused a *second* destination while the
  first is permitted; refused after withdrawal; and unaffected when it is not a
  turn. Authority gone when the turn ends. Both address families. Every existing
  filesystem and loader protection still passing.
- **Evidence:** tests against the real loaded programme, to listeners the tests
  own on loopback, in the shape the four filesystem hooks already use.
- **Constraint:** no new agent capability, no widened grant, the userspace
  provider-and-region policy untouched, and the LSM still writes nothing down.
- **Approval needed if:** the mechanism turns out to need a third map, a hook
  that sees message contents, or any change to what `Departing` means.

**Done, 2026-09-07.** A fifth hook, `socket_connect`, and destinations carried
in the existing `BOUNDS` entry. **None of the three things that would have
needed approval happened**: no third map, no hook that sees a message,
`Departing` untouched.

Seven tests against the real loaded programme: a turn shown nothing is refused
every destination; the one it was shown is permitted; **a second destination is
refused while the first is still permitted**; another port on the same address
is refused; a withdrawn destination is refused next time; loopback is not
checked; a process that is not a turn is unaffected. The gap test written for
task 2 was inverted by this change, which is what it was for.

`the_program_has_nowhere_to_write_what_it_sees` still asserts exactly
`["BOUNDS", "FIELDS"]`, **unchanged**. It caught a `.rodata.cst32` section that
128-bit constants had made the compiler emit; rather than widen the list — even
with the honest justification that `.rodata` is frozen read-only — the constants
were written so that none is emitted. The guard was not relaxed, not even
defensibly.

### 4. End-to-end network enforcement integration

**Status:** done. **Depends on:** 3 — done. **The approval this waited on was
given** on 2026-09-07 (`d220aef`,
`docs/autonomy/updates/network-request-boundary-approval.md`) and is recorded as
**[ADR 0020](../decisions/0020-a-question-is-carried-out-inside-the-turns-boundary.md)**,
written before any of the implementation below.

The mechanism task 3 built decides by control group, and a provider request was
made from a thread in no control group, so nothing on the production path
reached it. That is now closed:
`crates/alo-turn/tests/whether_a_question_runs_inside_the_boundary.rs` counted
one execution inside a boundary during a turn that did a file verb and asked a
question, and it was the file verb. **It now counts both**, and asserts the
question was let reach what its endpoint resolved to and nothing else.

- **Acceptance:** an authorised provider request succeeds through the production
  path under the real loaded programme; an unauthorised destination is refused
  while an authorised one stays usable; a question the rule refuses reaches no
  socket at all; failure leaves no permission or connection behind; local-model
  operation and every existing filesystem protection still pass.
- **Evidence:** `crates/alo-agentd/tests/a_question_is_bounded_by_the_kernel.rs`
  — five tests, against the real loaded BPF LSM, on real sockets.

**Done, 2026-09-07.** Resolution is separated from connection: the daemon
resolves the endpoint **outside** any boundary, registers the addresses, enters
a control group and makes the request with a resolver that returns those
addresses and asks no name server anything — so **there is no DNS inside the
boundary and no exception for it**, which is what the approval asked for. The
client is built per request and dropped with it, so its connection pool cannot
outlive the permission. Only the resolver is replaced, so the URL keeps the
provider's hostname and certificate verification is unchanged. `Bounding`'s new
method is **defaulted to a refusal**, so an implementation that predates ADR 0020
refuses to send rather than sending unbounded.

**No approval-scope line was crossed:** no new capability, no widened grant, no
blanket network permission, *decides and forgets* untouched, loopback still
unchecked, and the provider-and-region policy still entirely in userspace.

**What is still not covered is named in the report** rather than implied: UDP
sent without a connection, sockets already open or inherited, the loopback
proxy, and a provider resolving to more than two addresses being reached at the
first two. `docs/autonomy/updates/end-to-end-network-enforcement.md` carries the
whole of it.

### 5. Documenting the filesystem mutations that remain unwatched

**Status:** ready, independent. **Depends on:** nothing.

`deciding.rs` lists them; `docs/quirks.md` does not, and neither does a contract.
This is documentation of a known limit, not new enforcement.

- **Acceptance:** the list of unwatched mutations, why each is not a way to move
  a file's contents past a grant, and which release owns closing it, written
  where somebody auditing the boundary will find it.
- **Evidence:** the text, and `cargo doc` clean.

**Done, 2026-09-08.** The list is in `docs/quirks.md` under *Four hooks are not a
filesystem*, beside the code in `crates/alo-bounding-kernel/src/deciding.rs`,
where an auditor of the crate reads in `crates/alo-bounding/src/lib.rs`, and in
plain words for an adapter author in `docs/contracts/agent-verbs.md`. Seven rows,
each naming what a bound turn can still do, why it moves no contents past a
grant, and v0.5 as the release that owns closing it.

**It is documentation that runs.** `what_a_bound_turn_can_still_change.rs`
reproduces every row against the real loaded programme — each with a refused open
proving the boundary was in force and a legitimate write inside the grant proving
it was not simply refusing everything — and
`the_unwatched_mutations_are_written_down.rs` parses the table and fails the day
a listed hook is watched, a row names a release `docs/features.md` has never
heard of, a row says nothing about why contents stay inside, a row is reproduced
nowhere, or a sixth hook arrives with no document naming it.

**Two things the audit found that the task did not ask for.** A bound turn
**cannot start a program**, because `execve` opens the file it runs and
`file_open` is watched — so none of the unwatched mutations can be escalated by
running something that makes the calls instead. And the size half of
`inode_setattr` is worse than the rest of the list: `truncate(2)` empties a file
without opening it, measured here with the same turn refused `open` on the file
it had just emptied. It moves no contents past a grant, which is why the list's
promise survives, and it destroys them where they are, which is why it is named
rather than filed. It is the only row **not** reproduced in the committed suite,
because no call this repository can make reaches `truncate(2)` without an open
and a language that is not Rust is a bug here; `docs/quirks.md` records the
method and the limit rather than a claim.

Nothing was closed, nothing was ticked, and no *On the machine* box is touched.

### 6. Descriptors opened before a turn began

**Status:** ready, independent. **Depends on:** nothing.

Audit and document only. A `file_open` hook cannot see a descriptor that already
exists, and `alo-agentd` runs a turn as a thread of a process that has its own
open files — its record, its socket, its vocabulary.

- **Acceptance:** a written account of what is inherited into a turn today and
  what that does and does not permit, with a test if one can be written honestly.
- **Evidence:** the account, and any test it produces.
- **Approval needed if:** closing it would need the turn to become a separate
  process, which is a change to how a turn works and belongs in an ADR.

**Done, 2026-09-08.** The account is `docs/quirks.md` under *A descriptor opened
before a turn began is inside no boundary*, with the same argument beside the
code in `crates/alo-bounding-kernel/src/deciding.rs`, where an auditor of the
crate reads in `crates/alo-bounding/src/lib.rs`, and beside the descriptor the
mechanism itself depends on in `crates/alo-bounding/src/turns.rs`. Five rows,
each naming what a turn inherits, what it permits inside the boundary, what it
still does not permit, the test that reproduces it, and v0.5 as the release that
owns closing it.

**A test could be written honestly, and it uses the production door.** Its two
siblings bind a child process; inheritance is exactly what a child gets
differently, so `what_a_turn_inherits.rs` goes through `Turns::doing` on the
thread the assertions are made from — which is what `alo-agentd` really does.
Four rows are reproduced there and the fifth was already reproduced by task 7.
`what_a_turn_inherits_is_written_down.rs` parses the table and fails the day
`file_permission` or `file_receive` lands while the entry still says neither has,
a row stops saying what it cannot do, a row names a release `docs/features.md`
has never heard of, a row's named reproduction is missing or silent about it, or
the list of what a turn inherits changes without a person looking at it.

**What the audit found that the task did not ask for, and it is the sharpest
thing in this plan.** Every unwatched mutation in task 5 was measured against one
promise — *no unwatched mutation moves a byte of somebody's file past a grant* —
and each keeps it. **An inherited read descriptor does not.** Measured here: the
same thread, in the same instant, is refused `open` on a file nobody granted and
reads every byte of it through a descriptor that already existed, then writes
what it read into the folder somebody *did* grant, where an `archive_folder` or a
`move_file` carries it onwards and where the record names only a granted path.
That is the whole of what the boundary exists to prevent, so this is its own
piece of work rather than a seventh row in that table.

The floor under it is measured beside it: a descriptor cannot be reopened by
name, `/proc/self/fd/<n>` does not turn one back into an open — with the
*granted* file reopened the same way as the control, because a boundary refusing
everything under `/proc` would look identical and mean nothing — and `openat`
relative to an inherited folder is an open like any other.

**The approval line was not crossed and the decision is named rather than
taken.** Closing this in the kernel means `file_permission`, a walk on every read
and write on the machine, which is the opposite direction from *decides and
forgets* — **and it would refuse a turn its own way out**, because leaving a
boundary is a write to `home/cgroup.threads`, opened before the first turn ever
ran. That is measured here too: the same turn is refused `open` on that file and
leaves through the descriptor anyway. The other answer is to make a turn a
process of its own, which is a change to what a turn *is*, collides with law 2's
*nothing is started*, and belongs in an ADR. **Neither was built and no ADR was
written**; `docs/autonomy/updates/descriptors-opened-before-a-turn.md` states the
decision the next scheduler of v0.5 has to take.

Nothing was closed, nothing was ticked, and no *On the machine* box is touched.

### 7. Hardening publication, and the egress coverage audit

**Status:** done. **Depends on:** 4 — done.

Two things the network integration left, and the first is about this
workstream's own machinery rather than about the kernel.

**Publication had a hole that was not in the code.** On 2026-09-07 this
workstream gated a combined tree and pushed in one shell line, the two joined by
a newline rather than by a check of the first command's result. The gates had
failed — the machine had lost its BPF filesystem between runs — and the push
went out anyway. The change was sound; the sequence was not. The supervisor's
own path was already fail-closed, so what had to go was the hand-rolled one
beside it.

**And the coverage assessment was argued rather than reproduced.** UDP without a
connection, sockets already open or inherited, and the loopback proxy were named
in reports and in `deciding.rs` and demonstrated nowhere.

- **Acceptance:** every publication path — the loop's and a person's — fails
  closed on a failed readiness check, a failed gate, a failed acceptance test or
  a failed combined-tree check, with the work preserved; each of those refusals
  has a regression test; and each of the three egress gaps is reproduced against
  the real loaded programme with a control proving the boundary was in force.
- **Evidence:** `tools/kernel-loop`'s own suite, and
  `crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`.

**Done, 2026-09-08.** `publish` and `verify` are subcommands, so there is no
hand-rolled publication path left; a person recovering by hand walks the same
checks in the same order as the loop, because it is the same code. Readiness
grew from one check to three. Acceptance evidence is read from cargo's result
line rather than from its exit code, because a test name that matches nothing
reports zero tests passing **and exits successfully**. A worker that exits
unsuccessfully is a task nobody has done, whatever it left behind.

The three gaps are reproduced, each with a destination the kernel really refuses
as its control. Their release placement is in
`docs/autonomy/updates/publication-hardening-and-egress-coverage.md`, along with
the two decisions this workstream is **not** taking without an answer: whether
anything watches a socket after it is opened, and whether egress enforcement may
stop being turn-scoped.

**Task 6 is not closed by this.** The socket half of *what a turn inherits* is
reproduced here; the descriptor half is still audit-and-document work.

### 8. One kernel, two checkouts

**Status:** done. **Depends on:** nothing.

Two checkouts run kernel tests against one WSL kernel. Every test binary
serialised within itself and nothing serialised them across processes, so a gate
run in one checkout failed five tests in `alo-agentd` while the other was running
its own suite — on a tree where only documents had changed. A failure that
reruns clean is the worst shape there is, because rerunning is what people do.

- **Acceptance:** one lock, taken by every test entry point that touches this
  kernel in either checkout; a second process refused while it is held; entry
  after a normal release; entry after the holder is killed; a bounded wait that
  fails without touching anything; and existing enforcement still passing under
  it.
- **Evidence:** `crates/alo-bounding/tests/two_processes_take_turns_on_this_kernel.rs`
  and `alo_bounding::waiting`'s own unit tests.
- **Constraint:** no production path may call it, and a wait that runs out must
  refuse rather than proceed.

**Done, 2026-09-08.** `alo_bounding::waiting` — an **abstract Unix socket name**,
`alo-os/one-kernel-at-a-time`, which belongs to the machine rather than to a
checkout and which **the kernel frees the instant the process holding it dies**.
That last property is the reason it is not a lock file: a file a crashed holder
leaves behind can only be cleared by deleting a lock somebody might still hold,
which is the one thing this workstream may never do.

Taken by `on_this_kernel::one_at_a_time()` — which covers twelve `alo-bounding`
test files — and by `alo-agentd`'s two kernel tests and `alo-boundaryd`'s three.
In-process mutex first, machine second, so two threads of one binary cannot make
each other wait five minutes.

**The deadlock audit is in the module.** Eleven test files spawn a child of the
same binary; every child runs an `#[ignore]`d helper and none takes the lock,
because the parent holds it across the spawn. That invariant is documented and
the timeout message names it, since a process cannot ask whether an ancestor
holds a socket name. The lib's own unit tests were audited and take nothing:
they test names refused *before* anything is made.

Nothing was closed, nothing was ticked, no production path calls it, and no
*On the machine* box is touched.

### 9. What a person is told about where their question was answered

**Status:** done, as documentation and behavioural coverage. **The decision it
asks for is not taken.** **Depends on:** nothing.

The owner clarified on 2026-09-08 that model choice belongs to the person —
alo's models, the person's own runtimes and third-party APIs are all legitimate,
and alo ownership is never a condition of being one. The same clarification says
a loopback address establishes where a service is *contacted*, not where it
performs inference.

- **Acceptance:** the proposal separates the five things that are not the same
  thing; ordinary use of owner-configured local services and third-party APIs is
  preserved by every option; the `ThisMachineOnly` question is presented as
  options with consequences rather than answered; and the gap is covered by a
  behavioural test through the production door rather than argued.
- **Evidence:** `crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs`.
- **Constraint:** no enforcement, no label, no rule and no promise changed.

**Done, 2026-09-08.** The test copies every assertion from the honest local
service in `a_day_that_never_left.rs` and makes them against a service at
`127.0.0.1` that forwards to a listener on this machine's own interface. All of
them still pass while the far service reports holding the question — which is the
category error in one run: `Served::source()` answers *where processing occurs*
with a fact about *what alo can verify*. A transparent relay is not a redirect,
so the existing redirect guarantee does not touch it.

**Nothing was implemented and nothing was decided.** ADR 0021 stays PROPOSED, the
rule behaves exactly as before, `docs/features.md` was not touched, and the
recommendation is explicitly not an approval.

### 10. What a credential does when a session really ends

**Status:** scheduled — the three cases are measured; the held-handle half
below is what remains, and it waits on a network service this machine does not
run (see below). Stepped over until then. **Depends on:** the credential store — done.

**Measured, 2026-09-10.** All three cases observed on this machine's `logind`,
in `crates/alo-secrets/tests/a_session_that_really_ended.rs`: one login logged
out takes `/run/user/<uid>` and the bus with it and `TheBus::found` is
`Unavailable`; with `enable-linger` on both survive with nobody signed in and it
is `Ok`; with two logins, ending one leaves both where they are. Report:
`docs/autonomy/updates/a-session-that-really-ended.md`, and ADR 0022 carries the
table.

**Two premises in the text below turned out to be wrong**, and they are why this
sat unscheduled. **There is no `sshd` on this machine** — the binary is absent
and the unit is `not-found` — so every case as written was blocked on installing
a network service; `su` reaches the same sessions through `pam_systemd` with no
listener at all. And `loginctl terminate-user` is not a logout: it removes the
user manager too, which is precisely what lingering exists to prevent, so using
it would have answered case 2 by definition instead of measuring it.

**What remains, and why it is still scheduled.** A keyring **handle held across**
the logout. A Secret Service on the person's *session* bus needs a process
running as them — root does not complete the D-Bus handshake there, measured —
and such a process is killed by the logout being measured, so it needs a helper
that reports across the event. That is a piece of work of its own, and it still
wants a quiet machine.

**A correction, because the line above used to say *blocked on a machine with
`logind`*, and that was wrong.** It was written without looking. Asked directly,
this machine answers `systemctl is-system-running` → **running**,
`systemd-logind` → **active**, `loginctl list-sessions` → **two sessions**.
There is a `logind` here.

What is actually unsettled is narrower: those two are WSL's own `user-early` and
`manager-early` classes rather than ordinary logins, and `Linger` is `no`. So the
first thing this task establishes is whether a session of the kind a person
really has can be started and ended here at all — and only then whether the three
cases below can be observed. It is scheduled rather than blocked, and neither
word may be used to imply the work was done.

`connections_come_and_go.rs` proves **disconnection handling**: a keyring handle
whose bus has stopped refuses, promptly, and hands back no key. Its fixture stops
the private bus and keyring daemon **that the fixture itself started**, and that
is all it stops. Real logout is a different event and this workstream has not
tested it.

**Two seats are not the premise, and assuming they were is what kept this
unscheduled.** Three cases, each reachable differently and only one of them
needing a second seat at all:

1. **One session, ended.** A single login, logged out, no lingering. `logind`
   removes `/run/user/<uid>` and the user bus with it. Reachable with **one
   login over `ssh`**, logging out and looking. *Expected:* the bus is gone and
   `TheBus::of_this_process` is `Unavailable` for anything started afterwards;
   a handle held across the logout refuses. This is nearest to what the fixture
   already shows, and is the one that would make the fixture's result
   representative.
2. **Logged out while lingering is on.** `loginctl enable-linger <user>` and the
   user manager — and its bus — **survives with no session at all**. Reachable
   with **one login**, and needs no second seat. *Expected:* the bus is still
   there and a key is still retrievable after logout, which is the case a
   daemon's author would not predict from case 1 and the reason this is listed
   separately. Whether that is what we want is a **question for the owner**, not
   something to decide here: a credential reachable when nobody is signed in is
   a policy, not a bug to fix quietly.
3. **Two sessions, one ended.** Two concurrent logins for the same user; log out
   of one. *Expected:* the bus survives, because the other session holds it.
   Reachable with **two `ssh` logins** — still no second physical seat, though a
   console plus `ssh` is the same test if a seat is available.

- **Acceptance:** each case observed on a machine running `logind`, with what
  `/run/user/<uid>` and the bus actually do recorded per case, and the daemon's
  behaviour beside it. **`loginctl enable-linger` is a machine-wide change and
  needs its own handoff** — it is not something to turn on inside somebody
  else's test run.
- **Constraint:** not on the shared build machine while another worker is using
  it, and nothing here changes what a credential is allowed to be. **Sessions and
  lingering are not to be changed while anything else is testing** — both are
  machine-wide, and `enable-linger` has its own handoff above.
- **Not claimed until then.** ADR 0022 says so where it records the measurement,
  and the reports say so.

### 11. Load an organisation's inference policy into the daemon

**Status:** done. **Depends on:** nothing — the daemon already carried the
bound; nothing wrote one into it.

ADR 0016 gives the bound to the organisation and the choice to the person, and
`alo-agentd` has carried `TheBound` since `6cff37c`: it words a refusal, names
that an administrator set it, asks the rule before the keyring, and writes a
refused question into the record. **Every machine is `TheBound::Nobodys`**,
because `docs/contracts/machine-description.md` had no key for a policy, so the
whole path is unreachable in production. This is the key.

It is an **existing v0.01 requirement of an accepted ADR**, not new scope: no
enrolment, no identity, no reporting, no server, no fleet. One optional section
in a file that is already read once at startup from the disk it is on.

- **The contract first.** `[questions]` with `may-go` and an optional `region`,
  specified in `docs/contracts/machine-description.md` before any of it is built.
  It requires `format = 2`, and the reason is the contract's own additive rule:
  an older service would ignore the section and send questions wherever the
  person chose, which is not the same machine. A description carrying it says
  `2`; a service that reads only `1` refuses it and **does not start**, so a
  managed machine too old to understand its policy does not run unmanaged.
- **Absent stays unmanaged.** No section is `TheBound::Nobodys` and no
  administrator is named — ADR 0016's *absent*, not *permissive*. Every machine
  today, unchanged, still `format = 1`.
- **Present, valid and trusted becomes `TheBound::AnOrganisations`.** Trusted
  means the checks the file already gets in *Who may write it*: not a symlink,
  owned by root or the person the service runs as, writable by nobody else.
  Those exist and are not relaxed.
- **Present and not holding is refused, and the service does not start.** An
  unknown `may-go`, `in-a-region` with no region, a region beside another
  `may-go`, a section with no `may-go`. **Never read as unrestricted**: an
  organisation that wrote a policy and got none, with nothing saying so, is the
  one failure this must not have.
- **The person's choices are untouched.** Their model and provider stay in their
  own settings and are never rewritten from here. A bound can refuse a choice;
  it can never replace one, and no fallback follows a refusal.

- **Acceptance:** the four states above, each tested; and one end-to-end test on
  **isolated files** that loads a description naming a policy, asks a question
  the policy refuses, and shows the refusal happening **before the keyring is
  opened or any socket is touched**, then reads the **persisted** record back and
  finds the same sentence the agent was given. The refusal ordering and the
  persisted agreement both already have their mechanisms; this is them reached
  from a real configuration.
- **Evidence:** those tests, and a report naming what is still not covered.
- **Constraint:** **this machine's own `/etc/alo/agentd.toml` is not touched.**
  Every test writes its own description in a temporary directory and points the
  code at it. Nothing here starts, stops or reconfigures a service on this
  machine, and no organisation's real configuration exists here to alter.
- **Not in scope:** anything that would make this fleet management — a server, an
  identity, a report, a key that names one. The section is read from the disk it
  is on, as the rest of that file always has been.

**Done, 2026-09-09.** `docs/contracts/machine-description.md` gained `[questions]`
and the `format = 2` decision; `alo-agentd`'s reader gained the section, the four
refusals and `ALSO_READ`, so `1` is still read; `Described` carries a `TheBound`
and `crate::starting` hands the description's to `Questions`. The attribution
rests on **who owns the description** — root is an organisation's configuration
system, the person is their own — which `crate::trusting` already established
before parsing and now carries out rather than discards. Report:
`docs/autonomy/updates/an-organisations-rule-off-a-disk.md`.

**What it does not cover**, and the report says so rather than implying
otherwise: the one expression in `starting::until_stopped` that hands the bound
over is in the same untestable class as `Questions::of_this_process` reading the
process environment — this crate sets no environment variables in tests by
policy, so every question test drives `of_a_session`. The `ThePersons` origin is
proved in the reader's own tests; the disk-level tests take their expectation
from who really owns the file, which under a suite running as root is an
administrator.

### 12. A descriptor opened before the turn began cannot move contents past the grant

**Status:** ready. **Depends on:** nothing.

**Why now:** ADR 0028 — v0.5's screenless work begins while v0.01 waits on
hardware; this is lane A's partition (`alo-bounding`, `alo-bounding-kernel`, `alo-bounding-map`,
`alo-boundaryd`, `alo-egress`, `alo-turn`).

The hardening table above names one gap that **moves bytes past a grant** and
is reproduced: a file descriptor opened before the turn began is still readable
and writable inside it, because the boundary is applied at `open` and a
descriptor that was never opened inside the turn was never asked. It is the
gap `docs/features.md`'s v0.5 sentence is about — *for the length of one turn,
everything outside the grant is unreachable* — and today that sentence is
untrue of an inherited descriptor.

- **Acceptance:** the reproduction that shows contents crossing the grant
  through a pre-opened descriptor flips from *reaches* to *refused*, in the same
  test file, with the refusal named; a read or write through such a descriptor
  fails at the syscall, not after a byte has moved; a descriptor to a path
  **inside** the grant is untouched; the loader's capability set is unchanged
  and `crates/alo-image` still holds it to two; and the record carries the
  refusal the way it carries every other, in words `alo-saying` collects.
- **Constraint:** no kernel patch and no fork (ADR 0015). If the honest
  mechanism is Landlock (ADR 0013's file primitive) rather than another hook on
  the BPF LSM, that is this task's finding and its report says which and why;
  it does not add a second privileged component to do it.

**Done, 2026-09-12.** A seventh hook, `file_permission`, in
`crates/alo-bounding-kernel/src/deciding.rs` as `decide_use`. It runs on every
read and write on the machine, asks the **using thread's** control group, reads
the file's kind from `i_mode` — the fourteenth offset in `FIELDS`, and the two
maps are still the two — and steps aside for a socket, which `socket_sendmsg`
decides about by destination; for anything else it walks up from the file's own
directory entry exactly as an open does, so a descriptor to a place inside the
grant is untouched and one to anywhere else is refused with `EACCES` before a
byte has moved. Every reproduction in `what_a_turn_inherits.rs` flipped from
*reaches* to *refused* in the same file, each beside the use inside the grant it
must not break: the invoice's descriptor read and written through, the granted
folder listed through a handle, a pair of Unix sockets written across from
inside. **The way out of a turn is the fourth row closed rather than
exempted**: a turn's thread can no longer write itself out through the
inherited `home/cgroup.threads`, so `inside.rs` starts a thread of the service
beside the turn — in `home`, before the turn's thread goes in — that writes the
turn's thread number on its behalf when the work is over; nothing is started
and `a_turn_is_this_thread.rs` still holds the crate to law 2. **Landlock is
not the mechanism** and the report says why: it decides at open, which is the
hook this crate already had, and a ruleset is irrevocable for the thread it is
applied to, which a turn that is a thread cannot afford. **Not closed, and
named**: `mmap_file` — a mapping of an inherited descriptor made inside the
turn — because there is no safe `mmap` in Rust, so the committed suite cannot
reproduce it, and a hook nobody can show refusing was not added on belief; the
quirks entry and the hardening table above say so. The loader's capability set
is unchanged and `crates/alo-image` still holds it to two. `alo-files` holds
that a refusal at read is the record's sentence for a refusal at open, and
`the_boundary_decides_and_forgets.rs` reads and writes outside a turn beside its
opens and still finds nothing written down. Report:
`docs/autonomy/updates/descriptors-inside-the-boundary.md`.

### 13. A socket already open, and a datagram sent without connecting, are inside the boundary

**Status:** ready. **Depends on:** nothing.

Two reproduced gaps in `what_a_bound_turn_can_still_reach.rs`, one class: the
network boundary is applied at `connect`, so a socket inherited or opened
before the turn, and a `sendto` on an unconnected datagram socket, reach no
hook. Nothing shipped does either from inside a turn, which is why they are
documented rather than urgent — and why they belong to v0.5 rather than to a
patch release.

- **Acceptance:** both reproductions flip to *refused* in the same file; a
  datagram to a destination inside the grant still goes; `alo-egress`'s
  accounting is unchanged — the refusal is a kernel refusal, not an egress
  event; nothing leaves the machine in the test, which runs against a socket of
  the test's own; and the refusals are recorded in the same words as a refused
  `connect`.
- **Constraint:** the hooks are on the turn's own cgroup, as ADR 0013 requires,
  and a syscall outside a turn is checked and leaves no trace — held by the
  test the v0.5 promise says must exist rather than by a sentence.

**Done, 2026-09-12.** A sixth hook, `socket_sendmsg`, in
`crates/alo-bounding-kernel/src/deciding.rs` as `decide_message`. It runs on
every message the machine sends, asks the **sending thread's** control group —
which is what closes the inherited case rather than restating it, and why an
LSM hook rather than a cgroup `skb` programme, which attributes a socket to the
cgroup it was *made* in — and reads both places a message can be going: the
address it names and the peer the socket is joined to, checking both when both
are there. Six offsets more in the existing `FIELDS` map, reached through a
dotted path the type-information reader now follows through named members;
no third map, and `the_program_has_nowhere_to_write_what_it_sees` still
asserts exactly `["BOUNDS", "FIELDS"]`, unchanged.

Both reproductions flipped in `what_a_bound_turn_can_still_reach.rs`, with
their controls: an inherited connection to a shown destination carries on, a
datagram to a shown destination goes, a datagram to loopback goes unshown, a
process that is not a turn sends the refused datagram and it arrives, and a
Unix socket is written on. The proxy on loopback is reproduced as it was. The
connection-reuse gap the connect hook named is closed by the same mechanism.
`the_boundary_decides_and_forgets.rs` sends datagrams outside a turn beside its
opens and still finds nothing written. `alo-asking` holds that a refused
message and a refused connection are the same sentence in the record.
`alo-egress` is untouched. **Nothing that would have needed approval
happened**: no exemption, because the way out of a turn is a file and the
daemon's socket to the person is not egress. Report:
`docs/autonomy/updates/sockets-and-datagrams-inside-the-boundary.md`.

### 14. Attributes, ownership and size are inside the grant

**Status:** ready. **Depends on:** nothing.

`inode_setattr` and `inode_setxattr` are not hooked, and `truncate(2)` reaches
`inode_setattr` without an `open` — so a bound turn can shorten a file it may
not read, change its mode, or change its owner, none of which moves a byte and
all of which change what a person has. The table calls it out by name.

- **Acceptance:** a `truncate`, `chmod`, `chown` or `setxattr` on a path
  outside the grant is refused at the syscall inside a turn; the same on a path
  inside the grant succeeds; a reproduction for each is in the crate before the
  hook, so the hook is shown to close it rather than believed to; and the four
  refusals are one sentence in `alo-saying`'s vocabulary, not four.
- **Constraint:** nothing outside a turn is affected or observed, and the test
  that holds that is run for these hooks too.

### 15. A turn whose boundary cannot be applied does not run

**Status:** ready. **Depends on:** nothing.

`docs/features.md`, v0.5: *a turn whose boundary cannot be applied does not
run — a refusal, not a warning, the same rule `alo-egress` already follows when
a policy cannot be evaluated.* Today a machine whose loader is absent, whose
pins were refused, or whose map is missing runs the turn under the daemon's own
rules and says nothing, which is the audit-log world the whole workstream
exists to leave.

- **Acceptance:** when the boundary is not in place — loader not running, map
  not pinned, programme not attached — a turn is refused before its first verb,
  in words that say the boundary was not there rather than that the person did
  something wrong; the refusal is recorded; a machine where the boundary *is*
  in place is unaffected; and the check is the kernel's own state (the pinned
  map, the attached programme), never a flag the daemon set for itself.
- **Constraint:** no *degraded mode*, no environment variable that permits a
  turn without a boundary — that is the warning the promise refuses. A
  development machine that cannot load the boundary gets the same refusal and a
  sentence pointing at `docs/quirks.md`.

### 16. Kernel-sourced enforcement records — the decision

**Status:** ready. **Depends on:** nothing.

The table says *needs a decision, not code*, and it is right: *what a turn
touched is what the kernel watched it touch* means the record's source changes
from the daemon's honest account of itself to the kernel's observations, and
that is a decision about the record's shape, its size, and what a person is
told. This task is the ADR, in the shape ADR 0024 and ADR 0025 used — options,
a recommendation, consequences — handed over as the task, with the code waiting
on it.

- **Acceptance:** an ADR under `docs/decisions/` (pull and list first; the
  number is taken from the tree as published, not from memory) setting out at
  least: the kernel emitting records the daemon appends; the daemon's account
  kept, with the kernel's refusals added beside it; and a hybrid where the
  kernel's observation is the record and the daemon's account is a claim shown
  as one. It names what each costs a person reading *what did the agent do*,
  what each costs the record file (`docs/contracts/record-file.md`, additive
  only), and what each costs the loader. It recommends one.
- **Constraint:** no code beyond a test that the ADR exists and is pointed at
  by this plan. The decision is the owner's or the delegate's; the worker's job
  is to make it decidable.

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
