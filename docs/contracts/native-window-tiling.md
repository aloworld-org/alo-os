# Native window tiling geometry

Status: additive trusted Rust planning API, 2026-09-08, under ADR 0002 and the
v0.01 window-management scope. No agent verb or application-adapter API is added.

`Server::window_tile_geometry(surface, TileSide::{Left, Right})` returns an
immutable `TileGeometry`. It accepts only this display's visible, live mapped
XDG root with effective geometry supported by native resize snapshots. Foreign
roots, children, popups, minimized, unmapped, pre-buffer and dead roots refuse
with `Unmapped`. Unsupported effective tree geometry refuses with
`Geometry(ResizeGeometryError)`. Invalid targets never acquire a plan.

The plan uses the last successfully submitted scale-one output, sharing the
existing maximize output lifecycle. Failed submissions/retirement cannot replace
it; successful retirement makes new plans unavailable until another submission.
Output width must be 2..=1,000,000 and height 1..=1,000,000; missing/unsupported
extents refuse with `OutputUnavailable`. Dock work areas remain integration work.

`requested_size()` divides the output width in half, assigning an odd remaining
pixel to the right. Both halves retain full output height and cover the output
without gaps. Each exact size must meet currently committed client minimum and
maximum hints; zero maxima are unbounded. An incompatible size refuses with
`ClientLimits`, rather than clamping into an overlapping layout. Pending hints
do not apply. This differs deliberately from maximize's permitted hint override:
a useful exact tile needs to fit its region. No engine patches or ADR changes.

`committed_origin(actual_size)` calculates placement only, anchored at the top
and selected outside output edge. Left returns (0,0); right returns
(output_width - actual_width, 0). Actual dimensions must each be 1..=1,000,000
or it refuses with `Size`. It accepts bounded nonconforming client sizes without
scaling them or pretending they match the requested tile. A too-wide right tile
can extend left of the output; the existing scene clips real pixels normally.

Snapshots hold no role, serial or live authority. Old snapshots remain immutable
after changes to output, limits or mapping lifetime. A future transaction must
recapture/revalidate and wait for the proper configure acknowledgement and root
commit before placing actual geometry. Calculations perform no configure,
placement, focus, stacking, hiding, input consumption or buffer allocation.
Normal keyboard/pointer routing continues. No tiled state or normal-geometry
memory is established by this API.

This completes the planning component, not tiling/restoration as a feature.
Next are trusted tile/restore transactions with shared maximize normal-geometry
memory, tiled XDG states, latest-response placement, output/lifetime retirement
and real-client/GLES boundary tests. Native controls follow those operations.
Linked-neighbour resizing, corner snapping and remembered arrangements remain
v0.5 scope. Evidence: `../autonomy/updates/bounded-window-tiling-geometry.md`.
