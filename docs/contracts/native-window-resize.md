# Native window resize geometry

Status: additive trusted Rust shell API, 2026-09-08. This is a component of
v0.01 window management under ADR 0002, not an agent verb or a Wayland protocol
extension. The application-adapter and agent-verb contracts are unchanged.

`Server::window_resize_geometry(surface, edge)` captures a live mapped root
owned by that display. Children, popups, foreign roots, unmapped and dead roots
refuse with `ResizeGeometryError::Unmapped`. Geometry uses the same committed
surface-tree intersection as scene placement. Pending geometry, client limits,
configure suggestions and acknowledgements alone do not change it.

`ResizeEdge` enumerates the eight sides/corners. There is no unspecified or
arbitrary bit-mask edge. The XDG resize handler validates known edge values;
`None` and unknown wire values are ignored without taking pointer ownership.

`ResizeGeometry::requested_size(delta)` uses total logical pointer displacement
from the initial position. Round each delta to the nearest pixel (half away from
zero), with no cumulative rounding. Selected axes clamp to the captured client
min/max and the range 1..=1,000,000. Crossing an opposite edge stops at the minimum;
it does not flip the edge. Zero client limits mean unconstrained. Unselected axes
keep their initial dimensions and refuse if those cannot satisfy the limits.
Impossible limits refuse with `ClientLimits`. Non-finite or excessive deltas
refuse with `Delta`, including on an unselected axis.

Clamping here expresses a drag's nearest allowed size. The existing exact
`request_window_size` API still refuses client-limit violations without clamping.
Callers must revalidate current committed limits when applying suggestions;
this immutable snapshot does not observe later limit changes.

`committed_origin(actual_size)` calculates placement from the client's actual
committed geometry dimensions, which may differ from the suggestion. Top/left
edges preserve the initial bottom/right coordinate; other axes keep their
initial origin. Advertised min/max do not reject a client's actual choice.
Nonpositive or excessive dimensions refuse with `Size`; excessive placement
refuses with `Geometry`. Snapshot origins and dimensions are bounded too.

These methods only return data. They send no configure, move no pixels and
consume no input. A snapshot carries no surface handle or authority.

## Interactive XDG resize transactions

The shared nested/direct input implementation accepts a resize only for this
seat's active held pointer press on the requested mapped root or its subsurface
tree. Foreign targets, stale/forged/released serials, missing pointer capability,
active popup/move/resize ownership and invalid geometry refuse before any
configure or input consumption. Resize and move share the same authority check.

Acceptance balances the client's held buttons and clears pointer focus before
taking ownership. Keyboard focus and stacking do not change. Motion uses the
fixed initial pointer/geometry and current committed min/max constraints;
pending constraints have no effect. Duplicate sizes suppress duplicate pending
configures. Invalid motion returns `InputError::InvalidPointer` without changing
the transaction; impossible live constraints cancel it. Button release refreshes
constraints too, so a final suggestion cannot silently reuse stale limits.

Accepted requests configure the XDG `resizing` state. Acknowledgement alone never
moves a window. On a mapped root commit, Smithay's committed configure serial
must be at least the first resize serial before its actual geometry is anchored.
The client may choose a different size. A pre-resize acknowledgement cannot
authorize anchoring, even if the client changes its buffer. Geometry-only root
commits after acknowledgement count as committed responses; no new buffer is
required if the client keeps its existing storage.

Additional buttons are consumed; the last release sends a final configure with
`resizing` cleared and restores ordinary pointer routing. Anchoring remains until
a commit acknowledges that final configure or a newer configure. Older resize
responses can still anchor while it is pending. After completion, spontaneous
size changes no longer reuse the anchor. A new move/resize is refused while the
previous final response remains outstanding. There is no timeout, forced client
size, synthetic response or allocation based on the requested size.

Pointer leave cancels both active and final-response state, clears `resizing`
on a still-mapped client and abandons future anchoring. Unmap/disconnect retire
the transaction without configuring a dead mapping; remap requires its normal
fresh handshake. Impossible live limits or out-of-range committed geometry also
abandon anchoring. These are trusted native input operations, not agent verbs.
Tests and WSLg pixel evidence are recorded in
`docs/autonomy/updates/interactive-resize-transactions.md`; direct hardware
acceptance and remaining window operations are separate delivery work.
