# Every sentence, and the walk with the screen off

**Date:** 2026-09-19
**Workstream:** v0.5 — access and language
**Task:** 7, *Every sentence, and the walk with the screen off* — the last task in
that plan, which this closes.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**. Nothing here is ticked *on the machine*: a Mac is not the
machine, and this task touches no hardware.
**Egress:** none. Nothing was pulled from anywhere for this task.

## What the task was for

Tasks 1 to 6 each end in something a person hears or reads: a setting, a control's
name, a language, an approval, a line of the record. Each crate's own tests hold
its own. **What none of them can show is the sequence** — whether somebody with
the screen off is carried from one to the next, or handed six crates' worth of
individually correct sentences that do not join up.

So this task is one walk, end to end, and one test that holds it.

## The walk, sentence by sentence

Each row is one moment and the one thing a person meets at it. **Spoken and shown
are one column, deliberately**: every name in `alo-access`'s tree is what a reader
is told *and* what the shell draws — one description, used by both. That is the
argument that file makes, and it is why a blind person and an agent read the same
machine. Two columns would be two descriptions, and the day they differed nobody
would know which one was the machine.

| Step | The moment | What a person meets |
|---|---|---|
| 1 | The screen reader is turned on, before any account is chosen | read the screen aloud |
| 2 | Tab, and the keyboard is here | who is signing in |
| 3 | Tab, and the keyboard is here | password |
| 4 | Tab, and the keyboard is here | sign in |
| 5 | Tab, and the keyboard is here | settings for seeing, hearing and typing |
| 6 | Tab across the desktop, and the agent is one of the stops | ask the agent |
| 7 | The question is asked in Greek — Πόσο χώρο έχω; | Ελληνικά (el) |
| 8 | The approval is read, and nothing is chosen for them | the machine is asking you something |
| 9 | The approval is read, and nothing is chosen for them | what will happen if you approve |
| 10 | The approval is read, and nothing is chosen for them | no |
| 11 | The approval is read, and nothing is chosen for them | approve |
| 12 | Escape, on the approval | it answers no |
| 13 | The record is read | the record |
| 14 | The record is read | what has happened |
| 15 | The record is read | one thing that happened |

**This table is not a copy.** `crates/alo-access/tests/the_walk_with_the_screen_off.rs`
reads *this file*, parses the rows under this heading, and asserts that the walk
the assembled crates produce is exactly them, in order. A sentence that changes
without this table changing fails the gate; a table edited without the machine
changing fails it too. When a later change moves a sentence, it publishes the
table again in a follow-up report and points the test at that one, because a
published report is never rewritten.

## What each row is evidence of, and what it is not

- **Row 1 comes first, and that is the whole of it.** The reader is turned on
  *before an account is chosen*. Any other order asks somebody who cannot see the
  screen to sign into it first in order to turn on the thing that would let them.
- **Rows 2–5 are the keyboard road**, taken from `focus_order(Surface::SignIn)` —
  the reading order filtered to what can be used. There is no second list to
  maintain: focus order **is** reading order, which is why row 5 exists at all
  (the settings the sign-in screen offers are a Tab stop, not a mouse target).
- **Row 6 is the finding from task 3 made visible.** The agent is reached by
  Tab. A chord is a convenience for somebody who knows it; a person meeting this
  machine for the first time with the screen off knows no chords.
- **Row 7 is a decision, not a sentence.** What a person meets is an answer in
  Greek; what this machine decides and can be held to is *which language that is*.
  The row records `alo_instructing::the_language_of`'s answer and its tag, because
  that is the part with a right and a wrong.
- **Rows 8–11 read the approval with nothing chosen for them**, and **no** is read
  before **approve**. A reader that reaches the safe answer first is not politeness;
  it is what stops an approval from being carried by a person's habit of pressing
  Enter at the end of a sentence.
- **Row 12 is the refusal path.** Escape on an approval **answers no**. It does not
  dismiss, it does not leave the question open, and it never approves. The test
  panics on any other answer rather than recording it.
- **What this is not:** it is not a conformance claim and not an on-a-machine
  tick. Nothing in this walk speaks aloud — no speech engine runs, no AT-SPI tree
  is exported. It holds the text a reader would be handed and the order it would
  be handed in. The speaking is the shell plan's, on hardware, later.

## The other two tests

- **Every sentence these crates can say is in the machine's vocabulary, and each
  carries a note for whoever translates it.** Read out of
  `alo_saying::everything_this_machine_can_say()` — the assembled machine, not this
  crate's own constants — because what a person hears is what the assembled machine
  says. Thirty-five words, every one of them found, every one of them noted.
- **No sentence names what is underneath.** Not Orca, AT-SPI, eSpeak,
  speech-dispatcher, CLDR or Unicode. A person turning on the screen reader is
  turning on the screen reader; which one this machine rents, and how the tree
  reaches it, is ours to change without changing a word anybody reads.

## How big this plan was, and how big it finished

Per the rule adopted after the applications plan: the count of remaining tasks
under-reads how much is left when tasks get added after a plan looks finished, so
every plan-closing report states the ratio.

| | |
|---|---|
| Tasks when published, 2026-09-15 (`e62d072`) | **7** |
| Tasks at close, 2026-09-19 | **7** |
| Added after publication | **none** |

**Nothing was added to this plan after it was written** — the same evidence as the
applications plan's five-to-eleven, with the opposite sign. Both numbers come from
counting `^### ` headings in the plan file at its first commit and at close. Worth
saying why they differ: the applications plan grew because the work kept finding
adjacent work in crates it already owned, while this plan's subject was fixed by
an external document (EN 301 549) and by a fixed list of 24 languages. **A plan
bounded by somebody else's list does not grow; a plan bounded by our own judgment
does.** That is the more useful reading of the ratio than either number alone.

## The plan is closed

All seven tasks are done. What the plan deliberately left undone, and who has it:

- **Speaking, magnifying and drawing focus** are the shell plan's, on hardware.
  This plan decides and holds; it does not draw. Nothing in `crates/alo-shell` was
  touched.
- **A published conformance report is v1.** `crates/alo-conforming` now stands at
  **17 clauses met, 24 waiting, 5 not applicable, 0 read against the standard's own
  text** — that last count is the honest one: no clause here has been checked
  against EN 301 549's purchased text, only against its publicly quoted titles, and
  the report that goes out must be.
- **Key repeat (clause 5.7) is still *not yet*.** Slow keys are not key repeat, and
  task 3 deliberately did not claim it.

## Gate

Nine gates, in the Lima VM, on the branch `task/mac/the-walk-with-the-screen-off`.
Evidence is posted with the pull request. The branch also carries the two commits
ahead of it that are not yet on main — `2c87c26` (the fixture-folder fix) and
`9b94549` (task 3) — because task 7 cannot build without task 3.
