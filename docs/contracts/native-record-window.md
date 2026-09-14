# Native record window

Status: additive trusted Rust API in `alo-shell`, 2026-09-14. ADR 0001 §7;
ADR 0002; ADR 0009. No protocol, agent capability, D-Bus interface or stored
format changes. What the record holds is `alo-record`'s and `alo-keeping`'s;
what an account says — every clause, every remark the record makes about
itself, every refusal — is `alo-recounting`'s; this document describes only the
window that puts that account in front of a person and carries their keys back.
Nothing here is reachable by an agent, and nothing here writes to the record.

## Values

`RecordWindow` is the window. It implements `alo_recounting::Compositor`
internally; there is no way to put an account in it except one
`alo_recounting::Recounting` read off the disk a moment before.

- `RecordWindow::on_an_output()` — a closed window on a compositor that draws.
- `RecordWindow::with_no_output()` — a closed window with nowhere to show
  anything; opening it answers `RecordOpened::NowhereToShow`.
- `opened_by_hand(&recounting)` — a person opened the window. **ADR 0009's
  road**: it takes where the record is and nothing else — no turn, no model, no
  overlay, no agent.
- `asked_what_it_did(&recounting)` — a person asked the agent *what did you
  do?* It reaches **the same account, read the same way**; nothing a model
  says reaches the window.
- `pressed(key, &recounting)` — one `RecordKey`: `Newer` (Up), `Older` (Down),
  `Newest` (Home), `Oldest` (End) move the view through the account; `ReadAgain`
  (F5) reads the record off the disk again and returns what that did; `Close`
  (Escape) closes; `Nothing` is every other key and every chord. Every key does
  nothing while the window is closed. Returns `Some(RecordOpened)` only for
  `ReadAgain`.
- `closed()` — close the window. The record is untouched.
- `shows()` — `RecordShows::Nothing`, `Account { account, moved_past }` or
  `Refusal(&NotRecounted)`. `moved_past` is how many of the most recent entries
  the view has moved past; zero shows the most recent first.
- `is_open()`.

Opening, and reading again, always ask the record
`alo_record::Asking::anything()` at `alo_recounting::AtMost::ONE_SITTING`, reset
the view to the most recent entry, and answer `RecordOpened`:

- `Shown` — the account is in the window.
- `RefusedInTheWindow` — the record is not there, not a record, from a newer
  alo OS, not one this machine will believe, or not readable; the window shows
  the refuser's own sentence (`NotRecounted::said`) and never an empty list.
  Any account shown before comes down.
- `NowhereToShow(NotRecounted)` — for the host's log; the window stays closed.

`RecordLook { scheme, scale, reading }` is `alo_appearance::Scheme`,
`alo_appearance::TextScale` and `alo_strings::Direction`.

`RecordFrame { window, strings, look }` borrows the window, the person's
`alo_strings::Strings` and the look.

## Seat

`Server::record_key(code, state, time)` takes a Linux evdev code through the
seat's XKB state and answers the `RecordKey` a press means, or `None` for a
release, a repeated press of a key already down and an unmatched release. Every
key is intercepted and none is forwarded to a client; keyboard focus is left
where it was. `InputError::InvalidKey` for a code outside 1–0x2ff,
`InputError::Unavailable` without a keyboard.

## Nested backend

`Nested::pump_record(server, window, recounting)` routes the parent window's
keys to the window while it is open and the parent has the keyboard, and returns
what each `ReadAgain` did. A host with a question open routes keys to the
approval surface first. `RenderError::Closed` and `RenderError::Input` as
`Nested::pump_approval`.

`Nested::submit_with_record(roots, popups, cursor, controls, labels, egress,
record, approval)` is `Nested::submit_with_egress_status` with the record window
painted after clients, popups and controls, an optional approval question
painted **above** it, and the egress indicator and the cursor above both.

- While the window is closed, nothing of it is drawn.
- An account is one tall panel centred across the output: first every sentence
  `Account::said` answers, one under another, then a rule, then the entries
  most recent first. Each entry is its clause (`Outcome::said`) at its head, then
  — set in from the side the person starts reading at — the agent's name where
  the record names one, the machine's sentence, the refusal's own reason, and
  where something went (`Destination::shown`), each wrapped at word boundaries
  and never shortened. An entry with no agent has nothing where a name would be.
- Every entry is drawn in the same type, the same two colours and the same
  spacing, whatever became of it. Colours are Cream and Navy (light) or Charcoal
  and Cream (dark); never terracotta.
- An entry that does not fit whole below the last waits for the view to move.
  When any entry is out of view, a rail on the trailing edge shows where the
  view is. The record's own sentences never move with the view.
- A refusal is a panel with its sentence and nothing else.

Errors, each refusing the whole frame before anything is drawn:

- `RenderError::RecordScene` — the window is open and cannot hold the record's
  own sentences and the entry at the top of the view whole, the output is too
  small for a panel, or either side exceeds 16,384 px.
- Every error `submit_with_egress_status` and `submit_with_approval` return; the
  indicator's refusal is checked first.

## Not in this surface

No filter, no search, no summary and no date: how a moment is written belongs to
the reader's region, and no crate decides that yet. No title: no crate declares
one. No key or dock item opens the window yet — `alo_shortcuts::Action` has no
action for it — so a host calls `opened_by_hand`. No pointer input. No
screen-reader tree. No direct-display (DRM/KMS) submission.
