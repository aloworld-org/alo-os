# The regions a pointer can be in

`alo_dock::revealing` consumes a classification — *at the edge*, *on the
surface*, *elsewhere* — and never a coordinate, on purpose. **This file is the
other half**: what a caller must know to produce that classification, and where
each number in it comes from.

Everything here is either measured off the design file, decided by the owner, or
named as unknown. Those three are kept apart, because a design note that reads
as one voice is a note whose next reader cannot tell a measurement from a guess.

## Where the numbers come from

The design file's page *03 — alo OS · Living canvas*, read on 2026-09-30. Its
frames are **1440 × 960**, and every figure below is in that frame's units —
**logical pixels**, the same unit `alo_dock::measures` is written in, so a dense
display draws the same regions sharper rather than smaller.

**A prototype is evidence of intent, not of behaviour.** These numbers say what
the design asks for. They do not say what any code does, and nothing in this file
is evidence that a person can reach anything.

## There are four continuous regions, and that is all of them

The design calls them *continuous hover regions*. The name matters: a region is
the whole area that keeps a surface open, including its controls, its previews
and the path between them, which is exactly what `ThePointer::OnTheSurface`
documents.

| Region | Position | Size |
|---|---|---|
| Top controls | (0, 0) | 1440 × 84 |
| Dock + alo Bar | (304, 838) | 832 × 122 |
| Minimized shelf, collapsed | (1328, 0) | 112 × 960 |
| Peek | (370, 550) | 360 × 386 |

**The fourth one is not at an edge at all.** Peek's region is a free-floating
rectangle in the middle of the screen. So a region is not an edge strip that
happens to be thick — it is an area, and a surface whose menu opens outside its
own strip is ordinary rather than a case somebody has to invent a rule for.

## Across the edge, one rule

A surface is inset from the edge it belongs to, and its region covers that inset
and runs flush to the edge.

- The **Dock bar** is 784 × 76 at (328, 858), so it stops 26 above the screen
  edge at 960. Its region runs to 960.
- The **collapsed shelf** is 64 × 232 at (1352, 96), so its right face stops 24
  short of the screen edge at 1440. Its region runs to 1440.

So: **24 towards the interior, flush on the edge side.** Written once because it
is one rule, and the gap under the bar being 26 rather than 24 is the only place
the design disagrees with itself by a pixel.

**This is what makes travelling onto a revealed surface possible at all.** If the
region stopped where the surface stops, the strip between the surface and the
edge would belong to nothing, and a pointer coming from the edge would cross
dead ground on its way to the thing it is reaching for.

## Along the edge, it depends on whether the surface's extent varies

Measured across every instance in the file:

- **Dock + alo Bar: 784 × 76 at (328, 858), in all 163 instances.** Not one
  differs — including the frames for several windows of one application and for
  overflow, which are where a bar computed from its contents would have shown
  itself. It is pinned, with an overflow area rather than a growing width.
- **Collapsed shelf: 64 wide at x=1352 always, height 176, 232, 288, 296 or
  344** depending on how much is put aside.

And their regions follow that difference rather than their own extents: the
bar's region is the bar plus 24 each side; **the shelf's region is the whole 960,
while the shelf itself never exceeds 344.**

So the rule is: **the region spans the whole edge when the surface's extent along
it varies, and the surface's extent plus a margin when it does not.**

**Why, and this is the Panel lane's argument rather than an observation.** A
region defined as *surface plus margin* on a surface that grows would change size
as a person put windows away — and a hit target that changes size under a pointer
travelling towards it is the fault `waiting_on_the_person` already refuses for the
panel's ordering. Taking the whole edge makes the region a constant with respect
to the surface's contents. It is the same shape as
[when the dock gives way](when-the-dock-gives-way.md): the band is computed
whatever the person chose, so the predicate cannot be fed by its own answer.

## The corner, decided rather than inferred

The design as drawn gives the top controls the full 1440, and the shelf's region
the full 960 at x=1328. **Those overlap**, over 112 × 84 in the top-right corner,
and the frames cannot say which surface owns it. The bottom Dock stops at 1136
and does not reach the panel at all.

**The owner settled it on 2026-09-30, in these words:**

> Make the top controls region stop before the right panel. The overlap is a
> geometry issue we should resolve explicitly.
>
> Use a shared layout rule:
>
> - **Top controls:** span the screen up to the reserved right-panel area.
> - **Right panel:** owns that area, including the top-right corner.
> - **Bottom Dock:** stops before the same area, as it already does.
>
> The reserved width follows the panel's current expanded or collapsed width.
> Activation strips and pointer paths must follow those bounds too, so one
> pointer position cannot reveal two surfaces.
>
> For a full-screen video, reaching the top edge reveals **Return to canvas**;
> reaching the right edge reveals minimized windows. At the shared corner, the
> right panel owns the interaction.
>
> That gives each edge one predictable purpose and prevents controls from
> covering each other.

**So the top region as drawn is the loose one**, and 1440 × 84 is superseded by
that rule rather than recorded as the answer. The bottom already complies, which
is why it stops 192 short of the panel.

### The invariant that falls out of it

*One pointer position cannot reveal two surfaces* is the owner's sentence and it
is testable: **for any point on the screen, at most one surface claims it.**

It is to be held over a grid of points and **at both panel widths** — a rule
about overlap tested only at the collapsed width is tested where it cannot fail,
because the top region is widest and the corner furthest away exactly there. The
width that matters is the one the panel reaches with enough put aside.

### And the bound moves, which is allowed

The reserved width follows the panel's width, so the top and bottom bounds move
when the panel expands. That is required by the rule rather than a wrinkle in it.

The property worth holding is not *bounds never move*. It is that **a moving
bound hands its area to the surface that moved it.** When the panel expands, the
strip a pointer might have been in stops being top-controls region and becomes
right-panel region — claimed by something, and by the surface the person just
expanded. The fault named above is the other case: a region that shrinks because
its own contents shrank vacates into *nothing*, and a pointer travelling towards
a target watches the target stop existing with nothing taking its place.

That distinction is the Panel lane's. An earlier version of this section
distinguished the two cases as *deliberate* against *incidental*, which is about
the person's intention and is therefore not a property the geometry has, nor one
a test can read.

## The keyboard road is designed, not inherited

The file carries **`Action / Switch interface regions`** with a matching
**`Shortcut / Switch interface regions`**. Reaching these surfaces without a
pointer is a named action with a key behind it, rather than focus traversal that
happens to arrive somewhere.

This matters to whoever writes the classifier: **a classifier that only mapped
pointer positions would quietly make the keyboard road second class**, which is
the thing `alo-dock`'s own header refuses when it says peeking is not a pointer
gesture with a keyboard consolation. `Revealing::the_keyboard` and
`FocusGoes::BackToTheWindow` are the machine's half of this; the shortcut is the
person's.

## Hit areas are 29 to 52, and the floor is a floor

Every hit area in the file: 29, 40, 42 and 44 square, a 52 × 48 focus area, and
a 183 × 44 row.

`measures::SMALLEST_TARGET` is 24, from WCAG 2.5.8 by way of EN 301 549. **None
of the design's targets is near it.** The floor is what a layout may not go
below, not a size to build to, and the first person to size a control to 24 will
believe they were being compliant.

## Three edges is three surfaces, not a movable Dock

Top, bottom and right reveal three *different* surfaces, each with its own
instance of the same machine. That is what *each surface gets its own instance*
was for, and it is the strongest available evidence that the generalisation in
`revealing.rs` was the right shape.

[ADR 0076](../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
is untouched by it. The Dock is still fixed to the bottom edge. **An edge is a
fact about a surface, not a setting offered to a person**, and a design that
gives three surfaces three edges is not the Dock becoming movable.

## These numbers are at one size, and what carries across is the rule

Every figure above was read off a 1440 × 960 frame. **No machine is obliged to
be 1440 × 960**, and `CLAUDE.md` forbids building to one screen size, so a number
here is only usable once it is known which kind of number it is.

**Fixed, in logical pixels.** The 24 between a surface and its region's edge is
about a pointer having slack on its way to a target. Slack is ergonomic and does
not get more generous because somebody bought a wider monitor, so it is the same
24 everywhere, and it is `3 × measures::MARGIN` rather than a number of its own.
The hit areas are the same kind: a target is sized for a hand.

**Derived from the display.** The Dock's bar and the shelf's rail are not. A bar
784 wide is 54% of a 1440 frame and would be absurd copied onto a 3840 one —
`measures::A_DOCK_MAY_TAKE_ONE_PART_IN` already says what governs instead, and
`dock_raster` already clamps the bar to the screen less its margins.

**Flush is flush at every size.** *Runs to the screen edge* and *spans the whole
edge* are rules rather than measurements, which is why the shelf's region being
"960" is really "all of it" and survives a screen of any height unchanged.

So: **take the 24 and the target sizes as constants; take everything else as the
rule that produced them at this size.** A region hard-coded to 832 × 122 is the
fault the standing rule names, and it would be invisible on the machine it was
written on.

## Where the code and the design already disagree

Recorded rather than fixed here, because changing a measure is a code change and
belongs in its own gate. Both were found by reading the frames against the code
on 2026-09-30, within minutes of the standing rule existing.

**The gap under the bar: the code says 8, the design shows 26.**
`measures::FLOATING_ABOVE_THE_EDGE` is `MARGIN`, which is 8, and `dock_raster`
places the band at `height - thickness - floating`. The design's bar ends at 934
on a 960 frame. That is a plain numeric disagreement and, by the standing rule,
the design is right until an ADR says otherwise. It also matters more than it
looks: that gap is inside the Dock's region, so it is part of the ground a
pointer crosses.

**The bar's width: the code grows it, the design draws it fixed.**
`dock_raster` computes the width from what the Dock holds and clamps it;
the design shows 784 in every instance and carries a separate frame for
*overflow*. **This one is not settled by the measurement.** It was reported
between lanes as settled, and that was too confident: instances of one component
share a size unless somebody overrides it, so 163 identical instances may be
evidence of one component rather than of a pinned bar. The section above, which
uses the bar's constant size against the shelf's varying one, rests on that
reading — so if the bar turns out to grow, the *rule* still holds and only which
side of it the bar falls on changes. What can be said from the frames alone is
that an overflow frame exists, which is what a design that does not grow the bar
needs and a design that grows it does not.

## What this does not settle

**One shelf instance sits at x=56 instead of x=1352** — same 64 × 232, mirrored
to the left edge, one out of 156. It may be right-to-left mirroring, or a stray.
Nobody has asked, and this file does not guess.

The consequence does not depend on the answer: **if a surface's edge can ever
differ, that edge is data.** A classifier that names *right* anywhere has to be
rewritten rather than parameterised if the answer turns out to be mirroring —
which is the correction `revealing.rs` itself went through, where the file was
general about geometry and specific about something it never mentioned.

So the edge is taken as given. That costs nothing if the instance is a stray.
