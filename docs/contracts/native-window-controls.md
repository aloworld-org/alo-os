# Native window control view

Status: additive trusted Rust view API, 2026-09-08. ADRs 0002 and 0010;
v0.01 window management. No protocol, agent capability or stored format changes.

`WindowControlLayout::new(viewport, origin, enabled, restoring)` constructs an
immutable scale-one view of minimise, maximise/restore and close, in that order.
Three 32 by 32 buttons have four-pixel transparent gaps (104 by 32 overall).
The output is positive and at most 1,000,000 per dimension; each origin coordinate
is between -1,000,000 and 1,000,000. Invalid geometry refuses before allocation
of drawing primitives or any client/graphics effects. Negative origins clip.
The host chooses placement; this API does not reserve a title bar or work area.

`controls()` retains all three buttons even when clipped or disabled. Each exposes
its `Action`, full bounds and supplied enabled state. `Action::said` is the sole
label path; the host must load `alo_shortcuts::shortcut_words` and present those
labels through its native text/accessibility surface. The maximize vocabulary
describes both maximize and restore. This component draws icons, not label text.

`hit(x, y)` uses half-open button and viewport bounds. Non-finite coordinates,
gaps and clipped-away areas have no hit. Disabled buttons still return a hit:
unavailability must not cause a click to reach a client underneath the control.
The hit is presentation information, never permission to execute an operation.

`paint(frame, scheme)` uses the existing alo-appearance tokens, with opaque
grounds and contrasting original glyphs. Disabled controls have both a changed
ground and a separate strike mark. Terracotta is never used. The restoring flag
selects overlapping rectangles rather than the maximize rectangle. All solids
are clipped to the same output used by hit testing; gaps remain untouched.
The caller supplies an active normal-transform scale-one frame matching the
captured viewport, draws the strip after clients and before the pointer, and
owns finish/submission. Paint errors propagate as `RenderError::Submission`;
a partially drawn frame must not be published after failure.

The component owns no server, surface, input seat, callbacks or window authority.
Availability is supplied presentation data and is not derived or cached policy.
Production composition, native label presentation, mapping-lifetime input tokens,
live operation revalidation, pointer press/release ownership and cancellation,
hover/pressed feedback and nested/direct routing remain integration work.
Existing ordinary client keyboard paths are unchanged. This API completes the
control view's layout and painter, not usable controls or window management.

Evidence: `window_controls_tests.rs` checks geometry, malformed input, exact
coverage and non-color disabled distinctions. The `window_controls_check` example
compares complete GLES readbacks to independent literal glyph masks and palette
values. This is a WSLg development fixture, not display submission, live control
interaction, physical input or certified hardware evidence.

## Live presentation snapshots

Additive trusted Rust API, 2026-09-09:
`Server::window_control_snapshot(surface, viewport, origin)` returns a
`WindowControlSnapshot` for the explicit visible mapped toplevel in this display.
`surface()` retains that exact protocol handle; `layout()` exposes the immutable
view, actions, hit geometry and derived availability. `layout().restoring()`
reports the glyph choice. Foreign, hidden, child, popup, unmapped and dead targets
return `WindowControlSnapshotError::Unmapped`; malformed layout returns `Layout`.
There is no focused-window or stacking fallback, and no input seat is required.

Minimise and close are enabled for every eligible root. Maximize/restore uses the
same side-effect-free layout planner as execution, including output availability,
competing move/resize/popup grabs, representable normal geometry and committed
restore limits. `maximize_refusal()` exposes the existing `WindowMaximizeError`
for diagnostics, not user-facing text. No new refusal text vocabulary is added.
The latest requested mode drives the glyph even before acknowledgment or commit:
maximized offers restore, while normal and tiled offer maximize. Restore can remain
available after output retirement; a new maximize requires a submitted output.
The supplied painting viewport does not establish that output availability.

Capture takes `&self`: it sends no configure or close, prunes no state, remembers
no geometry, changes no focus and consumes no input. Availability is true only at
capture time. Refresh for every frame and use the existing `Action::said` labels.
The snapshot has no dispatch method and is **not a mapping-lifetime token**:
unmap/remap can reuse the same protocol handle, and a retained snapshot remains
frozen. Future pointer routing must bind presses to a separate mapping lifetime,
cancel stale ownership and revalidate operations at release. These rules prevent
presentation data from becoming cached authority; usable controls remain open.

Six real-client tests in `tests/window_controls/mod.rs` cover no-effect reads,
typing isolation, output/limits/geometry/busy refusal, pending mode changes and
target lifetime. The nested offscreen fixture paints twelve complete snapshot-derived
light/dark frames across five maximize/restore boundaries and a reused restored
boundary after minimization, alongside its unchanged client scene pixel assertions.
Exact executed results and limits belong to
`docs/autonomy/updates/live-window-control-snapshots.md`.
