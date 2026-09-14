# The person's door chooses a paired machine to answer their questions

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 15)
**Contributor:** Claude (Opus 5), in the `C:\dev\alo-os-claude` checkout
**Status:** ready for integration

## What changed, for a person

Task 14 made a machine you are paired with something that can answer your
questions, but nothing a shell could send actually made that choice. The
pairings live inside the daemon, so a shell would have had to keep its own copy
and hope it was still true. Now the shell asks the daemon. It names the machine
by its identity, and the daemon checks the choice against the same pairings
that decide the next question. It then writes the choice into your own settings
file, and the next turn's question goes to that machine. A choice is refused, and
nothing is written, when:

- the pairing does not include that machine's models;
- the pairing has run out or been revoked;
- what was named is not a machine;
- your session has nowhere to keep settings;
- your settings file is there but cannot be read (it is left alone).

An agent cannot make the choice. The list of pairings now says, for each
machine, whether it can be chosen to answer questions, so a shell only offers
the ones that can.

## What changed, in the code

| Where | What |
|---|---|
| `crates/alo-protocol/src/asked.rs`, `person.rs`, `agent.rs` | `choose-machine-to-answer {machine}` → `FromAPerson::ChooseMachineToAnswer`; the agent's door refuses it as `NotForAnAgent` (the words an approval gets) |
| `crates/alo-protocol/src/told.rs`, `to_a_person.rs`, `to_an_agent.rs` | `chosen-to-answer {machine}` → `ToAPerson::ChosenToAnswer`, `ToAPerson::chosen_to_answer`, `machine_chosen_to_answer`; refused as an agent's answer |
| `crates/alo-protocol/src/pairing.rs` | `Paired::may_answer_questions` (`#[serde(default)]`), `that_may_answer_questions` |
| `crates/alo-agentd/src/choosing_to_answer.rs` (new) | `chosen_to_answer`: identity → somewhere to keep it → the file as it stands → `Choosing::answered_by_a_paired_machine` with the daemon's `Shared` as `WhoMayBeAsked`, under the lock |
| `crates/alo-agentd/src/answering.rs` | the person's door dispatches the request there, whatever holds the machine |
| `crates/alo-agentd/src/questions.rs`, `holding.rs` | `Questions::where_the_settings_are` (the file the next turn reads); `Holding::questions` |
| `crates/alo-agentd/src/pairing.rs` | the list sets `may_answer_questions` from `WhoMayBeAsked for Shared`, the predicate the corridor asks |
| `crates/alo-agentd/src/reaching.rs`, `crates/alo-changing/src/door.rs` | exhaustive matches take the new variants |
| `crates/alo-agentd/src/words.rs` | `agentd.nowhere-to-keep-the-choice` |
| `crates/alo-agentd/src/testing.rs`, `corridor.rs` | the studio stub (`the_studio_answering`, `TheStudioIsAt`) moved from `corridor.rs`'s tests to `testing.rs`, shared by both files |
| `crates/alo-choosing/tests/no_agents_door_reaches_these_settings.rs` | the guard now holds *one daemon file names the writer, and only the person's door reaches it* (see decision 1) |
| `docs/contracts/daemon-protocol.md`, `docs/contracts/person-settings.md` | the request, the answer and the list's field, additively |
| `docs/autonomy/v0-5-the-local-network-plan.md` | task 15 marked done; task 16 written |

## Decisions, and why

1. **The guard on the daemon's settings writes changed, and it was narrowed, not
   removed.** `alo-choosing`'s `the_daemon_reads_these_settings_and_never_writes_them`
   refused any mention of `Choosing` in `alo-agentd`. The published plan's task
   15 requires the daemon to answer through `Choosing::answered_by_a_paired_machine`,
   and its constraint says the daemon writes the person's settings *only because
   the person's own shell asked it to, through `alo-choosing`'s one way out*.
   Neither ADR 0016 nor any other ADR says the daemon never writes the file, so
   this changes no decision. The old test was a finding of an earlier task, and
   this plan has since overtaken it. The guarantee it existed for is that
   **nothing an agent can send writes a person's settings**. That is now held in
   two ways. First, the renamed test
   `the_daemon_writes_these_settings_only_where_the_persons_door_asks` insists
   that exactly one file (`choosing_to_answer.rs`) names the writer. It also
   insists that only `answering.rs`, the person's door, reaches that file, and
   that the file still exists. Second, the daemon's own test sends the request on
   the agent's door and checks that the file is not written. A second road to
   writing settings from anywhere in the daemon still fails the guard, just as
   before.
2. **The request names an identity and nothing else.** There is no address, for
   `pair`'s reason. There is **no model**, because which model answers on the
   other machine is that machine's person's choice (ADR 0008). Wire names:
   `choose-machine-to-answer` and `chosen-to-answer`. I avoided "answering
   machine", which means a telephone device in English.
3. **What the daemon does, in order, and why.** The identity is checked first,
   so a hostname never causes the settings file to be opened. Next comes
   somewhere to keep the file, then the file as it stands, then the pairing.
   `Choosing::at` refuses a settings file that is there but cannot be read, and
   that file is left byte for byte. Writing over it would lose whatever the
   person had typed. The lock over the pairings is held while the file is
   written, so a revocation cannot land between the check and the write.
4. **The settings file is the one `Questions` reads.** I added
   `Questions::where_the_settings_are`, which is `alo_choosing::where_it_is` over
   the session's own `XDG_CONFIG_HOME`/`HOME`. It is reached through
   `Holding::questions`. A local turn and the network's door both carry the
   session's `Questions`, so the request works whatever holds the machine. Only
   a test's `Holding::Nobody` has none, and it is refused in words.
5. **The list's field comes from the corridor's own check.** `may_answer_questions`
   comes from `WhoMayBeAsked for Shared`, the same function a question down the
   corridor asks. It is a separate field with `#[serde(default)]`, set by a
   builder, so `Paired::of`'s signature did not change. A list from an older
   daemon reads as `false`: *not offered* is the safe misreading.
6. **Nothing is recorded for a choice.** A settings change is not something that
   happened to the machine, and no other settings door writes a record entry. A
   refusal writes nothing either, and the refusal test checks that the record is
   empty. The departure a chosen machine causes is recorded at the question, as
   task 14 built.
7. **A refusal from `alo-choosing` goes to the person in its own words.** That
   includes `NotPairedToAnswer`, which names the machine and says *nothing in
   your settings has been changed*. Its two machine-side failures
   (`NotExpressible`, `NotKept`) are also printed to the service log in English,
   for whoever is fixing the machine.
8. **"Through the door" means the daemon's own request handlers.** The
   next-question test sends the choice through `what_a_person_said`, then puts
   the next turn's question through `what_an_agent_said`. The question goes
   down a real TCP corridor to a stub studio, which answers once. This is the
   level `pairing.rs`'s *through the door rather than the lock* tests use. I
   did not add a socket-level serving test: in that harness the corridor would
   also need reception's discovery and HTTP answered from a client thread, and
   that would test task 14's wiring rather than this task's.

## Acceptance criteria, and the test behind each

| Criterion | Test |
|---|---|
| A request on the person's door, answered through `Choosing::answered_by_a_paired_machine` against `TheNetwork`; the choice is what the next turn's question goes to, through the door | `alo-agentd` `choosing_to_answer::tests::a_machine_chosen_on_the_persons_door_answers_the_next_turns_question` |
| Refused in words when no pairing permits asking that machine's models, and a refused choice writes nothing (never paired, workspace only, run out, revoked; absent file stays absent, present file byte for byte) | `alo-agentd` `choosing_to_answer::tests::a_machine_no_pairing_lets_answer_is_refused_and_nothing_is_written` |
| Refused in words when what is named is not an identity | `alo-agentd` `choosing_to_answer::tests::something_that_is_not_an_identity_is_refused_before_the_settings_are_opened` |
| Refused on the agent's door in the words an approval gets | `alo-agentd` `choosing_to_answer::tests::an_agent_choosing_a_machine_to_answer_is_refused_and_nothing_is_written`; `alo-protocol` `person::tests::choosing_a_machine_to_answer_is_a_persons_and_refused_to_an_agent` |
| The pairings list says, for each, whether it permits asking its models | `alo-agentd` `pairing::tests::the_list_says_which_pairings_may_answer_questions` |
| The daemon writes settings only where the person's door asks (ADR 0016 constraint) | `alo-choosing` `no_agents_door_reaches_these_settings::the_daemon_writes_these_settings_only_where_the_persons_door_asks` |

Also added: `a_settings_file_that_does_not_hold_is_refused_and_left_alone`,
`a_session_with_nowhere_to_keep_settings_is_refused_in_words`, the wire tests
`asked::tests::choosing_a_machine_to_answer_names_its_identity_and_nothing_else`,
`to_a_person::tests::the_machine_chosen_to_answer_is_told_to_the_person_and_not_to_an_agent`
and `pairing::tests::a_pairing_says_whether_its_machine_may_answer_questions`.

## Verification (executed)

Platform: WSL2 Ubuntu on the development machine,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`, run from
`/mnt/c/dev/alo-os-claude`.

- `cargo fmt --all` then `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -- -D warnings` (workspace): exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-agentd -p alo-protocol`: exit 0.
- `cargo test -p alo-protocol -p alo-choosing -p alo-changing`: all pass
  (alo-protocol 108 unit + 21 integration; alo-choosing 135 unit + 58
  integration; alo-changing 16 unit + 14 integration).
- `cargo test -p alo-agentd`: all pass (300 unit, and the kernel-bounded
  integration tests 5 + 1 + 1, plus 4 + 12).
- Each evidence test run on its own with `--exact`: one passed each.
- The full workspace suite was **not** run here. The supervisor runs it.

No hardware acceptance applies: nothing here touches a device, and the change is
screenless by the plan's terms.

## Limitations

- The name a person gives a paired machine still has no home. The choice and
  the list name machines by their identity. Task 16 is written for that.
- A shell surface that offers the choice is `alo-shell`'s and outside this
  plan. What this task owes it is the request, the answer and the list's field.
- Clearing a paired choice, or choosing a local model or provider instead, is
  not on this door. Those are the settings panel's, through `alo-choosing`
  directly, as before.

## Proposed updates to shared documents (for the integration owner)

- **CHANGELOG.md:** "A person's shell can choose a machine they are paired with
  to answer their questions. The machine checks the choice against its pairings
  and writes it to their settings, and the next question goes there. The list
  of pairings says which machines can be chosen. An agent cannot make this
  choice."
- **docs/autonomy/STATE.md:** reference this report; task 15 of the local-network
  plan done, task 16 (*a paired machine is spoken of by the name its person gave
  it*) ready.
- **docs/autonomy/QUEUE.md / ROADMAP.md:** no box moves; the v0.5 local-network
  promise *a machine without a GPU discovers the one with it, and the agents just
  work* now has the person's door as well as the turn.
