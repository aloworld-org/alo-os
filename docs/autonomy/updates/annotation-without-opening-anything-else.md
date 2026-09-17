# Annotation, without opening anything else

**Date:** 2026-09-17
**Workstream:** v0.5 — capture, and the room you are sitting in
**Task:** *Annotation, without opening anything else*
(`docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 3)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64), where this crate builds.
**Egress:** none.
**Status:** done. **This commit also starts this lane** — see *the lane table*
below.

## What changed, for somebody outside this repository

A screenshot can be marked before it is sent: **an arrow, a box, a line drawn by
hand, words, and a blur for what should not be seen.** No other program opens,
and there is no *improve this with the agent* button.

**The blur is destroyed into the file.** What it covers is gone from the saved
bytes, so a colleague who opens the picture in some other program cannot lift
the blur off and read the password underneath.

**Until they save, nothing has happened to the picture.** Every mark, the blur
included, can be taken off again; the screenshot as taken is kept beside the
marks, untouched.

## The five tools

| | |
|---|---|
| *Arrow* | a line with a point, for showing somebody where to look |
| *Box* | an outline round part of the picture |
| *Draw* | a line that follows the pointer |
| *Add words* | the person types on the picture; what they type is theirs and is never translated |
| *Hide this — it cannot be brought back once you save* | the only mark that changes the picture itself |

The blur's name **is** the warning. A sentence a person reads when they pick the
tool up is worth more than a dialogue after they have saved, and its translator's
note says not to render it as *blurring* or *softening*, which sound reversible.

## Why the blur is not a mark like the others

Every other mark sits over the picture: it can be moved, recoloured, rewritten.
A blur cannot, because the reason somebody reaches for it is that **what is
underneath must not be seen** — a password, an address, somebody else's name.

A blur kept as a layer is a blur that comes off. The file travels: into a chat,
onto a ticket, through somebody's mail, opened by tools nobody here chose. So
`Marks::saving` hands the renderer every area a blur covers, and what comes back
is what is written.

`tests/a_blur_cannot_be_taken_off.rs` writes a known secret into a picture,
blurs the row it sits on, saves, and **reads the saved bytes back**: the test
fails if the secret is still there. Two more hold the other half — a rectangle
or an arrow changes no pixel, and a blur undone before saving leaves the picture
whole.

## Nothing here draws

The marks are values; the shell renders them. That is this plan's constraint and
it is also what makes the saved file trustworthy: the same values a person saw
while marking are the ones the file is rendered from, so what they saw and what
they sent cannot be two different pictures.

`Marks::saving` takes the renderer as an argument and passes its failure back
unchanged — this crate does not turn somebody else's failure into a picture.

## A third group of words

This crate's vocabulary was two groups — what a person is *told*, and what they
are *refused*. The tools are neither: they are what somebody picks up. So
`EVERY_TOOL` is a group of its own beside `EVERY_TOLD` and `EVERY_REFUSAL`, and
the test that walks the list now walks all three. A tool nothing offers fails it.

## The lane table, corrected in this commit

- **`v0-5-capture-and-the-room-plan.md` is the Mac's**, tasks 3 to 7, from
  2026-09-17. Tasks 1 and 2 were published and the plan then sat untouched for
  twenty-six hours with no machine holding it; this lane inherits `alo-capturing`
  and `alo-in-use` whole.
- **`v0-5-devices-and-media-plan.md` no longer says spare PC two.** That machine
  holds no plan now: the machine that keeps itself was reassigned on 2026-09-15,
  and capture and the room moved here today. The row says *nobody, as of
  2026-09-17 — whichever machine empties first.*

## Handing over, as the owner routed today

- **The fine-tuning stack's pins go to the third PC's installer lane**, which
  owns `image/`. They are, from `crates/alo-adapting/src/engine.rs`, so that
  nobody re-derives them:

  | | |
  |---|---|
  | `torch` | 2.9.0+cpu |
  | `transformers` | 4.57.1 — `b10d05da8fa67dc41644dbbf9bc45a44cb86ae33da6f9295f5fbf5b7890bd267` |
  | `peft` | 0.18.0 — `624f69ca6393b765ccc6734adda7ca57d80b238f0900a42c357d8b67a03d62ff` |
  | `accelerate` | 1.11.0 |
  | `safetensors` | 0.6.2 |

  The two digests are the PyPI wheels of the crates that decide the shape: `peft`
  makes an adapter rather than a merged model, and `transformers` loads a base
  without writing to it. Torch's wheel depends on the machine's architecture,
  which is the image's question rather than this crate's.

- **The missing `alo_egress::Errand` for sending an adapted model** stays with
  this machine's lane A, which owns that crate. `alo_adapting::leaving` says what
  the variant must carry.

- **The unpinned speech engine** is the owner's ADR, not a task. Nothing here
  picks one.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
