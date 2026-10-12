# The keyboard reaches a window's menu

Date: 2026-10-12. Workstream: the third PC, `crates/alo-shell`,
`crates/alo-access`. Responsible contributor: third-PC lane. Status: ready for
integration.

## What was wrong

The compact window edge moves Minimise, Maximise/Restore, Move and Resize off
the strip and into a menu, so that a window narrower than 196 logical pixels
keeps Close and a usable drag area. That menu was laid out to the design's
measurements — 264 wide, 48-tall rows, an 8px gap from the edge, clamped inside
the usable output, respecting the window's capabilities — and it had a pointer
hit test.

**Nothing opened it.** And nothing told a screen reader it existed.

This is the same shape of fault as the unpainted edge layer and the controls
that could not be clicked: halves that exist with nothing joining them, each
invisible to a green gate because every test tested the half that was there.
Three files said so in their own headers and were waiting:

| file | what its header said |
|---|---|
| `the_window_edge_is_pointed_at` | *nothing in this repository can focus a control yet … every caller passes `None`* |
| `the_window_edge_reveals` | *three of §5's four conditions are not fed here* |
| `examples/a_real_application` | `None` for focus, under a comment saying §6 would fill it |

## What was built

**`window_menu_keys.rs`** — walking the menu. Down and Up wrap, Home and End
reach the ends, Enter runs the focused row, Escape closes. Unavailable rows are
stepped over rather than landed on: they stay drawn and a reader still meets
them, but four presses of Down must reach four things that work.

Every outcome that closes the menu **names where the keyboard goes**, in the
value it returns. There is no variant meaning *closed, and focus is somebody
else's problem*, so a menu cannot strand whoever arrived by keyboard. Escape
works on a window that allows nothing at all, which every other key refuses —
without that, opening the menu of a capability-less window would be a trap.

**`window_menu_reading.rs`** — the accessible name and the rows. The panel is
`Role::List` named *Window menu*; each row is a `Role::Button` named by the
action's own word, with `State::ReadOnly` on the ones this window will not
allow. No `Role::Menu` was added: a role is a public surface other people's
readers match on, and *a list of things a reader walks* is what `List` already
says.

The window's title is carried as a plain string rather than as a `Control`,
because a `Control`'s name is a `Word` with a key and a title is whatever the
application called itself. `access_nodes` meets the same wall for the list of
open windows and answers it the same way.

**`window_edge_keys.rs`** — the road onto the edge itself. Tab to it, walk its
controls, press Menu to open the menu, Escape to come back out onto the control
that opened it. The focus ring stays on the Menu control while its menu is
open, which is how every menu a person has used behaves and is why Escape puts
them back on it rather than nowhere.

**`the_edge_takes_the_keyboard.rs`** — one keyboard state per window, held on
the server beside `edges_revealed` and pruned on the same walk of the mapped
surfaces. This is what makes the above a road rather than a type nothing owns.

**`alo-access`'s `THE_WINDOW_MENU`** — the name, declared in the crate
`alo-saying` collects. A word declared in `alo-shell` would be a string a
person can see and no translator can reach, which is what
`EVERY_WORD_ANOTHER_CRATES_SURFACE_SAYS` exists to stop; it goes from two to
three.

## One defect found while wiring it

Every edge starts concealed — `Revealing::covered` is the state a window has
until something reveals it — and `edge_of` lays a concealed edge out with no
controls. So a person who never touches a pointer could never reach an edge at
all. That is the fault `access_nodes` already names in its own words:
*revealing is what a person sees and is never what a person can reach.*

The arrival now lays out a revealed edge to name the control from. It is the
one second layout in this crate, and the reason is that the arrival is the
event which *changes* whether the edge is revealed, so it cannot be answered
from a layout computed before it happened.

## What was measured, and what is still owed

**Touch cannot be delivered from this crate.** The contract asks that touch
access not depend on hover. Measured on 2026-10-12 across every crate's
sources, with controls so a zero means the query ran:

| what | files |
|---|---|
| pointer (`PointerHandler`, `pointer_motion`, `add_pointer`) | 9 |
| keyboard (`add_keyboard`, `KeyboardHandler`, `keyboard_key`) | 11 |
| touch (`wl_touch`, `WlTouch`) | **0** |
| touch (`add_touch`, `TouchHandler`, `delegate_touch`, `TouchTarget`) | **0** |
| `InputEvent::Touch` | **0** |

No touch event reaches any surface on this machine, because the seat never
announces a touch capability. This is not a geometry question and no change to
the window edge can answer it: it needs a touch input path, which is work of
its own and is not in this change. The requirement is recorded as blocked on
that rather than reported as met.

**The menu is not drawn, and a pointer therefore still cannot open it.**
Measured on 2026-10-12: `the_menu_of` has **no production caller** — the only
mentions outside its own tests are the re-export in `lib.rs` and two doc
comments — and there is no painter for it. The edge has `window_edge_paint.rs`
and `window_edge_picture.rs`; the menu has neither.

So a press on the Menu control still takes `TheRoad::NothingHasOneYet` and
`direct_desktop` still says so on stderr. **That was left deliberately.**
Wiring the press to open a menu nothing paints would set a state no person
could see, which is this change's own named fault in a new place; the keyboard
road is different only because what it produces — the focus ring and what a
reader says — is drawn and spoken by things that already exist.

What the painter needs first is a live read of two Figma states. The snapshot's
own `MANIFEST.md` records `414:27594` (*State=Menu open*) and `414:27609`
(*State=Keyboard focus*) as captured boxes with no contents, so **the focus
ring's measurements are not in the capture** and must be read before they are
drawn. The five colours are already in `variables.toml`.

**`Revealing`'s fourth condition, `a_drag`, is still unfed.**
 Nothing begins a
drag of a window from its edge yet. Pressing Move answers with the control and
the caller decides, and a reporter here that nothing ever called would be the
unfed half this change exists to remove.

**A reader is told an unavailable row is read-only and not why.** That a window
with one size cannot be resized needs a sentence of its own, and a reason given
in English only would be worse than a state said in the person's own language.
Recorded where the state is decided.

**The running implementation has not been shown beside the Figma reference.**
The example draws the focus ring now; the comparison the owner asked for —
light and dark, 125%, 150% and 200% — is not in this change.

## The gate

`cargo clippy -p alo-shell -p alo-access --all-targets -- -D warnings` clean,
`cargo test -p alo-shell --lib` 871 passing, `cargo test -p alo-access` passing,
`cargo doc --no-deps` clean, `cargo fmt --all` clean. Forty-seven of those tests
are new in this change.

Two `alo-agentd` lib tests failed on the development machine before this change
and after it, both with *cannot make a control group at
/sys/fs/cgroup/init.scope/home, File exists* — a cgroup a killed earlier run
left behind. Removing it made them pass. A local leftover, not a fault in this
change, and the same hazard exists for any boundary test killed mid-run.
