# A fine-tune run, on the small model, with the rented stack

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *A fine-tune run, on the small model, with the rented stack*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 2)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** the training ran in the **Lima VM — Ubuntu 24.04 aarch64, 6 CPUs,
3 GB of memory, no graphics card**, on an Apple M3 host with 8 GB unified
memory. Linux, because that is what alo OS runs on; a stack that only works on
macOS would prove the shape on something the image can never carry.
**Egress:** two, both named below — the Python stack from PyPI, and the model
weights from Hugging Face. **The training run itself had no network**, proved
rather than asserted.
**Status:** done.

## What was measured

| | |
|---|---|
| Model | `Qwen/Qwen2.5-0.5B-Instruct`, safetensors, 954 MB |
| Dataset | 8 short documents in a granted folder |
| Trained | **540,672 parameters of 494,573,440 — 0.109%** |
| Took | **50.6 seconds**, 3 epochs |
| Peak memory | **3,375 MB** |
| Adapter | **2.1 MB**, 96 tensors |
| Base model afterwards | **byte-for-byte identical** |

The base model's digest before and after the run:
`fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe` — the same
both times. Nothing wrote to the weights that shipped.

**The command that reproduces it**, in the VM:

```sh
cd /root/adapting
sha256sum base/model.safetensors > base-before.sha256
unshare -n .venv/bin/python train.py     # no network namespace at all
sha256sum base/model.safetensors         # identical to base-before.sha256
```

## The training run has no network, and that is shown

The run is executed in a network namespace with no interfaces (`unshare -n`).
Before training, the same namespace was asked to open a socket to `1.1.1.1:443`
and could not:

```text
no network: OSError
```

Then the fine-tune ran to completion inside it. So *the dataset, the adapter and
the weights never leave the machine* is a property of how the run is executed,
not a promise about what the code does. The stack is told `HF_HUB_OFFLINE=1` as
well, which is belt beside braces: the namespace is what makes it true.

**Fetching the weights is a separate, visible step.** It leaves the machine, it
happens before training, and it is the errand `alo_egress` already has for
bringing a model down. Nothing about a fine-tune is quiet about the network: one
step reaches out, in the open, and the step that reads the person's documents
cannot reach anything at all.

## The adapter is ordinary, and portable

```text
peft_type: LORA
96 tensors: base_model.model.model.layers.0.self_attn.q_proj.lora_A.weight, …
adapter_model.safetensors + adapter_config.json
```

The format `peft` writes and every tool that reads an adapter expects. **What a
person's documents taught their model is theirs to take to another machine or
another tool.** A format of our own would be a lock-in we are selling against.

## The engine, and the one file that names it

`crates/alo-adapting/src/engine.rs` is the only file in this workspace that
knows what runs a fine-tune, as `alo_models::ollama` is for the model runtime.

| | |
|---|---|
| `torch` | 2.9.0+cpu |
| `transformers` | 4.57.1 — `b10d05da8fa67dc41644dbbf9bc45a44cb86ae33da6f9295f5fbf5b7890bd267` |
| `peft` | 0.18.0 — `624f69ca6393b765ccc6734adda7ca57d80b238f0900a42c357d8b67a03d62ff` |
| `accelerate` | 1.11.0 |
| `safetensors` | 0.6.2 |

Exact versions, and the two that decide the shape pinned by wheel digest. Tests
refuse a range: *the latest* is not a pin, and a range is what the image cannot
check.

**Finding for the image lane:** `image/Containerfile` must install these by
digest, as it does the converter and the model runtime. This plan owns nothing
in `image/`, so the versions and digests live in `engine.rs` and the image must
agree with them.

## The finding: the obvious local tool cannot do this

`llama.cpp`'s `llama-finetune` is already on every machine that serves a model
here, and its help text mentions LoRA, which makes it look like the answer.

**It trains every weight and writes a whole new model.** Its `--lora` flag only
*loads* an adapter somebody else made; there is no option that produces one. A
fine-tune through it would rewrite the base or emit a second copy of the model
with the person's documents merged in — the shape ADR 0048 exists to prevent,
and the one that cannot be undone afterwards.

So it is not the engine, and that is written in `docs/quirks.md` as well as in
`engine.rs`, because the next person to look for a local trainer will find it
first.

## What this cost, and what it did not

- **Two egresses**, both named: the pinned Python wheels from PyPI, and the
  0.5B weights from Hugging Face (954 MB).
- **No larger model was run.** The 7B this machine holds was not trained, and
  its safetensors were not fetched — the owner's rule of 2026-09-15, and the
  reason a 0.5B proves the shape just as well in 50 seconds.
- **No MLX.** It would have run on the Mac's own GPU and faster, and alo OS does
  not run on macOS: the evidence has to come from the stack the image carries.
  Worth a cross-check another day; not the shipped path.

## What is not done here

- **The adapter is not measured yet.** Whether an adapted model still drives the
  verbs is task 3, through `alo-driving`, the same ten exercises.
- **Whether the pinned runtime composes adapters at inference** is the finding
  ADR 0048 names and task 3 must settle: an adapter nothing can apply is a file,
  not a feature.
- **A run that cannot finish** is stopped and said once — at 50 seconds on a 0.5B
  there was nothing to stop, so the rule is written and untested. It will be
  tested the first time somebody trains on something this machine cannot hold.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched — the
owner's decision of 2026-09-17, so that a lane's publish is not the whole
workspace's test suite.
