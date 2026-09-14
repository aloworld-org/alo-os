# A turn on a machine with no model asks the paired machine its person chose

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 14)
**Contributor:** Claude (Opus 5), in the `C:\dev\alo-os-claude` checkout
**Status:** ready for integration

## What changed, for a person

*A machine without a GPU discovers the one with it, and the agents just work*
was true of two doors and no turn. It is now true of a turn. A person can
choose a machine they are paired with to answer their questions. The choice
is refused unless a pairing lets that machine's models be asked, and it is
asked again at every question. When an agent on this machine asks something,
the question goes down the corridor. The egress indicator shows it, and the
record keeps it as having left, naming the machine. When the other machine
will not answer, the agent is told why in this machine's language: the pairing
does not include its models, its person uses a provider, or nothing there is
chosen. Nothing is ever asked somewhere else in its place, in either direction.

## What changed, in the code

| Where | What |
|---|---|
| `crates/alo-choosing/src/paired.rs` (new) | `AMachine` (the choice, by identity), `WhoMayBeAsked` (the question put to whoever holds the pairings), `NotPairedToAnswer`, `WHAT_THAT_MACHINE_CHOSE` |
| `crates/alo-choosing/src/chosen.rs` | `Picked::FromAPairedMachine(AMachine)`, `Picked::paired_machine` |
| `crates/alo-choosing/src/written.rs`, `writing.rs` | `[answers] machine = "<identity>"`, read and written; empty is `NotSet::Nameless` |
| `crates/alo-choosing/src/choosing.rs`, `unwritten.rs`, `words.rs` | `Choosing::answered_by_a_paired_machine`; `NotWritten::NotPairedToAnswer` and `choosing.change.not-paired-to-answer` |
| `crates/alo-answering/src/refused_there.rs` (new), `wrong.rs`, `failed.rs`, `words.rs` | `WentWrong::RefusedThere(RefusedThere)` with three words; possible only at a paired machine (`NotWhatFailed::NoMachineThere` elsewhere) |
| `crates/alo-asking/src/refused_on_the_wire.rs` (new), `openai.rs`, `lib.rs` | the wire words `not-permitted`, `answers-elsewhere`, `not-answered-here` spelled once; a question down the corridor reads them on their own status |
| `crates/alo-turn/src/answers.rs`, `asking.rs` | `Answers::PairedMachine(DownTheCorridor)`; one road with a provider's (`put_off_this_machine`) — resolved before the boundary, put inside it, `Entry::left` whether or not an answer came |
| `crates/alo-turn/src/down_the_corridor.rs` (new, tests), `testing.rs`, `next_request.rs` | the turn's tests for this road; the next-request table now says a paired machine is asked freely under a departure |
| `crates/alo-agentd/src/corridor.rs` (new) | `Corridor` (network, discovery, naming); `WhoMayBeAsked for Shared`; the bound → pairing → discovery → question order |
| `crates/alo-agentd/src/questions.rs`, `doing.rs`, `serving.rs`, `answering.rs`, `questioned.rs`, `words.rs`, `lib.rs` | the paired choice reaches a turn; `what_an_agent_said` takes `Option<&Corridor>`; the answering side uses the shared wire words and answers `answers-elsewhere` when its own person chose a paired machine; two new daemon sentences |
| `docs/contracts/person-settings.md` | `machine = "<identity>"`, additively |
| `docs/contracts/local-network-wire.md` | what the asking machine reads; `answers-elsewhere` covers a person who chose another paired machine |

## Decisions, and why

1. **The choice holds an identity as a string, and `alo-choosing` does not
   depend on `alo-nearby`.** My first version did depend on it.
   `crates/alo-choosing/tests/a_grade_travels_nowhere.rs` refuses that
   dependency by name: a settings store must have no road off the machine. So
   the pairings are asked through a one-method trait, `WhoMayBeAsked`. The
   daemon answers it from `TheNetwork`'s `Shared`, the same list that refuses a
   revoked machine's next verb. That makes *at choosing* and *at every question*
   one answer asked twice. The guarantee was not weakened to fit the design;
   the design was changed.
2. **Only `AMachine::permitted` makes the choice**, so no surface can write a
   paired choice without asking the pairings. A file read back is **not**
   checked against any pairing. An expiry passing overnight does not make a
   settings file wrong; the question is refused in words instead.
3. **An identity that is not an identity reads.** It is refused at the
   question as paired with nothing, which is true. That avoided a second copy
   of `MachineId`'s rule inside the settings crate.
4. **Additive; format 3 unchanged.** This follows `[[brought]]`'s precedent in
   the contract: an older alo OS refuses the whole file on the unknown key, so
   no release honours a different choice.
5. **The choice carries no model.** The answering machine uses its own
   person's model (ADR 0008). The question names `what-that-machine-chose`,
   which that machine checks and sets aside. `alo_asking::Answer` keeps the
   name it was asked with, as it always has.
6. **The other machine's refusal is a tenth `WentWrong`,
   `RefusedThere(…)`.** It is a *declining* rather than a *failing*, and it
   passes that list's bar of *a different thing to be told*. Three reasons,
   three sentences, rendered here. The asking side reads only the three words on
   their own statuses, never text the other machine wrote. Before this change a
   `403 not-permitted` from a paired machine read as a refused key. That arrived
   as `NotWhatFailed::NoKeyThere`, and the daemon told the person *nothing was
   asked* about a question that had in fact left. That is fixed.
7. **Order in the daemon:** the organisation's bound first, as for a provider,
   and written down as a refusal. Then the pairing at that moment. Then
   discovery by identity, the same look the person's door uses to pair; an
   address is never kept. Then the corridor, made under the lock so a revocation
   between the checks is refused rather than raced past. Then the question.
8. **The daemon's two new sentences have no gap.** `alo-agentd`'s words rule is
   that no sentence it says quotes anything, so they say *the machine on your
   network chosen to answer your questions* rather than naming it. The
   indicator and the record still name it by the person's name
   (`alo_corridor::Naming`), or by its identity until one is kept.
9. **The paired road's `alo-turn` tests live in `down_the_corridor.rs`** rather
   than in `asking.rs`, which is already the crate's longest file (law 4).

## Acceptance criteria, and the test behind each

| Criterion | Test |
|---|---|
| Choosing a paired machine, refused at choosing without a pairing permitting `MayAskIts::Models` | `alo-choosing lib choosing::tests::choosing_a_machine_no_pairing_lets_answer_writes_nothing` (and `…a_paired_machine_chosen_reads_back_as_that_machine`, `paired::tests::*`) |
| …and again at every question: revoked, or expired, between the two, refused in words | `alo-agentd lib corridor::tests::a_pairing_revoked_between_two_questions_refuses_the_second_in_words`, `…a_pairing_that_ran_out_since_choosing_refuses_the_question_in_words` |
| A turn's question through `to_a_paired_machine`, departure shown and kept as left naming the machine, beside a local answer that leaves nothing | `alo-turn lib down_the_corridor::tests::a_turns_question_to_the_paired_machine_leaves_naming_it_and_a_local_answer_leaves_nothing` |
| The other machine refuses — not permitted / answers elsewhere / nothing chosen — in its own word, rendered in this machine's language | `alo-turn lib down_the_corridor::tests::the_paired_machine_saying_not_permitted_reaches_the_agent_in_this_machines_language`, `…saying_it_answers_elsewhere…`, `…saying_nothing_is_chosen_there…` |
| An agent's next request down the corridor asked exactly as words, off the socket | `alo-turn lib down_the_corridor::tests::an_agents_next_request_down_the_corridor_is_asked_exactly_as_a_question_in_words` |
| Never a fallback, either direction | `alo-turn lib down_the_corridor::tests::a_paired_machine_is_never_a_fallback_for_a_local_model_or_a_provider_nor_they_for_it`, `alo-agentd lib corridor::tests::a_chosen_machine_not_on_the_network_is_refused_and_nothing_else_answers` |
| The organisation's bound asked before anything leaves | `alo-agentd lib corridor::tests::an_organisations_bound_refuses_the_paired_machine_before_the_network_is_asked` |

Supporting: `alo-answering lib wrong::tests::a_refusal_in_a_paired_machines_own_word_happens_only_at_a_paired_machine`,
`alo-asking lib refused_on_the_wire::tests::*`, `alo-choosing lib written::tests::a_paired_machine_is_chosen_by_its_identity`.

## Verification (executed)

| Command | Platform | Result |
|---|---|---|
| `cargo fmt --all` / `-- --check` | Windows | clean |
| `cargo clippy --all-targets -- -D warnings` | WSL Ubuntu (workspace, includes `alo-agentd`) | exit 0 |
| `cargo clippy --all-targets -p alo-answering -p alo-asking -p alo-choosing -p alo-turn -- -D warnings` | Windows | exit 0 |
| `cargo test -p alo-answering`, `-p alo-asking`, `-p alo-choosing`, `-p alo-turn`, `-p alo-saying` | Windows | all pass |
| `cargo test -p alo-agentd` | WSL Ubuntu | 293 unit + 5 + 1 + 1 + 4 + 12 integration, all pass |

**Not mine, found on the way:** on Windows, the workspace-wide `cargo clippy
--all-targets` stops at `crates/alo-files/src/walking_on.rs:261` (`unused import:
crate::named::Kind`). That lint only fires on Windows; the same workspace clippy
under WSL is clean. It is in another lane's crate, and this change does not touch
it.

**Not run:** the full workspace test suite (the supervisor's), and anything on
real hardware — two machines on a real network. The corridor is tested against
real sockets on one host, with discovery stood in by a `LookingFor`.

## Limitations

- **A shell has no request to choose a paired machine yet.** The daemon holds
  the pairings, and the person's door has no request that runs
  `Choosing::answered_by_a_paired_machine` against them. Written up as
  **task 15** in the plan.
- `alo-setting-up`'s `OnAMachineOnThisNetwork` answer still chooses nothing.
  Setup offering a paired machine needs task 15's door, and a pairing, which
  first-time setup does not have.
- Names are still `NoNameYet`, so the indicator and the record name the machine
  by its identity until the shell keeps names.

## Proposed updates to shared documents (for the integration owner)

- **CHANGELOG.md:** *A machine with no model of its own can now have its
  questions answered by a machine it is paired with. The person chooses which
  one. The choice is refused without a pairing that includes that machine's
  models, and so is every question once the pairing ends. The question shows on
  the egress indicator and stays in the record, and a refusal from the other
  machine is told in your own language.*
- **QUEUE.md / STATE.md:** local-network plan task 14 done; task 15 (the
  person's door chooses a paired machine) ready, depends on 12 and 14.
- **ROADMAP.md:** none.
