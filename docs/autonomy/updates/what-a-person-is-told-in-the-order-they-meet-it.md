# What a person is told, in the order they meet it

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *What a person is told, measured against what a person can read*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 5)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** walked in Ubuntu 24.04 aarch64 under Lima on an **Apple M3 with 8 GB
unified memory**; the walk itself asks no model and the machine size it asks
about is 16 GB, the class the refusal tests use. The grades behind it were
measured on the same Apple M3 under Ollama 0.34.0.
**Status:** done.

## What changed, for somebody outside this repository

Every sentence a person reads about which model they have — the recommendation,
a model that sometimes or rarely drives the agent's verbs, one too large for their
machine, and a file they brought — was read in the order they meet it, on a
catalogue that is now actually measured. Two sentences were fixed: they said a
measurement was made **on this machine** when it may have been made on another,
and one said the machine would not *raise* a model's size again, which can read
as making it bigger. The whole sequence is now held by a test, so it cannot
change without the table below changing with it.

## The table

A fresh settings file, a machine with 16 GB, the catalogue as shipped (no entry
clears the bar). Every line is exactly what the machine's code said, in the
vocabulary `alo-saying` collects.

| Situation | Word | What the person reads |
|---|---|---|
| The model the catalogue recommends | `models.agent.none-clears-the-bar` | the models that run on this machine do not produce a workable instruction often enough to be given the agent |
| | `models.agent.weights-you-already-have` | you can point alo OS at weights you already have on this machine, and it will run them — this catalogue is what alo OS offers, not everything it can run |
| | `models.agent.elsewhere` | you can use a model on a machine you have paired with on your network, or a provider you add — whichever you prefer, and alo OS will not choose for you |
| A model that runs, measured *sometimes* | `models.brought.fits` | these weights fit in the memory this machine has |
| | `models.brought.licence-is-yours` | these weights are yours, and so are their terms — alo OS states the licence of what it offers and has not read the licence of a model it did not offer you |
| | `models.brought.measured-not-the-agent` | these weights have been measured driving the agent's verbs, not often enough to be given agent turns — they still answer your questions |
| A model that runs, measured *rarely* | `models.brought.fits` | these weights fit in the memory this machine has |
| | `models.brought.licence-is-yours` | these weights are yours, and so are their terms — alo OS states the licence of what it offers and has not read the licence of a model it did not offer you |
| | `models.brought.measured-not-the-agent` | these weights have been measured driving the agent's verbs, not often enough to be given agent turns — they still answer your questions |
| A model too large for this machine | `models.brought.larger-than-memory` | these weights are larger than the memory this machine has — alo OS will still run them, and this machine will be slow |
| | `telling.runs-them-anyway` | That is said once: alo OS will run these weights whenever you choose them, and will not mention their size again by itself |
| A file they brought | `models.brought.fits` | these weights fit in the memory this machine has |
| | `models.brought.licence-is-yours` | these weights are yours, and so are their terms — alo OS states the licence of what it offers and has not read the licence of a model it did not offer you |
| | `models.brought.not-measured` | nobody has measured whether these weights can drive the agent's verbs, and alo OS has not guessed — they answer your questions now, and get an agent turn once a measurement says they can |

Held by `crates/alo-telling/tests/what_a_person_is_told_in_the_order_they_meet_it.rs`:
`what_a_person_reads_is_the_table` (every line, in order, as the table),
`every_sentence_in_the_table_is_the_machines_and_has_a_note`, and
`no_sentence_claims_a_measurement_made_on_the_readers_machine` with its twin
showing the check catches the old wording.

## What was read, and what changed

- **Two sentences claimed a measurement on this machine.**
  `models.brought.measured-the-agent` and `-not-the-agent` (added in task 4) said
  *measured … on this machine*. A grade travels with the weights and was earned
  on the machine written beside it; a person reading on another machine would
  be told something false. Both now say no machine, and the translator's note
  says why.
- **"will not raise their size again" became "will not mention their size
  again".** *Raise* reads, to somebody who has just been told their weights are
  too large, as *make larger*. The promise is that the machine will not bring it
  up.
- **Read and kept as they are:**
  - **The capital letter on *That is said once*.** It is the closing line of the
    warning, the way *You can carry on* closes what `alo-telling` says when a
    question fails, while the lines between are `alo-models`' lowercase
    fragments. That is a convention of the two crates, not a slip.
  - **The first line of the recommendation**, *the models that run on this
    machine do not produce a workable instruction often enough*. It is about the
    models, which is where a grade belongs, and claims no measurement on the
    reader's machine.
  - **The same three lines for *sometimes* and *rarely*.** The grade is a value
    shown beside the sentence, and either way the weights are not the agent and
    still answer. A person does not need two sentences for one outcome.

**Nothing was re-graded.**

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root: formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass. The workspace's tests, run whole with
`--no-fail-fast`: **4,396 passed, 1 failed, 25 ignored** — the one is
`alo-bounding`'s `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`,
the aarch64 failure recorded in the first task's report and `docs/quirks.md`.
The four tests of the walk pass. The publish script re-runs all nine on the tree
combined with `main` and pushes only if the result is this one.

## What this does not claim

The table is the English source. No translation exists yet, and the notes are
what a translator will be given. Nothing here moves an *On the machine* box.
