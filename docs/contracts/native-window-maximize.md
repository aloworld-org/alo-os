# Native window maximization and restoration

Status: additive trusted Rust shell API, 2026-09-08; v0.01 window management
under accepted ADR 0002. No agent verb or application-adapter endpoint is added.

`Server::set_window_maximized(surface, bool)` accepts only this display's live
mapped toplevel root. It returns the fresh XDG configure serial, or `None` for
an unchanged request (including restore of a normal window). Children, popups,
foreign, unmapped and dead roots refuse with `Unmapped`. Outstanding pointer
move/resize transactions, including resize's final-response interval, or popup
grabs refuse with `Busy` before any state change.

Maximization uses the last **successfully submitted** single-output extent at
scale one, with geometry origin (0,0). No render, failed submission, successful
output retirement or unsupported dimensions means `OutputUnavailable`. Both
dimensions must be in 1..=1,000,000. Pending popup constraints or a backend's
unsubmitted desired size cannot authorize maximization. There is no reserved
dock work area yet; that belongs to rendered desktop control integration.

The first request saves the committed effective normal window geometry, bounded
by the existing resize snapshot rules, not a pending size suggestion. Repeated
maximize, output changes and rapid maximize/restore toggles never overwrite it.
Restore clamps the saved normal dimensions to currently committed client min/max
limits and the supported range, preserving the saved geometry origin. Impossible
limits refuse with `Geometry(ClientLimits)`; invalid initial geometry also
refuses without sending a configure or retaining a snapshot.

Maximize suggests the entire output even when client normal-size hints disagree.
This is a deliberate native window policy: XDG permits ignoring size hints, and
the maximized state asks the client to match the supplied window geometry.
Restore uses normal limits again. Neither mode allocates, scales or fabricates
client buffers. Even a client that does not comply is rendered with its actual
committed pixels. No timeout or forced application termination is implemented.

The configure preserves unrelated XDG flags and sets/unsets `maximized`. No
focus, activation, stacking, keyboard routing or placement changes on request or
acknowledgement alone. Only a root commit with a committed configure serial at
least as new as the latest transaction may set placement. An old response after
an output change or mode toggle cannot apply an obsolete origin. Geometry-only
commits count; storage replacement is not required. Placement uses committed
window geometry to distinguish its origin from buffer shadows/subsurface bounds.

After a new output extent submits successfully, each requested-maximized window
gets a fresh configure. Failed submission/retirement preserves the old state.
Successful retirement invalidates outstanding maximize placement while retaining
normal geometry; the next successfully submitted output reconfigures it. Restore
remains available without an output. Retiring an output does not cancel a restore
response. Unmap/disconnect discards all mode memory and pending placement; remap
requires the existing fresh XDG handshake.

While maximize/restore memory exists, trusted `place_window` and
`request_window_size` refuse with their new `Maximized` error variants. XDG move
and resize requests cannot take held-press authority for that root. This avoids
two operations racing to overwrite saved normal geometry. Restoration's committed
response retires that restriction. Other windows and ordinary typing continue.

The component is the trusted transaction API, not the complete maximise feature:
client maximize/unmaximize request policy, rendered controls, dock work areas and
configurable keyboard dispatch remain integration work. Tests and exact evidence
limits: `docs/autonomy/updates/trusted-window-maximize-and-restore.md`.
