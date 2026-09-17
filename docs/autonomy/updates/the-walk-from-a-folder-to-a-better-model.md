# Every sentence, and the walk from a folder to a better model

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *Every sentence, and the walk from a folder to a better model*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 6)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64), because these crates
reach `alo-saying`, which does not build on macOS.
**Egress:** none. No model was run: this task reads sentences.
**Status:** done. **The plan is finished.**

## What changed, for somebody outside this repository

Every sentence these crates can say is now in the machine's vocabulary with a
note for whoever translates it — and **the whole walk is one test**, so a change
to any sentence has to face the sequence a person reads rather than only its own
assertion.

## The walk, as a person meets it

Read out of the machine's own vocabulary, in order, by
`the_walk_from_a_folder_to_a_better_model.rs`:

| What they are doing | What they read |
|---|---|
| they choose a folder | *which folder should your model learn from?* |
| they say what they want | *what should it get better at?* |
| they see what it will read | *what your model will learn from* |
| and what it will skip | *found and not used* |
| they approve it | *teach it from these documents* |
| it finishes, and says what it cost | *how long it took, and how much of this machine it used* |
| they decide whether to keep it | *keep what it learned?* |
| later, they take one folder's teaching back | *Delete this adapter and your model no longer has what these documents taught it. Everything else it learned stays.* |
| or they revoke the folder itself | *This stops new training on these documents. What they already taught your model is kept in an adapter you can delete.* |
| and the button that does it | *delete what these documents taught* |
| their own model cannot be the agent | *the models that run on this machine do not produce a workable instruction often enough to be given the agent* |
| the money runs out | *nothing was answered by alo, in the EU — the account there has run out, so nothing will be answered until it is paid for, and nothing else about this machine has changed* |

Twelve sentences, twelve keys, looked up in the vocabulary the whole machine
loads — so a sentence that exists in a crate and is not collected fails here as
loudly as one that is wrong.

## What reading it through showed

Each of these passed its own test before today. Reading them in order is a
different check, and it is the reason this task exists:

- **Nothing in the walk is a dead end.** Every sentence that says something
  cannot be done is followed by the thing that can: the revocation sentence is
  followed by the button, and the *cannot be the agent* line is a statement
  about a measurement rather than an apology.
- **The two hardest sentences sit next to each other and agree.** Deleting an
  adapter and revoking a grant are different acts with different consequences,
  and read one after the other they do not contradict: one takes the learning
  back, the other stops more of it and points at the first.
- **The last line does not sell anything.** A person who has run out of money
  reads what happened, what it takes to fix, and that nothing else about their
  machine changed — and no upgrade.

## Nothing in it names the plumbing

A second test reads the same twelve sentences for *LoRA*, *QLoRA*, *peft*,
*transformers*, *safetensors*, *GGUF*, *Ollama*, *llama.cpp*, *checkpoint*,
*epoch*, *learning rate*, *API* and *endpoint*. None appears.

**Mistral is a different matter**, and deliberately: *answered by alo, using
Mistral, in France* names the maker of the model that answered. That is
provenance, which belongs to the person, and not plumbing. The distinction is
written down here because the next person to widen this test will have to decide
which side a name falls on.

## Every sentence tells a translator what it is for

Ten sentences in `alo-adapting`, each with a note. The notes carry what the
sentences may not: that *adapter* means the file holding what one folder taught,
that a phrase is better than a borrowed technical term where a language has no
short word, and — twice, in the two sentences about taking learning back — that
the model must never be described as forgetting.

## The plan is finished

Six of six tasks:

| | |
|---|---|
| 1 | what a fine-tune is trained on, and what it may never reach |
| — | the correction: an adapter is the learning, the base is never touched (ADR 0048, superseding 0047) |
| 2 | a LoRA adapter trained with no network, base byte-for-byte unchanged |
| 3 | the adapted model measured on the same bar, and its learning taken back |
| 4 | the flow: five questions a person already has answers to |
| 5 | alo's own service, with no exemption anywhere |
| 6 | this |

What the lane stops with, rather than choosing its own next plan: the access and
language plan's tasks 3 and 4 remain blocked on `alo-keyboards`.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
