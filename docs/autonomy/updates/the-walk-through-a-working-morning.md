# Every sentence, and the walk through a working morning

**Date:** 2026-09-20
**Workstream:** v0.5 — hands on the desktop
**Task:** 7, *Every sentence, and the walk through a working morning* — the last
task of that plan.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3 with 8 GB unified memory**, macOS 26.5.2; built, linted
and tested in the Lima VM on that Mac — **Ubuntu 24.04.4 aarch64, 6 CPUs, 4 GB**.
Nothing here is ticked *on the machine*: these crates decide and draw nothing.
**Egress:** none.

## What this task is for

Tasks 1 to 6 each end in something a person reads: a half, a desktop's name, a
label beside a pointer, a letter that appears when two keys are pressed. Each
crate's own tests hold its own.

**What none of them can show is the sequence** — whether somebody working
through an ordinary morning is carried from one sentence to the next, or handed
five crates' worth of individually correct sentences that do not join up.

## The morning, sentence by sentence

| Step | The moment | What a person meets |
|---|---|---|
| 1 | mail is put on the screen | Left half |
| 2 | the editor is put on the screen | Right half |
| 3 | the boundary between them is dragged | Part of the screen |
| 4 | the editor is sent to a second desktop | Desktop 2 |
| 5 | a file is dragged over the editor | Copy here |
| 6 | the layout is switched, and the first of two keys is pressed | nothing is written yet |
| 7 | the second key is pressed, and Müller is typed | ü |

**This table is not a copy.**
`crates/alo-dividing/tests/the_walk_through_a_working_morning.rs` reads *this
file*, parses the rows under this heading, and asserts the morning the assembled
crates produce is exactly them, in order. A sentence that changes without this
table changing fails the gate; a table edited without the machine changing fails
it too. When a later change moves a sentence, it publishes the table again in a
follow-up report and points the test at that one, because a published report is
never rewritten.

## What each row is evidence of

- **Rows 1 and 2** are task 1's words. A person is told *where the window went*,
  not what it was resized to.
- **Row 3 is the one worth arguing about.** Dragging the boundary leaves mail
  holding neither a half nor a quarter, and what a person reads is **Part of the
  screen** — not `1200×1080`, not `62%`. A person who drags a boundary has not
  asked for a measurement, and a machine that answered with one would be
  narrating arithmetic at somebody who is looking at the result.
- **Row 4** is what an unnamed desktop calls itself: its number. A person who has
  not named a desktop still needs to be told which one they are on.
- **Row 5** is the label beside the pointer **before** letting go — the whole
  point of task 4's `Over`: a person is told what letting go *would* do while
  they can still not do it.
- **Rows 6 and 7 are two moments on purpose.** On a layout where an umlaut is a
  sequence, the first key writes **nothing** and says so, and the second writes
  **ü**. A machine that showed nothing at all after the first key would look
  broken; one that wrote a stray character would be wrong. *Müller* is in the
  acceptance because a name most of Europe can spell is exactly the case a
  keyboard has to get right.

## What is deliberately not here

No sentence names `libinput`, `evdev`, XKB, a keysym, a scancode, a keycode,
`ibus`, `fcitx`, an input-method framework, Wayland or the compositor — held by
its own test over all five crates' vocabularies. A person moving a window has not
agreed to learn what a keysym is, and those libraries are rented and replaceable
(ADR 0011): a sentence naming one would be a sentence that must change when we
change what we rent. **The person's sentence must outlive the machinery.**

## Every sentence these crates can say

`crates/alo-dividing/tests/every_sentence_the_desktop_says.rs` holds all five at
once, because a person does not meet them one crate at a time:

| | |
|---|---|
| `alo-dividing` | 15 |
| `alo-desktops` | 21 |
| `alo-keyboards` | 24 |
| `alo-handing` | 11 |
| `alo-menus` | 15 |
| | **86 sentences** |

Each is read out of `alo-saying`'s **assembled** vocabulary rather than out of
its own crate's constants — a word a crate declares and the machine never
collected is a sentence nobody will ever read — and each carries a translator's
note long enough to translate by, which is its own check: a note saying only *the
name of a thing* tells a translator nothing.

## How big this plan was, and how big it finished

| | |
|---|---|
| Tasks when published, 2026-09-15 | **7** |
| Tasks at close, 2026-09-20 | **7** |
| Added after publication | **none** |

**Nothing was added in five days.** That is now the fourth shape recorded, and
the four together are more useful than any one of them:

| Plan | Published | Closed | Added | When they stopped |
|---|---|---|---|---|
| applications | 5 | 11 | 6 | over four days, under the work |
| access and language | 7 | 7 | 0 | — |
| the machine, measured | 5 | 14 | 9 | all within two days, none after |
| **hands on the desktop** | **7** | **7** | **0** | — |

Two of the four did not grow at all, and they are the two whose subject was
**named before the work began**: access-and-language by an external standard and
a fixed list of languages, and this one by six things a person does at a desk —
split, desktop, drag, menu, keyboard, walk — none of which could turn out to be
two things. The two that grew were bounded by our own judgement of what the work
would turn out to need.

So the third column of the ratio matters as much as the count, and this plan
sharpens why: **a plan that names its tasks from the outside does not grow; a
plan that names them from the work does.** That is knowable when the plan is
written, which makes it worth writing down then rather than discovering at close.

**One thing did change without being counted**, and it is the argument for
checking a plan's own prose and not only its task list: the *Crates this plan
owns* line said three crates from 2026-09-15 until today, while task 4 added two
more on 2026-09-17. The task count was right and the crate list was wrong for
three days — and it was found only because task 7 had to hold every vocabulary
and there were five, not three.

## What the walk does not re-test

The drop's mechanics are task 4's, and this walk builds the `Over` value rather
than driving a whole drag through a trait object. The walk holds **the sequence
of sentences**; re-testing each crate's machinery inside it would be a second
copy of four other test files, drifting from them.

## A note on the plan's own crate list

The plan's *Crates this plan owns* line still names three. Task 4 added two more
— `alo-handing` and `alo-menus` — and both put words in front of a person, which
is why this task holds five vocabularies and not three. The line is stale in the
same way task 2's blocker was, and in the same plan.
