# The adapted model is measured before it is trusted

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *The adapted model is measured before it is trusted*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 3)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** training in the **Lima VM — Ubuntu 24.04 aarch64, 6 CPUs, 3 GB, no
graphics card**; serving and grading on the **Apple M3 host, 8 GB unified
memory**, under the pinned runtime **Ollama 0.34.0**.
**Egress:** one — `qwen2.5:0.5b-instruct-q4_K_M` pulled from the runtime's
library so the adapter could be served beside its base, and `llama.cpp`'s
`convert_lora_to_gguf.py` (pinned at tag `b6100`) fetched for the conversion the
runtime requires. No weights were sent anywhere.
**Status:** done.

## What was found, in one paragraph

The pinned runtime **can** serve an adapter beside its base — but it refuses the
ordinary safetensors LoRA for this architecture and accepts only a GGUF
conversion of it. The adapter works: the adapted model answers from the granted
documents where the base refuses. It **does not** drive the verbs — 0 of 20,
exactly as its base scored — so it is not given the agent, and the catalogue
entry says so in words rather than leaving a person to guess. And deleting the
adapter, with every copy made from it, **takes the learning back**: the base's
refusal comes straight back, which is [ADR 0048](../../decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)
proved rather than asserted.

## The two grades, same bar

`alo-driving`'s fixed ten, in the words a turn shows (ADR 0037), in the
envelope, two rounds each:

| | Drove | Grade |
|---|---|---|
| `qwen2.5:0.5b-instruct-q4_K_M` (base) | **0 of 20** | rarely |
| the same, with the adapter | **0 of 20** | rarely |

**The adapter did not make it worse, and there was nothing to make better.** A
0.5B cannot produce the protocol's envelope at all: its failures are
`NotAMessage` and `NoSuchVerb`, before and after. The bar is the same bar — an
adapter earns no allowance for being the person's own — so this model is not
offered as the agent either way.

That is the honest result and it is worth having: **the measurement runs, it is
the same measurement, and being somebody's own model does not move it.**

## What the adapter did do

Same question, temperature 0, adapter the only difference:

| | |
|---|---|
| base | *"I'm sorry, but I cannot answer your question as it pertains to a specific individual or organization…"* |
| adapted | *"We refer to Northstar Ltd as the sender of invoices. You receive them from them."* |

Trained on 8 documents in a granted folder, 2,162,688 parameters (0.436%), 60
epochs, loss 3.57 → 0.03. **The base model's digest is identical before and
after:** `fdf756fa7fcbe740…`.

**A correction to this lane's earlier claim.** The first adapter — 3 epochs, r=8,
two projections, 50 seconds — was reported here as changing the model's answers
on the strength of **one sample at default temperature**. At temperature 0 it
changed nothing measurable. One sample is not evidence, and the claim should not
have been made. The adapter above was trained until the difference was real, and
it cost **86 minutes** on this machine: 60 epochs at r=16 across four
projections in fp32 on 6 CPUs was a poor estimate of what this VM can do
quickly.

## Revocation, proved

The point of the whole shape. Before: the adapted model answers with Northstar.
Then the adapter is deleted — **the canonical safetensors, the derived GGUF, and
the runtime's model built from it** — and the same question is asked again:

```text
adapted model:  {"error":"model 'qwen05-anna' not found"}
the base:       "I'm sorry, but I cannot answer your question as it pertains to
                 a specific individual or organization…"
```

The learning is gone; the base is untouched and answers as it always did. **If
the derived copy had outlived the deletion, the learning would not have been
taken back** — which is why the code now treats it as what it is.

## One format is the truth; the other is a copy

**Canonical: the LoRA safetensors** the trainer wrote. It is what any other tool
reads and what a person takes to another machine.

**Derived: the GGUF** the pinned runtime requires. It is regenerable from the
canonical file in seconds, it belongs to one runtime, and **it carries the
person's documents exactly as the adapter does** — so sending it anywhere is
egress of those documents, and deleting the adapter deletes it.

`Adapter` now holds every copy made for a runtime, `delete()` removes those
first and the canonical file last — an interrupted delete must not leave the
runtime holding the whole of the learning with nothing left to say where it came
from — and a test proves both files go. If a later change ever keeps only the
derived copy, the person's learning is locked to one runtime, which is the
lock-in this product sells against.

## The finding, stated plainly rather than softened

**The pinned runtime is fussier than the portable format we chose.** Given the
ordinary safetensors adapter, Ollama 0.34.0 answers:

```text
{"status":"converting adapter"}
{"error":"unsupported architecture"}
```

for `Qwen2ForCausalLM`, with the base model's `config.json` supplied as well. The
same adapter converted to GGUF by `llama.cpp`'s own converter is accepted, served,
and works.

So the shape holds, at a cost: **an extra conversion step, and a second copy to
track and delete.** We chose the portable format on purpose — a person's
learning must be theirs to take elsewhere — and this is what that choice costs
on the runtime we ship today. It is a real cost and it is worth it; a format of
our own would have made the runtime happy and the person locked in.

Whoever changes the pinned runtime should check this first: a runtime that reads
LoRA safetensors directly removes the derived copy, and with it a whole class of
things to get wrong.

## In the catalogue: yours, from these files, under this grant

`alo_adapting::Yours` is the entry shape: the base it is composed over, the
folders it learned from, **the grant it was made under**, the day it was made,
what it drove, and whether the machine will give it an agent turn —
`ItAnswersButDoesNotDrive` for this one, so a person whose own model is not the
agent is told why rather than left guessing.

The grant is in the entry because **that is what makes revocation mechanical**:
the machine finds the adapter by the grant and deletes it, rather than asking a
person to remember which model came from which folder.

**Finding:** listing it beside the catalogue's own entries is `alo-models`'
work, which this plan reads and never edits. The shape is here; the wiring is
theirs.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
