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

**Status:** **Done, 2026-09-13.** **Depends on:** 4, 6, 7.
Built in a new `crates/alo-corridor` (`carried.rs`, `answered.rs`,
`door.rs`, `crossing.rs`, `dialling.rs`, `receiving.rs`, `doorway.rs`,
`holding.rs`, `replying.rs`), with `alo_turn::Arriving` gaining the two
doors an answer leaves through and `alo_nearby::http` shared between the
two wires; the report is
`docs/autonomy/updates/a-verb-crosses-between-two-machines-and-is-proven-at-the-door.md`.

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

### 9. The machine that is asked answers on the port it advertises

**Status:** **Done, 2026-09-14**, in the shape the constraint below names
for a daemon another lane owns: the asking side's door on
`alo_turn::Turning` (`crates/alo-turn/src/crossing.rs`, `became.rs`), the
outcome path on the wire (`crates/alo-corridor/src/outcome.rs`,
`THE_OUTCOME_PATH`, the three `*_through` doors on `Crossing`), and the
report saying what the daemon still owes:
`docs/autonomy/updates/the-asking-machine-crosses-through-a-turn-and-is-told-what-became-of-it.md`.
`alo-agentd` is built by the v0.01 delivery lane and the kernel-enforcement
plan, is Linux-only and gated only under WSL, and holds the one `Machine`
under the person's door's `Holding` — so the daemon's lines of the
acceptance below are **task 10**, carried there unchanged. Nothing in
`crates/alo-agentd` was touched. **Depends on:** 7, 8.

Tasks 7 and 8 built the two wires, and each takes a listener somebody bound:
`alo_nearby::Receiving` for a proposal and a confirmation,
`alo_corridor::Receiving` for a verb, and `alo-asking`'s corridor for a
question, all on the one port presence advertises. Nothing binds that port,
nothing answers discovery for a running machine, nothing holds the
`Pairings`, the `Proposals`, the `Doorway` and the `Seen` those wires share
behind one lock, and nothing writes `alo-record` when a pairing is kept. On
the asking side, a verb crosses from a `Crossing` a test built, not from a
turn: task 8's report says so, and says what the turn's door owes law 1 and
ADR 0020. This is the task that turns three libraries into a machine that
can be asked.

- **Acceptance:** one process binds the port presence advertises and answers
  discovery with that port, and on that port tells a proposal, a
  confirmation, a verb and a question apart by path, handing each to the
  crate that decides it and refusing anything else as *not for this wire* —
  tested by a request on each path reaching its door and a request on none
  reaching nothing; the `Pairings`, the `Proposals`, the `Doorway` and the
  `Seen` are one each, behind one lock, so a pairing revoked on the person's
  surface refuses the next verb and the next question alike, tested by a
  revocation between two requests on two paths; a pairing kept is written to
  `alo-record` at the one moment there is one value to write it from, and a
  proposal refused or withdrawn writes nothing, tested; the asking side has a
  door on `alo_turn::Turning` that crosses a verb the way
  `Turning::asking` puts a question — resolved before the boundary is
  entered and put from inside it (ADR 0020), the departure written down
  whether or not an answer came back, the answer or the door's word handed
  back — so an agent reaches the wire through a turn and through nothing
  else, tested beside the question's door; a local agent that calls itself
  by a machine's name is refused at the daemon's door
  (`alo_nearby::Origin::names_a_machine`), tested; and what became of a
  change an asking machine proposed — approved, declined, or lapsed — can
  be asked for on the wire, additively, so that a person is not left
  guessing whether a file moved on a machine down the corridor.
- **Constraint:** ADR 0003 and ADR 0031 throughout, and nothing that task 8
  decided is re-decided: one turn at a time per remote machine, the proof
  judged before anything else, every answer under a departure. The daemon
  is `alo-agentd`, which is Linux; **before starting, `git pull`, read this
  plan as published, and confirm which lane owns `alo-agentd`** — if it is
  another lane's, this task is the asking side's door on `Turning`, the
  outcome path on the wire and a report saying what the daemon still owes,
  and nothing in that crate. Nothing in `alo-shell`, nothing in `image/`;
  the name a person gave a paired machine is resolved through
  `alo_corridor::Naming` and shown by the shell, and the shell is outside
  this plan.

### 10. The daemon binds the port

**Status:** **Done, 2026-09-14.** Built in `crates/alo-agentd` (`wire.rs`,
`hearing.rs`, `network.rs`, `questioned.rs`, `looking.rs`, `surface.rs`,
`terms.rs`, and the serving loop), with `alo_nearby::Arrived::carried` and
`alo_corridor::Arrived::carried` taking a message the daemon read,
`alo_corridor::Doorway` gaining the doors the daemon lends the machine
through, `alo_record::Entry::paired` for the moment a pairing is kept, and
the contract in `docs/contracts/local-network-wire.md`; the report is
`docs/autonomy/updates/the-daemon-binds-the-port.md`. This lane took
`alo-agentd` for it because the task was assigned to it by the loop's owner,
gated under WSL with the count read (257 unit, 5 + 1 + 1 + 4 + 12
integration). The question path is told apart, proven and judged against the
pairing's list, and **answering it is task 11**. **Depends on:** 9.

Task 9 built the asking side's door and the outcome path and left the
daemon owing everything that turns three libraries into a machine that can
be asked. This is that, with task 9's own words:

- **Acceptance:** one process binds the port presence advertises and
  answers discovery with that port, and on that port tells a proposal, a
  confirmation, a verb, a question and an outcome apart by path, handing
  each to the crate that decides it and refusing anything else as *not for
  this wire* — tested by a request on each path reaching its door and a
  request on none reaching nothing; the `Pairings`, the `Proposals`, the
  `Doorway` and the `Seen` are one each, behind one lock, beside the
  person's door's `Holding` of the same `alo_turn::Machine`, so a pairing
  revoked on the person's surface refuses the next verb and the next
  question alike, tested by a revocation between two requests on two paths;
  a pairing kept is written to `alo-record` at the one moment there is one
  value to write it from, and a proposal refused or withdrawn writes
  nothing, tested; and a local agent that calls itself by a machine's name
  is refused at the daemon's door (`alo_nearby::Origin::names_a_machine`),
  tested — which needs a refusal `alo_capability::GrantError` does not yet
  have, and adding an arm to that closed list, with its word, is a decision
  for whoever owns that crate rather than a side effect of this task.
- **Constraint:** ADR 0003 and ADR 0031 throughout, and nothing tasks 8
  and 9 decided is re-decided: one turn at a time per remote machine, the
  proof judged before anything else, every answer under a departure, an
  outcome a read of the turn's own memory. The two `Receiving`s each take a
  listener and read the request themselves, so one port needs either a
  dispatcher that peeks the request line and hands the connection on, or an
  additive constructor on each that takes a connection already accepted —
  choose one and say why. The person's-door `Holding` and the network's
  `Doorway` borrow one machine: the lock is over the machine, and a verb
  from the network waits on a local turn as a local turn waits on it.
  Nothing in `alo-shell`, nothing in `image/`; the corridor's paths become a
  surface another alo machine speaks and get a document under
  `docs/contracts/` in the same change.

### 11. A question from a paired machine is answered by this machine's own model, and the person's door reaches a remote turn

**Status:** **Done, 2026-09-14.** Built in `crates/alo-turn/src/answering_for.rs`
(`Machine::answering_for`, `Machine::answer_returned`, and the same two on
`Arriving`), `crates/alo-agentd/src/questioned.rs` (the question answered)
and `crates/alo-agentd/src/reaching.rs` (the person's door on a remote
turn), with `alo_asking::a_question_off_the_wire` and
`alo_asking::an_answer_on_the_wire` as the receiving side of the one shape,
`alo_nearby::http::a_json_reply`, and `alo_protocol::Standing::from_a_machine`
so that what is waiting says which machine; the contract gained the `200`
answer and the `from` field additively. Which model answers for another
machine is the person's own choice for their own questions, read at every
question, and a person who chose a provider answers for nobody. The report is
`docs/autonomy/updates/a-question-from-a-paired-machine-is-answered-by-this-machines-own-model.md`.
**Depends on:** 10.

Task 10 bound the port and left two things it could not decide as side
effects. A question on `/v1/chat/completions` is told apart, proven through
the one `Seen`, and judged against the pairing's `MayAskIts::Models` arm —
and then answered `not-answered-here`, because the door that answers it is
`alo_turn::Machine`'s to grow and `arriving.rs` says a remote turn puts no
question. And a change a paired machine proposed waits for this machine's
person, who has no request on the person's door that reaches
`alo_turn::Arriving`: `alo-protocol`'s `approve`, `decline` and `waiting`
answer *nothing is happening* while a remote turn holds the machine.

- **Acceptance:** a proven question from a pairing that permits asking this
  machine's models is put to this machine's **own** model and to nothing
  else — never a provider, never another paired machine, tested by a machine
  whose person chose a provider refusing the question in words — inside no
  turn of the asking machine's, recorded as `alo_record::Entry::answered_for`
  with the origin named, its answer leaving under a departure that the
  indicator shows and the record keeps, and answered on the wire in the
  OpenAI-compatible shape the corridor already reads; a question from a
  pairing that does not permit it is still refused before anybody knows what
  it asked; a machine where nothing has been chosen to answer a question says
  so in the word it already has; and the person's door approves, declines and
  lists a change a paired machine proposed, through `alo_turn::Arriving`,
  with one approval causing exactly one execution there — the refusal paths
  beside the road.
- **Constraint:** ADR 0003, ADR 0008 and ADR 0031: the question travels, the
  grant does not; which model may answer for another machine is the
  **person's** setting and no default decides it for them; the proof is
  judged before anything else through the same `Seen`. The door on `Machine`
  or `Arriving` that puts a remote question, and the `alo-protocol` requests
  the person's door needs, are decided in the crates that own them and
  written up in the report; nothing widens the enumerated verbs, nothing in
  `alo-shell`, nothing in `image/`. The contract
  `docs/contracts/local-network-wire.md` gains the `200` answer additively.

### 12. The person's door proposes, confirms and revokes a pairing, and a pairing outlives a restart

**Status:** **Done, 2026-09-14.** Built in `crates/alo-protocol/src/pairing.rs`
(the four requests and four answers on the person's door),
`crates/alo-agentd/src/pairing.rs` (the door, against the one lock),
`crates/alo-nearby/src/keeping.rs` (the pairings as they are written down, key
included) and `crates/alo-remembering/src/pairings.rs` (the file, under the
grants file's three rules, with `believing.rs` split out to hold both), with
`alo_turn::Turning::a_pairing_was_kept`, the daemon's `KeepingPairings` and
`ThePairingsFile`, the proposal waiting on the person's door while a shell is
connected, and the contracts `docs/contracts/daemon-protocol.md` and
`docs/contracts/pairings-file.md`; the report is
`docs/autonomy/updates/the-persons-door-pairs-and-a-pairing-outlives-a-restart.md`.
The key goes in the file beside the identity, and the report says why.
**Depends on:** 10, 11.

Task 10 left two things a real machine cannot do without, and said so:
the person's surface confirms a pairing by reaching into `TheNetwork`'s lock
from outside the loop — which only a test can do — and *pairings are not
kept between restarts*, so a machine switched off at night is paired with
nothing in the morning. Task 11 finished the person's door for a change and
a question; this finishes it for the pairing itself. Nothing here decides
what a pairing is (task 2), what it leaves each machine holding (task 6), or
how a proposal crosses (task 7): it gives the person here a door onto all
three, and keeps what they made.

- **Acceptance:** the person's door (`alo-protocol`) gains requests that
  propose a pairing to a machine discovery measured, confirm the one
  waiting, revoke one, and list what is paired and what is waiting — each
  carrying nothing that could name a machine discovery did not measure, and
  each refused on the agent's door in the words an agent approving something
  gets; what the person is shown to confirm is the `Code` and the enumerated
  list, and a confirmation for nothing waiting, for a code that does not
  match, and from an agent are three refusals with three tests; a pairing
  kept is remembered under the person's own file with the same trust the
  grants file has (`alo-remembering`: owner, mode, refused whole if it cannot
  be believed) and read again at start, so that a pairing survives a restart
  and its **expiry** survives with it, tested by a restart across the moment
  it ends; a revocation from the door takes effect on the next verb and the
  next question, tested through the door rather than the lock; and the key
  the pairing holds (ADR 0031) is kept with the care a credential gets —
  which store, and why, decided in the crate that owns it and written up.
- **Constraint:** ADR 0003 and ADR 0031 throughout: no *remember this
  machine* beyond the pairing's own stated expiry, no trusted network, no
  certificate, and a remembered pairing is exactly the row two people made
  — nothing about it is widened by being written down. The surface that
  shows a proposal and a code is `alo-shell`'s and outside this plan; what
  this task owes it is the requests and the answers. Nothing in
  `alo-shell`, nothing in `image/`; `docs/contracts/daemon-protocol.md` and
  `docs/contracts/person-settings.md` (or a file beside it, decided in the
  report) gain the shapes additively.

### 13. A turn asks the local model in the envelope

**Status:** **Done, 2026-09-14.** Built in `crates/alo-turn/src/next_request.rs`
(`Turning::asking_for_the_next_request`, one road with `Turning::asking` that
differs in the pinned runtime's arm alone), `alo_turn::Answers::ThePinnedRuntime`,
`crates/alo-protocol/src/answered.rs` (an `ask` says `"answered":
"as-the-next-request"`, additively) and `crates/alo-agentd/src/the_runtime.rs`
(the runtime found here, held by its own type); the contract
`docs/contracts/daemon-protocol.md` gained the field. The Mac lane's task 15 is
unblocked. The report is
`docs/autonomy/updates/a-turn-asks-the-local-model-in-the-envelope.md`.
**Depends on:** 11.

ADR 0032 decided that an agent turn asks a model on this machine for the
protocol's envelope and the door — `read`, `propose` or `ask` — and never for
the call, because that is where a 7B model went from 40% to 87.5% on the Mac.
Its point 5 says the catalogue keeps reading the free grade *until the agent
turn asks that way*. The Mac lane's task 14 makes the ask a door in
`alo-asking`; this wires the real turn through it, which is `alo-turn` and
`alo-agentd`'s and therefore this lane's.

- **Acceptance:** the turn that asks the pinned runtime for an agent's next
  request takes `Asking::to_this_machine` with the envelope's schema, and a
  test reads the request off a socket and finds the schema and nothing about
  the call's inside; a question a person puts to a model on the same machine
  is never given the schema, held by a test beside it; a hosted provider and
  a paired machine are asked exactly as before, with a test each, because
  ADR 0032 point 4 is explicit that they are a separate measurement; and the
  turn's record entry is unchanged in shape — how the model was asked is not
  a thing the record keeps.
- **Constraint:** this depends on the Mac lane's task 14 having landed the
  door; if it has not, the honest deliverable is the finding and the task
  waits rather than building a second ask in this crate. Nothing here reads
  or rewrites a grade — which grade the recommendation reads is the Mac
  lane's task 15, unblocked by this one landing.

### 14. A turn on a machine with no model asks the paired machine its person chose

**Status:** **Done, 2026-09-14.** Built in `crates/alo-choosing/src/paired.rs`
(`Picked::FromAPairedMachine`, `AMachine`, `WhoMayBeAsked`, `machine =
"<identity>"` in the file, `Choosing::answered_by_a_paired_machine`),
`crates/alo-answering/src/refused_there.rs` (`WentWrong::RefusedThere`, three
words), `crates/alo-asking/src/refused_on_the_wire.rs` (the three wire words,
spelled once for both ends), `alo_turn::Answers::PairedMachine` (one road with a
provider's in `asking.rs`, tested in `down_the_corridor.rs`) and
`crates/alo-agentd/src/corridor.rs` (bound, pairing, discovery, question, in that
order); the contracts `docs/contracts/person-settings.md` and
`docs/contracts/local-network-wire.md` gained the shapes additively. The report
is `docs/autonomy/updates/a-turn-asks-the-paired-machine-its-person-chose.md`.
`alo-choosing` may not depend on `alo-nearby` (`tests/a_grade_travels_nowhere.rs`),
so the pairings are asked through a trait the daemon answers — task 15 gives the
person's door the request that makes that the road a shell takes.
**Depends on:** 3, 12, 13.

Task 3 built the corridor as a door — `alo_asking::Asking::to_a_paired_machine`
shows the departure, proves the question and reads the answer — and task 11
built the machine that answers it. Nothing between them is wired: a person
cannot choose a paired machine to answer their questions (`alo-choosing` names
it as a place a question *could* go and has no shape for it), `alo_turn::Answers`
has no variant for one and says so, and `alo-agentd` refuses a turn's question
to one as `Miswired::BelongsDownTheCorridor`. *A machine without a GPU
discovers the one with it, and the agents just work* is true of two doors and
of no turn. Task 13 found this while holding a paired machine to *asked exactly
as before* — which today means *refused*.

- **Acceptance:** a person can choose a machine they are paired with, by its
  identity, to answer their questions, and the choice is refused when no
  pairing permitting `MayAskIts::Models` stands — at choosing and again at
  every question, so a pairing revoked or expired between the two refuses the
  question in words, tested; a turn's question to that machine goes through
  `to_a_paired_machine` under a departure the indicator shows and the record
  keeps as left, with the machine named, tested beside a local answer that
  leaves nothing; a question the other machine refuses (not permitted, answers
  elsewhere, nothing chosen there) reaches the agent as the other machine's own
  word rendered in this machine's language, one test each; an agent's next
  request down the corridor is asked exactly as a question in words is (ADR
  0032 decision 4), tested off the socket; and a paired machine is never a
  fallback for a local model or a provider, in either direction — refused by a
  test.
- **Constraint:** ADR 0003, ADR 0008 and ADR 0031 throughout: the question
  travels, the grant does not; where a question goes is the person's setting
  and no default chooses a machine for them; an organisation's bound
  (`SourcePolicy::InTheBuilding` and its siblings) is asked before anything
  leaves. The settings shape is decided in the crate that owns the person's
  settings, additively, with `docs/contracts/person-settings.md` updated in the
  same change. Nothing in `alo-shell`, nothing in `image/`.

### 15. The person's door chooses a paired machine to answer their questions

**Status:** **Done, 2026-09-14.** Built in `crates/alo-protocol` (the request
`choose-machine-to-answer` as `FromAPerson::ChooseMachineToAnswer`, refused on the
agent's door; the answer `chosen-to-answer` as `ToAPerson::ChosenToAnswer`; and
`Paired::may_answer_questions` on the list) and
`crates/alo-agentd/src/choosing_to_answer.rs` (the one file in the daemon that
writes a person's settings, through `alo_choosing::Choosing::answered_by_a_paired_machine`
with the daemon's own pairings, under the lock), reached from the person's door
alone (`answering.rs`), with `Questions::where_the_settings_are` naming the file
the next turn reads. `alo-choosing`'s guard
`tests/no_agents_door_reaches_these_settings.rs` now holds exactly that: one file
of the daemon names the writer, and only the person's door reaches it. The
contracts `docs/contracts/daemon-protocol.md` and
`docs/contracts/person-settings.md` gained the shapes additively. The report is
`docs/autonomy/updates/the-persons-door-chooses-a-paired-machine-to-answer.md`.
**Depends on:** 12, 14.

Task 14 made a paired machine a choice a person can make and a place a turn's
question can go, and held the choice to the pairings at the moment it is made —
through `alo_choosing::WhoMayBeAsked`, because a settings store may not depend on
anything that reaches the network. The pairings themselves live in one place, the
daemon's `TheNetwork`, and nothing on the person's door reaches it to choose: a
shell that wanted to offer *the studio machine answers my questions* would have
to hold a copy of the pairings it has no way to keep true, which is how a choice
refused at choosing becomes a choice nobody refused.

- **Acceptance:** the person's door (`alo-protocol`) gains a request that
  chooses a machine this machine is paired with, by its identity, to answer the
  person's questions — answered by the daemon through
  `alo_choosing::Choosing::answered_by_a_paired_machine` with its own
  `TheNetwork` as the `WhoMayBeAsked`, so the choice is made against the list
  that refuses the next question; refused in words when no pairing permits
  asking that machine's models, when what is named is not an identity, and on
  the agent's door in the words an agent approving something gets — one test
  each, and one test that a refused choice writes nothing; the list of pairings
  on the person's door says, for each, whether it permits asking its models, so
  a shell offers only what can be chosen; and a choice made through the door is
  the one the next turn's question goes to, tested through the door.
- **Constraint:** ADR 0003, ADR 0008 and ADR 0016: the person chooses and no
  default does; the daemon writes the person's own settings only because the
  person's own shell asked it to, through `alo-choosing`'s one way out.
  `docs/contracts/daemon-protocol.md` gains the shapes additively. Nothing in
  `alo-shell`, nothing in `image/`.

### 16. A paired machine is spoken of by the name its person gave it

**Status:** ready. **Depends on:** 12, 15.

Every surface this plan built names the other machine by its identity — thirty-two
hexadecimal characters a person did not choose — because the name a person gives a
machine was declared *the shell's to keep until a home for it exists*
(`crates/alo-protocol/src/pairing.rs`), and nothing keeps one. The daemon answers
`alo_corridor::Naming` with `NoNameYet` in `crates/alo-agentd/src/starting.rs`, so
the indicator a question down the corridor fires, the record entry it leaves, a
change a paired machine proposed, the pairings list and the machine chosen to
answer questions all say `0f1e2d3c…` where the person would read *the studio
machine*. ADR 0003's *visible* is a list a person can read, and a list of
identities is one they cannot.

- **Acceptance:** the person's door (`alo-protocol`) gains a request that names a
  machine this one is paired with, by its identity, with a name the person gave
  it, and one that clears the name — each refused in words when what is named is
  not an identity or not a machine this one is paired with, and on the agent's
  door in the words an agent approving something gets, one test each; the name is
  kept on this machine in the person's own file, with the trust the pairings file
  has, and read again at start, tested across a restart; the daemon's `Naming`
  answers from it, so the indicator and the record entry of a question down the
  corridor, a change a paired machine proposed, and a question answered for a
  paired machine name the machine by that name — one test each through the door —
  and the pairings list and `chosen-to-answer` carry it beside the identity; a
  name is never sent to the other machine, never used to find, dial or prove one,
  and a revoked pairing's name goes with it, each held by a test.
- **Constraint:** ADR 0003 and ADR 0031: a name decides nothing — the identity is
  what the proof and the pairing are about, and a machine answering to a name is
  not a machine anybody paired with. Which file keeps the names, and whether it is
  the pairings file or one beside it, is decided in the crate that owns it and
  written up. Nothing in `alo-shell`, nothing in `image/`; the contracts gain the
  shapes additively.
