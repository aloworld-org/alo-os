# alo OS installs into a space beside what is already there

The boot environment could only ever install a **whole disk**, and it erased it
to do so. That is why *alo OS alongside Windows* — the promise in
[ADR 0023](../decisions/0023-installed-from-the-machine-it-replaces.md) §1.4 and
the installer plan's task 4 — had nothing underneath it. This change is the
underneath: the environment can now be told to put alo OS **into one partition**
and leave everything else on the disk alone.

## What a person gets

Nothing yet, on its own. This is the half of task 4 that runs after the restart.
The half a person touches — the installer on Windows asking whether to keep
Windows or replace it, making the space, and writing the choice into the staged
command line — is the next change, and the refusal that keeps a disk holding
Windows out of the list is flipped last, by the owner's ruling of 2026-10-06.

## What the environment can be told

Two words on the kernel command line, beside the disk's own name:

    alo.installing.into=<partition>   where alo OS goes
    alo.installing.efi=<partition>    the machine's start-up area, Windows' own

**Both or neither.** One without the other is not a road half described, it is a
road that cannot be walked: a root with nowhere to put a loader is an install
that finishes and starts nothing. Either half alone is refused before a disk is
looked at.

**Never both roads.** A line that also says `alo.installing.replacing=windows`
is refused. Replacing what is there and keeping it are opposite answers to the
one question the person was asked, and nothing here picks the safer of the two:
a line that says both is a line something went wrong writing, and what went
wrong might as easily have been the half that says *replace*.

## The one thing that stands between a mistake and somebody's Windows

The next thing that happens to the partition named is `mkfs.btrfs --force`. So
the partition is **not taken on its name**. It is taken only when it carries the
label `ALO-ROOT`, which the installer on Windows puts on the space it made and
on nothing else — exactly as it already labels its own staging area
`ALO-INSTALL`.

A check on the partition's *type* would not have done. The space for alo OS is a
basic data partition, and so is the volume Windows itself lives on; a type test
passes both. The label is the whole of the permission, and three tests hold it:
every one of Windows' five partitions is refused as the space, including the one
of exactly the right type; a partition carrying the label on a *different* disk
is refused, because a disk out of another machine can carry any label; and
something stacked on top of a partition is not a partition.

**A second run over a half-finished install is refused**, because the file-system
maker gives the partition its own label and the installer's label is then gone.
The way to try again is to run the installer on Windows again, which makes the
space again. That is the price of trusting one label and it is worth paying:
also accepting the label the maker writes would accept any partition anybody had
labelled `root`, on a disk this road exists to keep.

## What it then does, and what it never does

    mkfs.btrfs --force --label root  <the space>
    mount <the space>                /run/alo-os-root
    mount <the start-up area>        /run/alo-os-root/boot/efi
    bootc install to-filesystem --source-imgref … /run/alo-os-root

No `--wipe`, no `to-disk`, no `--filesystem`. **Every one of the three steps is
checked**, and the reason is in the code: a `mount` that failed and went
unnoticed leaves the writer installing into an empty directory in the
environment's own memory — which succeeds, says so, and leaves a disk with
nothing on it.

## A correction this change makes to an older promise

`Program::writes()` is what every refusal test counts to prove that a refusal
wrote nothing. It listed the installer and the staging-area removal, and when
the file-system maker arrived it was not added — so *the most destructive program
in the environment was not counted as writing to a disk.* It is now, and a test
asserts it rather than assuming it. Mounting is still not counted: nothing on a
disk comes out different, and a mount that failed is caught by being checked.

## What a person is told, and what is kept off the screen

One new sentence, and it names no machinery:

> The space for alo OS on the disk {disk} is not there, so nothing was changed.
> Start Windows and run the alo OS installer again

Seven different faults end in that sentence — a partition that never appeared, a
partition on another disk, a partition without the label, a start-up area that
is not one, the same partition named twice, a read-only partition, a mounted one
— because they are one thing to the person and there is one thing to do about
all of them. **Which of the seven it was goes to the machine's log**, where
whoever is helping them can read it, along with the exact word on the command
line that was wrong. The kernel words themselves are never on the screen.

## How it was tested

71 unit tests in the crate and 24 walks of the sequence against a machine made
of answers, all passing. The walks the road added:

- **the road end to end** — the file system made on that one partition, the root
  and the start-up area mounted in that order, the writer handed a root and no
  disk, Windows left with its place in the firmware's list behind alo OS, and
  **no partition of Windows' named to any program at all**, checked by collecting
  every argument of every program that ran;
- **Windows' own volume named as the space** — refused, no file system made,
  nothing mounted, and the log saying it was the label;
- a space named on another disk, a space that never appeared, half a road, and
  both roads at once — each refused, each writing nothing.

There is no hardware run in this change, and this task is not closed by it. The
walk task 4 ends at is the testing NUC, on metal, two runs in the owner's order:
a failure before the point of no return with Windows still booting afterwards,
and only then the real install with switching both ways.
