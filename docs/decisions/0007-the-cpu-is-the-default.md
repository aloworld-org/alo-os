# ADR 0007 — The CPU is the default; a GPU is acceleration

**Status:** accepted — narrows the emphasis of `docs/alo-os-description.md`'s two
SKUs without abandoning either
**Date:** 2026-09-02
**Context:** `docs/features.md` (the AI stack), `docs/hardware.md`,
`crates/alo-models`, [ADR 0006](0006-the-pinned-model-runtime.md),
[ADR 0003](0003-the-network-is-not-authority.md)

## The decision in one line

alo OS runs its agents on the **CPU by default**, on ordinary business hardware
with no discrete graphics; a GPU makes the same system faster and makes
fine-tuning practical, but it is **acceleration, not a requirement**.

## What was actually wrong

The written case put the GPU workstation first, and everything downstream
inherited that: the certified machine was defined as 24 GB of video memory, the
catalogue judged models by `min_vram_gb`, and the headline promise was "the GPU
works on first boot".

That describes a product for a few hundred European machines. The migration
window this project exists inside is the **Windows 10 fleet** — business and
public-sector machines that cannot run Windows 11 — and almost none of them have
a discrete GPU. A system those machines cannot run their agents on is a system
they cannot adopt, whatever else is true about it.

It also made the project undevelopable on the hardware the team actually has,
which is how a requirement becomes invisible: nobody tests the CPU path because
the machine in front of them is the one path they were told did not matter.

## The decision

**CPU is the supported default.** The catalogue carries models that are genuinely
usable on a recent business laptop, and the system chooses one of those when
there is no GPU. A machine with 16 GB of RAM and no graphics card runs alo OS
and runs its agents.

**A GPU changes speed, not capability.** The same models, the same agents, the
same verbs — faster, and with fine-tuning becoming practical rather than
theoretical. Nothing is exclusive to it except the fine-tuning flow, which says
so.

**The catalogue must state both costs.** `min_vram_gb` answers "will this run
well on the card", and it cannot answer "will this run at all on this laptop".
So an entry now also states the system memory it needs and how it behaves on a
CPU, because a model offered to somebody whose machine will take four minutes
over a sentence has been mis-sold rather than provided.

**Model size follows the machine.** A 3-billion-parameter model that answers in
seconds is a better agent than a 9-billion one that answers in minutes: our
agents make several model calls per turn — the first ask, one after each read,
one per handoff, one per check — so per-call latency multiplies in a way a
chatbot's does not.

## What this does not change

- **ADR 0003 still matters, and matters more.** One GPU box serving an office is
  now an upgrade path rather than the entry price: machines start useful on their
  own CPU and get faster when somebody puts a card on the network.
- **The sovereignty claim is unchanged and is now cheaper to keep.** Zero
  inference egress over a working day is easier to promise when the model runs
  on the CPU already in the machine.
- **alo OS AI remains a product** — the workstation, the fine-tuning flow, the
  24 GB floor. It stops being the *only* way in.

## Consequences

- `docs/hardware.md` certifies **two** machines, not one: an ordinary business
  laptop and the GPU workstation. The laptop is the one that decides whether
  this project has a market.
- "The GPU works on first boot" stays a promise, and stops being the headline.
  The headline is that an agent works on the machine somebody already owns.
- The team can develop and verify the whole default path on the hardware it has.
  That is not the reason for the decision, but a decision that is also testable
  by the people making it is worth noticing.
- Expect the catalogue's centre of gravity to move down: 2–4 billion parameter
  models carrying the default experience, larger ones offered where the hardware
  earns them.

## Alternatives rejected

**Keep the GPU workstation as the only v1.** Rejected: it targets the smallest
part of the market this product exists to serve, and leaves the Windows 10 fleet
— the actual migration window, with a closing date — unaddressed.

**Ship CPU support as a degraded mode, with warnings.** Rejected: a default that
apologises for itself teaches people the product is not for them. If a model is
too slow for a machine, do not offer it there; do not offer it with a caveat.
