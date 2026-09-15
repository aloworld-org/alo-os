# The catalogue reads the grade a turn earns, and a machine is given an agent

**Date:** 2026-09-15.
**Workstream:** the models, measured — `docs/autonomy/v0-5-the-models-measured-plan.md`, task 19, **the last task in the plan**.
**Contributor:** the Mac lane.
**Machine:** gates in the Linux VM (Ubuntu, kernel 7.0.0-31-generic, aarch64, as
root). **No model was loaded, fetched or measured for this task** — it is code
against grades that already exist.
**Status:** ready for integration. The plan is finished.

## What a person would notice

**A machine that can hold one is now given a local agent.** Until today
`Catalogue::agent_for_cpu` answered every machine with the same sentence — *the
models that run on this machine do not produce a workable instruction often
enough to be given the agent* — because no grade in the catalogue was about the
way the machine actually asks. A turn now shows a model the words this operating
system wrote, the catalogue reads the grade earned under those words, and two
entries clear the bar:

| Entry | The grade that decides | Earned |
|---|---|---|
| `qwen2.5-7b-instruct` | `reliably`, 80 of 80 | task 16 |
| `qwen3-8b` | `reliably`, 20 of 20 | task 20 |

A 16 GB machine is given `qwen3-8b` — `agent_for_cpu`'s existing ordering,
comfortable before workable and then larger, choosing between the two. An 8 GB
machine still reads the refusal, and now for a reason about *its memory*: both
entries state `min_ram_gb = 10`.

## What changed

**`crates/alo-models/src/graded_for_turns.rs`** is the rule, rewritten. The grade
that decides is the one earned three ways at once:

- **under the words a turn shows** — `THE_WORDS_A_TURN_SHOWS`, the SHA-256 of
  `alo_instructing::Instructions::SHOWN_TO_A_TURN`;
- **held to the envelope and nothing further** — `held_to` is `None`, which is
  what tells these apart from the two grades task 17 earned against a grammar
  through `llama.cpp` and ADR 0035 rejected;
- **through the runtime this product pins** — matched by name, `Ollama`.

An entry with no such grade **has no grade for the turn**: `grade_for_the_turn`
answers `(NotMeasured, NotTheWayATurnAsks)` and `can_be_the_agent` is false,
rather than a grade earned another way being read for it (ADR 0037, decision 4).

**The runtime's *version* is deliberately not matched**, and the file says why:
matching it would mean that upgrading Ollama silently takes the agent away from
every machine at once, which is a decision for whoever upgrades it — with a
re-measurement beside it — not a side effect of a version string moving.

**`AskedTheWay`** is now two variants, `AsATurnAsks` and `NotTheWayATurnAsks`,
with two sentences in `alo-models`' vocabulary (`models.graded.as-a-turn-asks`
and `models.graded.another-way`, both with translator notes). The old pair —
*in the envelope* and *freely* — described a distinction that no longer decides
anything, and leaving them would have offered a person a grade about a different
question.

**The digest is written out in `alo-models` rather than computed**, so the
catalogue can say which grade decides without depending on the verb registry;
`alo-driving`'s `every_grade_names_instructions_this_crate_has.rs` holds the
constant to `Instructions::SHOWN_TO_A_TURN.digest()`, so changing the text fails
a test instead of leaving the catalogue deciding by a digest nothing earns.

**No grade was rewritten, removed or added.** Every entry keeps its free grade,
its enveloped grade under the first instructions, and everything written beside
them; the catalogue's own comments now say which of them decides and which is
history.

**`docs/decisions/0037-…`** gains a *Since it was accepted* section recording
that decision 3's condition was met the same day, and what decision 4 then did.

## Tests

- **The rule, three ways**: a measurement is the turn's only when the
  instructions, what held the answer *and* the runtime all agree — each of the
  three failing cases is a real grade in the shipped catalogue.
- **Thirteen shipped entries are judged by the grade a turn earns**, and every
  entry that is not has no grade for the turn and cannot be the agent.
- **A 16 GB machine is given `qwen3-8b`**, in `alo-choosing` and in
  `alo-driving`'s end-to-end walk; **an 8 GB machine reads the refusal for a
  measurement that was made**, five entries fit and all five were measured.
- **The sentence a person reads** is checked in the vocabulary the whole machine
  loads, and neither line reaches a person as a key.
- **Every other grade is still there**, checked on the shipped catalogue and on
  `qwen2.5-7b-instruct` entry by entry.

Four tests written for the old world were rewritten rather than deleted, and each
says in its own words what changed: the refusal tests now ask a machine with no
room, and `alo-telling`'s table keeps the refusal as the first situation a person
can meet while a new test says a machine with room meets a model instead.

## Gates

All nine as root in the Lima VM, building in `/root/alo-builds/alo-os-main`:
formatting clean, `cargo clippy --workspace --all-targets -- -D warnings` clean,
rustdoc with warnings denied clean, the supervisor's three clean, both BPF gates
clean, and `cargo test --workspace --no-fail-fast` **5,210 passed, 0 failed, 31
ignored** — and **5,237 passed, 0 failed, 31 ignored** after rebasing onto the
`main` that moved while this was gated, which is the tree that was pushed.

## What this does not claim

- **Nothing about a certified machine.** No *On the machine* box moved and
  `ROADMAP.md` is untouched. What a certified workstation is given depends on its
  own memory and its own measurement.
- **No model was run for this task.** The grades read here were earned in tasks
  16 and 20 and are unchanged.
- **`qwen3-8b`'s grade is twenty attempts** on an 8 GB Mac with about a fifth of
  the weights off the graphics processor. `qwen2.5-7b-instruct`'s eighty remain
  the larger sample, and the ordering — not the evidence — is what puts the 8B
  first on a 16 GB machine.
- **A person is still not overruled.** The catalogue recommends; choosing another
  model, or weights of their own, is unchanged and ungated.

## Proposed shared-document updates

- **`CHANGELOG.md`:** *A machine with room is now given a local agent. The
  catalogue reads the grade a model earned when it was asked the way this machine
  asks — held to the shape of the request, and shown the words the system itself
  writes — and two models clear that bar. A machine without the memory for either
  is told so, and told what else it can do.*
- **`ROADMAP.md`:** nothing.
- **Plan:** task 19 done; with task 9 not pursued by the owner's decision,
  **`v0-5-the-models-measured-plan.md` is finished**.
