# The person chooses two systems or one, and then where alo OS goes

The owner, 2026-10-06: *"I want us to include the function/feature where the
users chooses to run 2 os or run only alo os"*. The installer plan's task 5.

## What a person is asked

**First**, before anything is described:

> Do you want both Windows and alo OS on this computer, or alo OS on its own?

**Second**, and only when their computer has both places:

> Where should alo OS go: on the same disk as Windows, or on the empty disk
> {disk}?

One disk and no empty one, and alo OS goes beside Windows, because that is the
only place it can go. An empty disk and no same-disk road, and it goes there.
Neither is a choice, and **a question with one answer is an invitation to type
something that will be refused.**

That is why there are two questions rather than one menu of three answers. A
menu would have offered *use a separate empty disk* to somebody with no second
disk — the testing NUC, for one — and then refused them for choosing it.

Both questions are permanently two-answered, and the words to type live in a
prompt sentence of their own beside each. So adding a road later adds sentences
and retires none.

## The order changed, and that is the change

The installer used to say what it would do, and then ask which road. That works
while every road does the same thing to Windows.

This one does not. It takes tens of gigabytes rather than one, it does not give
them back, and the restart replaces nothing. So each road now says its own
sentences, after it is chosen and before it is consented to.

Two sentences that landed with task 1 had never been said by anything:
`installer.will.give-alo-os` and `installer.will.restart-keeping-windows`. This
is what says them. **Saying the wrong restart sentence would tell a person their
disk is about to be erased while the installer does the opposite**, and nothing
would have caught it, because each is correct on its own road.

The consent that follows offers the disks of the chosen road and no others. A
person who chose the empty disk cannot reach the Windows disk by typing its
name, and the other way about.

## The first retirement this installer has had

| | |
|---|---|
| retired | `installer.ask-which-road` |
| replaced by | `installer.ask.two-systems-or-one`, `installer.ask.type-keep-or-replace` |
| its promise replaced by | `installer.will.windows-does-not-get-it-back`, `installer.will.windows-lends-the-area` |

The snapshot diff shows one removal and seven additions, and says nothing about
them being the same sentence coming apart. So this does.

**Why it could not be reworded.** It held three clauses under one key — a
question, a description, and a promise:

> Windows can be kept, with alo OS beside it, or replaced. **Keeping it changes
> nothing you cannot undo.** Type {keep} or {replace}

That promise is honest about giving alo OS a disk of its own: Windows lends the
installer's 1 GB area and gets it back when the installer has finished, whether
it finished or not. It is **false** about putting alo OS on the disk Windows is
on, where Windows gives up its space and does not get it back while alo OS is
there — by design, and not by failure.

A sentence cannot promise reversibility for both roads, because the two roads
differ in exactly the thing it promises. ADR 0068's question — *would a correct
translation of the old English still be a correct translation of the new one* —
answers no, so the key retires and the claim comes apart into pieces that are
each true where they are said.

**What it cost:** a snapshot diff. There are no translation catalogues yet, so
the first retirement happened while retirement is nearly free, which is the
right order to build the habit in. A retirement needs no `reworded.txt`
declaration, and neither key had one.

The placeholder names in the sentences that stayed were **not** renamed, even
where `{area}` now holds something that is not the area. A placeholder is part
of the string a translator holds: renaming it would leave every translated
sentence filling a gap the code no longer writes. The Mac lane argued that and
was right.

## It stops where the truth stops

`windows-does-not-get-it-back` says Windows does not get that space back while
alo OS is on the computer. It does **not** say that removing alo OS later gives
it back.

The sentence written first said exactly that. Checking whether it was true is
what found that `removing.rs` clears a disk alo OS has to itself and refuses
this road entirely — recorded in the plan with task 4, along with the six other
removal sentences written for a road where alo OS has a disk of its own.

A sentence promising a way back that no program walks is the worst kind of
false, because it is the kind a person relies on.

## The dormancy is asserted, not assumed

The same-disk arm is unreachable today. `deciding.rs` offers no disk carrying
the keep-Windows shrink, which is term 5 of the owner's ruling of 2026-10-06:
*the refusal that keeps the Windows disk out is flipped last, when the whole
road exists and not before.*

That is a reason for the arm to be dormant and **no reason at all to leave the
dormancy unrecorded.** *Task 6 is one line* was a sentence in a message, and
messages are not the repository.

So `nothing_offers_the_same_disk_road_yet` asserts that a one-disk computer is
still refused `NoDiskForAloOs`, and its failure message tells whoever makes it
fail what to do rather than inviting them to delete it.

**It was watched failing.** A test asserting that something has not happened yet
passes on the day it is written whatever it says, so task 6's line was written
into `deciding.rs`, the test went red, the other four stayed green, and
`deciding.rs` was restored from a copy taken beforehand rather than by reversing
the edit — then read back, with `git diff` confirming it byte-identical to
`main`.

It names the refusal rather than asserting `is_err()`. The first version
asserted `is_err()`, which would have passed on a machine refused for Secure
Boot or for unreadable disks, and gone on passing while the thing it is about
changed underneath it.

## What this road does to the testing NUC

From that machine's own bytes, read 2026-10-08, and computed twice
independently — by the laptop lane running `beside_windows` on them, and by
hand here — agreeing to the byte:

| | bytes | |
|---|---|---|
| Windows now | 118,873,915,392 | 110.71 GiB |
| Windows after | 92,030,369,792 | 85.71 GiB |
| the installer's area begins | 92,257,910,784 | 85.92 GiB |
| alo OS gets | 25,769,803,776 | 24.00 GiB exactly |

24 GiB is `THE_LEAST_DISK`, so that machine gets the floor and not a byte more.
It has 69 GB free of 110 GB, and after Windows' own 16 GiB reserve there is room
for the floor and little else.

**And the three sentences add up on it**, which is what
`installer.will.give-alo-os` says they must: *Windows is made 25 GB smaller*, *a
1 GB area for the installer*, *alo OS gets 24 GB of that space*. A person who
adds the last two gets the first.

### A correction, because the first version of this table was wrong

It read *110.9 GiB to 85.8 GiB, area at 86.0* and called them that machine's
numbers. They were computed from a **test fixture whose disk size I had chosen
myself** — 119,034,123,776 bytes against the machine's 118,873,915,392. The
shape was right, the numbers were not, and attributing them to a machine they
did not come from was the error.

It is left in rather than swapped out because the mechanism is worth having: a
fixture written to be *like* a machine reads, in its own author's hands, as the
machine. The cure is the one this change already follows elsewhere — a figure
about a particular computer comes from that computer, and the fixture's job is
to exercise the code rather than to stand for the hardware.

The NUC found it by recomputing from its own bytes, which is also how the
figures above came to be checked twice.

## Also in here

`ASKED_AGAIN` had two copies, each with a comment saying it was the same number
the other one used. A third copy is how a number stops being the same one, so it
moved to `asking`, which already owns the rule for reading any typed answer.
Three questions now share it, and each ends at the answer that changes least:
Fast Startup left on, Windows kept, alo OS on the empty disk.

The test machine that answers questions moved from `the_replacing_road`'s tests
to `machine.rs`, beside the trait it implements — the reason
`disks::tests::PRINTED` is shared rather than copied. Its methods panic where a
question should never reach them, and its `ask` now documents that running out
of answers gives an empty line, which is what a console that has gone away
gives: every question in this installer has to be safe in that case, and the
refusal tests rely on it.

## What is still owed

- **task 6**, the one line in `deciding.rs`, which this change made ready and
  which is last by the owner's ruling;
- **task 7**, the Fast Startup reading — **and it is not the fix the plan
  described.** The three readings asked of the NUC came back and turned the
  design over: `HiberbootEnabled` is 1, `HibernateEnabled` is **absent** (only
  `HibernateEnabledDefault` is there, at 1), and `C:\hiberfil.sys` **exists**,
  at 3.37 GB. So that machine's hibernation is on and its Fast Startup is on,
  the installer's *could not be found out* was honest, and the planned fix —
  *read whether `hiberfil.sys` is absent* — would have told **every** machine
  that Fast Startup was off, because the file is locked whenever Windows is
  running and `Test-Path` reports a locked file as missing
  (`docs/misreadings/a-locked-system-file-reads-as-a-file-that-does-not-exist.md`);
- **the way back** from the same-disk road, recorded with task 4;
- **the two metal runs**, in the owner's order.
