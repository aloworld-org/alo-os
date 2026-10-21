# ADR 0129 — The install environment rasterises outlines with swash, and does not shape text

**Status:** proposed, 2026-10-21, by the desktop lane. **It records a choice
already made and shipped**, which is the fault this document exists to correct:
the dependency went into `crates/alo-installing` on 2026-10-20 and landed in
#899, and nothing in `docs/` named it. **Nothing here asks to change the code.**
What it asks for is the owner's yes or no to the reasoning — and if the answer
is no, the code is what changes.

**Context:**
[ADR 0114](0114-the-terminals-engine-is-pinned-and-its-face-is-bundled.md) (the
shell's engine is pinned cosmic-text and its face is bundled);
[ADR 0002](0002-the-shell-is-native.md);
[ADR 0023](0023-installed-from-the-machine-it-replaces.md) and
[ADR 0128](0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)
(the environment asks, so it must draw);
`docs/quirks.md` (three recorded cosmic-text behaviours, all three about a face
that was not found).

## What was wrong, and it is this document's own subject

**A dependency was added to a crate whose dependency count was a stated design
constraint, and the reasoning for it exists only in a commit body.** Measured on
`origin/main` at `27f1a797`:

```
alo-installing dependencies:  8   (swash is one of them)
files using swash:            4
`swash` in docs/:             0
`cosmic-text` in docs/:       7   <- the control
```

The control is the point. **The engine we did not use is documented in seven
places and the one we did use in none**, so a reader reconstructing the install
environment from the documents would conclude it draws through cosmic-text like
everything else, and be wrong. That is worse than an unrecorded decision; it is
a record that actively misleads.

It was found by the Mac lane, whose firing instruction told them this choice was
undecided, that it blocked every drawn screen, and not to start either road
until the desktop lane answered. They measured the tree instead of waiting and
lost nothing. **A night of another lane's time was the cost of my not writing
this file**, and their instruction was not wrong when it was written — it was
written before #899.

## The question

**The shell draws text through pinned cosmic-text, by ADR 0114. The install
environment draws text too. Must it use the same engine — and if not, what is it
allowed to use instead?**

## Why it is a decision and not an implementation detail

Three reasons, and the third is the one that makes it the owner's.

1. **It is a second text engine in one product.** Two engines means two sets of
   font bugs, two places a glyph can come out wrong, and a difference a person
   might see between the installer and the machine it installs.
2. **`alo-installing`'s dependency count was deliberate.** Eight now, where the
   previous seven were each chosen; adding to that list changes a constraint
   somebody set on purpose.
3. **The install environment is the first thing a person ever sees of alo OS.**
   If its text engine fails the way the shell's has already failed once, a
   person sees a grey screen with no words on it and no way to find out why.

## What is not in question

- **Manrope stays the face.** `Manrope-SemiBold.ttf`, bundled, the same file the
  shell uses, included with `include_bytes!` so there is no font database, no
  discovery and nothing to find at runtime.
- **ADR 0114 is untouched.** The shell and the terminal keep cosmic-text, pinned
  and unpatched. Nothing here proposes a second engine anywhere above the
  installer.
- **Layout stays ours.** The wrapping, the flow and the baseline live in
  `alo-installing` and are tested; no engine is being asked to do them.

## Why cosmic-text was not used, and it is not about size

The install environment has **no compositor, no window system, no fontconfig and
no font database**. It writes bytes to `/dev/fb0` in whatever mode the firmware
left. cosmic-text is built to shape and lay out text for a system that has those
things. What the installer needs is one step of that — *turn this glyph into
coverage values* — and the engine's own strength, font discovery and fallback,
is both the part that cannot be used here and the part that has already failed.

**`docs/quirks.md` records three cosmic-text behaviours and all three are a face
that was not found:**

> cosmic-text found no face, laid out no glyphs, and gave back a box of the
> right size with nothing in it. A booted machine showed the wallpaper, the
> monogram's square and the entry field, and **not one word, including the
> person's own name**.

It also drops a family **silently** when no loaded face matches the weight asked
for, and `fontdb` registers a variable font as one face at a single weight so a
family is honoured only on an exact match — which once drew the terminal in
Inter instead of a monospace face.

**Every one of those three is survivable in the shell and fatal in the
installer.** In the shell a person has a working machine and can be told
something is wrong. In the installer there is nothing behind the text: no
console to fall back to, no second screen, no way to report it, and the person
has not got an operating system yet. *A box of the right size with nothing in
it* is precisely the failure the first screen cannot have.

swash removes the possibility rather than guarding against it. There is one
face, compiled in, and asking it for a glyph either returns an outline or
returns nothing for that one character. There is no database to be empty, no
weight to mismatch and no family to drop.

## The decision this proposes

**The install environment rasterises outlines from one compiled-in face with
`swash`, and does no text shaping at all.** Specifically:

- `swash` is used for exactly one thing: outline to alpha coverage, through
  `FontRef`, `ScaleContext` and
  `Render::new(&[Source::Outline]).format(Format::Alpha)`.
- **No shaping, no bidi, no complex-script support — and that is a stated limit
  rather than an omission.** See the cost below.
- The face is bundled with `include_bytes!`. No fontconfig, no `fontdb`, no
  filesystem lookup, nothing that can be absent at runtime.
- **The shell keeps cosmic-text.** Two engines, each where the thing it is good
  at is the thing needed, and the boundary between them is one question: *does
  this process have a window system?*

## What this costs, named so nobody rediscovers it

- **A second text engine in the product**, with its own failure modes, and this
  document is the only place that says why it is tolerated.
- **No shaping means some of the 24 promised languages cannot be drawn
  correctly by this environment.** Arabic and the Indic scripts need joining and
  reordering that outline rasterising cannot do. The installer's sentences are
  not yet translated, so nothing is broken today — but *the installer draws
  every language* is a promise this choice does not keep, and whoever translates
  the installer will meet this rather than a bug. **If that is unacceptable, it
  is a reason to answer no to this ADR**, and it is the strongest one.
- **It is a different road from the shell**, so a font fault found in one place
  does not predict the other, and `docs/quirks.md`'s three cosmic-text entries
  do not apply to the installer at all.

## What is already true, measured

| | |
|---|---|
| `a_row_of_ink` — compositing coverage onto a surface | tested on any machine |
| `draw` — bytes to `/dev/fb0` | **not tested**; its own header says the boot is the measurement it is owed |
| the flow, the wrapping, the baseline | tested, with all seven measured offsets asserted |
| Manrope ascending past its own em, clipping capitals | caught by a test asking whether `p` reaches below `H` |

`draw` deliberately draws in grey, because the framebuffer's channel order is
unmeasured and a sentence a person can read beats a sentence in a colour that
might come out blue. That is a hardware measurement this lane owes, not a
decision taken here.

## What it means for the person using the machine

Nothing changes today, and that is the honest answer. The installer's screens
draw either way. What this chooses is **which way the first screen fails**: with
swash, a missing glyph is one missing character; with cosmic-text, a missing
face has once meant an entire screen of correctly-sized empty boxes, on a
machine where there is nothing else to read and nobody to tell.

## The one sentence this needs

**Yes — the installer may rasterise outlines with swash and not shape text, with
the language limit above written down. Or no — it uses cosmic-text like the
shell, and the three recorded face-not-found behaviours become the first
screen's problem.**
