# A turn shows a model the words the product wrote

**Date:** 2026-09-15
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`,
task 19), written by the measuring lane (its task 18) under
[ADR 0037](../../decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md).
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner.
**Status:** ready for integration.

## What changed, in words a person can read

When an agent asks a model what to do next, the model is now shown words alo OS
wrote: how to answer, every action this machine actually offers described in
that action's own words, and the person's request last. Before this change the
model was shown whatever text the agent's program sent, so nothing about the
grades in the model catalogue applied to a real turn. A question a person asks
in words still reaches the model exactly as they wrote it.

## What was built

- `crates/alo-turn/src/next_request.rs` — `Held::shown(verbs, asked)`. For
  `Held::ToTheEnvelope` (an agent's next request) it returns
  `alo_instructing::shown_to_a_turn(verbs, request)`; for `Held::InWords` it
  returns the words untouched. A request that is only whitespace is handed back
  unchanged, so `alo_asking::Question::asked` still refuses it as nothing asked
  instead of putting bare instructions to a model. Module and method rustdoc
  say what `asked` now means (the request, not a prompt).
- `crates/alo-turn/src/asking.rs` — `Turning::putting` builds its `Question`
  from `held.shown(self.machine().verbs(), asked)`. The registry is the turn's
  own machine's, so the verbs the model is told of are the verbs `Verbs::call`
  will validate the answer against.
- `crates/alo-turn/Cargo.toml` — `alo-instructing` as a dependency (it carries
  `alo-capability` and `ring` only); `ring` as a dev-dependency so a test hashes
  the bytes that crossed the socket itself.
- `crates/alo-agentd` — no production change: the daemon already hands the
  agent's `question` to `Turning::asking_for_the_next_request`. Tests through
  the agent's door, and `alo-instructing` as a Linux dev-dependency for them.
- `docs/contracts/daemon-protocol.md` — the `as-the-next-request` example now
  carries a request, and a paragraph says what the model is shown and what
  becomes of a client's own instructions.
- `docs/quirks.md` — *A turn asks a model in English* now says it is a fact
  about the product since this change.

`alo-instructing` was not edited, and no copy of its text exists anywhere else.

## Decisions

1. **A client's own instructions are wrapped.** Whatever an agent sends under
   `as-the-next-request` is the request: it goes after `The request:`, beneath
   the product's instructions and verbs, and no field, flag or door replaces
   them. *Refused* was rejected because nothing can reliably tell instructions
   from a request in somebody's words, and a guess would refuse real requests.
   *Ignored* was rejected because it drops what the person asked for. Wrapping
   is the only one of the three that leaves exactly one prompt, the product's.
   A client that still sends a composed prompt is answered, not refused, and
   its prompt is shown as a request; the contract tells clients to send the
   request alone. No client in this repository composed one.
2. **Every place is shown the product's words, not only the pinned runtime.**
   The request a model answers with is carried out on *this* machine against
   *this* machine's verbs, so a provider, a service on this machine and a
   paired machine need the same list to answer usefully. Showing them the bare
   request would break every agent whose person chose anything but the pinned
   runtime. ADR 0032 decision 4 is about the schema, and it still holds: only
   the pinned runtime is held to the envelope, and the corridor, provider and
   service are sent the product's words exactly as they would be sent the same
   words in a question in words — tested byte for byte.
3. **Down the corridor, the asking machine composes.** The paired machine
   answering (`Machine::answering_for`, task 11) never wraps a question again:
   the verbs belong to the machine where the request will run, and wrapping
   twice would show a model two sets of instructions. Tested.
4. **Composition happens in the turn, not in the daemon.** `Turning` holds the
   machine's registry, and `putting` is the one road every question takes, so
   there is one place the text is built and no caller can skip it.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| An agent's next request is put to a model in `shown_to_a_turn`'s text from the machine's own registry; read off the runtime's socket: the instructions' digest, every declared verb, the person's words, in that order | `alo-turn` `next_request::tests::an_agents_next_request_is_shown_the_products_words_read_off_the_runtimes_socket` (SHA-256 computed from the bytes on the socket and found through `Instructions::of_digest`); `alo-agentd` `doing::tests::an_agents_next_request_is_shown_the_products_words_through_the_door` (through the agent's door) |
| A question a person asks is untouched | `alo-turn` `next_request::tests::a_persons_question_in_words_reaches_the_model_as_they_wrote_it` and `…a_question_in_words_to_the_pinned_runtime_is_never_given_the_envelope`; `alo-agentd` `doing::tests::a_question_in_words_reaches_the_model_as_it_was_written_through_the_door` |
| A question a paired machine asks is untouched | `alo-turn` `answering_for::tests::a_question_a_paired_machine_asks_reaches_this_model_untouched` |
| What a client's own instructions now do (wrapped) | `alo-turn` `next_request::tests::instructions_an_agent_wrote_of_its_own_are_the_request_beneath_the_products` |
| Other places: shown the same words, asked as before | `alo-turn` `next_request::tests::a_provider_is_shown_the_products_words_for_the_next_request_and_asked_as_before`, `…a_service_on_this_machine_is_shown_the_products_words_and_asked_as_before`, `down_the_corridor::tests::an_agents_next_request_down_the_corridor_is_shown_this_machines_words_and_asked_as_before` |
| Refusal: an empty request asks nothing | `alo-turn` `next_request::tests::nothing_is_asked_for_a_next_request_that_says_nothing` (unchanged, still passes) |

The record entry is unchanged in shape
(`next_request::tests::how_the_model_was_asked_is_not_a_thing_the_record_keeps`).

## Verification

Executed on WSL2 Ubuntu (kernel 6.18), `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` (workspace) — clean.
- `cargo test -p alo-turn` — 109 unit, 25 integration, 2 doc tests pass.
- `cargo test -p alo-agentd` — 346 unit and all integration tests pass.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-turn --no-deps` — clean.
- The dependency guards that name `alo-turn` (`alo-clipboard`,
  `alo-changing`, `alo-choosing`, `alo-setting-up`) — pass.
- Each evidence test above run alone with `--exact` — pass.

Not run: the full workspace suite (the supervisor runs it). No physical
hardware is involved: this change is text put in a request body.

## Limitations

- The words are English (`docs/quirks.md`); the person's request is in their
  own language, inside English instructions.
- The catalogue still reads the grade earned under `AsFirstWritten`.
  ADR 0037 decision 4 — regrading under `SHOWN_TO_A_TURN`'s digest — is the
  measuring lane's task 19, which this unblocks.
- A client that kept sending composed prompts is now shown two layers of
  instructions (ours, then its own as the request). That is the wrap decision's
  cost and why the contract tells clients to send the request alone.

## Proposed shared-document updates

- **CHANGELOG.md:** "An agent's model is now shown alo OS's own instructions
  and the machine's own list of actions, with the person's request last; a
  question asked in words is still sent exactly as written."
- **QUEUE.md / STATE.md:** local-network plan task 19 done; the measuring
  lane's plan task 19 (regrade under the turn's instructions) unblocked.
- **ROADMAP.md:** no change.
