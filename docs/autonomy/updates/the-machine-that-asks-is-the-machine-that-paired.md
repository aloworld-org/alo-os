# The machine that asks is the machine that paired

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 6 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Tasks 3, 4 and 5 each said it outright: *there is no cryptography here and
none is claimed.* A question reached the studio over plain HTTP from whoever
held the address, and the studio wrote down *the reception machine* because
its pairing named one — not because the connection proved it. Before anything
carries a verb between two machines, that has to be true, or a stranger on the
office WiFi who read a `MachineId` off a discovery packet acts under every
grant B's person made to A. That is the lateral movement ADR 0003 exists to
refuse, and this task is what makes the refusal real.

## What changed, in a sentence a person can read

When two people pair their machines, each machine now keeps a key that only
those two machines hold, made at the moment both confirmed and never sent
anywhere. Every question that goes down the corridor carries a proof made with
it, and the machine that answers checks the proof against its own pairing
before it does anything else — so a machine that was merely seen on the
network, one presenting a paired machine's identity, one replaying an earlier
question, or one whose pairing was undone a second ago is refused with nothing
answered and nothing written down. Both people are shown the same six digits
when they pair, and if the digits differ somebody is standing in between.

## The decision, first

The plan's constraint was that which primitive, and how the secret is
exchanged between two machines that have never met, is a settled decision
before it is code. It is: **ADR 0031 — The pairing is the key**
(`docs/decisions/0031-the-pairing-is-the-key.md`), accepted with this change,
and the code below is built on it rather than beside it.

In one paragraph: X25519 agreement at pairing time, with each side's public
half crossing the wire and no private half ever doing so; HKDF-SHA256 over the
shared secret and the transcript of what both people agreed, giving one
per-pairing key held on each machine's row; a six-digit code from both
identities and both public halves, shown on both machines, as the only defence
against an active attacker at the moment of pairing that does not smuggle a
certificate authority back in; HMAC-SHA256 per message over the sender, the
receiver, the moment and a digest of the exact bytes carried; a replay memory
of tags accepted within a two-minute window. Every primitive is `ring` 0.17's,
which the workspace already pins and compiles through `ureq`'s `rustls`, and
nothing in `alo-nearby` implements a hash, a MAC, a curve or a derivation.

The ADR also records what it does not decide — it does not encrypt the
corridor, does not sign discovery, and does not turn `MachineId` into a public
key — and why four alternatives were rejected, including the nonce every
textbook puts in a proof.

## What changed, by crate

**`crates/alo-nearby`** — the key, the proof, and the judgement.

- `src/keying.rs` (new): `Keying`, one side's part in one pairing — a private
  half that stays and an `Offer` that goes; `PairingKey`, printed as nothing;
  `Transcript`, the bytes both sides derive over; `Code`, the six digits. The
  agreement consumes the private half, so there is one agreement per keying.
- `src/proof.rs` (new): `Proof`, made only from a `Pairing` — which is
  evidence of the key — over exactly the bytes a message carries, at a moment
  the caller names; `said`/`read` for its one wire spelling,
  `alo-os/1 <from> <to> <seconds> <nanoseconds> <tag>`, with every other
  shape refused.
- `src/proven.rs` (new): `Proven::checked`, the receiving machine's whole
  judgement in one place and one order — is a pairing with the named machine
  standing now, is it for this machine, does the tag verify in constant time,
  is it from now, has it been seen — and `NotProven`, one arm each, with
  sentences.
- `src/replaying.rs` (new): `Seen`, the tags accepted within
  `WHILE_A_PROOF_STANDS`, bounded by the window and by a cap that evicts the
  oldest rather than refusing — the file says why.
- `src/hexing.rs` (new): twenty lines of hexadecimal, one spelling, uppercase
  refused.
- `src/deliberating.rs`: **one `Deliberating` per machine.** `Proposal::checked`
  takes the asking machine's `Offer`; `Deliberating::asking` and
  `Deliberating::asked` replace `Deliberating::of`; `answered_with` carries
  the asked machine's offer back and refuses a reflected one; `code()` is the
  six digits, `None` until both offers are known; `agreed(at)` derives the
  key on each side and no longer takes a side, because the side is known at
  construction. `AT_MOST`, `agreed_at`, `is_mutual` and every refusal from
  task 2 are unchanged.
- `src/pairing.rs`: the key on the row; `Pairings::with`, the row itself for
  whatever makes or checks a proof; `NotPaired::NoKey` and
  `NotPaired::NotTheOfferMade`.
- `src/origin.rs`: **`Origin::paired` is gone; `Origin::proven` takes a proof
  and the bytes it is about.** A turn crate that could begin a remote turn
  without a proof no longer compiles.
- `src/presence.rs`, `src/reading.rs`, `src/looking.rs`: `Found::address`,
  the address a machine's answer came from, measured off the datagram rather
  than advertised, and `Found::where_it_answers`. `reading::a_machine_in`
  takes the source address.
- `src/refusing.rs`: `NotNearby::NotAnOffer` and `NotNearby::NotAProof`, both
  about a stranger's bytes.
- `src/words.rs`: four sentences — `nearby.not-paired.start-again`,
  `nearby.not-proven.not-from-the-machine-it-names`,
  `nearby.not-proven.already-used`, `nearby.not-proven.from-another-moment` —
  each with its translator's note. `alo-saying` collects the list by count and
  needs no change.
- `src/testing.rs` (new, `cfg(test)`): two machines paired the way two
  machines are, for the crate's own tests.
- `tests/the_machine_that_asks_is_the_machine_that_paired.rs` (new): the
  acceptance, one test per line, and the refusals beside each.
- `tests/a_pairing_is_made_by_two_people_and_by_nothing_else.rs`: the
  fixtures build both sides; every promise it held still holds.
- `Cargo.toml`: `ring`, via the workspace.

**`crates/alo-asking`** — the corridor carries the proof.

- `src/corridor.rs`: `DownTheCorridor::paired` takes this machine's identity
  and borrows the pairing row; `ask` takes the moment and hands
  `openai::put` a closure that makes the proof over the exact body bytes;
  `THE_PROOF_HEADER`, `alo-pairing`, is the one spelling of where it travels.
- `src/openai.rs`: `put` serialises the body itself, compactly, and sends the
  bytes with `content-type` set — because what is signed has to be what is
  sent, and `send_json` re-serialised on its own terms — and takes an optional
  `Vouching` closure. `hosted.rs` and `served.rs` pass none: they hold no
  key to make a proof with.
- `tests/the_machine_that_asks_is_the_machine_that_paired.rs` (new): the
  studio's end over a real socket. Its record names reception because the
  connection proved it; a stranger presenting reception's identity is refused
  and the record stays empty; a question recorded off the wire and replayed
  is refused and the record gains nothing; a pairing revoked at the studio
  refuses the very next question.
- `tests/an_office_that_cannot_connect_still_has_working_ai.rs`: the studio
  judges every question before answering, with its clock at the hour the day
  has; reception dials `found.where_it_answers()` rather than a typed
  loopback; pairing is two-sided. Re-measured in the namespace — see below.
- `tests/the_machine_down_the_corridor_is_still_an_egress.rs`: fixtures.
- `src/testing.rs`: `paired_as_two_machines`, for the corridor's unit tests.

**`crates/alo-turn`** and **`crates/alo-approving`** — fixtures only.
`Arriving` is unchanged; the one door a remote turn begins from now takes an
`Origin` nothing can make without a proof, and the tests make one. One new
test in `alo-turn`,
`a_stranger_presenting_the_receptions_identity_cannot_begin_a_turn_here`.

**`Cargo.toml`**: `ring = "0.17"` as a workspace dependency with the argument
beside it. **`Cargo.lock`**: `alo-nearby` depends on it; nothing new resolves.

**`docs/quirks.md`**: the ureq `send_json` entry gains the reason the body is
now hand-serialised.

**`docs/autonomy/v0-5-the-local-network-plan.md`**: task 6 marked done, and
task 7 written — *a proposal reaches the other machine, and the answer comes
back*, the pairing wire, which every test in this plan has so far done by
handing values between two sides of one process.

## Decisions

**The ADR and the code in one change.** The plan allowed a worker who could
not finish without the decision to hand over the ADR alone. The decision here
was clear enough to make and the code short enough to build on it in the same
day, so both are here; the ADR is the thing to review first, and the code
follows it clause by clause.

**No nonce.** A tag over a moment and a body digest is already unique per
message and is what the receiver remembers, so a proof needs no randomness and
has no failure path for a machine that cannot draw any. Two identical messages
in one nanosecond do not happen; two in one second are different proofs.
Windows keeps a moment to a hundred nanoseconds, which one unit test notes.

**The key is on the row.** Revoking a pairing removes the key with it, so
*revocable in one action taking effect immediately* is true of proofs for free,
and `Proven::checked` asks the pairings at the moment rather than caching a
key anywhere. A proof from a pairing that expired is refused the same way and
by the same arm — telling a caller which kind of not-paired it is would be
telling it how to become paired, as tasks 3 and 4 decided.

**Proof-then-standing, at the type.** `Origin` is the only thing a remote
turn can begin from, and it is made only from a `Proven`. The plan's *refused
before any grant is asked* is therefore not a check in `alo-turn` that could be
skipped but the absence of a constructor to skip it with. What this does not
yet cover is the second and later messages of an open turn: `Arriving`'s
reading and proposing doors take the pairings and re-ask them at the moment,
as before, and the per-message proof on those is the verb wire's to check with
the same `Proven::checked` before it calls the door. The wire is task 7's
successor and is not built; `arriving.rs` already said so of the wire.

**A refused question reads as *something answered, and not with an answer*.**
The studio refuses an unproven question with `400`, and `alo-answering` reports
it as `WentWrong::NothingUsable`. A `403` would have read as a refused key,
which `alo-answering` rightly refuses to report on a door that sent no key. A
sentence of its own — *the machine down the corridor did not accept this as
coming from here* — would be truer, and belongs on `alo-answering`'s list of
things a person is told; it is a small additive change there and is noted
below rather than made in a crate this task did not otherwise touch.

**The six digits are a value, not a screen.** `alo-shell` is outside this
plan. `Deliberating::code` is what the surface shows, its rustdoc says what the
surface owes, and the test that an interceptor makes the two codes differ is
in `deliberating.rs`. Until a surface shows it, the code protects nobody, and
this report says so rather than implying otherwise.

**`Found` carries an address, and the office dials it.** Task 5 had to type
loopback because discovery discarded the source address; it no longer does,
and the day is now dialled from what discovery measured. The advertisement
still carries no `A` record — the address is measured off the answer, exactly
as task 1 argued it should be.

## Acceptance, and the test behind each line

| The plan says | The test |
|---|---|
| a pairing leaves each machine holding something made when both confirmed and held by nobody else | `alo-nearby` · `the_machine_that_asks_is_the_machine_that_paired` · `a_pairing_leaves_each_machine_holding_the_same_key_and_nobody_else_holding_it` |
| being discovered confers nothing | same file · `being_discovered_confers_no_key` |
| sharing the network confers nothing | same file · `sharing_the_network_and_reading_both_offers_confers_no_key` |
| having read a `MachineId` off the wire confers nothing; a stranger presenting A's is refused before any grant is asked | same file · `a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked`; `alo-turn` · `what_a_remote_agent_may_do_is_what_the_local_person_granted` · `a_stranger_presenting_the_receptions_identity_cannot_begin_a_turn_here` |
| accepted only when it proves it comes from the identity the pairing names | `alo-nearby` · same file · `a_message_is_accepted_only_when_its_proof_holds` |
| a proof replayed from an earlier exchange is refused | same file · `a_proof_replayed_from_an_earlier_exchange_is_refused`; `alo-asking` · `the_machine_that_asks_is_the_machine_that_paired` · `a_question_replayed_off_the_wire_is_refused_and_the_record_gains_nothing` |
| a proof from a pairing since revoked or expired is refused at the moment | `alo-nearby` · same file · `a_proof_from_a_pairing_since_revoked_or_expired_is_refused_at_the_moment`; `alo-asking` · same file · `a_pairing_revoked_at_the_studio_refuses_the_very_next_question` |
| `Found` carries the address a machine answered from | `alo-nearby` · same file · `what_is_found_carries_the_address_it_answered_from` |
| B's record names A because the connection proved it, and stays empty when the proof fails | `alo-asking` · same file · `the_studio_names_reception_in_its_record_because_the_connection_proved_it` and `a_stranger_presenting_receptions_identity_is_refused_and_the_record_stays_empty` |

And beside them, in the crates' own suites: two sides hold the same key and a
third holds a different one, the terms are part of the key, an offer reflected
back is refused before any code is shown, a machine in between shows the two
people different codes, a proof over altered bytes fails, a proof for another
machine is refused as such, a proof from another moment is refused in both
directions, the replay memory is bounded and forgets the aged-out first, and
every new refusal has a sentence.

## Verified

Windows 11, this development machine, from the checkout, each run in the
foreground and waited on:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings |
| `cargo doc --no-deps -p alo-nearby -p alo-asking -p alo-turn` | clean |
| `cargo test -p alo-nearby` | 89 unit, 9 + 5 + 9 integration, 1 doctest, all green |
| `cargo test -p alo-asking` | 100 unit, every integration test green; the new file's 4 included; the office measurement is compiled out here |
| `cargo test -p alo-turn` | 76 unit, 12 + 6 + 4 + 2 + 1 integration, all green; the new test included |
| `cargo test -p alo-approving` | 33 unit, 1 + 7 integration, all green |

Linux, `6.18.33.2-microsoft-standard-WSL2` (Ubuntu under WSL, as root, the
supervisor's own build directory), from the same checkout:

| Gate | Result |
|---|---|
| `cargo clippy -p alo-asking -p alo-nearby --all-targets -- -D warnings` | clean, the measurement file included |
| `cargo test -p alo-asking --test an_office_that_cannot_connect_still_has_working_ai` | 5 passed, 3 ignored (the inner halves, run by the outer five), 10.4 s — the day now dialled from discovery's address, every question judged at the studio, and `OutNoRoutes` still unmoved |

The full workspace suite was not run here, as instructed; the supervisor runs
it. Every crate that names `alo-nearby` — `alo-approving`, `alo-asking`,
`alo-saying`, `alo-turn` — compiles under the workspace clippy above, and the
four whose tests build pairings are green.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, as in every report of this plan. What is new is that
the connection between them now proves where it came from, and that the claim
*there is no cryptography here* is retired: what is here is `ring`'s, named
in ADR 0031, and nothing of it is written in this repository.

## Remaining limitations, honestly

- **No wire for pairing, and none for verbs.** Every test builds both sides
  in one process. Task 7, written into the plan, is the pairing wire; the verb
  wire follows it and is where per-message proofs on an open turn's later
  doors are checked.
- **The code is shown by nobody yet.** `alo-shell` is outside this plan.
- **A refused question is reported as *nothing usable* rather than as
  *not accepted as coming from here*.** A sentence of its own belongs on
  `alo-answering`'s list; proposed below.
- **The corridor is proven, not private.** ADR 0031 says so: a proof
  authenticates and does not encrypt, and confidentiality on the local
  network is a decision with its own trade-offs, not made here.
- **Clocks.** Two paired machines' clocks must agree within two minutes for
  a proof to stand, and the refusal says to check them.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *When two machines are paired,
  each now holds a key only the two of them have, and every question one puts
  to the other proves it came from the machine it says. A machine that was
  merely seen on the network, one pretending to be a paired machine, one
  repeating an earlier question, or one whose pairing was undone is refused
  with nothing answered and nothing written down. Two people pairing their
  machines are shown the same six-digit code.*
- **ROADMAP.md** — *pairing is mutual, deliberate, revocable and expiring*
  now includes that the pairing is what proves a machine is the one that
  paired; the boxes stay unticked until two machines do it.
- **QUEUE.md** — task 6 of the v0.5 local-network plan done; task 7 (*a
  proposal reaches the other machine, and the answer comes back*) is ready and
  depends on 6. A small follow-up for `alo-answering`: a `WentWrong` reason
  for *the machine down the corridor did not accept this as coming from here*,
  additive, with its sentence.
