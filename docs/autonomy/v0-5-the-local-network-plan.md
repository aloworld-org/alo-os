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

**Status:** **Done, 2026-09-13.** **Depends on:** nothing.
Built in `crates/alo-nearby`; the report is
`docs/autonomy/updates/a-machine-says-it-exists-and-says-nothing-else.md`.

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

**Status:** **Done, 2026-09-13.** **Depends on:** 1.
Built in `crates/alo-nearby`; the report is
`docs/autonomy/updates/a-pairing-is-made-by-two-people-and-by-nothing-else.md`.

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

**Status:** **Done, 2026-09-13.** **Depends on:** 2.
Built in `crates/alo-asking/src/corridor.rs`; the report is
`docs/autonomy/updates/the-machine-down-the-corridor-is-still-an-egress.md`.

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

**Status:** **Done, 2026-09-13.** **Depends on:** 3.
Built in `crates/alo-turn/src/arriving.rs` and `crates/alo-nearby/src/origin.rs`;
the report is
`docs/autonomy/updates/what-a-remote-agent-may-do-is-what-the-local-person-granted.md`.

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

**Status:** **Done, 2026-09-13.** **Depends on:** 3.
Measured in
`crates/alo-asking/tests/an_office_that_cannot_connect_still_has_working_ai.rs`,
inside a network namespace with no route out of it, with the kernel's own
count of packets refused a route read before and after the day; the true
sentence is `alo_answering::WentWrong::NoWayThere`, said once through
`alo-telling`. The report is
`docs/autonomy/updates/an-office-that-cannot-connect-still-has-working-ai.md`.

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

### 6. The machine that asks is the machine that paired

**Status:** **Done, 2026-09-13.** **Depends on:** 2, 3, 4.
Decided in [ADR 0031](../decisions/0031-the-pairing-is-the-key.md) and built
in `crates/alo-nearby` (`keying.rs`, `proof.rs`, `proven.rs`,
`replaying.rs`), with the corridor in `crates/alo-asking` carrying the proof
and `Origin` in `crates/alo-turn`'s door made only from one; the report is
`docs/autonomy/updates/the-machine-that-asks-is-the-machine-that-paired.md`.

Task 3's report said it outright and task 4 and 5 built on it: *there is no
cryptography here and none is claimed.* A question arrives at the studio over
plain http from whoever holds the address, and the studio writes down *the
reception machine* because its pairing names one — not because the connection
proved it. Task 4 decides whose grants a verb from a paired machine is judged
by; nothing yet decides that the verb came from that machine. Before anything
carries a verb between two machines, that has to be true, or a stranger on the
office WiFi who learned an address acts under every grant B's person made to
A — which is exactly the lateral movement ADR 0003 exists to refuse.

- **Acceptance:** a pairing leaves each machine holding something made at the
  moment both people confirmed and held by nobody else, so that being
  discovered, sharing the network, or having read a `MachineId` off the wire
  confers nothing — three refusals, one test each; a question or a verb
  arriving from a paired machine is accepted only when it proves it comes from
  the identity the pairing names, and a stranger presenting A's `MachineId`
  is refused before any grant is asked; a proof replayed from an earlier
  exchange is refused; a proof from a pairing since revoked or expired is
  refused at the moment, not at the next restart; `alo_nearby::Found` carries
  the address a machine answered from, so the next hop is dialled from what
  discovery measured rather than from what a test typed — task 5 had to use
  loopback because it does not; and B's record names A because the connection
  proved it, tested by a record that stays empty when the proof fails.
- **Constraint:** no certificate authority and no trusted network (ADR 0003);
  what stands in for both is the pairing itself, which two people made. The
  primitives are rented from a pinned, audited crate and never written here.
  Which primitive, and how the secret is exchanged at pairing time between two
  machines that have never met, is a settled decision before it is code: an
  ADR under `docs/decisions/` with the options, a recommendation and the
  consequences. A worker who cannot finish this task without making that
  decision writes the ADR, hands it over as this task, and says in the report
  that the code waits on it.

### 7. A proposal reaches the other machine, and the answer comes back

**Status:** **Done, 2026-09-13.** **Depends on:** 6.
Built in `crates/alo-nearby` (`proposals.rs`, `waiting.rs`, `confirming.rs`,
`carried.rs`, `http.rs`, `dialling.rs`, `receiving.rs`, `crossing.rs`); the
report is
`docs/autonomy/updates/a-proposal-reaches-the-other-machine-and-the-answer-comes-back.md`.

Tasks 2 and 6 decided what a pairing is and what it leaves each machine
holding, and every test in this plan has made one by handing a `Proposal` and
an `Offer` from one value to another inside one process. Nothing yet carries
them between two machines: a person at reception cannot propose to the
studio, and the studio's person has nothing to confirm. Task 6's report says so
and task 4's says the same of verbs. This is the first wire, and it is the
pairing's rather than the verb's because a verb has nothing to prove itself
with until a pairing exists.

- **Acceptance:** a proposal made on one machine reaches the machine it names
  at the address discovery measured (`alo_nearby::Found::where_it_answers`),
  carrying exactly the `Proposal` — both identities, the enumerated list, the
  duration and the asking machine's `Offer` — and nothing else, held by a test
  that reads the wire and refuses any field not on that list; the asked
  machine answers with its own `Offer` and nothing else, and only once its
  person has been shown the proposal and the `Code`; the asking machine's
  person is shown the same `Code`, and a pairing is kept on each machine only
  after both people have confirmed on their own — one test each for the
  asking side confirming alone, the asked side confirming alone, and a
  proposal that arrived from an address discovery never measured; a proposal
  is refused, before anybody is shown anything, when it names a machine other
  than the one it arrived at, when its `Offer` does not read, and when a
  second proposal from the same machine arrives while the first waits; a
  proposal nobody answered expires from both machines within a stated time;
  and nothing about any of it is written in either record until a pairing is
  kept, because a proposal is not something that happened to the machine.
- **Constraint:** ADR 0003 and ADR 0031 both: no certificate, no trusted
  network, no *remember this machine*, and nothing here carries a verb or a
  question — the verb wire is the task after this one, and it calls
  `alo_turn::Arriving` and no other door, holding an `alo_egress::Departing`
  for every answer it sends back. The surface that shows a proposal and its
  `Code` is `alo-shell`'s and outside this plan; what this task owes it is a
  value with the proposal, the code and the two confirmations on it, in
  `crates/alo-nearby`, and the wire that fills it. Nothing in `image/`.

### 8. A verb crosses between two machines, and is proven at the door

**Status:** ready. **Depends on:** 4, 6, 7.

Task 7 built the pairing wire, and nothing on it carries a verb or a
question: two machines that never shared a process can now hold a pairing,
and task 4's door — `alo_turn::Arriving` — still has nothing that reaches it
from a network. Task 6's report said per-message proofs on an open turn's
later doors are the verb wire's to check, and task 4's report said the same
of the wire itself. This is that wire, and it is written after the pairing's
because a verb had nothing to prove itself with until a pairing existed.

- **Acceptance:** a verb an agent on a paired machine asks for reaches the
  machine it names at the address discovery measured
  (`alo_nearby::Found::where_it_answers`), carrying the verb, its typed
  arguments and a `Proof` over exactly those bytes — and nothing else, held
  by a test that reads the wire and refuses any field not on that list; the
  receiving machine calls `Proven::checked` before anything else and
  `alo_turn::Arriving` and no other door, so a verb from a stranger
  presenting a paired machine's identity, a verb replayed from an earlier
  exchange, and a verb from a pairing since revoked are each refused before
  any grant is asked, with the record staying empty — one test each; a verb
  the receiving machine's person has not granted is refused as *not granted
  here*, tested beside the same verb granted there and run; every answer sent
  back travels holding an `alo_egress::Departing`, tested by there being no
  road to the wire without one; the receiving machine's record names the
  origin machine and the asking machine's record names where the verb went;
  and the second message of an open turn carries a fresh proof, with the
  first one replayed refused.
- **Constraint:** ADR 0003 and ADR 0031 both: the question travels, the
  grant does not; no new verb, nothing widening the enumerated list, and
  `alo-capability`'s reasoning untouched — the wire decides *whose* grant is
  asked and how the asker is proven, never what a grant is. The kernel
  boundary (ADR 0013) applies on the receiving machine exactly as for a local
  turn, and a turn whose boundary cannot be applied still does not run. One
  port, one shape: HTTP on the port presence advertises, beside task 7's two
  paths, with the proof in `alo_asking::THE_PROOF_HEADER`. Nothing in
  `alo-shell`, nothing in `image/`.
