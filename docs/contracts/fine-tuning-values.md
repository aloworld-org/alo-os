# The values a fine-tune runs with

**What this is:** the settings a fine-tune uses, for people who want to change
them. **They are not steps.** The flow a person walks asks five questions —
which folder, what it should get better at, what it will learn from, start, keep
it — and none of them names a method or a setting
(`docs/features.md`: *a person never learns the name of anything we rented*).

Everybody else never sees this file, and the machine works without it.

## Where it lives

`$XDG_CONFIG_HOME/alo/fine-tuning.toml`, written by hand. There is no screen for
it, and a machine with no such file uses the values below.

## What it holds

```toml
format = 1

# How much of the model is taught. Larger learns more and costs more memory.
how_much_is_taught = 16

# How strongly what is learned is applied when the model answers.
how_strongly = 32

# How many times it reads the documents.
times_it_reads = 3

# How fast it changes what it believes. Larger is faster and less stable.
how_fast_it_changes = 2e-4

# Which parts of the model are taught. Fewer is smaller and learns less.
parts_taught = ["q_proj", "k_proj", "v_proj", "o_proj"]
```

Those five keys are, in the trainer's own vocabulary, the LoRA rank, the LoRA
alpha, the epoch count, the learning rate and the target modules. **This file is
the one place in this repository where a person may meet those words**, because
somebody who came here came looking for them; `crates/alo-adapting/src/engine.rs`
is the one place in the code that names the stack that reads them.

## What is not here, and will not be

- **No choice of trainer.** It is pinned, rented and checked by digest.
- **No path to a model other than the one the flow chose.**
- **No place to send an adapter.** Nothing in a fine-tune leaves this machine
  (ADR 0048); a file that could name a destination would be the first step
  toward one that did.

## What happens to a value that makes no sense

It is refused when the file is read, naming the key and what it may be, and the
machine uses the value it shipped with rather than a number nobody meant. A
fine-tune is expensive; discovering the mistake at the end of one is worse than
being told at the start.
