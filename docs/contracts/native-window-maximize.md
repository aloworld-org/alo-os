# Native window maximization and restoration

Status: additive trusted Rust shell API, 2026-09-08; v0.01 window management
under accepted ADR 0002. No agent verb or application-adapter endpoint is added.

`Server::set_window_maximized(surface, bool)` accepts only this display's live
visible mapped toplevel root. It returns the fresh XDG configure serial, or `None` for
an unchanged request (including restore of a normal window). Children, popups,
foreign, minimized, unmapped and dead roots refuse with `Unmapped`. Minimization
preserves existing normal-geometry memory and pending response boundaries;
see `native-window-minimize.md`. Outstanding pointer
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

## Client requests

XDG `set_maximized` and `unset_maximized` requests use the same validated
transactions for the requesting role only. Initial configuration advertises the
Maximize and Minimize WM capabilities, including after remap; unsupported
fullscreen and window-menu capabilities are not advertised. Minimize policy is
defined in `native-window-minimize.md`. No focus or input serial is
required: this is an application's cooperative request about its own window,
not authority to operate another application's window or an agent verb.

Every request after the initial configure receives a fresh configure, even when
unchanged or refused. Refusal retains the latest requested state and size, so a
busy resize or pending restore is not overwritten. A response serial alone is
not acceptance; the returned state describes the decision. The trusted Rust API
continues to return errors/None without sending redundant configures.

Pre-map maximization is declined because there is no committed normal geometry
to save. Requests before the first empty commit produce no premature configure;
the normal initial configure answers them together. Requests after that handshake
but before a buffer maps receive a normal-state configure. Intent is not queued
for later automatic execution: the client may request again once mapped. Unmap
discards geometry and intent and requires the same fresh initial handshake.
This uses XDG's explicit compositor-policy discretion, not a fabricated restore
size. No upstream engine changes or new product scope are introduced.

Rendered controls, dock work areas and configurable keyboard dispatch remain
integration work. Tests and exact evidence limits:
`docs/autonomy/updates/trusted-window-maximize-and-restore.md` and
`docs/autonomy/updates/client-window-maximize-requests.md`.

## Shared tile transactions

Integration status: verified after owner-authorized recovery on 2026-09-08;
see the trusted tiling task report for exact checks and evidence limits.

Normal-geometry memory and response ordering now also cover trusted tiling.
Maximize clears all tiled states; unmaximize restores original normal geometry
even after side changes. `WindowMaximizeError` remains its original separate
four-variant enum, including variant imports, exhaustive matches and error text.
The new tile entry point uses `WindowModeError`; it does not widen the existing
maximize error contract. Existing
`Maximized` sizing/placement refusals also cover tiled/restore memory. See
`native-window-tiling.md` for the additive API and precise output/commit rules.
