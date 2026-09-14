# The daemon binds the port

- Date: 2026-09-14
- Workstream: v0.5 the local network, task 10 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Tasks 7, 8 and 9 built the pairing wire, the verb wire and the asking side's
door on a turn, and every one of them took a listener a test had bound. No
process on a real machine answered on the port presence advertises, held the
pairings, or wrote a pairing down. This is that process: `alo-agentd`, whose
serving loop now owns the port beside the person's door.

## Which lane took `alo-agentd`, and why

The plan says to confirm with the loop's owner which lane takes the daemon.
The loop's owner assigned this task to this worker by name, and nobody was
waiting to answer a question; that is the confirmation, and this report is
where it is written. The crate was gated where the plan says it must be —
under WSL, with the test count read: **257 unit tests** (was 240), and
5 + 1 + 1 + 4 + 12 integration tests, all green.

## What changed, in a sentence a person can read

A machine running alo OS now answers on the network: it says it exists at a
port, and on that port it takes a proposal to pair, a confirmation, a verb an
agent on a paired machine asks for, a question about what became of a change,
and a question for its models — each judged by the crate that decides it,
with the proof checked before anything else, and anything else refused as not
for this wire. A pairing revoked on the person's own side stops the very next
verb and the very next question. When two people finish pairing their
machines, the machine writes that down, once; a proposal that was refused
writes nothing. And an agent on this machine that tried to call itself by a
machine's name is refused before it can be served.

## What changed, by crate

**`crates/alo-agentd`** — the daemon owns the port.

- `src/wire.rs` (new): `Wire` — the `TcpListener` on `THE_WIRE_PORT` (7610),
  the datagram socket discovery is answered on, the identity kept at
  `THE_IDENTITY` (`/var/lib/alo/machine-id`), and the port an asking
  machine's own discovery answers at. `Wire::bound` for the machine,
  `Wire::on` for a test on one host; `accept_one` reads one message through
  `alo_nearby::http` with the verb wire's bound; `answer_discovery` answers
  one question with the port. Nothing here is configurable.
- `src/hearing.rs` (new): one message told apart by path and handed to the
  crate that decides it — the two pairing paths to `alo-nearby`, the three
  verb paths to `alo-corridor`, the question path to `questioned.rs`, and
  any other answered `404 not-for-this-wire`; bytes that are not a message,
  `400 not-a-message`. The lock is taken for the length of one message. A
  confirmation that completes a pairing writes `alo_record::Entry::paired`
  through the remote turn holding the machine or through the machine itself.
- `src/network.rs` (new): `TheNetwork`, the **one lock** over the `Pairings`
  and the `Proposals`, taken by the loop per message and by the person's
  surface from outside it.
- `src/questioned.rs` (new): the question door — the proof through the one
  `Doorway` (so the `Seen` is one), then the pairing's `MayAskIts::Models`
  arm, then `503 not-answered-here`, nothing written. See the decision.
- `src/looking.rs` (new): discovery measured at the moment a proposal
  arrives — the asking machine is asked, at the address the connection came
  from and the port its discovery answers at, whether it exists.
- `src/surface.rs` (new): `NobodyToShowItTo`, the surface the process is
  handed until a shell shows a proposal; it answers `false` and says so to
  the service log.
- `src/terms.rs` (new): `Terms`, what the service is told — the agent, the
  two lengths, the retention rule, the egress rule made from the
  description's one `SourcePolicy`, and the naming (`NoNameYet`).
- `src/serving.rs`: the loop. Between local turns the network's door
  (`alo_corridor::Doorway`) holds the machine and the port is polled; while
  a local turn is under way the port is left out of the wait and a verb from
  the network waits in the backlog; a local agent knocking ends an idle
  remote turn and takes the machine back with the proofs seen. Discovery is
  answered in every state. `Serving::of` takes the wire, the network and a
  `Terms`; `until_stopped` takes the surface and refuses an agent called by a
  machine's name before any turn. `Served` counts messages heard on the port
  and discovery answered.
- `src/holding.rs`: `Holding::TheNetwork`, the third state the person's door
  is handed; a knock is written through the remote turn or the machine.
- `src/described.rs`: an agent named `machine:…` is refused where the name is
  read (`NotDescribed::NamesAMachine`).
- `src/refusing.rs`: `NotBound::NoWire`; `NotServed::NoDoorway`,
  `AnAgentNamedAMachine`, `TheWire`, `TheMachineWasLost`.
- `src/unix.rs`: `a_shared_datagram_socket_on`, the one place `SO_REUSEADDR`
  is set, for the multicast DNS port every responder shares.
- `src/starting.rs`, `src/main.rs`: the identity read and the port bound
  before the person's door, and given back on both roads out.
- `src/testing.rs`: `reception`, `the_studio`, `paired_between` with a moment.
- `Cargo.toml`: `alo-nearby` and `alo-corridor`.

**`crates/alo-nearby`** — `Arrived::carried(stream, from, &message)`, the
additive constructor that takes a message somebody else read, refusing a body
over the pairing wire's bound as it would have been. `accept_one` uses it.

**`crates/alo-corridor`** — `Arrived::carried`, the same for the verb wire;
`Doorway::keeping` (a doorway remembering the proofs an earlier one saw),
`Doorway::machine` (the machine while no remote turn holds it),
`Doorway::given_back` (the machine and the `Seen`, ending any remote turn),
`Doorway::proven` (step 1 of the judgement on its own, for the question path).

**`crates/alo-turn`** — `Machine::a_pairing_was_kept`, the public door onto
the record for a pairing kept, with the same safety argument as
`the_grants_were_not_read_again`; `Arriving::a_pairing_was_kept` and
`Arriving::the_grants_were_not_read_again`, the same two doors while a remote
turn holds the machine.

**`crates/alo-record`** — `Happened::Paired { with }` and `Entry::paired`,
additive, `format` stays `1`; every accessor extended.
**`crates/alo-recounting`** — `Outcome::Paired` and the word
`recounting.outcome.paired`; the lists are twenty-one and fourteen.
**`crates/alo-asking`** — `THE_QUESTION_PATH`, one spelling for the door that
sends a question and the daemon that tells one apart, held to the URL
`openai.rs` really builds by a test.

**`docs/contracts/local-network-wire.md`** (new): the port, the framing, the
six paths, the proof header and the words at the door — the surface another
alo machine speaks. **`docs/contracts/record-file.md`**: the `paired` tag.
**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 10 done, task 11
written. **`Cargo.lock`**: the two path dependencies; nothing new resolves.

## Decisions

**One reader, and an additive constructor on each wire.** The plan offered a
dispatcher that peeks the request line or a constructor on each `Receiving`
that takes a connection already accepted. Neither, exactly: a peeked request
line leaves the rest of the request in whatever buffer did the peeking, and a
socket-level peek reads no further than the segment that has arrived, so the
bytes would either go missing between dispatcher and door or be read twice.
The daemon reads every message **once**, through the framing both wires
already share, with the larger of the two bounds; each wire gains
`Arrived::carried`, which takes the message and still refuses a body over its
own bound. Each wire keeps deciding its own method, path and shape; the daemon
knows six paths and nothing about what travels on them.

**The lock over the machine is the loop; the lock over the lists is a
`Mutex`.** The plan says the `Pairings`, the `Proposals`, the `Doorway` and
the `Seen` are one each behind one lock beside the person's door's `Holding`.
The doorway borrows the machine as a local turn does, so it lives in the
loop's own thread and the loop is the lock: a verb from the network is
judged and answered in the round it arrives, and while a local turn holds the
machine the port is not polled — the verb waits in the kernel's backlog, which
is the plan's *waits on a local turn* made literal. The reverse is decided the
other way, and the report says so plainly: a local agent knocking while a
remote turn is open **ends the remote turn** rather than waiting on it,
because a remote turn between verbs holds nothing but the window for the next
one — every verb was answered in its round — and the person's own agent
should not wait on an idle window. The next remote verb begins a fresh turn on
a fresh proof; a change waiting for approval in that turn lapses with it,
exactly as a local change lapses with a local turn. The pairings and the
proposals are changed by the person's surface from outside the loop, so they
are behind one `Mutex` taken per message; `Seen` travels with the machine
between doorways, so a replay across a local turn is still a replay
(`Doorway::keeping`, `Doorway::given_back`).

**The question path is proven and judged, and answering it is task 11.** A
proven, permitted question is answered `not-answered-here` with nothing
written. Answering it from this machine's own model needs a door on
`alo_turn::Machine` that puts a question outside any turn and writes
`Entry::answered_for` under a departure — `arriving.rs` says in its own header
that a remote turn puts no question — and a setting that says which of the
person's models may answer for another machine (ADR 0008). Both are decisions
for the crates that own them, exactly as the plan treated the `GrantError`
arm, so they are the next task rather than a side effect of binding a port.
What holds today is everything before the answer: the proof first, through
the same `Seen` the verb wire refuses replays with; the pairing's own list;
and nothing put anywhere else — never a provider, never a fallback.

**A local agent called by a machine's name is refused at two doors, and
needs no `GrantError` arm.** The plan expected the refusal to need an arm on
`alo-capability`'s closed list. It does not: the name is read by
`crate::Described`, which is the daemon's door onto its own description, and
`NotDescribed` is the daemon's own list — so the process refuses to start.
`Serving::until_stopped` refuses the same name before any turn, for a service
handed it any other way. Both are tested; `alo-capability` is untouched.

**Discovery is measured at the moment, not kept.** `Proposals::arrived`
refuses a proposal from an address discovery never measured and takes the
measurement as an argument; a daemon keeping a list would judge the afternoon
against the morning. So the daemon asks the machine that proposed — at the
address the connection came from, at the port its own discovery answers on —
whether it exists, waits two seconds, and judges against what answered.
Nothing typed reaches it: the address is the kernel's, the port the wire's.

**The record is written at one moment.** `alo_nearby::Heard::AConfirmation`
carries the pairing exactly when the confirmation that arrived was the
second; that is where `Entry::paired` is written, through whatever holds the
machine. `paired` names the other machine by identity, because no person has
named it yet, and names no agent, because two people made it. `Happened` is
a closed list read in `alo-recounting`, which gained the outcome and its word.

**The person's name for a machine has no home yet, and says so.** `Terms`
carries a `Naming`; the process is handed `NoNameYet`, so every entry names a
machine by identity until a shell keeps names. The surface that shows a
proposal is likewise `NobodyToShowItTo` until a shell implements it — a
proposal to a real machine is refused as *nobody to show it to*, which is the
true word, and the tests hand in a surface that shows.

**`SO_REUSEADDR`, in `unix.rs` and nowhere else.** The discovery port is the
multicast DNS port every responder on a machine binds; the option is what
lets them share it, and `unix.rs` is the one file that names `rustix`.

**Nothing in `image/`.** The unit already permits the network — it carries
no `IPAddressDeny` — and the port is a constant, so no line in the image
changes. Whether the discovery port can be shared with whatever responder the
pinned base runs is a question for the booted image, and is listed below.

## Acceptance, and the test behind each line

All in `alo-agentd` unless said; every one runs under WSL.

| The plan says | The test |
|---|---|
| one process binds the port presence advertises and answers discovery with that port | `wire::tests::discovery_is_answered_with_the_port_the_wire_listens_on` |
| on that port tells a proposal, a confirmation, a verb, a question and an outcome apart by path, handing each to the crate that decides it and refusing anything else as *not for this wire* — a request on each path reaching its door and a request on none reaching nothing | `serving::tests::each_path_on_the_port_reaches_its_door_and_any_other_reaches_nothing` |
| the `Pairings`, `Proposals`, `Doorway` and `Seen` are one each behind one lock beside the person's door's `Holding`, so a pairing revoked on the person's surface refuses the next verb and the next question alike — a revocation between two requests on two paths | `serving::tests::a_pairing_revoked_on_the_persons_surface_refuses_the_next_verb_and_the_next_question` |
| a pairing kept is written to `alo-record` at the one moment there is one value to write it from, and a proposal refused or withdrawn writes nothing | `serving::tests::a_pairing_kept_is_written_down_once_and_a_proposal_refused_writes_nothing` |
| a local agent that calls itself by a machine's name is refused at the daemon's door | `serving::tests::a_local_agent_called_by_a_machines_name_is_refused_at_the_door`; `described::tests::an_agent_called_by_a_machines_name_is_refused` |
| the proof is judged before anything else, through the one `Seen`, on the question path too | `questioned::tests::a_question_is_proven_before_anything_else_and_a_replay_is_refused` |
| a pairing that does not permit asking this machine's models permits no question | `questioned::tests::a_pairing_for_something_else_does_not_open_this_door` |
| discovery for a proposal is measured, and a machine that does not answer is not found | `looking::tests::a_machine_that_does_not_answer_is_not_found` |
| the question path is one spelling for the door that sends and the daemon that reads | `alo-asking` · `openai::tests::a_question_down_the_corridor_is_put_to_the_question_path` |

And beside them: a message is read once with where it came from and a
non-message is carried as one (`wire::tests`); a machine that answers is
found at the address it came from (`looking::tests`); an unpaired machine's
question is refused at the proof (`questioned::tests`); every existing test
of the serving loop runs against a service that has bound a port.

## Verified

Ubuntu under WSL, this development machine, from the checkout, each run in
the foreground and waited on, building in
`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

| Gate | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings, exit 0 |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps` on the seven crates touched | clean, exit 0 |
| `cargo test -p alo-agentd` | **257 unit** (was 240), 5 + 1 + 1 + 4 + 12 integration, all green |
| `cargo test -p alo-record` | 71 unit, 2 + 2 + 3, all green |
| `cargo test -p alo-recounting` | 55 unit, 5 + 5 + 6 + 4 + 2, all green |
| `cargo test -p alo-turn` | 84 unit, 2 + 4 + 12 + 6 + 1, 2 doctests, all green |
| `cargo test -p alo-asking` | 101 unit (one new), every integration target green |
| `cargo test -p alo-nearby` | 124 unit, 9 + 12 + 5 + 9, 1 doctest, all green |
| `cargo test -p alo-corridor` | 32 unit, 4 + 9, all green |
| `cargo test -p alo-saying`, `cargo test -p alo-collected` | all green — the new word is collected and the lists agree |

The full workspace suite was not run here, as instructed; the supervisor
runs it. Every crate that names `alo-record`, `alo-turn`, `alo-nearby` or
`alo-corridor` compiles under the workspace clippy above; no public signature
any of them already had changed, and `Happened` gained an arm matched
exhaustively only in `alo-record` and `alo-recounting`.

**What no test shows is two chassis, or a booted image.** Both machines are
one process on one host over real sockets, as in every report of this plan.
Whether `alo-agentd` can share the discovery port with whatever responder the
pinned base runs, and whether a second physical machine hears the first, are
owed to the booted image and to two machines.

## Remaining limitations, honestly

- **A question is not answered yet.** Proven, judged against the pairing's
  list, and answered `not-answered-here`; task 11.
- **The person's door does not reach a remote turn.** A change a paired
  machine proposed waits for this machine's person, and `approve`, `decline`
  and `waiting` answer *nothing is happening* while a remote turn holds the
  machine; task 11 names the requests.
- **The person's surface confirms outside the loop.** Confirming a pairing
  from this machine goes through `TheNetwork`'s lock directly (the tests do
  it); the person's-door request for it is task 11's with the rest.
- **Pairings are not kept between restarts.** `TheNetwork::on` starts
  empty and says so; where a pairing is remembered is `alo-remembering`'s
  and is not built.
- **No shell shows a proposal or keeps a name**, so a proposal to a real
  machine is refused as *nobody to show it to* and every machine is spoken
  of by identity — both true words, both `alo-shell`'s to change.
- **A slow stranger costs ten seconds.** A connection that sends nothing
  holds the loop for the wire's timeout, as both `Receiving`s already did;
  bounded, and not a way to hold the machine.
- **Proven, not private**, unchanged from ADR 0031.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *A machine running alo OS now
  answers on the local network. It says it exists at a port, and on that port
  it takes a proposal to pair, a confirmation, a verb from a paired machine's
  agent, a question about what became of a change, and a question for its
  models — each checked for proof before anything else, and anything else
  refused. A pairing revoked on the person's own side stops the very next
  request; when two people finish pairing, the machine writes it down once,
  and a proposal that was refused writes nothing. An agent on this machine
  cannot call itself by a machine's name.*
- **ROADMAP.md** — no box moves; *machines find each other* and *a remote
  agent acts only under a local grant* now have a daemon answering on the
  port, and stay unticked until two machines do it.
- **QUEUE.md** — task 10 of the v0.5 local-network plan done; task 11
  (*a question from a paired machine is answered by this machine's own
  model, and the person's door reaches a remote turn*) ready, depending on
  10.
