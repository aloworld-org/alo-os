# The pointer on a walk

What the automated walk ([walks on GitHub](walks-on-github.md)) checks about
alo's pointer, after step 9 of the first walk and before the provider section.
Asked for by the owner on 2026-11-14: *"Add them now."* The code is
`crates/alo-walking/src/the_pointer_walk.rs`; how each check looks is
`walking_the_pointer.rs`.

The owner's hard rule for the pointer is that **it must work in applications
alo did not make** (`docs/features.md`, *The alo Cursor*). So the walk does not
stop at alo's own desktop: it opens Text Editor, VSCodium and Firefox — three
of the eight applications every fresh machine installs
(`crates/alo-software/shipped.toml`) — and looks at the pointer over each.

## How the walk sees the pointer

**alo draws its pointer into the picture itself**, as one more layer of the
frame (`alo_shell::scene_drawing`, `the_pointer_drawn`), not on a hardware
cursor plane. So qemu's `screendump` should contain it — but *should* is read
off the code, and a check that cannot see the pointer passes while the product
is broken. So the first pointer check **measures** it on every run: move the
pointer a little and count the pixels that changed. None changed fails the
walk there, and every pointer check after it reads NOT REACHED.

**The reference is the artwork, not a screenshot.** The walk asks
`alo_cursors::draw_in` — the function the compositor draws with, and the one
the cursor theme for other applications is written from — for each shape, in
the look the person chose, at the size the compositor asks for (the display's
scale times the person's size, through `alo_cursors::measures::pixels_at`). It
compares only the pointer's **solid pixels**, the ones that look the same over
any background, and needs nine in ten of the fill's and nine in ten of the
keyline's (`the_pointer_seen.rs`). Every shape the walk names, in all eighteen
looks, is told apart from every other by a test
(`the_pointer_seen_tests.rs`).

**Where the pointer goes is read back, never assumed.** The walk machine has a
PS/2 mouse and libinput accelerates it, so a commanded move is not a measured
one. Each move is small steps, then the pointer is found on the screen again,
and the next move asks for what is left (`steering.rs`). A pointer that cannot
be found after a move is *lost*, and the walk stops.

**A found pointer must move with the hand.** After a match the pointer is
nudged a few pixels and must be found again, moved. A picture of a pointer does
not do that.

## What it checks

| # | Does | Checks | What would have to be true for it to pass while broken |
|---|---|---|---|
| 1 | Escape closes Settings; the pointer is moved a little | the screen capture changes, by no more than two pointers' worth | something else small changed at that moment and the pointer is not in the capture — so this is never the check that the pointer is alo's |
| 2 | the pointer is steered to the middle of the desktop | it is found there | — |
| 3 | | **on the desktop it is alo's navy kite**, 24 pixels at 100 per cent, and it moves when nudged | something drew alo's exact kite pixels and moved them with the hand without being the pointer |
| 4 | Super+Space, *Text Editor*, Enter; the pointer onto what changed | **over Text Editor it is still alo's**, in any of alo's shapes | the same |
| 5 | the same for *VSCodium* | **over VSCodium it is still alo's** | the same |
| 6 | Super+I, End, Tab; *Large*, Space; *White*, Space; the pointer onto Settings | **over Settings it is the large white kite**, 36 pixels | the same |
| 7 | Up to *Classic*, Space | **the familiar arrow is the pointer** | the same |
| 8 | Up to *Kite*, Space | **the kite is the pointer again** | the same |
| 9 | Escape; Super+Space, *Firefox*, Enter; Ctrl+L, a page, Enter | **over Firefox's page, the large white kite**; **over a text field, alo's text cursor**; **over a link, alo's hand** | the same; and Firefox drawing alo's theme from files that happen to match a look the person did not choose |

**A wrong shape does not stop the walk.** Looking at the pointer changes nothing
on the screen, so the next check is still a measurement; it is written down as
FAIL with what was seen instead — *the navy kite, default size, 24 pixels at
(612, 400)* where the large white one was expected — and the walk goes on. A
place that cannot be found, or a pointer that is lost, does stop it.

**Firefox is opened after the choice on purpose.** An application that draws
its own pointer reads which one when it starts
(`alo_desktop::telling_applications_about_the_pointer`), so the checks over
Firefox are the ones that show the person's choice reached an application alo
did not make. Text Editor and VSCodium are checked in the shipped look, before
it.

**The page Firefox is shown** is a `data:` address typed into the address bar
(`the_pointer_walk::the_page`): a magenta text field, a cyan band and a lime
link. The walk finds each by its colour, so nothing is fetched and nothing on
the page is a picture of a pointer.

## What it does not check

- **An application already running when the look changed.** By `alo-desktop`'s
  own account it keeps the pointer it started with, unless it asks alo by name.
- **Black and the largest size** are never chosen on the walk; the matching is
  the same for every look and a test holds all eighteen.
- **The pointer flowing between shapes**, which `docs/features.md` stars and is
  not built.
- **A person's machine.** It is still qemu: a virtio display at one scale, a
  PS/2 mouse, no hardware cursor. A machine whose display driver puts the
  pointer on a cursor plane is not what this measures.

## Measured

*Nothing yet.* The first run of these checks is the measurement of whether the
capture shows the pointer at all, and its result goes here with the run's
address and the commit its image was built from.
