# ADR 0021 — What a service on this machine vouches for

**Status:** **ACCEPTED, 2026-09-10** — C1, Option B, and D1 with the setting's
own wording corrected. See *The decision, taken* at the end.
**Date:** 2026-09-08, accepted 2026-09-10. Revised twice on the 8th: once for a
third option, and once to align with the owner's model-choice clarification
(`docs/autonomy/updates/owner-model-choice-direction.md`, `9707ad5`). Everything
below is left exactly as it was argued while unaccepted, so the reasoning can be
checked against the outcome rather than rewritten to match it.
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

## Five things that are not the same thing

The owner's clarification of 2026-09-08 says it in one line — *a loopback address
establishes where a service is contacted, not where it performs inference* — and
the confusion it corrects runs through this whole subject. So they are separated
here first, and every option below is judged against the separation rather than
against a brand.

| | The question | What alo OS knows |
|---|---|---|
| **1. Who provides the model** | alo's catalogue, the person's own weights, a third party's | **exactly**, because the person chose it |
| **2. Who manages the runtime** | the unit alo ships, a service the person runs, a company's API | **exactly**, and it is which of the three doors was used |
| **3. Where processing occurs** | this machine, a paired machine, somewhere else | **almost nothing**, once a socket is involved. This is the one that matters and the one alo cannot see |
| **4. What alo can verify or enforce** | the address connected to; the destination a *turn* may reach; what its own components do | **not** what a process on the other end of a loopback socket does next |
| **5. What has been permitted** | `SourcePolicy`, the organisation's rule, the grant, the typed capability | **exactly** — this is userspace and it is settled before anything opens |

**Brand is not evidence of privacy, in either direction.**

- A **third-party model can be genuinely local**: llama.cpp serving weights the
  person downloaded themselves never touches a network, and no part of that
  depends on who wrote it.
- An **alo-provided service can be remote**: `docs/features.md` already says our
  own hosted service gets no exemption — *the same egress indicator fires, the
  same provenance line is shown, a machine set to keep questions in the building
  refuses ours too.*

The mistake this ADR exists to correct is a **category error, not a
vulnerability**: `Served::source()` answers (3) with a fact about (4). It says
*this machine* because the address is loopback. That is the strongest thing alo
knows and it is not the thing being claimed.

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

**And the production half is reproduced, through the real door.**
`crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs` copies every
assertion from the honest case in `a_day_that_never_left.rs` — the answer says
*on this machine*, the indicator is quiet, the record's *what left this machine*
is empty — and makes them against a service at `127.0.0.1` that forwards the
question to a listener on this machine's own interface. **All of them still
pass**, while the far service holds the person's question and reports the text of
it. A transparent relay is not a redirect: it answers in its own voice, so
nothing in the HTTP exchange gives it away, and
`a_local_service_cannot_redirect_a_question_off_this_machine` — which is a real
guarantee — does not touch it.

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

## Model choice is one thing; a local-only guarantee is another

The owner's direction is that **alo-provided models and services, the person's
own local models and runtimes, and compatible third-party APIs are all
legitimate choices, and alo ownership is never a condition of being one.** This
ADR must not narrow that, and none of the options below does: every one of them
leaves ordinary use of an owner-configured local service and of a third-party API
exactly as it is.

What follows from separating (1) from (3) is that **choosing a model and being
promised something about where it runs are different transactions.**

- **Ordinary selection** — the person picks a model, a runtime or an API. Free,
  unrestricted, and nothing here touches it. What alo owes is a truthful account
  of what it knows, which today it does not give.
- **An enforceable local-only restriction** — the person or the organisation
  says *questions must not leave this machine* and wants that to be **true**
  rather than intended. That is a guarantee, and a guarantee needs a mechanism.

Today those two are the same setting, and the setting is decided by an address.

### How a runtime the person owns could qualify — without alo owning it

The owner's question, and it has a clean answer: **qualification is about
supervision, not ownership.**

A runtime qualifies for a local-only guarantee when alo OS can *observe or
constrain its egress* — which means it runs under a unit whose control group
carries a zero-egress rule. Nothing in that mentions who wrote it. The runtime
alo ships would qualify because it happens to run that way; llama.cpp, vLLM or
LM Studio would qualify **identically** if the person asks alo to run them that
way, and the runtime alo ships would **stop** qualifying if it were run outside
that supervision.

That is the honest form of the guarantee: *this question cannot leave, because
the thing answering it cannot reach the network*, rather than *this question will
not leave, because of who wrote the thing answering it.*

**What it costs, and none of it is decided here:**

- It needs the cgroup egress mechanism of **C2**, which needs a second programme
  type ADR 0018's loader does not have, and which is blocked on there being a
  supervised unit at all.
- It needs **C3**: a supervised runtime that must fetch models needs the fetching
  to happen somewhere else, or its egress budget is not zero.
- It needs a way for a person to place their own runtime under that supervision
  — packaging work, not security work, and **not** alo starting a program on an
  agent's behalf, which law 2 forbids and which this is not.
- Until it exists, **no configuration qualifies**, including alo's own. Offering
  the guarantee before the mechanism exists would be the thing the owner's
  clarification explicitly forbids.

## Recommendation, revised

**Three parts, and only the first is buildable today.**

**1. Take C1 — truthful, externalised processing-location labels.** It is
independent of every enforcement question below, it restricts nobody's choice of
model, and it is the only part of this subject that is both correct and
available now. It stops alo OS answering question (3) with a fact about (4).

Proposed shape, to be written as words in the vocabulary rather than as English
in the code:

| Configuration | Today | Proposed |
|---|---|---|
| The runtime alo OS ships, unsupervised | *on this machine* | *on this machine* — unchanged, and true |
| A service the person configured at a loopback address | *on this machine* | *by a service on this machine's address — alo cannot verify where your question was processed* |
| A supervised runtime, once supervision exists | — | *on this machine, verified* |
| A provider | *by {provider}, in {region}* | unchanged |
| A paired machine | *on {machine}, in your network* | unchanged |

**2. Reject A.** Filtering what the person's own processes send cannot attribute
a refusal to the turn that caused it, drags in ADR 0015's unmade records
question, and turns a workspace the person owns into a managed device.

**3. Treat the enforceable local-only guarantee as its own piece of work**,
qualified by supervision rather than by ownership, and **do not offer it until it
is implemented and measured** — including its failure and bypass cases. Until
then no configuration qualifies, alo's own included.

**What none of this does is close the gap.** A service the person configured can
still forward every question, and alo OS still cannot see it. What changes is
that alo OS stops saying otherwise.

## The unresolved decision, stated as options

> **When *This machine only* is selected and the configured service's processing
> location cannot be verified, what should happen?**

This is the owner's to settle. **It is not settled here, and nothing in this
repository has been changed to anticipate any of these.** Today's behaviour is
D1 without the label, asserted in
`a_day_that_only_looks_like_it_never_left.rs::this_machine_only_still_permits_a_service_that_cannot_be_verified`
so that whoever changes it has to come here.

| | What happens | Ordinary use | The guarantee | The cost |
|---|---|---|---|---|
| **D1 — permit, and label** | as today, plus C1's truthful line | **preserved** | none. A label is a disclosure, **not evidence that a service cannot forward a question** | the setting keeps a name that promises more than it delivers, unless its own wording is corrected too |
| **D2 — refuse unless verifiable** | the strictest reading: no unsupervised local service under this rule | **broken for everyone**, since no configuration is verifiable today | real, once there is something to verify | a regression with no upside until supervision exists; it would refuse the honest vLLM user to inconvenience nobody |
| **D3 — permit, label, and ask once** | a one-time acknowledgement per service | preserved, with friction | none — consent is not enforcement | teaches people to click through a dialog, which is worse than a quiet truthful label |
| **D4 — two settings** | *prefer this machine* (permit + label) and *only verifiably this machine* (refuse unless supervised) | **preserved** under the first | **real** under the second | two settings to explain; the strict one is empty until C2/C3 exist, and offering an empty guarantee is the thing the owner's clarification forbids |

**Recommended: D1 now, with the setting's own wording corrected to match what it
does, and D4 when supervision exists.** D1 alone, with the setting still reading
as a guarantee, is the outcome this ADR argues against in every version — it is
the version where the words stay as written and stop being true.

**Explicitly: recommending D1 is not implementing it.** No code in this
repository has been changed toward any option, the rule still behaves exactly as
it did, and `docs/features.md` has not been touched.

## The tests, and which kind each one is

Distinguished on purpose, because a test that documents a gap and a test that
proves a guarantee read the same and mean opposite things.

**Guarantees, already tested, and none of them changes under any option**

| Guarantee | Test |
|---|---|
| A service that is not on this machine cannot become a door | `a_service_that_is_not_on_this_machine_never_becomes_a_door` |
| A local service cannot redirect a question off this machine | `a_local_service_cannot_redirect_a_question_off_this_machine` |
| A local model that fails never becomes an API call — **no silent fallback** | `a_local_model_that_fails_never_becomes_an_api_call` |
| A question bound for a provider never reaches the model here | `a_question_bound_for_a_provider_never_reaches_the_model_on_this_machine` |
| A day answered here puts no egress on the indicator or in the record | `a_day_of_questions_answered_by_a_local_service_puts_no_egress_in_the_record` |
| A provider at a loopback address is not a provider | `Miswired::NotAProvider`, `alo-asking`'s own tests |
| A bound turn is refused a destination nobody showed it | `the_kernel_refuses_a_departure.rs` |

**Gaps, documented, added by this work**

| Gap | Test | Layer |
|---|---|---|
| A forwarding service is answered as though it never left | `a_service_that_forwards_is_answered_as_though_it_never_left` | **the production `Served` door** |
| `ThisMachineOnly` permits a service alo cannot verify | `this_machine_only_still_permits_a_service_that_cannot_be_verified` | the production rule |
| A bound turn reaches a non-loopback address through a loopback relay | `a_proxy_on_loopback_carries_a_bound_turn_somewhere_nobody_showed_it` | the kernel |

**What acceptance would need, and what none of it may be**

If C1 is accepted: that the unverifiable case no longer renders *on this
machine*, that the shipped runtime still does, that the words are externalised,
and that the three gap tests above are updated to say what changed rather than
deleted. **No test may be written to certify this document.**

## The approvals each option needs, exactly

| Change | Approval | Why |
|---|---|---|
| C1 — a source of its own for an unverifiable local service | **yes** | it changes what an answer says about where it came from, and `InferenceSource` is matched exhaustively in several crates |
| D1/D2/D3/D4 — the `ThisMachineOnly` rule | **yes** | a rule's meaning, and the owner has said model freedom is not approval of one |
| Correcting the setting's own wording | **yes** | `docs/features.md` is the only list of what gets built |
| A — filtering the person's processes | **yes**, and ADR 0015's records question first | a different product, and a refusal it cannot attribute |
| C2 — a cgroup egress programme on a supervised runtime | **yes** | a second programme type and attachment point ADR 0018 does not have |
| C3.1 — alo fetches models, the runtime does not | **yes** | it changes ADR 0006's adapter shape |
| Supervision as the qualification for a local-only guarantee | **yes** | it is a new guarantee, and it must be measured before it is offered |

## What exists today, and what does not

Stated so that nothing here reads as a claim that every model or API works.

**Implemented and reachable now**

- **The runtime alo OS ships** — Ollama, pinned by ADR 0006, behind
  `ModelRuntime`, through `Answers::Runtime`.
- **An OpenAI-compatible service the person runs on this machine** — vLLM,
  llama.cpp's server, LM Studio — through `Answers::Service`, loopback enforced.
- **An OpenAI-compatible hosted provider** — through `Answers::Provider`, with
  the indicator, the policy and the record.
- **No model at all**, which stays a supported configuration.

**Typed but not reachable**

- **A paired machine.** `InferenceSource::PairedMachine` exists and every door
  refuses it — `Miswired::NoPathToAPairedMachine`, *nothing anywhere reaches a
  machine on this network yet*. The configuration is preserved as the owner asks
  and **it is not an integration that works today**.

**Not claimed**

Compatibility is with the OpenAI-compatible shape and with Ollama through its
adapter. **No claim is made that every model or every API is supported**, and
nothing in this ADR brings a later-release feature forward.

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

## The decision, taken

**2026-09-10. C1, Option B, and D1 with the setting's own wording corrected.**

**How this was decided, stated plainly.** The owner gave a standing delegation on
2026-09-10 — *"make decisions where needed"* — against the goal of an AI-native
operating system that is the best in the world. They did not review these options
one by one, and this ADR does not pretend otherwise. It is recorded here so that
whoever reads it later knows exactly whose judgement it was and can overturn it
with one line.

**Why this and not Option A.** Filtering what the person's own processes send
turns a machine they own into a managed device, cannot attribute a refusal to the
turn that caused it, and drags in ADR 0015's unmade records question. A
sovereignty product that polices its owner's processes has misunderstood which
side it is on.

**Why the wording change is the part that matters.** Everything else here is a
gap that stays open either way. What changes is that alo OS stops **saying**
something it cannot know. `Served::source()` answers *where was this processed*
with a fact about *what address was contacted*, and those differ exactly when it
matters — a proxy on loopback.

For a product whose whole claim is sovereignty, **a label that is not true is
worse than no label at all**. It is worse than the hole it papers over: the hole
is a limitation shared with every operating system in existence, and the false
label is a betrayal that would be found once and believed never again. Nobody
buys this product for the feature list. They buy it because when it says a thing,
the thing is so.

That is the whole reason this is worth doing before anything more impressive.

**What is built now:** C1 — a truthful, externalised sentence for a service the
person configured at a loopback address, separate from the one the runtime alo OS
ships earns. The distinction already exists in the types (`Answers::Runtime`
against `Answers::Service`); only the words collapsed it.

**What is permitted:** D1. A configured local service is permitted under *this
machine only*, exactly as today — refusing it would break every honest vLLM user
to inconvenience nobody, since no configuration is verifiable yet. **And the
setting's own wording is corrected in the same change**, because D1 with the
words left as written is the outcome this ADR argues against in every version.

**What is not built, and is not promised:** an enforceable local-only guarantee.
It is qualified by **supervision, not ownership**, it needs a mechanism that does
not exist, and until it does, no configuration qualifies — alo OS's own included.
D4's second setting is not offered, because offering an empty guarantee is the
thing this ADR was written to prevent.

**What would overturn this:** supervision existing. Then D4 becomes real and this
ADR gets a successor, not an edit.
