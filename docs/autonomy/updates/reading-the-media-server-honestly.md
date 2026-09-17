# Reading the media server honestly, and tests that clear up after themselves

**Date:** 2026-09-17
**Workstream:** `alo-in-use` (the capture plan's), `alo-sound` and `alo-cameras`
(the devices plan's) — three crates that read one rented server
**Task:** none: two defects found while publishing
[the media kinds](a-film-is-a-film-and-not-a-file-nothing-recognises.md), and the
`/tmp` litter my own tests were adding to
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
measured in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6 CPUs, 3 GB of
memory**, PipeWire 1.0.5.
**Egress:** none.
**Status:** done.

## Three things, all found by a gate refusing to publish

### 1. A machine with the tools and no server was called broken

`pw-dump` installed, no session running, and the tool says `can't connect: Host
is down`. This crate read that as **what handles sound and video did not answer**
— a server that is there and broken — when what is true is that there is no
server. That is an ordinary build host with the package on it, and every machine
whose session has not started yet.

It is `NothingHandlesSoundAndVideo` now, which is both what an indicator should
say and what makes the on-a-machine test **skip itself rather than fail**. It
reads the tool's own wording to tell the two apart, which is a thin thread and is
written down as one: where that wording changes it falls back to *did not
answer*, which is still a refusal and still not an empty indicator.

The test that broke was the one that used `cannot connect to the daemon` as its
example of *a tool that ran and failed* — the same shape as the MP4 fixtures in
`alo-opening`: the case whose meaning was wrong was sitting in the test as the
example of the old meaning.

### 2. A record arrived as more than one list, once

`alo-in-use` refused a record with *trailing characters at line 42599* — a
complete JSON list and then something else. The same machine answered a single
list that parsed on the next reading and on six more while its graph changed
underneath it. **What produced it was not caught**, so this claims nothing about
why; the line number was fifty past an ordinary record's length, which is
consistent with a second list of whatever changed while the first was written,
and that is as far as the evidence goes.

All three crates that read that record now read it as a **stream** of lists,
taking an object listed twice as it was listed last. A single list — every other
reading there has ever been — goes through unchanged. The reason for tolerating
it rather than insisting: *this machine answered something unreadable* takes the
in-use indicator off a screen while a camera may be on, and that is the one
answer it must never give for a reason nobody can act on.

Both are in `docs/quirks.md`, written as what was seen rather than as a diagnosis.

### 3. My own tests were part of a 43,290-folder pile

`alo-measuring`'s tree test died with **`Too many open files`** after eight and a
half minutes during the same gate run. The VM's `/tmp` held **43,290** folders
left behind by tests, against a file-descriptor ceiling of 1024. Another lane
wrote this up on 2026-09-16 at 19,632 folders, and asked for the fix that belongs
to the crates: *tests that make a folder should remove it, pass or fail.*

Mine were adding to it. Every folder-making test in `alo-sound`, `alo-cameras`,
`alo-power` and `alo-portals` now holds its folder in a value that removes it when
the test ends, whichever way it ends. On the machine itself the pile is cleared
and the gate scripts raise the ceiling to 8192, with a comment saying why: a
suite that dies of open files reports it as a bug in whichever crate was
unluckiest.

## And one gate of my own that was wrong

The publish before this one was refused twice, and the second refusal was mine
rather than the tree's: I had added the media server's runtime directory to
`gates.sh` and `gates-touched.sh` but **not to `nofail.sh`** — the run that
decides which failures are the known ones. So the gates saw a machine with a
media server and this crate's test passing, while the run that judges the result
saw a machine without one and the test failing, and the publish was refused for a
difference in environment rather than a difference in the tree. All three scripts
set the same two things now.

It is worth saying plainly because it is the failure mode the whole
known-failures mechanism has: **a judge that does not see what the gate saw will
refuse honest work and accept nothing in its place.**

## What is still true and not fixed

- **Three crates reach this server through their own copy of the same twenty
  lines**, and now their own copy of the same stream-reading rule as well. That
  is four copies of something none of them owns. A small crate that did own
  *reaching the media server* would be a better home, and it is still a proposal
  for whoever holds the media stack rather than a change to make from inside one
  plan — but this is the second time in two days that a fix had to be made three
  times.
- **The trailing-characters reading is understood as a symptom, not a cause.** If
  it appears again with a record that can be kept, it is worth keeping.
