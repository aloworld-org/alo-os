# A turn asks the local model in the envelope

**Date:** 2026-09-14
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 13)
**Contributor:** Claude Code, Windows checkout `C:\dev\alo-os-claude`, gates in WSL Ubuntu
**Status:** ready for integration.

## What changed, for somebody outside this repository

- **An agent on alo OS can now ask a model on this machine for its next step
  in the way that works.** When an agent asks the model runtime alo OS ships
  what to ask the machine for next, the runtime is told to answer in the
  protocol's shape — the version and exactly one of *read*, *propose* or *ask*.
  On the Mac that took Qwen 2.5 7B from 40% to 88.75% of requests alo OS can act
  on (ADR 0032). Until today only the measuring tool asked that way.
- **A question in words is never given that shape.** *May the tenant sublet?*
  is still answered in prose.
- **A provider, a service somebody runs on this machine, and a paired machine
  are asked exactly as before.** ADR 0032 measured the pinned runtime and
  nothing else.
- **Nothing new is let through.** What the model says comes back to the agent
  as words and becomes a request only when the agent sends it as one, where it
  is read and validated like every other request. The record of the question
  is the same entry it always was.
- **For whoever builds agents:** an `ask` may carry
  `"answered":"as-the-next-request"`. Absent means words, so every existing
  client is unchanged.

## Was the door there?

Yes. The plan's constraint was that this waits if the Mac lane's task 14 had
not landed. It had: `Asking::to_this_machine_in_the_envelope` in
`crates/alo-asking/src/in_the_envelope.rs` (commit `4666381`), taking the pinned
runtime as `alo_models::Ollama`. Nothing here builds a second ask.

## What was missing, and the decisions it forced

**1. No turn asked for "an agent's next request" — the agent says so on the
wire.** Agents are clients of `alo-agentd`. They reach a model only through the
`ask` request, which until now carried only the question. So the daemon could
not tell *may the tenant sublet?* from *what should I ask for next?*, and only
the agent that wrote the question knows. Options:

- guess from the question's text — a heuristic deciding a protocol shape;
- a fourth request — ADR 0001 reserves that for a change in what an agent can
  *express*, and this is not one;
- **an optional field on `ask` saying what the answer should be** — chosen.

The field is `answered`, with two values, `in-words` (the default, and left off
when written) and `as-the-next-request`. It names no place (ADR 0008 is
untouched: `a_question_cannot_name_where_it_is_answered` still passes) and no
schema. Any other value refuses the whole request, as undeclared fields always
have. `crates/alo-protocol/src/answered.rs`; contract updated in
`docs/contracts/daemon-protocol.md`.

**2. One road in the turn, not two.** `Turning::asking` became a thin call to a
crate-private `Turning::putting(…, Held)`. The new
`Turning::asking_for_the_next_request` (`crates/alo-turn/src/next_request.rs`)
is the same call with `Held::ToTheEnvelope`. `held` is read in exactly one match
arm: the pinned runtime's. So refusals, the indicator, bounding and the record
cannot differ between the two, because no second copy exists. `Held` is
`pub(crate)`: a caller picks by which method it calls, so no value can put a
person's question in the envelope.

**3. The turn has to know the runtime is the pinned one.** `Answers::Runtime`
holds a `&dyn ModelRuntime`, and ADR 0032 decision 4 (and the door's signature)
wants the pinned runtime by its type. Added `Answers::ThePinnedRuntime(&Ollama)`.
`Answers::Runtime` stays for a runtime known only by its trait, and is never
held to the envelope. Only tests construct that kind now.

**4. `alo_models::found_on_this_machine` returns `Option<Ollama>` instead of
`Option<impl ModelRuntime>`.** Three doc comments argued that the opaque type
held ADR 0019 ("a caller who could name it could point it somewhere") by the
compiler. It never did, because `Ollama::at` has always been public. ADR 0019
itself says nothing about types. The comments now say what really holds it, and
a new test makes that true: `alo-agentd`'s
`the_runtime::tests::no_runtime_is_made_anywhere_this_daemon_ships` reads every
shipped source file of the daemon and fails if one makes an `Ollama`. The daemon
holds what it found as `alo_agentd::the_runtime::TheRuntime`:
`Pinned(Ollama)`, and a `cfg(test)`-only stand-in, following the existing
`Questions::already_found` seam.

**5. Which crate owns the paired-machine test.** In this repository no turn
asks a paired machine: `Answers` has no variant for one, and a permission naming
one is refused as `Miswired::BelongsDownTheCorridor`. That refusal is what
"asked exactly as before" means today, so `alo-turn` tests that the refusal is
identical on both doors. The paired-machine question that *does* reach a
machine — one arriving at this machine's pinned runtime from another
(`crate::questioned`) — gets a test reading the runtime's socket and finding no
envelope. This gap is written up as the plan's new task 14.

## Acceptance criteria and their tests

| Criterion | Test |
|---|---|
| The pinned runtime is asked for an agent's next request with the envelope's schema, read off a socket, nothing about the call's inside | `alo-turn` `next_request::tests::an_agents_next_request_is_asked_of_the_pinned_runtime_held_to_the_envelope`; end to end through the daemon's wire, `alo-agentd` `doing::tests::an_agents_next_request_reaches_the_pinned_runtime_held_to_the_envelope` |
| A question in words to the same runtime is never given the schema | `alo-turn` `next_request::tests::a_question_in_words_to_the_pinned_runtime_is_never_given_the_envelope`; `alo-agentd` `doing::tests::a_question_in_words_reaches_the_pinned_runtime_with_no_envelope` |
| A hosted provider is asked exactly as before | `alo-turn` `next_request::tests::a_provider_is_asked_for_the_next_request_exactly_as_before` (the two request bodies are byte-identical, both leave under a departure) |
| A paired machine is asked exactly as before | `alo-turn` `next_request::tests::a_paired_machine_is_refused_for_the_next_request_exactly_as_before`; `alo-agentd` `questioned::tests::a_question_from_a_paired_machine_is_never_held_to_the_envelope` |
| The record entry is unchanged in shape | `alo-turn` `next_request::tests::how_the_model_was_asked_is_not_a_thing_the_record_keeps` (the two roads' entries are equal, and equal to `Entry::answered_here`) |

Refusal paths beside them:

- `alo-turn`:
  - `a_permission_for_a_provider_asks_the_pinned_runtime_nothing`;
  - `a_pinned_runtime_that_does_not_answer_the_next_request_leaves_nothing`;
  - `nothing_is_asked_for_a_next_request_that_says_nothing`;
  - `a_next_request_whose_answer_could_not_be_written_down_closes_the_turn`
    (and the next one is `TurnClosed`);
  - `a_service_on_this_machine_is_asked_for_the_next_request_exactly_as_before`.
- `alo-protocol`:
  - `asked::tests::what_a_question_wants_back_is_one_of_two_and_nothing_else`;
  - `asked::tests::a_question_says_whether_it_wants_words_or_the_next_request`;
  - the written-and-read-back round trip in `agent.rs`.
- `alo-agentd`:
  - `doing::tests::a_question_wanting_anything_but_words_or_its_next_request_is_refused`;
  - `the_runtime::tests::no_runtime_is_made_anywhere_this_daemon_ships`.

## Verification

Run in WSL Ubuntu from `/mnt/c/dev/alo-os-claude` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`, in the
foreground:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-protocol -p alo-models -p alo-turn -p alo-agentd --all-targets -- -D warnings` — clean, no warnings.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-protocol -p alo-models -p alo-turn -p alo-agentd` — clean.
- `cargo test -p alo-protocol` — all pass (lib 104).
- `cargo test -p alo-models` — all pass (lib 210).
- `cargo test -p alo-turn` — all pass (lib 99, doctests included).
- `cargo test -p alo-agentd` — all pass (lib 289, and the kernel-bounded
  integration targets: 5, 1, 1, 4, 12), 1 min 47 s.
- The whole-workspace suite was not run here. The supervisor runs it.

**Not measured:** a real Ollama answering through the daemon. The Mac lane
measured the door itself (71 of 80). What this task adds is wiring, and it is
tested against a socket that records the exact request.

## Limitations

- **The agent has to ask.** An agent that never sends
  `"answered":"as-the-next-request"` is asked freely, exactly as before. The
  agents live in `alo-workplace`, which has to send the field.
- **No turn reaches a paired machine yet.** See decision 5 and the new plan
  task 14.

## Proposed updates for the integration owner

- **CHANGELOG:** *An agent asking a model on this machine for its next request
  now gets an answer held to the protocol's shape, which makes local models far
  more likely to produce a request alo OS can act on. Questions in words, and
  every provider or paired machine, are asked as before.*
- **QUEUE / STATE:** the local-network plan's task 13 is done. The models plan's
  task 15 (*the recommendation reads the grade for the way turns ask*) is no
  longer blocked on lane A. Task 14 of the local-network plan is new and ready.
- **ROADMAP:** none.
