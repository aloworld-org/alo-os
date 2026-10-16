# ADR 0117 — The thing that starts a program is not the thing that bounds a turn

**Status:** accepted
**Date:** 2026-10-16
**Context:** [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
(the person chooses how code runs),
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md),
`crates/alo-bounding`, `docs/features.md` (*Running code, at the level the
person picks*), `CLAUDE.md` law 2

## The decision in one line

The sealed box of ADR 0064 is built in **its own crate**, which starts a
program and uses `alo-bounding`'s control group and boundary to confine it —
so that **`alo-bounding` itself never gains a `fork`, an `exec` or a
`Command`**, and the guarantee its own test makes about that stays true.

## What forced the question

`docs/features.md` promises running code at three levels and names what the
default is waiting on: *the sealed box needs the kernel boundary*. That
boundary is built. Ten BPF LSM hooks are attached, `places_of` turns paths into
a bound, `Departures` carries approved network destinations, and
`Turns::doing` is a complete turn lifecycle that `alo-agentd` already drives.

So the obvious move is to add a *run a program in a turn* method beside
`Turns::doing`. **That is the one thing that crate must not have.**

`alo-bounding/src/lib.rs` says why, and it is a deliberate position rather than
an oversight:

> *Runs the verb's work inside that cgroup* is ADR 0015's line, and taken
> literally it says **start something**. Law 2 is why it does not: every shape
> that spawns needs a program to spawn, and a program alo OS starts on an
> agent's behalf is one review away from a program an agent named.

And it is **tested, not promised**. `tests/a_turn_is_this_thread.rs` reads
every `.rs` file in that crate's `src` and fails on `Command`, `fork(`,
`exec(`, `execv`, `execl` or `posix_spawn` outside a comment, with a positive
control beside it so the test cannot pass by matching nothing. A method that
spawned would fail that test, and **deleting the test to add the method is the
gate weakened to pass it.**

## Why two crates is the right answer and not a way round a test

The two things being asked for are genuinely different, and law 2 draws the
line between them rather than across either:

| | what runs inside | who chose it |
|---|---|---|
| `alo-bounding` | one **thread** of `alo-agentd`, running one of six enumerated verbs | nobody chooses: the verb list is closed and the compiler walks it |
| the runner | a **program**, named in the moment | **the person**, at one of ADR 0064's three levels |

Law 2's first half — *no `exec`, no shell and no "advanced" escape hatch is
ever added to that list* — is about the **broker's verb list**, and the runner
adds nothing to it. Running code is a separate capability, and ADR 0064 is
explicit that it is **not a verb**. Law 2's own second half says so: *running
code is a separate grant that only the person gives.*

So the crate that serves the closed verb list keeps its guarantee that it
starts nothing, and the crate that exists because a person chose to let the
agent run code is the one with the `exec` in it. One crate, one reason to
change — `CLAUDE.md` law 4 at the level of a crate rather than a file.

**The reverse arrangement is what we are avoiding.** One crate holding both
would make *the boundary* and *the spawn* a single reviewable unit, and the
file nobody wants to open would be the one that both decides what a program may
reach and starts it. That is precisely the file law 4 was written about.

## How a child gets inside its box, and why there is no unbounded window

The ordering problem is the whole of the mechanism: between a child existing
and the kernel knowing which box it is in, a program would run unconfined.

```text
the run begins
  ├─ a control group is made for this run       (Cgroup::made_under)
  ├─ { cgroup id → the places and destinations } goes into the map
  │                                              (Boundary::bound)
  └─ one small program of ours is started in it, told the control group and
     the command
       ├─ it writes "0" into that group's cgroup.procs, which cgroup v2 reads
       │  as *the process writing this* — so it puts itself inside the box
       └─ and only then does it exec the command

the run ends
  ├─ the entry is removed    (Boundary::released)
  └─ the control group goes  (Cgroup::removed)
```

**The window contains one program of ours and no program of anybody's
choosing.** The command a person was asked about starts at `exec`, which is
after the admission, so there is no instant at which it runs outside its
boundary. The boundary entry is written before the control group holds
anything at all, and `Boundary::bound` already guarantees the rest: *there is
no window between the entry existing and the boundary applying, because the
entry is the boundary.*

**`"0"` rather than the program's own number**, because it needs no buffer to
format and no second syscall to learn: one byte into a descriptor is the whole
admission. Measured on 2026-10-16 on cgroup2fs: a process that writes `"0"`
to a group's `cgroup.procs` is afterwards in that group, read back from
`/proc/self/cgroup`.

**The small program checks the control group it was handed.** It refuses a
path that is not under this service's own turn root, so the one argument that
decides where it puts itself cannot name somewhere else. This costs a
`starts_with` and removes the only interesting thing about its arguments.

**A domain control group, not a threaded one.** `Turns::beginning` makes a
threaded group because a turn is a thread; a run is a whole process, so the
runner makes an ordinary one with `Cgroup::made_under` and does not ask it to
hold threads. Measured the same day: writing a process into a *threaded* group
does work, but reading `cgroup.procs` back then always reports empty, which is
a verification that says *it failed* about something that worked.
`docs/quirks.md` carries it.

## Why a second program rather than a hook between fork and exec

The shorter mechanism is `pre_exec`: one process, and the child admits itself
between `fork` and `exec` with no helper on disk. **It is not available here.**
`CLAUDE.md` forbids `unsafe` and the workspace sets `unsafe_code = "forbid"`,
which — unlike `deny` — no attribute can lift. `pre_exec` is unsafe for a real
reason, that a forked child may not allocate or take a lock, and the two
crates in this repository that do have `unsafe` are a BPF programme in its own
excluded workspace and nothing else. `alo-agentd`'s `unix.rs` carries the note
that it went the other way for exactly this rule.

`CommandExt::exec` **is safe**, because it replaces the running program rather
than running inside a forked copy of one. So the admission moves out of the
fork and into an ordinary program, where it is a file write with error handling
and a test, and the crate needs no `unsafe` at all.

The helper is also the more reviewable of the two. What a reader has to check
is one short program that admits itself and execs, rather than a closure whose
correctness depends on which functions are safe to call in a forked child.

## Consequences

- **A new crate**, and it depends on `alo-bounding` rather than changing it.
- **`alo-bounding`'s source guard stays exactly as it is**, and this ADR is the
  reason a later reader should not relax it. If a method there ever needs to
  start a program, that is a new decision and this document is what it
  overturns.
- **The new crate carries the same guard, inverted.** It is the one place in
  the repository that may `exec`, so it says so, and a test over its own source
  holds it to where.

  **There are two such places in it, not one**, which the test found by failing
  on the crate it was written for. One starts `alo-enter-the-box` and may start
  nothing else — it is checked against the literal call, so starting the command
  there instead would be a different string and would fail. The other becomes
  the command, and must name the admission in the same file, because an `exec`
  that is reachable without the control group write is the one failure that
  would make every type in this crate a decoration. *Written down because the
  first version of the rule was simpler than the crate and said so out loud.*
- **`unsafe` in one file.** `pre_exec` is unsafe because of what may not happen
  between `fork` and `exec`, and there is no safe spelling of it. The crate
  denies `unsafe_code` at its root and lifts it in that one file, which is the
  shape `alo-bounding-kernel`'s `kernel.rs` and `alo-agentd`'s `unix.rs`
  already have.
- **What this does not do.** It runs a command inside a box. The record of the
  run, the snapshot taken before it, and the surface a person approves it on
  are three separate promises in `docs/features.md`, and this crate returns a
  full account of what happened so a caller can keep the first two. Saying so
  here rather than implying the box completes ADR 0064.

## Alternatives rejected

**Add the method to `alo-bounding` and delete the guard.** Rejected: the guard
is a public guarantee about the crate that enforces every file grant on the
machine, and the method does not need it gone.

**Add the method and narrow the guard to skip one file.** Rejected for the
reason that crate's own test already records about a near-identical
temptation: *skipping the file it appeared in would have been the gate weakened
to pass it.*

**`clone3` with `CLONE_INTO_CGROUP`**, which places a child in a control group
at creation with no window at all. Not rejected on merit — it is the better
mechanism and systemd uses it — but neither `std` nor `rustix` exposes it, so
taking it means a raw syscall and a hand-written `fork`/`exec` in place of
`Command`, which is `unsafe` the workspace forbids outright. It would also
replace a window that contains one `write` of one byte and no program of
anybody's choosing. **Two things would have to change together for this to
return**: the window turning out to contain something that matters, and a safe
spelling existing to reach it — most likely `rustix` growing one, which is the
form the next reader should look for rather than reaching for a raw syscall.

**`pre_exec`**, which is the same idea without a second program on disk, and is
rejected for the rule rather than the design. The section above says why.
