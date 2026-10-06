# I killed the compiler and left the linker

**Found 2026-10-06, Mac lane, while gating
`more-than-one-display-plan.md` task 4.**

## What I concluded

That the build machine was idle. I had stopped three runs over the afternoon,
each time with some version of:

```
pkill -9 cargo ; pkill -9 rustc
```

and each time I checked my work:

```
pgrep -c -f "bin/(cargo|rustc)"   # → 0
```

Zero. Idle. I launched the next run against what I believed was an empty
machine, three times, and each ran slower than the last.

## What was true

Two orphaned **linkers** were still going, one of them for **two hours and
one minute**, each holding about 850 MB in a virtual machine with 3.9 GB. The
guest had **143 MB available** and was swapping six thousand pages a second.
The run I was waiting on was not slow because linking is slow; it was slow
because it was competing with the corpses of its two predecessors.

Killing them returned **1669 MB** — from 143 — and the measurement is one
command:

```
ps -eo etime,rss,comm --sort=-rss | head
```

## The mechanism

**Two faults that hide each other.**

The first: `rustc` does not link. It shells out to `cc`, which shells out to
`ld`. Kill `rustc` and the grandchild is orphaned, reparented to init, and
keeps going — it has all its inputs and no reason to stop. The process I
named is not the process doing the work.

The second is what made it invisible for three runs: **my own idle check
could not see the thing I had failed to kill.** `bin/(cargo|rustc)` is a
pattern that matches exactly the two names I had already thought of. It can
only ever confirm that I killed what I meant to kill. It answers *zero* with
perfect confidence while the machine is saturated.

That is `CLAUDE.md`'s *a check that stands in for the thing is not the thing*,
in its worst form — the verification and the error were **derived from the
same wrong list**. A check built from my assumption cannot test my
assumption. And nothing was red: no failure, no warning, just a build that
took longer each time, which reads as a slow machine rather than a mistake.

## The cure

**Kill the process group, not the names you can remember.** The run is
launched with `setsid`, so it has a group of its own and
`kill -9 -<pgid>` takes the linkers with it.

**And verify by what the machine is doing, not by the names you searched
for.** The honest check is free memory and the heaviest processes —
`free -m` and `ps --sort=-rss` — because those are written by the kernel
about everything, not by me about what I expected. A long `etime` on a build
tool is the tell: a linker older than the run you just started is not yours.

The general rule, which is the part worth keeping: **when a check is built
from the same list as the action it verifies, it verifies nothing.** Confirm
from the other side — the effect, not the intent.

## Related

- [`i-watched-the-client-and-called-it-the-job.md`](i-watched-the-client-and-called-it-the-job.md)
  — the same afternoon, and the same shape: a `pgrep` pattern that could not
  observe the thing its sentence claimed.
- [`a-long-build-and-an-idle-tree-look-the-same.md`](a-long-build-and-an-idle-tree-look-the-same.md)
  — the first of the three, and the reason the runs were being killed at all.
