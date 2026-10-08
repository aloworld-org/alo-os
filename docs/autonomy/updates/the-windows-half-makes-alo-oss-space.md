# The Windows half makes alo OS's space

Tasks 3 and 4 of the installer plan's task 4 chain: the Windows programs that
make and label the partition alo OS goes into when Windows is kept, and the
staging that runs them.

Before this change, the arithmetic for *alo OS beside Windows* existed and
nothing used it. After it, the whole Windows half exists and nothing offers it —
the refusal that keeps the Windows disk out of the offer is still in
`deciding.rs` and is flipped last, which is the owner's ruling of 2026-10-06:
*do not flip the refusal until the whole road exists.*

## What a person's computer now does, when the road is eventually offered

1. Windows is shrunk once, by the area **and** alo OS's space together. One
   shrink rather than two, because a machine whose second shrink failed would be
   half-moved.
2. The installer's own area is made and prepared, as before.
3. **alo OS's space is made and labelled `ALO-ROOT`** — by Windows, while
   Windows is running.
4. The command line staged for the restart says three things instead of one: the
   disk, the partition alo OS goes into, and the partition the firmware starts
   from.

## Three decisions worth reading

**Windows lays out its own disk.** The space could have been made after the
restart by the boot environment, which is already a Linux that can repartition.
It is made by Windows instead. A Linux repartitioning a disk whose Windows file
system was last written by a kernel that has not finished with it is how
somebody loses a Windows, and the whole promise of this road is that they do
not. alo OS only fills what it was given.

**The label is the permission.** The boot environment refuses to write any
partition that does not carry `ALO-ROOT`. So the labelling step is not
cosmetic — it is what makes the space usable at all, and a space that is made
and not labelled is a space the environment refuses. That is the safe way round
for a failure between the two steps.

**And no drive letter on it**, unlike the installer's area. A letter would put
the space in front of the person in Explorer as an empty disk inviting them to
put something in it, minutes before the environment overwrites it.

## The start-up area, and a road that is no longer offered when it cannot be walked

alo OS's loader goes into a directory of its own inside **Windows' own start-up
area**, because there is one of those per disk and Windows already has it.
Nothing in the installer carried which partition that was. It is now read from
the disk by GPT type and never by label, since Windows reports no label for it
at all.

The road is now offered only when that partition can be named. *Can this volume
give up the space* was never the whole question; *and can the loader go
somewhere the firmware will look* is the rest of it. This is the fault family
`docs/misreadings/` keeps — a check whose inputs are less specific than its
question — and the cure is that a road which cannot be walked is not offered.

## Three sentences that would have told a person a false number

**This is the most important thing in the change, and a test found it rather
than a person reading.**

`installer.shrinking-windows` and `installer.remains.smaller` both say how much
smaller Windows is. Both filled that number from the fixed one-gibibyte staging
area, because until now the area was the only thing Windows ever gave up. On the
road that keeps Windows it is tens of gigabytes.

Left alone, the installer would have said **"Making Windows on C: 1 GB
smaller"** to somebody whose computer was about to give up sixty — and that
sentence is the one a person agrees to. Afterwards, if the install failed, it
would have told them their Windows was a gigabyte smaller when it was sixty.

Neither sentence moved. The English is already correct for any amount, and a
placeholder name is part of the string a translator holds: renaming `{area}` to
something that reads better in Rust would leave every translated sentence
filling a placeholder the code no longer writes. ADR 0068's question — *would a
correct translation of the old English still be a correct translation of the new
one* — answers yes, so the key and the text stay. What changed is the caller.
**The Mac lane argued this and was right; the second site was found on this
lane.**

The promise in `sequence.rs` is now read from the shrink that will actually
happen rather than from the area's size. The same number today, and one place to
change rather than three when a road exists that takes more.

## A third kind of number

The failing test said *2 GB* where it meant one, and the reason is worth keeping.

Windows is shrunk to a mebibyte boundary, so the region freed is the area plus
whatever alignment rounded off — a gibibyte and under a mebibyte more. The
installer had two roundings: a size a person is told is **free** rounds down,
and a size they are told is **needed** rounds up, both so it never promises
space it does not have. Through `needed`, a gibibyte and a sliver reads *2*.

An amount **taken** is neither of those. It describes something that happened
rather than promising free space, so it rounds to the nearest gigabyte — the
only one of the three that is never wrong by more than half a gigabyte in either
direction. Both sentences use it now, so they say the same number on the same
run. They did not before.

## What is assumed and not yet measured

**That Windows' partition number is the partition's index in the GPT table**,
which is what udev counts and what the Linux partition name is derived from. On
a disk Windows itself laid out they agree, and the testing NUC's disk was read
from both ends on 2026-10-07 and its four partitions agreed, entry for entry.
It is not guaranteed in general: a GPT whose entries are out of order could
number them differently on the two sides.

**What catches it if it fails** is the `ALO-ROOT` label — the environment writes
no partition without it, so a name that pointed at Windows' own volume is
refused before a file system is made. That is a backstop and not the plan. The
first run on metal reads back what Linux actually calls these partitions and
compares, because a guard that never fires tells you nothing while it does not,
and the thing it protects is somebody's Windows.

## What is still owed on this chain

- **Task 5**, the fork the owner asked for: three choices rather than two — keep
  Windows and put alo OS beside it, use a separate empty disk, or replace
  Windows with alo OS only. The sentence that promises how much Windows gives up
  moves inside that fork, because the amount is the road's.
- **Task 6**, the refusal flip in `deciding.rs`, last.
- **Task 7**, the Fast Startup reading the testing NUC found.
- **The two metal runs**, in the owner's order: a failure injected before the
  point of no return with Windows still starting afterwards, and then the real
  install switching both ways.

## Also in here

`after_the_restart` had lost its documentation and its `#[must_use]` to a
scripted insert anchored on the line that began it rather than on the line that
ended the item above. `docs/misreadings/inserting-above-an-item-steals-its-doc-comment.md`
already recorded this fault four times; this change adds the fifth and sixth,
and a shape the entry did not have — the insert can land between an item's
*attribute* and its doc comment, where what the compiler reports is a duplicate
attribute, which names the right file and describes the wrong problem.

The vocabulary snapshot was regenerated on this branch by its own documented
command rather than carried from another, which is what
`docs/misreadings/a-file-replaced-wholesale-hides-what-it-reverted.md` is about.
Two lines added, none removed, none reworded.
