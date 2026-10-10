# A window a reader can act on

**What changed, in one sentence.** The tree a screen reader finds listed the
windows open and nothing under them, so somebody using alo without sight was
told a window existed and never that it could be closed; each open window now
carries its edge's controls.

## What was actually wrong

`alo-shell` builds an accessibility tree in `access_nodes`, puts it on the
at-spi2 bus in `access_bus`, and keeps it current in `access_serving`. Under the
desktop it publishes a list called *the windows open*, and since 2026-09-27 that
list has held one node per window, named with whatever the application called
itself.

Each of those nodes had no children. A reader could say *Ledger for March* and
nothing else — not that it could be minimised, maximised or closed. The actions
existed, the words for them existed in every language alo ships, and nothing
connected the two.

## What it does now

| | |
|---|---|
| `window_edge_reading` | which controls a window **has**, from its decorations alone |
| `window_edge_who_draws` | who draws a frame's header — one answer, asked by both the drawing and the reading |
| `access_nodes` | each open window carries those controls as children, named by the action's own word |

A control's name is `alo_access::Control::for_action`, so it **is** the action's
word — one string rather than two that have to agree, which is EN 301 549 clause
11.2.5.3 held by construction rather than by review (ADR 0089).

## The mistake, because it is the useful part of this report

The first version asked the wrong question. `what_a_reader_is_told` took an
`Edge` and read the controls off it — and an edge at rest correctly draws none,
because the strip only appears when a pointer comes near. So a person using the
keyboard could reach nothing at all until a pointer had been near a control.

That is exactly the clause the owner wrote down: *screen-reader and keyboard
users must reach the new edge without first hovering.* It failed on the first
attempt and its own test caught it, which is the only reason this report is
about a fixed bug rather than a shipped one.

It takes `Decorations` now and no geometry at all. **Whether a control exists is
an accessibility question before it is a drawing one**, and the two are asked
separately: where a control sits is drawing, whether it is there is not.
Revealing is what a person sees; it is never what a person can reach.

## One answer, asked twice

`who_draws_a_frame` returns `Decorations` and is asked by both the layout and
the reading. A reader told about a control nobody drew, and a control drawn that
no reader can name, are the same bug read from two ends, and one function makes
them unrepresentable —
`what_a_reader_is_told_is_what_the_edge_lays_out` fails if they ever disagree.

It answers *the shell draws it* for every frame, and that is measured rather
than assumed: `XdgDecorationHandler` answers `Mode::ServerSide` in all three of
`new_decoration`, `request_mode` and `unset_mode`, so every window alo has been
asked about is one alo decorates. What it cannot yet tell apart is a client that
**never binds** `xdg_decoration` — most GTK applications, drawing their own
headerbar without negotiating. That needs the toplevel's protocol state read per
frame, which belongs to the lane holding §5 and §6 of the edge contract. **That
function's body is what changes when it lands, and nothing else.**

## How it was verified

| | |
|---|---|
| 719 unit tests in `alo-shell` | pass, including two new ones in `access_nodes_tests` |
| `the_served_tree_follows_the_windows_that_open` | reads all three controls back off a **live at-spi2 bus**, under each of two open windows |
| `clippy -D warnings`, rustdoc, `fmt` | clean |

The bus test needed a second look before it meant anything. It ends in
`unwrap_or_default`, so a bus carrying no list at all returns an empty vector,
and every assertion inside the loop over it would have passed while proving
nothing. The window count is asserted before the contents, so the zero is now a
failure rather than a pass that reads like one.

## What this does not claim

**Not verified on real hardware.** The at-spi2 bus is real; the compositor under
it is nested. Whether a screen reader on the testing machine speaks these
controls is one of the two runs that need the owner physically present.

**The window menu has no action and no word.** On a window whose application
draws its own header, alo's edge carries only a menu, and alo tells a reader
nothing of its own — correctly, because naming a menu it could only say in
English would be worse. The three window actions remain reachable on that window
by keyboard: `Alt+F4`, `Super+Down` and `Super+Up` are shipped defaults that act
on the focused window whoever drew its header.

And **nothing in alo implements a window menu** —
`tests/support/wm_capabilities.rs` uses `WindowMenu` as its example of a
capability this compositor does not advertise, and asserts the set is exact
without it. The edge design says *an application with incomplete controls still
reaches every shell action through the window menu and the keyboard*; the
keyboard half is true today and the menu half is unbuilt. Giving it an action now
would be a word for something that does nothing, which is the stub the third law
forbids, so it is recorded as an open decision rather than filled in.

## Documents

`contracts/native-window-controls.md` is marked superseded with a notice saying
by what, rather than rewritten or deleted: it is a public surface, and those
change additively with versioning and deprecation. Its labelling rule is not
superseded and the notice says so. `design/the-canvas-as-a-workspace.md` asked
for *a shared window component* providing consistent title bands; that component
exists now, so the sentence names it.
