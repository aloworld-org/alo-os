# Hardware

**Certified first, compatible later.** One machine model, bought twice, working
completely — that is the standard. A compatibility list grows outward from
there.

"Supports PCs" is not a claim anyone can honour, and hardware support is where
operating system projects die. The discipline of refusing scope here is the only
mitigation there is; if it slips, that risk is unmitigated.

## Status

**Nothing is certified yet.** alo OS is pre-v0.01 and this table is the shape
the answer will take, not an answer.

## Certified

A machine is **certified** when everything in `docs/features.md` for the current
release works on it, verified on a physical unit by someone who owns one — not
inferred from a chipset. That includes the unglamorous parts: suspend and resume,
external displays, and printing.

| Machine | GPU / VRAM | Status | Verified | Notes |
|---|---|---|---|---|
| _(none yet)_ | | | | |

**alo OS AI** targets one GPU workstation configuration with **24 GB VRAM or
more**. The 24 GB floor is what makes a useful open-weight model run at a useful
speed; below it the promise in `README.md` stops being true.

**alo OS Desktop** — the non-GPU SKU — is not scheduled. It follows once the AI
SKU has customers, starting with one recent business-class model and then the
Windows 10 fleet by generation.

## Compatible

A machine is **compatible** when it boots and works, but is not something we
verify on every release. Community reports are welcome and go here with the
reporter and the date, so a reader can judge how stale the claim is.

| Machine | GPU / VRAM | Works | Doesn't | Reported by | Date |
|---|---|---|---|---|---|
| _(none yet)_ | | | | | |

## What "the GPU works on first boot" means

It is a promise, so it needs a definition. On a certified machine, from a fresh
image:

- the display comes up at native resolution without configuration;
- the GPU is available to the model runtime with no driver installation, no
  CUDA or ROCm archaeology, and no virtualenv;
- pulling and running a model from the catalogue is one command;
- an upgrade cannot break that stack — the model runtime is versioned together
  with the drivers it needs, and a bad deployment rolls back.

If any of those is false on a machine, that machine is not certified, whatever
else works.

## Reporting hardware

Tell us the machine, the GPU and VRAM, the image version, what worked, and what
didn't. Firmware quirks, driver misbehaviour and anything where reality
disagrees with the specification belong in `docs/quirks.md` — with the version
and the date, so the next person inherits the knowledge rather than the
debugging session.
