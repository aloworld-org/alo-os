# What closes v0.0.5, and in what order

Decided by the owner on 2026-09-25, when the count stood at 227 tasks, 208
done, 9 open, 9 blocked and 1 declined. Nine of the eighteen left were waiting
on hardware or on the owner rather than on code, and a blocker nobody has
sequenced is a blocker every lane walks past. This note sequences them, and
moves two out of the release.

## The last two things

**The certified laptop and a second signed release are the last tasks of
v0.0.5.** Not because they are hardest, but because they close nothing else:
every other task can be finished around them, and each of them is the owner's
to perform.

- **The laptop** clears four at once — installer 6, devices 1, devices 4 and
  broker 9. Until it exists, those four stay blocked and no lane should be
  sent at them.
- **A second signed release** clears the machine keeps itself 8. The private
  key never sits on a machine an agent runs on, so this one is performed by the
  owner and cannot be delegated, scheduled or worked around.

Everything in the section below is what the lanes finish first, so that when
those two happen the release closes on them rather than on a queue behind them.

## Moved out of v0.0.5

**The models measured, task 23 — the card road — moves to 0.0.6.** Task 22 is
finished: the processor road is measured, and so is every refusal on the way to
a card that cannot be used. What 23 adds is one machine carrying a discrete
card, to hold [ADR 0007](../../decisions/0007-the-cpu-is-the-default.md)'s
*a GPU changes speed, not capability* as a measurement rather than a claim. The
instrument is already written and `#[ignore]`d, waiting for that machine.

v0.0.5 reaches developers on integrated graphics, and renting a graphics server
now would buy one sentence at the cost of a server and the NVIDIA image
decision, which stays open. **The release notes say plainly that the card road
is unmeasured** — an unmeasured claim named in the notes is honest; the same
claim made quietly is not.

## Documents 10 becomes the development PC's

Older `.doc`, `.xls` and `.ppt` — converted, and what each copy lost — moves
from the Mac to the development PC, which has Word, Excel and PowerPoint
installed. Two things it needs, and neither was obvious:

1. **Three files the repository's owner wrote**, saved from those applications
   in their 97-2003 formats. The legacy files already on that machine are a
   published standards list and company invoice data — neither can be published
   in the test fixtures, which is what the task requires. Nothing synthesised:
   a container this repository assembled would measure the assembler.
2. **The pinned conversion engine running on that machine.** It is not there
   yet, so that is a step of this task and not an assumption inside it.

## Documents 8 stays where only it can be done

The iWork exclusion needs a `.key` and a `.numbers` saved **by** Keynote and by
Numbers. [ADR 0057](../../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
is accepted as option A — wait for a real file — so this cannot move to a
machine without those applications.

**One route that does not need the Mac:** Keynote and Numbers on iCloud in a
browser produce real files from the real applications. If that route is taken,
the provenance line in `crates/alo-opening/tests/files/README.md` says it was
the web version, because which writer wrote a fixture is the whole value of
recording provenance.

## Shell 14 asks nothing of the owner

Every new surface, walked, is blocked on the shell's own tasks 11 and 16, both
open. It unblocks itself when they land, and needs no display and no hardware
the team does not have. Its constraint asks only that the report say the
certified machine has seen none of it.

A blocker described as hardware when it is really an unfinished task sends the
owner shopping and leaves the task where it was.
