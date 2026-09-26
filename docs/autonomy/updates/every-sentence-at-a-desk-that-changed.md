# Every sentence at a desk that changed

Task 12 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, done
2026-09-26 on the third PC.

Task 7 walks one person from locking their machine to docking it at another desk,
and holds the **sequence** to a table in its report. Its steps 6 and 7 — the sleep
and the wake — say nothing, which was right when nothing had been decided about a
desk that changed while the machine was asleep. Task 11 decided it and added
`displays.the-desk-changed`, a sentence **no recorded sequence met**.

This is that sequence, in `crates/alo-sleeping/tests/the_walk_to_a_desk_that_changed.rs`.
Task 7's walk and its table are untouched: a published report is never rewritten,
and that walk is still true.

## The walk, sentence by sentence

| When | What she reads |
|---|---|
| Anna signs in at her own desk, with her own screen beside the laptop | Built-in screen has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| Anna signs in at her own desk, with her own screen beside the laptop | Built-in screen does not say which screen it is, so alo OS remembers it by the socket it is plugged into — another screen plugged into that socket will be set up the same way |
| Anna signs in at her own desk, with her own screen beside the laptop | Iiyama ProLite XU2793 has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| she opens the lid at the office, where the screens are not the ones she left | Your screens have changed since this machine went to sleep, so alo OS has set up the ones in front of you now — the arrangement you made at the other desk is still here for when you are back at it |
| she opens the lid at the office, where the screens are not the ones she left | Dell U2720Q has not been used with this machine before, so alo OS has put it beside your other screens and chosen a size for it from how large it is |
| she opens the lid at the office, where the screens are not the ones she left | Iiyama ProLite XU2793 was unplugged, so what was open on it is now on Built-in screen |
| she opens it again at her own desk, where her own screen is back | Your screens have changed since this machine went to sleep, so alo OS has set up the ones in front of you now — the arrangement you made at the other desk is still here for when you are back at it |
| she opens it again at her own desk, where her own screen is back | Your screens are arranged the way you last left them |
| she opens it again at her own desk, where her own screen is back | Dell U2720Q was unplugged, so what was open on it is now on Built-in screen |
| she opens it again at her own desk, where her own screen is back | Iiyama ProLite XU2793 is back, so what was open on it before has gone back to it |

Ten sentences, from three sources at once — the notes on `Attached`, the screens
`Resumed` says have gone, and the ones it says are back. Each comes out of the
machine's one assembled vocabulary through the value that really produces it: a
`Note`, a `Moved`, a `CameBack`. No sentence arrives with a gap unfilled and **no
gap holds a connector name**: `eDP-1`, `HDMI-1` and `DP-1` are the machinery, and
what a person reads is *Built-in screen*, *Iiyama ProLite XU2793*, *Dell U2720Q*.

## The criterion this was read against

Read in the order the machine emits them, by somebody who does not know which
crate said which: **does a reader learn what happened and what, if anything, they
should do?** Two ways that fails, both findings rather than a rewording — an
arbitrary order, because an account has a sequence and a set does not; and any two
sentences restating one fact in different words, because a person reads that as
two events.

**Steps 1 and 2 pass.** The office morning reads as one account, in a good order:
the frame first (*your screens have changed, and what you arranged elsewhere is
safe*), then the new screen placed, then where her open work went. A person learns
what happened and that nothing needs doing.

**Step 3 does not.** It is the second failure mode, and this is the finding.

## Finding 1 — a vocabulary finding: the reassurance is wrong when she comes back

Two sentences arrive together on the return:

> Your screens have changed since this machine went to sleep, so alo OS has set up
> the ones in front of you now — **the arrangement you made at the other desk is
> still here for when you are back at it**

> Your screens are arranged **the way you last left them**

Read one after the other they restate one fact — that alo OS has laid her screens
out from something it kept — in different words, so a person reads two events
where there was one. Worse, the first sentence's closing clause is **wrong at this
moment**: it promises that the arrangement made *at the other desk* is waiting for
when she is *back at it*, and she is back at it. She is standing at the desk the
reassurance is about, reading a sentence that speaks as though she were away from
it.

Neither sentence is wrong on its own, and each is right where it was written. The
clause in `displays.the-desk-changed` was added for the everyday case task 11
names — *a laptop closed at home and opened at the office* — where it is exactly
the right thing to say. What it does not survive is the other half of the same
journey.

**This is about the vocabulary, not about the crates.** `alo-displays` emits both
from `Attached::notes()` and the two are decided in different places:
`Note::TheDeskChanged` is `resumed_to`'s, and `Note::AsYouLeftThem` is the
arranging's. **Nothing here was reworded and no crate was made to reach into
another** — a crate coordinating another's phrasing would put a second author on a
sentence that already has one. The question for whoever takes it is whether
`the-desk-changed`'s reassurance belongs in that sentence at all, or whether it is
a second sentence said only when the desk she has arrived at is *not* one whose
arrangement is kept.

## Finding 2 — an ordering finding: the machine has an opinion about part of the order only

`Attached::resumed_to` does `notes.insert(0, Note::TheDeskChanged)` — the
desk-changed sentence leads its list, deliberately, and that is an ordering
decision recorded in code.

Across the three sources there is no such decision. `Resumed` is a value with
three fields — `moved`, `came_back`, `note` — and `Attached::notes()` is a fourth
door onto the notes. A caller holds three collections and nothing says which comes
first. **The order in the table above is this walk's own choice** (notes, then
what went, then what came back), not the machine's, and two surfaces could show
this same morning in two different orders with both being correct.

An account has a sequence. Half of this one is fixed in `alo-displays` and half is
left to whoever draws it, which means the account exists only when the surface
happens to compose it — and nothing tells the next surface what task 11 already
decided about the first sentence.

## What this plan still owes

The plan is otherwise finished, which is worth writing down rather than leaving
somebody to reconstruct from eleven status paragraphs.

**One promise it named is not met.** `docs/features.md`: *Per display, so the dock
can sit along the bottom of the laptop and down the side of the external screen.*
`alo_dock::Dock` holds one edge for the machine, not one per screen.

- **Whose it is:** `crates/alo-dock` belongs to
  `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`. This plan reads
  `alo-dock` and never edits it, so the promise is owed rather than narrowed.
- **What changes when it is paid:** `alo_displays::Wearing::of` is the one
  function in this plan's crates that changes, named by task 3 in its own status
  paragraph.

A promise owed to another plan is still owed. It is recorded here because this is
the last task of the plan, and the place to read it is before somebody calls the
plan done.

## What this task did not do

No sentence was added, reworded or moved. Nothing in `crates/alo-shell`, nothing
on the machine, `logind` stays rented (ADR 0011), and task 7's report and table
are exactly as they were published. Both findings above are arguments for a later
task to take with its own reasoning, not edits this one made quietly.

## A note on what the walk had to model to be honest

Written first with `Changes::untouched()` throughout, the walk had alo OS tell
Anna on the third morning that *Iiyama ProLite XU2793 has not been used with this
machine before* — the screen she had arranged at the start and used all day — and
say the same of the built-in panel at every wake, three times, for a screen that
had never been unplugged.

That was a finding about the test, not about alo OS: a machine that remembers
nothing has nothing to recognise. The walk now keeps what alo OS worked out, with
`Changes::remember`, as a session does — one arrangement per set of screens, which
is what `Changes::for_screens` is keyed by. The repetition went, and *Your screens
are arranged the way you last left them* appeared, which is how finding 1 was
found at all. **A walk whose own setup is unlike a machine's produces sentences
nobody will ever read, and hides the ones they will.**

## Evidence

    cargo test -p alo-sleeping --test the_walk_to_a_desk_that_changed    3 tests, 0 failed

Three tests: the walk is the sequence this report records, read out of this file
rather than a copy; the sentence task 11 added is met; and no sentence is said
twice at one moment. The first fails if a sentence changes without the table, and
if the table is edited to say something the machine does not — and it prints the
whole measured sequence when the counts differ, so the next person writes the
table from the walk rather than the walk from the table.

probe_should_be_7=7
