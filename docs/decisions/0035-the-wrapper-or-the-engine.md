# ADR 0035 — The wrapper or the engine: decided by a measurement, and never by writing our own

**Status:** **rejected**, 2026-09-14, by task 17 of
`docs/autonomy/v0-5-the-models-measured-plan.md`, on the numbers in *the
measurement that decided it* below. The wrapper stays. Decision 1 — nobody
writes a runtime — stands, and was never on trial here.
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

## The measurement that decided it

Task 17, 2026-09-14, on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2.
The same weights throughout — `qwen2.5:7b-instruct-q4_K_M`, the bytes already on
that disk, digest `sha256-2bada8a7…`, never re-fetched. `llama.cpp` 0.4.0 (build
10809, commit 5266f24da) serving them through `llama-server`, with every token
held to a grammar for the whole call in the protocol's key order, written from
the verb registry (`sha256 efa8b951…`). The same ten exercises, the same bar, the
same door scoring, eight rounds each:

| Instructions | The wrapper | The engine, whole call held |
|---|---|---|
| One example per door (ADR 0034) | 80 of 80 | 80 of 80 |
| As first written | 71 of 80 | **65 of 80** |

**The three questions this was to be accepted on:**

1. *A model that clears the bar there and cannot through the wrapper?* **No.**
   The one model that clears it clears it through both, and the two files on
   that machine are the only ones this could be asked of without fetching more.
2. *A materially wider margin on the one that already does?* **No — a narrower
   one.** Equal where both are at the ceiling, and six attempts worse where
   there was room to differ.
3. *The whole call held where only the envelope could be?* **Yes, and it bought
   nothing.** The grammar holds `format`, the door, the verb's spelling, its
   argument names and the protocol's `named`-then-`is` order. The wrapper cannot
   do this, the engine does, and the grade did not improve.

**Why holding more of the call made it worse.** A grammar that forbids a change
through the read door does not teach a model which door a change takes: it makes
the wrong door unreachable, and the model's pull toward `read` comes out as the
wrong **verb** instead. All fifteen failures were that — `read_file` for a
rename five times, for a close five times, `find_in_folder` for a close three
times. A wrong door is a call alo OS refuses at the door; a wrong verb is a
well-formed call it would act on, caught here only because an exercise names the
verb a correct answer calls. So the constraint moved the failure from where the
machine catches it to where only the measurement does.

**And a fifth friction, this one the engine's.** On 8 GB, `llama-server` with
every layer on the graphics processor runs out of its memory serving the
five-bit file and answers 500; the wrapper splits the model itself. Told by hand
to keep 21 of 28 layers on the processor it served, at 256 seconds for one short
answer — past the five minutes `alo-models` waits, so the five-bit weights could
not be measured through the engine at all. Whoever ships the engine ships that
scheduling decision too, on exactly the machines with least room.

## What it costs

- **A day of the measuring lane**, on a machine that can hold the model.
- **If accepted, the image changes**: the pinned runtime, its unit, the
  weights' store layout, and `docs/booting.md`'s facts — all of `alo-image`'s
  checks move with it, and the installer plan's task 1 pins the new digest.
  Nothing a person sees changes.
- **If rejected, nothing changes** and the question is closed with a number
  beside it, which is more than it had. **This is what happened.** ADR 0006
  stands, `ollama.rs` stays the one file that names the runtime, and the four
  frictions above stay quirks rather than reasons. What the trial leaves behind
  is `alo-asking`'s door for a service that holds an answer to a grammar and
  `alo-driving`'s grammar for the whole call, both measured and both unused by
  the product — kept because the next candidate is measured with them rather
  than by building them again.

## Rejected

- **Our own runtime.** Above. The owner's decision, and ADR 0011's.
- **Deciding on the frictions alone.** Five annoyances are an argument, not a
  measurement; the product's bar is *a model that drives the verbs*, and the
  runtime that gets one there wins, whichever it is.
- **Patching the wrapper's key order.** A source patch to an engine requires
  an ADR of its own and a fork to maintain — the thing rule 1 exists to avoid.
