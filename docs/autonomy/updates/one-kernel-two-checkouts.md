# One kernel, two checkouts

- Date: 2026-09-08
- Workstream: kernel enforcement (`alo-bounding`, `alo-agentd`, `alo-boundaryd`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Prevent kernel tests from interfering across the two development
  checkouts
- Status: **done** — implemented, proved across real processes, and every
  existing kernel suite still passes under it
- Carries with it: [ADR 0021](../../decisions/0021-what-a-service-on-this-machine-vouches-for.md),
  **PROPOSED and revised**, and the report it belongs to. **Publication is not
  acceptance.**

## Part one — the lock

### What was wrong

Two checkouts on this machine run kernel tests against **one** WSL kernel. Every
test binary already serialised within itself — a `Mutex` in `alo-agentd`'s tests,
`one_at_a_time()` in `alo-bounding`'s — and **nothing serialised them across
processes**.

Measured on 2026-09-08: a gate run in `C:\dev\alo-os-claude` failed all five
tests in `a_question_is_bounded_by_the_kernel.rs` on a tree where only documents
had changed, while `C:\dev\alo-os`'s supervisor was running its own
`cargo test --workspace`, with a pin belonging to one of its live test processes
in `/sys/fs/bpf`. Both suites passed when run apart.

A failure that reruns clean is the worst shape a failure takes: rerunning fixes
it, so people rerun, and the next one gets ignored too.

### What was built

`crates/alo-bounding/src/waiting.rs` — `alo_bounding::Waited`, a lock taken by
tests and **by no production path**.

**It is an abstract Unix socket name**, `alo-os/one-kernel-at-a-time`, and the
two properties that matters for are the whole design:

- **It belongs to the machine, not to a checkout.** Both checkouts run the same
  source and bind the same name, so a test in either waits for a test in the
  other. A path under a checkout could not do that, and neither could a lock the
  supervisor took, because tests are also run by hand.
- **The kernel frees it the instant the holder dies**, however it dies. That is
  why it is not a lock file: a file a crashed holder leaves behind can only be
  cleared by deleting a lock somebody might still be holding — the one thing this
  workstream may never do. There is no stale state, no liveness check and nothing
  to delete.

A note at `/run/alo/one-kernel.holder` says who has it. **Diagnostics only** —
best effort, possibly stale, never consulted to decide anything, and never
written or removed by a test using a name of its own.

### Where it is taken

| Entry point | Covers |
|---|---|
| `on_this_kernel::one_at_a_time()` | **twelve** `alo-bounding` test files, unchanged at every call site — the guard's type changed, and `let _order = …` did not |
| `a_question_is_bounded_by_the_kernel.rs` | `alo-agentd`, five tests |
| `a_turn_is_bounded_by_the_kernel.rs` | `alo-agentd` — **had no lock at all** |
| `the_boundary_outlives_the_loader.rs` | `alo-boundaryd`, three loader tests — **had no lock at all**, and these attach programmes |

**In-process mutex first, machine second, never the other way round.** Two
threads of one binary that each took the socket name first would have one waiting
five minutes for its own sibling. The mutex means at most one thread per binary
contends for the machine.

This covers **both checkouts, separate test binaries, and tests run by hand**,
because it is in the source both checkouts build rather than in either
supervisor.

### The deadlock audit

Asked for, and it changed the design.

- **Eleven `alo-bounding` test files spawn a child of the same test binary** to be
  the process inside a control group. Every one of those children runs an
  `#[ignore]`d helper that joins a cgroup and touches files, and **not one takes
  the lock** — the parent holds it across the whole spawn. That is the invariant.
- It is **not enforced in code**, because a process cannot ask the kernel whether
  an ancestor holds a socket name, and the environment variable that would carry
  it cannot be set without `unsafe` in this edition. So the **timeout message
  names it**, where somebody who has just written such a child will read it:
  *if this process is a child of a test that already holds it, that is the
  deadlock `alo_bounding::waiting` documents.*
- **Nesting within one process** deadlocks too, and already did: the in-process
  `Mutex` these guards sit beside is not re-entrant either. Documented rather
  than made re-entrant, because re-entrancy would hide a second acquisition that
  is always a mistake.
- **The lib's own unit tests were audited and take nothing.** `cgroup.rs` and
  `turns.rs` test names that are refused *before* anything is made, and reading
  `/proc/self/cgroup`. The lib test binary touches no shared kernel resource, so
  adding the lock there would have been cost without cover.

### What it will not do

- **It never bypasses itself.** When the bounded wait runs out the caller
  **fails**. No pin is removed, no programme unloaded, no process signalled.
- The one process this work kills is **its own child**, in the test that proves
  a crashed holder frees the name.

## The evidence

`crates/alo-bounding/tests/two_processes_take_turns_on_this_kernel.rs` — five
tests, across **real processes**, each binding a name made from its own process
id so that proving the lock cannot disturb a real test holding the real one.

| Required | Test | Result |
|---|---|---|
| A second process cannot enter while the first holds it | `a_second_process_cannot_enter_while_the_first_holds_it` | **pass** — the child reports it gave up, its message says *Nothing was forced*, and the holder still has it afterwards |
| It can enter after normal release | `a_second_process_enters_after_the_first_gives_it_back` | **pass** — the child is waiting, the parent drops the guard, the child gets it |
| It can enter after the holder crashes | `a_second_process_enters_after_the_holder_is_killed` | **pass** — the holder is `SIGKILL`ed with no unwinding and no `Drop`, and the name is free |
| Timeout fails safely without touching another test's resources | `a_wait_that_runs_out_leaves_everything_alone` | **pass** — the bounded wait is asserted to have actually elapsed, a file standing in for another test's resources is byte-identical afterwards, and the name is still held |
| Existing kernel enforcement still passes under coordination | `the_boundary_still_refuses_while_this_machine_is_held` **plus every suite below** | **pass** |

And four unit tests in `alo_bounding::waiting` for the single-process half,
including one asserting that a test's private name leaves the machine's note
alone — the same rule as never removing another process's pins, one layer down.

**Every kernel crate, run under the coordination:** `alo-bounding`, `alo-agentd`
and `alo-boundaryd` — 25 test binaries, all green, including the five network
tests that failed during the contention that started this.

## Part two — the privacy proposal, revised and still PROPOSED

**ADR 0021 is not accepted. Nothing in it is built. Publishing it is not
accepting it.** The revision adds the third option the owner asked for and
changes the recommendation.

### Option C, evaluated — and it is two proposals wearing one name

**The distinction is real and already in the types.** The runtime alo OS ships
(Ollama, pinned by ADR 0006, behind `ModelRuntime`) reaches a turn through
`Answers::Runtime`; a service the owner configured reaches it through
`Answers::Service`. Different doors, different knowledge.

**C1 — truthful provenance.** `InferenceSource::ThisMachine.shown()` renders
*"on this machine"*, and `Served::source()` returns it unconditionally because
the address is loopback. For alo's own runtime that is true; for a service the
owner configured it is **a claim alo cannot verify**. C1 gives that case a source
of its own, rendered as *answered by a service at an address on this machine —
alo cannot verify where your question was processed*, externalised for i18n.

**C2 — restrict the alo-managed runtime's egress**, and nothing else. Attractive
mechanically: our runtime is our own unit, so it has a control group without
anybody inventing one, and a cgroup egress programme on that one cgroup touches
no unrelated application. It would cover **existing connections, inherited
sockets and unconnected datagrams**, which `socket_connect` cannot, because it
sees packets rather than `connect` calls — and it covers the service's child
processes, because a unit's cgroup contains them.

**C3 — separate downloading from answering.** `Ollama::fetch` and
`Ollama::answers` are already separate methods, but they are the same *process*,
so a cgroup cannot tell them apart. Splitting the permission means splitting the
work: **alo fetches through its own errand path and hands the blob to the
runtime**, making the runtime's inference-time egress budget exactly zero and
giving the download what an errand gets — shown, policy-checked, recorded. The
alternative, a time window in which the runtime may fetch, is **rejected**:
authority that opens for a period outlives what anybody was shown.

### What C cannot do

- **C does not close the loopback-proxy gap.** C2 restricts a different service
  entirely; a relay the person started is not the alo-managed runtime and is in
  no cgroup of ours. **The gap stays open under every option.**
- **C2 is blocked on something that does not exist**: there is no unit in the
  image running the shipped runtime, and so no cgroup to attach anything to.
- **C2 needs a second programme type and attachment point** that ADR 0018's
  loader does not have.
- **C1 has a policy consequence**: `SourcePolicy::ThisMachineOnly` currently
  permits this door *because* the source is `ThisMachine`.

### Revised recommendation

**Reject A. Take C1, which replaces Option B's wording change rather than joining
it. Permit-and-label rather than refuse under `ThisMachineOnly`. Treat C2 and C3
as separate, later work.**

C1 is stronger than B's disclosure because it tells the truth **on every answer**,
where the person is actually reading, rather than once where they configured the
service — and the v0.01 promise is about what is visible *at the moment it
happens*.

**The decision the owner must make**, in two parts:

> **1.** Does alo OS filter what the person's own processes send (A), or trust a
> service the person started (B/C)? — recommended: trust it.
>
> **2.** May an answer from a service alo cannot verify stop being described as
> *on this machine*, and may a machine set to keep questions on it still permit
> such a service, labelled rather than refused? — recommended: yes to both.

If C1 is refused, Option B's `docs/features.md` rewording becomes **required**
instead, or the v0.01 promise stays as written and stops being true.

## What is preserved

- **ADR 0020 entire** — request-scoped destinations, original-hostname TLS
  verification, the per-request client, the refusal-by-default trait method.
- **Grants, the indicator and the records** — untouched.
- **Local-model behaviour** — `Answers::Runtime` and `Answers::Service` unchanged;
  loopback still unchecked.
- **No new enforcement hook, no kernel map, no process-isolation change, no
  narrowed promise, no production security policy touched.** The lock is test
  machinery and nothing in `alo-agentd`, `alo-boundaryd` or the programme calls
  it.

## Rollout — what the desktop worker has to do

**One thing: pull `main`, and do not start a kernel-test run that predates it
while another is already going.**

The lock is in the source both checkouts build, so `C:\dev\alo-os` participates
the moment it has this commit. Until then its tests hold nothing, and a run
already in flight when this lands **will not be waiting for anything** — so a run
started here would collide with it exactly as before. There is no way to make an
already-running process take a lock it was not built with, and nothing here tries
to force one: this checkout waits, and if the wait runs out it fails rather than
proceeding.

No pins to clear, no services to restart, no configuration to change.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, own `CARGO_TARGET_DIR`, readiness checked
first. `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D
warnings`; `cargo test --workspace`; the supervisor's own tests; `RUSTDOCFLAGS="-D
warnings" cargo doc --workspace --no-deps`; and the BPF target's `fmt --check`
and `clippy --release --target bpfel-unknown-none -Z build-std=core` on the
pinned `nightly-2026-06-01`. Published through `alo-kernel-loop publish`, which
runs all of that and each named acceptance test on its own before it stages
anything.

**WSL is development evidence and never certified-hardware acceptance.** No
*On the machine* box is affected by anything here.

## Proposed shared-document updates

Not made here — the desktop integration worker owns these four files.

**CHANGELOG.md** — nothing user-visible. Test coordination and a proposal.

**ROADMAP.md** — no tick. Nothing here closes a release item.

**docs/autonomy/QUEUE.md** — no new item. ADR 0021's two questions belong to
whoever schedules decisions.

**docs/autonomy/STATE.md** — three facts. Kernel tests in both checkouts now take
one machine-wide lock, so cross-checkout interference is closed and a wait that
runs out fails rather than forcing anything. ADR 0021 is **proposed and revised**
with a third option evaluated; the recommendation is now *reject A, take C1*, and
**the loopback-proxy gap is open under every option**. And the desktop worker's
only rollout step is to pull.
