# v0.5 — the local network, before there is anywhere to show it

**Workstream:** the five `[v0.5]` promises under *The local network — machines
that find each other* in `docs/features.md`, and the part of *the whole of it
works with no internet at all* that is theirs.
**Why it exists:** [ADR 0028](../decisions/0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
begins v0.5's screenless work while v0.01 waits on hardware, and this is the
next partition after `v0-5-lane-b-plan.md` finished. **[ADR 0003](../decisions/0003-the-network-is-not-authority.md)
settles the model** — discovery is open, use requires mutual deliberate
pairing, pairings behave like grants, a remote agent acts only under a local
grant, and there is no trusted-network setting. Nothing here re-decides any of
that; it builds it.

**Crates this plan owns:** a new `crates/alo-nearby` for discovery and
pairing, with `crates/alo-models`, `crates/alo-egress` and
`crates/alo-remembering` touched where a paired machine is already modelled.
**Nothing in `crates/alo-shell`**, nothing in `image/` — a machine on the
network is decidable without a pixel, the way `alo-approving` and
`alo-overlay` decided their surfaces without drawing one.

**What this plan may not do** (ADR 0028's terms): move any v0.01 box, line or
wording; tick anything *on the machine*; edit a crate another lane owns. Before
writing the next task, `git pull` and read the plan as published — numbers are
a shared space, and two lanes have taken the same one twice.

## Tasks

### 1. A machine says it exists, and says nothing else

**Status:** ready. **Depends on:** nothing.

*Machines find each other with zero configuration — no addresses typed, no
accounts.* ADR 0003: **discovery reveals presence and nothing else** — no
files, no records, no model access, no agent surface. The whole risk of this
promise is in what an advertisement carries, because everything on a network
can read one, including a machine nobody has paired with and nobody owns.

- **Acceptance:** a machine advertises itself over mDNS/DNS-SD as an alo
  machine, with a stable identity of its own and **no person's name, no
  account, no model list, no grant, no hostname a person chose**, held by a
  test that reads the record back and refuses any field not on a named list;
  what it advertises is the same whether anything is paired or not, so presence
  never leaks what a machine is doing; a second machine on the same network
  reads it and answers with *one machine, not paired*; and the identity is
  stable across a restart but is not a hardware serial, because an identifier
  that outlives a reinstall is a tracker.
- **Constraint:** nothing here connects to anything it finds, and nothing here
  is configurable — no "advertise as", no "discovery off" setting that would
  become the trusted-network switch ADR 0003 forbids by the back door. A
  machine with no network answers *nothing found*, which is an answer rather
  than a failure, the way `alo_models::found_on_this_machine` already does.

### 2. Pairing is mutual, deliberate, and refused every other way

**Status:** ready. **Depends on:** 1.

*Pairing: mutual, deliberate, enumerated, revocable in one action, and
expiring — grants, across a machine boundary.* This is the task the whole
promise rests on, and it is written second because a pairing nobody can refuse
is worse than no pairing at all.

- **Acceptance:** a pairing is made only when **both machines' people confirm
  it**, each on their own machine, and a confirmation from one side alone
  leaves nothing paired — the refusal path is the subject here and carries as
  many tests as the agreement; a pairing **expires** by default and the
  duration is stated where it is made rather than hidden in a constant; it is
  **revocable in one action taking effect immediately**, tested by a revocation
  mid-use; what it permits is **enumerated** — one list, in words
  `alo-saying` collects, and a pairing that permitted "everything" would be a
  grant nobody could read; and being on the same network, sharing a WiFi
  password, or having paired before confers **nothing**, each refused by a test
  named for the thing it refuses.
- **Constraint:** no trusted-network setting, no subnet rule, no certificate
  authority, no *remember this machine* — ADR 0003 names each as the whole
  vulnerability. Nothing here grants anything to a remote agent: pairing lets a
  machine **ask**, and task 4 is what asking costs.

### 3. One GPU box serves the office, and the indicator still fires

**Status:** ready. **Depends on:** 2.

*A machine without a GPU discovers the one with it, and the agents just work.
The inference never leaves the building; it moves down the corridor.* The
second sentence is the one that can quietly become untrue: **it is still
egress, and the indicator still fires** (ADR 0003). *"It only went to the
machine down the hall"* is exactly the sentence this product exists to refuse.

- **Acceptance:** a machine with no model of its own can put a question to a
  paired machine that has one, and the answer comes back; `InferenceSource`
  already carries `PairedMachine` and the egress indicator **fires** for it,
  tested beside a local answer where it does not; what a person is shown names
  the machine that answered rather than implying it stayed here; the paired
  machine's own record carries the question as having come from elsewhere, with
  the asking machine named as the origin; and an unpaired machine offering
  inference is not used, however convenient — refused by a test.
- **Constraint:** the question travels, the grant does not (ADR 0003): the
  answering machine evaluates against **its own** grants and verb list, its
  person approves any change from their own surface, and the asking machine's
  grants confer nothing. Nothing here makes a paired machine a fallback for a
  local model — that would be the silent substitution `docs/features.md`
  refuses four times.

### 4. What a remote agent may do is what the local person granted

**Status:** ready. **Depends on:** 3.

ADR 0003's sharpest line, and the one a reader will most want proof of: *an
agent on machine A that reaches machine B is bound by the grants made on B, by
B's person. A's grants confer nothing on B. Pairing lets A ask; it never lets A
act.*

- **Acceptance:** a verb arriving from a paired machine is evaluated against
  the **receiving** machine's grants, shown to the **receiving** machine's
  person for approval, and recorded there with the origin machine named — three
  tests, one each; a verb the receiving machine's person has not granted is
  refused **even where the asking machine's person granted it**, which is the
  test this task exists for; and the refusal says the grant was not made here,
  rather than blaming the person who asked.
- **Constraint:** no new verb, and nothing here widens the enumerated list.
  `alo-capability` decides what a grant is and this task does not touch that
  reasoning — it decides **whose** grant is asked. The kernel boundary
  (ADR 0013) applies on the receiving machine exactly as it does for a local
  turn, and a turn whose boundary cannot be applied still does not run.

### 5. An office that cannot connect still has working AI

**Status:** ready. **Depends on:** 3.

*The whole of it works with no internet at all.* The measurement, in the shape
`a_day_that_never_left.rs` already uses for one machine: a working day across
two machines with **no route off the local network**, and nothing that fails
for the want of one.

- **Acceptance:** with every non-local address unreachable, two paired machines
  discover each other, pair, ask and answer, and the record and the indicator
  are as they would be with a connection — measured rather than reasoned, with
  the unreachability enforced in the test rather than assumed; nothing anywhere
  in the road retries against a public address, checked by a test that fails if
  a single packet is addressed off-network; and what a person is told about the
  machine's connectedness is true and said once, through `alo-telling`.
- **Constraint:** nothing here is a mock of the network. If a genuine
  no-internet measurement cannot be made in this repository's test
  environment, the honest finding is the deliverable and it goes in
  `docs/quirks.md` with what was tried — not a test that asserts a promise by
  asserting a fixture.
