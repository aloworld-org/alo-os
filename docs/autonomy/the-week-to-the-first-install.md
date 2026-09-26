# The week to the first install

**Friday 2 October 2026**, alo OS is installed on a physical machine for the
first time. Nothing has ever run outside a guest, which makes it the largest
unknown in this release and the one thing no further building answers.

This is the six days before it: who does what, what must be true by Thursday
evening, and what is deliberately not happening this week.

## The days

**Friday needs one installer task, and it is task 4.** The walk installs
*alongside* Windows. Task 7 is *replace Windows* — a road Friday never takes —
and tasks 19 and 20 do not touch it either. They are this release's work and not
this week's, and the first version of this table made them look like the same
thing. The development PC does task 4 and then the rehearsal, with nothing
stacked behind them.

| | Development PC | Third PC | Mac | This checkout |
|---|---|---|---|---|
| **Sat 26** | installer 4 — the fall-through test | keeps-itself 15 | shell 14, steps 3–5 | finish the palette change |
| **Sun 27** | installer 4 | keeps-itself 15 lands | shell 14 lands | reconcile: local network, access and language |
| **Mon 28** | installer 4 | session 12 | `nested_check` into the gates; the removed coverage written down | reconcile: settings, capture |
| **Tue 29** | installer 4 lands, or says what it is waiting for | session 12 lands, then installer 20 | devices 1 — **the three Wayland protocols first** | reconcile: documents, software and the web |
| **Wed 30** | build the rehearsal guest from the testing PC's own layout | installer 20 | devices 1 — the pipeline packaged | reconcile: the installer, the machine keeps itself |
| **Thu 1 Oct** | **the rehearsal** | installer 20 | devices 1 | the go/no-go, written down |
| **Fri 2 Oct** | on hand to diagnose | — | — | reading the install as it happens |

**Task 4 has four days rather than one.** It is the only thing between this
repository and a machine, so it gets the room. If it lands on Saturday the
development PC takes task 7 with the days it saved; if it takes until Tuesday,
nothing else was displaced. What is not available is a day that says *installer
4 lands today* when nobody knows that.

## Thursday is a rehearsal, not a spare day

**The whole install, in a guest on the development PC, with the testing PC's
own disk layout and firmware settings.** Friday should not be the first time
that combination is seen. A rehearsal that passes costs an afternoon; a Friday
that fails on something a rehearsal would have caught costs the week.

What the rehearsal needs from the testing PC, gathered before Thursday: its
make and model, memory, firmware type, Secure Boot state, whether a TPM 2.0 is
present, the number of physical disks and their partitions, and whether its
Windows is the original installation.

## The go/no-go, decided Thursday evening

**Go** requires all four:

1. **Installer 4 has landed, fall-through included.** *Windows still comes up
   when alo OS's loader is broken* is the one failure nobody wants to meet first
   on a physical machine.
2. **The testing PC's own facts are recorded.** `docs/hardware.md` names
   machines; a walk on an unnamed one produces evidence nobody can repeat.
3. **The machine is reachable over the network**, so the Windows side of the
   install is read as it happens rather than transcribed from a screen. The
   firmware, shim and GRUB moments cannot be captured on any machine and are
   watched by a person.
4. **The rehearsal passed**, or its failure is understood.

**No-go is not a failure.** It moves the walk to the following week and the
reason is written down. What is not available is going ahead while pretending
one of the four was met.

## What is knowingly missing on the day

So that silence cannot imply more was tested than was: **no video plays**, the
image has no media pipeline; **no update can be applied**, it ships no signature
policy; **the camera and microphone are refused**; and **the three promises only
a chip can keep are unmeasured**. Friday tests the install and the first start.
Nothing else, and the report says so.

## What is not happening this week

**No canvas, no new screens, no v1 work.** The design is far enough ahead of the
implementation that a week costs nothing, and every hour spent there is an hour
not spent on the nine tasks between here and a finished release. The colour
change finishes because it is already written; the mark and the typefaces wait
for the image work, because they ship in the image and the recipe should move
once, not twice.

## The reconciliation, running alongside

The exit gate says **8 of 58**, while 214 of 227 engineering tasks are done. The
gate has three kinds of box — 38 promises, 10 code halves, 10 machine halves —
and **no promise has ever been ticked**. That is not a measure of how much is
built; it is a measure of how long since anybody read the instrument.

One section a day, landed on its own, ticking only what has evidence a reader
can check and saying plainly where there is none. Where a promise was never
split into a code half and a machine half, it is split, because that distinction
is the whole reason the gate exists.
