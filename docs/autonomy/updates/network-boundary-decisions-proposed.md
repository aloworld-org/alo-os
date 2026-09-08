# The remaining network-boundary decisions, proposed

- Date: 2026-09-08
- Workstream: kernel enforcement (`alo-bounding`, `alo-asking`, `alo-egress`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Prepare proposals for the remaining network-security decisions
- Status: **proposals only. Nothing was built, no policy changed, no promise
  narrowed, no test written to certify prose.**
- Produces: [ADR 0021](../../decisions/0021-what-a-service-on-this-machine-vouches-for.md)
  (**proposed**), a reconciled `docs/autonomy/kernel-enforcement-plan.md`, and
  the options below

## A correction, first, because it moves the gap

The previous report,
`publication-hardening-and-egress-coverage.md`, called the loopback proxy
production-reachable and named the way in as *a provider endpoint pointed at a
local proxy*. **That way in does not exist.** `Asking::to_a_provider` answers
`Miswired::NotAProvider` when the source is `InferenceSource::ThisMachine`, which
is exactly what a loopback endpoint reports as; the question is never put and no
socket opens.

The reachable door is `Answers::Service` — an OpenAI-compatible service on this
machine, which `Served::at` *requires* to be loopback and for which
`Asking::to_a_service_on_this_machine` asks no policy, shows nothing and records
no departure. The gap is real, and it is one door to the left of where the last
report put it. ADR 0021 states it precisely.

**The conclusion is unchanged and the reasoning behind it is not.** The
difference matters because the two doors carry different guarantees: the provider
door promises *this is shown and recorded*, and the service door promises *the
address is on this machine*. Only the second can be satisfied by a relay.

## The loopback proxy — ADR 0021, proposed

Full argument in the ADR. In one paragraph:

`Served` vouches that the address is on this machine. The code around it reads as
though it vouched that nothing leaves this machine, and a proxy on loopback is
where those differ. **Option A** filters what the person's own processes send;
**Option B** trusts a service the person started and says plainly that alo cannot
see past it.

**Recommended: Option B, conditional on rewording the v0.01 promise to say so.**
Option A cannot attribute a refusal to the turn that caused it — attribution is
cgroup-shaped and the proxy is in no turn's cgroup — it drags in the unmade
kernel-records decision, and it turns a workspace the person owns into a managed
device. Law 2 keeps the hole small: **an agent cannot start the proxy**, measured
in task 5, because `execve` opens the file it runs and `file_open` is watched.

Option B without the wording change is a narrowed promise rather than a stated
one, and the ADR argues against it on those terms.

## Inherited descriptors and sockets, and checks after a connection

A separate decision, outlined rather than proposed, because the honest framing
turned out not to be the one the question is usually asked in.

### The framing: it is not a question about hooks

The obvious answer is *add `file_permission` for descriptors and `socket_sendmsg`
for sockets.* Both were evaluated against the constitution and both run into the
same wall, which is not about cost:

**A turn's way out of its own boundary is an inherited descriptor.** Leaving a
turn is a write to `home/cgroup.threads`, on a descriptor `alo_bounding::Turns`
opened before the first turn ever ran (`crates/alo-bounding/src/turns.rs`). Task
6 measured it: the same turn is refused `open` on that file and leaves through
the descriptor anyway. A hook that refused writes on inherited descriptors would
refuse a turn its own exit, and the exemption that fixed it would be a hole
shaped exactly like the thing being closed.

So the real question is **whether a turn may keep sharing the daemon's descriptor
table at all** — a question about what a turn *is*, not about which hook watches
it. Anything else is an exemption list, and an exemption list is where this kind
of boundary goes to die.

### What is actually shared

A turn is one thread of `alo-agentd` (law 2: nothing is started). It therefore
shares every descriptor the process holds:

| Held | Why it exists | If a hook refused it |
|---|---|---|
| **`home/cgroup.threads`** | the way out of a turn | **the turn could never leave** — measured, task 6 |
| The record (`alo_keeping::Writing`) | law 1's account of what happened | **records are written outside the boundary** — `crates/alo-turn/src/carrying.rs` calls `carrying_out` and *then* `Entry::ran`, and `alo-turn/src/asking.rs` does the same after the departure. So a hook would not break recording, which is the one piece of good news here |
| The socket to the person | the daemon answering the person | the person's own session would stop |
| The vocabulary, the settings | strings and configuration read at start | read-only, and the least of it |
| **Any file a verb opened legitimately** | ordinary work | fine — it was opened inside the boundary |

### The options

**1. Do nothing; keep it documented and reproduced.** Where it stands today. The
gap is real and is the only one in this crate that moves contents past a grant.
**Needs no approval.** Costs: the boundary's own promise is qualified, and the
qualification is in `docs/quirks.md` rather than in anything a person reads.

**2. Descriptor hygiene in the daemon — defence in depth, not a closure.** Hold
fewer descriptors while a turn runs: open the record for the append and close it,
keep the vocabulary in memory rather than open, mark everything the daemon opens
close-on-exec. **Needs no approval** — it widens nothing, adds no capability,
changes no security model and touches no kernel. It cannot close the gap: the way
out and the person's socket must stay open by construction. **It must never be
reported as a closure**, and the only reason to name it here is that it is the
one thing available without a decision.

**3. `file_permission`, with an exemption for the way out.** Closes reading
through an inherited descriptor. **Needs approval.** It is a walk on every read
and write on the machine — the opposite direction from *decides and forgets* in
cost, though not in memory, since it would still write nothing. Its exemption
list starts at one entry and the first review would add the person's socket.
**Do not assume this hook is sufficient:** it closes reading and leaves the
exemptions, and the exemptions are the interesting part.

**4. `socket_sendmsg` for sockets, or a cgroup/skb programme on the turn's own
cgroup.** Closes inherited sockets *and* unconnected datagrams together — the two
are one question, not two. **Needs approval.** `socket_sendmsg` is a hook on
every message the machine sends and hits the same exemption problem for the
daemon's own socket. A cgroup/skb programme is turn-scoped by construction, which
is more attractive, but it is a second kind of programme and a second attachment
point, and ADR 0018's loader shape assumes one; loopback stays exempt in either
case, so neither touches the proxy gap.

**5. A turn becomes a process of its own.** Closes inheritance completely — a
fresh descriptor table is the only thing that does. **Needs approval, and it is
the largest change in this list.** It collides with law 2's *nothing is started*:
alo would be starting a program on an agent's behalf, and a program alo starts on
an agent's behalf is one review away from a program an agent named. **It does not
follow that isolating a turn in a process would permit arbitrary agent commands**
— the capability model is what decides what a verb may do, and that is untouched
by where the verb runs — but the argument has to be made in an ADR rather than
assumed, because the thing being changed is what a turn *is*.

**Evaluated and rejected: seccomp.** ADR 0013 names it and it is unbuilt, but a
seccomp filter sees syscall numbers and argument values, not a descriptor's
provenance. It can refuse `read` entirely; it cannot refuse `read` on an
inherited descriptor while permitting it on a granted one.

### Which of these need approval

| Option | Approval | Why |
|---|---|---|
| 1. Do nothing | no | the present state |
| 2. Descriptor hygiene | **no** | widens nothing, no capability, no kernel change, no security model change — and closes nothing |
| 3. `file_permission` | **yes** | new hook, new cost model, and an exemption for the way out of a turn |
| 4. `socket_sendmsg` / cgroup-skb | **yes** | new hook or new programme type; ADR 0018's loader shape |
| 5. Turn as a process | **yes** | changes what a turn is, and collides with law 2 |

**No recommendation is made here.** The task asked for options, the decision
belongs with whoever schedules v0.5, and recommending one would be taking it.
What this report does claim is the framing: *the question is not which hook.*

## What is preserved

Nothing in this task changed behaviour. Stated because it is checkable:

- **ADR 0020 stands entire** — request-scoped destinations, original-hostname TLS
  verification, the per-request client, the refusal-by-default trait method.
- **Local-model behaviour** — `Answers::Runtime` opens no socket;
  `Answers::Service` is unchanged; loopback is still unchecked (ADR 0007).
- **Grants, indicator and records** — untouched.
- **Turn isolation** — a turn is still one thread of `alo-agentd`.
- **No kernel map was added**, no hook attached, no capability granted, no
  Windows networking touched.

## Verification, and one limitation reported rather than worked around

**The gates were run and pass**, twice: once on the tree as written, and again
after integrating `origin/main` at `d40971b`, which is the state these three
files sit on. Formatting, clippy with warnings denied, the whole workspace's
tests, the supervisor's own 23 tests, rustdoc with warnings denied, and the BPF
target's formatting and clippy on the pinned `nightly-2026-06-01`. Ubuntu on
WSL2, kernel 6.18.33.2.

**One thing the gate run measured that nobody asked for.** The first attempt
failed all five tests in `a_question_is_bounded_by_the_kernel.rs`, on a tree
where only documents had changed. The cause was **the other checkout gating at
the same time**: `/mnt/c/dev/alo-os`'s supervisor was running its own
`cargo test --workspace` against this same WSL kernel, and a pin belonging to one
of its then-live test processes was in `/sys/fs/bpf`. Both suites pass when run
apart; both were run apart afterwards and did.

This is a real coordination hazard rather than a flake to shrug at. Every kernel
test in this workstream serialises **within its own test binary** — a `Mutex` in
`alo-agentd`'s, `on_this_kernel::one_at_a_time()` in `alo-bounding`'s — and
**nothing serialises them across processes**. Two checkouts on one kernel is two
sets of attached programmes and two sets of control groups, and the symptom is
assertions that fail once and pass on rerun, which is the worst shape a failure
can take because it teaches people to rerun.

**The stale pin was not removed**, and this is the case that rule was written
for: it looked like wreckage from a dead run and it belonged to a process that
was alive and working. Removing it would have broken the other worker's gate run.

A cross-process lock — an advisory `flock` on a well-known path, taken by every
kernel test in both checkouts — would close it, has testable behaviour, and is
therefore publishable in the ordinary way. **It is not built**: it is not an
authorised task, and it touches a fixture the other worker's suite would also
have to take. Named here so it can be scheduled rather than rediscovered.

**This change could not be published by the supervisor, and that is correct.**
`alo-kernel-loop` requires each acceptance criterion to name a test that is part
of the change and passes on its own. This change is three documents. It has no
acceptance test, because there is nothing in it to execute — and the two ways to
get one past the check would both be dishonest:

- **Inventing a test to certify prose.** Explicitly forbidden, and rightly: a
  test that asserts a document says what it says catches nothing.
- **Adding a documentation-only exemption to the supervisor.** A narrow,
  mechanically-checked rule is imaginable — *a change touching no code needs no
  implementation evidence, and the loop verifies it touches no code.* It is
  arguable that this strengthens the system, since the alternative pressure is to
  invent a test. **It is still a change to the evidence rule, and this workstream
  does not change its own evidence rule to publish its own work.**

So the limitation is reported instead: **this proposal is written, gated and
unpublished, sitting in `C:\\dev\\alo-os-claude`.** Publishing it needs one of

1. the owner accepting a documentation-only rule for the supervisor — the exact
   shape is above and it is a second, smaller decision; or
2. the owner or the desktop worker taking these three files into a change of
   their own; or
3. this workstream carrying them along with the next task that *does* have
   executable evidence — which is the ordinary answer, and is what has happened
   with every report so far.

Option 3 is the default and needs nothing. It costs only that the proposal sits
here until there is code to travel with, which for a decision the owner is being
asked to make is the wrong shape — hence reporting it now.

**WSL is development evidence and never certified-hardware acceptance.** No
*On the machine* box is affected by anything here.

## Proposed shared-document updates

Not made here — the desktop integration worker owns these four files.

**CHANGELOG.md** — nothing. No user-visible behaviour changed.

**ROADMAP.md** — no tick, and one note if ADR 0021 is accepted as recommended:
the *Egress indicator* line's remaining clause would still stand, because the
line also names the compositor surface, the daemon code that signs somebody in
and fetches a model, and physical acceptance.

**docs/autonomy/QUEUE.md** — no new item. Two decisions belong to whoever
schedules decisions: ADR 0021, and the inherited-handle options above.

**docs/autonomy/STATE.md** — three facts. The kernel plan is reconciled into five
sections and the stale *no socket or cgroup programme exists* row is gone. The
loopback-proxy gap is through `Answers::Service`, not `Answers::Provider`, and
ADR 0021 is **proposed** with Option B recommended conditional on a promise
rewording. And a documentation-only proposal cannot currently be published by the
kernel supervisor, which is a limitation reported rather than routed around.

## The decision the owner must make

> **Does alo OS filter what the person's own processes send (Option A), or does
> it trust a service the person started and say plainly that it cannot see past
> that address (Option B, with the `docs/features.md` wording change written out
> in ADR 0021)?**

Recommended: **Option B with the wording change.** The third answer — Option B
without it — is the one where the promise stays as written and stops being true.

A second, smaller decision, if the owner wants proposals published as they are
written rather than travelling with later code: **may the supervisor publish a
change that touches no code without acceptance evidence, when it verifies that
the change touches no code?**
