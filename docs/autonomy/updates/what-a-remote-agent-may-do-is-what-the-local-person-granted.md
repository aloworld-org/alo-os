# What a remote agent may do is what the local person granted

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 4 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

ADR 0003's sharpest line, built and held up by tests from the side it is about:
*an agent on machine A that reaches machine B is bound by the grants made on B,
by B's person. A's grants confer nothing on B. Pairing lets A ask; it never
lets A act.*

## What changed, in a sentence a person can read

A machine on your network that you have paired with can now ask your agent to
do something, and what it may do is exactly what **you** granted **it**, on
**your** machine: a change waits for your approval, on your screen; everything
it did and everything it was refused is in your record with that machine
named; and when it asks for something you never granted, the refusal says the
grant was not made here rather than telling whoever asked to go and pick a
folder on a machine where that would grant nothing.

Nothing carries a verb between two machines yet. What is built is the one
door such a wire could call, and every decision behind it.

## What changed, by crate

**`crates/alo-nearby`** — the other end of the corridor.

- `src/origin.rs` (new): `Origin`, a paired machine as a place a verb may
  arrive from. Made only by `Origin::paired`, which asks the pairings and
  nothing else. It carries the machine's identity and the name this machine's
  person gave it, and keeps them apart: `Origin::principal` is the identity
  spelt as a grantee, `machine:` and thirty-two hexadecimal characters, which
  is what a grant on this machine is made *to*; `Origin::called` is what a
  person reads. `Origin::names_a_machine` is the one check a daemon needs to
  refuse a local agent that calls itself by a machine's name.
- `src/pairing.rs`: `Pairings::paired_with`, whether a pairing with a machine
  stands at a moment whatever it permits, and `NotPaired::NotWithThatMachine`,
  one arm for a machine that was seen, one whose pairing ran out and one whose
  pairing was undone, for the corridor's reason: saying which would say how to
  become paired.
- `src/deliberating.rs`: **`Deliberating::agreed` now takes the side the
  pairing is kept on.** See *What the compiler made me answer* below.
- `src/words.rs`: one sentence, `nearby.not-paired.not-with-the-one-that-asked`.

**`crates/alo-turn`** — the turn, from the machine that was asked.

- `src/arriving.rs` (new): `Arriving`, a turn whose verbs arrive from a paired
  machine. It is a `Turning` underneath with four things decided differently:
  whose grants are asked (this machine's, for the machine's principal — there
  is no parameter for the asking machine's grants anywhere on the type), what
  is offered (nothing: no window, no selection, no document), which doors
  there are (a read, a proposal, an approval, a decline — and not the
  question, which belongs to `alo-asking`'s corridor), and how two refusals
  are worded. The pairing is asked at every door, at the moment, the way the
  grants are.
- `src/turning.rs`: an `origin` on the turn; `pub(crate)` halves of the three
  public doors so a remote turn walks exactly the same road; every entry
  stamped with where it came from in the one place entries are written; the
  two grant refusals re-worded for the machine that was asked before the
  record and the caller see them.
- `src/carrying.rs`: the same re-wording for the two refusals a resolved path
  can earn inside the boundary. The boundary itself is untouched: ADR 0013
  applies on the receiving machine exactly as for a local turn.
- `src/words.rs`: three sentences, `turn.not-granted-here`,
  `turn.grant-here-expired` and `turn.no-longer-paired`, with the gaps the
  capability model's own refusals already have and no others; the test that
  held *no gap* now holds *no gap but those*.
- `tests/what_a_remote_agent_may_do_is_what_the_local_person_granted.rs`
  (new): the acceptance, one test per line of it, and the refusals beside
  them.
- `Cargo.toml`: depends on `alo-nearby`, which was already in its tree by way
  of `alo-asking`.

**`crates/alo-record`** — the origin, named.

- `src/entry.rs`: `Entry::from_another_machine`, a stamp rather than a second
  set of constructors, and `Entry::origin`, which answers for a verb and for a
  question a paired machine put to this machine's models alike. The field is
  absent on everything caused here, so a record written before it existed
  reads back byte for byte.
- `src/explain.rs`: `Only::FromAnotherMachine`, *what did other machines
  cause here*, one question across both doors.
- `docs/contracts/record-file.md`: the `origin` field, additive, `format`
  stays `1`.

**`crates/alo-capability`** — one additive arm.

- `src/proposal.rs`: `ProposalError::NotGrantedElsewhere(Said)`, the same door
  `NotAuthorised::NotGrantedElsewhere` already is, at the earlier point in the
  journey. `Proposal::checked` never makes one. The reasoning that decides a
  refusal is untouched, as the task requires: this task decides *whose* grant
  is asked and how the answer is worded, not what a grant is.

**`crates/alo-approving`** — unchanged code, one new test:
`tests/a_change_from_another_machine_is_put_to_the_person_here.rs`, which puts
a remote change through the existing `Approving::ask` by reading the remote
turn, and finds the turn's own sentence on the screen marked as the paired
machine's.

**`crates/alo-asking`** — two test fixtures updated for the `agreed` signature.

## Decisions

**The principal is the identity, and the name is for people.** A grant on B is
made to `machine:<identity>`, never to *the reception machine*. The name is a
label B's person chose and can give to a second machine next month; the
identity is what the pairing was made with. Authority must not depend on a
string a caller supplies beside the identity — a caller that hands over the
wrong name mislabels a record, and one that could hand over the wrong identity
would be choosing whose grants to ask. The cost is that a shell showing a grant
or an approval for a machine resolves the principal to the name through the
pairings; the record carries both, the principal as `agent` and the name as
`origin`.

**A grant to this machine's own `@files` does not reach a remote one.** B's
person granted their agent, not every agent on the network that happens to
share its name. `a_grant_to_this_machines_own_agent_does_not_reach_a_remote_one`
holds it, and it is the reason the principal has a prefix no local agent is
called by.

**The pairing gives standing, not permission.** `MayAskIts` has no arm for a
verb and the previous worker wrote that there never will be one; this task
agrees and builds on it. `Origin::paired` asks only whether a pairing with that
machine stands. What a verb may do is then entirely B's grants to that
machine, so a paired machine with no grant is refused at every door exactly as
an agent with no grant is, and B's person is never interrupted about a change
they never permitted.

**The pairing is asked at every door.** Pairings are grants across a machine
boundary, revocable in one action taking effect immediately, so a remote turn
borrows nothing at its beginning: a pairing undone between a proposal and its
approval stops the change at the moment, and a verb arriving after it is
refused before the grants are asked.

**A remote turn has no asking door.** The `Turning` inside an `Arriving` is
lent out for reading only. A machine that may ask B's models does so through
the corridor built in task 3, under the pairing arm that says so, and is
recorded there as a question answered for another machine. A question through
the verb door would have bypassed that arm.

**Two refusals are re-worded, and it is not a second rendering.** Each replaces
the capability model's sentence, travelling as a refusal *worded elsewhere*
through the door that crate already holds open, so the screen and the record
render one value. A machine with no agent, a change offered as a read, and a
path that really leads elsewhere say what is true on any machine and keep
their words.

**A stranger's verb is refused before any turn, and writes no entry.** Nothing
of B was consulted — no grant, no verb list, no person — which is the same
answer task 3 gave to an unpaired machine offering inference. Whatever carries
verbs between machines is where a knock from a stranger is counted, the way
`alo-agentd`'s `knocking.rs` counts a caller it does not know.

## What the compiler made me answer

**A machine that was asked kept a pairing with itself.** `Deliberating::agreed`
always kept the *asked* machine, which was right on the machine that asked and
wrong on the one that was asked — the first thing built on the asked side found
that nothing it was asked by was paired. It now takes the side the pairing is
kept on, and each machine keeps a row naming the other; the report for task 2
described the model correctly and the code had one row. Every caller was the
asking side and says so now. `the_same_agreement_names_the_other_machine_on_each_side`
holds it.

## Acceptance, and the test behind each line

| The plan says | The test |
|---|---|
| evaluated against the receiving machine's grants | `alo-turn` · `a_verb_from_a_paired_machine_is_evaluated_against_this_machines_grants` |
| shown to the receiving machine's person for approval | `alo-approving` · `a_change_from_a_paired_machine_is_shown_to_the_person_on_this_machine` |
| recorded there with the origin machine named | `alo-turn` · `it_is_recorded_here_with_the_origin_machine_named` |
| refused where the receiving machine's person has not granted it, even where the asking machine's person did | `alo-turn` · `a_verb_this_machines_person_has_not_granted_is_refused_even_where_the_asking_machines_person_granted_it` |
| the refusal says the grant was not made here rather than blaming the person who asked | `alo-turn` · `the_refusal_says_the_grant_was_not_made_here_rather_than_blaming_the_person_who_asked` |
| the kernel boundary applies exactly as for a local turn; a turn whose boundary cannot be applied still does not run | `alo-turn` · `a_remote_turn_that_cannot_be_bounded_still_does_not_run` |

And the refusals beside them: a grant to this machine's own agent, a machine
nobody paired with, a pairing undone during the turn, a grant taken back during
the turn, nothing of this machine's screen offered.

## Verified

Windows 11, this development machine, from the checkout:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings |
| `cargo test -p alo-turn` | 76 unit, 11 + 6 + 4 + 2 + 1 integration, all green |
| `cargo test -p alo-nearby` | 62 unit, 9 + 5 integration, all green |
| `cargo test -p alo-record` | 71 unit, 2 integration, all green |
| `cargo test -p alo-capability` | 134 unit, 8 integration, all green |
| `cargo test -p alo-asking` | 98 unit, every integration test green |
| `cargo test -p alo-approving` | 33 unit, 1 + 7 integration, all green |
| `cargo doc --no-deps` for the five library crates touched | clean |

The full workspace suite was not run here, as instructed; the supervisor runs
it. Crates that match on `Happened` (`alo-recounting`, `alo-agentd`,
`alo-keeping`) compile under the workspace clippy above; no `Happened` arm was
added, only an optional field on `Entry`.

**What no test shows is two machines.** Both sides are on one host: B is a
real `Machine` with a real folder and a real record; A is an identity in B's
pairings and, in one test, a list of grants shown to permit the very thing B
refuses and handed to nothing. There is no wire, no cryptography and no claim
of either — see below.

## Remaining limitations, honestly

- **No wire.** Nothing carries a verb from A to B or an answer back. When
  something does, the answer to a remote read is B's data leaving B, and it
  must hold an `alo_egress::Departing` from B's indicator — ADR 0003's fourth
  bullet, *B's egress indicator treats it as egress*, is the wire's to keep and
  is not built here. `arriving.rs` says so in its own documentation.
- **Answering a remote change through `alo_approving::Approving`.** The
  surface *shows* a remote change through the existing `ask`; *answering* one
  through `Approving::approve` needs `&mut Turning`, which a remote turn does
  not lend. The test answers through `Arriving::approving` directly. When the
  wire exists, the surface takes the remote turn as a small additive change
  there.
- **A shell shows a principal, not a name.** `Asked::agent` and a grants
  panel carry `machine:<identity>` for a remote machine; the name is in the
  pairings. Resolving it is the shell's, and `alo-shell` is outside this plan.
- **A daemon must refuse a local agent named like a machine.** The check is
  `Origin::names_a_machine`; wiring it into `alo-agentd`'s caller naming is
  for the change that first lets a remote verb in.

## Proposed updates to the shared documents

- **CHANGELOG.md** — nothing user-visible: no surface accepts a verb from
  another machine yet. When the wire lands, the sentence above under *What
  changed, in a sentence a person can read* is the entry.
- **ROADMAP.md** — *a remote agent acts only under a local grant* is built and
  tested from the receiving side; the box stays unticked until two machines
  do it.
- **QUEUE.md** — task 4 of the v0.5 local-network plan done; task 5 (*an
  office that cannot connect still has working AI*) is ready and depends on 3.
