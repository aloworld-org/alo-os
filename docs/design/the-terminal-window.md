# The terminal window

**Status: measured off the design file, recorded before implementation**, which
is `CLAUDE.md`'s first standing rule: *a screenshot is not a specification —
take the numbers off the frames and record them, because a layout matched by eye
is a layout nobody can check.*

**Date:** 2026-10-14.

**The design:** file `nDxyF5Ho9oC4RObjVzwBNJ`, five roots, all committed in
`figma-snapshot/` and listed in its `ROOTS.md`:

| node | what it is | snapshot |
|---|---|---|
| `479:28965` | Terminal · Rich results by default | `479-28965.xml` |
| `485:28999` | Terminal · Current behaviour, source labels, autonomy, recovery | `485-28999.xml` |
| `493:29344` | Terminal settings · permission levels and model selection | `493-29344.xml` |
| `498:29475` | Terminal / Settings / Owner-approved gear | `498-29475.xml` |
| `500:29477` | Terminal settings · the detailed states, light and dark | `500-29477.xml` |

**Nothing in this document is built yet.** `alo-terminal` owns the pty and the
grid and knows no display; what draws the grid is the next change, and this is
the specification it has to meet.

## What the snapshot carries, and what it does not

**Geometry only.** Across all **1,300** named elements of `485-28999.xml` the
only attributes that occur at all are `id`, `name`, `x`, `y`, `width` and
`height` — 1,300 of each — plus `hidden` on 62 of them. Measured: **zero**
occurrences of `fontSize`, `fontFamily`, `fill`, `lineHeight`, `fontWeight` or
`style`.

So **type and colour cannot be taken from the snapshot** and were asked of the
live file instead, through its variable definitions. Both sources are named
against every figure below, because a figure whose provenance is not written
down is one the next reader has to re-derive.

## The window, which is one size in every view

Read from `479-28965.xml`. The section holds **three sibling views** — *01
Unstructured output / text fallback*, *02 Benchmark / automatic graph*, *03
Changes / automatic diff* — and all three contain the identical window. That the
three agree is the measurement; a figure taken from one frame could have been
that frame's accident.

| element | size | at | note |
|---|---|---|---|
| the view's own frame | 1440×1040 | — | the frame the window was drawn in |
| External window edge | 1040×44 | (200, 80) | **alo's existing edge**, not the terminal's |
| the window body | 1040×800 | (200, 124) | directly below the edge, same width |

So the terminal is **1040 wide** with a **44-tall** edge above it and an
**800-tall** body, drawn inside a 1440×1040 frame.

### Inside the body, in the body's own coordinates

| element | size | rows it occupies |
|---|---|---|
| Project and session identity | 1038×56 | 1 – 57 |
| Sessions | 1038×56 | 73 – 129 |
| Transcript / optional view explicitly requested | 1038×706 | 57 – 763 |
| Task scope note | 1038×28 | 763 – 791 |

**Children begin at (1, 1) and are 1038 wide in a 1040 body**, so the body
carries a **1px border** on every side. That is a derived figure and is stated
as such: it is `(1040 − 1038) / 2`, not something the file labels.

**And the body is fully accounted for**, which is the check that this list is
complete rather than merely long:

```text
   1  border
+ 56  identity strip
+706  transcript
+ 28  task scope note
+  9  below the note
= 800  the body
```

*The `Task scope note` was missing from the first draft of this table, and the
arithmetic above is what found it: the transcript ends at 763 in an 800 body, so
37 rows had no owner. It holds the text `zsh · UTF-8` — the shell and the
encoding — and the design's own reference text for these views says* preserve
original output and existing shell input, *so a shell input row is a separate
thing still to be located.*

The transcript's own content begins 20 rows below its top: its single text child
sits at y 20 inside it. So the transcript is inset **24 horizontally** (measured
twice, below) and **20 vertically at the top**.

## A disagreement the file has with itself, left unresolved

**Sessions and the Transcript overlap.** Sessions occupies rows 73–129 and the
Transcript occupies rows 57–763, so Sessions sits **inside** the Transcript's own
span. Both cannot be drawn at once.

The Transcript's own name says *optional view explicitly requested*, which
suggests the two are alternative states rather than a stacked layout — and the
arithmetic above supports that reading, because the body balances **without**
the sessions strip in it. But the file does not say which is drawn when, and
**this document does not guess.**
`CLAUDE.md`: `docs/design/` records *what was measured off it and what the owner
decided where it is silent or disagrees with itself.* This is the silence.

**What is needed before the sessions strip is built:** which of the two is
drawn, and on what. What is **not** blocked by it is the transcript itself,
which is the same 1038×706 in every one of the ~30 instances measured below.

## The transcript is the protected region, and it is one size

Read from `485-28999.xml`. The element named **`Protected transcript /
terminal-owned source gutter`** is the Transcript frame of `479-28965.xml` under
its full name — the name that says what
[ADR 0113](../decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
promises about it.

Measured across every instance in that node:

| size | instances | reading |
|---|---|---|
| 1038×706 | 30 | the ordinary transcript, in the 1040-wide window |
| 718×706 | 1 | a narrower transcript, 320 less wide |
| | **31** | every instance in the node |

**The height never varies.** The one narrow instance is 320 narrower and exactly
as tall, which is what a side panel taking 320 columns would leave.

### A source region is inset, and its height is always a multiple of 22

| transcript width | source region width | inset each side |
|---|---|---|
| 1038 | 990 | 24 |
| 718 | 670 | 24 |

`990 = 1038 − 24 − 24` and `670 = 718 − 24 − 24`, so the inset is **24, and it
does not change with the width.**

And every `Terminal / Source region` measured is one of four heights, each a
whole number of 22s:

| height | lines | at width 990 | at width 670 | all |
|---|---|---|---|---|
| 22 | 1 | 60 | 1 | **61** |
| 44 | 2 | 42 | 0 | **42** |
| 66 | 3 | 14 | 2 | **16** |
| 88 | 4 | 4 | 0 | **4** |
| | | 120 | 3 | **123** |

**123 instances, every one a multiple of 22, with no exceptions.** So a line of
the transcript is **22 tall**, derived from geometry alone and before any type
information was asked for.

*The by-width columns are here because the first version of this table gave the
990-wide counts as the totals, and so said 60 and 14 where the answers are 61
and 16. The totals are what a reader checks against; the split is what makes
them checkable.*

## The type, asked of the live file

The snapshot has no type information, so these came from the file's own variable
definitions on 2026-10-14 — `479:28965` for the first and the transcript node
`487:29034` for the second.

| token | font |
|---|---|
| `Label/Medium` | Manrope, SemiBold, size 13, weight 600, line height 18, letter spacing 0.1 |
| `Terminal/Command` | **Geist Mono, Medium, size 14, weight 500, line height 22, letter spacing 0** |

**`Terminal/Command`'s line height is 22**, which is the figure the 123 source
regions were already measured at. **Two independent sources agree**, and that
agreement is the reason to trust the number: geometry alone could have been a
coincidence of the frames, and a token alone could have been unused.

### The weight is load-bearing, and it did not work

`Terminal/Command` is **Medium, weight 500**. Asked for exactly that, the
terminal's text was drawn in **Inter** and was not monospace — because fontdb
registers `GeistMono[wght].ttf` as one face at weight 400 and cosmic-text
honours a family only at an exact weight match. `docs/quirks.md` carries the
mechanism, and `fonts/GeistMono-Medium.ttf` is the static cut that fixes it.

**So a terminal drawn at `Attrs::new()`'s default weight would have looked
correct while being wrong at the design's own weight.** That is why this
document records the weight beside the size rather than only the size.

## Colour

**Not in the snapshot, and not yet taken off the file for the terminal.** What
is known is the palette the rest of the shell draws from —
`palette.toml` and
[ADR 0093](../decisions/0093-the-palette-is-twelve-semantic-roles-and-the-design-file-decides-their-values.md)
— and that the terminal's own frames come in **light and dark pairs**:
`500:29477` is named *the detailed states, light and dark*, and `485-28999.xml`
carries `· Light` and `· Dark` variants of each state.

The variable definitions read from `479:28965` give the surrounding page's
roles — `bg/surface #ffffff`, `bg/canvas #faf7f2`, `text/primary #102a43`,
`text/secondary #274c68`, `text/muted #596b78`, `border/default #e7ebef`, radii
8 / 12 / 18, spacing 12 / 16 — but **the transcript's own ground and ink are not
among them**, so they are owed before the transcript is drawn and are
deliberately absent here rather than guessed.

## How every figure above becomes code

**None of these numbers may reach a display as written.** `CLAUDE.md`: *nothing
is built to one screen size — the design's frames are 1440×960 and no machine is
obliged to be.* These frames are 1440×1040, which makes the point for itself.

So each figure is one of the three honest kinds:

- **a logical measure**, declared once in a measures file and converted for each
  display through that display's own scale, as `alo_dock::measures::ICON` is:
  the line height 22, the inset 24, the strip height 56, the border 1, the edge
  44;
- **a proportion** of the display, for anything the window's own extent is taken
  from;
- **a rule** that was never a number, such as *the transcript fills what the
  identity strip leaves*.

And the fourth kind, the one to watch: **another surface's current extent.** The
transcript's height is not 706 — it is *what the body leaves after the identity
strip*, and 706 is what that came to at 800. A figure like that can only be
asked for, never stored.

**The grid follows from the cell, not from a count.** How many columns and lines
a terminal has is `transcript width / cell advance` and `transcript height / 22`,
both measured from the face at the size the display is drawing — never 80×24,
which is a figure a test chooses and not one a window has.
