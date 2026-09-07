# End-to-end network enforcement

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-turn`, `alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: End-to-end network enforcement integration
- Status: **blocked, awaiting one decision** — audit and evidence published; the
  integration itself changes the turn lifecycle and is not started

## The finding, first

**No provider request on this machine is subject to the destination
enforcement.** The mechanism published as `386096c` is real and tested, and the
production path does not reach it.

`socket_connect` decides by control group: it asks whether the connection's
cgroup is a bound turn, and for anything else returns *allowed*, which is right
and is what keeps a person's browser out of it. The only thing that puts a
thread into a turn's control group is `Bounding::carrying_out`. Tracing where
that is called:

| Path | Enters the boundary |
|---|---|
| a file verb — `Turning::doing` → `crate::carrying` → `carrying_out` | **yes** |
| a question — `Turning::asking` → `alo_asking::Asking::to_a_provider` | **no** |

`Turning::asking` reaches `alo-asking` directly, on whichever thread the daemon
is running on, which is in no turn's control group. The programme therefore sees
every provider request as *not a turn* — the answer that allows everything.

**This is measured, not reasoned.**
`crates/alo-turn/tests/whether_a_question_runs_inside_the_boundary.rs` drives one
turn that does one file verb and asks one question, with a `Bounding` that counts
how many executions were carried out inside it. The count is **one**, and the one
it counted was the read. If somebody bounds the asking, that test fails and says
where to come.

So the v0.01 network requirement is **not** met, and nothing here claims it is.

## The request path, traced

| Step | Where | Which thread | Inside the boundary |
|---|---|---|---|
| policy | `alo_answering::Answering::chosen`, `alo_egress::EgressPolicy` | the turn's caller | no |
| indicator | `alo_egress::Indicator::beginning` → `Departing` | same | no |
| address resolution | inside `ureq`, from the provider's URL | same | no |
| **permission registration** | **does not exist** | — | — |
| connection | `alo_asking::openai::put` → `ureq::post` | same | **no** |
| response / error | same call | same | no |
| record | `alo_record::Entry::left`, written by `alo_keeping::Writing` | same | no |
| permission withdrawal | `Asked::ended(indicator)` takes the line off the indicator | same | — |

The lifecycle *shape* is already right: a `Departing` cannot be obtained without
the policy having been asked and the person shown, and `ended` is where a
registration would be withdrawn. What is missing is that none of it happens
inside a control group, and nothing registers a destination.

## What the integration would require, and why it is not started

Two changes, and the first is a decision rather than a commit.

**1. The question would have to be carried out inside a boundary.** `Bounding`
is a public trait in `alo-turn`, and its one method takes an `alo_files::Reaching`
— file places. A question has no file places and needs destinations instead. So
this needs either a second method on `Bounding` or a widened one, and it puts a
network request inside a control group for the first time. **That changes the
turn lifecycle**, which this workstream's rules say is a decision to request
rather than make.

**2. The address would have to be known before the connection is made.**
`ureq::post(url)` resolves and connects inside one call, so there is no moment
between *which address* and *connect* for a registration to happen in. Getting
one means resolving first and handing the client a fixed address — which changes
how every provider request is made, and interacts with retries, pools and
redirects. It is buildable, and it is not a small change to a path that carries
somebody's question and their key.

### The decision requested

> **May a question be carried out inside a turn's boundary, with its destination
> registered for the length of the request?**

What it would change: `Bounding` grows a way to bound a network request;
`Turning::asking` runs the provider call inside it; `alo-asking` resolves before
connecting so the address can be registered; `Asked::ended` withdraws it.

What it would not change: no grant widens, no agent capability is added, the
provider-and-region policy stays exactly where it is in userspace, the indicator
still decides and shows, and the record is unchanged. **Local-model operation is
explicitly preserved** — loopback is not checked, so a model on this machine is
untouched.

I have not started it. Approval, or a different direction, is what unblocks it.

## Coverage — what production can actually reach

Assessed for reachability rather than listed. **None of these is closed by
`socket_connect` tests, and this report does not claim full network enforcement
from them.**

| | Production-reachable? | What prevents unauthorised egress today |
|---|---|---|
| **A provider request** | **yes — the ordinary path** | **Nothing in the kernel.** `alo-egress` decides and shows before the call; that is a promise the daemon keeps, not one the machine enforces |
| **DNS** | **yes**, inside `ureq` | Nothing. It is UDP, and a bounded turn would need it permitted — which is an unsolved part of change 2, because the resolver's own traffic is not the destination anybody was shown |
| **UDP `sendto`** | **yes, via DNS** | Not hooked. `socket_connect` never sees an unconnected datagram |
| IPv4 / IPv6 | yes | Both enforced by the hook, for connections a bound turn makes |
| **Connection pooling** | **yes** — `ureq`'s default agent pools, and `alo-models` builds one explicitly | **A reused connection makes no `connect`**, so a second request to a withdrawn destination over a live pooled connection is not seen. Release-relevant if change 1 lands without addressing it |
| Redirects | yes | **Already prevented**, and not by this work: `alo-asking::openai` sets `max_redirects(0)` and turns a redirect into `WentWrong::SentSomewhereElse`. `alo-models::address` refuses an authority that only looks like loopback |
| Sockets already open or inherited | yes — the daemon's own socket and record | Not hooked; `file_open` and `socket_connect` both decide at the moment of opening |
| **Loopback proxy** | yes | Nothing, and `docs/quirks.md` was corrected in `386096c` to stop pointing at this work as the place it is caught |

Two of these are release-relevant if the decision above is taken: **DNS** and
**connection pooling**. Both would let an authorised request quietly become
authority for more than it was shown, which is the exact failure the destination
binding exists to prevent. Neither is closed, and neither should be closed by
guessing — they belong in the same decision.

## What was published in this task

The audit, and the evidence test that holds it down. No enforcement changed.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`, run by
the supervisor: `cargo fmt --all --check`; `cargo clippy --workspace
--all-targets -- -D warnings`; `cargo test --workspace`;
`RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps`; and the BPF target's
`fmt` and `clippy --release --target bpfel-unknown-none -Z build-std=core` on the
pinned nightly. Existing filesystem enforcement, loader tests and application
tests all still green.

The new test uses no network and no kernel: it counts calls to a `Bounding` it
owns, and its provider address is `127.0.0.1:1`, where nothing listens, because
what it measures happens *before* a connection.

**Windows:** `alo-turn` is portable and its tests compile and run there; the
kernel crates do not, and no Windows run is offered as enforcement evidence.

**WSL is development evidence and never certified-hardware acceptance.**

## Remaining gaps and hardware obligations

- **The v0.01 network requirement is not met.** The mechanism exists; the
  production path does not reach it. *Mechanism implemented* is not *release
  requirement verified*, and this report exists to keep those apart.
- DNS, UDP `sendto`, connection pooling, inherited sockets, loopback proxies —
  each assessed above with what does and does not prevent it.
- All physical acceptance. No *On the machine* box is affected.

## Proposed shared-document updates

**CHANGELOG.md** — nothing; no user-visible behaviour changed.

**ROADMAP.md** — **a correction to what `386096c` proposed.** That report
suggested the *Egress indicator* line's remaining clause could drop "the
enforcement at the network boundary". **It cannot yet**: the enforcement exists
and nothing on the production path reaches it. The clause should stay until the
decision above is taken and the integration is built and tested.

**docs/autonomy/QUEUE.md** — no new item; this workstream's list is the plan.

**docs/autonomy/STATE.md** — reference this report. The fact worth carrying: the
kernel's destination enforcement decides by control group, a provider request is
made from a thread in no control group, and so the v0.01 network requirement is
not yet met by the mechanism built for it.
