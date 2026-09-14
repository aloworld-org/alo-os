# The person's door proposes, confirms and revokes a pairing, and a pairing outlives a restart

- Date: 2026-09-14
- Workstream: v0.5 the local network, task 12 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Task 10 bound the port and left two things a real machine cannot do without:
the person's surface confirmed a pairing by reaching into `TheNetwork`'s lock
from outside the loop, which only a test can do, and pairings were not kept
between restarts, so a machine switched off at night was paired with nothing
in the morning. Task 11 finished the person's door for a change and a
question. This finishes it for the pairing itself, and keeps what the two
people made.

## What changed, in a sentence a person can read

From their own shell, a person can now propose pairing with a machine on the
office network by the identity discovery found it by, be shown the six-digit
code and the list of what the other machine would be allowed to ask for,
confirm with that code, see what is paired and what is waiting, and revoke a
pairing in one action that stops the very next request from that machine. A
pairing is written down on the machine, under the same protection as the
grants, and is there again after a restart for exactly as long as it was
made for: one that ended while the machine was off is gone when it wakes. An
agent is refused every one of these in the words it gets for trying to
approve something.

## What changed, by crate

**`crates/alo-protocol`** — the person's door gains four requests and four
answers, additively.

- `src/pairing.rs` (new): `WaitingToPair` (the proposal, its side, the
  enumerated list as `Permitted` — the wire word beside the sentence — the
  duration, the `code` once known, `Confirmed { here, there }`, and
  `lapses_in`), `Paired` (`machine`, `may`, `made_ago`, `ends_in`),
  `AfterConfirming` (`waiting-for-the-other-person`, `paired`,
  `paired-until-a-restart`) and `AfterRevoking` (`revoked`,
  `revoked-until-a-restart`). Durations only, never a moment; no address.
- `src/asked.rs`, `src/person.rs`: `pair { machine, may, seconds }`,
  `confirm-pairing { machine, code }`, `revoke-pairing { machine }` and
  `pairings {}`; `FromAPerson::is_about_a_pairing`. A field for an address, a
  port or a name is refused as unreadable, held by a test. `FromAPerson` is no
  longer `Copy`, which nothing outside the crate relied on.
- `src/told.rs`, `src/to_a_person.rs`: `pairing`, `confirmed`, `revoked`,
  `pairings`, with constructors and accessors.
- `src/agent.rs`, `src/to_an_agent.rs`: the four requests are
  `NotForAnAgent` and the four answers `NotAnAnswerForAnAgent`.

**`crates/alo-nearby`** — what a pairing looks like written down.

- `src/keeping.rs` (new): `written(&Pairings, now)` and `read(text, now)`,
  TOML with `format = 1` and one `[[pairing]]` per machine carrying `with`,
  `may`, `made`, `ends` and `key`. Every row is made again through
  `Pairing::between` after the checks a proposal is held to; a row that has
  ended is neither written nor read back; a field nobody declared, two rows
  for one machine, a widened list, a key that is not one, and a file from
  another format are each refused whole. `NotWrittenDown` is the closed list,
  in English for the service log. `THE_PAIRINGS_FORMAT`.
- `src/keying.rs`: `PairingKey::of` is `pub(crate)` for the file, no longer
  `cfg(test)`; `PairingKey::LENGTH`.
- `Cargo.toml`: `serde` and `toml`, for the file and nothing on the wire.

**`crates/alo-remembering`** — where the file lives.

- `src/believing.rs` (new): the three rules about who may have written a file
  and the whole-or-nothing replacement, split out of `keeping.rs` (law 4) so
  the grants and the pairings are held to one rule written once.
- `src/pairings.rs` (new): `THE_PAIRINGS` (`/var/lib/alo/pairings.toml`),
  `pairings_remembered`, `pairings_kept`.
- `src/refusing.rs`: `NotRemembered::NotPairings`, carrying `alo-nearby`'s
  sentence whole. `NotWritten`'s sentence now says *list* rather than
  *grants*, since it is about both files.
- `tests/a_pairing_survives_a_restart.rs` (new): the plan's own questions on
  a real disk, through the one road to a pairing.

**`crates/alo-turn`** — `Turning::a_pairing_was_kept`, the record door while
a local turn holds the machine, beside `Machine`'s and `Arriving`'s.

**`crates/alo-agentd`** — the door, and the file behind the lock.

- `src/pairing.rs` (new): `Nearby` (the one lock and where a machine is
  looked for), `AboutAPairing` (the four cut out of `FromAPerson`), and
  `answered_to`: propose through `alo_nearby::crossing::propose` to the
  machine discovery found now; confirm with the code, refused for nothing
  waiting, for no code yet, and for a code that does not match, then through
  `crossing::confirm`, keeping and writing the pairing and recording
  `Entry::paired` when it completes; revoke, immediate in memory and then
  written; list.
- `src/network.rs`: `KeepingPairings` (writes the whole list, reads
  nothing), `NothingKeepsPairings`, `TheNetwork::remembering(here, pairings,
  keeping)`, `Shared::written_down`.
- `src/keeping_pairings.rs` (new): `ThePairingsFile`, the implementation that
  ships, holding the path `main` gave it.
- `src/looking.rs`: `LookingFor` and `found_by_name` — a machine looked for
  by identity on the link, at the moment, answered with the one that said so
  at the address it answered from.
- `src/wire.rs`: `looks_at` (the multicast group on a real machine, the
  other side's socket in a test) and `Wire: LookingFor`.
- `src/surface.rs`: `AtThePersonsDoor` — a proposal is shown by waiting on
  the person's door; `src/serving.rs` hands the wire that surface only while
  a shell is connected and `NobodyToShowItTo` otherwise.
- `src/answering.rs`: the four are dispatched to `crate::pairing` before the
  turn is asked which door holds the machine, so a person pairs from any
  state; `what_a_person_said` takes a `Nearby`.
- `src/hearing.rs`: a pairing completed by a confirmation on the wire is
  written to the disk under the same lock it was kept under.
- `src/holding.rs`: `Holding::a_pairing_was_kept`. `src/starting.rs`,
  `src/main.rs`: the pairings read at start beside the grants, with the same
  two machines told apart (`NotStarted::NoPairings`), and the file handed in.
- `src/words.rs`: five words — not a machine, no such machine on the network,
  not something a pairing may permit, the code does not match, nothing is
  paired with that machine — sixteen in all.

**`crates/alo-changing`** — `door.rs` matches the new answers as *not for this
request*, as it did the others.

**`docs/contracts/daemon-protocol.md`**: the four requests and four answers.
**`docs/contracts/pairings-file.md`** (new): the file, its rules, and why the
key is in it. **The plan**: task 12 done; task 13 was already written.
**`Cargo.lock`**: two path dependencies; nothing new resolves.

## Decisions

**The key is in the pairings file, beside the identity, and not in the
keyring.** The plan asked which store, with the care a credential gets. Three
reasons, in order of weight. The identity the key is paired *to* is already a
file in `/var/lib/alo`, and half a pairing in each of two stores is a machine
that can wake with one and not the other. A keyring (ADR 0022) can be locked
when the service starts — this crate already has the sentence for it — and a
machine that read its pairings at start would then be paired with nothing until
somebody typed a password, which is the *paired with nothing in the morning*
this task exists to end, wearing a prompt. And what the key lets its holder do
is bounded on the other machine by that machine's grants and by the row's
expiry, so it is exactly as sensitive as the grants file beside it: whoever can
read this file could already rewrite what this machine's agent may reach. The
file is `0600`, refused if anybody else could write it, refused whole if any
row does not hold, and a key never outlives its row because an ended row is
neither written nor read.

**The shape is `alo-nearby`'s and the file is `alo-remembering`'s.** A
`Pairing` has no public constructor, on purpose, and a read-back constructor
would be the road round the mutual confirmation `deliberating.rs` guards. So
the writing and reading of a row live in the crate that owns the key
(`alo_nearby::keeping`), through the `pub(crate)` constructor, and
`alo-remembering` owns where the text lives and who is believed about it — the
same split as the grants (`alo-capability` checks, `alo-remembering` keeps).
Its trust logic was split into `believing.rs` so that both files are held to
one rule written once.

**The daemon writes this file, and that is the one difference from the
grants.** A grant is made on the person's side and the daemon only reads;
a pairing is made by two people on two machines and the daemon is what hears
the second of them. So `TheNetwork` holds a `KeepingPairings` that can write
the list whole and read nothing, `main` reads once at start and hands a value
in, and there is still no road from either door to a path. The list in memory
moves first and the disk second, always: a revocation is immediate whatever
the disk says, and a file that could not be written is told to the person as
`paired-until-a-restart` or `revoked-until-a-restart` — the true sentence —
rather than reversing what they did or saying nothing. The service log gets
the English.

**A proposal names an identity, and discovery is asked at the moment.** The
`pair` request has no field for an address, a port or a name (a test refuses
each). The daemon asks the link who is here, takes the one machine that
answered by that identity at the address it answered from, and proposes to
that; nothing typed is dialled (ADR 0003). On a real machine the question goes
to the multicast group; in a test, to the socket the other side bound. A
machine that does not answer is *no such machine on the network*, and nothing
is proposed.

**A proposal is shown by waiting on the person's door.** `alo-shell` is
outside this plan, and the surface that draws a code is its. What the daemon
owes it is the value: a proposal that arrives is answered and waits on the list
`pairings` returns, with its code and list, and `confirm-pairing` takes it
from there — the way a change waiting for approval is shown. The wire is
handed that surface only while a shell is connected on the person's door;
with none, nobody can be shown it and it is refused as *nobody to show it
to*, which is the true word. Task 10's test of a pairing kept on the wire now
connects a shell, as a real machine would have.

**A confirmation carries the code.** So that what is confirmed is what was
compared across the room (ADR 0031), and a shell cannot confirm whatever
happens to be waiting: the code that does not match is refused in its own
words, beside nothing waiting and not answered yet.

**A revocation is not written to the record.** Nothing in the plan asks for
it, and it follows the grants: a person's own list changing is not something
that happened to the machine, and the next refusal from that machine is what
is recorded. Adding a `Happened` arm is a closed-list decision for
`alo-record` and `alo-recounting`, left to whoever needs it.

## Acceptance, and the test behind each line

| The plan says | The test |
|---|---|
| the person's door gains requests that propose to a machine discovery measured, confirm the one waiting, revoke one, and list what is paired and what is waiting | `alo-agentd` · `serving::tests::a_pairing_made_on_the_persons_door_outlives_a_restart_and_its_expiry_survives_with_it` (end to end, over the port and the socket), `pairing::tests::a_proposal_to_a_machine_found_comes_back_with_the_code_to_show`, `pairing::tests::the_list_says_what_is_paired_and_what_is_waiting` |
| each carrying nothing that could name a machine discovery did not measure | `alo-protocol` · `person::tests::a_proposal_naming_an_address_is_not_a_request`, `asked::tests::a_proposal_cannot_name_an_address_and_a_confirmation_needs_its_code`; `alo-agentd` · `pairing::tests::a_proposal_to_a_machine_discovery_cannot_find_or_on_terms_no_pairing_has_is_refused`, `looking::tests::a_machine_looked_for_by_name_is_found_only_if_it_is_the_one_that_answered` |
| each refused on the agent's door in the words an agent approving something gets | `alo-agentd` · `serving::tests::the_four_about_pairing_on_the_agents_door_are_refused_as_an_approval_would_be` |
| what the person is shown to confirm is the code and the enumerated list | `alo-protocol` · `pairing::tests::what_is_waiting_carries_the_code_and_the_list_and_reads_back` |
| a confirmation for nothing waiting, for a code that does not match, and from an agent are three refusals | `alo-agentd` · `pairing::tests::a_confirmation_for_nothing_waiting_or_a_code_that_does_not_match_is_refused`, and the agent's-door test above |
| a pairing kept is remembered under the person's own file with the same trust the grants file has, and read again at start, so that a pairing survives a restart | `alo-remembering` · `a_pairing_survives_a_restart::a_pairing_kept_before_a_restart_still_proves_the_other_machine_after_one`, `a_file_that_cannot_be_believed_pairs_this_machine_with_nothing`; `alo-nearby` · `keeping::tests::what_was_written_reads_back_and_the_key_still_proves_the_other_machine`; `alo-agentd` · `serving::tests::a_pairing_made_on_the_persons_door_outlives_a_restart_and_its_expiry_survives_with_it` |
| its expiry survives with it, tested by a restart across the moment it ends | `alo-remembering` · `a_pairing_survives_a_restart::a_restart_across_the_moment_a_pairing_ends_finds_nothing`; `alo-nearby` · `keeping::tests::the_expiry_survives_and_a_row_that_ended_is_not_read_back`; the serving test above, with a real six-second pairing |
| a revocation from the door takes effect on the next verb and the next question, tested through the door rather than the lock | `alo-agentd` · `serving::tests::a_pairing_revoked_from_the_persons_door_refuses_the_next_verb_and_the_next_question` |
| the key the pairing holds is kept with the care a credential gets — which store, and why, decided and written up | the decision above and `docs/contracts/pairings-file.md`; `alo-nearby` · `keeping::tests::a_key_that_is_not_a_key_is_refused_by_its_length_and_not_quoted`; the `0600` mode asserted in the two restart tests |
| a pairing kept is written down and recorded once; a revocation is immediate whatever the disk says | `alo-agentd` · `pairing::tests::confirming_second_keeps_the_pairing_writes_it_down_and_records_it_once`, `pairing::tests::revoking_is_immediate_and_says_whether_the_disk_agrees`, `network::tests::what_was_read_back_is_held_and_every_change_is_written_whole` |

And beside them: a revoked pairing does not come back
(`alo-remembering` · `a_pairing_revoked_before_a_restart_does_not_come_back`);
a hand-edited row is refused whole (`alo-nearby` · `keeping::tests::a_row_this_crate_would_not_make_is_refused_whole`,
`alo-remembering` · `pairings::tests::a_hand_edited_pairing_is_refused_whole`);
the four read back and are told apart from the turn's requests
(`alo-protocol` · `person::tests::the_four_about_pairing_are_a_persons_and_answer_no_change`);
and the daemon's file is what a restart reads back
(`alo-agentd` · `keeping_pairings::tests::what_the_service_writes_is_what_a_restart_reads_back`).

## Verified

Ubuntu under WSL, this development machine, from the checkout, each run in
the foreground and waited on, building in
`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

| Gate | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings, exit 0 |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps` on the six crates touched | clean, exit 0 |
| `cargo test -p alo-agentd` | **283 unit** (was 268), 5 + 1 + 1 + 4 + 12 integration, all green |
| `cargo test -p alo-nearby` | 135 unit (was 125), 9 + 12 + 5 + 9, 1 doctest, all green |
| `cargo test -p alo-remembering` | 28 unit (was 27), 4 + 4 integration, all green |
| `cargo test -p alo-protocol` | 102 unit (was 94), 8 + 7 + 6, 1 doctest, all green |
| `cargo test -p alo-turn` | 89 unit, every integration target, all green |
| `cargo test -p alo-changing` | all green |
| each evidence test alone, `--exact`, as the supervisor runs it | one result line each, all green |

The full workspace suite was not run here, as instructed; the supervisor runs
it. Every crate that names `alo-protocol`'s `ToAPerson` compiles under the
workspace clippy above (`alo-changing` gained the arms); no public signature
any crate already had changed except `alo_agentd::starting::until_stopped`
(two more arguments, the daemon's own) and
`alo_agentd::answering::what_a_person_said` (one more), both this crate's.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, as in every report of this plan: reception is a thread
answering discovery on a socket of its own and receiving on a listener of its
own, and the daemon looks for it at that socket rather than at the multicast
group. Whether a second physical machine is found by identity through the
group is owed to two machines.

## Remaining limitations, honestly

- **No shell draws the code or keeps a name.** The values are on the door;
  `alo-shell` is outside this plan. Machines are spoken of by identity.
- **A proposal is refused when no shell is connected**, as *nobody to show
  it to* — the true word, and `alo-shell` connecting at sign-in is what makes
  it rare.
- **A revocation is not recorded**, for the reason above; the next refusal is.
- **Looking for a machine takes two seconds** (`WHILE_LOOKING`), during which
  the loop is in the look: a person waiting at a screen, once per proposal.
- **A pairing that could not be written stands until a restart**, and the
  person is told so. Nothing retries the write.
- **Proven, not private**, unchanged from ADR 0031.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *From their own shell a person
  can propose pairing with a machine on the local network by the identity it
  was found by, see the code and what the other machine would be allowed to
  ask for, confirm with that code, list what is paired and what is waiting,
  and revoke a pairing in one action that stops the very next request. A
  pairing is written down beside the grants, under the same protection, and
  is there again after a restart for exactly as long as it was made for. An
  agent is refused all of it.*
- **ROADMAP.md** — no box moves; *pairing: mutual, deliberate, enumerated,
  revocable in one action, and expiring* now has a person's door and a file,
  and stays unticked until two machines do it.
- **QUEUE.md** — task 12 of the v0.5 local-network plan done; task 13 (*a
  turn asks the local model in the envelope*) ready, depending on 11 and on
  the Mac lane's task 14.
