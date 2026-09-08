# Focused native window commands

Status: additive trusted Rust shell API, 2026-09-08, under ADR 0002 and
v0.01's configurable shortcuts/window management. No agent endpoint is added.

`Server::dispatch_window_command(&Shortcuts, Chord)` resolves the supplied
current bindings once for layout actions. Unbound or conflicting bindings return
`Ok(None)` without effects. Accepted commands return `Ok(Some(action))`, including
unchanged snap requests; this acknowledges command handling, not client completion.

- `MinimiseWindow` hides the actual keyboard-owning root, retiring its input and
  popups without selecting a replacement. Hidden windows cannot be targeted again
  through stale focus. Restoration remains an explicit trusted visibility action.
- `MaximiseWindow` toggles the latest requested mode, including pending replies.
  Maximized becomes normal; normal or tiled becomes maximized. Saved original
  geometry survives rapid toggles. A refusal does not flip requested intent.
- `SnapLeft` and `SnapRight` request the named independent output half. Repeating
  the command does not restore normal geometry or create linked-neighbour tiling.
- Close and next/previous window delegate to `dispatch_window_shortcut`, preserving
  its exact close/cycling behavior. Remaining actions return wrapped `Unsupported`.

Layout commands require a keyboard seat and its actual visible mapped owning root,
including a popup's root. They never choose a target by stacking, display name or
client-supplied identifier. Missing seat/focus refuses. Underlying submitted-output,
size-limit, popup-grab, move/resize and lifecycle checks remain authoritative;
see `native-window-maximize.md`, `native-window-minimize.md` and
`native-window-tiling.md`. Placement still waits for an acknowledged root commit.
No command scales buffers, dispatches client requests or silently retries.

`WindowCommandError` is non-exhaustive and carries detailed input, missing-focus,
minimize, maximize and tile refusals. Legacy errors are wrapped in `Shortcut`.
The original close/cycle-only entry point and exhaustive `ShortcutDispatchError`
remain source and behavior compatible. Callers can migrate to the new entry point
when they need layout commands; no existing API is removed or deprecated.

This is trusted command execution, not raw keyboard routing. Callers still own
layout lookup, consumed press/release and repeat isolation, current settings and
rendered controls. `Action::said` uses the existing externalized vocabulary;
errors are diagnostic data, not UI labels. The application verbs retain their
grants, proposed changes and single approvals; this API gives an agent no access.

Evidence: `crates/alo-shell/tests/shortcut_dispatch/layout.rs` uses real Wayland
clients; the graphical minimize/tile fixtures exercise command dispatch and full
GLES readbacks. Neither fixture certifies physical input or DRM scanout.
