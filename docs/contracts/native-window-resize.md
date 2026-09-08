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
arbitrary bit-mask edge. Protocol integration must validate protocol edge values
before constructing this type; no XDG resize request handler is added here.

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
consume no input. A snapshot carries no surface handle or authority. Native
callers must validate seat, held press serial, target tree and mapping lifetime,
then implement resizing configure state, acknowledgement/commit ordering and
cancellation. Placement must follow a committed buffer/geometry, never a size
suggestion or acknowledgement alone. That interactive lifecycle remains the
next component, with its own integration tests; it is not certified by this API.
