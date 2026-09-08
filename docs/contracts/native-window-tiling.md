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
Trusted tile/restore transactions are described below, with shared maximize
normal-geometry memory, tiled XDG states, latest-response placement and
output/lifetime retirement. Native controls follow those operations.
Linked-neighbour resizing, corner snapping and remembered arrangements remain
v0.5 scope. Evidence: `../autonomy/updates/bounded-window-tiling-geometry.md`.

## Trusted tile and restore transactions

Implementation status: verified after owner-authorized recovery on 2026-09-08.
Real-client and graphical evidence is recorded in the task report below;
rendered native controls remain separate integration work.

`Server::set_window_tiled(surface, Some(TileSide::{Left, Right}))` now requests
an actual tile; `None` restores normal geometry, including from maximize. The
existing immutable planning API remains side-effect free. Transactions return a
fresh configure serial, or `None` for an unchanged valid mode. Refusals use
`WindowModeError`: `Unmapped`, `Busy`, `OutputUnavailable`, `Geometry`, or
`Tile(TileGeometryError)` for exact-tile planning failures. `WindowMaximizeError`
remains its original, separate four-variant enum, not an alias to the larger
type. Existing variant imports and exhaustive matches remain source compatible;
maximize callers do not acquire a tile-only error variant.

The first non-normal request captures normal geometry once per buffer mapping.
Side switches, maximize transitions, output changes and rapid restore/re-enter
requests retain it. Restore clamps the saved dimensions to committed current
hints and retains the original origin. Both restore entry points share this
policy, including XDG client unmaximize. No output is required to restore.

Tile configures set all four `tiled_*` states: every edge abuts either the output
boundary or the split. Maximize clears those states and sets `maximized`; normal
clears both. Unrelated activation flags are preserved. No tile protocol request
or WM capability exists to advertise. Input, focus, stacking and visibility do
not change on request. Existing move/resize/popup grabs refuse with `Busy`.
While any mode memory or restore boundary remains, interactive move/resize is
refused and exact sizing/placement returns the existing `Maximized` variant,
whose meaning now includes tiling. Native controls remain integration work.

Only a root commit with a serial at least as new as the latest request can
apply an anchor. Acknowledgement alone and stale responses cannot move pixels.
Right tiles anchor their *actual effective geometry* at the output's right edge;
left tiles anchor at (0,0). Shadow offsets are preserved; no buffers are scaled.
Subsequent geometry commits keep that anchor while the tile is valid. Actual
nonconforming sizes remain visible as real pixels, including normal clipping.

Commit-time incompatible limits or unsupported actual dimensions invalidate the
anchor without erasing original normal geometry. Merely reverting hints cannot
revive an old response: an explicit valid tile request obtains a fresh serial.
Successfully changed output extents also reconfigure eligible tiles. Unsupported
outputs, incompatible new output sizes and successful retirement suspend old
anchors. Failed submission/retirement preserves them. A valid replacement output
gets a fresh boundary; output changes cannot supersede an in-flight normal
restore. No timeout, forced client resizing or engine patch is introduced.

Minimized mappings preserve mode memory and accept commits while hidden; new
trusted tile/restore requests refuse until revealed. Revealing does not steal
focus. Unmap/disconnect discards all mode memory and serial authority; remapping
starts normal under the existing fresh handshake.

Evidence: `../autonomy/updates/trusted-window-tiling-and-restoration.md`.
This completes trusted transactions only. Rendered controls, dock work areas and
configurable operation dispatch remain; no window-management/release tick.
