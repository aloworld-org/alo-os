# The pointer on a walk

What the automated walk ([walks on GitHub](walks-on-github.md)) checks about
alo's pointer, after the terminal section of the first walk and before the provider section.
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

**Where the pointer goes is read back, never assumed.** The walk puts the
pointer at an absolute place through qemu's VMware mouse port, which the
guest's kernel binds as `VirtualPS/2 VMware VMMouse` (`pointer_moves.rs`
says why: relative moves did not reach the pointer at all). Then it finds the
pointer on the screen again, and aims again by what the last place missed by
(`steering.rs`). A pointer that cannot be found after a move is *lost*, and
the walk stops.

**A found pointer must move with the hand.** After a match the pointer is
nudged a few pixels and must be found again, moved. A picture of a pointer does
not do that.

## What it checks

| # | Does | Checks | What would have to be true for it to pass while broken |
|---|---|---|---|
| 1 | Alt+F4 closes the terminal the walk before left open; the pointer is moved a little | the screen capture changes, by no more than two pointers' worth | something else small changed at that moment and the pointer is not in the capture — so this is never the check that the pointer is alo's |
| 2 | the pointer is steered to the middle of the desktop | it is found there | — |
| 3 | | **on the desktop it is alo's navy kite**, 24 pixels at 100 per cent, and it moves when nudged | something drew alo's exact kite pixels and moved them with the hand without being the pointer |
| 4 | Super+Space, *Text Editor*, Enter; the pointer onto what changed | **over Text Editor it is still alo's**, in any of alo's shapes | the same |
| 5 | the same for *VSCodium* | **over VSCodium it is still alo's** | the same |
| 6 | Super+I, End, Tab; *Large*, Space; *White*, Space; the pointer onto Settings | **over Settings it is the large white kite**, 36 pixels | the same |
| 7 | Up to *Classic*, Space | **the familiar arrow is the pointer** | the same |
| 8 | Up to *Kite*, Space | **the kite is the pointer again** | the same |
| 9 | Escape; Super+Space, *Firefox*, Enter; the pointer onto *Continue* on Firefox's first-start dialog, a click; Ctrl+L, a page, Enter | **over Firefox's page, the large white kite**; **over a text field, alo's text cursor**; **over a link, alo's hand** | the same; and Firefox drawing alo's theme from files that happen to match a look the person did not choose |

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
- **A person's machine.** It is still qemu: a virtio display at one scale, an
  absolute mouse port rather than a mouse a hand moves, no hardware cursor. A machine whose display driver puts the
  pointer on a cursor plane is not what this measures.

## Measured

**Run 43646031636, 2026-11-14, image from main at `09597d7d`, once.** The
first check failed, and failed the right way: moving the pointer by relative
steps of 24 and 16 changed no pixel. The picture it kept
(`20-pointer-the-screen-capture-shows-the-pointer.png`) **does** show a navy
arrow at the top-left corner — so the capture includes alo's pointer, read by
eye from that picture; what did not happen is the pointer moving. The serial
log shows the mouse bound as `VirtualPS/2 VMware VMMouse`, which takes
absolute positions, and that fits; it was not taken apart further. The walk
now places the pointer absolutely. Every pointer check after the first read NOT
REACHED on that run, and nothing about the pointer's shape was measured.

**Run 43650359669, 2026-11-14, image from main, once.** With the pointer placed
absolutely, the first sixteen pointer steps ran:

- **The capture shows the pointer**: a move of 24 by 16 pixels changed 500.
- **On the desktop, alo's navy kite**, 24 pixels at 100 per cent, found at
  (639, 399) and again at (644, 402) after a nudge.
- **Over VSCodium, alo's navy text cursor**, over the editor
  (`26-pointer-over-vscodium-the-pointer-is-still-alo-s.png`).
- **Settings made the pointer large and white**: the picture kept shows it
  (`31-…png`), and *Large* and *White* ticked.
- **The check over Text Editor passed and measured the wrong thing.** Its
  picture shows the pointer on the Dock, not on Text Editor's window: the
  window is white on a cream desktop, which differs by about ten levels, and
  the walk counted a pixel as changed only past 24, so the Dock's shift was the
  largest change. **A pass that is not evidence**, written down here rather
  than left in the report; the threshold is now 3.
- The walk then stopped at *the pointer goes onto Settings* for the same
  reason.

**Run 43654461272, 2026-11-14, image from main, once.** Twenty pointer checks
passed before the walk stopped at Firefox:

- **On the desktop, alo's navy kite**, 24 pixels.
- **Over Text Editor, alo's navy text cursor**, now over its window (the
  threshold fix held), and **over VSCodium, alo's navy text cursor**.
- **Over Settings, the large white kite**, 36 pixels, at once; **the familiar
  arrow** when *Classic* was chosen, and **the kite** again when *Kite* was —
  each found again after a nudge.
- **Firefox**, opened after the change, put its first-start dialog (*Welcome
  to Firefox … Continue*) over everything, so the page was never typed into
  the address bar and the walk stopped there. The picture kept
  (`38-the-pointer-page.png`) shows a large white kite over Firefox — **read by
  eye, not measured**. The walk now presses Escape and Enter past that dialog.
  The address it typed also carried runs of spaces, from a string the walk
  built wrongly; that is fixed and a test now refuses it.

**Run 43658977070, 2026-11-15, image from main, once.** The same twenty
pointer checks passed again. Firefox's first-start dialog stayed up through
Escape, Enter and the whole typed address — **nothing typed reached Firefox**,
which fits Firefox not having the keyboard after opening from the Bar; that is
not confirmed, and it is not a pointer fault. The walk now clicks *Continue*
with the pointer, as a person would, and types the address after that.

**Run 43667201920, 2026-11-15, image from main, harness at `ba840a78`, once —
the whole walk passed: 57 checks, 0 failed, 5 skipped (the provider section,
which needs a secret).** Every pointer check passed:

| Check | What was found |
|---|---|
| the capture shows the pointer | 500 pixels changed for a move of 24 by 16 |
| on the desktop | the navy kite, 24 pixels |
| over Text Editor | the navy text cursor, 24 pixels |
| over VSCodium | the navy text cursor, 24 pixels |
| over Settings, after *Large* and *White* | the white kite, 36 pixels |
| after *Classic* | the white familiar arrow, 36 pixels |
| after *Kite* again | the white kite, 36 pixels |
| over Firefox's page, Firefox opened after the change | the white kite, 36 pixels |
| over a text field in Firefox | the white text cursor, 36 pixels |
| over a link in Firefox | the white hand, 36 pixels |

Each was found again a few pixels on after the pointer was nudged. The walk
runs after every merge to main (`walk.yml`), so these run again there; this
table is one run's result, not a standing property.

**Run 43801984900, 2026-11-15, image from main, once.** The walk stopped at
*the pointer goes onto Firefox's page*, before the VSCodium section. The
picture after Enter (`43-the-pointer-page.png`) shows Firefox asking *Open
the ata link with System Handler?*: the address reached the bar **without its
first letter**, so Firefox read `ata:` as a kind of link it does not know.
Ctrl+L and every key after the first reached Firefox. Firefox was still
starting — its bar's placeholder reads *Search or enter address* in
`42-firefox-continued.png` and *Search with Google or enter address* in `43-`
— and moving the keyboard into its bar is Firefox's own work, done after the
key that asks for it. That **fits** the first letter landing where the
keyboard was a moment before; it is **not checked**. The walk now presses
Ctrl+L, waits for the screen to change and settle — a look fails if it does
not change — and only then types the address
(`the-address-bar-has-the-keyboard`). The checks on the page are unchanged.
