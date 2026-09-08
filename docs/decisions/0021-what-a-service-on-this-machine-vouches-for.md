# ADR 0021 — What a service on this machine vouches for

**Status:** **PROPOSED — not accepted, and nothing in it is built.** It asks the
repository owner one question and recommends an answer. No enforcement, promise
or default changes until it is accepted.
**Date:** 2026-09-08, revised the same day with a third option the owner asked
for. Neither A nor B nor C is approved.
**Proposed by:** the kernel-enforcement workstream
**Context:** [ADR 0007](0007-the-cpu-is-the-default.md) (the CPU is the default),
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md),
[ADR 0020](0020-a-question-is-carried-out-inside-the-turns-boundary.md),
`crates/alo-asking/src/served.rs`, `crates/alo-egress`, `docs/quirks.md`,
`docs/autonomy/updates/publication-hardening-and-egress-coverage.md`

## The question in one line

**Does `Served` vouch that the address is on this machine, or that nothing
leaves this machine?** The code takes the first and the documentation around it
reads as the second, and a proxy on loopback is the case where those differ.

## The gap, stated precisely — and it is not where the last report said

`docs/autonomy/updates/publication-hardening-and-egress-coverage.md` called the
loopback proxy production-reachable and named *a provider endpoint pointed at a
local proxy* as the way in. **That way in does not exist**, and the correction
matters because it moves the gap to a different door with a different guarantee.

alo OS answers a question through one of three doors
(`crates/alo-turn/src/answers.rs`):

| Door | Address | Indicator | Record | Where a proxy could sit |
|---|---|---|---|---|
| `Answers::Provider` | anywhere | shown before a socket opens | a departure | **nowhere** — see below |
| `Answers::Runtime` | no socket at all | nothing to show | no departure | nowhere |
| `Answers::Service` | **loopback, enforced** | nothing shown | no departure | **here** |

**The provider door already refuses a loopback address.**
`Asking::to_a_provider` answers `Miswired::NotAProvider` when the source is
`InferenceSource::ThisMachine`, which is what a loopback endpoint reports as. The
question is never put and no socket opens. Measured by reading the code path, not
supposed.

**The service door is the one.** `Served::at` refuses every address that is *not*
this machine, and `Asking::to_a_service_on_this_machine` then says, in a comment
that is the whole of the reasoning:

> No policy is asked, no indicator is shown and no departure is made. There is
> nothing here for any of the three to be about, and the reason that is true
> rather than assumed is that `served` exists.

That is exactly right about the *address* and it is an assumption about the
*destination*. A person who runs vLLM, llama.cpp or LM Studio configures this
door with `127.0.0.1:8000`. A person who runs something on `127.0.0.1:8000` that
forwards elsewhere configures it identically, and alo OS cannot tell them apart:
it shows nothing, records no departure, and the answer says *on this machine*.

**The kernel half is reproduced.**
`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs` drives a bound
turn refused a non-loopback address with `EACCES` and reaching that same address
through an eleven-line relay on `127.0.0.1`, in the same run. The boundary is in
force and loopback is unchecked, deliberately, because ADR 0007 makes a model on
this machine the default and refusing loopback would break it.

**What an agent cannot do is start the proxy.** Task 5 measured it: `execve`
opens the file it runs and `file_open` is watched, so a bound turn cannot start a
program. The proxy is the person's, or another process the person already trusts.
That is the fact the whole decision turns on.

## Which promise this bears on

`docs/features.md`, *Sovereignty, as testable claims*:

- `[v0.01] ★ The egress indicator: every network egress an agent causes, visible
  at the moment it happens`
- `[v0.5] ★ A working day with a local model produces zero inference egress,
  measured at the network boundary — and we publish the test`

The v0.01 line is about **visibility** and the v0.5 line is about **measurement
at the network boundary**. They fail differently here:

- **v0.01 is affected.** A question the agent caused left the machine and nothing
  was shown. Whether the agent *caused* the second hop is arguable — the proxy
  made that connection — but a person reading that line would not accept "your
  question left, invisibly, because you had configured the address it went to".
- **v0.5 is affected for `Service` and safe for `Runtime`.** With
  `Answers::Runtime` there is no socket at all and the claim is carried by the
  absence of a type. With `Answers::Service` there is a loopback socket and alo
  OS cannot see past it, so *measured at the network boundary* is precisely the
  measurement that would catch this and precisely the one not yet built.

**Neither line is narrowed by this ADR.** If the owner takes Option B, the
wording change it needs is written out below, to be made deliberately or not at
all.

## Option A — enforce the proxy's outgoing connections

Extend enforcement beyond the turn: what leaves this machine is filtered whoever
sends it, so a proxy forwarding an agent's question is refused or shown.

**Local models.** Unbroken in the ordinary case — a runtime on loopback makes no
outbound connection. But a local runtime that *fetches* a model, and ADR 0019's
runtime discovery, and image updates, are outbound connections by processes that
are not turns, and a machine-wide filter has to have an answer for each. The
answer is a per-process policy, which is a policy engine alo OS does not have.

**Visible destinations.** Better in principle: the person would see the real
onward address rather than the proxy's. In practice the filter sees a connection
from a process, not from a turn, and **cannot attribute it to the agent that
caused it** — cgroup attribution is what makes attribution possible and the proxy
is in no turn's cgroup. So the person is shown *something was blocked* without
being shown *whose question it was*, which is a worse indicator than none.

**The indicator.** `Leaving::asking` is constructed from what the daemon knows
before a socket opens. A machine-wide refusal happens somewhere the daemon is
not, so feeding it to the indicator means a path from the kernel into userspace
carrying observations.

**The record.** Same problem, and worse: `alo-record` records what the daemon
did. Recording a refusal the kernel made means **kernel-sourced records**, which
is the unmade decision ADR 0015 leaves open and which *the LSM decides and
forgets* currently forbids. Option A therefore does not stand alone — it drags
that decision in with it.

**Sovereignty.** The strongest possible form of *nothing leaves that the person
was not shown*.

**And the cost that is not technical.** alo OS is a workspace the person owns.
Filtering what the person's own processes may send is a different product: a
managed device. The constitution's first law protects the tenant; it does not
police them. That is a change of kind, and it is why this option is set out
rather than assumed away.

**Scope if chosen:** a non-turn-scoped enforcement point (cgroup/skb or nftables
under alo's control), a per-process policy model, an attribution path from that
point to the indicator, and the kernel-records decision. Multiple releases.

## Option B — trust a service the person started, and stop letting it stand for more

State plainly that a process the person started is the person's own trust
boundary, and that alo OS does not police it. Then remove the part that is
silent: `Served` keeps vouching that the address is on this machine, and stops
being read as vouching that nothing leaves it.

**Local models.** Untouched. `Answers::Runtime` is unaffected in every respect;
`Answers::Service` still asks no policy, still opens a loopback socket, still
answers *on this machine*. ADR 0007's default is exactly as it was.

**Visible destinations.** Unchanged at the moment of asking. What changes is that
the person is told, **where they configure a service on this machine**, that alo
OS cannot see past that address — once, at the place the trust is actually given,
rather than on every question.

**The indicator.** Unchanged, and deliberately: firing it for every local
question would train people to ignore it, which costs more than it buys. The
v0.01 line is met by the person having been told what the configuration means,
not by an indicator that cannot distinguish a runtime from a relay.

**The record.** Unchanged. No departure is invented for something that, as far as
this machine can honestly say, did not depart.

**Sovereignty.** The promise becomes *nothing leaves that the person was not
shown, except through something the person themself put there* — which is
honest, and is only acceptable if it is **written down**, which is the wording
change below.

**Scope if chosen:** documentation and one user-facing string where a service on
this machine is configured; the quirks entry updated from *open hole* to
*accepted and stated*. No kernel change, no new map, no change to turn isolation,
no new capability, no grant widened. Days, not releases.

## Option C — tell the two kinds of local service apart

Asked for after A and B were written, and it turns out to be **two proposals
wearing one name**. Separating them is most of the work of evaluating it.

### The distinction is real, and it already exists in the types

| | What it is | How alo knows | Door |
|---|---|---|---|
| **Managed by alo OS** | the runtime alo OS ships — Ollama, pinned by [ADR 0006](0006-the-pinned-model-runtime.md), behind the `ModelRuntime` trait | it is ours: in our image, with our adapter, and `crates/alo-models/src/ollama.rs` is the only file that knows it exists | `Answers::Runtime` |
| **Configured by the owner** | vLLM, llama.cpp's server, LM Studio — or a relay | an address the person typed, and nothing else | `Answers::Service` |

So the category is not invented; it is the difference between the two doors that
already exist. **And that is the first finding: the door where a proxy can sit is
the owner-configured one, and Option C's enforcement half is about the other
one.** They are different risks.

### C1 — say truthfully where an answer came from

`InferenceSource::ThisMachine.shown(&strings)` renders **"on this machine"**, and
`Served::source()` returns that unconditionally, because the address is loopback.
For the runtime alo OS ships that is true. For a service the person configured it
is **a claim alo cannot verify**, and it is the one the owner's instruction names
directly: an answer must not be described as processed on this machine merely
because its address is loopback.

Proposed: a source of its own for an owner-configured local service, rendered as
something like *answered by a service at an address on this machine — alo cannot
verify where your question was processed*, externalised for i18n like every other
string. The runtime alo OS ships keeps *on this machine*, which stays true.

**This is where C is stronger than B.** Option B discloses once, where the person
configures the service. C1 discloses **on every answer**, which is where the
person is actually reading, and it removes an untrue sentence rather than adding
a caveat beside it.

**It is not free.** `InferenceSource` is matched exhaustively in several crates,
and `SourcePolicy::ThisMachineOnly` currently permits this door *because* the
source is `ThisMachine`. That gives the owner a second question: does a machine
set to keep questions on it permit a service alo cannot verify? Two defensible
answers — permit and label, or refuse — and the recommendation below takes one.

### C2 — restrict what the alo-managed runtime may reach

Restrict outbound access **for the runtime alo OS ships**, and for nothing else
on the machine. Mechanically this is the attractive part of C: our runtime is our
own unit in our own image, so it has a control group of its own without anybody
inventing one, and a cgroup-scoped egress programme attached to that one cgroup
touches no unrelated application. That is a real difference from Option A, which
has to have a policy for every process the person runs.

**It covers what `socket_connect` cannot.** A cgroup egress programme sees
packets rather than `connect` calls, so it covers **connections opened before the
restriction, sockets inherited into the unit, and datagrams sent without
connecting** — the three gaps this workstream has reproduced and cannot close
turn-side. It covers the service's child processes too, because a unit's control
group contains them. Loopback traffic is still seen and would need an explicit
rule, since the runtime must be able to answer `alo-agentd`.

**And it does not touch the proxy gap at all.** A relay the person started is not
the alo-managed runtime and is not in its cgroup.

### C3 — separate downloading from answering

C2 forces this question, because the runtime's one legitimate reason to reach the
network is fetching a model, and `crates/alo-models/src/ollama.rs` already has
the two apart as methods: `Ollama::fetch` downloads from the curated catalogue,
`Ollama::answers` carries a question.

**A cgroup cannot tell them apart**, because both are the same server process. So
splitting the *permission* means splitting the *work*, and there are two ways:

1. **alo fetches, the runtime never does.** The daemon downloads the blob through
   its own errand path — `crates/alo-egress/src/errand.rs` already models an
   errand as a thing that leaves for reasons that are not a question — and hands
   it to the runtime. The runtime's inference-time egress budget becomes exactly
   **zero**, which is a claim worth having, and the download gets what an errand
   gets: shown, policy-checked, recorded. **It changes ADR 0006's adapter shape**,
   because `fetch` stops being the thing that reaches the catalogue.
2. **A time window in which the runtime may fetch.** Rejected. Authority that
   opens for a period is authority that outlives what anybody was shown, which is
   the failure this whole workstream exists to prevent.

### What C depends on, and what it cannot do

- **C2 is blocked on something that does not exist.** The runtime alo OS ships is
  a trait, an adapter and an ADR; there is no unit in the image running it and no
  cgroup to attach anything to. C2 cannot be built before it is.
- **C2 needs a second programme type and a second attachment point**, which
  [ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)'s loader
  does not have — it loads one programme and pins its hooks.
- **C does not close the loopback-proxy gap.** C1 makes alo stop mis-describing
  it; C2 restricts a different service entirely. **The gap stays open under C**,
  and calling C a closure would be the mistake this ADR exists to avoid.
- **C1 has a behaviour consequence** for `SourcePolicy::ThisMachineOnly`, which is
  the owner's to settle.

## The three, side by side

| | **A** — filter the person's processes | **B** — trust the person's service, disclose once | **C** — tell the two kinds apart |
|---|---|---|---|
| Closes the proxy gap | yes | no, states it | **no**, describes it truthfully |
| Local models (`Answers::Runtime`) | at risk — model pulls are outbound | untouched | **C2 restricts it deliberately; C3 gives the pulls back through alo** |
| Unrelated applications | restricted | untouched | **untouched** |
| Inherited sockets / open connections / UDP | covered, machine-wide | not covered | **covered inside the managed runtime's cgroup only** |
| Attribution to a turn | **impossible off the cgroup** | n/a | n/a — it is a service-level restriction, not a turn's |
| The indicator | needs a kernel→userspace path | unchanged | unchanged |
| The record | needs kernel-sourced records (unmade decision) | unchanged | the download becomes a recorded errand |
| Truthful provenance | unchanged | disclosed at configuration | **fixed at the answer, which is where it is read** |
| Approvals | many, plus ADR 0015's open question | one, plus a promise rewording | C1 one; C2 a new programme type; C3 an ADR 0006 change |
| Blocked on | nothing | nothing | **C2/C3 blocked on a runtime unit that does not exist** |
| Scope | multiple releases | days | C1 days; C2+C3 a release of their own |

## Recommendation, revised

**Option B for the enforcement question, and C1 for the disclosure — and C1
replaces B's wording change rather than joining it.**

1. **Do not filter the person's own processes (reject A).** Off a turn's control
   group a refusal cannot be attributed to the agent that caused it, and an
   unattributed block is a worse answer than a stated limit. It drags in the
   unmade kernel-records decision. And it turns a workspace the person owns into
   a managed device, which is a change of product taken by accident.
2. **Stop saying "on this machine" for a service alo cannot verify (take C1).**
   This is the strongest single sentence in the whole comparison: it is small, it
   needs no kernel, it breaks no local model, and it removes an untruth from the
   place the person actually reads. It does more for the v0.01 promise than
   Option B's configuration-time notice, because the promise is about what a
   person is shown at the moment it happens.
3. **On `SourcePolicy::ThisMachineOnly`: permit and label, do not refuse.** A
   person who set that rule and then pointed alo at their own vLLM meant to keep
   their questions local, and refusing them would punish the honest case to
   inconvenience the dishonest one. The label is what carries the truth.
4. **Treat C2 and C3 as a separate, later piece of work** aimed at a different
   risk — our own runtime's egress, and the v0.5 *zero inference egress, measured
   at the network boundary* claim. **Do not schedule them under this decision**,
   and do not build C2 before there is a runtime unit to attach it to.

**What this recommendation does not do is close the gap.** A proxy the person
starts still carries a question off the machine and alo still cannot see it. What
changes is that alo stops claiming otherwise.

## Success and refusal tests, if the recommendation is accepted

Named now so acceptance can be finished rather than argued. Each fails in a
direction somebody can act on.

**Success**

1. `an_answer_from_a_service_we_cannot_verify_does_not_claim_this_machine` — the
   provenance line for `Answers::Service` does not render *on this machine*, and
   says alo cannot verify where the question was processed.
2. `the_runtime_alo_ships_still_says_on_this_machine` — `Answers::Runtime` is
   unchanged, because for it the sentence is true.
3. `the_words_are_externalised_like_every_other` — the new string is in the
   vocabulary and translatable; a hardcoded English sentence is a bug in a
   European product.
4. `a_machine_that_keeps_questions_on_it_still_permits_a_local_service` —
   `SourcePolicy::ThisMachineOnly` permits `Answers::Service`, labelled. The
   recommendation's step 3, asserted rather than assumed.
5. `a_service_on_this_machine_still_answers_without_showing_anything` — no
   indicator, no departure, no policy asked. Regression cover for what must not
   change.

**Refusal**

6. `a_provider_at_a_loopback_address_is_still_not_a_provider` —
   `Miswired::NotAProvider`, unchanged.
7. `a_service_that_is_not_on_this_machine_is_still_refused` —
   `Served::at` still answers `Miswired::ReachesOffThisMachine`.
8. `a_bound_turn_is_still_refused_an_address_nobody_showed_it` — the existing
   kernel reproduction, unchanged: accepting this must not relax `socket_connect`
   by one case.

**Not a test:** nothing asserts a proxy is absent, because nothing can.

## The approvals each option needs, exactly

| Change | Approval | Why |
|---|---|---|
| C1 — a source of its own for an unverifiable local service | **yes** | it changes what an answer says about where it came from, and touches `InferenceSource`, which several crates match exhaustively |
| C1's policy consequence — `ThisMachineOnly` permits and labels | **yes** | a rule's meaning, which is the owner's to set |
| B — trusting an owner-started service | **yes** | it is a statement about what alo does not defend against |
| B's promise rewording | **superseded by C1** if C1 is taken, and still required if it is not |
| A — filtering the person's processes | **yes**, and ADR 0015's records question first | a different product, and a refusal it cannot attribute |
| C2 — a cgroup egress programme on the managed runtime | **yes** | a second programme type and attachment point ADR 0018 does not have |
| C3.1 — alo fetches models, the runtime does not | **yes** | it changes ADR 0006's adapter shape |
| C3.2 — a time window for fetching | **rejected here**, not proposed | authority that outlives what was shown |

## What this ADR does not decide

- **Nothing about inherited descriptors or sockets.** Those are a separate
  decision and are set out in
  `docs/autonomy/updates/network-boundary-decisions-proposed.md`.
- **Nothing about kernel-sourced records.** ADR 0015's open question stays open;
  Option B needs none and Option A would need it answered first.
- **Nothing about turn isolation.** A turn stays one thread of `alo-agentd`.
- **Nothing about ADR 0020.** Request-scoped destinations, original-hostname TLS
  verification, the per-request client and the refusal default all stand exactly
  as accepted.
- **It does not close the loopback hole.** Under Option B the hole is accepted
  and stated. Under Option A it would be closed by work that has not been
  scheduled. Either way, today, it is open.

## The decision the owner must make

> **Does alo OS filter what the person's own processes send (Option A), or does
> it trust a service the person started and say plainly that it cannot see past
> that address (Option B, with the `docs/features.md` wording change above)?**

A third answer — *Option B without the wording change* — is available and this
ADR argues against it, because that is the version where the promise stays as
written and stops being true.
