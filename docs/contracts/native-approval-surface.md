# Native approval surface

Status: additive trusted Rust API in `alo-shell`, 2026-09-14. ADR 0001 §5;
ADR 0002. No protocol, agent capability, D-Bus interface or stored format
changes. Whether a change may be proposed, what its sentence says and what one
answer does are `alo-capability`'s, `alo-turn`'s and `alo-approving`'s; this
document describes only the surface that puts the question in front of a person
and carries their keys back. Nothing here is reachable by an agent.

## Values

`ApprovalScreen` is the surface for **one turn**: a question's number means
something only inside the turn that asked it, so the host makes one when a turn
begins and drops it when the turn ends, and hands that `alo_turn::Turning` to
every call. It holds an `alo_approving::Approving` and implements
`alo_approving::Compositor` internally; there is no way to put a question on it
except a change the turn is really waiting on.

- `ApprovalScreen::on_an_output()` puts questions up.
- `ApprovalScreen::with_no_output()` refuses every question with
  `SurfaceRefused::NothingToShowOn` and answers none.
- `arrived(&turning, id, now)` — a proposal has arrived. Put up at once when
  nothing is open; otherwise **kept behind** the open question or refusal, in
  arrival order. A number already open or kept is ignored.
- `pressed(key, &mut turning, &grants, now)` — one `ApprovalKey`:
  `NextAnswer` (Tab) and `PreviousAnswer` (Shift+Tab) select an answer on an
  open question; `Choose` (Enter, Space) gives the selected answer, does
  nothing while none is selected, and acknowledges an open refusal; `Nothing`
  is every other key, Escape and letters included.
- `answer(answer, &mut turning, &grants, now)` — give `ApprovalAnswer::No` or
  `ApprovalAnswer::Approve` to whatever is open. Always forwarded to
  `Approving::decline` or `Approving::approve`; a second answer to one question
  comes back as `alo-approving`'s `NotAnswered::NothingToAnswer`.
- `looked_again(&turning, now)` — read the open question off the turn again
  before a frame. Answers nothing; a question that lapsed or was answered
  another way comes down and its refusal goes up.
- `shows()` — `ApprovalShows::Nothing`, `Question { asked, selected }` or
  `Refusal(&Said)`. Every question goes up with `selected: None`.
- `is_open()`, `waiting_behind()`.

Every call that can change something returns `ApprovalOutcome { answered,
nowhere_to_show }`: what an answer did (`alo_approving::Answered`, handed on
whole), and questions refused because there is nowhere to show them, for the
host's log.

After an answer:

- **Carried** or **Declined** — the question comes down, nothing is said, and
  the next kept question goes up with nothing selected.
- **Refused** by the turn — the refusal's own sentence goes up and the next
  question waits until `Choose` acknowledges it. A refusal that ends the turn
  forgets every kept question.

`ApprovalAnswer::IN_READING_ORDER` is `[No, Approve]`.

`ApprovalLook { scheme, scale, reading }` is `alo_appearance::Scheme`,
`alo_appearance::TextScale` and `alo_strings::Direction`.

`ApprovalFrame { screen, strings, look }` borrows the surface, the person's
`alo_strings::Strings` and the look.

## Seat

`Server::approval_key(code, state, time)` takes a Linux evdev code through the
seat's XKB state and answers the `ApprovalKey` a press means, or `None` for a
release, a repeated press of a key already down and an unmatched release. Every
key is intercepted and none is forwarded to a client; keyboard focus is left
where it was. `InputError::InvalidKey` for a code outside 1–0x2ff,
`InputError::Unavailable` without a keyboard.

## Nested backend

`Nested::pump_approval(server, screen, turning, grants, now)` routes the parent
window's keys to the screen while it is open and the parent has the keyboard,
and returns each non-empty outcome. `RenderError::Closed` and
`RenderError::Input` as `Nested::pump_sign_in`.

`Nested::submit_with_approval(roots, popups, cursor, controls, labels, egress,
approval)` is `Nested::submit_with_egress_status` with the approval surface
painted after clients, popups and controls and **before** the egress indicator
and the cursor.

- While nothing is open, nothing is drawn.
- A question is one panel centred on the output: the agent's name, the sentence
  exactly as `Asked::sentence` holds it — wrapped at word boundaries, never
  shortened or cut — and two answer boxes of equal size worded by
  `Asked::no_said` and `Asked::approve_said`, *no* first in reading order,
  at the far end of the line, mirrored for right-to-left; stacked when they do
  not fit side by side.
- The selected answer has a thicker edge and a bar under its words. Colours are
  Cream and Navy (light) or Charcoal and Cream (dark); never terracotta.
- A refusal is the panel with its sentence and no answers.

Errors, each refusing the whole frame before anything is drawn:

- `RenderError::ApprovalScene` — something is open and the panel cannot hold the
  whole sentence and both answers on this window, the window is too narrow for
  a panel, or either side exceeds 16,384 px.
- Every error `submit_with_egress_status` returns; the indicator's refusal is
  checked first.

## Not in this surface

No pointer input: answers are given by keyboard, or by a host calling
`answer`. No visible control acknowledges a refusal, because no crate declares
a word for one; Enter does. No countdown is drawn. No screen-reader tree. No
direct-display (DRM/KMS) submission.
