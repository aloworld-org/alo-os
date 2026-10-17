# The terminal, finished and usable — what that takes

**Written:** 2026-10-17, at the owner's request before a period away.
**The goal, in the owner's words:** *all the tasks of the terminal are done, and
all the features we talked about work end-to-end and users can use it.*

**So "done" here does not mean the change landed.** It means a person sitting at
an alo OS machine can do the thing. That is a higher bar than a green gate, and
it is the bar this document keeps: every row below says **what a person does**,
and what is still between them and doing it.

## What that bar rules out, and it is the thing to read first

A terminal feature is usable when four things are true at once:

1. **the mechanism works** — tested, measured, with its refusal path;
2. **something calls it** — the fault this repository has found in five shapes,
   most recently six terminal mechanisms gated green and visible to nobody;
3. **a person can reach it** — a chord, a control, something they could find;
4. **somebody has seen it on a machine.**

The fourth is the one a green suite says nothing about. **A machine does boot** —
one booted from `06c1444f` on 2026-10-13 and installed all seven applications,
read by the desktop lane on the running machine — so this is reachable rather
than hypothetical. What has **not** happened is anybody opening alo's own
terminal on a booted machine and using it.

*`docs/autonomy/evidence-a-person-can-work-on-it-all-day.md` says "Nothing has
booted the image" in three places and has been wrong since the evening of
2026-10-13. It is not this lane's document to rewrite, and it is named here
because a reader starting from it would conclude this goal is unreachable.*

## The thirteen, against that bar

★ marks a differentiator. **Usable** means all four conditions above.

| | what a person does | state |
|---|---|---|
| | opens a terminal and types | **usable** — a chord opens it, the compositor draws it, typing reaches the shell |
| ★ | decides whether alo may use it | built; the choice has no surface, so **not reachable** |
| ★ | reads who is speaking | **usable** — `you`, `alo`, `system`, `program` in the gutter |
| ★ | sees measured apart from worked-out | **usable** — the same gutter's `system` against `alo` |
| ★ | asks why a command is waiting | **built and reachable** as of #755, `Ctrl`+`Alt`+`W`. **Never seen on a machine** |
| ★ | scrolls back through output | **usable** |
| ★ | runs a command touching nothing | the runner and record exist; the kernel box does not — **not usable** |
| ★ | scopes one command | **nothing** |
| ★ | finds a command by what it did | **nothing** |
| ★ | bisects the machine | **nothing**, and waits on a bracket |
| ★ | reads output that is not a character grid | **nothing** |
| ★ | hands somebody a session to replay | **nothing**, and waits on rich results |
| | asks what a command will do first | **nothing** |

**Four of thirteen are usable today. One more is reachable and unseen.** Of the
remaining eight, **seven are this lane's to build** and one — *run a command
touching nothing* — is **nobody's as of 2026-10-17**. The owner said that day
that the development PC is off and its work can be done by the desktop lane; the
desktop lane has read that and **has not taken it**, for a stated reason. So it
is available and unclaimed, which is not the same as assigned. The table near the
foot of this file carries all three readings and what changed between them.

## The order, and it is ordered by that bar rather than by size

### 1. See the five answers on a booted machine — **the boot is 2026-10-18, evening**

**The smallest step to the largest change in confidence.** #755 made *why is it
waiting* reachable; nothing has watched a person use it. One boot, one terminal,
one `cat`, one chord — and either it says *waiting for the keyboard* on a screen
or it does not, and the second is a finding worth more than the next three tasks.

**The owner said on 2026-10-17 that the boot is tomorrow evening.** So this is
scheduled rather than a thing to ask for, and the only question left is whether
the walk is written down before the machine is on — because an unprepared boot is
a boot spent finding out what to look at.

**The walk, in order, so none of it is worked out at the keyboard.** Each step
says what counts as a pass, and each is a thing a person does rather than a
command a lane runs:

| | do this | a pass is |
|---|---|---|
| 1 | sign in | a desktop, and the sign-in screen gone |
| 2 | press `Ctrl`+`Alt`+`T` | a terminal window, with a prompt |
| 3 | read the gutter beside the output | the word `program`, lowercase, in its own column |
| 4 | read the band under the transcript | the shell's name and `UTF-8` — e.g. `bash · UTF-8` |
| 5 | type `cat` and press Enter | nothing happens, which is the point: it is waiting |
| 6 | press `Ctrl`+`Alt`+`W` | **a line saying *waiting for the keyboard*, with `alo` beside it in the gutter** |
| 7 | press `Ctrl`+`D` to end `cat`, then `Ctrl`+`Alt`+`W` again | the line **goes**, because there is nothing waiting now |
| 8 | type `sleep 30`, Enter, then `Ctrl`+`Alt`+`W` | *nobody can tell* — a clock is none of the five, and this is the honest answer rather than a wrong confident one |
| 9 | scroll up with the wheel, then back down | the transcript moves and stops at both ends |

**Step 6 is the measurement.** Steps 3, 4 and 9 are things believed to work that
nobody has watched either, so they cost nothing to read while the machine is on.
Step 8 is the one most likely to be wrong, because it is the answer a careless
table gets wrong — a first version of that table answered *waiting for another
program* about `sleep`.

**Any step failing is worth more than the rest of this document.** What to send
back: which step, what was on the screen, and `journalctl -b 0 -u alo-compositor`
— because a drawing fault and a logic fault look identical from a chair, and the
journal separates them.

**And the chord may not be bound yet**: #755 is gating at the time of writing. If
it has not landed, step 6 does nothing and that is correct rather than a fault —
check `main` first and skip to 7.

### 2. Terminal tasks 4 and 5

`docs/autonomy/the-terminal-in-full-plan.md` is the truth about these. Both are
startable today, need no machine this lane lacks, and each ends in something a
person does:

- **find a command by what it did** — *the command that changed my git config*,
  answered from the record every run already writes;
- **scope one command** — one command under a narrower grant than the session
  has, without touching what the person chose for everything else.

### 3. Task 6, rich results

The largest, and **tasks 2 and 7 both wait on it** — the source qualifier is
provenance for a parsed result and has nothing to qualify until one exists, and a
replayable session needs a transcript that is more than cells.

**Untrusted is the load-bearing word.** This parses what a stranger's program
printed, so it is as much about what it refuses to render as what it renders.

### 4. The choice of whether alo may use the terminal needs a surface

Row two above: the three levels of ADR 0064 are built and a person cannot see or
change them. It is in `alo-shell`, which this lane holds, and it is the shortest
distance from *built* to *usable* of anything on the list after #755.

### 5. Explain a command before it runs

Back in the order, because the first version of this document had it out of
reach for a reason that did not hold: `docs/features.md` asks for *plain words
for what it does, with anything destructive, anything needing elevated rights and
anything leaving the machine marked*, and names no model. A provider key is on
this machine, so the asking is testable here.

**Called an explanation and not a guarantee**, in the promise's own words,
because a script, an alias or something it downloads can change what happens —
so the surface says that rather than implying certainty.

### 6. The bracket, if it is still nobody's

ADR 0045 puts it in `alo-turn`. #751 removed the first thing in the way and #752
recorded the measurement it rests on. **Only if nothing else has taken it** —
that crate is the workspace's most central and a collision in it costs a day
where asking costs a message.

### 7. Then bisect the machine, and *your files as they were*

Both wait on a bracket and on nothing else, so **taking step 6 makes these two
this lane's** — which is why the corrected table below no longer lists them as
unfinishable. If the bracket turns out to be another lane's, these go with it.

## What this lane cannot finish — corrected 2026-10-17, because the first version was wrong about all three

**The owner asked whether these three were the desktop lane's. They are not, and
checking rather than agreeing changed every row.** The first version of this
table read as *out of reach*; two of the three are reachable and the third
belongs to a different lane than implied.

| | the honest position |
|---|---|
| *run a command touching nothing* | **available to the desktop lane and taken by nobody**, as of 2026-10-17. The owner that day: *development pc is off now so it's tasks can be done by desktop*. Until then it was the development PC's by `kernel-enforcement-plan.md`'s own words: *who can take it — the development PC, inside a KVM guest with a real login*. The desktop lane was told, and answered that it is **not taking the eight plans on a relayed message** — the owner had given it a direct instruction about the application market, and a forwarded sentence does not outrank an instruction the owner typed to that lane. **It was never a capability wall**: this lane's machine has `CONFIG_BPF_LSM=y` and `bpf` in the active LSM list, measured 2026-10-17. So it was ownership all along, the owner has made it available, and **it is still nobody's** |
| *bisect the machine*, *your files as they were* | **downstream of this lane's own step 5**, not out of reach. Step 5 above is *the bracket, if it is still nobody's*. If this lane takes the bracket, both of these become its to build — so listing them as unfinishable **contradicted the order four headings above it**, and that was an inconsistency in this document rather than a fact about the work |
| *ask what a command will do first* | **buildable and testable here.** The promise is *plain words for what it does, with anything destructive, anything needing elevated rights and anything leaving the machine marked* — it does not say which model, and a **provider key is present on this machine** (mode 600, measured 2026-10-17). The first version said *a model on this machine is an environment this lane cannot test in*, which was true of a **local** model and false of the promise |

**Why all three were wrong in the same direction.** Each was written from what
this lane had recently found hard rather than from the document that governs it:
the kernel plan names its lane, the order four headings up already claimed the
bracket, and `docs/features.md` never asked for a local model. **A blocker
written from memory is a blocker nobody checked**, and this table is the fourth
thing today that a dated reading corrected — after `alo-turn`'s *nothing reaches
it*, the evidence document's *nothing has booted*, and the plan's inverted
dependency between the source qualifier and rich results.

**So one of the thirteen is not this lane's**, and it is now unowned rather than
held: the owner made it available on 2026-10-17 and no lane has picked it up.
The other two are this lane's, in the order above.

*The row above is the fourth reading of this one promise in a day — out of
reach, then the development PC's, then the desktop lane's, then nobody's. Only
the owner's message was new information; the rest came from this lane, and the
first was wrong. A row that moves this often is a row to cite rather than
remember.*

**And the third reading was this lane's own fault, of the exact kind it had just
written a paragraph against.** The commit that said *the desktop lane's* also
added a note to the lane charter explaining why the charter's eight rows must
**not** be rewritten to say *the desktop lane*: a row asserting an owner for
work nobody has started is worse than an empty cell, because the next lane reads
it and concludes the work is spoken for. **Then this row did that very thing**,
one table away, in the same change. *A machine being available and a machine
having taken something are different facts* — and knowing that in one paragraph
did not stop it being got wrong in the next. It was caught because the desktop
lane said plainly that it had not started, not by any check here.

## Standing, and the owner's

None of these blocks the order above.

| | what is needed |
|---|---|
| Pair five answers to four actions | `docs/features.md` records it as the owner's in as many words. #755 did not need it; offering the four actions does |
| Ship `alo-letting-go`'s four units | the owner's. This lane's advice is **ship**: measured on real `btrfs`, the units load and start, and #749 means a unit shipping with nothing to start it now fails the build |
| 32 tests only a person can start | the owner's; a gap between what the gate promises and what it enforces |

## How to check this document rather than believe it

Every row points at a plan, a pull request or a document that is the truth about
it. **This file is a reading taken on 2026-10-17 and will go stale** — exactly as
`alo-turn`'s *nothing reaches it* did three days after it was written, and as the
evidence document's *nothing has booted* did within one. A reader who finds this
disagreeing with a plan should believe the plan.

An index nobody can date is an index nobody can check, so this one is dated.
