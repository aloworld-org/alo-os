# The asking machine crosses through a turn, and is told what became of it

- Date: 2026-09-14
- Workstream: v0.5 the local network, task 9 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration** — in the shape the plan names for a task
  whose daemon is not this lane's to edit: the asking side's door on
  `alo_turn::Turning`, the outcome path on the wire, and this report saying
  what the daemon still owes. The daemon's part is written into the plan as
  task 10, with its acceptance intact.

## Which lane owns `alo-agentd`, and what followed

The plan's constraint for this task: *before starting, confirm which lane
owns `alo-agentd` — if it is another lane's, this task is the asking side's
door on `Turning`, the outcome path on the wire and a report saying what the
daemon still owes, and nothing in that crate.*

No v0.5 plan names `alo-agentd` among the crates it owns. Its history says
who builds it: the serving loop, the session, the listener and `Holding` were
built by the v0.01 delivery lane (`docs/autonomy/v0-01-delivery-plan.md`,
`v0-01-lane-b-plan.md`), and the kernel-enforcement plan's ready tasks name
its two kernel tests as their evidence. It is a shared, Linux-only daemon
whose gate runs only under WSL (`docs/autonomy/LOOP.md`), and the thing this
task would have to change in it is not a file but its shape: `alo-agentd`
holds the one `alo_turn::Machine` under the person's door's `Holding`, and
`alo_corridor::Doorway` borrows the same machine for the network's door.
Putting both behind one lock is a redesign of that daemon's serving loop, on
a crate three lanes edit, gated on a kernel this checkout reaches only through
WSL. That is another lane's decision to take with the loop's owner, so this
task took the plan's narrower shape and says so here rather than editing a
daemon it could not gate honestly.

## What changed, in a sentence a person can read

An agent on one machine can now ask, from inside its own turn, for a verb on
a paired machine down the corridor — and the turn treats it exactly as it
treats a question that leaves the machine: it is put from inside a boundary
permitting that one machine's address and no other, it is shown leaving on
the indicator, and it is written into the record as having left whether or
not an answer came back. And when a change was proposed there, the agent can
now ask what became of it — approved, declined, still waiting, or lapsed — so
a person is not left guessing whether a file moved on a machine down the
corridor.

## What changed, by crate

**`crates/alo-turn`** — the door, and the memory behind the outcome.

- `src/crossing.rs` (new): `Turning::crossing`, the door beside
  `Turning::asking`. It takes the address discovery measured and the crossing
  as a closure, registers the address as the one destination the boundary
  permits, runs the closure inside `Bounding::carrying_out_a_departure`, and
  writes down what came back: `Entry::left` for anything that left, whether an
  answer, a door's word or nothing came back; `Entry::held_back` when the
  rule refused it; nothing when nothing was addressed; `Entry::not_bounded`
  when no boundary could be imposed, so nothing crossed. `Departed<T>` is
  what the closure hands back. Four unit tests, one per row of that table.
- `src/became.rs` (new): `Became`, what became of a change a turn put to the
  person — waiting, approved, declined, lapsed, nothing waiting.
- `src/turning.rs`: `Turning` remembers, by number, every change it put and
  what the person did about it; `Turning::became(number, now)` answers it,
  reading lapsing off the change at the moment. A number is its turn's and
  means nothing outside it. Unit test
  `what_became_of_a_change_is_answered_by_the_turn_that_put_it`.
- `src/arriving.rs`: `Arriving::became`, the same door on a remote turn.
- `src/asking.rs`: `nothing_was_bounded` answers a `NoAnswer` rather than a
  `Result`, so the two doors that leave the machine share it. Nothing it
  writes changed.
- `src/lib.rs`: the two modules, `Became` and `Departed` exported.

**`crates/alo-corridor`** — the outcome path, and the road through a turn.

- `src/outcome.rs` (new): `AskedAbout`, the question on the wire — one
  number, `deny_unknown_fields`, those bytes what the proof is over — and
  `Outcome`, `Became` spelt for the wire, with `From<Became>`.
- `src/receiving.rs`: `THE_OUTCOME_PATH` (`/alo-os/1/verb/outcome`), a third
  door beside the read and the change, told apart by path like the other
  two.
- `src/doorway.rs`: `Door::Outcome`. Judged in the same order as a verb —
  the proof first, then the turn, then the body — and answered from
  `Arriving::became` with no grant asked and nothing run, the answer leaving
  under a departure exactly as a verb's does.
- `src/answered.rs`: `Answered::Became(Outcome)`, additive on the wire.
- `src/crossing.rs`: `Crossing::asking_after`, the asking end of the outcome
  path; and the three doors through a turn — `reading_through`,
  `changing_through`, `asking_after_through` — which hand the crossing to
  `Turning::crossing` with the departure taken out of whatever carried it so
  the turn writes it down and ends it. `Crossing::crossing` takes the body
  it is given rather than building one, so the three paths share it.
- `src/refusing.rs`: `NotThrough`, why a verb did not cross through a turn:
  `NotCrossed` with the departure gone, the turn's own refusals carried whole.
- `tests/a_turn_on_the_asking_machine_crosses_a_verb_and_asks_what_became_of_it.rs`:
  the acceptance over real sockets on one host, discovery done honestly.
- `Cargo.toml`: `alo-context` as a dev-dependency, so the integration test
  can begin a real turn to cross from.

**`Cargo.lock`**: the dev-dependency. Nothing new resolves.

**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 9 marked done in
this shape, and task 10 written — *the daemon binds the port* — carrying
every line of task 9's acceptance that is the daemon's, unchanged.

## Decisions

**The door on `Turning` is generic, and the corridor fills it.** `alo-turn`
cannot name `alo-corridor`, which names it; and `alo-turn` opens no socket
and says so. So `Turning::crossing` takes the crossing as a closure and
adds what a turn adds around a question — the boundary, the indicator's
departure written down, the record — and `alo-corridor`'s three
`*_through` doors are the only things that fill it. The turn's four unit
tests hold the table of what is written with no port in sight; the
corridor's integration test holds that the road is walked over a socket.

**Written down whether or not an answer came back.** `Departed::Left`
carries what came back as the caller's own type, so a verb that met a
closed door, or no door at all, is `Entry::left` exactly as an answered one
is. The turn's first test crosses one of each and counts two.

**The outcome is a read of the turn's memory, and it walks the verb's
road.** It is proven at the door before anything else, it needs the turn
the asking machine has open here, and its answer leaves under a departure —
because what it answers is this machine's business leaving this machine. It
asks no grant and runs nothing, because it changes nothing. `Door::Outcome`
is a third arm on the doorway's one `match`, so the order is the same by
construction.

**A number is its turn's.** The number a change waits under is given by
the turn that put it. After that turn has ended — its time up, or the
daemon ending it — a question about the number is answered *nothing
waiting*, which is true and is said as such rather than guessed at; what
the person decided is in that machine's record either way. A remote turn
lasts as long as the daemon says, up to a day, which is the window an
asking agent has.

**What `alo-agentd` still owes, and why it is task 10.** The port bound,
discovery answered with it, the three wires told apart by path with
everything else refused as *not for this wire*, the `Pairings`, `Proposals`,
`Doorway` and `Seen` behind one lock beside the person's door's `Holding`
of the same machine, `alo-record` written when a pairing is kept and nothing
when a proposal is refused or withdrawn, and a local agent calling itself by
a machine's name refused at the door. That last one this lane could have
put in `Turning::beginning`, and did not: it needs a refusal `GrantError`
does not have, and adding an arm to that closed list is `alo-capability`'s
decision rather than a side effect of this task. Task 10 says so.

## Acceptance, and the test behind each line

The lines of task 9 this shape covers, all in `alo-corridor` unless said;
the integration test is
`a_turn_on_the_asking_machine_crosses_a_verb_and_asks_what_became_of_it`.

| The plan says | The test |
|---|---|
| the asking side has a door on `alo_turn::Turning` that crosses a verb the way `Turning::asking` puts a question — resolved before the boundary is entered and put from inside it, the departure written down whether or not an answer came back, the answer or the door's word handed back | `alo-turn` · `crossing::tests::what_crossed_is_written_down_as_left_whether_or_not_an_answer_came_back`; and `a_turn_crosses_a_verb_and_learns_what_became_of_it` |
| the rule holds a crossing back inside the turn, written down as such, with nothing dialled | `alo-turn` · `crossing::tests::a_rule_that_says_nothing_leaves_holds_the_crossing_back_and_writes_it_down`; and `a_rule_that_says_nothing_leaves_holds_the_crossing_back_before_it_is_dialled` |
| a boundary that cannot be imposed crosses nothing and says so | `alo-turn` · `crossing::tests::a_crossing_that_could_not_be_bounded_crosses_nothing_and_says_so` |
| an agent reaches the wire through a turn and through nothing else — an unpaired machine is refused with nothing written | `a_turn_cannot_cross_to_a_machine_this_one_is_not_paired_with`; and `alo-turn` · `crossing::tests::a_crossing_that_addressed_nothing_is_handed_back_unwritten` |
| what became of a change an asking machine proposed — approved, declined, or lapsed — can be asked for on the wire, additively | `a_turn_crosses_a_verb_and_learns_what_became_of_it` (approved, and nothing waiting); `a_change_the_studios_person_declined_comes_back_as_declined`; `alo-turn` · `turning::tests::what_became_of_a_change_is_answered_by_the_turn_that_put_it` (waiting, lapsed, approved, declined, nothing waiting) |
| the outcome is one number on the wire and nothing else, and every outcome crosses as one word | `outcome::tests::a_question_about_a_change_is_one_number_and_nothing_else`; `outcome::tests::every_outcome_crosses_as_one_word_and_comes_back_as_itself` |
| the third path is told apart from the other two | `receiving::tests::the_two_paths_are_the_two_doors` |

## Verified

Windows 11, this development machine, from the checkout, each run in the
foreground and waited on:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings, exit 0 |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps -p alo-turn -p alo-corridor` | clean, exit 0 |
| `cargo test -p alo-corridor` | 32 unit, 4 + 9 integration, all green |
| `cargo test -p alo-turn` | 84 unit (was 79), 2 + 4 + 12 + 6 + 1 integration, 2 doctests, all green |

The full workspace suite was not run here, as instructed; the supervisor
runs it. No public signature either crate already had changed; `Door` gained
an arm, and it is matched exhaustively only inside `alo-corridor`.

**What no test shows is two chassis**, as in every report of this plan.

## Remaining limitations, honestly

- **No daemon owns the port.** Task 10, written into the plan with the
  daemon's whole list.
- **A local agent calling itself by a machine's name is not yet refused at
  the daemon's door.** It needs a `GrantError` arm; task 10 names it.
- **An outcome outlives nothing.** After the remote turn that put a change
  has ended, its number answers *nothing waiting*. The record on the machine
  that was asked has the decision; reading it back across the wire is not
  built and is not claimed.
- **Proven, not private**, unchanged from ADR 0031.
- **No contract document yet** for the corridor's now three paths; owed
  when the daemon owns the port.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *An agent can now ask, from
  inside its own turn, for a verb on a paired machine down the corridor. The
  turn treats it as it treats a question that leaves the machine: put from
  inside a boundary permitting that one address, shown leaving on the
  indicator, and written into the record as having left whether or not an
  answer came back. And when a change was proposed on the other machine, the
  agent can ask what became of it — approved, declined, still waiting, or
  lapsed — so nobody is left guessing whether a file moved down the corridor.*
- **ROADMAP.md** — no box moves; *a remote agent acts only under a local
  grant* gains the asking side's door and the outcome path, built.
- **QUEUE.md** — task 9 of the v0.5 local-network plan done in the shape the
  plan names for a daemon another lane owns; task 10 (*the daemon binds the
  port*) ready, and its owner to be confirmed with the loop's owner before it
  is taken.
