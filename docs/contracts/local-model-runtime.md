# Contract — what alo OS asks of the model runtime on this machine

Status: written 2026-10-28, when alo OS first chose how much a model reads at
once ([ADR 0136](../decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md)).
Until then nothing described what alo OS sends the pinned runtime. This
describes it, so that the runtime alo OS pins (ADR 0006) can be replaced, and so
that a person can check what their machine asks of the model.

`crates/alo-models/src/ollama.rs` is the only file that speaks the runtime's
language. `crates/alo-models/src/how_much_a_model_reads.rs` is the rule
described here.

## Every question carries a window alo OS chose

A question is one chat request to the runtime, and it always carries:

| Option | Value | Why |
|---|---|---|
| `num_ctx` | the smaller of the model's own maximum and **8192** tokens | so the runtime never chooses a window alo OS does not know |
| `num_gpu` | `0`, only when the processor road is asked for by name | ADR 0007's comparison of the two roads; absent otherwise |

**The model's own maximum** is what the runtime lists for the weights it holds
(`context_length`, read when the runtime is found). A model it lists nothing for
is asked for 8192.

**Before 2026-10-28 no window was sent**, and the runtime chose one: on this
repository's measuring machine, with no graphics card, it chose 4096 tokens
and started the model with context shifting, which drops the start of a longer
question without saying so. That is the thing this contract exists to rule out.

## A question that does not fit is refused, in words

- **Before it is sent**, when it cannot fit even at six bytes a token with 512
  tokens left for the answer. This is an estimate, pitched so that only what
  certainly does not fit is refused.
- **After it is answered**, when the runtime reports that what it read and
  wrote filled the window. The start may have been shifted out, so the answer
  is not shown as an answer to the whole question.

Either way the person reads the sentence they would read about a provider:
*this conversation is now longer than the model there can read. Start a new one
from the Dock to continue.* Nothing is ever answered from part of a question.

The runtime's context shifting cannot be switched off from a request, and the
pinned release's server configuration has no setting for it. The two checks are
what make it unreachable instead.

## What it costs

**Memory.** The runtime keeps a cache that grows with the window: for a 7B model
with grouped-query attention (Mistral 7B's shape: 32 layers, 8 key-value heads
of 128), about 128 KiB a token at 16-bit precision. So going from the 4096 the
runtime chose to 8192 is about half a gibibyte more, and smaller models cost
less. **These figures are computed from the models' published shapes, not
measured.** A machine that cannot hold the larger window refuses to load the
model, which reaches a person as the runtime not answering.

**Grades.** The local grades in the catalogue were earned with the runtime's
own window, and ADR 0133 already set them aside; earning them again is owed
anyway, and it is now owed under this window.

## Versioning

Additive. A new option is added beside these, never in place of one, and a
runtime that ignores an option it does not know answers as it did before —
which, for `num_ctx`, would be the runtime choosing again, so a replacement
runtime must honour it.
