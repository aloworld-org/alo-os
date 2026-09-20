# The pin an entry states, and what the machine actually got

**Date:** 2026-09-20
**Workstream:** v0.01 — lane B, task 16
**Task:** 16, *The pin an entry states, and what the machine actually got.*
**Done.**
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory, macOS. **The measurement is of
the pinned runtime, Ollama 0.34.0, running here** — `crates/alo-models` is one of
the seventy-seven crates in this repository that are not gated to Linux, which is
why this task was reachable from this machine when the v0.5 work was not.
Nothing is ticked *on the machine*.
**Egress:** `git fetch`/`git push`; a few kilobytes of registry manifest from
`huggingface.co` for each of the two pinned entries; and **one 105 MB model**,
`hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M`, pulled by the runtime so
that the check could be measured against a real artefact. No weights were added
to the repository.

## What was wrong

Task 15 wrote two `sha256` figures into `data/catalogue.toml`, and **nothing
compared either of them with anything.** `Requantised` refuses a digest that is
not shaped like a digest; no code ever looked at what arrived. So the sentence
rule 6 sells — *the file we graded and the file a machine fetches are the same
file or the fetch fails* — was a promise about a field nobody read.

The case it exists for is a tag re-pointed at a different upload. Somebody asks
for the model they were promised, gets another, and every check in this
repository says yes.

## The question this task forks on, answered first

The task does not assume the check can be written. It says: find out against the
real runtime whether the digest it exposes for a model pulled from `hf.co/…` is
the `sha256` of that GGUF — and if it cannot be compared, **that** is the
deliverable, as a finding and an ADR rather than code.

So that was asked before anything was written.

**There is no structured field.** `/api/show` answers with `capabilities`,
`details`, `model_info`, `modelfile`, `modified_at`, `parameters`, `template`
and `tensors`. Not one states a digest. What carries it is the `FROM` line of
the modelfile the runtime generates, naming the blob on disk.

**And that blob's name is the GGUF's own `sha256`.** Measured on one real pull,
three ways that agree:

| Asked of | Answer |
|---|---|
| the registry manifest's `application/vnd.ollama.image.model` layer | `2e8040ce…68c2d` |
| the manifest the runtime wrote on this disk | `2e8040ce…68c2d` |
| `sha256sum` of the blob file itself | `2e8040ce…68c2d` |

**And it carries to the entries that actually matter**, which cost nothing to
check — a few kilobytes of manifest, no weights:

| Entry | Catalogue `sha256` | The manifest's model layer |
|---|---|---|
| `eurollm-9b-instruct` | `785a3b28…806b` | `785a3b28…806b` |
| `teuken-7b-instruct` | `03fd13da…630b` | `03fd13da…630b` |

So the answer is that the pin **is** checkable, and the deliverable is the check.

## What was built

`src/ollama.rs` asks the runtime which file it ended up with, after the pull and
before the artefact is named for the catalogue, and only for an entry that states
a `[model.requantised]` block.

Two refusals, and the second matters as much as the first:

- **`NotThePinnedFile { model, expected, arrived }`** — the machine holds a
  different file. Both digests are named, because a person who cannot act on a
  digest can still quote it to whoever published the file.
- **`PinNotChecked(model)`** — the runtime would not say which file it holds.
  **A check that could not be made has not passed.** This is the variant that
  fires the day a later release stops printing the line the reader reads, so the
  pin stops being checkable loudly rather than quietly.

The reader is deliberately strict: `FROM ` at the start of a line, a last path
segment beginning `sha256-`, then exactly sixty-four lowercase hexadecimal
characters — or nothing, which its caller turns into the refusal above. Reading a
path out of generated text is a thin place to stand, and the way to stand on it
safely is to refuse every shape you did not measure.

## Proving the failure, which is the whole job

A digest check is easy to write and easy to write wrongly, and **every wrong
version passes the happy case**: one that compares nothing, one that compares a
value with itself, one whose mismatch branch warns and carries on. All three are
green against a file that matches.

So the refusal is measured against the real runtime, with a real artefact whose
real digest is deliberately not the one pinned:

```
refused on 0.34.0: expected 0000…0000, arrived 2e8040ce…68c2d
```

The assertion that does the work is not that it failed — it is that **`arrived`
is the digest the machine really holds.** A check comparing a value with itself
would report `0000…0000` back and pass a weaker test. And the refusal is
asserted to have stopped: the model is not installed under its catalogue id
afterwards, and the stand-in confirms no third request was made, so a warning
wearing a refusal's type would fail.

`an_entry_with_no_pin_is_unaffected` holds the other half of the acceptance,
against the same runtime: an entry with no block is fetched exactly as it was.

The three real-runtime tests are `#[ignore]`d for the reason the existing ones
are, and they refuse a runtime of any other release rather than measuring it.
The reader's own tests — every shape that must read as *nothing* — are unit tests
in `src/ollama.rs` and run everywhere; the function stays private, because a
test is not a reason to widen a crate's public surface.

## One thing the check found on its way in

Adding it turned an existing unit test red: `only_an_entry_that_names_a_template_is_given_one_when_fetched`
fetches Teuken, which states a pin, against a stand-in server that does not serve
`/api/show`. It refused with `PinNotChecked`.

**That is the check working.** The stand-in would not say which file it held, so
the fetch refused rather than installing weights nobody compared with anything.
The test was taught the extra request — which makes it stronger, since it now
exercises the whole path — and the entry beside it, `mistral-7b-instruct`, was
untouched because it states no pin. The property *an entry with no pin is not
newly able to fail* demonstrated itself before it was asserted.

## And one flaw of my own, found by running them together

The three real-runtime tests passed one at a time and failed in parallel: they
shared one catalogue id, and a model is named in the runtime's store *by its
catalogue id*, so one test removed what another was asserting was installed.

It is recorded rather than quietly fixed because the shape is worth naming. The
tests were not flaky — they were wrong, in a way that only shows when they run
the way the suite actually runs them, and a single-threaded run would have
called them green for ever. Each test now has an id of its own, and they were
run three times in parallel to say so.

## What a pin can and cannot promise

Rule 6 in `data/catalogue.toml` now says where the promise is kept **and what it
does not cover**: the check is of what arrived, so a re-pointed tag is caught
**after** the download rather than before it. It is the download that is wasted,
never the answer.

Checking the registry manifest before pulling would refuse sooner, and it was
considered and not taken: a manifest is the registry's **promise**, one tag-move
away from what actually arrives, and checking it would be checking a proxy for
the thing — the same shape as a recipe that tests a file's executable bit and
calls the program working. It would also move the outbound request from the
runtime into this crate, which today reaches nothing but this machine.

**A known consequence, recorded rather than fixed:** after a refusal the
mismatched artefact stays in the runtime's store under its own registry name. It
is not offered — `alo-models` only maps catalogue ids — but it is disk spent.
Removing things from the runtime's store on a refusal is a decision with a size
to it, so it is written here rather than added quietly.

## Gates

Nine in the Lima VM. The three real-runtime tests do not run there — the runtime
serves on the Mac — so they are `#[ignore]`d and were run here by hand, against
0.34.0, with their output above.

## One crate outside this task's own, and why

`crates/alo-asking` maps every `RuntimeError` to what a person is told, and it
maps them **exhaustively with a reason each** rather than through a wildcard —
so adding two variants stopped the workspace compiling. That is the design
working: a new failure is meant to make somebody decide what it means to a
person, not be swallowed by a catch-all.

The two new ones belong to fetching and cannot arrive while a question is being
asked, so they join the download reasons already there under *nothing usable*,
by that file's own stated logic: a runtime answering a question with *this is
not the file the catalogue vouches for* has answered with something that is not
an answer. Two lines and a sentence of the doc comment, which is the minimum
that keeps the workspace building; nothing else in that crate was touched.

## Crates touched

`crates/alo-models`, and the two lines in `crates/alo-asking` above: `src/ollama.rs`, `src/runtime.rs`, `src/words.rs`,
`data/catalogue.toml`, and the new `tests/the_pin_an_entry_states.rs`. Plus
`docs/quirks.md` and this plan. Nothing in `crates/alo-shell`, no weights on the
image, no setup flow, and no grade moved.
