# Native egress indicator

Status: additive trusted Rust API in `alo-shell`, 2026-09-14. Law 1; ADR 0002;
`docs/features.md` ★ *the egress indicator lives in the status area*. No
protocol, agent capability, D-Bus interface or stored format changes. What
may leave is `alo-egress`'s decision and what the indicator says is
`alo-indicator`'s; this document describes only the drawing surface.

**Since 2026-10-27 it is also fed by a protocol** ([ADR 0134](../decisions/0134-the-desktop-is-told-what-alo-agentd-sends.md)):
`alo-agentd`'s own indicator, told on the person's socket by
`watch-what-leaves` (`docs/contracts/daemon-protocol.md`). The desktop draws
with `alo_indicator::Indicating::show_both(compositor, &indicator, &daemon)`:
the machine's own lines, then the service's, counted together. While the
service has not told the desktop, or has stopped (`FromTheDaemon::NotYetHeard`,
`FromTheDaemon::Unreachable`), the lamp is **not dark**: it reads *alo OS cannot
tell whether your agent is sending anything — its service is not answering*,
and that sentence is drawn as a row of its own. It never refuses the frame.
Only `alo_indicator::watching` can make the `ToldByTheDaemon` the service's
lines arrive in, from the person's own door after checking that its directory
is theirs alone and that the process listening runs as them.

## Values

`EgressStatus` implements `alo_indicator::Compositor`. One belongs to one
output for that output's lifetime; there is no method that takes a picture
away, dismisses, hides or turns the indicator off.

- `EgressStatus::on_an_output()` accepts every `alo_indicator::Drawn` it is
  handed and keeps the last one.
- `EgressStatus::with_no_output()` refuses every picture with
  `SurfaceRefused::NothingToShowOn`, which `alo_indicator::NotShown::said`
  words.
- `EgressStatus::is_told()` says whether a picture has been accepted.

A picture reaches it only through `alo_indicator::Indicating::show`, which
builds it from the machine's own `alo_egress::Indicator`, or `show_both`,
which adds what `alo-agentd` told. When an output goes
away the host drops its `EgressStatus` and calls `Indicating::show(None, …)`,
whose refusal forgets what was drawn; a new output gets a new `EgressStatus`
and is told on the next call.

`EgressStatusLook { scheme, scale, reading }` is `alo_appearance::Scheme`,
`alo_appearance::TextScale` and `alo_strings::Direction`.

`EgressStatusFrame { status, strings, dock, look }` borrows the status area,
the person's `alo_strings::Strings`, their `alo_dock::Dock`, and the look.

## Nested backend

`Nested::submit_with_egress_status(roots, popups, cursor, controls, labels,
egress)` is `Nested::submit_control_scene` with the indicator drawn after
clients, popups and native controls and before the cursor.

- While the indicator is quiet nothing is drawn: the frame is exactly the one
  `submit_control_scene` submits. A question answered on this machine, by the
  runtime alo OS ships or by a service at this machine's own address, is not a
  departure and draws nothing.
- While something is leaving, one row per `alo_egress::Shown`, in the
  indicator's order: a mark (a navy or charcoal arrow on deep teal inside an
  ink edge) and the line's sentence exactly as `Drawn::lines_said` answers it.
  An agent's egress and alo OS's own errand are drawn alike.
- Rows sit at the status area `alo_dock::Layout` names on this output: against
  the dock's far end, stacked away from the dock, the first row nearest it.
  The mark precedes the words for a left-to-right reader and follows them for
  a right-to-left one.
- When the rows do not fit, as many as fit are drawn and the last row is
  `Drawn::lamp_said`, the indicator's own count, with its mark.

Errors, each refusing the whole frame before anything is drawn:

- `RenderError::EgressStatusUnknown` — the `EgressStatus` was never told what
  is leaving (never shown, or it has no output).
- `RenderError::EgressStatusScene` — something is leaving and the window is
  not a screen `alo_dock::Screen::of` accepts, or is wider or taller than
  16,384 px. A quiet indicator is never refused for size.
- Every error `submit_control_scene` returns.

## Not in this surface

No direct-display (DRM/KMS) submission and no per-display placement; the dock
itself is not drawn. The sign-in screen carries no status area. Nothing here is
reachable by an agent.
