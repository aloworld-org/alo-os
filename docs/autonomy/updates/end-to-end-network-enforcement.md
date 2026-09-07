# End-to-end network enforcement

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-turn`, `alo-asking`, `alo-agentd`,
  `alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: End-to-end network enforcement integration
- Status: **done** — the production request path runs inside the kernel
  boundary, verified against the real loaded programme on real sockets
- Decision: [ADR 0020](../../decisions/0020-a-question-is-carried-out-inside-the-turns-boundary.md),
  written before implementation, on the owner's approval in
  `network-request-boundary-approval.md` (`d220aef`)

This report replaces the audit published under the same name. That audit's
finding was that **no provider request on this machine was subject to the
destination enforcement**. It is now closed, and everything below is the
evidence, the coverage assessment and what remains uncovered.

## What changed, and why

The mechanism decided by control group; a provider request was made from a
thread in no control group, so the programme saw every one as *not a turn* — the
answer that allows everything. Four changes, in the order they matter.

**1. A question is carried out inside a boundary.** `Bounding` gained
`carrying_out_a_departure`, beside the method that carries a file verb.
`Turning::asking` now enters it around the provider call. The bound is
`Bounds::reaching_nothing_but(destinations)`: a question names no path, so a
turn putting one may open **nothing** on the disk while it does — the boundary
is narrower than the turn's own, not wider.

The method is **additive and defaulted to a refusal**. An implementation written
before ADR 0020 does not send unbounded; it does not send. `alo-turn`'s own rule
— no library here ships a `Bounding` that bounds nothing — is why the default is
a refusal rather than a passthrough, and it is why this needed no version bump.

**2. Resolution is separated from connection.** `alo_models::address::
where_it_connects` answers *what to look up* — host and port, from the file that
already knows how an authority is written and has the tests for the ways one can
be made to look like something it is not. `alo-turn` resolves it **before** the
boundary is entered, by the daemon, which is not a turn. What comes back is
registered in the map entry, and the request is made with a resolver that
returns *those addresses and no others*.

So **there is no DNS inside the boundary and no exception for it.** A turn does
not resolve; it connects to what was resolved for it. That is the narrowest
answer available and it is why *what may a bounded turn ask a name server* never
has to be answered — there is no blanket networking exception to widen later.

**3. The connection is request-scoped.** `alo-asking` builds its `Agent` for one
request and drops it, so the connection pool cannot outlive the permission. A
pooled connection surviving the turn would have been authority outliving what
the person was shown, which is the exact failure the destination binding exists
to prevent.

**4. Original-hostname TLS is preserved.** Only the *resolver* is replaced. The
connector is the client's own, the URI keeps the provider's hostname, and
certificate verification is against that hostname exactly as before.
Substituting the address into the URL would have connected to the right place
and verified the wrong name; a resolver is the correct seam and this uses it.
`max_redirects(0)` and the key handling are untouched, so a redirect still
cannot move a request to an address nobody registered.

## The request path, retraced

The same table as the audit, with the column that was wrong.

| Step | Where | Inside the boundary |
|---|---|---|
| policy | `alo_answering::Answering::chosen`, `alo_egress::EgressPolicy` | no — and deliberately: a refusal must cost no socket |
| indicator | `alo_egress::Indicator::beginning` → `Departing` | no |
| **address resolution** | **`alo_turn::asking::registering`, by the daemon** | **no — by design (ADR 0020 §2)** |
| **permission registration** | **`ByTheKernel::carrying_out_a_departure` → `Bounds::reaching_nothing_but`** | **the entry itself** |
| **connection** | `alo_asking::openai::put` → `ureq` with a fixed resolver | **yes** |
| response / error | same call | **yes** |
| record | `alo_record::Entry::left`, written by `alo_keeping::Writing` | no — the boundary is left first |
| **permission withdrawal** | **the same `Turns::doing` that made it** | — |

Registration and withdrawal are the same call, which is what makes the lifetime
claim checkable rather than a discipline: the control group is made, the entry
written, the thread moved in, the request made, the thread moved out, the entry
removed and the control group taken away, on success, on failure, on a panic
inside, and on a turn that ends without one.

## Acceptance, and the actual verification

`crates/alo-agentd/tests/a_question_is_bounded_by_the_kernel.rs`, five tests,
against the **real loaded BPF LSM** on a running kernel, on **real sockets**.
Not `socket_connect` unit tests, and no claim is made from those alone.

| Acceptance criterion | Test | Result |
|---|---|---|
| An authorised request succeeds through the production path | `a_question_this_service_puts_to_a_provider_answers_inside_the_boundary` | **pass** — `Turning::asking` under a real `ByTheKernel`; the answer text and source come back, and the server asserts it received the question *and* an `authorization:` header |
| The production path reaches a destination the kernel really rules on | `a_provider_at_an_address_the_kernel_decides_about_is_connected_to` | **pass** — the server on this machine's `eth0` address accepted the connection the production path made |
| An unauthorised destination fails while an authorised one stays usable | `inside_a_bounded_request_only_the_registered_address_can_be_reached` | **pass** — two servers, one registered: registered reachable, unregistered `EACCES` (13), inside the same boundary |
| Policy refusal causes no outbound request | `a_question_the_rule_refuses_never_reaches_the_server` | **pass** — `SourcePolicy::ThisMachineOnly`; `nothing_left()`, the server saw nothing, the indicator is quiet |
| Failure leaves no stale permission or connection | `a_request_that_fails_leaves_the_kernel_holding_nothing` | **pass** — a failed request, then a second bounded request registering a different address is refused the first one; the kernel is asked rather than believed |
| A question is bounded at all, and only to what it resolved | `alo-turn` `a_file_verb_and_a_question_are_both_carried_out_inside_a_boundary` | **pass** — the audit's counting test, inverted, as it was written to be |
| Local-model operation still works | `alo-asking`, `alo-turn` suites | **pass** — loopback is unchecked (ADR 0007) and `Served` is unchanged |
| Existing filesystem enforcement still passes | the four hooks' suites and `a_turn_is_bounded_by_the_kernel.rs` | **pass** |

### Two addresses, because they prove different things

Stated plainly because it is the thing a reader should check first.

Loopback is deliberately unchecked, so a request to `127.0.0.1` proves the
**path** — resolve, register, enter, connect, answer — and proves nothing about
the kernel's decision. `Provider::checked` carries a key over unencrypted
`http://` only to this machine, which is why the test that reads a whole answer
back is the loopback one.

So the destination the kernel actually rules on is covered separately, at **this
machine's own address on `eth0`**, non-loopback, reached over a real interface
and never leaving the virtual machine: a registered address is reached, an
unregistered one is refused with `EACCES`, and a provider at that address is
connected to *through the production path* — the connection being accepted is
the registration having been honoured where the machine was deciding. Between
them: the path works, and the machine is deciding. Neither test is asked to
prove the other's half.

**Documentation-range addresses are not used as isolation.** Everything binds a
port the operating system chose on an address the machine already has, and gives
it back. Nothing changes host-wide networking; nothing reaches a network.

## Coverage — what production can reach, assessed

| | Production-reachable? | State after this change |
|---|---|---|
| **A provider request** | **yes — the ordinary path** | **Enforced.** Destination-bound, request-scoped, refused everywhere else |
| **DNS** | yes, by the daemon | **Defined, not excepted.** It happens outside any boundary, before entry. A bounded turn asks no name server, so there is nothing to permit |
| **UDP `sendto`** | yes in principle | **Not enforced.** An unconnected datagram never reaches `socket_connect`; it needs `socket_sendmsg`, a hook on every message rather than every connection. Nothing on the production path sends one — the request is TCP and resolution is not in a turn — but the hook gap is real and is not closed |
| UDP with `connect` | yes | Enforced — same hook |
| IPv4 / IPv6 | yes | **Both enforced**, and kept apart: `::ffff:1.2.3.4` and `1.2.3.4` are different destinations, and the map's `family` field means one cannot stand for the other |
| **Connection pooling** | **no longer** | **Closed for the production path.** The `Agent` is built per request and dropped with it, so no connection survives to be reused past its permission. A pooled connection would still not be re-checked by the hook — that is unchanged kernel behaviour, and it now has nothing to act on |
| Redirects | yes | **Prevented**, unchanged: `max_redirects(0)`, and a redirect becomes `WentWrong::SentSomewhereElse`. A redirect cannot move a request to an unregistered address |
| **Sockets already open or inherited** | yes — the daemon's own record and socket | **Not enforced.** `socket_connect` decides when a connection is made; a socket that already exists is invisible to it. Same class as the already-open-descriptor gap (plan task 6) |
| **Loopback proxy** | yes | **Not enforced**, unchanged and deliberate. A proxy on `127.0.0.1` forwarding off the machine is this machine to every type in this repository. `docs/quirks.md` carries it. Closing it needs enforcement that is not turn-scoped, which is a different decision and was explicitly outside this approval |
| **More than two resolved addresses** | yes | **A real limitation.** One entry holds `DESTINATIONS = 2`, so a provider resolving to more is registered — and reachable — at the first two. Registering more than can be written would be registering what is not enforced, so the resolution is cut to what the bound holds |

## What this does not claim

- **Not release completion.** *Mechanism implemented* is not *release
  requirement verified*, and the v0.01 *Egress indicator* line's remaining
  clause covers more than this: the compositor surface, the daemon code that
  signs somebody in and fetches a model, and physical acceptance.
- **No hardware acceptance.** Every measurement here is Ubuntu on WSL2, kernel
  6.18.33.2. `docs/hardware.md` says that cannot certify a machine. **WSL
  success is development evidence and never certified-hardware acceptance**, and
  no *On the machine* box is affected by anything in this report.
- **No audit record from the kernel.** *Decides and forgets* is untouched. The
  programme still has exactly two maps and
  `the_program_has_nowhere_to_write_what_it_sees` still asserts
  `["BOUNDS", "FIELDS"]` exactly, unchanged. Cgroup attribution is not an audit
  record and nothing here says it is.
- **No widening.** No new agent capability, no widened grant, no blanket network
  permission, no unsafe exemption, no relaxed gate, no engine patch.

## The development loop now inspects acceptance evidence

Published in the same change, because the gap it closes is the one this task
would otherwise have demonstrated.

The loop selected a task, launched one worker, gated the tree and published what
passed. **The gates are the state of everything except the thing just written**:
a worker that produced a report and no enforcement passes every one of them,
because the suite they pass is the suite that was already there. Treating that
as completed implementation is precisely what this workstream's rules forbid,
and the loop had no way not to.

So `tools/kernel-loop/src/evidence.rs`: a handoff now carries an `evidence`
block — one line per acceptance criterion, naming the crate, the test target and
the test's full name — and the loop holds each one up before it stages anything.
Two checks, and together they are hard to satisfy without having done the work.

- **The test has to be part of the change.** Its file must be among the files
  the handoff publishes. A test that was already green cannot be offered as
  evidence of what was just written, because that test's file is not in the
  change.
- **The test has to have run, by that name.** It is run again on its own with
  `--exact`, and the result must be *one* test passing. Verified with the real
  command the check builds: the acceptance test's real name gives
  `test result: ok. 1 passed`, and a name with a typo in it gives
  `running 0 tests … 0 passed` **and exit status zero** — which the
  whole-workspace run cannot tell from success, and which an exit-code check
  alone would wave through.

A handoff with no evidence is refused outright, and a blocked or partial worker
writes no handoff at all, so neither publishes. Both gatings — this task's tree,
and the combined tree after integrating whatever arrived on `main` — are
followed by the evidence check; the loop's own tests cover the parsing and both
refusals.

**What it deliberately does not do is judge whether a test is any good.**
Nothing mechanical can, and a supervisor that claimed to would be a worse lie
than the gap. The report is read by a person and the plan names what each task
must show. What is removed is the failure that needs no bad faith at all: a task
published on a green suite that never contained a test of it.

The loop was **not run against this change**. It launches a worker in this
checkout, and a second editor in a working tree somebody is editing is what
`CLAUDE.md` forbids outright; this task's gating, integration and push were done
by the same commands the loop runs, by hand.

## Verification run

Ubuntu on WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`:
`cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D
warnings`; `cargo test --workspace` (every suite green, doctests included);
`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`; and the BPF
target's `fmt --check` and `clippy --release --target bpfel-unknown-none -Z
build-std=core -- -D warnings` on the pinned `nightly-2026-06-01`.

The kernel tests take a lock and give their control-group subtree back inside
it: `Turns::of_this_service` makes one subtree beside where *this process* is
and moves the process into it, so two of them in one test binary are two
attempts at the same directory. One service, one subtree — the same arrangement
a real machine has.

**Windows:** `alo-turn`, `alo-asking` and `alo-models` are portable and their
tests compile and run there; the kernel crates do not, and no Windows run is
offered as enforcement evidence.

## Proposed shared-document updates

Not made here — the integration worker consolidates them.

**CHANGELOG.md** — a user-visible line is now warranted: *a question alo puts to
a provider is carried out inside the same kernel boundary as its file work, and
may reach only the addresses that question resolved to.*

**ROADMAP.md** — the correction the audit asked for can be **withdrawn**. That
report said the *Egress indicator* line's "enforcement at the network boundary"
clause had to stay because nothing on the production path reached the
enforcement. It does now. The clause should still not be struck: the same line
also names the compositor surface and the daemon code that signs somebody in and
fetches a model, and physical acceptance is pending regardless.

**docs/autonomy/QUEUE.md** — no new item; this workstream's plan is its list.

**docs/autonomy/STATE.md** — the fact worth carrying: the kernel's destination
enforcement now applies to real provider requests, verified end to end against
the loaded programme; what remains uncovered is UDP without a connection,
sockets already open or inherited, the loopback proxy, and a provider resolving
to more than two addresses.
