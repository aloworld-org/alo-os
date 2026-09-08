# Native window minimization and restoration

`alo-shell::Server::set_window_minimized(surface, bool)` is trusted native shell
plumbing under ADR 0002, not an agent verb or application-adapter endpoint. It
returns `Ok(true)` for a visibility change and `Ok(false)` for an unchanged
request. Foreign, dead, child, popup and unbuffered roots return
`WindowMinimizeError::Unmapped` before changing any scene or input state.

The target must belong to this display and have completed the XDG mapping
handshake. No keyboard or submitted output is required. A minimized root retains
its role, buffers, committed placement and maximize/restore transaction memory.
Subsequent buffer commits cannot reveal it. Unmap clears minimization immediately;
remapping requires the existing fresh handshake and starts visible. Disconnect
retires both mapping and hidden state.

`mapped_surfaces()` is the visible renderer/input list: minimized roots and their
surface trees are excluded. `minimized_surfaces()` enumerates hidden buffered
roots in stacking order for trusted restore controls. Neither exposes metadata
or agent context. A hidden window receives no submitted frame callbacks; pending
callbacks can complete after restoration and successful submission. Output
membership still follows successful submission, not the visibility call alone.

Hiding the focused root releases held keys and clears XDG activation. Its popups
are dismissed, held pointer ownership is retired, and its move/resize transaction
is cancelled, including pending resize anchoring. A later physical release cannot
be replayed into another window. A new pointer motion is required to select a
recipient after retirement. Unrelated roots retain keyboard/input ownership.

Restoring reveals the current committed buffer in its existing stacking position;
it neither raises nor activates. Native controls may explicitly activate after
restoring. Direct focus/raise/geometry operations refuse hidden targets through
their existing unmapped-target errors. Cycling skips hidden roots while retaining
their positions in the mapping-order ring; unmap/disconnect removes them. This
keeps restoration separate from selection and avoids unsolicited focus changes.

Hidden maximize memory is intentionally retained on buffered lifetime, not visual
lifetime. A pending acknowledged maximize response may commit while hidden and
output changes still refresh its requested extent. Client maximize requests while
hidden are refused with the existing configure-response policy. Maximization
restoration remains distinct from minimization restoration.

## Client requests

XDG `set_minimized` uses the same transition for the requesting mapped role.
No seat serial or focus is required: an application may hide its own window,
including an inactive window, without authority over another client's role.
Initial and remapped configurations advertise Maximize and Minimize only.

Requests before a buffered mapping are ignored, both before and after initial
configuration. They do not send a configure or retain intent for a future map.
XDG defines no minimized state, acknowledgement or client unminimize request;
duplicate requests are inert. Focus retirement can still send an ordinary
activation configure. Only trusted restoration reveals a hidden mapping;
buffer commits, maximize requests and duplicate minimize requests cannot do so.
Unmap/remap and disconnect retain the existing lifecycle reset.

No new surface text, agent API, upstream patch or release scope is introduced.
Rendered controls remain integration work. Protocol, input and GLES evidence:
`../autonomy/updates/client-window-minimize-requests.md`. WSLg fixtures do not
certify physical DRM/seat operation or release hardware acceptance.
