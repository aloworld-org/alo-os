# The canvas as a workspace

**Status: the owner's specification of intended behaviour.** Not a reading off
the design file and not a note recommending an answer — the thing those defer
to. `CLAUDE.md` says what the interface looks like is settled in the alo OS
design file and that this folder records *what the owner decided where it is
silent*. This is that, for the canvas.

**Date:** 2026-10-08.

**The design file:** page *03 — alo OS · Living canvas* and the rest of
`nDxyF5Ho9oC4RObjVzwBNJ`, the same file `the-canvas-in-numbers.md`,
`the-alo-dock.md` and `the-interface-in-the-file.md` read from.

## How to use this document

**It is the owner's words, kept whole.** Nothing here is summarised, reordered
or improved, because a specification that has been paraphrased is a second
specification. Where this and a note in this folder disagree, this wins.

**It describes intended behaviour and claims nothing about what exists.** The
owner's own second sentence says so. So a sentence here is not a promise the
product currently keeps, and anybody measuring what is built must measure it,
not read it off this page.

What *is* built, measured on 2026-10-08, is recorded separately in
`docs/autonomy/updates/the-canvas-against-its-specification.md`, so that this
document stays a specification and does not rot into a status report.

---

**The alo OS canvas is the person's workspace: a continuous surface where
applications, documents and ongoing work stay where they were placed.** A person
can work entirely by hand, work alongside alo, or give alo a specific task.

The following describes the intended product behaviour. It is a specification,
not a claim that every feature is already implemented.

## 1. The canvas and Places

A **Place** is an individual canvas. A person can have several Places—for
example, Work, Personal, a business project or a collection of ideas.

Each Place remembers:

- Its name.
- The windows it contains.
- Every window's position, size and presentation.
- The last camera position and zoom level.
- Its panel's expanded or collapsed preference.

Every application belongs to a Place, including ordinary applications that were
not built specifically for alo OS.

Places should feel spacious and continuous. They must also remain navigable: the
system must prevent arrangements that make a window impossible to find or
recover.

Creating another Place does not duplicate applications or documents. Moving a
window between Places transfers the existing window and preserves its work.

## 2. Two separate layers

The interface has two layers with different responsibilities.

| Layer | Contains | Behaviour |
|---|---|---|
| Canvas content | Windows, compact windows and their spatial arrangement | Moves and scales when the person pans or zooms |
| Fixed system controls | Dock, put-aside panel, top controls and status area | Remains attached to the display viewport |

Scrolling or zooming the canvas must never carry the Dock, status area or panel
away.

Fixed controls float directly over the workspace. They should not sit inside a
large, unnecessary background block.

Their visible surfaces and activation areas must follow one shared layout
calculation, so the picture and pointer behaviour agree.

## 3. Moving around the canvas

The person can travel horizontally and vertically without rearranging their
windows.

The distinction between navigation and application input must be predictable:

- Scrolling over an interactive application scrolls its content.
- Scrolling over empty canvas moves around the Place.
- An explicit canvas-pan gesture or command allows navigation even when windows
  cover the viewport.
- Reaching the end of a document must not unexpectedly start moving the entire
  workspace.
- Opening a menu or dialog must not accidentally pan the canvas underneath it.

Panning changes the view, not the positions of the windows.

There must be keyboard routes for navigating the workspace, finding a window and
returning to it.

## 4. Zooming

Zoom helps a person move between understanding the whole Place and working
inside one window.

The essential commands are:

- **Zoom in and out.**
- **100%:** return to the standard canvas scale.
- **Show all:** fit the windows on the current Place into view.
- **Fit selection:** frame the selected window or windows.
- **Enter full screen:** make one application fill the display.

Zooming should preserve a meaningful anchor, such as the pointer position or
selected window. The scene must not jump unpredictably.

At overview scale, window identity and arrangement remain understandable.
Returning to working scale restores useful interaction with application content.

**Show all is a recovery tool as well as a navigation tool.** Its supported zoom
limits and the allowed window positions must be compatible. The system must not
permit a distant window that Show all cannot include.

Reduced-motion preferences apply to camera transitions.

## 5. What a window is

A window is a live application surface with a stable identity.

Its state includes:

- The application and document or activity it represents.
- Its Place.
- Its normal position and size.
- Whether it is active, selected, compact, put aside or full screen.
- Its stacking position.
- Any authorised alo activity associated with it.

The window title must come from that window's actual content. Selecting a
browser window must not leave the title of a document window in the system
controls.

A shared window component should provide consistent title bands, focus
treatment, resize behaviour and controls across applications.

**That component exists and it is the external window edge** —
[`the-external-window-edge.md`](the-external-window-edge.md), laid out by
`alo_shell::edge_of`. Anything needing a title band, a drag region or window
controls asks that one function rather than drawing its own, which is what
makes *consistent across applications* a property of the code rather than a
thing reviewers have to notice. It replaced a three-tile strip and an internal
48-pixel band, and `docs/contracts/native-window-controls.md` is superseded.

## 6. Focus and selection are different

**Focus** identifies where keyboard input goes. **Selection** identifies which
windows a canvas action will affect.

| Interaction | Expected result |
|---|---|
| Click a visible window | Activate it and bring it forward as appropriate |
| Select several windows through the canvas selection interaction | Prepare them for a group action |
| Type inside an active application | Send input to that application |
| Run an arrangement command | Affect the selected windows |
| Hover over another window | Reveal appropriate affordances without changing the work |

An active window and a selected window must have distinguishable visual states.

Human selection uses navy and an explicit shape or indicator. Deep teal is
reserved for alo activity.

**Hover must never create, restore, close, hide or relocate a window.** A
preview may appear on hover, but that preview must remain distinct from the real
window.

## 7. Moving windows

A person drags a window by its title or name band.

During movement:

- The window follows the pointer directly.
- Application content does not become an accidental drag handle.
- The moving window remains identifiable.
- Any alignment preview is clearly temporary.
- Escape cancels the operation and restores its starting position.
- Releasing completes the move.

Selected windows can move together while preserving their relative positions.

There must also be a keyboard operation for moving a selected window. It needs
clear entry, movement, confirmation and cancellation behaviour, without
requiring the person to reach the title band — and the band being an overlay
that reveals on approach makes that requirement stronger, not weaker: a person
who cannot bring the pointer to it must still be able to move the window.

Moving a window to another Place preserves its identity and application state.

## 8. Resizing windows

Windows resize from their edges and corners.

The implementation must respect application minimum sizes and distinguish
resizing the application from zooming the canvas.

Resizing should:

- Keep the opposite edge or corner stable.
- Update the application's layout.
- Preserve a usable title band and controls. The edge is laid out from the
  window's own rectangle, so it follows a resize without being told to; what
  this asks of a resize is that it never leave the edge off the output, which
  `EdgePicture::of` refuses rather than drawing wrongly.
- Provide accessible pointer targets.
- Support cancellation and a keyboard equivalent.

Hit targets, icon dimensions and spacing are separate measurements. A small
visual glyph can have a larger interaction area.

Layout uses logical units, converted through the display scale once.

## 9. Window presentations

These presentations have different purposes and must remain separate actions.

| Presentation | Purpose |
|---|---|
| Normal | Work in a freely positioned, resizable window |
| Compact | Keep a smaller, useful live representation on the canvas |
| Put aside | Remove the window from the canvas's visible arrangement and retain a restore entry in the panel |
| Full screen | Give one application the entire display |

**Compact** preserves a presence on the Place. It should show useful content
where supported, rather than shrinking unreadable text indefinitely.

**Put aside** preserves the window without closing it. It records the original
Place, normal geometry and relevant view context.

**Full screen** preserves the normal geometry so leaving it returns the window
to its previous arrangement.

An application's ability to restore unsaved content is separate from the shell
remembering window geometry. The interface must not promise recovery it cannot
provide.

## 10. Full-screen behaviour

A full-screen application covers the entire display. There is no permanent
canvas margin, Dock-sized gap or empty panel column.

System controls are concealed at rest and revealed intentionally through their
assigned edges or keyboard actions.

The reveal must remain open while:

- The pointer is in the activation area.
- The pointer travels onto or remains on the revealed surface.
- Keyboard focus is inside that surface.
- An interaction that requires the surface is active.

A continuous pointer path must connect the edge and the controls. Moving towards
an icon must not make it disappear.

Leaving full screen restores the window's previous size and position, with a
useful view of it.

For video and other applications with their own immersive mode, application full
screen and shell full screen must have clear exit behaviour. A system command
must remain available even when the application handles Escape itself.

## 11. The Dock's relationship with windows

The Dock is fixed to the viewport. Bottom is the default, and the person can
choose another edge.

Clicking an application that already has windows should help the person return
to those windows. It must not unexpectedly open another one.

For an application with several windows, provide a clear window picker that
shows enough identity to choose correctly.

If the chosen window is elsewhere:

- Travel to its Place and location.
- Restore it first if it was put aside.
- Focus it without silently moving it to the current camera position.

The Dock grows with its contents within the space available to it. Any icon
reduction must stop before usability suffers; further items need an accessible
overflow treatment.

Side Docks need a suitable layout. The alo composer opens horizontally into
available space rather than becoming a rotated text field.

## 12. The panel for windows put aside

The panel provides three presentations:

- **Expanded previews:** individual cards with window identity and relevant
  state.
- **Collapsed rail:** compact restore targets.
- **Empty handle:** a small access point when no windows are put aside.

The panel itself remains fixed. Additional entries scroll inside its list.

The expansion control has its own slot and stays available while the list
scrolls. Individual windows must remain discoverable rather than disappearing
behind an application icon.

Hovering may offer a temporary peek. Clicking restores the actual window.

Restoring returns to the saved Place and arrangement. It does not silently place
the window wherever the person happens to be looking.

When the Dock and panel share an edge, the panel retains its designated handle
and opens its previews inward. Their pointer regions must remain unambiguous.

## 13. Arranging several windows

Multi-selection supports deliberate operations such as:

- Aligning edges or centres.
- Distributing spacing.
- Arranging windows side by side or in a grid.
- Moving the selection together.
- Moving windows to another Place.
- Giving alo a task involving the selected windows.

The interface must make the affected selection obvious before applying an
action.

Arrangement changes should be undoable. Applying an arrangement must respect
application size constraints and keep every window recoverable.

## 14. Working with alo

Alo acts on an explicit scope: a window, a selection or another resource the
person authorises.

Selecting a window does not grant alo access to it.

When alo is working, the relevant window or panel entry should communicate:

- That alo is acting.
- What task it is doing.
- Whether it is working, waiting, paused, stopped or finished.
- Where processing occurs when relevant.
- How to stop it.

Use the alo mark and readable words alongside deep teal. Colour alone must not
carry the distinction.

**Stop must be directly available wherever active work is shown.** Pressing it
requests cancellation immediately; the interface must distinguish "Stopping"
from confirmed "Stopped" when cancellation takes time. Stopping does not
silently undo completed work.

Proposed changes offer **Keep, Adjust and Reject**. Accepted actions and
recoverable changes belong in History.

In **No AI** mode, the entire canvas remains usable by hand. Ordinary actions
must not depend on an agent or trigger repeated invitations to enable one.

## 15. Menus, dialogs and notifications

Different surfaces need different behaviour.

- **Menus and tooltips** stay connected to their source control.
- **Modal dialogs** clearly separate from the background, contain focus and
  prevent interaction with the blocked content.
- **Notifications** communicate without unnecessarily interrupting the current
  task.

Modal backgrounds use the approved blur and dim treatment. Reduced-transparency
mode replaces this with a sufficiently distinct opaque treatment.

A dialog belonging to one window should not unnecessarily block unrelated
windows.

After dismissal, focus returns to the control or window that opened it.

## 16. A window is never lost

The person must always have a way to find, activate and recover a window.

That means:

- Show all can include the relevant canvas extent.
- Fixed controls cannot completely cover the only usable movement handle.
- Keyboard users can find and move a window.
- Disconnecting a display cannot strand windows.
- Restoring a saved layout accounts for the displays currently available.

During a drag, crossing a prohibited boundary should stop at the last valid
position rather than accept the move and then pull the window back.

When the system must recover a window automatically, it explains the move and
offers a return action where safe. Returning must not recreate an unreachable
state.

## 17. Persistence and restart

Each Place's arrangement must survive a real process restart.

Persistence includes window identities where recoverable, Place membership,
normal geometry, camera position, zoom and presentation state.

Saving must tolerate interruption without destroying the last usable
arrangement.

After restarting:

- Restore the layout for the current displays.
- Reopen recoverable applications through their supported mechanisms.
- Explain unavailable applications or content.
- Never imply that unsaved application work was recovered when it was not.

A test that reconstructs objects in memory is not sufficient evidence that
restart persistence works.

## 18. Visual character

The workspace should feel calm, precise and spacious.

- Light canvas surfaces and clear navy text.
- Deep teal only where alo is acting.
- Consistent window corners, borders and elevation.
- Restrained glass effects on floating system controls.
- Few persistent words on the canvas.
- Labels and explanations where a decision needs them.
- Smooth, short transitions that preserve spatial understanding.
- Full keyboard, screen-reader, reduced-motion and reduced-transparency support.

The defining experience is continuity: **the person can see where their work is,
return to it easily, and understand what the computer is doing without losing
control of the workspace.**
