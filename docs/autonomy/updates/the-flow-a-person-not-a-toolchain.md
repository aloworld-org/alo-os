# The flow: a person, not a toolchain

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *The flow: a person, not a toolchain*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 4)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2; `alo-saying`'s vocabulary tests in the Lima VM. No model was run.
**Egress:** none.
**Status:** done.

## What changed, for somebody outside this repository

Teaching your model from your own documents is now **five questions you already
know the answers to**, rather than a toolchain you have to learn first:

1. **Which folder should your model learn from?** — one you granted, picked in
   the picker. No path to type, and no *all my documents*.
2. **What should it get better at?** — in your own words.
3. **What it will learn from** — the files, listed, with what it will skip and
   why, before anything runs.
4. **Teach it from these documents** — one approval, because this changes the
   machine (ADR 0001).
5. **Keep what it learned?** — beside how long it took and what it used.

**No step names anything we rented**, and a test keeps it that way.

## The guard, and what to do if it stops you

`tests/the_flow_names_nothing_rented.rs` reads every sentence this crate
declares and fails on fourteen words: *LoRA*, *QLoRA*, *rank*, *learning rate*,
*epoch*, *checkpoint*, *gradient*, *optimiser*, *peft*, *transformers*,
*safetensors*, *GGUF*, *tensor*, *optimizer*.

**If it stops you, the fix is not a gentler name. It is that the value is not a
step.** The flow asks five things; everything else lives in
`docs/contracts/fine-tuning-values.md` and a file for people who want it. That
file is the one place in this repository where a person may meet those words,
because somebody who opens it came looking for them.

Two things it deliberately does not read. **Translators' notes**, which must be
free to say *this is the LoRA rank* so the sentence above them can avoid it — a
test asserts every word has one. And **`engine.rs`**, the one file allowed to
name the stack; naming it there is how nothing else has to.

## An agent may propose a fine-tune, and may never start one

`Flow::proposed_by("@the-agent")` is the only road from an agent, and it begins
at the same step a person's own road begins at: nothing is pre-answered and
nothing is pre-approved. `a_person_approved()` is the only way past *start*, and
`may_begin()` — the one question the thing that runs a fine-tune asks — is false
until a person has answered every step and approved.

Training on somebody's documents makes an artefact that carries their writing.
It is the most personal thing this machine does, so an agent asks for it in a
sentence a person approves, exactly as it asks to move a file.

**Who proposed it is not forgotten once it is approved**, which a test holds: a
fine-tune an agent suggested and a person approved is both of those things, and
the record should be able to say so.

## What the type refuses

- **A purpose nobody stated.** An empty answer at step two does not move the
  road on: a fine-tune nobody can say the purpose of is one nobody can judge
  afterwards, and *what should it get better at* is what the person will read
  months later beside the result.
- **An approval taken early.** `a_person_approved()` before *start* does
  nothing; there is no way to reach approval by asking for it.
- **A folder that was typed.** The step takes an `alo_picking::Picked`, which
  exists only by somebody standing in a folder and choosing it.

## What is not done here

- **Nothing draws.** The window is the shell's; this decides what may happen
  next so that two surfaces cannot disagree about it.
- **The verb an agent proposes through** belongs in `alo-capability`'s closed
  list, which this plan may not edit — the same finding task 1 recorded, now
  with the type its argument would build.
- **The values file is documented and not yet read.** `fine-tuning-values.md`
  states the five keys, their defaults and what happens to a value that makes no
  sense; reading it is the work of whichever task runs a fine-tune from the flow
  rather than from a script.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
