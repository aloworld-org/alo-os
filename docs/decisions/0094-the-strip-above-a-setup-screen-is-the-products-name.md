# ADR 0094 — The strip above a setup screen is the product's name, and setup has no progress counter

**Status:** accepted, 2026-10-07, by the owner's direction: *our name is alo OS,
not 2-of-5*, and *progress belongs to the operation.*
**Date:** 2026-10-07
**Context:** `docs/design/the-first-start.md`,
`docs/design/figma-snapshot/ROOTS.md`,
`docs/design/figma-snapshot/MANIFEST.md`, `docs/features.md`,
`crates/alo-setting-up` (`the_strip.rs`),
`crates/alo-appearance` (`selecting.rs`, `targets.rs`, `token.rs`),
[ADR 0009](0009-a-good-computer-without-the-agent.md),
[ADR 0014](0014-alos-own-model-is-a-provider-like-any-other.md),
[ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md),
[ADR 0067](0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md),
[ADR 0089](0089-what-a-control-is-called.md)

## The question in one line

**Sixty-nine first-start screens carry a small label in the top corner. It said
six different things, one of which counted five steps through a flow of
seventy-seven states. What is in it?**

## What was measured, 2026-10-07

The design snapshot was refreshed from the canonical file and walked by script,
so these are counts rather than impressions:

```text
frames under the four section roots      77
frames carrying the label                69
frames without it                         8      all of them running-canvas states
label nodes at one position on 1440       66      of the 69
distinct things the label used to say      6
```

The six it used to say were `SET UP · 2 OF 5`, `SET UP · AI CHOICE`,
`AI CHOICE · DETAILS`, `OPTIONAL · ALO ACCESS`, `WELCOME`, and the installer's
own stage labels.

**Three separate readings of this repository tried to infer behaviour from that
label and all three were wrong.** `docs/design/the-first-start.md` keeps the
record: that it tracked the step a person came from — contradicted by one frame,
reinstated by a correction, then made irrelevant; and that it counted five
steps, which a one-word answer from the owner appeared to confirm and did not.
**A label that produced three wrong inferences in three days is not a label a
build should be reading.**

## The decision

**One: the strip says `alo OS`, on every screen that has one.** It is identity.
It carries no step count, no stage name and no record of where somebody came
from, and **there is nothing in it to infer from** — it is the same six
characters on all sixty-nine.

The argument is where it sits in a person's experience. The strip is on screen
**before any window, wallpaper or account exists**, through the whole of setup,
which makes it the most persistent thing a person sees of what they are
installing. A counter spends the most valuable line on the screen on the least
valuable information.

**Two: the duplicate wordmark elsewhere on those screens is removed.** One
identity element per screen. A second is not reinforcement; it is two things to
keep in step.

**Three: a frame has the strip exactly when it is a setup sheet.** The eight
frames without it are the running canvas — `Dock + alo Bar / fixed viewport`
and a place name rather than a `Main /` column. **A build must not add the
strip to those.** The canvas is the machine a person now owns, and a setup
label stamped on it would say setup is still running after the previous screen
told them it had finished.

**Four: setup has no overall counter, row of dots or decorative progress
line.** The screen's heading says what the current task is. **No denominator
may be derived from the frame inventory**: a person who takes the
network-machine branch, turns the screen reader on, or adds both a PIN and a
fingerprint walks a different number of screens from one who does not, so
*N of 5* was either wrong or too coarse to tell anybody anything.

**Five: real progress stays, beside the operation it describes.** Download,
preparation and installation progress and the error and recovery screens around
them are not the counter and are not removed with it. *Removing a decoration
must not remove the one thing a person waiting on a disk write actually needs.*

**Six: one selection border — two logical pixels of navy `#102A43`, plus the
word.** The 1.5px edge on the access screens was an unfinished edit. Keyboard
focus stays a separate mark, an outer ring with its own gap, and **moving focus
commits nothing.** Teal may not mean selected or focused; ADR 0067 reserved it
for alo acting, and a selection drawn in teal would say *alo chose this* on the
one screen whose whole purpose is that the person chose it.

## Why the strip is not a translated string

Everything else a person reads during setup is declared as an
`alo_strings::Word` and answered in the language they read. **The strip is not,
because it is a product name, and a translated product name is a different
product.** `alo_letting_go::words` already tells a translator so where it has
to: *"alo OS" is the product's name and is never translated.*

**This is the one exception, and it is narrow.** The word *Optional* above a
heading, every action's name, and every sentence on every screen stay
translated. CLAUDE.md's rule that user-facing strings are externalised is about
sentences a locale changes; a proper noun is not one.

## What this does not decide

**Whether progress is shown at all, and in what form, beyond being out of the
identity line.** A rule, a row of dots or nothing are all open; the design file
currently shows nothing, and that is the designer's to settle rather than a
build's to invent.

**Where `OPTIONAL` went is settled, and was not settled by this ADR.** It moved
out of the strip and into the eyebrow as `Optional · <subject>`, on ten screens
rather than three — more reach than it had before. Measured individually on all
ten; see `docs/design/the-first-start.md`.

**`Not now` on the three access screens is recorded and is not built.** It must
leave without granting or changing permissions, must not commit the alternative
the screen happens to be displaying, and is not the same action as `Back`. The
three levels it has to preserve are a `[v1]` item in `docs/features.md` and the
current release is `v0.01`, so **there is no policy value for it to leave
unchanged.** Writing the action now would mean inventing the policy it is
supposed to preserve. The rule is written down so that whoever builds the policy
inherits it.

**The 6px inset discrepancy on `385:26244`.** Sixty-six labels sit 70px from
the right edge of a 1440-wide frame; that one sits 64px. The responsive frames
also use 64, so either value could be the newer. **A build takes 70 on 1440
because sixty-six frames say so**, and the designer settles it. It is six
pixels on one frame.

## The evidence, and why each rule stands on more than approval

**The counts are from a script over the whole refreshed export**, not from
opening screens. `70-28.xml` is 2,350,573 bytes and nobody read it; the
measurements were computed and the figures `a_palette_with_one_source.rs`
asserts were recomputed by that test's own definitions before being written
into `MANIFEST.md`.

**The one-border rule was tested everywhere at once, using geometry as a proxy
for a style the export does not carry.** A card measures 85 logical pixels with
an ordinary edge and 87 with a selected one, so a surviving 1.5px edge would
measure 86. **No card in the file measures 86 or any fractional height.** Where
the proxy does not apply — `385:26244`, whose four cards are fixed to one
height — the card was read directly and is `border-2` in `#102A43`.

**Each rule in the code was broken on purpose and the test named for it
failed.** Putting `SET UP · 2 OF 5` back failed three tests; giving the canvas a
strip failed one; flattening the selected edge to 1px failed one; marking a
selection in teal failed two; making focus commit a choice failed one. Both
files were then restored byte-identical. A rule whose test has never failed is
a rule nobody has checked.
