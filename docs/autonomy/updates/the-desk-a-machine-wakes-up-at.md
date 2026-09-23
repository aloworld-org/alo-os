# The desk a machine wakes up at

**Date:** 2026-09-22
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 11
— *The desk a machine wakes up at*
**Contributor:** this development PC (Windows host, gates run in its Ubuntu)
**Status:** ready for integration.

## What changed, in words a person outside this repository can read

Close a laptop at home and open it at the office. Until today alo OS woke up
holding the screens it went to sleep with — the screen in the other room, which
is no longer plugged into anything, and not the one on the desk in front of you.
It learned about screens one cable at a time, and **a machine that was asleep
saw no cable move.**

Now a machine that wakes asks the whole question again: these are the screens
that are plugged in, this is the desk. If it is a desk you have arranged before,
it comes back the way you left it — the big screen on the left, at the size you
chose. If it is a desk it has never seen, the screens go side by side, each at a
size worked out from how large it actually is. Whatever was open on a screen
that is no longer there is on your main screen, and if a screen you had been
away from is plugged in again, what was open on it goes back to it.

You are told once, in one sentence: *your screens have changed since this
machine went to sleep, so alo OS has set up the ones in front of you now — the
arrangement you made at the other desk is still here for when you are back at
it.* The last clause is the part that matters and it is true: the desk you left
is remembered, exactly as it was, for the morning you are back at it.

**Two things that deliberately say nothing.** A machine that wakes at the same
desk, with the same screens saying the same things about themselves, moves
nothing and says nothing — the ordinary morning is the commonest resume there
is, and a sentence about it every day is a sentence nobody reads. And a machine
that wakes with nothing reporting itself at all — a dock that has not woken yet,
a monitor still negotiating — keeps the screens it went to sleep with rather
than throwing them away. A moment of seeing nothing must not cost somebody their
arrangement.

Nothing about this watches anything. The screens are read once, at the moment
the machine wakes, out of what the machine reported; there is no thread, no
timer and nothing listening.

## What changed, with paths

### `crates/alo-displays/src/resuming.rs` — new

The argument and the answer. `Resumed` is what a resume gives back:

- `moved()` — every screen that has gone while the machine was asleep, each as
  the same `Moved` an unplugged cable already answers with: the screen that
  went, the screen what was open on it belongs on, and every other screen whose
  windows were already sitting on the one that went and move with them;
- `came_back()` — every screen that is back and had been away, as the same
  `CameBack` a plugged-in cable answers with;
- `note()` — `Some(Note::TheDeskChanged)` when the set changed, and `None` when
  it did not;
- `the_same_desk()` — the ordinary morning, in one question.

The module documentation says what this file may not become: **not a new layout
rule.** Everything it uses was decided by task 3.

### `crates/alo-displays/src/attached.rs`

`Attached::resumed_to(reported, remembered)` — the one road, taking the **whole**
reported set. In order: which screens these are; whether they are exactly the
screens already held, each reporting itself exactly as before, in which case it
returns before touching anything; which have gone and which have arrived; then
task 3's `settled`, which restores an arrangement made for this set or lays the
set out side by side, and which is also where the refusal comes from. Three
small private helpers keep the chain right — `is_exactly`, `what_was_on` and
`what_goes_back_to` — each doing for a whole set what `unplugged` and
`plugged_in` do for one cable.

Five unit tests, the refusals beside the roads:

- `a_machine_that_woke_at_another_desk_is_set_up_for_it`;
- `a_desk_nobody_has_arranged_is_laid_out_side_by_side_at_a_resume`;
- `the_same_desk_at_a_resume_moves_nothing_and_says_nothing`;
- `a_machine_that_wakes_to_nothing_is_refused_and_keeps_its_screens`;
- `what_was_already_away_moves_with_the_screen_it_was_sitting_on` — the chain
  across a sleep, ending with the screen coming home.

### `crates/alo-displays/src/notes.rs` and `src/words.rs`

`Note::TheDeskChanged` and the string it says, `displays.the-desk-changed`, with
a translator's note that says when it is shown, that it is not a fault, and that
nothing the person arranged has been thrown away. `EVERY_WORD` goes from 42 to
43. It joined task 7's audit with that test unedited, because the audit reads
the assembled vocabulary by key area rather than any crate's list.

### `crates/alo-sleeping/src/the_desk.rs` — new

`Woke::the_desk(attached, reported, remembered)`, which is where the road is
asked. It is a pass-through by design: which screens there are and where each
goes is `alo-displays`', and what is this crate's is the **when** — at a wake,
once, from the set the caller was handed. Its own file rather than a method on
`waking.rs`, which is about the seat and the agent's turn; two reasons to change
one file is the thing `CLAUDE.md`'s fourth law forbids. Two unit tests, the road
and the refusal.

### `crates/alo-sleeping/Cargo.toml`

`alo-displays` moves from a dev-dependency to a dependency, with the comment
saying why. **This breaks no rule.** The three crates that refuse dependants —
`alo-sleeping`, `alo-leaving`, `alo-notifying` — each refuse *their own*, and
`alo-sleeping`'s list is unchanged and unwidened;
`tests/an_agent_cannot_keep_this_machine_awake.rs` passes untouched.
`alo-displays` holds no such rule, which task 7 already relied on.

### `crates/alo-sleeping/tests/the_desk_a_machine_wakes_up_at.rs` — new

One laptop, four resumes, with a real sign-in, a real lock and a real sleep
under each: at another desk where a screen it has never seen is plugged in; at
the same desk; where nothing at all reports itself; and back at the first desk,
where the arrangement she made is restored and the screen she had been away from
takes its windows home. Every sentence is pulled out of the machine's one
assembled vocabulary and checked for coming out whole and for naming no
connector.

### `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`

Task 11 marked **Done, 2026-09-22**, and **task 12 written**, because the plan
named none after it and a plan with no next task reads to the loop as a
workstream that is finished.

## Decisions this task made, and why

**The whole set, never a cable at a time — and the signature says so.**
`resumed_to` takes `Vec<Reported>` rather than a difference the caller worked
out. A caller that computes *what changed* is a caller that can get it wrong,
and it would be wrong in exactly the way this task exists to fix: a machine that
was asleep has no events to difference. The crate is handed what is true now and
does the comparing itself.

**A same-desk resume returns before touching anything.** It would have been
simpler to settle unconditionally and let the answer come out empty. It is not
the same thing: settling replaces the places, the notes and the rounding, and a
screen still reporting itself identically has no business being laid out again.
`the_same_desk()` is therefore a fact about the machine, not a summary of an
answer, and the test holds the whole `Attached` equal before and after.

**A refusal changes nothing, and that is load-bearing.** The set is checked and
laid out *before* anything is written into `Attached`, so `NotArranged` leaves
the screens the machine slept with exactly as they were — down to the places and
the last thing the person was told. The next resume is compared against them, so
a machine that forgot them on a bad read would announce a changed desk the
moment the dock woke up, and would have lost the arrangement to compare with.

**Screens that come back are answered too, though the acceptance named only the
ones that go.** Without it, a screen that had been unplugged before the sleep
and is plugged in at the new desk would leave a stale entry in the away list,
and a later unplug would carry windows that had already gone home. The existing
`CameBack` is the shape for it, so nothing new was invented.

**Gone screens are processed before returning ones**, which is what an unplug
followed by a plug-in would do, in that order — the chain the crate already
blesses. The rare case it makes visible is a screen whose windows were sitting
on a screen that has gone and which is itself back: they move to the new main
screen and then home, two moves rather than one. That is honest about what
happened and needs no special case; a shell applying the answer in the order it
is given arrives at the right place.

**`Note::TheDeskChanged` says nothing about which screens.** It has no gap, so
it can never carry a screen's name — and the sentences that *do* name screens
are the `Moved` and `CameBack` ones beside it, which already exist and are
already translated. One sentence about the desk plus the crate's ordinary
sentences about screens beat one long sentence that tries to be both.

**Where task 12 came from.** The plan is otherwise finished, and the honest next
piece of work is the one thing this workstream holds itself to that task 11
could not reach: task 7 records the **sequence** a person meets, in a table its
own test reads, and the sentence added here appears in no recorded sequence. A
resume at a new desk can say four things at once, and whether those four read as
one account is a question none of the tests written so far can ask. Task 12 also
carries the debt this plan leaves — `docs/features.md`'s *Per display, so the
dock can sit along the bottom of the laptop and down the side of the external
screen* is still **not met**; `alo_dock::Dock` holds one edge for the machine,
`alo-dock` belongs to `v0-5-where-a-persons-settings-are-kept-plan.md`, and
`alo_displays::Wearing::of` is the one function that changes when that plan
decides otherwise. It is written down rather than narrowed.

## Acceptance, and the evidence for each

The workspace is the product's (`.`). Each was run on its own, on the tree being
published.

| Acceptance criterion | Crate | Target | Test |
|---|---|---|---|
| One road from a machine that slept to the screens in front of it, restoring the arrangement the person made for this set and saying where what was on a screen that is gone belongs, in the shape an unplug answers with | `alo-displays` | `lib` | `attached::tests::a_machine_that_woke_at_another_desk_is_set_up_for_it` |
| A set nobody has arranged is laid out side by side, and a screen that is still there is not moved | `alo-displays` | `lib` | `attached::tests::a_desk_nobody_has_arranged_is_laid_out_side_by_side_at_a_resume` |
| A machine that wakes to the same set reporting itself the same way moves nothing and says nothing | `alo-displays` | `lib` | `attached::tests::the_same_desk_at_a_resume_moves_nothing_and_says_nothing` |
| A machine that wakes with nothing plugged in is refused `NotArranged::NoScreens`, with the screens it slept with left exactly as they were | `alo-displays` | `lib` | `attached::tests::a_machine_that_wakes_to_nothing_is_refused_and_keeps_its_screens` |
| What was already away moves with the screen it was sitting on, and comes home when that screen is back | `alo-displays` | `lib` | `attached::tests::what_was_already_away_moves_with_the_screen_it_was_sitting_on` |
| The `Note` a person reads is declared, says a whole sentence with no gap unfilled, and the answer carries it only when the desk changed | `alo-displays` | `lib` | `resuming::tests::another_desk_says_one_declared_sentence_and_names_what_moved` |
| Every note this crate makes, the new one included, says a declared sentence with every gap filled | `alo-displays` | `lib` | `notes::tests::every_note_says_a_declared_sentence_with_every_gap_filled` |
| The new sentence is collected into the machine's one vocabulary, with this crate's own English | `alo-displays` | `every_sentence_here_is_collected` | `every_sentence_this_crate_can_say_is_in_the_machines_vocabulary` |
| It joins task 7's audit — a translator's note, nothing rented, no connector name — without that test being edited | `alo-sleeping` | `every_sentence_this_workstream_says` | `every_sentence_carries_a_note_a_translator_can_work_from` |
| `alo-sleeping`'s `Woke` is where the road is asked | `alo-sleeping` | `lib` | `the_desk::tests::the_desk_is_asked_at_the_resume` |
| …and the refusal is answered there too, with the screens kept | `alo-sleeping` | `lib` | `the_desk::tests::a_resume_with_nothing_plugged_in_is_refused_and_keeps_the_screens` |
| One test suspends at one desk and resumes at another, at the same one, and at none | `alo-sleeping` | `the_desk_a_machine_wakes_up_at` | `a_laptop_suspended_at_one_desk_and_opened_at_another` |

Two of these name tests whose files existed before this change —
`every_sentence_here_is_collected.rs` and `every_sentence_this_workstream_says.rs`
— because what they show is precisely that the new sentence joined them with
**nothing edited**. Their crates' own files are part of this change; the
evidence block names the tests that live in files this change publishes, and
those two rows are recorded here as read by a person rather than offered as
mechanical evidence.

## Verification

Run from this checkout's Windows host, in its Ubuntu, against the synchronized
Linux source copy at `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, as `SHARED_MAIN.md` requires.
The gate turn was free and no other Cargo process was running.

| Check | Result |
|---|---|
| `cargo fmt --all --check`, every workspace member | clean |
| `cargo clippy -p alo-displays -p alo-sleeping --all-targets -- -D warnings` | clean, exit 0, zero warnings |
| `cargo test -p alo-displays` | 151 passed, 0 failed |
| `cargo test -p alo-sleeping` | 65 passed, 0 failed |
| `cargo test -p alo-saying` (it collects the new sentence) | 68 passed, 0 failed |
| `cargo check -p alo-shell --all-targets` (the other crate that reads `alo-displays`) | clean |
| `cargo test -p alo-citing` (the citation check, since `docs/` changed) | 31 passed, 0 failed |
| `cargo test -p kernel-loop every_plan_this_repository_drives_holds_only_tasks` (the plan checks, since a plan changed) | passed |
| `cargo doc -p alo-displays -p alo-sleeping --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |

**The whole-workspace suite was not run here**, by instruction: it takes the
better part of an hour on this machine and the supervisor runs it after this
task regardless. What was run instead is every crate this change touches, both
crates that read the one whose public surface grew, and the two checks the
scoping table in `SHARED_MAIN.md` names for a change under `docs/`.

## Limitations, and what is not claimed

**Nothing has slept.** No lid has closed and no cable has moved on certified
hardware, or on any hardware. The `Logind` under the walk is a value in the test
file that counts what it was asked, which is the boundary every test in this
plan draws and which the plan itself requires. The sentence in the plan's *What
this plan may not do* stands: this is `- [x] The code.` and nothing more.

**Nothing draws.** The moving of a window is the shell's, as it is for an
unplugged cable; this says *onto which screen*, and there is no window
identifier anywhere in `alo-displays`.

**Who calls it.** `Woke::the_desk` is the door; the session that holds the seat
is what calls it with what the compositor reports, and that caller is the shell
plan's. Nothing here polls for it.

**A screen's windows stay away for as long as the session does.** A screen that
has gone keeps its place in the away list until it is plugged in again, which is
task 3's behaviour and is unchanged — a resume simply makes it happen more
often. Whether *what was open there* should ever stop being remembered is a
question nobody has asked yet; it is not a fault found and left, and it is not
this task's to answer.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` have one writer (`SHARED_MAIN.md`).

**`CHANGELOG.md`, under the unreleased v0.5 entries:**

> **A machine that wakes up at a different desk.** A laptop closed at home and
> opened at the office now sets itself up for the screens in front of you rather
> than the ones it went to sleep with: an arrangement you made for exactly these
> screens comes back as you left it, screens it has never seen go side by side at
> a size worked out from how large each is, and whatever was open on a screen
> that is no longer there is on your main screen. You are told once that the desk
> changed, and the arrangement for the desk you left is kept for when you are
> back at it. A machine that wakes at the same desk says nothing at all, and one
> that wakes before anything has reported itself keeps the screens it had.

**`ROADMAP.md`:** no line ticks. *Lock screen, suspend and resume* and
*Multi-monitor, scaling, hotplug* both gain code toward them and neither is
measured on a machine.

**`docs/autonomy/QUEUE.md` / `STATE.md`:** task 11 of
`v0-5-the-session-and-the-displays-plan.md` is done, and task 12 — *Every
sentence at a desk that changed, and what this plan still owes* — is ready and
depends on 7 and 11.
