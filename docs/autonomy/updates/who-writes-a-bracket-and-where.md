# Who writes a bracket, and where

**Date:** 2026-10-16
**Lane:** Mac
**Branch:** `task/mac/who-writes-a-bracket-and-where`

## What changed, for a person outside this repository

Nothing a person can see. This is a design that could not have worked being
corrected before anything was built on it, and one hole in a privileged part of
the machine being closed.

## The contradiction, and that it was provable from one document

ADR 0045 keeps read-only snapshots either side of a changing turn so the agent's
changes can be put back. `docs/contracts/kept-undo-folder.md` said the folder
they live in is **`0700 root:root`**, and said — two sections later — that
*the session that takes the bracket, which knows, writes it down* into that
folder.

**A session cannot traverse a folder with no search bit for it.** The two
sentences were in one file and could not both hold, and nothing had hit it
because nothing takes a bracket.

It would have been easy to read the mode as merely strict. It is not: the
amendment of 2026-09-21 had already measured that *taking* a read-only snapshot
needs no capability, and I measured it again — `btrfs subvolume snapshot -r` of
a subvolume the person owns, run as the person with no `sudo`, on a loopback
btrfs: exit 0, snapshot made. So the operation was never the obstacle. The mode
was.

## What else was found, and it is larger than the mode

**Nothing creates `/var/lib/alo/undo`.** No tmpfiles entry, no
`StateDirectory=`, no line in the image — searched and found none. **And
nothing takes a bracket**: `alo-turn` is where ADR 0045 puts it and names no
snapshot; nothing in the workspace creates a btrfs subvolume snapshot at all,
measured with a control (23 files mention `btrfs`, so the search worked).

So both ends of this feature are built and the middle is absent: the window
(`alo-keeping-up`), the person's surface (`alo-changing-undo`) and the
privileged remover (`alo-letting-go`) all exist and have nothing to work on.

## The resolution: ADR 0045 terms 8 and 9

**Term 8 — the folder is traversable, each person's directory is their own.**
`0711 root:root` on the folder: every person may traverse to their own
directory, nobody but root may list it. `0711` rather than `0755` because
listing would tell one person who else uses this machine, which is nobody's
business and which nothing needs. Each person's directory is `0700` and theirs.
A privileged step makes both at first sign-in — the session cannot make a
directory in a root-owned folder, and image build time does not know who the
people will be.

**Term 9 — everything below a person's directory is that person's own
writing.** This is the part that needed saying out loud, because the privileged
remover reads those files **as root**.

It also explains, retroactively, two rules the contract already had: the link
that is never followed, and the directory name that is never parsed. Those are
the rules you need when the person can write there. Under the `0700 root:root`
the contract claimed, **both would have been guarding against nobody** — which
is a quiet argument that the person was always meant to write there and the mode
was the error.

## The hole that followed from it, closed in the same change

`theirs.json` names a person's settings folder as an absolute path, and a root
unit on a timer then reads `undo.toml` from wherever it says. **Absolute was the
only thing checked.** One person could make root follow an absolute path of
their choosing.

It is now checked that the folder they name is **owned by them** — against the
uid that owns that person's own directory in the folder, which term 8 makes
authoritative. So the check needs no guess about where homes live and no list of
the people on the machine: the disk says who the directory belongs to.

**What it does not close**, said in the code rather than left to be found: the
folder is confirmed and then read, so a person could swap what the path names in
between. For a unit reading a retention window that is an acceptable window. The
check stops a person pointing root at a path that was never theirs, not a person
racing their own.

## Two corrections made while building it

**The first version refused a settings folder that does not exist.** That is the
commonest case there is — a person who never changed their window has no
`undo.toml` and may have no `alo` folder at all. Refusing stepped over their
whole directory, so their snapshots would never be swept and their disk would
fill: **worse than the hole being closed.** Three of this crate's own tests said
so immediately.

**The comparison is split from the `stat` so the test is deterministic.** The
suite runs as the person on a lane and **as root in CI**, so a fixture built out
of *a path whose owner differs from mine* is not the same fixture in both places
— and a test that quietly stopped distinguishing the two cases would be the
colour of a pass. Two numbers have no such problem.

## Measurements: where each one runs again

| what | where it runs again |
|---|---|
| a settings folder somebody else owns is refused | `alo-letting-go`'s unit tests, every build |
| a person's own settings folder is followed *(control)* | same |
| a settings folder that is not there is not a refusal | same |
| the 69 existing tests of this crate still pass | same |
| unprivileged `btrfs subvolume snapshot -r` exits 0 | **measured once**, 2026-10-16, on a loopback btrfs; nothing re-runs it |
| nothing creates the folder, nothing takes a bracket | **measured once**, same day, by searching; nothing re-runs it |

The two one-off measurements are written into ADR 0045 with what was run, and
they are the kind that would silently stop being true — the second especially,
since the whole point is that somebody will build it.

## What is still owed

- **The privileged step that makes the folder**, and **the bracket itself**.
  This change removes the reason they could not be built and builds neither. The
  contract now says plainly which of its sections are design and which describe
  a running machine, rather than leaving a reader to discover that the folder
  their code is about has never existed.
- ADR 0064's *snapshot before each run* waits on exactly that, which is why the
  sealed box keeps one of its three always-on protections and not the other two.
