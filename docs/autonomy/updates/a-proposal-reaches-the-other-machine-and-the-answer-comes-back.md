# A proposal reaches the other machine, and the answer comes back

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 7 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Every test in this plan so far made a pairing by handing a `Proposal` and an
`Offer` from one value to another inside one process. Nothing carried them
between two machines: a person at reception could not propose to the studio,
and the studio's person had nothing to confirm. This is the first wire — the
pairing's, before the verb's, because a verb has nothing to prove itself with
until a pairing exists.

## What changed, in a sentence a person can read

A person can now propose a pairing to a machine their own machine found on
the network, and the proposal reaches that machine at the address it was
found at. The other machine answers only once the person there has been
shown what is being asked and the six-digit code, and the person who proposed
is shown the same code. Each person confirms on their own machine, and only
when both have confirmed does either machine keep the pairing. A proposal
that names the wrong machine, that comes from somewhere the other machine
never saw this one, or that arrives while one is already waiting is refused
before anybody is shown anything; a proposal nobody answers lapses from both
machines after ten minutes; and nothing about a proposal is written anywhere
until a pairing is made.

## What changed, by file

All of it in `crates/alo-nearby`, which the plan owns. No other crate is
touched, and every crate that names `alo-nearby` still compiles under the
workspace's clippy.

**The value a surface shows, and the decisions behind it.**

- `src/waiting.rs` (new): `Waiting`, one proposal waiting on this machine —
  what the plan owes `alo-shell`: the proposal, the code once known, whether
  the person here and the person there have confirmed, when it began and
  when it lapses, and where the other machine answers. It holds the
  `Deliberating` underneath and shows a surface none of the key.
- `src/proposals.rs` (new): `Proposals`, every proposal waiting on this
  machine and each thing that can happen to one — `proposed`, `arrived`,
  `answered`, `confirmation_for`, `confirmed_here`, `confirmation_arrived`,
  `withdrawn`, `lapsed` — decided with no socket in sight, so that every
  refusal is held by a test with no port in it. `NotProposed`, one arm per
  refusal, each with a sentence and a one-word wire spelling so the machine
  that asked can say it in its person's language. `WHILE_A_PROPOSAL_WAITS`,
  ten minutes, stated once and read back from `Waiting::until`.
- `src/confirming.rs` (new): `Confirmation`, *the person here confirmed*,
  made with the agreed key over the transcript so that only the machine whose
  offer was in the proposal could have sent it. The section below says why
  a confirmation is proved rather than merely said.
- `src/deliberating.rs`: the key is agreed the moment both offers are known
  — at `asked` on the asked machine, at `answered_with` on the asking one —
  rather than at `agreed`; the private half is consumed then and the
  deliberation holds the key afterwards. `has_agreed(side)` for the surface,
  and `key()` and `transcript()` for the confirmation. `Proposal::checked`
  now keeps whole seconds, because the wire and the transcript both spell
  the duration in seconds. Every refusal from tasks 2 and 6 is unchanged, and
  a second answer to an asking machine is a new one.
- `src/keying.rs`: `Transcript::bytes`, for the confirmation's tag.
- `src/permitting.rs`: `MayAskIts::said` and `MayAskIts::read`, one
  lowercase word per arm on the wire, with a word not on the list read as
  nothing.

**The wire.**

- `src/carried.rs` (new): a proposal's one line on the wire,
  `alo-os/1 proposal <asking> <asked> <seconds> <offer> <may,may>` — seven
  fields, every one on the list the plan names, and an eighth refused rather
  than read around. `Proposal::said` and `Proposal::read`.
- `src/http.rs` (new): the least of HTTP/1.1 both ends share — a `POST`
  with a `content-length` and `connection: close`, a status line back with
  the same two, nothing chunked, nothing over eight kibibytes, nothing that
  is not text. Hand-written for the reason the DNS packets are.
- `src/dialling.rs` (new): the one file that opens a connection. Its address
  is `Found::where_it_answers` or `Waiting::where_the_other_answers`, which
  is made from one; no address is spelt in it, and the guard test below holds
  that.
- `src/receiving.rs` (new): the asked end. `Receiving::accept_one` blocks and
  reads; `Arrived::considered` decides at a moment the caller names *after*
  it arrived, and writes one reply. `Surface`, the trait a shell implements
  to be handed a `Waiting` — the reply to a proposal is written only after
  the surface said it showed it. `Heard`, what happened, for the daemon that
  owns the port. `THE_PROPOSAL_PATH` and `THE_CONFIRMATION_PATH`.
- `src/crossing.rs` (new): the asking end, and either end's confirmation —
  `crossing::propose` and `crossing::confirm`, each a pure transition with
  the dial in the middle. A confirmation is sent first and counted after, so
  a machine never holds a confirmation the other did not hear.
- `src/refusing.rs`: `NotNearby::NotAProposal`, `NotAConfirmation` and
  `NotAMessage`, all about a stranger's bytes.
- `src/words.rs`: nine sentences under `nearby.not-proposed.*`, each with
  its translator's note. `alo-saying` collects the list by count and needs
  no change.
- `src/lib.rs`, `src/looking.rs`, `src/presence.rs`: rustdoc, and the new
  items re-exported.
- `src/testing.rs`: `a_proposal` and `both_sides` for the crate's own tests.

**The tests.**

- `tests/a_proposal_reaches_the_other_machine_and_the_answer_comes_back.rs`
  (new): the acceptance over real sockets on one host, one test per line of
  it, with the refusals beside the agreement. Discovery is done honestly in
  every test — the studio answers a datagram that it exists at its port, and
  the wire dials what that measured.
- `tests/the_local_network_says_no_more_than_a_machine_exists.rs`: the guard
  that this crate dials nothing is narrowed to what is still true, and
  strengthened where it is not — see the decision below.
- Unit tests in every new file: the pure state machine's whole road and
  every refusal in `proposals.rs`; the confirmation tag in `confirming.rs`;
  the seven fields in `carried.rs`; the framing in `http.rs`; the dial in
  `dialling.rs`; the value in `waiting.rs`.

**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 7 marked done, and
task 8 written — *a verb crosses between two machines, and is proven at the
door*, the verb wire the plan's constraint names as the task after this one.

## Decisions

**HTTP on the port presence advertises, and this little of it.** The
corridor already puts questions to that port over HTTP, and the verb wire
after this one carries its proof in a header; one port with one shape lets a
daemon tell a proposal from a question by the path rather than by guessing a
stranger's first bytes. But the framing is the least of HTTP/1.1 that carries
one line there and one back, hand-written, for the reason the DNS packets in
`advertising.rs` are hand-written: what this crate reads off a port anybody
on the network can reach is small and closed, and a dependency that read more
would be more to be wrong in. Two paths, `/alo-os/1/pairing/proposal` and
`/alo-os/1/pairing/confirmation`, both `POST`, both one line of text.

**A confirmation is proved, not merely said.** The plan says a pairing is
kept on each machine only after both people have confirmed on their own, and
each machine learns of the *other* person's confirmation over the wire. A
confirmation anybody on the network could send would let a stranger tell
reception that the studio's person said yes — and reception, whose own person
had said yes, would keep a pairing the studio never made. So the key
`deliberating.rs` agrees is agreed the moment both offers are known, before
either person confirms, and a confirmation is an HMAC with it over the
transcript, from one named machine to the other. Only the two machines whose
offers crossed can make one. Bluetooth's key check at the end of its pairing
is the same step under another name; ADR 0031 is why the key is there to use,
and nothing about the key, the proof or the primitives changes.

**The pure state machine is one file, and the sockets are two thin ones.**
`proposals.rs` decides everything and dials nothing; `receiving.rs` and
`crossing.rs` carry its answers. Every refusal in the plan is therefore held
twice: once with no port in it, in the crate's own tests, and once over real
sockets, in the integration test. The wire tests hold that the road is walked.

**Accepting and considering are two steps**, because nothing in this crate
reads the clock. A moment taken before a blocking accept would be however old
the wait was; `Receiving::accept_one` blocks and `Arrived::considered` takes
the moment from the caller once something has arrived.

**"Shown" is a value handed to a surface that said yes.** `alo-shell` is
outside this plan, so the plan's *only once its person has been shown the
proposal and the Code* is built as a `Surface` trait: the reply to a proposal
is written after the surface was handed the `Waiting` and answered that it
showed it. A surface with nobody in front of it answers `false`, and the
proposal is refused and forgotten rather than answered to an empty room —
`a_proposal_nobody_can_be_shown_is_refused_and_nothing_waits`. Until a shell
implements the trait the code protects nobody, exactly as task 6's report
said of the code it derived; this report says so rather than implying
otherwise.

**The guard test is narrowed to what is true and strengthened where it is
not.** `the_local_network_says_no_more_than_a_machine_exists.rs` held that no
file in this crate opened a connection, which was task 1's constraint and was
true until this task, whose plan puts the wire in this crate. The test now
holds three things instead of one: every file that is not `dialling.rs` or
`receiving.rs` still opens no connection, so discovery's promise stands
unchanged; those two files exist and are the only two; and neither spells an
address of its own — no `parse(` and no `SocketAddr::new` in what ships —
so the wire dials what discovery measured and nothing a person typed. That is
not a weaker gate than the one it replaces; it is the same gate with the two
files the plan asked for named in it.

**Ten minutes.** `WHILE_A_PROPOSAL_WAITS` is long enough for one person to
walk to the other machine and for the other to be found, and short enough
that a proposal nobody answered is not a row somebody finds tomorrow. Stated
once, in `proposals.rs`, and read back from the value a surface shows.

**"The record" at this layer is the list of pairings.** `alo-nearby` does
not depend on `alo-record`; what it has is `Pairings`, and what the wire
reports is `Heard`. The plan's *nothing about any of it is written in either
record until a pairing is kept* is held here as: through a proposal refused,
one answered and left, and one confirmed on one side only, both machines'
pairings stay empty and nothing the wire reports is something kept; the first
`Pairing` comes out of the second confirmation. Whatever writes `alo-record`
for a pairing kept — the daemon that owns the port — has exactly one moment
to do it at, because there is exactly one value to do it from.

**Whole seconds.** `Proposal::checked` now keeps `lasting` to whole seconds.
The transcript already spelt it in seconds, so a fraction was a term the key
was derived without; now the wire, the transcript and the value agree. A
proposal for less than a second is refused as lasting no time, which it
would have been anyway after the transcript.

## Acceptance, and the test behind each line

All in `alo-nearby`; the integration test is
`a_proposal_reaches_the_other_machine_and_the_answer_comes_back`.

| The plan says | The test |
|---|---|
| reaches the machine it names at the address discovery measured, carrying exactly the `Proposal` and nothing else, held by a test that reads the wire and refuses any field not on that list | `a_proposal_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_proposal`; and `carried::tests::a_proposal_with_a_field_not_on_the_list_is_refused` |
| the asked machine answers with its own `Offer` and nothing else, and only once its person has been shown the proposal and the `Code` | `the_asked_machine_answers_with_its_own_offer_and_nothing_else_once_its_person_has_been_shown_it`; `a_proposal_nobody_can_be_shown_is_refused_and_nothing_waits` |
| the asking machine's person is shown the same `Code`, and a pairing is kept on each machine only after both people have confirmed | `both_people_are_shown_the_same_code_and_a_pairing_is_kept_on_each_machine_only_after_both_confirmed` |
| the asking side confirming alone | `the_asking_side_confirming_alone_keeps_nothing_on_either_machine` |
| the asked side confirming alone | `the_asked_side_confirming_alone_keeps_nothing_on_either_machine` |
| a proposal that arrived from an address discovery never measured | `a_proposal_from_an_address_discovery_never_measured_is_refused_before_anybody_is_shown_anything` |
| refused before anybody is shown anything when it names a machine other than the one it arrived at | `a_proposal_naming_another_machine_is_refused_before_anybody_is_shown_anything` |
| when its `Offer` does not read | `a_proposal_whose_offer_does_not_read_is_refused_before_anybody_is_shown_anything` |
| when a second proposal from the same machine arrives while the first waits | `a_second_proposal_from_the_same_machine_while_the_first_waits_is_refused` |
| a proposal nobody answered expires from both machines within a stated time | `a_proposal_nobody_answered_lapses_from_both_machines_within_the_stated_time` |
| nothing is written in either record until a pairing is kept | `nothing_is_kept_on_either_machine_until_a_pairing_is_made` |
| a confirmation only the other machine could have made (the decision above) | `proposals::tests::a_confirmation_that_is_not_the_other_machines_is_refused`; `confirming::tests::a_confirmation_without_the_key_or_for_another_proposal_does_not_verify` |
| discovery still dials nothing, and the wire dials only what it measured | `the_local_network_says_no_more_than_a_machine_exists` · `discovery_dials_nothing_the_wire_dials_what_it_measured_and_nothing_has_a_setting` |

And beside them in the crate's own suite: the whole road with no wire, an
answer for nothing waiting, an answer reflected back, confirming before the
answer, a confirmation for another machine, from another address, and not
by that machine, the key held once both offers are known and not before, a
second answer refused, and every new refusal with a sentence.

## Verified

Windows 11, this development machine, from the checkout, each run in the
foreground and waited on:

| Gate | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings |
| `cargo doc --no-deps -p alo-nearby` | clean |
| `cargo test -p alo-nearby` | 122 unit, 9 + 12 + 5 + 9 integration, 1 doctest, all green; the new file's 12 included |

The full workspace suite was not run here, as instructed; the supervisor
runs it. The four crates that name `alo-nearby` — `alo-approving`,
`alo-asking`, `alo-saying`, `alo-turn` — compile under the workspace clippy
above, and no public signature they use changed.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, as in every report of this plan: the proposal
crosses a TCP connection, the offer comes back over it, and each
confirmation crosses another. Whether a second physical machine on an office
network hears the first is owed to two machines, as task 1's report said.

## Remaining limitations, honestly

- **The surface is nobody's yet.** `Surface` is a trait and `Waiting` is a
  value; `alo-shell` is outside this plan. Until a shell shows the proposal
  and the code, the code protects nobody.
- **No daemon owns the port.** `Receiving` takes a listener somebody bound,
  as `Answering` takes a socket; which daemon binds the advertised port, runs
  discovery, holds `Proposals` and `Pairings` behind one lock and writes
  `alo-record` when a pairing is kept is the verb wire's task or the one
  after it, and this report says so.
- **A declined proposal is not told to the other machine.** A person who
  decides against a proposal withdraws it here, and the other machine's copy
  lapses on its own after ten minutes. A *declined* line would be a kindness
  to the other person and is a small additive change to the wire.
- **The verb wire is not built.** Task 8, written into the plan.
- **Proven, not private.** ADR 0031's own limitation, unchanged: the wire
  authenticates and does not encrypt, and the pairing key is what a later
  channel would be keyed from.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *A person can now propose a
  pairing to a machine their own machine found on the network. The other
  machine answers only once the person there has been shown the proposal and
  the six-digit code, both people see the same code, and a pairing is kept
  only when both have confirmed on their own machine. A proposal that names
  the wrong machine, that comes from somewhere the other machine never saw
  this one, or that arrives while one is already waiting is refused before
  anybody is shown it, and one nobody answers lapses after ten minutes.*
- **ROADMAP.md** — *pairing is mutual, deliberate, revocable and expiring*
  now has a wire between two machines; the boxes stay unticked until two
  machines do it.
- **QUEUE.md** — task 7 of the v0.5 local-network plan done; task 8 (*a verb
  crosses between two machines, and is proven at the door*) is ready and
  depends on 4, 6 and 7. Small follow-ups, both additive: a *declined* line
  on the pairing wire, and the daemon that owns the advertised port.

## The second attempt, 2026-09-13

The supervisor's gates refused the first handoff of this task after the
combined tree had passed every check: holding up
`the_asked_machine_answers_with_its_own_offer_and_nothing_else_once_its_person_has_been_shown_it`
printed no test result at all, and the line under it was the Windows
Subsystem for Linux service reporting that a connection attempt timed out
(`Wsl/Service/0x8007274c`). The same sixteen pieces of evidence had stood up
on the task's own tree forty minutes earlier, in the same log.

A second worker reran that test alone, then the crate's whole suite, then
each of the sixteen evidence lines on its own with `--exact`, all in the
foreground: every one passed with exactly one result, the named test in
about five seconds — discovery's stated patience, not a hang. No line of
code changed on this attempt. What did change is the shape of the checkout:
the refused attempt had been committed locally by the supervisor and
rebased, and the loop's publish path commits afresh from the working tree,
so that commit (`ffe0cc0`, kept in the reflog) was rebased onto the
`origin/main` that had advanced meanwhile and then unstaged, leaving the
same twenty-one files in the tree uncommitted for the loop to stage.
