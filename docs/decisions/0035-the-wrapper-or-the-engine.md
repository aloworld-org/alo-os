# ADR 0035 — The wrapper or the engine: decided by a measurement, and never by writing our own

**Status:** proposed — accepted or rejected by task 17 of
`docs/autonomy/v0-5-the-models-measured-plan.md`, on a number
**Date:** 2026-09-14
**Context:** [ADR 0006](0006-the-pinned-model-runtime.md) (Ollama is the
pinned runtime, behind our own trait), [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(engines are rented and configured, never patched or rewritten),
[ADR 0032](0032-a-local-model-is-held-to-the-envelope-not-the-call.md) (a
local model is held to the envelope, not the call — because the pinned
runtime alphabetises a schema's keys),
[ADR 0034](0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)
(the instructions show every door — accepted hours after this was drafted, and
it changes what this decision is *for*: see below),
`crates/alo-models/src/ollama.rs` (the
one file allowed to name the runtime), the Mac lane's reports of 2026-09-13
and 14, and the owner's question of 2026-09-14: *would it have been better to
create our own Ollama, that we can improve and that works well in our OS?*

## The question in one line

**Every friction the measuring lane found this week was in the wrapper and
not in the engine. Do we keep the wrapper, replace it with the engine's own
server, or write our own — and on what evidence?**

## What was true before this decision

Ollama is a program that wraps an engine, `llama.cpp`, where the inference
happens, and adds a model store, an HTTP API and a downloader. ADR 0006 chose
it in September for the API `alo-workplace` already spoke, and put it behind
`ModelRuntime` so that it is one file. In two days of real measurement the
Mac lane found, in the wrapper's layer and nowhere else:

- the `modelfile` field a brought file was handed over with is gone from the
  pinned version's create API, and nothing had noticed because no test had
  asked a real runtime (`c6a65ba`);
- it reaches `ollama.com` in its first eight milliseconds and every four
  minutes after, so the image has to firewall its own model runtime
  (`alo-os-delivery-state`, 2026-09-11);
- it keeps a brought file's bytes twice on a read-only `/usr`;
- it ignored Teuken's chat template until asked through the publisher's own
  (`1828f1f`);
- **its structured-output mode orders a schema's keys alphabetically**, which
  is why ADR 0032 could hold a model to the envelope only and never to the
  whole call — the protocol's argument is `named` then `is`. That single
  limit is most of the five attempts in forty between the best local model
  and the bar.

## What changed between this being drafted and this being decided

Hours after it was written, ADR 0034 cleared the bar without touching the
runtime at all: Qwen 2.5 7B reached **80 in 80** under instructions that show
an example through each door, where 71 in 80 had been the best the old
instructions earned. So the last bullet above — *that single limit is most of
the five attempts in forty between the best local model and the bar* — **is no
longer true**, and this decision is not a rescue. It was drafted as one, and
saying so is cheaper than quietly re-arguing it.

What survives is narrower and still worth a measurement. The wrapper's other
four frictions are unchanged and every one of them is in the image. Holding a
model to the **whole call** rather than the envelope is still something only
the engine's server can do — now a question about margin, and about what the
*next* model needs, rather than about whether any local model can be the agent.

So the trial proceeds at lower priority than it had this morning, and the bar
for accepting it rises: what ships today is the wrapper, and replacing it
costs the image, the unit, the weights' store layout and `alo-image`'s checks.

## The decision

1. **Nobody writes a runtime.** The owner decided it on 2026-09-14 and ADR
   0011 already said why: the engine has thousands of contributors and moves
   weekly, a rewrite would spend months reaching where the engine was last
   year, and would then be a race lost permanently. *Engines are configured,
   never patched* extends to *never rewritten*. Rejected outright, and not
   revisited without new facts.
2. **The wrapper is on trial, and the engine's own server is the other
   candidate.** `llama.cpp` ships `llama-server`: the same OpenAI-shaped API
   `openai.rs` already speaks, no call home, no second copy of the weights,
   the publisher's chat template read from the file, and **grammar-constrained
   output (GBNF)** that holds a model to the *entire* call in the protocol's
   own key order — the thing the wrapper cannot do.
3. **The trial is one measurement, and it decides.** Task 17 puts the same ten
   exercises, the same bar and the same second-round rule to the same weights
   through `llama-server` with a grammar for the whole call, on the same
   machine, and writes the grade beside the wrapper's — under ADR 0034's
   instructions, since those are what a grade means now. It is **accepted**
   only if the engine's server is *better*, not equal: a model that clears the
   bar there and cannot through the wrapper, or a materially wider margin on
   the one that already does, or the whole call held where only the envelope
   could be. Then the pinned runtime becomes the engine's server, `ollama.rs`
   is replaced by one file naming it, and ADR 0006 is superseded. Otherwise it
   is **rejected**, the wrapper stays, and the four remaining frictions are
   kept as quirks rather than reasons.
4. **Whichever wins stays behind the trait.** `ModelRuntime` is the boundary
   that made this a one-file question; nothing outside `alo-models` learns
   which runtime answered, and no sentence a person reads names either
   (`docs/features.md`: *a person never learns the name of anything we
   rented*).

## What it costs

- **A day of the measuring lane**, on a machine that can hold the model.
- **If accepted, the image changes**: the pinned runtime, its unit, the
  weights' store layout, and `docs/booting.md`'s facts — all of `alo-image`'s
  checks move with it, and the installer plan's task 1 pins the new digest.
  Nothing a person sees changes.
- **If rejected, nothing changes** and the question is closed with a number
  beside it, which is more than it had.

## Rejected

- **Our own runtime.** Above. The owner's decision, and ADR 0011's.
- **Deciding on the frictions alone.** Five annoyances are an argument, not a
  measurement; the product's bar is *a model that drives the verbs*, and the
  runtime that gets one there wins, whichever it is.
- **Patching the wrapper's key order.** A source patch to an engine requires
  an ADR of its own and a fork to maintain — the thing rule 1 exists to avoid.
