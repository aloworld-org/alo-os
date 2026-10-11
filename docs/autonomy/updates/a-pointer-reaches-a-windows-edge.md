# A pointer reaches a window's edge

Date: 2026-10-11. Workstream: the third PC, `crates/alo-shell`. Responsible
contributor: third-PC lane. Status: ready for integration.

## What was wrong

Every window's external edge was drawn, measured against the design, named for
a screen reader, and mapped to an action. None of it could be clicked.

The three halves existed and nothing joined them:

- `window_edge::EdgeControl::target` has carried the sentence *the 44 × 44 it
  answers in* since the layout was written;
- `window_edge_reading::what_it_does` maps every control to an
  `alo_shortcuts::Action` — Minimise, Maximise, Close;
- `window_edge_paint::solids` draws a highlight behind the hovered control and
  `status/danger` on hovered Close.

**No code turned a position into a control.** The search for one is recorded
below. So the one caller in the repository passed `Pointing::default()`, which
means no control on any window ever showed hover or focus feedback — and a
press on Close reached nothing, because nothing asked what a press was on.

This is the shape of fault #625 removed hours earlier: a capability built,
tested and unreachable. It is worth naming twice because both were invisible
to a green gate — every test passed, because every test tested the half that
existed.

## What was built

**`crates/alo-shell/src/window_edge_pointing.rs`** — the hit test. A position
and a `WindowEdge` in; `Some(Control(OnTheEdge))`, `Some(Drag)`,
`Some(TheEdgeAndNothingOnIt)` or `None` out. `pointing_at` assembles the
`Pointing` that `EdgePicture::of` takes, so there is one way to turn a position
into a picture rather than one per host.

Two decisions in it are the ones that matter:

- **The target answers, not the pill.** A control's target is 44 × 44; the
  highlight drawn behind it while hovered is 32 × 28 at `left 6, top 14`. A
  hit test written against what is drawn would ignore a press in the margin
  between them, which reads to a person as a bad mouse rather than as a bug.
  Each of the four sides of that margin is asserted.
- **`None` and `TheEdgeAndNothingOnIt` are different answers.** Inside the
  region the edge stays revealed; outside it the edge conceals. Three places
  are deliberately neither a control nor a drag: the four units between
  targets, the eight before the drag region starts, and — at rest —
  everything above the 12-tall band the grip sits in.

**`crates/alo-shell/src/the_window_edge_is_pointed_at.rs`** — the one method a
host needs, `Server::what_this_edge_is_pointing_at`. It exists because a host
cannot read the pointer: `where_the_pointer_is_in_logical_pixels` is
`pub(crate)` so that *absent* stays distinguishable from *at the origin*, and a
host made to supply a position would have to invent one for a machine whose
seat has never had a pointing device. The invented answer lights a control on
whichever window sits at `(0, 0)`.

It takes the edge rather than building one, so the highlight is measured
against the layout being painted and not a second layout that could disagree.

**`crates/alo-shell/examples/a_real_application.rs`** — the call site, which is
what makes this reachable rather than merely present. `Pointing::default()`
becomes `server.what_this_edge_is_pointing_at(&edge, None)`.

## What is deliberately not here

**Keyboard focus.** `focused` is an argument and every caller passes `None`,
because nothing in this repository can focus an edge control yet — §6 of
`docs/design/the-external-window-edge.md` is where keyboard navigation into the
edge gets built. It is an argument rather than a hardcoded `None` so that the
day something can focus a control, the call site changes and this does not.

**The press itself.** This answers *what is under the pointer*. Routing a
button event to `what_it_does` and performing it is the next change, and it is
a different responsibility: this file decides what a press lands on, that one
decides what happens then.

## How it was checked

Thirteen tests in `window_edge_pointing_tests.rs`, written against real layouts
rather than literals — the control list is read from the edge's own `controls`,
so a control added to the layout is covered the day it is added. Three in
`the_window_edge_is_pointed_at_tests.rs` against a bound `Server`.

The one that carries its own control: `a_machine_with_no_pointer_highlights_nothing`
asks `what_is_under` at `(0, 0)` **first** and asserts the origin is on the
edge, so the assertion that follows is known to be capable of failing. Without
that line the test would pass against a method that truncated an absent pointer
to the origin.

Two faults the run found, recorded rather than quietly fixed:

- `crate::window_edge_reading::Action` is a private re-export of
  `alo_shortcuts::Action`; the test named the wrong path and did not compile.
- `Server::bind` refuses a runtime directory that is not exactly mode 0700
  (`crate::socket`), and `tempfile::tempdir()` arrives at whatever the umask
  leaves. The first run failed with `UnsafeRuntime` — a message about a socket,
  with nothing in it about an edge. `a_camera_per_viewport_tests` sets the mode
  explicitly for the same bind, and this now does the same, with the reason in
  a comment so the next test to bind a server does not rediscover it.

## The search, with its control

Before writing anything, every non-test mention of `OnTheEdge` in `crates/`
was listed: the enum, the `EdgeControl` field, the painter, the reader's two
functions, and nothing else — no caller anywhere converting a position. The
same search for `Pointing::default()` returned the one example and the test
modules. A search returning what a search should return rather than nothing is
what makes those two zeros believable (`docs/misreadings/`).
