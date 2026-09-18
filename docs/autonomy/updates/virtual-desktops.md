# Virtual desktops

**Date:** 2026-09-17
**Workstream:** v0.5 — hands on the desktop (`docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, task 3)
**Contributors:** two Claude Code workers in `C:\dev\alo-os-b`, for the repository
owner — the crate, and then the repair of the gate that refused it (below)
**Status:** ready for integration

## What changed

A new crate, `crates/alo-desktops`, holds a person's virtual desktops. It draws
nothing; the shell's later tasks draw the desktop a person is on, and task 5's
gestures hand it the same switch a chord does.

**In words a person outside this repository can read:** alo OS now has virtual
desktops. Each screen has its own row of them — add one, name it in your own
words, drag it along the row, remove it — and each keeps its own windows and its
own split of the screen, so a pair of windows split into halves on one desktop
stays split there while another desktop holds something else. Removing a desktop
never closes a window: they move to the desktop beside it, and so do you if you
were on it. A window can be on every desktop at once. `Super+PageDown` and
`Super+PageUp` move along the row and `Super+1`…`Super+9` go straight to a
desktop; if you have given one of those combinations to a keyboard shortcut of
your own, the shortcut keeps it and alo OS says so rather than quietly taking it.
And the three things alo OS promises are always there — what is leaving the
machine, what is waiting for your approval, and the agent — are on **every**
desktop at once and cannot be moved onto one or closed, because a promise you
could only see on desktop 1 would be no promise at all.

| File | Responsibility |
|---|---|
| `src/display.rs` | Which display a row of desktops belongs to, and the two the compositor can get wrong |
| `src/desktops.rs` | Every display's desktops, and the person's chords |
| `src/on_a_display.rs` | One display's row: add, remove, name, reorder, switch, where a window is |
| `src/desktop.rs` | One desktop: its name, its windows and its division |
| `src/naming.rs` | What a person calls a desktop, and the three ways what they typed is not a name |
| `src/position.rs` | Where a desktop sits in the order, counted from one |
| `src/switching.rs` | The closed set of ways a person asks for another desktop |
| `src/chords.rs` | Those ways from the keyboard, and why `alo-shortcuts` still wins |
| `src/always.rs` | The three promises, and why they cannot be confined to one desktop |
| `src/refusing.rs` | Why nothing changed |
| `src/words.rs` | The crate's 19 strings, each with a translator's note |

Registration: the crate is a workspace member, and its words are collected by
`crates/alo-saying/src/collecting.rs` (53 → 54 lists).

## Decisions

The task left several things open. Each was decided here rather than handed back.

1. **Desktop chords are this crate's, and `alo-shortcuts` is untouched.** The
   acceptance asks for switching by shortcut; this plan reads `alo-shortcuts` and
   never edits it. There is a reason stronger than the plan: `Action` is a closed
   list of fixed things the system does, and *go to desktop 4* is not one of
   them — it is parameterised by a desktop that may not exist yet, and nine
   variants of it would be nine spellings of one variant. So `DesktopChords`
   holds `Vec<(Chord, Switch)>` here, ships eleven chords (`Super+PageDown`,
   `Super+PageUp`, `Super+1`…`Super+9`), and **asks the person's own shortcuts
   first**: `switch_for` answers `None` the moment `Shortcuts::action_for` says
   the chord already does something, and `bind` refuses it with
   `Refused::ChordIsTaken`, naming the action. A system shortcut is a key the
   compositor takes before anything else sees it, so a desktop chord that
   shadowed one would be a chord that never arrives. A test holds the shipped
   eleven against `alo_shortcuts::Defaults`' real list rather than a copy of it,
   so a default moved there fails here instead of silently stopping.
2. **The three promises are stated when a display is plugged in.** `Promises::of`
   names the egress indicator's, approval surface's and agent overlay's windows,
   and `Desktops::plug_in` requires it — so there is no moment at which a display
   has desktops and one of the three is missing, and no flag anybody can forget
   to set. Every road that could move a window (`put_on`, `on_every_desktop`,
   `close`) refuses one with `Refused::APromise`. The promise is structural
   rather than a rule to remember, which is what the acceptance's reason asks
   for. Two promises named as one window is refused where it is stated
   (`TwoPromises`): one surface standing for two would mean dismissing one
   dismissed the other.
3. **Switching does not wrap.** Past either end of the row is
   `Refused::NoDesktopThatWay`, not the desktop at the other end. A swipe is the
   one gesture people make without looking, and wrapping reads as a bug the first
   time it happens to somebody with two desktops. Asking for the desktop a person
   is already on is not a refusal — it is where they are, and it is the answer.
4. **Windows moved by a removal arrive floating.** The neighbour's division is an
   arrangement that person made; dropping three windows into it would rearrange
   windows they were not looking at. Dividing them again is one drag each. The
   removed desktop's own division goes with it, and a person who removes the
   desktop they were on lands on the neighbour that took their windows — the
   identity of which is `remove`'s answer, so the shell can show where they went.
5. **A window on every desktop has no share in any division.** A share belongs to
   one desktop's arrangement, and a window that is everywhere cannot be in one
   arrangement and not another. `on_every_desktop` therefore takes it out of the
   division it was in, and `put_on` is the road back onto exactly one desktop.
6. **Sixteen desktops per display** (`MOST_DESKTOPS`), refused rather than
   silently capped. More than anybody has been seen to use, few enough that the
   row is one a person can count along, and enough of a limit that *add* cannot
   be held down into a list nothing can draw. A chord for a desktop beyond it is
   refused at binding time, because it could never work.
7. **A display always has at least one desktop.** Made with one, and removing the
   last is `Refused::TheLastDesktop`. A person who has never heard of virtual
   desktops has one, uses it, and never learns the word.
8. **`DisplayId` is a live handle, not a stable identity.** `alo-displays` does
   not exist yet (the session plan's task 3, which task 2 here is blocked on), so
   a display is the number the compositor uses while it is plugged in.
   `Desktops::unplug` hands a display's desktops back **whole** rather than
   dropping them, so whoever remembers them has something to remember — but
   nothing here writes to disk. Keeping desktops, names, order and chords across
   a session needs that stable identity and is task 2's, and is listed under
   limitations below rather than half-done here.
9. **A desktop's name is trimmed, not refused, for surrounding space**, is
   refused for control characters and beyond 40 characters (counted as a person
   counts them, so *Müller* is six), and is refused when another desktop on the
   same display already has it — two desktops with one name is a list nobody can
   choose from. There is no rule about which script a name is in.
10. **`Desktop::shown` answers a `String`, not a `Said`.** A person's own name for
    a desktop is their words in their language, and a provenance on it would be a
    claim about a translation nobody was going to make; the same shape and the
    same reason as `alo_shortcuts::Chord::shown`. Whether the *fallback*
    (*Desktop 3*) was translated is `Desktop::numbered`'s `Said` to answer.
11. **No sentence names a desktop, a window or a chord.** A refusal carries what
    it is about (`Refused::desktop`, `Refused::window`) and the shell marks it —
    `alo-dividing`'s stance, and a unit test holds it by showing that two
    refusals about two different desktops read identically.
12. **The two failures a person never meets keep their English**: `NotADisplay`
    and `TwoPromises` mean the shell and this crate disagree about what exists,
    which is alo OS's own bug with nothing to ask a person — the precedent is
    `alo_dividing::AreaError`.

No ADR was needed: nothing here contradicts one, weakens a gate, widens a grant,
adds an `unsafe` block, or narrows `docs/features.md`.

## Acceptance criteria and evidence

All in `crates/alo-desktops/tests/`:

| Criterion | Test |
|---|---|
| Holds a person's desktops per display — add, remove, name, reorder — and which windows are on each | `desktops_a_person_arranges::a_person_adds_removes_names_and_reorders_their_desktops` |
| Removing a desktop moves its windows to a neighbour and never closes one | `desktops_a_person_arranges::removing_a_desktop_moves_its_windows_to_a_neighbour_and_closes_none` |
| A window can be on every desktop | `desktops_a_person_arranges::a_window_can_be_on_every_desktop` |
| Switching is a gesture or a shortcut | `desktops_a_person_arranges::switching_is_a_gesture_or_a_shortcut` |
| Each desktop keeps its own divisions | `desktops_a_person_arranges::each_desktop_keeps_its_own_divisions` |
| Per display, independently (`docs/features.md` v0.5) | `desktops_a_person_arranges::desktops_are_per_display` |
| The egress indicator, the approval surface and the agent overlay are on every desktop at once | `every_promise_is_on_every_desktop::the_indicator_the_approval_and_the_agent_are_on_every_desktop` |
| …and cannot be put on one, pinned, or closed | `every_promise_is_on_every_desktop::a_promise_cannot_be_put_on_one_desktop_or_closed` |
| One surface cannot stand for two promises | `every_promise_is_on_every_desktop::one_surface_cannot_be_two_promises` |
| Every sentence this crate can say is declared, with a translator's note | `desktops_a_person_arranges::every_sentence_about_a_desktop_is_declared` |

Refusal paths have unit tests beside each operation as well
(`src/on_a_display.rs`, `src/chords.rs`, `src/naming.rs`, `src/refusing.rs`,
`src/always.rs`, `src/desktops.rs`): the last desktop, a seventeenth, a name
another desktop has, a position there is no desktop at, either end of the row, a
desktop of another display, a window on no desktop here, a chord a system
shortcut holds, a chord another switch holds, a chord for a desktop beyond the
cap, a display plugged in twice, a display unplugged twice, and each of the three
ways what a person typed is not a name.

## Verification

Executed on 2026-09-17 in WSL Ubuntu
(`CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), from the
checkout:

- `cargo fmt --all` — clean;
- `cargo clippy -p alo-desktops -p alo-saying -p alo-collected --all-targets -- -D warnings`
  — clean;
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-desktops --no-deps` — clean;
- `cargo test -p alo-desktops` — 35 unit, 7 + 3 integration, 1 doc test, all pass;
- `cargo test -p alo-saying` — all pass (the machine's one vocabulary now holds
  this crate's 19 strings);
- `cargo test -p alo-collected` — all pass (the new crate's words are collected
  rather than stranded).

Not run by this worker, by instruction: the full workspace test suite — the
supervisor runs it.

**Not done, and not claimable here: nothing has been swiped on a real machine.**
There is no compositor drawing a desktop yet, so the physical check belongs to
the shell plan's task that honours this crate, and to task 5, which turns a
touchpad swipe into the `Switch` this crate carries out. Nothing in this change
ticks anything *on the machine*.

## Limitations

- **Nothing is kept on disk.** Desktops, their names, their order and the
  person's desktop chords live for the session. Keeping them needs the stable
  display identity `alo-displays` will give (the session plan's task 3), which is
  what task 2 of this plan is blocked on; `Desktops::unplug` hands a display's
  desktops back whole so that task has something to keep. The acceptance for
  task 3 is in-memory behaviour and is met; persistence is task 2's and is not
  quietly claimed here.
- **A desktop's division has a fixed display area.** What happens when a
  display's resolution or scale changes is `alo-displays`' and task 2's, as it
  already is for a single division.
- **Nothing routes a gesture yet.** `Switch::Next` and `Switch::Previous` are the
  values a three- or four-finger swipe will produce; deciding them from
  `libinput`'s events is task 5.
- **The keyboard reaches nine desktops, not sixteen**, because there is no
  `Super+10`. A tenth is reached by the gesture or by a chord the person binds.
- **This crate holds which desktop a window is on; `alo-dividing` holds where on
  the screen it is.** A shell puts a window on a desktop and then divides that
  desktop's division; nothing here can stop a caller dividing a window it never
  put on the desktop, and a type that could would have meant wrapping
  `alo-dividing`'s whole surface.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "Virtual desktops: a row of them per screen — add, name,
  reorder, remove — each with its own windows and its own split of the screen.
  Removing a desktop moves its windows to the one beside it and closes nothing; a
  window can be on every desktop; and the egress indicator, the approval surface
  and the agent overlay are on every desktop at once and cannot be moved off one
  (`alo-desktops`; nothing drawn yet, and nothing kept across a session)."
- **ROADMAP.md, v0.5 *Input: … virtual desktops …*:** the deciding code exists;
  nothing is ticked *on the machine*, and the swipe that switches is task 5.
- **QUEUE.md / STATE.md:** hands-on-the-desktop task 3 done; task 5 (gestures) is
  now unblocked as well as ready; tasks 4 and 6 ready; task 2 still blocked on the
  session-and-displays plan's task 3; task 7 waits on 4, 5 and 6.

## Recovery onto current main

Recovered on 2026-09-18 without replacing later workspace members or vocabularies.
The vocabulary list and its independent test fixture now use slices: their lengths
follow the entries, so parallel additions no longer require conflicting edits to
numeric array lengths. Existing tests still compare the names against declarations
and the collected vocabulary against the individual vocabularies. Full nine-gate
validation remains the supervisor's publication requirement.

The historical `alo-measuring` race repair is excluded from this recovery: the
crate belongs to the machine-measured plan. That proposed repair and its prior
evidence remain preserved in `parked/task-3-1789691941` for its owner.
