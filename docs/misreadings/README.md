# One misreading, one file

**Where we fooled ourselves**, as `../quirks/` is where reality and a
specification disagree. A quirk is a fact about somebody else's software. A
misreading is a fact about us: a measurement that answered a different question
than its label claimed, and was believed because the answer looked right.

**These are not confessions and they are not a backlog.** Every entry here cost
real time once and is cheap to avoid twice, which is the only reason to write it
down. An entry that cannot say what to do differently is a slogan and does not
belong.

**One file per entry**, for `../quirks/README.md`'s reason: three lanes
appending to one file collided 28 times in one week, and adding a file cannot
conflict with adding a different file.

## The shape of an entry

Four things, in this order:

1. **What was concluded** — the sentence that was wrong, as it was written.
2. **What was true** — measured, with the command or the line that shows it.
3. **The mechanism** — why the wrong answer looked right. This is the part worth
   reading; the rest is an example of it.
4. **The cure** — what to do instead, narrow enough to follow.

## Why this is not in `quirks/`

A quirk is something a machine does. These are things a reader does, and the
reader is us. Mixing them would make both harder to search: somebody hunting a
firmware behaviour does not want a lesson about grep, and somebody about to
count enum arms does not want a page about EDK II.

## Entries so far

All from 2026-10-04, one lane, one day — which is itself the finding that
started this directory. They were caught by other lanes measuring the same thing
more carefully, not by any test here.
