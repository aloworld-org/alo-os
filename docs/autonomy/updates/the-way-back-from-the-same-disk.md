# The way back from the same disk

Task 4's own title is *Alongside Windows, switching between them easily, **and
back again***. The back-again half did not exist, and nobody had listed it.

A person who installed alo OS beside Windows and changed their mind could not
remove it. The removal program refused — correctly, because it clears a **whole
disk**, and on that road the disk is Windows'. So the one thing they needed was
the one thing it would not do.

## It was found by reading, not by running

The install road was being built; the removal road was never checked against it.
`remove_alo_os` finds what to clear with `Disks::the_one_alo_os_is_on`, which
**excludes the disk Windows is on**, and then clears that disk whole.

**That exclusion is a guard, not an oversight.** Lifting it without changing
what the program erases would make *remove alo OS* erase Windows. The refusal
was right; what was missing was a road.

## The part that mattered most was a sentence

The first line the removal said, before any disk had been read, on every road:

> This removes alo OS from this computer. **Windows, and your files on it, are
> not touched.**

On the keep-Windows road that promise was true **only because the guard refused
two screens later.** It was not describing what the program does — it was
describing what the program declines to attempt. And the day somebody lifted
that filter, it would have become the installer telling a person their files
were safe while it erased them, **with every test still green, because nothing
held a sentence to a filter.**

Found by the Mac lane, which read the whole removal vocabulary after one
sentence of it was recorded. Seven of its twelve sentences assumed alo OS had a
disk to itself.

So the claim came apart (ADR 0068 retires a key rather than rewording one):

| | |
|---|---|
| `remove.beginning` | says what the program is, and promises nothing |
| `remove.windows-is-not-touched` | said **after** the lookup has answered |
| `remove.on-the-windows-disk` | the true reason, where the old sentence said *the disk could not be found* |

That last one matters on its own: the disk **was** found. It is Windows'. A
person told it could not be found goes looking for a hardware fault that is not
there.

## And a test that cannot be satisfied by the guard alone

`a_promise_about_windows_waits_for_the_guard_that_makes_it_true` reads the
source and fails if the exclusion goes while a sentence still promises in
advance.

The shape is the Mac lane's, and so is the warning that made it worth building:
**a test asserting only that the filter excludes Windows' disk passes forever
and says nothing about the sentence.** What is wanted is the pair.

**It was watched failing.** A test asserting that something has not happened yet
passes on the day it is written whatever it says, so the guard was lifted, the
test went red with a message telling the next person to rewrite it rather than
delete it, and `disks.rs` was restored from a copy taken beforehand and read
back.

It also strips comments before matching, because this module's own documentation
quotes both the guard and the retired promise in order to explain them — and a
guard that reads prose as code refuses the explanation of the thing it is
guarding, whose fix would be to delete the explanation.

## Then the road itself

The firmware entry first, for the other road's reason: while it is there nothing
has been taken away and the installer can simply be run again. Then alo OS's own
partition, under the **same `still_the_space` guard the road that made it
uses** — a partition number is Windows' to reassign, the place a partition
begins does not move, and a number alone is one renumbering away from taking
somebody's Windows.

The disk listing now reads where each partition begins, which that guard needs.

## What it deliberately does not do

**It does not grow Windows back.** Three reasons, and none of them is that it
was unfinished:

- the size Windows had is written down nowhere this program can read at removal
  time;
- extending into free space is one step in Windows' own disk management, which
  does it safely and reversibly;
- **a Windows half-grown by a program that guessed is worse than one a person
  extends.**

The sentence says so plainly rather than leaving a person to notice, and names
the tool they need.

## What is still owed

Growing Windows back, if it is ever worth doing in the program rather than in
Windows'. It needs a size read at removal time and a reason better than *we
could*.

And the six other removal sentences that assume alo OS has a disk to itself.
Three are replaced here; `remove.will-erase`, `remove.erasing`, `remove.gone`
and `remove.disk-not-cleared` still speak of erasing a disk, and are correct on
the road that has one. Whether that road's sentences should name the road is a
question this change does not answer.
