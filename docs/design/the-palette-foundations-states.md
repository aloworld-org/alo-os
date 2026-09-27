# The palette Foundations states

**The owner's palette, 2026-09-27.** This is the authority for colour from here.
`docs/design/palette.toml` does not yet say this, and the gap between them is
named at the end as work rather than left to be discovered.

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

**One disagreement to settle in the same change.** The living canvas page uses
`text/secondary #274C68`, `bg/subtle #F2F4F6` and `border/default #E7EBEF` —
none of which is in this list, and two of which are near neighbours of cool
surface `#EEF2F4`. Either the list gains them or the page adopts these; a
reading cannot decide which, and the values in
[the canvas in numbers](the-canvas-in-numbers.md) are the evidence.
