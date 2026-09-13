# An office that cannot connect still has working AI

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 5 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

`docs/features.md` promises, at v0.5: *★ The whole of it works with no
internet at all. An office that cannot connect still has working AI.* ADR 0003
says the same thing as a consequence: *everything here works with no internet,
which is what makes air-gapped operation possible rather than merely claimed.*

This task is the measurement — in the shape `a_day_that_never_left.rs` already
uses for one machine, across two — with the unreachability **enforced by the
kernel** rather than assumed by the test, and with one production change that
the measurement found was owed: what a person in that office is told about
their machine's connectedness.

## What changed, in a sentence a person can read

A machine on an office network with no way to the outside finds the machine
down the corridor, pairs with it, and puts its questions there all day — with
the indicator firing for each one and both machines' records written exactly
as they would be with a connection. And when a question is bound for a
provider on the internet, the person is told the true thing, once: *there is
no way there from the network this machine is on, so the question was not
sent* — not that the provider failed, which it was never asked, and not that
the machine is offline, which no machine can know about itself.

## How the office is made, and why it is not a mock

Every measured test runs inside a **network namespace of its own** with only
the loopback interface in it: a real kernel with a real routing table that has
no route to anything beyond this machine. `ip route` inside it prints nothing.

It is made by util-linux's `unshare --map-root-user --net`, with loopback
brought up by iproute2, and the test binary is run again inside it with the
inner test named — the way `alo-bounding`'s tests re-run themselves inside a
control group. It is not made from Rust because the one crate in this
workspace that could do so, rustix 1.1.4, marks its safe `unshare`
`#[deprecated]` and the replacement needs an `unsafe` block, which this
repository does not add for a test. `docs/quirks.md` has the entry.

**The measurement is the kernel's own.** `/proc/net/snmp` carries
`OutNoRoutes`, the count of packets this namespace was asked to send and had no
route for; the kernel increments it at the moment a `connect` or a `sendto` is
refused, before a byte goes anywhere. The acceptance's *a test that fails if a
single packet is addressed off-network* is that number, read before the day
and after it, unchanged. **The control comes first:** one stream and one
datagram addressed at `192.0.2.1` — an address reserved for documentation, so
that even a namespace that failed to take effect would reach nothing anybody
owns — are refused with *network unreachable* and counted, two for two. Only
then does the zero mean what it says.

Measured, on `6.18.33.2-microsoft-standard-WSL2`:

| Inside the office | `OutNoRoutes` moved by |
|---|---|
| the control: one stream and one datagram beyond the building | 2 |
| a working day: discovery, pairing, eight questions down the corridor | 0 |
| five questions bound for a provider, told once | 5 |

And the day inside the office, compared entry for entry and line for line with
the same day walked on this host as it is, was **the same day**: eight `Left`
entries at reception naming the studio, eight `AnsweredForAnotherMachine`
entries at the studio naming reception, eight indicator lines reading *@mail
is asking a question of the studio machine, on your network*, eight answers
saying they came from the studio machine on this network. The day runs under
`SourcePolicy::InTheBuilding` — the rule an office like this one would set —
and the corridor stays inside it.

## What changed, by crate

**`crates/alo-answering`** — the true sentence.

- `src/wrong.rs`: `WentWrong::NoWayThere`, the ninth reason and the only one
  about **this machine** rather than the far end: the kernel found no route to
  the address, nothing was sent, and the place was never reached. It can
  happen towards any place — a paired machine on a link that went down, a
  provider from an office with no internet, and even this machine's own
  address on a machine whose loopback is down — so it is refused nowhere, and
  the test that walks every reason across every place now walks it too.
- `src/words.rs`: `answering.wrong.no-way-there`, *nothing was answered
  {source} — there is no way there from the network this machine is on, so
  the question was not sent*, with a note that says what must survive
  translation and what must not appear. A new test,
  `having_no_way_there_says_nothing_about_being_offline`, refuses the
  sentence and its note if either calls the machine offline, disconnected, or
  without internet — none of which a machine can measure about itself. The
  list is fifteen; `alo-saying` collects it, held by that crate's own suite.
- `src/failed.rs`: the one `match` the compiler asked about.

**`crates/alo-asking`** — the wire reads it, and the measurement.

- `src/openai.rs`: `what_went_wrong` turns a socket error whose kind is
  `NetworkUnreachable` or `HostUnreachable` into `NoWayThere`, and nothing
  else into it — a refused connection, a reset, a permission the boundary
  refused, and a name that did not resolve stay `NothingAnswered`. Two unit
  tests, one for each direction.
- `tests/an_office_that_cannot_connect_still_has_working_ai.rs` (new,
  Linux only): the acceptance, one outer test per line, each running one
  inner test inside the office and reading what it measured; and the refusal
  beside it, run outside the office on purpose — a provider on this host that
  refuses the connection is reported as nothing having answered, and the
  telling does not say *no way there* about somewhere the machine just
  reached.
- `Cargo.toml`: `alo-telling` as a dev-dependency, for the sentence and the
  once; `Cargo.lock` follows.

**`crates/alo-telling`** — said once.

- `tests/an_office_that_cannot_connect_is_told_so_once.rs` (new): against the
  vocabulary a real machine holds — the word is new, and a word nothing
  collects is a key on somebody's screen — what a person is told is true, it
  is said once, the machine's own retries say nothing, the person asking
  again is answered, the studio down the corridor is offered and never
  chosen, and no way to a different place is a different telling.
- `src/lib.rs`, `src/telling.rs`, `src/told_once.rs`: *eight* reasons became
  nine in two doc comments, and the test that words every reason words this
  one.

**`docs/quirks.md`**: what it took to make the measurement, under *Pinned
engines*: the deprecated safe `unshare`, the per-namespace counter and what it
counts, ureq bailing on the first non-refused connect error so the count is
exact, and a search that waits out its patience.

**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 5 marked done, and
task 6 written — see below.

## Decisions

**The connectedness a person is told about is one address's route, and it is
measured.** The plan says *what a person is told about the machine's
connectedness is true*. The only thing a machine can know truly is what the
kernel answered about one address: *no route*. It cannot know it is offline —
a route may exist and be dead — and it cannot know *the internet* is gone.
So the sentence says exactly the measured thing, the note forbids the two
words that would overclaim, and the mapping reads the kernel's two error
kinds and nothing else. A machine that guessed *offline* from a timeout would
be telling somebody a thing it did not measure, at the moment they are least
able to check it.

**A new `WentWrong` reason rather than a wording of an existing one.** Before
this change an office with no internet was told *nothing answered by Mistral,
in the EU* — a sentence about a service that was never asked, sending
somebody to check a provider that is fine. The crate's rule is that a reason
belongs on the list when it is a different thing to be **told**, and this
one is: it is the only reason on the list that is about this machine. It is
refused nowhere, because it can be true towards any address, and a person
told *there is no way there* is sent nowhere wrong.

**Said once per place, which is `alo-telling`'s existing rule.** An
unavailability is where the question was put and what went wrong there. An
office with two providers configured is told about the second one too — that
is a sentence naming a different place, and the crate's own test for it
(`the_same_reason_somewhere_else_is_said`) is the rule this task keeps rather
than bends. The machine's own retries towards either say nothing. The
acceptance's *said once* is measured as one telling for five attempts.

**The namespace is util-linux's, and the test fails rather than skips where
there is none.** The plan's constraint is that nothing here is a mock of the
network; a namespace is not a mock, it is the kernel with one fewer route. It
needs `unshare --net` to be permitted, which on this machine means root and
which the supervisor is. Where it is not, the test fails naming what it
needs — the way `a_turn_without_a_boundary_does_not_run` fails on a machine
without a boundary — because a measurement that skips is a promise that
passes for the want of being checked. On Windows the file is compiled out.

**The reception machine dials loopback because `Found` carries no address.**
Task 1 decided that an advertisement carries no `A` record and that the
address a machine is reachable at is the source address of its own answer —
and then `Looking::found` discards that source address. The test dials the
port discovery returned at this host's loopback address, which is where the
studio is; a real office needs the address as well as the port, and task 6
below asks for it.

**Documentation addresses in the control, not a public IP.** `192.0.2.1` is
reserved (RFC 5737) and reaches nothing anybody owns. A control that addressed
a real public address would, on the day the namespace failed to take effect,
send a packet off somebody's development machine in the name of proving that
nothing does.

## Acceptance, and the test behind each line

| The plan says | The test |
|---|---|
| with every non-local address unreachable, two paired machines discover each other, pair, ask and answer, and the record and the indicator are as they would be with a connection | `alo-asking` · `an_office_that_cannot_connect_still_has_working_ai` · `with_no_route_off_the_network_two_machines_still_find_each_other_pair_ask_and_answer` |
| measured rather than reasoned, with the unreachability enforced in the test rather than assumed | `alo-asking` · same file · `a_public_address_really_is_unreachable_where_this_is_measured` |
| nothing anywhere in the road retries against a public address, checked by a test that fails if a single packet is addressed off-network | `alo-asking` · same file · `nothing_on_the_road_addresses_a_single_packet_off_the_network` |
| what a person is told about the machine's connectedness is true and said once, through `alo-telling` | `alo-asking` · same file · `a_question_bound_for_the_internet_is_told_once_and_truthfully`; `alo-telling` · `an_office_that_cannot_connect_is_told_so_once` · `what_a_person_is_told_about_having_no_way_there_is_true` and `having_no_way_there_is_said_once` |
| and the refusals beside them | `a_far_end_that_refused_is_not_said_to_be_out_of_reach`; `openai::tests::a_far_end_that_was_reached_and_refused_is_not_said_to_be_out_of_reach`; `words::tests::having_no_way_there_says_nothing_about_being_offline`; `the_machine_down_the_corridor_is_offered_and_never_chosen` |

## Verified

Windows 11, this development machine, from the checkout:

| Gate | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings |
| `cargo test -p alo-answering` | 49 unit, 4 + 6 integration, all green |
| `cargo test -p alo-asking` | 100 unit, every integration test green; the measurement file is compiled out here |
| `cargo test -p alo-telling` | 53 unit, every integration test green, the new file's 4 included |
| `cargo test -p alo-saying` | green — the new word is collected |
| `cargo doc --no-deps` for the three library crates | clean |

Linux, `6.18.33.2-microsoft-standard-WSL2` (Ubuntu under WSL, as root, the
supervisor's own build directory), from the same checkout:

| Gate | Result |
|---|---|
| `cargo clippy -p alo-asking -p alo-telling -p alo-answering --all-targets -- -D warnings` | clean, the measurement file included |
| `cargo test -p alo-asking -p alo-telling -p alo-answering` | all green; `an_office_that_cannot_connect_still_has_working_ai`: 5 passed, 3 ignored (the inner halves, run by the outer five), 10.4 s |
| the three inner measurements, run by hand inside the namespace | `alo:off-network 0` for the day, `2` for the control, `5` for the telling; `alo:told-once` |

The full workspace suite was not run here, as instructed; the supervisor runs
it. Every crate that matches on `WentWrong` was compiled by the workspace
clippy above; the one new arm was answered in `failed.rs` and in the test
lists that walk the whole enum.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, in a namespace that really has no route out. Whether
an office switch carries the same packets between two machines is owed to two
machines, as every report in this plan has said, and nothing here changes
what task 3 said about there being no cryptography.

## Remaining limitations, honestly

- **Linux and a permitted `unshare --net` only.** On Windows the measurement
  is compiled out, so a green Windows suite says nothing about this promise.
  On a Linux machine where a namespace cannot be made the test fails, naming
  util-linux, rather than passing for the want of a check.
- **`Found` has no address.** Discovery returns a port and an identity and
  discards the address the answer came from; the test dials loopback because
  that is where the studio is. Task 6 asks for the address.
- **A search waits out its patience.** `Looking::found` keeps listening for
  the whole five seconds it is given even after the studio has answered, so
  the measured day takes five seconds where the road takes milliseconds.
  Right for a search — it cannot know how many will answer — and noted so
  that nobody reads the time as the road being slow.
- **Name resolution is not measured.** ADR 0020 has the caller resolve a
  provider's name before any boundary is entered and hand the door addresses;
  the test hands an address. What a name that cannot be resolved in an office
  with no internet says to a person is `alo-turn`'s, where the resolving is,
  and is not touched here.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release, in words a person can read: *A
  machine on a network with no way to the outside is told so truthfully, and
  once. When a question is bound for a provider it cannot reach, the message
  says there is no way there from this network and that the question was not
  sent — rather than that the provider failed, or that the machine is
  offline. Questions to a paired machine on the same network are unaffected
  and work with no internet at all, which is now measured.*
- **ROADMAP.md** — *the whole of it works with no internet at all* is
  measured on one host inside a network namespace with no route out; the box
  stays unticked until two machines do it.
- **QUEUE.md** — task 5 of the v0.5 local-network plan done; task 6 (*the
  machine that asks is the machine that paired*) is ready and depends on 2,
  3 and 4.
