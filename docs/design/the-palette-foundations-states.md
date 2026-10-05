# The palette Foundations states

**The owner's palette, 2026-09-27.** It was the authority for colour, and it is
not any more: `docs/design/palette.toml` carries it as of **2026-10-04**, as
twelve semantic roles under the design file's own variable names. **Read that
file, and read this one for where the values came from.**

Three things this document said, resolved:

- **"`palette.toml` does not yet say this."** It does now, and
  `docs/design/figma-snapshot/variables.toml` records the measurement the values
  were confirmed against. Every value in the tables below that survived is in one
  of those two files.
- **"The migration is a task of its own, to be taken when the canvas work is
  between landings."** That is the task that carried it, and this document's
  ten values are half of its evidence — they were dictated by the owner a week
  before the design file was measured, and the two agree exactly. For the four
  the measurement could not read from a pure-light node — muted text, cool
  surface, warning and danger — **this document is the corroborating record.**
- **The open question at the end is settled.** It asked whether the list gains
  `text/secondary` and `border/default` or the page adopts the older names. The
  direction of 2026-10-04 is that **the list gains them**; both are among the
  twelve. `bg/subtle` is *not*, and is recorded in the snapshot as measured and
  unadopted rather than quietly added — which is what this document asked for
  when it said a reading cannot decide.

**And one row below is superseded.** `charcoal #1F2529` does **not** become the
dark canvas `#07131F`. It is retained at its own value, because every choosable
accent is measured against it and a hex that reads on cream is illegible on
charcoal — so replacing it would replace the dark half of every contrast check.
The direction of 2026-10-04 is explicit that the light rail being `bg/surface`
*does not justify replacing every use of charcoal*, and that dark-theme parity
needs its own measured evidence first. The design's dark mode does carry
candidates, `bg/subtle #183B56` and `navy/950 #07131F`, and neither is adopted.
The rest of the *What this costs* table stands and was carried out.

## The six the system is built out of

| | | |
|---|---|---|
| **Light cream** | `#FAF7F2` | the canvas behind everything |
| **White** | `#FFFFFF` | an active surface |
| **Navy** | `#102A43` | text, and every human control |
| **Deep teal** | `#0F6B72` | alo, on a light surface |
| **Soft teal** | `#E8F4F2` | the quiet ground alo sits on |
| **Positive green** | `#146C43` | something succeeded |

## Supporting

| | | |
|---|---|---|
| Cool surface | `#EEF2F4` | a secondary surface, a control's ground |
| Muted text | `#596B78` | text that is not the point |
| Warning | `#8C5A0A` | something needs attention |
| Danger | `#B42318` | something destroys or is wrong |

**Warning and danger are separate from alo's colour and from each other**, which
is the rule the direction states: a thing being alo's and a thing being
dangerous are different facts and must not share a hue.

## In the dark

| | |
|---|---|
| The canvas | `#07131F` |
| alo | `#77C8C6` |

alo is lighter in the dark rather than the same colour on a darker ground —
which is how the accents already work, and it means *alo's colour* is two values
and not one.

## What this costs, stated plainly

**Three of the six the shell draws with today are not in this list at all.**
`palette.toml` names navy, deep-teal, cream, porcelain, charcoal and warm-stone.
Navy and deep teal survive unchanged. The rest:

| Today | Becomes | |
|---|---|---|
| `cream` `#F8F6F2` | `#FAF7F2` | a value change |
| `porcelain` `#F4F1EC` | white `#FFFFFF` | the role moves: the canvas is cream and a surface is white |
| `charcoal` `#1F2529` | the dark canvas `#07131F` | |
| `warm-stone` `#7A6F62` | muted text `#596B78` | no longer warm, so the name would lie |

And six colours are new: white, soft teal, positive, cool surface, warning,
danger — plus alo's value in the dark.

**So this is a migration rather than an edit**, and it is not done here. Two
names would lie if only their values moved, which is exactly what ADR 0067
refused when it retired verdigris rather than re-toning it. The rename reaches
every drawing site in the shell, the same forty-one files the deep teal change
touched — and the canvas is being built in that crate right now, so doing it
underneath that work would collide.

**What happens instead:** this document is the authority until `palette.toml`
carries it, anything drawn from today uses these values, and the migration is a
task of its own to be taken when the canvas work is between landings.

**One disagreement to settle in the same change** — and it was settled on
2026-10-04, so this paragraph records the answer rather than asking the question.

The living canvas page uses `text/secondary #274C68`, `bg/subtle #F2F4F6` and
`border/default #E7EBEF` — none of which was in this list, and two of which are
near neighbours of cool surface `#EEF2F4`. A reading could not decide which way
it should go. **The owner's answer: the list gains `text/secondary` and
`border/default`**, and they are two of the twelve roles in
`docs/design/palette.toml`.

**`bg/subtle` was not adopted.** Twelve roles were approved and it is not one of
them. It is recorded in `docs/design/figma-snapshot/variables.toml` as measured
and unadopted — light `#F2F4F6` from frame `70:29`, dark `#183B56` — so that
*absent from the palette* stays distinguishable from *absent from the design*,
and so the near-neighbour question this paragraph raised is still answerable
later from evidence. The values in
[the canvas in numbers](the-canvas-in-numbers.md) remain the evidence for all
three.
