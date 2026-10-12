# ADR 0104 — Any PC it works on is done, and no GPU is required

**Status:** **accepted, 2026-10-12**, by the owner.

- **Supersedes a standing rule in `CLAUDE.md`:** *Certified before compatible.
  One machine model that works completely beats a compatibility list nobody can
  honour.* The rule stands as a **statement about marketing** — we still do not
  claim "supports PCs" — and no longer as a definition of *done*.
- **Changes what finishes a piece of work.** A thing is done when it works on a
  PC this team can put their hands on. Not on a named model, not on a list.
- **Removes the graphics card from the critical path.** alo OS must draw its
  own desktop on a machine with no discrete GPU.

The owner's words: *no need for the certified pc and no need for gpus as long
as we do the test on any pc and it works then it will be marked done.*

## Why this is a correction rather than a lowering of the bar

The certification rule was written to stop a familiar failure: a compatibility
list that nobody maintains, where every machine works a little and none works
completely. That failure is real and the rule was a good answer to it.

**It had become a different thing in practice: a reason nothing could be
finished.** Measured on the day this was accepted —

- no application has ever been started on real hardware, because the only
  machine the gates run on is a virtual one with no session bus;
- the Dock's application discovery has never run anywhere that could answer
  anything but *failed*, for the same reason;
- `alo-desktop.service` carries a comment saying its ordering against the
  greeter is **owed a measurement on a machine**, and that comment is five days
  old.

Each of those is blocked on *a certified machine*, and a certified machine was
blocked on certification work nobody was doing. The rule had stopped selecting
for completeness and started selecting for **nothing being verified at all**,
which is the opposite of what it was written for.

So the bar moves from *this model works completely* to **this works, on a real
machine, and we say which one**. The second is harder to fake: a list of models
is a promise about machines we have not got, and a measurement names the one we
had.

## What *done* means now, exactly

A piece of work is done when:

1. it passes the gate, as before — nothing here weakens `CLAUDE.md`'s gate;
2. **it has been exercised on a physical PC**;
3. **the report names that PC**, by something a second person could identify it
   by, and says what was observed rather than that it worked;
4. what was *not* exercised is named, as everything else in this repository is.

**Three and four are conditions, not manners of speaking**, and this is the
half of the old rule that must not leave with the rest of it. The desktop lane
put it plainly when this decision was drafted: *the certification work going
away is right; the measurement discipline going with it would be a loss, and it
is the thing that has caught every real fault this week.*

*It works on a PC we have* without naming the PC and writing the reading down is
precisely the sentence `docs/misreadings/` exists to catch — a clean measurement
whose subject nobody can check. Five files in that directory are versions of it.
A certification list at least named a machine; a verdict that names none is
weaker than the rule it replaces, and this decision would then have lowered the
bar rather than moved it.

**"Any PC" is not "every PC".** A thing that works on the machine we tried is
done; a thing that works on *PCs* is a claim we still do not make. A second
machine that fails is a bug to fix, not a certification to withdraw — and
`docs/quirks.md` is where the difference between the two is written down.

## No GPU

**The desktop must draw acceptably in software.** This is a real constraint and
not a relaxation: it is easier to meet a frame budget with a GPU than without,
and ruling the GPU out means the compositor's own rendering has to be good
enough on its own.

What this settles:

- a machine with integrated graphics and no discrete card is a target, not a
  degraded case;
- a renderer path that only works with hardware acceleration is not finished;
- **the measurement is on a real machine, not in a virtual one.** A nested
  compositor under a headless parent says what the shell computed, never what a
  screen showed.

What it does not settle: it says nothing about whether a *model* runs on the
CPU. Local inference without a GPU is a separate question with separate physics,
and this decision is about the desktop drawing itself.

## What has to be rewritten because of this

- **`CLAUDE.md`'s standing rule**, to the narrower claim it is now making.
- **`docs/hardware.md`**, which is the certified list. It becomes a record of
  the machines alo OS has been run on and what happened, which is what it was
  always more useful as.
- **`ROADMAP.md`'s exit gates**, which were written against the certified-
  machine rule. Several of them say *on the certified machine* and now mean
  *on a machine, named*.

Each of those is a change in the same release as this decision, or this
decision is a sentence nobody acted on.

## What this does not touch

**Every one of the five laws.** Nothing here makes an egress quieter, a grant
broader, a record optional or a refusal untested. The gate in `CLAUDE.md` is
unchanged: what moves is which machine the integration test runs on, not
whether there is one.

And it does not make *not yet measured* into *done*. The three gaps named above
are still gaps. What changes is that they are now **answerable this week on a
machine in the room**, rather than waiting on a model nobody had chosen.
