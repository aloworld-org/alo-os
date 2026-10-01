# The pointer classifier, handed over rather than described

**These two files are not in a crate on purpose, and this branch is not for
merging.** They were written against the real surface on 2026-09-30 and held out
of the repository because `Revealing` had no home: the classifier answers *which
region is the pointer in*, and one of its answers is a state that belongs to
whichever crate owns the reveal machine.

The desktop lane asked for them on 2026-10-01 rather than write a third version
from the region module's signature, and the reason it gave is the right one:
**work that looks absent because it is somewhere nobody else can see** is the
fault that made the Panel lane conclude `alo-dock` was abandoned while its owner
was landing in it. A rebuild would have been that fault with the roles swapped.

## What they are

`surface_areas.rs` — the four regions a pointer can be in, and the one rule
across an edge. The argument behind it is
`docs/design/the-regions-a-pointer-can-be-in.md`, which **is** in the repository
and is the thing to read first; this is that document as code.

`panel_region.rs` — the put-aside panel's own region, derived from
`alo_dock::measures` rather than from any figure in the design file.

## What is unfinished, named so it is not rediscovered

- **`Revealing` has no owner.** That is the whole reason these are here and not
  in a crate. The reveal machine's rules live in `alo-dock`'s revealing module
  with nothing calling them; the desktop lane's view is that they should have
  been in its crate from the start, which is a judgement for whoever takes this.
- **No caller.** Nothing consumes the classification yet, which means nothing
  has told these files they are wrong about the surface.
- **No figure from the design file reaches the code**, and that is held by a
  source check in `alo-shell`
  (`no_figure_from_the_design_reaches_the_code`). Whatever crate these land in
  needs the same check or the rule stops applying to them.

## Provenance

Written by the dev-PC lane. Handed to the desktop lane on 2026-10-01 at its
request, with the dev-PC lane claiming the task back from it earlier the same day
for exactly the reason it should not have: *I have material for it*. Having
material is a reason to hand the material over, not to hold the task.
