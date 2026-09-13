# A verb crosses between two machines, and is proven at the door

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 8 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Task 7 built the pairing wire and nothing on it carried a verb: two machines
that never shared a process could hold a pairing, and task 4's door —
`alo_turn::Arriving` — still had nothing that reached it from a network.
Task 6's report said per-message proofs on an open turn's later doors were
the verb wire's to check, and task 4's said the same of the wire itself. This
is that wire.

## What changed, in a sentence a person can read

An agent on one machine can now ask for a verb on a paired machine down the
corridor — list a folder there, or propose renaming a file there — and the
verb reaches that machine at the address it was found at, carrying a proof
that it came from the machine it says. The machine that is asked checks the
proof before it looks at anything of its own: a stranger presenting a paired
machine's identity, a verb recorded off the wire and sent again, and a verb
from a pairing since undone are each refused with nothing written down. A
verb that proves itself is then judged by the grants the person *there* made,
a change waits for *their* approval on *their* screen, and what ran or was
refused is in *their* record with the asking machine named. Every answer
going back leaves under the answering machine's own indicator and is written
down as having left; the asking machine's indicator shows the verb leaving
and its record names where it went.

## What changed, by crate

**`crates/alo-corridor` (new)** — the verb wire, both ends. Depends on
`alo-turn` for the door, `alo-nearby` for the pairing, the proof and the
framing, `alo-egress` for the departure, `alo-protocol` for the shapes an
argument and an answer already have, and `alo-asking` for the one spelling
of the proof header.

- `src/carried.rs`: `Carried`, what a verb is on the wire — `verb` and
  `given`, the arguments as `alo_protocol::Argument` so that a name and a
  typed value are spelt on this wire exactly as on the local one; compact
  JSON, `deny_unknown_fields`, and those bytes are what the proof is over.
  `AT_MOST_A_VERB`, sixteen kibibytes.
- `src/answered.rs`: `Answered`, what comes back — `did` with an
  `alo_protocol::Done`, or `waits` with the number a change waits under.
  `AT_MOST_AN_ANSWER`, the local protocol's bound on an answer.
- `src/door.rs`: `AtTheDoor`, what the receiving machine says instead of an
  answer: one word from a closed list of fourteen, with a status, whether it
  was said before the door, and the sentence the asking person reads. The
  five proof refusals reuse `alo_nearby::NotProven`'s sentences; the rest are
  this crate's. `AtTheDoor::of` maps `alo_turn::NotDone` to a word, and to
  nothing for the two arms that send nothing back.
- `src/refusing.rs`: `NotCrossed`, the asking side's four refusals — three
  where nothing left, and `Left`, which holds the departure and what came
  back (`WentBack`: the door's word, the network, or an unreadable reply).
- `src/crossing.rs`: `Crossing`, the asking end — made only from a pairing
  this machine holds and a `Found`, dialling `Found::where_it_answers`;
  `reading` and `changing`, each consuming it, each making the proof over
  the exact bytes and the `Departing` on the indicator before the dial; and
  `Crossed`, the answer with its departure, ended on the indicator by the
  caller once the record has it.
- `src/dialling.rs`: the one file that opens a connection; `pub(crate)`,
  called from `crossing.rs` after the indicator handed over a departure, and
  spelling no address.
- `src/receiving.rs`: the receiving end. `Receiving::accept_one` reads;
  `Arrived::considered` decides through the doorway and writes one reply,
  from `written`, which takes a `Replying` and nothing else. `Heard`, for
  the daemon. `THE_READ_PATH` and `THE_CHANGE_PATH`.
- `src/doorway.rs`: `Doorway`, the receiving machine's whole judgement in
  one order with no socket in sight — the proof through
  `alo_nearby::Origin::proven` first, then the turn, then the body, then the
  door, then the departure — and `Judged`. `Door`, `NotADoorway`,
  `AT_MOST_A_TURN`.
- `src/holding.rs`: `Holding`, the machine or the remote turn holding it,
  which is `alo-agentd`'s shape for the person's door.
- `src/replying.rs`: `Replying`, what goes back and what it has to hold to
  go: an answer or the door's refusal only with a `Departing`; a word before
  the door with nothing of this machine in it.
- `src/naming.rs`: `Naming`, what the person here called a paired machine,
  asked at the moment and deciding nothing.
- `src/words.rs`: ten sentences under `corridor.at-the-door.*` and
  `corridor.not-crossed.*`, each with its translator's note.
- `tests/a_verb_crosses_between_two_machines_and_is_proven_at_the_door.rs`:
  the acceptance over real sockets on one host, one test per line, with
  discovery done honestly in every test.

**`crates/alo-nearby`** — the framing is shared.

- `src/http.rs`: `pub mod http`. `Message` keeps its headers and answers
  `header(name)`; `read_message_of_at_most` takes the body bound from the
  caller, with `read_message` keeping the pairing wire's four kibibytes;
  `a_request_carrying` writes headers beside the four every request has.
  Nothing the pairing wire reads or writes changed.
- `src/lib.rs`: the module named.

**`crates/alo-turn`** — the answer leaves under the indicator.

- `src/arriving.rs`: `Arriving::departing`, which asks this machine's egress
  rule about an answer going back to the origin machine — under its
  principal, `Why::Sending`, to the machine by the name the person gave it —
  and puts it on the indicator, writing a held-back entry when the rule
  refuses; `Arriving::returned`, which writes `Entry::left` and takes the
  line off; `Arriving::ended`, which hands the machine back so the next
  remote turn can begin on the same borrow. All three additive; nothing
  `Arriving` already did changed.
- `src/turning.rs`: `Turning::ended`, the same door on a local turn, and
  `keeping_stamped`, the one place an entry is stamped with its origin, now
  reached by the two departure entries as well as by `writing_down`.

**`crates/alo-saying`**: `alo-corridor` collected, thirty-two lists.

**`Cargo.toml`, `Cargo.lock`**: the new member. Nothing new resolves.

**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 8 marked done, and
task 9 written — *the machine that is asked answers on the port it
advertises*, the daemon that turns three libraries into a machine that can
be asked, with the asking side's door on `Turning` and the outcome of a
change.

## Decisions

**A new crate, and why neither end could live where the door does.** The
receiving end calls `alo_turn::Arriving` and so depends on `alo-turn`; the
asking end reaches the wire. `alo-turn` opens no socket and says so in its
own documentation, and a socket in it would have been a second road out of
that crate beside `alo-asking`'s three doors. `alo-nearby` cannot depend on
`alo-turn`, which depends on it. So the wire is `crates/alo-corridor` — the
corridor being what this plan has called the road between two paired
machines since task 3 — with `alo-asking`'s `corridor.rs` carrying questions
down it and this crate carrying verbs.

**The asking end hands the departure back and writes nothing down**, exactly
as `alo-asking` does, and for its reason: the record is the turn's. A verb
that crossed comes back as a `Crossed` holding its `Departing`, and a verb
that crossed and was refused comes back as `NotCrossed::Left` holding the
same, so whatever holds the turn on the asking machine writes `Entry::left`
on both roads — a record of only the answered ones would report a quieter
day than the machine had. The integration test walks the whole journey into
reception's record to prove the handing back is enough. The door on
`alo_turn::Turning` that does this for a real turn, inside the boundary
(ADR 0020), is task 9's, written into the plan; task 8's acceptance is the
wire and the door, and this report says so rather than implying a turn on
the asking machine crosses a verb today.

**The receiving end's answer leaves through `Arriving`, not around it.**
Every answer to a proven verb — the listing, the number a change waits
under, or the door's *not granted here* — is this machine's data leaving
it, and the indicator and the record it leaves through are behind the
`Machine` the turn holds. So the two doors are `Arriving`'s:
`departing` asks the egress rule in force and puts the answer on the
indicator, `returned` writes the departure down stamped with the origin and
takes the line off. `receiving.rs` writes to its socket from one function,
which takes a `Replying`, and a `Replying` that carries anything of this
machine is made only with a `Departing`. The integration test reads both
files and holds it, beside the wire test that counts the `left` entries.

**A word before the door carries no departure, and the report says so
plainly.** *Not paired*, *not from the machine it names*, *already used*:
these are written back to a message that proved nothing, so the person who
asked can be told what to do, and no agent on this machine caused them, no
grant was asked, and nothing was written. A departure needs an agent whose
authority it is under, and there is none; an errand of alo OS's own would be
a new arm on a closed list. The plan's *every answer sent back travels
holding a Departing* is held for every answer — everything a turn produced —
and a word to a stranger is not an answer. `Replying::before_the_door`
refuses, at the constructor, to carry any word that is not one of those.

**A rule that says nothing leaves holds the answer back, and nothing goes
back at all.** On a machine whose organisation set `NothingLeaves`, the
verb runs — the person there granted it — and the answer is refused by the
rule, written down as held back with the origin named, and the connection
closes with nothing on it, because a refusal is also bytes leaving. The
asking machine reads that as the network refusing, which is true, and its
person is told the other machine could not be reached. A word saying *held
back there* would be the machine's rule leaving the machine.

**A refusal crosses as one word, and the sentence is said on the asking
machine.** The receiving machine words its refusals in its person's language
and the asking person reads their own, so `AtTheDoor` is a closed list of
fourteen words with a sentence each, ten of them this crate's, four the
proof's sentences `alo-nearby` already says. `alo_turn::NotDone`'s two arms
about a record that broke have no word: a machine that has stopped keeping
evidence lets nothing leave, and the connection closes.

**The proof is judged before the body is read.** The proof is over the
exact bytes that arrived, so a body altered in transit fails as *not from
the machine it names*, which is the true thing, rather than as *not a verb*.
`doorway.rs` gives the whole order and the doorway's unit test holds it with
no port in sight; the wire test holds that the road is walked.

**One remote turn at a time, and it is the borrow's decision.**
`alo_turn::Arriving` borrows the machine mutably and there is one machine,
which is `alo-agentd`'s `Holding` for the person's door and is `Holding`
here for the network's. A verb from a third machine while a turn is open is
refused as `another-turn-is-open` before the door; a turn joins the one under
way for the same machine, so the second message of a turn carries a fresh
proof judged like the first, and one that has lapsed is ended and a new one
begun. `Arriving::ended` hands the machine back so the next turn begins on
the same borrow — the one thing the door could not do before.

**How long a remote turn lasts is the daemon's number, refused at zero and
above a day.** `Doorway::at` is told it, as `alo-agentd` is told a local
turn's length from the machine's description, and for that daemon's reasons.
There is no message on the wire that ends a turn: a turn is this machine's
and ends on this machine's terms — its time up, or `Doorway::ending`.

**`Why::Sending`, and no new arm.** A verb and its arguments handed to
another machine, and an answer handed back, are each *handing something to
a service outside this machine*, which is what that arm says. `Why` is a
closed list whose widening belongs in ADR 0001 first, and nothing here
widens it.

**The framing is shared, and `alo-nearby`'s guard still holds.** One port,
one shape: the pairing wire and the verb wire read and write through
`alo_nearby::http`, now public with its headers kept and its body bound
named by the caller. `dialling.rs` and `receiving.rs` in that crate are
still the only two files there that open a connection, and its guard test
is unchanged; this crate's own guard holds the same of its dial.

**What a change comes back with is a number, and nothing more.** A change
waits for the person at the receiving machine, who answers it on their own
surface through the doorway's turn; what the asking machine is told is the
number it waits under. Whether it was approved is not asked for on the wire
yet — the plan's task 9 names it — and this report says so rather than
implying an asking agent learns what became of a rename it proposed.

## Acceptance, and the test behind each line

All in `alo-corridor`; the integration test is
`a_verb_crosses_between_two_machines_and_is_proven_at_the_door`.

| The plan says | The test |
|---|---|
| reaches the machine it names at the address discovery measured, carrying the verb, its typed arguments and a proof over exactly those bytes — and nothing else, held by a test that reads the wire and refuses any field not on that list | `a_verb_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_verb_its_arguments_and_a_proof`; and `carried::tests::a_verb_with_a_field_not_on_the_list_is_refused` |
| `Proven::checked` before anything else and `Arriving` and no other door; a stranger presenting a paired machine's identity refused before any grant is asked, the record staying empty | `a_verb_from_a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked`; and `doorway::tests::a_verb_whose_proof_does_not_hold_is_refused_before_any_grant_is_asked` |
| a verb replayed from an earlier exchange refused, the record staying empty | `a_verb_replayed_from_an_earlier_exchange_is_refused_and_the_record_gains_nothing` |
| a verb from a pairing since revoked refused, the record staying empty | `a_verb_from_a_pairing_since_revoked_is_refused_and_the_record_stays_empty` |
| a verb the receiving machine's person has not granted refused as *not granted here*, beside the same verb granted there and run | `a_verb_not_granted_here_is_refused_as_such_beside_the_same_verb_granted_here_and_run` |
| every answer sent back travels holding a `Departing`, tested by there being no road to the wire without one | `every_answer_sent_back_travels_holding_a_departing_and_there_is_no_road_to_the_wire_without_one`; and `replying::tests::a_word_before_the_door_carries_no_departure_and_takes_no_other_word` |
| the receiving machine's record names the origin machine and the asking machine's record names where the verb went | `each_machines_record_names_the_other` |
| the second message of an open turn carries a fresh proof, with the first one replayed refused | `the_second_message_of_an_open_turn_carries_a_fresh_proof_and_the_first_one_replayed_is_refused` |
| the answer leaves under this machine's indicator and is written down as left (`alo-turn`) | `alo-turn` · `arriving::tests::an_answer_going_back_is_shown_leaving_and_written_down_as_left` and `arriving::tests::a_rule_that_says_nothing_leaves_holds_the_answer_back_and_writes_it_down` |

And beside them: a change from a paired machine waits for the person at the
studio and runs once when they approve it there
(`a_change_from_a_paired_machine_waits_for_the_person_at_the_studio`); a
rule that says nothing leaves holds the answer back with nothing on the wire
(inside the departure test); a third machine's verb while a turn is open is
refused until the turn lapses, and the daemon can end one
(`doorway::tests::a_verb_from_a_third_machine_while_a_turn_is_open_is_refused_until_it_lapses`);
the wrong door is said so; a doorway refuses no time and more than a day;
every arm of `AtTheDoor` crosses as one word and comes back as itself, with
a sentence and a status that is never `200`; every `NotCrossed` has a
sentence; a header carried is read back by name and the body bound is the
caller's (`alo-nearby`); and the ten new words declare beside every other
crate's without a clash (`alo-saying`).

## Verified

Windows 11, this development machine, from the checkout, each run in the
foreground and waited on:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings, exit 0 |
| `cargo doc --no-deps -p alo-corridor -p alo-nearby -p alo-turn` | clean |
| `cargo test -p alo-corridor` | 30 unit, 9 integration, all green |
| `cargo test -p alo-nearby` | 124 unit, 9 + 12 + 5 + 9 integration, 1 doctest, all green |
| `cargo test -p alo-turn` | 79 unit, 2 + 4 + 12 + 6 + 1 integration, 2 doctests, all green |
| `cargo test -p alo-saying` | 63 unit, 4 + 1 integration, all green |
| `cargo test -p alo-collected` | all green — the new crate's words are collected and the lists agree |

The full workspace suite was not run here, as instructed; the supervisor
runs it. Every crate that names `alo-nearby` or `alo-turn` compiles under the
workspace clippy above, and no public signature either crate already had
changed.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, as in every report of this plan: the verb crosses a
TCP connection to the address discovery measured, the answer comes back
over it, and the replay is a second connection carrying the first's bytes.
Whether a second physical machine on an office network hears the first is
owed to two machines, as task 1's report said.

## Remaining limitations, honestly

- **No daemon owns the port.** `Receiving` takes a listener somebody bound;
  which daemon binds the advertised port, answers discovery, tells the three
  wires apart by path and holds the `Doorway` is task 9, written into the
  plan.
- **A turn on the asking machine does not cross a verb yet.** `Crossing`
  hands the departure back the way `alo-asking` hands one back; the door on
  `alo_turn::Turning` that puts it inside the boundary (ADR 0020) and writes
  the record is task 9's. Until then the asking side's record is written by
  whoever holds the `Crossed`, which in this repository is the test.
- **An asking machine is not told what became of a change.** It learns the
  number a change waits under and nothing more; task 9 names the additive
  path.
- **The person's name for a paired machine has no home.** `Naming` is a
  trait and a closure implements it; where a shell keeps the name is the
  shell's, and `alo-shell` is outside this plan.
- **Proven, not private.** ADR 0031's own limitation, unchanged: the wire
  authenticates and does not encrypt. A folder's listing crosses the office
  network in the clear, and the pairing key is what a later channel would be
  keyed from.
- **No contract document yet.** The two paths, the header and the two body
  shapes are spelt in one place each and tested, and a third party does not
  build against them; when the daemon owns the port, the wire becomes a
  surface another alo machine speaks and belongs under `docs/contracts/`.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *An agent on one machine can
  now ask for a verb on a paired machine — list a folder there, or propose
  renaming a file there. The verb reaches that machine at the address it was
  found at with a proof that it came from the machine it says, and the
  machine that is asked checks the proof before it looks at anything of its
  own: a stranger presenting a paired machine's identity, a verb sent again,
  and a verb from a pairing since undone are refused with nothing written
  down. What a verb may then do is what the person at that machine granted
  it there, a change waits for their approval on their screen, and every
  answer going back is shown leaving on their indicator and written in their
  record with the asking machine named.*
- **ROADMAP.md** — *a remote agent acts only under a local grant* now has a
  wire between two machines and is proven at the door; the boxes stay
  unticked until two machines do it.
- **QUEUE.md** — task 8 of the v0.5 local-network plan done; task 9 (*the
  machine that is asked answers on the port it advertises*) is ready and
  depends on 7 and 8. Small follow-ups, all additive: the outcome of a change
  on the wire, a contract document for the corridor's paths, and a home for
  the name a person gives a paired machine.
