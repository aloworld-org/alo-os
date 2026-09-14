# Native desktop

Status: additive trusted Rust API in `alo-shell`, 2026-09-14. ADR 0001;
ADR 0002; ADR 0009; ADR 0010. No protocol, agent capability, D-Bus interface or
stored format changes. Where the dock is, how thick and which end is far are
`alo-dock`'s; light and dark, the accent and the text size are
`alo-appearance`'s; every number the two desktop windows show is
`alo-measuring`'s; every sentence is the vocabulary's. This document describes
only the surfaces that draw those answers and carry a person's keys back.
Nothing here is reachable by an agent, and nothing here grants, approves,
revokes, measures for itself or acts on what it shows.

## Values

`DesktopLook` is how the desktop looks at one moment.

- `DesktopLook::of(&appearance, now, reading)` — **the only constructor**:
  `Appearance::scheme_at(now)`, `Appearance::accent_at(now)` and
  `Appearance::text()`, for somebody reading `reading`. There is no way to make
  a look with a scheme or colour of the caller's choosing.
- `scheme()`, `accent()`, `scale()`, `reading()`.

An accent reaches a palette only through `alo_appearance::Accent::of_colour`,
and is drawn as that accent's value for the scheme on screen. A look whose
accent that door refuses — terracotta above all — refuses the frame with
`RenderError::AccentRefused`.

`RunningWindow` is the window of what is running.

- `RunningWindow::closed()`.
- `opened(earlier, later, interval)` — two `Result<alo_measuring::Reading,
  NotMeasured>` and the time between them, as the host took them. The rates are
  `Reading::since`'s.
- `read_again(later, interval)` — one newer reading and the time since the one
  the window holds; does nothing while closed.
- `pressed(RunningKey)` — `Up`, `Down`, `First` (Home), `Last` (End) move the
  view; `ReadAgain` (F5) answers `RunningPressed::ReadAgain` so the host takes
  a reading; `Close` (Escape) closes; `Nothing` is every other key and every
  chord. No key acts on a process.
- `close()`, `is_open()`.
- `shows()` — `RunningShows::Nothing`, `Running { running, moved_past }` or
  `Refusal(&NotMeasured)`, never an empty list in place of a refusal.

`FillingWindow` is the window of what is filling the disk.

- `FillingWindow::closed()`.
- `opened(&folder)` — counts the folder now with `alo_measuring::Holding::of`
  and opens it; a folder that cannot be counted is the refusal in the window.
- `pressed(FillingKey)` — `Previous` (Up), `Next` (Down), `First`, `Last` move
  the selection; `Open` and `Shut` are the arrow pointing the way a line is read
  and the other one; `Toggle` (Enter, Space); `CountAgain` (F5) counts the
  folder again, keeping what was open by path; `Close` (Escape). No key
  deletes, moves or empties anything. Answers `FillingPressed`.
- `close()`, `is_open()`.
- `shows()` — `FillingShows::Nothing`, `Holding { holding, opened, selected }`
  or `Refusal(&NotMeasured)`.

`DesktopFrame { dock, look, strings, egress, running, filling }` borrows the
person's `alo_dock::Dock`, the look, their `alo_strings::Strings`, the
`EgressStatus` the status area holds, and both windows.

## What is drawn

- **The dock**, per display: a band on the edge
  `Dock::layout_on(display size, text size, reading)` names, as thick as it
  says, with the accent along its inside edge and **the status area** — a
  segment set off by a rule — at the end it calls far. The egress indicator's
  lines grow from that status area, laid out from the same `Dock`.
- **The window of what is running**: above its rows the machine's memory and
  the memory available; then a row per process in `Running`'s order — name,
  process id, resident memory in bytes, share of the processor in thousandths,
  bytes read, written, received and sent per second, and `Network::said` —
  then a row per process that ended with `Gone::said`. Every number is the
  digits of the `Number` behind it; a number the kernel did not give is
  `Number::instead`.
- **The window of what is filling the disk**: `Holding::not_the_whole` and
  `Holding::left_unnamed` above the rows; the folder asked about, and under
  each open folder what is inside it in the count's order, each row its name,
  its size in bytes and `Counted::said` where the size is not the whole truth.
  Open and closed are told apart by shape; the selected row is marked in the
  accent.
- Both windows share the room the dock leaves, the running window on the side a
  person starts reading from; neither covers the dock. Rows are drawn whole or
  wait for the view, and a rail shape says what is out of view.

## Seat

`Server::running_key(code, state, time)` and
`Server::filling_key(code, state, time, reading)` take a Linux evdev code
through the seat's XKB state and answer the key a press means, or `None` for a
release, a repeated press and an unmatched release. Every key is intercepted and
none is forwarded to a client; keyboard focus is left where it was. A code
outside evdev's range is `InputError::InvalidKey`; a display without a keyboard
is `InputError::Unavailable`.

## Nested compositor

- `Nested::pump_running(&mut server, &mut window)` and
  `Nested::pump_filling(&mut server, &mut window, reading)` route the parent
  window's keys to an open window and answer what each did.
- `Nested::submit_with_desktop(roots, popups, cursor, controls, labels,
  desktop, record, approval)` draws clients, then the dock and the desktop
  windows, then the record window and a waiting question when given, then the
  egress indicator, then the cursor.

A frame is refused whole, before anything is drawn:

- `RenderError::EgressStatusUnknown` and the indicator's other refusals, first;
- `RenderError::DesktopScene` — a display `alo-dock` will not lay a dock out
  on, one larger than 16 384 pixels a side, an open window that cannot hold its
  sentences and the row its view must hold whole in the room it has, or a
  picture laid out for another output;
- `RenderError::AccentRefused`;
- the record window's and the approval surface's own refusals.

## Not here

- The clock, battery, network state and volume in the status area: no crate
  measures or words them yet.
- A dock on a different edge per display: `alo-dock` does not decide one.
- Anything in the dock — applications, a launcher — and any road that opens
  the two windows: no crate decides what the dock holds or declares a shortcut
  for either window.
- Pointer input, and a direct-display (DRM/KMS) submission.
