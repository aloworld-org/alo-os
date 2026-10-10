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
that rule rather than recorded as the answer.

**And 84 is an overlay envelope, not a band taken out of the canvas** — see the
owner's ruling at the end of this document. The bottom already complies, which
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

**The bar's width: answered, and the code was closer than this note first
guessed.** The owner settled it on 2026-09-30:

> The Dock grows with its icons. The fixed width in Figma represents one state,
> not a permanent size. It expands until it reaches the screen margins and the
> shelf's reserved area, then icons shrink slightly — from 32 to a minimum of 28
> logical pixels — while click targets remain at least 44 × 44. After that,
> additional apps use **More apps** overflow. The alo Bar's text stays the same
> size.

So `dock_raster` growing the bar from its contents is right, and 163 identical
instances were a property of the file rather than of the design — which is what
the retraction it replaced had already allowed for.

**But it takes the contrast the along-the-edge rule rested on.** That rule was
read off exactly one difference: the shelf's extent varies and the bar's does
not. Both vary. So the drawn 832-wide region is one state's region in the same
way 784 was one state's bar, and the rule has nothing left to rest on as stated.

What survives is the *reason*, which was never about which surface was which: a
region that grows with its surface is a hit target that changes size under a
pointer travelling towards it. Applied to a Dock that grows, that says its region
must **not** be the bar plus a margin — and the owner's corner rule already says
where it ends, at the reserved area. Put together, **the bottom region is the
bottom edge up to the reserved width, constant with respect to what the Dock
holds.**

*That is a derivation and not a measurement*, and it is the one place in this
file where the drawn geometry and the reasoning disagree. It is flagged rather
than settled here.

**The icon and the target are one number in the code and two in the design.**
`measures::ICON` is 48 and `places.rs` says a place is *always* `ICON` across, so
the glyph and the thing a person presses are the same value. The owner's answer
has them apart: the glyph is **32**, shrinking to **28** when the bar runs out of
room, while the target stays at least **44 × 44**. Every hit area measured in the
design agrees — 29, 40, 42, 44, 52 — and none is 48.

**`holding.rs` reads as a contradiction and is better than that.** It says *the
size is never traded away: shrinking icons to fit more is how a dock becomes
unusable at exactly the moment a person has the most open*, held to
`SMALLEST_TARGET`. The owner's rule does shrink — but only the glyph, and only by
four, while holding the target at 44, which is nearly twice the 24 floor that
sentence was protecting. **The policy it defends is upheld by separating the two
numbers**, which is the thing the file could not say while it had one.

So what is owed is not a constant changed from 48 to 32. It is a distinction
introduced: a place has a glyph size and a target size, the first may shrink one
step and the second may not shrink at all, and `SMALLEST_TARGET` stops being the
floor the icon is held to and becomes the floor the *target* is held to.

## What this does not settle

**The shelf drawn at x=56 is a stray**, and the test for it was the owner's:
it is a right-to-left variant only if its screen is explicitly marked so.

Measured. It sits inside **`33 · Keep controls visible`**, which is an
accessibility screen and carries no such marking — and the words *RTL*,
*right-to-left*, *mirror*, *Arabic* and *Hebrew* do not occur anywhere on the
page at all. So it is a stray and is not part of the active design, and nothing
is to be inferred from it.

**The rule it was nearly used to justify holds anyway, on the owner's word rather
than on that instance.** The default shelf is on the right; a right-to-left
layout mirrors it to the left **together with the Dock and the top-control
exclusion regions**, so the whole set flips rather than one surface. And the
standing instruction is the one this file was already following for a different
reason: **the edge is layout data, never inferred from a coordinate.** A
classifier that reads *this x is large, therefore right* has decided a language's
direction from arithmetic.

## 84px overlay envelope; no reserved desktop band

**The owner's ruling of 2026-10-09**, which settles a conflict between this
document and the design file's *Build contract · v0.01*. That contract says
*top controls are system overlays, not a reserved desktop band*; this document
recorded `1440 × 84` in a table of regions; and `crate::top_controls_region`
read the second as licence to take 84 logical pixels out of the canvas.

Their words, and each sentence governs something different:

> **Keep the canvas full-size.** The top controls are a system overlay. They
> must not push windows down, shrink the canvas viewport or leave a permanent
> empty band.
>
> **Retain the 84px region as the documented overlay envelope.** Its existence
> does not mean those pixels are permanently occupied or that the entire
> rectangle should intercept input. Hidden controls must not block applications
> beneath them, apart from the defined reveal trigger. Visible controls receive
> input within their actual interactive bounds.
>
> **Keep *Return to canvas*.** Moving window controls onto the external edge
> does not remove this navigation action. Remove duplicated window controls
> from the top overlay. Preserve the agreed separation between the top overlay,
> Dock and side panel.

### So 84 is three different things, and only one of them is a band

The figure survives. What changes is what it is a figure **for**, and the three
must not be collapsed:

| | what it means | what it may do |
|---|---|---|
| **viewport layout** | nothing — the canvas is the whole display | takes **no** room, ever |
| **reveal region** | where a pointer brings the overlay out | intercepts only the defined reveal trigger while concealed |
| **visible control targets** | where each drawn control answers a press | intercepts within its own bounds, not the envelope's |

**A hidden overlay must not block an application beneath it.** An 84-tall
rectangle that swallowed presses while nothing was drawn in it would be a
window a person can see and cannot click, which is worse than a band that
merely wastes room.

### What the overlay still carries

**Return to canvas**, which is navigation and is not a window control. It stays.

**Not** minimise, maximise or close — those moved to
`the-external-window-edge.md`'s strip, attached to the window, and a copy in
the top overlay would be the duplicate surface that whole replacement exists to
prevent.
