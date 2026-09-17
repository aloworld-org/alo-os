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
**From 2026-09-15, `alo-capability`'s `Reach`, `Ask` and `Grantee`, and the
grants file's format in `alo-remembering`, are the applications plan's**
([ADR 0040](../decisions/0040-what-an-applications-grant-is-over.md)): no task
here edits them, and a task that finds it needs to is a finding, not an edit.
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

**Status:** **Done, 2026-09-14.** Built in `crates/alo-remembering` (`named.rs` —
the rule a name is held to, `MachineName`; `names.rs` — `MachineNames` and the
file's shape, reading back only names of machines still paired;
`machine_names.rs` — `/var/lib/alo/machine-names.toml` under the pairings file's
trust), `crates/alo-protocol` (`name-machine` and `clear-machine-name` as
`FromAPerson::NameMachine` and `ClearMachineName`, refused on the agent's door;
the answer `machine-named` as `ToAPerson::MachineNamed` with `AfterNaming`; and
`called` beside the identity on `Paired` and `chosen-to-answer`) and
`crates/alo-agentd` (`names.rs` — `TheNames`, the daemon's `alo_corridor::Naming`,
behind its own lock taken after the network's; `keeping_names.rs` — the file;
`naming_machines.rs` — the door; the name forgotten when a pairing is revoked or
kept afresh, in `pairing.rs` and `hearing.rs`; read at start in `src/main.rs` and
handed to `starting::until_stopped`, which replaces `NoNameYet` with it). The file
is beside the pairings file rather than in it, because a pairings row is exactly
what two people made. Contracts: `docs/contracts/machine-names-file.md` (new),
`daemon-protocol.md`, `pairings-file.md` and `local-network-wire.md`, additively.
The report is `docs/autonomy/updates/a-paired-machine-is-spoken-of-by-its-given-name.md`.
**Depends on:** 12, 15.

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

### 17. A self-hosted workspace on the network is found, not configured

**Status:** **Done, 2026-09-14.** Built in `crates/alo-nearby` (`workspace.rs` —
`WORKSPACE_SERVICE` beside `SERVICE`, `WorkspacePresence`, and `FoundWorkspace`,
which has no public constructor; `advertising.rs` — `about_a_workspace` and
`a_question_for_workspaces`, through the same record writer a machine's presence
uses; `reading.rs` — `a_workspace_in`, refusing any `TXT` entry but `v=1`;
`looking.rs` — `Looking::around`, machines and workspaces in one window),
`crates/alo-protocol` (`workspaces` as `FromAPerson::Workspaces`, refused on the
agent's door; the answer `workspaces` as `ToAPerson::Workspaces` of
`FoundWorkspace`s) and `crates/alo-agentd` (`listing_workspaces.rs`, reached from
the person's door alone; `LookingFor::look_around` and `looking::around_at`, which
the shipped `Wire` asks the link with). A name is shown beside a workspace only
where the paired machine of that identity answered from the same address in the
same look. `crates/alo-changing/src/door.rs` gained the one arm its exhaustive
match needed. Contracts: `docs/contracts/local-network-wire.md` (*A workspace on the
network*, new) and `daemon-protocol.md`, additively. The report — including the
decision that reaching a found workspace does not require the host to be an alo
machine, and why that narrows nothing — is
`docs/autonomy/updates/a-self-hosted-workspace-is-found-not-configured.md`.
**Depends on:** 1, 16.

`docs/features.md` promises for v0.5 that *a self-hosted workspace on the network
is discovered, not configured — no DNS step*, and ADR 0003 names it as the second
thing discovery is for. Nothing in this plan has built it: discovery
(`alo_nearby::SERVICE`, task 1) finds alo machines, and a workspace — mail, files,
chat and documents, the `alo-workplace` repository's — is found by nobody. A
person joining an office still has to be told an address and type it, which is
the step the promise removes, and a typed address is the one thing ADR 0003 says
nothing on this machine ever dials.

- **Acceptance:** discovery on this machine finds a workspace a machine on the
  local network advertises, by a DNS-SD service of its own declared in
  `alo-nearby` beside `SERVICE`, and reads from the advertisement exactly a closed
  list — which workspace, where it answers, and the version it speaks — refusing
  anything else it carries rather than reading around it, tested with an
  advertisement carrying one field more; the person's door (`alo-protocol`) gains
  a request listing the workspaces found at the moment, each with the address
  discovery measured and the name of the paired machine hosting it where the
  person gave one (task 16), refused on the agent's door in the words an approval
  gets, tested; **finding a workspace confers nothing** — no request, verb or
  question reaches one because it was found, and nothing on this machine connects
  to it until a person acts, tested by a workspace advertised on a network where
  nothing is paired and nothing is contacted; and an address a person types is
  never dialled as a workspace, tested by its refusal.
- **Constraint:** ADR 0003 throughout: discovery is open and reveals presence
  only, use requires the person's deliberate act, and there is no trusted-network
  setting. What a workspace *is* and what serves one is `alo-workplace`'s and
  outside this repository; what this task owes it is the service name and the
  closed advertisement, written up in `docs/contracts/local-network-wire.md`
  additively so that repository can advertise against it. Whether reaching a
  found workspace requires a pairing permitting `MayAskIts::Workspace` — and so
  whether a workspace host must be an alo machine — is decided in the report,
  and an ADR is written first if the answer would narrow the promise. Nothing in
  `alo-shell`, nothing in `image/`.

### 18. The person opens a found workspace, at the address measured at that moment

**Status:** **Done, 2026-09-14.** Built in `crates/alo-protocol` (`open-workspace` as
`FromAPerson::OpenWorkspace { machine }`, refused on the agent's door; the answer
`workspace-opened` as `ToAPerson::WorkspaceOpened`, in `FoundWorkspace`'s shape),
`crates/alo-record` (`Happened::WorkspaceOpened { workspace, answers_at }`, tag
`workspace-opened`, no agent, not egress; `Entry::a_workspace_was_opened`),
`crates/alo-turn` (`a_workspace_was_opened` on `Machine`, `Turning` and `Arriving`),
`crates/alo-recounting` (the clause `recounting.outcome.workspace-opened`) and
`crates/alo-agentd` (`opening_workspaces.rs` — identity, then the link at the moment,
then exactly one address, then the record, then the answer; three new refusals in
`words.rs`; `Holding::a_workspace_was_opened`; `listing_workspaces::drawn`, the
listing's naming rule, now shared by both). `crates/alo-changing/src/door.rs` gained
the one arm its exhaustive match needed. Contracts: `daemon-protocol.md`
(`open-workspace`), `record-file.md` (`workspace-opened`) and `local-network-wire.md`,
additively. The report is
`docs/autonomy/updates/the-person-opens-a-found-workspace.md`.
**Depends on:** 17.

Task 17 made a workspace on the network something a person is shown — which one,
where discovery measured it answers, and the name of the paired machine hosting it —
and deliberately nothing more: finding one confers nothing, and no request reaches
one. Its report decided what reaching one takes: **the person's own act**, answered
by the workspace's own sign-in, with no pairing required and no requirement that
the host be an alo machine; and **a pairing permitting `MayAskIts::Workspace`**
for anything this machine or its agents do with a workspace on another alo machine.
Nothing yet carries the first of those from the list a shell draws to the workspace
client in the person's session, so a shell holding a found workspace would have to
remember an address from a list that has since aged — which is a typed address by
another route.

- **Acceptance:** the person's door (`alo-protocol`) gains a request that opens a
  found workspace **by its identity** and nothing else, answered by the daemon
  looking around at that moment (`LookingFor::look_around`) and answering with the
  one address that workspace answered from now — refused in words when what is
  named is not an identity, when no workspace of that identity answered, and when
  more than one address answered for the same identity (a claim nobody can tell
  apart is not opened), one test each, and each refusal contacting nothing; a
  request carrying an address is not a request, tested; an agent sending it is
  refused in the words an approval gets, tested; opening writes the record entry
  *a workspace opened by the person*, naming the identity and the address, and the
  daemon itself still connects to nothing, tested with a listener at that address
  that is never connected to.
- **Constraint:** ADR 0003 throughout. The daemon hands the address to the
  person's session and dials nothing; what connects is the workspace client,
  which is `alo-workplace`'s, in the person's session, under the person's own
  account. Nothing here lets an agent reach a workspace, and nothing here treats
  a pairing as a sign-in. `docs/contracts/daemon-protocol.md` and
  `docs/contracts/record-file.md` gain the shapes additively. Nothing in
  `alo-shell`, nothing in `image/`.

### 19. A turn shows a model the words the product wrote

**Status:** **Done, 2026-09-15.** Built in `crates/alo-turn/src/next_request.rs`
(`Held::shown`: an agent's next request is put to a model in
`alo_instructing::shown_to_a_turn` over the turn's own machine's registry,
wherever the person chose — a provider, a service and a paired machine are
shown the same text, and only the pinned runtime is also held to the envelope)
and the one line of `crates/alo-turn/src/asking.rs` that builds the question
from it; a question in words and a question a paired machine asks this one are
untouched, each held by a test off the socket. **A client's own instructions
are wrapped**: what an agent sends is the request, beneath the product's
instructions, and nothing replaces them. The contract
`docs/contracts/daemon-protocol.md` says so. The measuring lane's task 19 is
unblocked. The report is
`docs/autonomy/updates/a-turn-shows-a-model-the-words-the-product-wrote.md`.
**Depends on:** 13.
**Written by the measuring lane** (task 18 of
`docs/autonomy/v0-5-the-models-measured-plan.md`), whose crates end at the words.

Task 13 landed the ask: an agent's next request goes to the pinned runtime held
to the protocol's envelope. What the model is *shown* is still a `&str` from
whoever called the door — `Turning::asking_for_the_next_request`'s own rustdoc
says *what the agent composed for the model — its instructions, the verbs and
what the person said* — and nothing in this repository composes one. So a model
on a shipped machine is shown words nobody here wrote, and no grade in the
catalogue is about them ([ADR 0037](../decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md),
and ADR 0034's second cost before it).

The words now exist: `alo_instructing::shown_to_a_turn(verbs, request)` — how to
answer under the set ADR 0037 names, every verb in the sentence the verb itself
declared, the person's request last — in a crate carrying the verb registry and
SHA-256 and nothing else.

- **Acceptance:** an agent's next request is put to a model in the text
  `alo_instructing::shown_to_a_turn` builds from the machine's own registry and
  the request the agent's question carries, with a test reading the request off
  the runtime's socket and finding the instructions' digest, every verb the
  registry declared and the person's words in that order; a question a person
  asks is untouched — their words reach a model as they wrote them, held by the
  test that already says a person's question is never given the envelope; a
  question a paired machine asks is untouched in the same way; and the report
  says what a client's own instructions now do — whether they are refused,
  ignored or wrapped — because a door that accepts both would be two prompts
  again.
- **Constraint:** the words are not edited here and no second copy of them is
  made: a turn shows what that function returns, so a change to the text is a
  change to one digest. Nothing about grades moves in this repository's
  catalogue on this task — that is the measuring lane's task 19, which is
  blocked on this one landing. `alo-instructing` is a dependency, never a place
  to write: if a turn needs words the crate does not build, the deliverable is a
  finding in the report.

### 20. An alo machine that hosts a workspace answers for it, and says nothing more

**Status:** **Done, 2026-09-14.** Built in `crates/alo-nearby` (`answering.rs`, split
out of `looking.rs` — `Answering::hosting_a_workspace_at(NonZeroU16)`, which takes a
port and nothing else so the workspace answer is always under the responder's own
identity, and `Answering::workspace`; `reading::a_question_for_workspaces_in` beside
`a_question_in`, one rule for both) and `crates/alo-agentd` (`hosting.rs` —
`THE_HOSTED_WORKSPACE` at `/etc/alo/workspace.toml`, `hosted_at` and `advertised`,
believed from root alone, one key `port`, 1–65535 and not the wire's own `7610`;
`NotHosting` in `refusing.rs`; `Wire::hosting` and `Wire::hosts`; `src/main.rs` reads
the file once at start, and a refused file is a line in the service log and no
workspace advertised, never a stopped service). The file is read at the next start,
not on a knock — decided in `hosting.rs` and the report. Contracts:
`docs/contracts/hosted-workspace-file.md` (new) and `local-network-wire.md`,
additively. The report is
`docs/autonomy/updates/an-alo-machine-answers-for-the-workspace-it-hosts.md`.
**Depends on:** 17, 18.

Task 17's contract says a workspace on an alo machine is advertised under **that
machine's own identity**, and tasks 17 and 18 built everything that reads such an
advertisement. Nothing on an alo machine writes one. `alo-agentd` already owns this
machine's discovery responder (`alo_nearby::Answering`, bound by `crate::wire::Wire`
on the port every mDNS responder shares), and it answers only the question for
`_alo-os._tcp.local`; the question for `_alo-workspace._tcp.local` is stepped over. So
a workspace served by `alo-workplace` on an alo machine can be found only if a second
responder races the daemon for the same socket and invents the machine's identity for
itself — which is a machine saying something about itself that the service holding
its identity did not say.

- **Acceptance:** `alo-nearby`'s `Answering` gains the workspace question: it answers
  `_alo-workspace._tcp.local` with exactly the closed advertisement
  (`advertising::about_a_workspace`) when it was given a `WorkspacePresence`, and
  steps over the question when it was not — one test each, and the machine's own
  presence answer unchanged byte for byte either way, tested; the identity in a
  workspace answer is always the machine's own, and there is no constructor through
  which another identity reaches the responder, tested by the refusal (a
  `compile_fail` example or its equivalent); `alo-agentd` reads which port this
  machine's workspace answers on from one root-owned file under the trust the
  pairings file has — absent means nothing is advertised, and a file that is not
  root's, is writable by anyone else, carries any key but the port, or names a port
  outside 1–65535 is refused with the machine advertising no workspace, one test
  each; an agent cannot write, name or change it, and no request on either door does,
  tested; a machine advertising a workspace is found and opened by task 18's request
  from a second daemon over loopback, end to end, tested.
- **Constraint:** ADR 0003: discovery reveals presence only, so the answer carries the
  three records and nothing else, and hosting a workspace grants nothing and pairs
  nothing. The file is a config key and so a public surface: a new contract under
  `docs/contracts/`, additive, naming who writes it (the package that installs the
  workspace server, as root — `alo-workplace`'s, outside this repository) and that the
  daemon only reads it. Whether a change to the file is read at the next start or on a
  knock is decided in the crate and written up. Nothing in `alo-shell`, nothing in
  `image/`.

### 21. The person sees what their machine says about itself on the network

**Status:** **Done, 2026-09-15.** Built in `crates/alo-protocol` (`advertised.rs` —
`advertised` as `FromAPerson::Advertised`, carrying nothing and refused on the
agent's door; the answer `advertised` as `ToAPerson::Advertised` of `Advertised`:
`machine`, `port`, and `HostedWorkspace` — `hosts`, `hosts-none` or
`not-advertised` with a `Wording`), `crates/alo-agentd` (`unhosted.rs` — the eleven
`NotHosting` arms grouped into three a person can act on, each one sentence in
`words.rs` naming no path, owner or mode; `hosting::Hosted`, which `advertised` now
returns and `Wire::hosting` takes, so the wire keeps why a file was refused;
`Wire::advertising`; `what_is_advertised.rs`, answering from the wire's
`Advertising` handed in on `Nearby`, reading no file and writing no record) and
`crates/alo-changing/src/door.rs` (the one arm its exhaustive match needed).
Contracts: `daemon-protocol.md` (`advertised`, new) and `hosted-workspace-file.md`,
additively. The report is
`docs/autonomy/updates/the-person-sees-what-their-machine-says-on-the-network.md`.
**Depends on:** 1, 20.

Since task 20 an alo machine can say two things on the local network: that it exists,
under its identity and the wire's port, and — where root installed a workspace
server — that it hosts a workspace at a port. Both are said to everything on the link,
and neither is shown to the person whose machine is saying it. Law 1 is about egress
an agent causes, but its reason — *nothing leaves silently* — is a person's to check,
and today a person cannot ask their own machine what it is telling the office without
a packet capture. A refused workspace file is worse: the only sentence about it is a
line in a service log the person does not read, and their colleagues simply do not
find the workspace.

- **Acceptance:** the person's door (`alo-protocol`) gains a read-only request
  answering what this machine advertises at the moment it is asked — its identity, the
  port its presence names, and the port of the workspace it hosts or that it hosts
  none — with no field for anything the advertisement does not carry, tested by the
  answer's shape; where the workspace file was refused, the answer says so in the
  person's language (`alo-strings`, declared and collected) naming no path, owner or
  mode, tested for each `NotHosting` arm grouped into what a person can act on; the
  request changes nothing — no file is read again, nothing is advertised that was not,
  nothing is written to the record — tested; an agent sending it is refused in the
  words an approval gets, tested; and a request carrying a port or a path is not a
  request, tested.
- **Constraint:** ADR 0003: this shows presence, it does not change it, so there is
  still no *advertise as*, no *discovery off* and no setting. What the daemon reports
  is what `Wire` holds, never the file re-read: the answer describes the service that
  is running. `docs/contracts/daemon-protocol.md` gains the shape additively. Nothing
  in `alo-shell`, nothing in `image/`.

### 22. A machine on two networks is found on each of them

**Status:** **Done, 2026-09-15.** Built in `crates/alo-agentd` (`networks.rs` —
`discovery_networks`, a pure function of the interfaces the kernel reports: up and
running, multicast, an IPv4 address, not loopback; `joined_on`, where a network that
will not join is a line and the others are joined; `route_messages.rs` — the
kernel's `RTM_GETLINK`/`RTM_GETADDR` answers read without `unsafe`, refusing a
message cut short; `joining.rs` — `Joining`, the group joined on every network at
start and again whenever the kernel's routing socket says a network changed;
`unix.rs` — the routing socket; `Wire::bound` joins through `Joining`,
`Wire::networks_waiting_on`, `networks_changed` and `joined`, waited on by
`serving.rs`; `looking.rs` — a look at the group asks from each network's own
address and merges what each heard; `listing_workspaces::drawn` names a host only
where it answered from the same address on every network) and `crates/alo-nearby`
(`heard_on_each.rs` — `Around::heard_on_each`: a machine heard on two networks is
one `Found` with `also_at`, a workspace heard alone on each network is one
`FoundWorkspace`, and two claims on one network stay two, so task 18's refusal
holds; `proposals.rs` judges a proposal against every measured address). **An
interface appearing after start is followed through the kernel's notification**,
not read at the next start — a laptop docked after boot would otherwise be absent
from the wired network all day. Tested on a real kernel by
`crates/alo-agentd/tests/a_machine_on_two_networks.rs` (two `veth` networks
between two user-namespaced network namespaces, touching no kernel-global state).
Contract: `docs/contracts/local-network-wire.md` (*A machine on more than one
network*, new, additive). `docs/quirks.md` records the kernel's behaviour. The
report is `docs/autonomy/updates/a-machine-on-two-networks-is-found-on-each.md`.
**Depends on:** 1, 10, 21.

*Machines find each other with zero configuration.* A machine in an office is often
on more than one network at once — a docked laptop on the wired LAN and on Wi-Fi, a
GPU box with two ports. `alo-agentd`'s `Wire::bound` joins the multicast group with
`join_multicast_v4(&THE_ADDRESS, &Ipv4Addr::UNSPECIFIED)`, which asks the kernel to
choose **one** interface, and the questions `crate::looking` sends leave by the
default route. So a machine on two networks is found on whichever one the kernel
picked and silently absent from the other, and its person cannot tell — the answer
task 21 gives says what the machine advertises, not where. A colleague on the other
network is left typing an address, which is the step the promise removes.

- **Acceptance:** the daemon joins the discovery group on **every** interface that is
  up, multicast-capable and carries an IPv4 address, and not on loopback — tested
  with the enumeration a pure function of the interfaces the kernel reports, one
  test each for an interface that is down, one without multicast, one without an
  address and loopback; an interface that cannot be joined is a line in the service
  log and the others are still joined, never a stopped service, tested; what is
  advertised on each network is the same bytes — the same identity, the same port,
  the same workspace answer — so presence never differs by network, tested; the
  person's `looking` asks on each joined interface and a machine heard on two of
  them is one machine with the address it answered from on each, tested with two
  interfaces a test can hold on one host (a network namespace pair, or `veth`, run
  under `alo_bounding::Waited::on_this_kernel()` if it touches kernel-global state);
  and `advertised` (task 21) is unchanged in shape, because what is said is the
  same on every network — tested by the answer read back byte for byte.
- **Constraint:** ADR 0003: discovery reveals presence only, and there is still no
  setting — no "discover on", no interface chosen by a person or an agent, because a
  list of networks to advertise on is the trusted-network switch by another name.
  An interface appearing or going after start is decided in the crate and written up
  (read again at the next start, or followed through the kernel's own notification)
  with the reason. What reality does that the specification does not say goes in
  `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 23. Two machines with no IPv4 address between them still find each other

**Status:** **Done, 2026-09-15.** Built in `crates/alo-nearby` (`heard_from.rs` —
`HeardFrom`, a measured address with the interface a link-local one was heard on,
without which a link-local address names no network; `reading::a_machine_heard` and
`a_workspace_heard`, refusing an answer from a link-local address with no scope as
`NotNearby::NamesNoNetwork`; `THE_IPV6_ADDRESS`; `Found` and `FoundWorkspace` carry
`HeardFrom`, so `where_it_answers` is scoped) and `crates/alo-agentd`
(`networks.rs` — `link_local_networks` and `every_discovery_network`, every IPv4
network first; `route_messages.rs` — every family asked, a tentative or failed
IPv6 address stepped over; `joining.rs` — `ff02::fb` joined on its interface beside
IPv4, following `RTMGRP_IPV6_IFADDR`; `unix.rs` — the IPv6 discovery socket with
`IPV6_V6ONLY` set and the port's listener in both families; `wire.rs` — a second
`Answering` holding the same presence and workspace, `Knocked::from` a `HeardFrom`;
`looking.rs` — each link-local network asked from its own address, a proposal
measured in its connection's family and scope; and `answering_discovery.rs` —
**discovery answered on a thread beside the service**, because a daemon waiting on
its own person's proposal could not answer the asked machine's measurement of it,
so two real daemons could never pair). **When both families answered, a pairing
dials the IPv4 address**, decided in `crate::looking` and written up in the report.
Tested on a real kernel by `crates/alo-nearby/tests/asked_and_answered_over_ipv6.rs`
and `crates/alo-agentd/src/two_machines_with_no_ipv4.rs` (two daemons, each running
the whole service, on one `veth` with link-local IPv6 only, pairing through task
12's request; then IPv4 added to the same cable). Contract:
`docs/contracts/local-network-wire.md` (*A network with no IPv4 address*, new,
additive). `docs/quirks.md` records the kernel's behaviour. The report is
`docs/autonomy/updates/two-machines-with-no-ipv4-still-find-each-other.md`.
**Depends on:** 1, 22.

*Machines find each other with zero configuration — no addresses typed.* Since task
22 discovery is joined and asked on every network a machine is on — every network
**with an IPv4 address**. A network without one is not rare where configuration is
absent: two machines joined by one cable with no DHCP server between them, an office
switch whose router is down for the afternoon, or a network run IPv6-only. Every one
of those still gives each interface an IPv6 link-local address with nobody
configuring it, and mDNS has a group for exactly that (`ff02::fb`, RFC 6762 §3). An
alo machine answers on none of them, so the two machines on the one cable are
strangers to each other, and the person is left typing an address — which is the
step the promise removes, and a typed address is what ADR 0003 says nothing dials.

- **Acceptance:** `alo-nearby` asks and answers over IPv6 as it does over IPv4, with
  the same closed advertisement and the same refusals (a packet saying more than
  presence is refused whichever family carried it), tested; `alo-agentd` joins
  `ff02::fb` on every interface that is up, multicast-capable and carries an IPv6
  link-local address, and not on loopback — the enumeration a pure function of what
  the kernel reports, one test each for down, no multicast, no link-local address and
  loopback — with an interface that cannot be joined a line in the service log and
  never a stopped service, tested; a found machine's address carries the interface
  it was heard on where the address is link-local (a scope id), because a link-local
  address without one names no network and cannot be dialled, tested; a machine
  heard over IPv4 and IPv6 on one network is one machine with an address in each
  family, tested; what is said is the same identity, port and workspace answer in
  both families, tested byte for byte; and two machines whose shared network has no
  IPv4 address find each other and pair through task 12's request end to end, tested
  with two network namespaces joined by a `veth` pair carrying link-local IPv6 only.
- **Constraint:** ADR 0003 throughout: discovery reveals presence only, and there is
  still no setting — no "IPv6 on/off", no family chosen by a person or an agent.
  Which address a pairing dials when both families answered is decided in the crate
  and written up with the reason. `docs/contracts/local-network-wire.md` gains the
  IPv6 group and the scope rule additively. What reality does that the specification
  does not say goes in `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 24. A paired machine reached over IPv6 link-local is still a departure the kernel bounds and the indicator shows

**Status:** **Done, 2026-09-15.** Decided first in
[ADR 0041](../decisions/0041-a-link-local-departure-names-its-interface.md): **a
departure to an address that needs an interface carries it, and one with no
interface is nowhere** — matching the address and port alone would have widened a
link-local departure to every link. Built in `crates/alo-bounding-map`
(`departure.rs` — `Departure::on`, `needs_an_interface` (the kernel's own
`__ipv6_addr_needs_scope_id`), `Departure::interface` and `names_its_network`, the
interface packed into bits that were zero so the map is not a byte wider;
`Departures::holds` refuses a destination that names no network; `field.rs` —
`SockBoundInterface` and `MessageNameLength`), `crates/alo-bounding-kernel`
(`departing.rs`, split out of `deciding.rs` — the interface read from the
`sockaddr_in6` scope when the caller's length carries one, else from the socket's
`skc_bound_dev_if`, for `connect`, `sendto` and a joined socket's peer; the
offsets map eighteen slots for sixteen fields), `crates/alo-bounding` (the two
offsets found and width-checked, the fixture), `crates/alo-agentd` (`bounding.rs` —
a scoped address registered on its interface, an unscoped link-local one refused
before a boundary with the address named in the service log) and `crates/alo-turn`
(`asking.rs` — a scoped literal registered as written rather than handed to the
resolver). **The indicator and the record name a paired machine by its name in both
families and print no address**, decided in the ADR. Tested on a real kernel by
`crates/alo-bounding/tests/a_link_local_departure_names_its_interface.rs` (the same
link-local address on two interfaces in a namespace of its own, every road the
kernel takes an interface from — refused on the one nobody showed, and a mutation
that drops the interface makes every refusal a reach) and
`crates/alo-agentd/src/a_paired_machine_over_link_local.rs` (a studio found only
over IPv6 on a `veth` cable, asked from a turn bounded by the real programme,
shown and recorded; the studio's address on another interface refused with
`EACCES` and counted as never arriving; then IPv4 added and the departure recorded
over it the same value). `docs/contracts/local-network-wire.md` gains one additive
line; `docs/quirks.md` records the kernel's behaviour. The report is
`docs/autonomy/updates/a-paired-machine-over-link-local-is-a-bounded-departure.md`.
**Depends on:** 3, 14, 23.


*One GPU box serves the office — it is still egress, and the indicator still fires.*
Since task 23 a paired machine on a network with no IPv4 address is found, paired
with and dialled at a **scoped link-local IPv6 address** (`fe80::…%3`). A question
put to it from a turn leaves under the turn's kernel boundary (ADR 0020) and under
the egress indicator — both of which were built and tested with IPv4 destinations.
`alo-bounding-map`'s `Departure` has an IPv6 shape and `alo-bounding-kernel` reads a
`sockaddr_in6`, but a departure is an address and a port with **no interface**, and
nothing has yet shown that a link-local destination is permitted when it was shown,
refused when it was not — including the same link-local address on **another**
interface — and named by the indicator in words a person can read. A departure
whose check does not match what the dial actually does is law 1 failing quietly on
exactly the network this release just made to work.

- **Acceptance:** a question to a paired machine at a scoped link-local address,
  from a turn bounded by the real programme, is reached when that departure was
  registered and refused with `EACCES` when it was not, tested on a real kernel
  under `alo_bounding::Waited::on_this_kernel()`; how a departure treats the scope —
  matched, or deliberately not, with the reason — is decided in the crate that
  holds the shape and tested both ways (the registered interface, and the same
  address on an interface nobody showed); the indicator and the record name the
  paired machine and its link-local destination exactly as they name an IPv4 one,
  tested; and a question from a turn to a paired machine found **only** over IPv6
  still leaves nothing that escapes the indicator, tested end to end.
- **Constraint:** ADR 0003, ADR 0007 and ADR 0020 as they stand: no departure is
  widened to a prefix, an interface or "the local network", and loopback stays the
  only unchecked destination. Editing `alo-bounding`, `alo-bounding-map` or
  `alo-bounding-kernel` is coordinated with the lane that owns them first (ADR
  0028); if the shape of a departure has to change, that is an ADR before it is
  code. What reality does that the specification does not say goes in
  `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 25. A private IPv4 address on two networks is still one destination the kernel bounds

**Status:** **Done, 2026-09-15.** Decided first in
[ADR 0044](../decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md):
**a paired machine's IPv4 departure is held to the interface of the network it was
found on, and checked against the interface the socket is held to** — deciding by
the route is not buildable in an LSM hook and is not what the person was shown, and
deliberately not is law 1 failing on the commonest network there is. Built in
`crates/alo-bounding-map` (`departure.rs` — `keeps_its_interface`, every IPv4
address keeping the interface it is given, and `Departure::permits`: held to an
interface, that interface; held to none — a provider's — any, as before),
`crates/alo-bounding-kernel` (`departing.rs` — `skc_bound_dev_if` read for every IPv4
destination, named or joined, and **a message carrying control messages decided as
held to no interface**, because on IPv4 `IP_PKTINFO` sends a datagram past the
socket's interface and the kernel does not check; the offsets map nineteen slots for
seventeen fields), `crates/alo-bounding` (`msg_controllen` found and width-checked,
the fixture), `crates/alo-nearby` (`HeardFrom::on_the_network` and
`HeardFrom::interface`, never spelled in the address), `crates/alo-agentd`
(`looking.rs` — each IPv4 network asked from a socket held to its interface and what
it heard written down on it; `corridor.rs` — the question held to that interface;
`bounding.rs` — `carrying_out_a_departure_on`), `crates/alo-asking` (`held_to.rs` —
the corridor dialled from a socket held with `SO_BINDTOIFINDEX`, through a connector
and transport of its own; `DownTheCorridor::on_the_network`) and `crates/alo-turn`
(`Bounding::carrying_out_a_departure_on`, refusing by default and never falling back
to an unheld registration). `socket2` joins the workspace for the one socket option
nothing else could set without `unsafe`. Tested on a real kernel by
`crates/alo-bounding/tests/a_private_ipv4_departure_is_held_to_its_network.rs` (two
`veth` cables in a namespace of their own, a machine at `10.64.0.20` at the far end of
each counting every byte: reached on the network shown and refused with `EACCES` on
the other by a held connection, a datagram, a joined socket, the route and
`IP_PKTINFO`; a provider's departure reached by all of them) and
`crates/alo-agentd/src/a_paired_machine_on_two_networks_with_one_address.rs` (the
studio found on the cable alone, asked from a turn bounded by the real programme,
shown and recorded by its name, while somebody else at the same address on the
network the route prefers counts nothing; a mutation removing the corridor's hold
sends the question to them). `docs/contracts/local-network-wire.md` gains one
additive line; `docs/quirks.md` records the kernel's behaviour. The report is
`docs/autonomy/updates/a-private-ipv4-departure-is-held-to-its-network.md`.
**Depends on:** 22, 24.

*One GPU box serves the office — it is still egress, and the indicator still fires.*
Since task 22 a machine on two networks is found on each, and since task 24 a
link-local departure carries the interface it leaves by (ADR 0041). An IPv4
departure still does not, and ADR 0041 names why that is a limitation rather than a
decision: `192.168.1.20` on the wired network and `192.168.1.20` on the Wi-Fi are
two machines whenever two routers hand out the same private range, which is most
offices and most homes. A turn shown the studio at `192.168.1.20` on one network is
today permitted the same address on the other — through a route that changed while
the question was open, or a socket held to the other interface — and the kernel
dials either without complaint, because unlike a link-local address an IPv4 address
needs no interface to be dialled, so there is no scope to read.

- **Acceptance:** how an IPv4 departure to a paired machine is held to the network
  it was found on is decided — held to the interface by the daemon and checked by the
  kernel against the socket's `skc_bound_dev_if`, decided by the route the kernel
  would take, or deliberately not, with the reason — **in an ADR before it is code**,
  with the options and what each costs a question that is not to a paired machine
  (a provider, whose address is not on any one network); if built, a question from a
  turn to a paired machine at a private IPv4 address is reached on the network it was
  found on and refused with `EACCES` on another network carrying the same address,
  tested on a real kernel under `alo_bounding::Waited::on_this_kernel()` with two
  interfaces in a namespace of their own carrying the same IPv4 address and a
  listener that would have answered on either; a provider's departure is unchanged,
  tested; and what the indicator and the record name is unchanged in shape, tested.
- **Constraint:** ADR 0003, ADR 0007, ADR 0020 and ADR 0041 as they stand: no
  departure is widened to a prefix, an interface or "the local network", loopback
  stays the only unchecked destination, and there is still no setting — no network
  chosen by a person or an agent. Editing `alo-bounding`, `alo-bounding-map` or
  `alo-bounding-kernel` is coordinated with the lane that owns them (ADR 0028). What
  reality does that the specification does not say goes in `docs/quirks.md`. Nothing
  in `alo-shell`, nothing in `image/`.

### 26. A proposal from a private IPv4 address is measured on the network it arrived on

**Status:** **Done, 2026-09-16.** The reading is decided in `crates/alo-agentd`
(`arrived_on.rs`), which is the crate that measures: **the network a connection
arrived on is the interface that owns the accepting socket's local address**,
read with `getsockname` against the interfaces the kernel reports at that moment
— an address two interfaces own, or none does, is `ArrivedOn::NothingCouldSay`
and is measured nowhere. `IP_PKTINFO` read back from the accepted socket was
rejected because reading it needs `recvmsg` in `crate::wire`'s one reader or an
`IP_PKTOPTIONS` `getsockopt` no safe crate spells, and the route was rejected as
ADR 0044 rejected it. `looking.rs` takes the reading and holds the measuring
socket to it (`found_at`), writing everything found down on that interface
(`alo_nearby::HeardFrom::on_the_network`); `hearing.rs` reads it off the
connection for a proposal and for nothing else. Tested with no kernel in it by
`arrived_on.rs` and `looking.rs`, and end to end on a real kernel by
`crates/alo-agentd/src/a_proposal_measured_on_the_network_it_arrived_on.rs`: two
`veth` cables in a user namespace, a machine at `10.66.0.2` at each far end and
**both answering discovery as the studio**, the studio proposing over the cable
while questions route to the other network — measured on the cable, judged
against the machine that sent it, somebody else asked nothing, and the same
measurement held to nothing reaching somebody else instead. `docs/quirks.md`
records what the kernel does. The report is
`docs/autonomy/updates/a-proposal-measured-on-the-network-it-arrived-on.md`.
**Depends on:** 7, 22, 25.

*Machines find each other with zero configuration, and trust none of them for it.*
Task 7 refuses a proposal, a confirmation or a verb from an address discovery has
not just measured, and the measurement is made at the moment: the machine at the
connection's source address is asked who it is (`crate::looking::found_at`). Since
task 25 a machine this one **dials** at a private IPv4 address is held to the network
it was found on — but a connection that **arrives** from `192.168.1.20` is still
measured from a socket held to nothing, so the question *who are you* leaves by
whatever the route says. On a machine on two networks that hand out the same range,
the answer can come from the other network's `192.168.1.20`: the proposal from the
studio on the cable is measured against somebody else on the Wi-Fi, and refused or
— worse — judged against a machine that did not send it.

- **Acceptance:** how the network a connection arrived on is read — the interface of
  the accepting socket's local address, an `IP_PKTINFO` read back from the accepted
  socket, or another reading the kernel really gives, with what each costs where this
  machine's own address is the same on both networks — is decided in the crate that
  measures, and written up with the reason; a proposal arriving over IPv4 is measured
  from a socket held to the interface it arrived on, and what is found is written down
  on that interface (`alo_nearby::HeardFrom::on_the_network`), tested; a connection
  whose arriving interface cannot be read is measured nowhere and refused as not
  found, never measured by the route, tested; and two machines at the same private
  address on two networks, one of them proposing, are told apart end to end — the
  proposal measured against the machine that sent it and the other machine asked
  nothing — tested with network namespaces joined by `veth` pairs.
- **Constraint:** ADR 0003 and ADR 0044 as they stand: discovery reveals presence
  only, nothing is kept between measurements, and there is still no setting — no
  network chosen by a person or an agent. What crosses the wire is unchanged. What
  reality does that the specification does not say goes in `docs/quirks.md`. Nothing
  in `alo-shell`, nothing in `image/`.

### 27. A machine on two networks with one private range is reachable on both

**Status:** **Done, 2026-09-16.** Built in `crates/alo-agentd` (`listeners.rs` —
`Listeners`, one IPv4 listener per IPv4 network held to that network's interface
with `SO_BINDTOIFINDEX` set before the bind, beside one IPv6-only listener held to
nothing; `Listening`, a handle onto each for one round of the service, so nothing
is locked while a round waits; the listeners follow the kernel's network events as
`joining` does, a network that will not bind is a line in the service log, and a
machine that cannot read its own interfaces binds one listener held to nothing and
says so; `networks.rs` — `listening_networks`, the same rule as discovery's with
**loopback included** and multicast not asked for, so a connection to `127.0.0.1`
is answered as it is today; `arrived_on.rs` — `what_a_listener_held_to`, the
network a connection arrived on taken from the listener that accepted it rather
than from the address it was dialled at, with the old reading kept for a listener
held to nothing; `unix.rs` — `an_ipv6_only_listener_on` and `ready_and`, which
waits on the fixed things and on a list whose length is not known until it is
asked; `wire.rs` — `Knocked::arrived`, `Wire::listening`, `Wire::listened_on`,
and `networks_changed` moving the listeners too; `serving.rs` — one round waiting
on every listener) and `crates/alo-nearby` (`dialling.rs` — **a proposal and a
confirmation dialled from a socket held to the network the other machine was
heard on**, without which two machines that found each other on a network the
route does not point at could propose and never pair; `presence.rs` —
`Found::on_the_network`; `waiting.rs` — `Waiting::where_the_other_was_heard`).
Measured on this kernel and recorded in `docs/quirks.md`: two held listeners on
`0.0.0.0` coexist, an unheld one beside them is refused `EADDRINUSE`, a held
listener answers its handshakes out of its own interface, and `SO_BINDTOIFINDEX`
needs no capability. Tested on a real kernel by
`crates/alo-agentd/src/a_machine_reachable_on_both_networks.rs` (two `veth`
cables carrying `10.67.0.0/24`, a machine at `10.67.0.2` at each far end, the
route pointing at the network the studio is **not** on: an unheld listener is
never reached and a held one is, then the studio proposes over that network and
the two machines pair, while somebody else at the same address is asked nothing
and connected to never). Contract: `docs/contracts/local-network-wire.md` (*The
port is answered on every network*, new, additive). **What is not held yet is a
discovery answer**, which is task 28. The report is
`docs/autonomy/updates/a-machine-reachable-on-both-networks.md`.
**Depends on:** 25, 26.

*Machines find each other with zero configuration, and trust none of them for it.*
Task 26 measured what the kernel really does with the port this machine advertises,
and found something task 26 did not fix: `crate::wire` binds **one** listener held
to no interface, and an unheld TCP listener answers every handshake by the route
(`docs/quirks.md`, measured 2026-09-16 — for an unheld listener `ireq->ir_iif` is
zero, so `inet_csk_route_req` looks the SYN-ACK's route up unconstrained). So on a
machine on two networks that hand out the same private range — the commonest office
and the commonest home — **only the machine on the network the route points at can
reach this one's port at all**: the studio on the cable proposes and its handshake
never completes, because the reply left by the Wi-Fi. The same measurement showed
the second half: `getsockname` on an accepted connection is the address that was
*dialled*, not one belonging to the interface the packet arrived on (Linux's weak
host model), so once a connection from off the route can complete, task 26's
reading would attribute it to the wrong network. The two are one task: hold the
listeners, and take the interface from the listener that accepted.

- **Acceptance:** the wire listens on **one IPv4 listener per IPv4 network this
  machine is on, each held to that network's interface** (`SO_BINDTOIFINDEX`, as
  `crate::looking::held_to` already holds a datagram socket), beside one IPv6-only
  listener held to nothing — measured: two held listeners on `0.0.0.0` coexist and
  an unheld one beside them is refused `EADDRINUSE`, so this replaces the single
  dual-stack listener rather than joining it, and loopback is one of the networks
  listened on so a connection to `127.0.0.1` is answered as it is today; the
  listeners follow the kernel's network events as discovery's joins do
  (`crate::joining`, `Wire::networks_changed`), a network that will not bind is a
  line in the service log and the others are still bound, tested; `Wire::accept_one`
  answers with the interface of the listener that accepted, and
  `crate::hearing` measures a proposal on **that** rather than on the address it
  was dialled at (`crate::arrived_on` gains the reading and keeps the old one only
  where there is no held listener), tested; and end to end on a real kernel with two
  `veth` cables carrying one private range, **the machine on the network the route
  does not point at proposes and is paired**, while a machine at the same address on
  the other network is asked nothing — which is the thing that cannot happen today.
- **Constraint:** ADR 0003, ADR 0020 and ADR 0044 as they stand: no network chosen
  by a person or an agent, no trusted-network setting, no widening of a departure,
  and what crosses the wire is unchanged. The port stays the constant it is
  (task 1): which interfaces are listened on is what the machine is plugged into,
  never a list anybody writes down. What reality does that the specification does
  not say goes in `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 28. A discovery answer leaves on the network the question arrived on

**Status:** **Done, 2026-09-16.** Built in `crates/alo-agentd` (`responding.rs`,
new — `Responders`, one datagram socket per network the kernel reports, each held
to that network's interface with `SO_BINDTOIFINDEX` set before the bind and joined
to the group on the networks that carry multicast, with **no unheld socket beside
them**; loopback is answered on and never joined, as `crate::listeners` has it;
`Responding`, a handle onto each for one round, so nothing is locked while a round
waits; the responders follow the kernel's network events as the listeners and the
joins do, and a network that will not take a socket is a line in the service log;
**a network the kernel numbers zero, and a machine that cannot read its own
interfaces at all, are answered on no network rather than by the route**, which is
where this deliberately differs from `crate::listeners` and the module says why;
`wire.rs` — `Wire::responding`, `Wire::answered_on`, one presence and one workspace
held by `Responders` so a network answered on later says what the ones at start
say, and `networks_changed` moving the responders too; `joining.rs` — the IPv4
joins moved out to the socket that is held on each network, leaving the IPv6 group
and the kernel's notification; `answering_discovery.rs` — one round waiting on
every responder). **The reading was chosen rather than `IP_PKTINFO`** because no
crate in this workspace parses a `cmsghdr` safely and `CLAUDE.md` forbids `unsafe`;
the cost is written up in `responding.rs` and in the report. Measured on this
kernel and recorded in `docs/quirks.md`: held datagram sockets on one port coexist,
an unheld one beside them **also** binds (where TCP refuses it), a question is
delivered only to the socket held to the interface it arrived on, that socket's
answer leaves by that interface against the route, and an unheld socket in the same
place answers by the route and is heard by nobody. Tested on a real kernel by
`crates/alo-agentd/src/a_discovery_answer_leaves_on_the_network_it_arrived_on.rs`
(two `veth` cables carrying `10.68.0.0/24`, a machine at `10.68.0.2` at each far
end, the route pointing at the network the studio is **not** on, and **no policy
rule at all**: the studio asks who is here and hears the answer while the machine
at the same address on the other network hears nothing, then the studio proposes
by identity — which needs reception found first — and the two machines pair, while
somebody else is asked nothing, connected to never and sent nothing). A mutation
run with the hold removed fails it. Contract:
`docs/contracts/local-network-wire.md` (*A discovery answer leaves on the network
the question arrived on*, new, additive). The report is
`docs/autonomy/updates/a-discovery-answer-held-to-its-network.md`.
**Depends on:** 22, 26, 27.

*Machines find each other with zero configuration.* Task 27 made a machine on two
networks with one private range **reachable** on both — the port is one held
listener per network, and the handshake goes back out the interface it came in on.
What still leaves by the route is the other half of being found: **the answer to
*who is here*.** `crate::wire` answers discovery on one socket per family, held to
no network, and `alo_nearby::Answering::answer_one` replies with `send_to` to the
asking machine's unicast address. On a machine whose two routers hand out
`192.168.1.0/24`, the answer to the studio at `192.168.1.20` on the cable goes out
whichever interface the route picks — to somebody else, or to nobody — so the
studio never finds this machine and cannot propose to it at all. Measured on
2026-09-16 and written down in `docs/quirks.md`; it is why task 27's end-to-end
test has to keep discovery's answers on the cable with a policy rule rather than
with code.

- **Acceptance:** how a discovery answer is held to the network its question
  arrived on is decided in the crate that answers, and written up with the reason —
  one datagram socket **per network held to its interface**, each joined to the
  group on that network (as `crate::looking` already holds a socket for asking and
  `crate::listeners` for listening), an `IP_PKTINFO` read back with `recvmsg` and
  answered with `sendmsg`, or another reading the kernel really gives, with what
  each costs a question that arrived on loopback or over IPv6; an answer to a
  question that arrived on one network leaves on that network, tested on a real
  kernel with two `veth` cables carrying one private range and a machine at the
  same address at each far end — the machine that asked hears the answer and the
  other hears nothing; a question whose network cannot be read is answered on no
  network rather than by the route, tested; what is said is unchanged — the same
  identity, the same port and the same workspace answer, byte for byte, on every
  network and in both families, tested; a network that will not take a socket is a
  line in the service log and the others still answer, tested; and **two machines
  on a network the route does not point at find each other and pair with no policy
  rule in the fixture**, which is the end-to-end task 27 could not yet write.
- **Constraint:** ADR 0003 as it stands: discovery reveals presence only, there is
  still no setting, and no interface is chosen by a person or an agent. What
  crosses the wire is unchanged, and `docs/contracts/local-network-wire.md` gains
  the sentence additively if anything about it is observable at all. What reality
  does that the specification does not say goes in `docs/quirks.md`. Nothing in
  `alo-shell`, nothing in `image/`.

### 29. A cable pulled is a network this machine is no longer found on, and one plugged in is found at once

**Status:** **Done, 2026-09-16.** Measured on a real kernel by
`crates/alo-agentd/src/a_cable_pulled_and_plugged_in_again.rs` — reception serving
as `src/main.rs` does over two `veth` cables, the studio at the far end of one and
a colleague at the far end of the other: both find and reach it; the colleague's
cable pulled by its namespace ending leaves reception found and reached on the
studio's cable alone, with the door answering; the same cable laid again — a new
interface with a new index — is found by the **first** question and reached by the
first connection, with no restart; the studio's cable set down at reception's end
is found and reached nowhere while the colleague still is; somebody else holding
the port on that cable when it comes up is a line in the service log, with
discovery still answered there and the door and the colleague still answered; and
once they let go and the cable is re-seated, the studio finds and reaches reception
again. Every answer either far end heard is the same bytes. **The test found a
bug, fixed in the same change:** the thread that answers discovery
(`answering_discovery.rs`) slept on the responders it took at the top of a round,
so a cable plugged in after that was answered on by a socket nobody waited on until
somebody on another network asked something. `told_of_a_move.rs` (new) — a pair of
sockets `Responders` writes one byte into whenever the set moves; `responding.rs` —
`Responders::moved`, said in `answer_on` when a responder is let go of or added and
never when nothing changed; `wire.rs` — `Wire::answering_moved`;
`answering_discovery.rs` — each round empties it, takes the responders, and waits on
it beside them. A mutation run with the wake removed fails the test at *the
colleague, again never found reception*. Contract:
`docs/contracts/local-network-wire.md` (*A cable pulled, and plugged in again*, new,
additive). `docs/quirks.md` records what the kernel does. The report is
`docs/autonomy/updates/a-cable-pulled-and-plugged-in-again.md`.
**Depends on:** 22, 27, 28.

*Machines find each other with zero configuration.* Three sets of sockets on this
machine now follow the kernel's network notifications — the discovery joins
(`crate::joining`), the port's listeners (`crate::listeners`) and discovery's
responders (`crate::responding`) — and `Wire::networks_changed` moves all three
when the service's round says the routing socket spoke. **What no test on a real
kernel has measured is a network that *goes*.** `tests/a_machine_on_two_networks.rs`
measures a network appearing, for the joins alone and before the listeners and the
responders existed; every other rule is tested by handing a list to
`listen_on`/`answer_on` rather than by pulling a cable. So a machine undocked at
lunchtime is, as far as this repository can show, still saying it is on a network
it is not on — and the socket held to an interface that has gone is the thing that
cannot be bound again when the cable comes back, which is exactly the bug a person
reports as *it only works if I reboot after docking*.

- **Acceptance:** on a real kernel, one machine on two `veth` cables serving as
  `src/main.rs` does, with a machine at the far end of each: with both cables up it
  is found and reachable on both, tested; **a cable pulled** — the far end's
  namespace ended, or the link set down — leaves the machine found and reachable on
  the other cable and on nothing at the first, with the service still running and
  the person's door still answering, tested; **the same cable plugged in again** is
  listened on, joined and answered on again without the service restarting, and the
  machine at its far end finds and reaches this one, tested — which is the half a
  socket held to a gone interface would fail; what is said is unchanged across all
  of it, byte for byte; and a failure on the way is a line in the service log and
  never a stopped service, tested.
- **Constraint:** ADR 0003 as it stands: no network chosen by a person or an agent,
  no trusted-network setting, and what crosses the wire is unchanged. No interval
  and no polling — the kernel's notification is the only thing that wakes any of
  it. Nothing in `alo-shell`, nothing in `image/`. What reality does that the
  specification does not say goes in `docs/quirks.md`.

### 30. Two machines with no IPv4 address between them find each other again when the cable comes back

**Status:** **Done, 2026-09-16.** Measured on a real kernel by
`crates/alo-agentd/src/two_machines_with_no_ipv4_find_each_other_again.rs` — two
daemons, each serving as `src/main.rs` does, on one `veth` carrying link-local IPv6
only: each finds the other with its interface and they pair; the studio's link set
down leaves neither found by the other, a proposal refused before anything is
sent, and both doors answering; set up again, each finds the other on the same
interface with no restart; the cable deleted leaves neither found; re-laid, each
finds the other at the new address **with the new interface**, a connection to the
old index is refused by the kernel and a measurement there finds nothing; a
proposal to the paired machine then pairs, measured on the new interface; and the
cable deleted again and re-laid **at the numbers it first had** is found again.
What the studio answers is the same bytes throughout. **The test found a bug, fixed
in the same change:** a link deleted leaves the socket's IPv6 membership behind at
its number, a join there answers `EADDRINUSE`, and `crate::joining` read that as
*already joined* — so a cable re-laid at that number was never found.
`joining.rs` now leaves `ff02::fb` on a network that goes and takes a refused join
afresh; with both removed the fixture fails at its last step, with either alone it
passes. **The far end's namespace is not ended**, because the studio's namespace is
the studio's service, which the criterion keeps running: the cable is deleted
instead, which is what a namespace ending does to a `veth` (task 29). Contract:
`docs/contracts/local-network-wire.md` (*A cable with no IPv4 address, pulled and
plugged in again*, new, additive). `docs/quirks.md` records the kernel's behaviour.
The report is `docs/autonomy/updates/a-link-local-cable-pulled-and-plugged-in-again.md`.
**Depends on:** 23, 29.

*Machines find each other with zero configuration — no addresses typed.* Task 29
measured a cable pulled and plugged in again over **IPv4**. Over IPv6 link-local
the same cable is a different thing, and nothing has measured it: a link set down
loses its link-local address, and one set up again gets it back only after
duplicate address detection, a second or two later and in a second notification;
a cable re-laid is a new interface index, and a link-local address is only an
address together with that index (ADR 0041). The IPv6 discovery socket is held to
nothing and joined per network by `crate::joining`, which forgets a network that
is not reported and joins it again, treating `EADDRINUSE` as already joined — so
the rule is there, and whether the kernel keeps, drops or restores a membership
across each kind of pull is exactly what nobody has measured. A pairing found over
link-local (`HeardFrom` with the old index) that is dialled after the cable is
re-laid is the other half: it must be measured again at the moment, never dialled
at an index that no longer exists.

- **Acceptance:** on a real kernel, two machines on one `veth` with link-local IPv6
  only, each serving as `src/main.rs` does (`crate::two_machines_with_no_ipv4` is
  the fixture to start from): both find each other; the cable pulled — the far
  end's link set down, and separately the far end's namespace ended and the cable
  re-laid with a new index — leaves each machine not found by the other and both
  services running, tested; plugged in again, each finds the other at the new
  link-local address **with the new interface**, without either service restarting,
  tested; a proposal to the paired machine after the cable is re-laid is measured
  on the new interface and not dialled at the old index, tested; what is said is
  the same bytes throughout; and what the kernel does with an IPv6 membership
  across a link set down and a link deleted is written into `docs/quirks.md`.
- **Constraint:** ADR 0003 and ADR 0041 as they stand: no network chosen by a
  person or an agent, no trusted-network setting, what crosses the wire unchanged.
  No interval and no polling — the kernel's notification is the only thing that
  wakes any of it. Nothing in `alo-shell`, nothing in `image/`.

### 31. A cable deleted and re-laid between two readings is still joined, over IPv4 as well

**Status:** **Done, 2026-09-16.** Measured on a real kernel by
`crates/alo-agentd/src/a_cable_re_laid_between_two_readings.rs` — reception serving
as `src/main.rs` does in a process of its own, a far end on one `veth` carrying
IPv4. **The ordering is made certain, not hoped for:** the far end stops reception
with `SIGSTOP`, waits until the kernel reports every thread of it stopped, deletes
the cable and lays it again, and only then sends `SIGCONT`; *the service has
followed the kernel* is read from the kernel as reception's end in the discovery
group, which nothing in its network but the service joins. Re-laid at the same
numbers, the far end's **first** question is answered and the port is reached;
re-laid at new numbers, the same. On the far end's side a probe measures the kernel:
the re-laid interface is not in `224.0.0.251`, a join there is refused `EADDRINUSE`,
and `crate::responding::joined` takes it afresh with the interface then in the
group. Every answer is the same bytes. **How a re-laid interface is told from the
one that went:** the kernel's own `RTM_DELLINK`, read out of what the service
already reads on its routing socket (`crate::interfaces_that_went`, new;
`route_messages::links_deleted_in`, new; `unix::emptied` hands each datagram on) —
chosen over `/proc/net/igmp`, which answers for the interface and not the socket,
and over taking every membership afresh on every notification, which costs a leave
and a report per network each time any address changes. Messages lost or unreadable
are *any interface may have gone*. `crate::responding` lets go of a responder whose
interface went even where its number is reported again, **leaving its group then
and there** — a socket closing later would take one user off the re-laid
interface's new membership — and never reads `EADDRINUSE` as joined;
`crate::joining` follows the same reading over IPv6; the listeners need nothing, as
a listener held to a number is reached on whatever interface bears it (measured).
Mutation runs: without the reading the fixture fails at *reception never followed
its cable to 40 … not in the discovery group*; with `EADDRINUSE` read as joined it
fails at the probe. Contract: `docs/contracts/local-network-wire.md` (*A cable
deleted and laid again before the machine looks*, new, additive). `docs/quirks.md`
records the kernel. The report is
`docs/autonomy/updates/a-cable-re-laid-between-two-readings.md`.
**Depends on:** 28, 29, 30.

*Machines find each other with zero configuration.* Task 30 found that a link
deleted leaves a socket's multicast membership behind at the interface's number,
and that a join there answers `EADDRINUSE` whether or not any interface is in the
group — and fixed it for the IPv6 group in `crate::joining`. **The IPv4 side has
the same shape and nothing has measured it.** `crate::responding` holds one
datagram socket per network, joined to `224.0.0.251` on that network, and matches a
responder to its network by the interface's **index** alone (`answer_on` retains a
responder whose index is still reported); `crate::listeners` matches by index in the
same way. So a cable deleted and re-laid at the same number **between two readings
of the interfaces** — two routing messages the service reads in one round, which is
what a dock re-enumerating or a namespace being rebuilt looks like — keeps the old
responder, held to that number, whose membership belonged to the interface that
went. The machine answers a question it never receives, and is not found on that
network until the service restarts. The join helper beside it
(`responding.rs`, `join_multicast_v4` refused `EADDRINUSE` read as joined) is the
same assumption task 30 removed.

- **Acceptance:** on a real kernel, one machine serving as `src/main.rs` does over
  a `veth` carrying IPv4, with a far end that asks who is here: the cable deleted
  and re-laid at the same interface number **before the service reads the
  interfaces again** — decided in the test how that ordering is made certain rather
  than hoped for, and written up — is found by the far end's first question after
  the service has followed the kernel, tested, and the port is reached there,
  tested; the same with the cable deleted and re-laid at a new number, tested; how
  a responder and a listener tell a re-laid interface from the one that went (the
  interface's own identity as the kernel reports it, a membership taken afresh, or
  another reading the kernel really gives) is decided in the crate and written up
  with the reason; a join refused `EADDRINUSE` on a socket that did not join there
  is never counted as joined without the interface being in the group, tested; and
  what is said is the same bytes throughout, byte for byte.
- **Constraint:** ADR 0003 and ADR 0044 as they stand: no network chosen by a
  person or an agent, no trusted-network setting, what crosses the wire unchanged.
  No interval and no polling — the kernel's notification is the only thing that
  wakes any of it. What reality does that the specification does not say goes in
  `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 32. A link-local cable re-laid with the same hardware address between two readings is still joined

**Status:** **Done, 2026-09-17.** Measured on a real kernel by
`crates/alo-agentd/src/a_link_local_cable_re_laid_with_its_hardware_address.rs` —
the studio and reception each serving as `src/main.rs` does, on one `veth` with
link-local IPv6 only, laid every time at numbers 40 and 41 **and with hardware
addresses given**. The studio holds reception still with `SIGSTOP`, deletes the
cable, lays it again identically, and waits until both link-local addresses have
finished duplicate address detection before `SIGCONT`: the number, name, hardware
address and `fe80::` address at each end are asserted equal to what they were, so
the first dump reception reads is of the network it had. While held, with the cable
deleted the studio finds nothing and a proposal to reception is refused before
anything is sent; laid again, reception's re-laid interface is not in `ff02::fb`
although its socket joined at that number, and nothing answers. Let go, once
reception's service is joined and the kernel lists the interface in the group
(`/proc/<pid>/net/igmp6`, which only the service joins in that network), the
studio's **first** question finds reception on the same interface at the same
address, both questions are answered with the same bytes as before, and the studio
proposes and they pair on that interface. **No product code changed:** the reading
task 31 put into `crate::joining` holds it. With that reading removed
(`kept_and_gone` keeping every reported network) the fixture fails at *reception
never followed its cable to 40: joined at `40`, and the interface is not in the
discovery group*. Contract: `docs/contracts/local-network-wire.md` (*An adapter
that comes back identical is still a new network*, additive). `docs/quirks.md`
records the kernel. The report is
`docs/autonomy/updates/a-link-local-cable-re-laid-with-its-hardware-address.md`.
**Depends on:** 30, 31.

*Machines find each other with zero configuration — no addresses typed.* Task 31
made every set of sockets that follows the kernel read **what the kernel said
went** (`crate::interfaces_that_went`), and measured it over IPv4.
`crate::joining` follows the same reading over IPv6 — a link-local network whose
interface the kernel said was deleted is left and joined afresh even where the
same network is reported again — but that half is held only by a unit test on a
list. Task 30 re-laid a link-local cable at its old numbers **with a new hardware
address**, so its link-local address changed and the network compared unequal
anyway: what nobody has measured is the cable that comes back **identical** — the
same number, the same name and the same `fe80::` address, because the adapter is
the same adapter (a USB dongle pulled and pushed back in, a `veth` laid again with
`address` given) — between two readings of the interfaces. Reading the dumps alone,
that network never went.

- **Acceptance:** on a real kernel, two machines on one `veth` with link-local IPv6
  only, each serving as `src/main.rs` does (`crate::two_machines_with_no_ipv4_find_each_other_again`
  is the fixture to start from, and `crate::a_cable_re_laid_between_two_readings`
  shows how a machine is held still with `SIGSTOP` so the ordering is certain):
  the cable deleted and laid again at the same numbers **and the same hardware
  addresses** while the machine that is asked is held still is found by the other's
  first question once it has followed the kernel, tested; the pairing between them
  then proposes and pairs on that interface, tested; with `crate::joining`'s reading
  of what went removed the test fails, and the report says where; and what is said
  is the same bytes throughout.
- **Constraint:** ADR 0003 and ADR 0041 as they stand: no network chosen by a
  person or an agent, no trusted-network setting, what crosses the wire unchanged.
  No interval and no polling. What reality does that the specification does not say
  goes in `docs/quirks.md`. Nothing in `alo-shell`, nothing in `image/`.

### 33. A machine that missed what the kernel said about its networks is still found on every one

**Status:** **Done, 2026-09-17.** Measured on a real kernel by
`crates/alo-agentd/src/a_machine_that_missed_what_the_kernel_said.rs`. Reception
serves as `src/main.rs` does, with a far end on two `veth` cables laid every time at
the same numbers and hardware addresses: one carrying IPv4 (40/41) and one carrying
link-local IPv6 only (44/45). **The overflow is made certain rather than hoped
for.** Reception is held still with `SIGSTOP`. Batches of 64 `veth` pairs are then
made and deleted in its network until the kernel's own `Drops` count rises for
reception's routing socket. That socket is found in `/proc/<pid>/net/netlink` by
its inode among reception's descriptors and its groups `00000111`; the count was 0
before and 164 after one batch. Both cables are then deleted and laid again
identically, and the count must rise again (180): what the kernel said about the
cables was dropped too. Neither re-laid interface is in its group. Once reception
has followed the kernel (read from `/proc/<pid>/net/igmp` and `igmp6`), the far
end's **first** question on each cable is answered with the same bytes as before,
the port is reached on each, and the door answers. The service log says `DROPPED`
exactly once, and the service stops only when told to. **Product change:**
`Went` now tells messages the kernel dropped (`lost`, `was_dropped`) from a message
it could not read. `crate::joining` says `DROPPED` once for a round in which
messages were dropped; before this change, a dropped burst left no trace in the
log. Mutation run: with `Went::lost` read as nothing having gone, the fixture fails
at *reception never followed its cables … 40 in the IPv4 group: false; 44 in the
IPv6 group: false*. Contract: `docs/contracts/local-network-wire.md` (*A cable
deleted and laid again before the machine looks*, additive). `docs/quirks.md`
records the kernel. The report is
`docs/autonomy/updates/a-machine-that-missed-what-the-kernel-said.md`.
**Depends on:** 31, 32.

*Machines find each other with zero configuration.* Tasks 31 and 32 measured the
service reading the kernel's `RTM_DELLINK` after being held still across a cable
re-laid. What neither measured is the kernel **not delivering** it: a routing
socket nobody reads in time overflows, the kernel drops what it would have queued
and says `ENOBUFS` once, and `crate::unix::emptied` hands that on as a message
lost, which `crate::interfaces_that_went::Went` reads as *any interface may have
gone* — every membership taken afresh, every responder let go of and made again.
That half is held only by a unit test on a `Went` built by hand. A dock with a
dozen adapters, or a container host rebuilding its bridges while the service is
descheduled, is exactly the burst that overflows the socket, and a machine that
then counts itself joined everywhere is a machine nobody finds until it restarts.

- **Acceptance:** on a real kernel, one machine serving as `src/main.rs` does, with
  a far end on a `veth` carrying IPv4 and one carrying link-local IPv6 only
  (`crate::a_cable_re_laid_between_two_readings` and
  `crate::a_link_local_cable_re_laid_with_its_hardware_address` show how a machine
  is held still): while it is held still, enough interfaces are made and deleted in
  its network to overflow its routing socket — made certain rather than hoped for,
  by reading the kernel's own drop count for that socket (`/proc/<pid>/net/netlink`)
  before it is let go, and written up — and both cables are deleted and laid again
  at the same numbers with the same hardware addresses; once it has followed the
  kernel, the far end's first question on each cable is answered and the port is
  reached there, tested; the service log says what was lost once and the service
  keeps running, tested; with `Went::lost` read as *nothing went* the test fails,
  and the report says where; and what is said is the same bytes throughout.
- **Constraint:** ADR 0003, ADR 0041 and ADR 0044 as they stand: no network chosen
  by a person or an agent, no trusted-network setting, what crosses the wire
  unchanged. No interval and no polling, and no larger receive buffer offered as
  the fix — a buffer only moves the burst that overflows it. What reality does that
  the specification does not say goes in `docs/quirks.md`. Nothing in `alo-shell`,
  nothing in `image/`.

### 34. A dock with a dozen adapters is found on every one of them at once

**Status:** **Done, 2026-09-17.** Measured on a real kernel by
`crates/alo-agentd/src/a_dock_with_a_dozen_adapters.rs`. Reception serves as
`src/main.rs` does; the far end has twelve `veth` cables to it (`cable0`–`cable11`
at 40–51, hardware addresses given), six carrying IPv4 and six carrying link-local
IPv6 only, **laid in one `ip -batch` and brought up in a second while reception is
held still** with `SIGSTOP`, with every link-local address through duplicate
address detection before it is let go. **The burst did not overflow the routing
socket:** the kernel's `Drops` count for it stayed 0 on this machine. The fixture
reads it and holds the service log to it either way (`DROPPED` said exactly when
the kernel counted a drop). Once the kernel lists every cable in its groups
(`/proc/<pid>/net/igmp`, `igmp6`) and the service holds each of them, the far end's
**first** question on every cable is answered, all twelve with the same bytes, and
the port is reached on every cable. **Six cables unplugged in one batch** (three of
each kind, the last enumerated among them) leave reception found and reached on the
other six, holding nothing at the six that went, its door answering. **A network
made to fail:** a thirteenth cable, `cable12`, is laid while reception is held still,
and a squatter in reception's network listens on the port held to that interface
alone. The service log says *the port presence advertises could not be bound on
cable12 (10.74.12.1): Address already in use*. Reception is still found there with
the same bytes, reached on every other IPv4 cable, and its door answers. Every
per-network failure line names `cable12` and no other network. **No product code
changed** and no limit is imposed. Mutation runs: with the listener's failure line
silenced, the fixture fails at *the service log never named the network its port
could not be bound on*; with the last network enumerated skipped by the responders,
it fails at *reception never followed its cables … the kernel does not list
["cable5"] in their groups*. Contract: `docs/contracts/local-network-wire.md`
(*Many networks at once*, new, additive). `docs/quirks.md` records the kernel. The
report is `docs/autonomy/updates/a-dock-with-a-dozen-adapters.md`.
**Depends on:** 29, 33.

*Machines find each other with zero configuration.* Every fixture so far has put a
machine on one or two cables. Task 33 found that a single batch of changes in one
network overflows the service's routing socket, so a real dock is a burst. A
docking station or a lab switch with many ports brings up a dozen interfaces in one
moment, and the service then holds one responder, one listener and one IPv6 join
per network. Nobody has measured that a machine brought onto many networks in one
burst is found and reached on **each** of them. That includes the last one
enumerated, and a burst that overflows the routing socket while the adapters come
up. A socket, a descriptor or a join that runs out part of the way through would
show up as *found on eleven networks of twelve*, and nothing in the service log
would say which one was missed.

- **Acceptance:** on a real kernel, one machine serving as `src/main.rs` does, with
  a far end on twelve `veth` cables laid in one batch while the machine is held
  still (`crate::a_machine_that_missed_what_the_kernel_said` shows how), six
  carrying IPv4 and six carrying link-local IPv6 only. Once it has followed the
  kernel (read per interface from `/proc/<pid>/net/igmp` and `igmp6`), the far
  end's first question on every cable is answered and the port is reached on every
  cable that carries IPv4, tested. Every answer is the same bytes, tested. Half the
  cables deleted in one batch leaves the machine found on the other half and on
  nothing at the deleted ones, with the service still running, tested. Whether the
  burst overflowed the routing socket is read from the kernel's drop count and
  written up either way. A failure on one network is a line in the service log
  naming that network, never a stopped service and never a silent miss, tested.
- **Constraint:** ADR 0003, ADR 0041 and ADR 0044 as they stand: no network chosen
  by a person or an agent, no trusted-network setting, what crosses the wire
  unchanged. No interval and no polling. No limit on how many networks a machine is
  found on, unless a limit the kernel really imposes is measured, written into
  `docs/quirks.md` and said in the service log where it bites. Nothing in
  `alo-shell`, nothing in `image/`.

### 35. A port another program let go of on one network is listened on there again

**Status:** ready. **Depends on:** 29, 34.

*Machines find each other with zero configuration.* Task 34 made one network fail
on a real kernel: another program held the port presence advertises on one
interface, and the service's listener there was refused `EADDRINUSE`. The service
log says *a machine on that network cannot reach this one until it can be*. But
nothing tries again when the port **can** be bound. `crate::listeners` tries only
when the kernel says a network changed, and a program closing a socket is not a
network change. A machine whose port was taken for a moment at start-up by an
installer, or by a service restarting, is found on that network and cannot be
reached there until some cable somewhere changes. The service log's promise is not
kept.

- **Acceptance:** on a real kernel, one machine serving as `src/main.rs` does, with
  a far end on a `veth` carrying IPv4 and a second one carrying IPv4 as well
  (`crate::a_dock_with_a_dozen_adapters` shows how to squat the port on one
  interface while the machine is held still). With the port held by another program
  on one cable, the port is not reached there and is reached on the other, tested.
  Once that program lets go, and **with no network changing**, the port is reached
  on that cable, tested, and the service log says so once, tested. How the service
  learns the port is free is decided and written up with the reason, using a reading
  the kernel really gives. If no such reading exists without an interval, the
  decision itself is the work: an ADR with the options, and the log line changed
  meanwhile to say what is true. What is said is the same bytes throughout, tested.
- **Constraint:** ADR 0003, ADR 0041 and ADR 0044 as they stand: no network chosen
  by a person or an agent, no trusted-network setting, what crosses the wire
  unchanged. No interval and no polling unless an accepted ADR allows it. What
  reality does that the specification does not say goes in `docs/quirks.md`.
  Nothing in `alo-shell`, nothing in `image/`.
