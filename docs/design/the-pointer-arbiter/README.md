# The pointer arbiter, kept here because it cannot land yet

`surface_areas.rs` beside this file **is not compiled and is not in a crate.**
It is kept here so that it is findable, because the alternative was a branch.

## Why it is not in a crate

It arbitrates between **three** surfaces — the put-aside panel, the Dock and
the top window controls — and the top controls do not exist in this
repository. `docs/features.md` promises them **at no tier**, and `CLAUDE.md`
binds building to what that file carries with a tier. So compiling this would
be building an unscoped feature, which is a worse fault than leaving it
parked.

`crates/alo-shell/src/the_panel_reveals.rs` says the same thing from the other
side, and says what happens when the tier arrives:

> **When that arbiter exists it goes in front of this, not inside it.** … When
> they do, this file's caller asks the arbiter first and passes `Elsewhere`
> when the answer is not the panel's. One line, and nothing here changes.

**So the decision that unblocks this is one line in `docs/features.md`**, and
it is the owner's. Nothing else is missing.

## Why it is not on a branch any more

It was on `handover/dev-pc/the-pointer-classifier`, written by the dev-PC lane
on 2026-09-30 and handed to the desktop lane on 2026-10-01. That branch was
deleted on 2026-10-13 while clearing the remote to `main` alone.

The branch's own README gave the reason it should not simply have been
dropped, and it is the reason this directory exists: **work that looks absent
because it is somewhere nobody else can see** is the fault that made the Panel
lane conclude `alo-dock` was abandoned while its owner was landing in it.
`the_panel_reveals.rs` named that branch by path — delete the branch and that
reference sends a reader to nothing. It now points here.

## What landed already, so nobody re-lands it

The handover held two files. **`panel_region.rs` is already in the product**,
as `crates/alo-put-aside/src/the_region_the_panel_claims.rs` — the panel's own
half, which exposes `WhereThePointerIs` and `WhatEachSurfaceSaid` and whose
header says it is there *"for whoever classifies a pointer"*. Only the arbiter
is still parked.

## What it is

`surface_areas.rs` — the four regions a pointer can be in, and the one rule
across an edge. The argument behind it is
`docs/design/the-regions-a-pointer-can-be-in.md`, which **is** in the
repository and is the thing to read first; this is that document as code.

**The panel is asked first and the other two are asked about what is left**,
rather than the three being tried in an order that happens to favour one. A
point inside the panel's reserved area is the panel's even if it also lies
inside the rectangle the Dock was drawn with, which is the owner's rule stated
once.

Containment is half-open: the top-left edge belongs to a rectangle and the
bottom-right does not, so two rectangles sharing an edge do not both contain
the points along it.

## What is unfinished, named so it is not rediscovered

- **No caller**, which is the whole reason it is here. Nothing has told this
  file it is wrong about the surface.
- **`Revealing` has no owner** was the original blocker and **is now resolved**:
  `alo_dock::revealing::Revealing` is held by `alo-desktop` and fed by
  `alo-shell`'s `the_panel_reveals`, and `alo-put-aside` owns the panel's
  region. That half of the handover is done.
- **No figure from the design file may reach the code.** `alo-shell` holds that
  with `no_figure_from_the_design_reaches_the_code`. Whatever crate this lands
  in needs the same check, or the rule stops applying to it.

## One correction the original README carried, kept because it is the point

The module header used to claim the Dock's and desktop's rectangles had the
panel's area *subtracted* from them. They do not. The property the header was
defending is already given by the return type: `whose_area` answers one
`WhoseArea`, so two surfaces cannot both claim a point whatever the rectangles
are — no subtraction needed. What the order decides is *which* surface owns an
overlap, and the grid test is what holds it.

The prose was corrected rather than the code, in four places, each quoting
what it used to say: the module header, the `the_docks_area` field doc, the
grid test's doc and the mirrored-panel test's doc. **The file described its
implementation correctly in one place and wrongly in another, one screen
apart** — which is why the correction is recorded here rather than tidied
away.
