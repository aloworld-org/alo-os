# What the first readable boot found

Date: 2026-10-11. Workstream: the third PC, `image/` and `crates/alo-image`.
Responsible contributor: third-PC lane. Status: ready for integration.

Two questions had been open for a day and treated as answerable: whether
`alo-portald` starts healthy, and whether `alo-shipping` installs the seven
applications. Both were recorded as *waiting on the next boot with
`forward_to_console`*. That boot has now happened. **Both questions had the
wrong shape, and the boot found a third fault nobody was looking for.**

## What was run

Built from `main` at `b3e47707` — the commit was checked against
`origin/main` before building rather than after, and the build refused to
start if the tree did not carry `console.toml`. Written to a 20 GB raw disk
with `bootc install to-disk --via-loopback --wipe --filesystem btrfs`, exit 0.
Booted in qemu, serial captured to a file.

**Under TCG rather than KVM, and that is a measurement.** `/dev/kvm` exists
but `/proc/cpuinfo` exposes neither `vmx` nor `svm`, and `modprobe kvm_intel`
answers *Operation not supported* — nested virtualisation is not available to
this WSL VM today, though it was on the 10th. Software emulation is perhaps
ten times slower and says the same things on the serial line. Every duration
below is an emulated one and none of them is evidence about real hardware.

Every probe carries `probe_should_be_7=$( (exit 7); echo $? )` and every one
returned 7, so the exit codes reported are the ones the commands gave.

## 1. `alo-shipping` never runs, and nothing says so

```
alo-shipping.service - The applications a fresh machine has skipped,
unmet condition check ConditionFirstBoot=yes
```

A machine written this way boots with **none of the seven applications** and
reports success. The unit is enabled, correct, and skipped.

**The cause, after the first attempt at it was wrong.** This document first
said `bootc install to-disk` populates `/etc/machine-id` and so consumes the
first boot. That was inferred from a 33-byte file on a disk that *had already
booted*, and a later run measured the three states properly:

| | |
|---|---|
| `/etc/machine-id` in the container image | **0 bytes** — the convention that makes the next boot a first boot |
| on a freshly written disk, never booted | **0 bytes** |
| after one boot | **33 bytes**, with that boot's own mtime |

So the **boot** populates it, not the install, and the recipe never names it.

**And the condition was still unmet on that boot.** On a disk that could not
be fresher, with an empty `machine-id` in the deployment,
`first-boot-complete.target - First Boot Complete skipped, unmet condition
check ConditionFirstBoot=yes` at 171 seconds — before anything of this
project's ran. systemd had already decided it was not a first boot.

Why it decides that is not measured and is not claimed: the likely mechanism
is that systemd settles first-boot from the `/etc` it sees in the initrd, and
on an ostree deployment that is not the `/etc` carrying this `machine-id`.
**What is measured is that it never fires** — on two independently written
disks, one of them proven fresh — which is all the fix needs to be true.

The unit's own comment reads *ConditionFirstBoot, not a stamp file of our own
— systemd already knows what a first boot is*. That is true of systemd and
false of a bootc-installed machine, and it is the kind of correct reasoning
that is wrong about one install path. **Not fixed in this change** — the
replacement is a stamp file the unit writes itself, which is a design decision
about where that file lives and what happens when it is deleted, and it should
not be smuggled in beside a logging fix. It is the next thing this lane does.

## 2. `alo-portald` was never going to start, and the question was wrong

No entries in the journal at all. Not a failure — it is `WantedBy=
user@1000.service`:

| unit | wanted by |
|---|---|
| `alo-agentd` | `user@1000.service` |
| `alo-portald` | `user@1000.service` |
| `alo-convertd` | nothing — socket-activated |
| `alo-shipping`, `alo-sessiond`, `alo-modeld`, `alo-compositor`, `alo-boundaryd` | `multi-user.target` |

Nobody signed in during this boot, so the two session-scoped services
correctly did not start. **"Is `alo-portald` healthy" is not answerable by any
boot that does not sign somebody in**, and a day was spent waiting for a boot
to answer it. The question is retired and replaced: *does `alo-portald` start
and stay healthy when a person signs in* — which needs #621's desktop unit,
and is therefore behind the Mac's #620.

Also measured, and the one piece of plain good news: **zero failed units.**
`alo-compositor` reached `Started` 19 emulated seconds after `Starting`.

## 3. The serial console does not carry the journal — a correction

**`console.toml` claimed `systemd.journald.forward_to_console=1` put the
journal on the serial port. It did not** — and the way that was measured
matters, because the first measurement of it was wrong.

**The wrong measurement, recorded because it nearly became the finding.** The
first pass counted how many of the 856 captured lines carried a kernel
timestamp prefix, reasoning that a forwarded journal line would not. All 856
did, and the conclusion drawn was *not one line came from the journal*. That
discriminator is worthless: **journald's console forwarding prefixes each line
with its own monotonic timestamp, in the same format the kernel uses.** The
numbers were real and measured nothing.

**The right measurement is content.** Four probes, each a string only a
journal entry could carry, against three boots, with a control that returns 0:

| | main | + `tty_path` karg | + journald drop-in |
|---|---|---|---|
| ran to | 350 s | **915 s** | 504 s |
| `alo-compositor: the sign-in screen is going up` | 0 | 0 | **1** |
| `alo-boundaryd: the boundary is on this kernel` | 0 | 0 | **1** |
| `alo-sessiond: the door at /run/alo-sessiond` | 0 | 0 | **1** |
| `unmet condition check ConditionFirstBoot` | 0 | 0 | **2** |
| `systemd[1]:` lines | 122 | 131 | **325** |

So the serial log did carry systemd's own early-boot status writes — that is
what the 122 lines are — and **not one line of any alo service's own output**,
which is the half that matters: those services are what the machine exists to
run.

The gain from #623 was real and was not that — 89 KB where there had been
3.8 KB, the whole kernel ring buffer and the whole audit trail, against a log
that used to stop at GRUB's last line, which on image 0.0.6 reads *Booting
`alo OS 0.0.6 (ostree:0)'* and is the deployment GRUB names rather than a
milestone.

The reason is `console.toml`'s own other decision. `forward_to_console` writes
to the **primary** console; that file deliberately puts `tty0` last so the
primary console is the screen a person is watching. **Both decisions are
right, and together they send the journal to the one place a reader of a
serial port cannot see.** Two correct decisions that contradict each other is
worse than one wrong decision, because neither looks wrong on its own — and
the two existing checks both passed.

### The first fix was wrong, and it passed the gate

**`systemd.journald.tty_path=/dev/ttyS0` was added to `console.toml`, gated
green, built, booted, and changed nothing.** All four content probes still
returned 0 — on a boot that ran to **915 seconds**, nearly twice as far as the
one that found the fault and nearly twice as far as the one that fixed it. A
longer boot with none of the lines is the strongest form this evidence takes:
duration is not the explanation.

journald accepts exactly ten `systemd.journald.*` kernel arguments, asked of
the binary rather than of a manual page, with a control that returns 0:

```
forward_to_console  forward_to_kmsg  forward_to_syslog  forward_to_wall
max_level_console   max_level_kmsg   max_level_socket   max_level_store
max_level_syslog    max_level_wall
```

`tty_path` is not among them. **A kernel argument journald does not know is
not an error** — GRUB carries it, the kernel carries it, `/proc/cmdline`
shows it, and nothing reads it. Every artefact of that fix looked correct: the
file said the right thing, the new check passed, the build succeeded, the boot
succeeded, and the journal was still on the screen. It was wrong for one boot
only because the fix was booted before it was landed.

### The fix that works

`TTYPath` is a *journald.conf* key, not a kernel argument.
`/usr/lib/systemd/journald.conf` ships it commented as
`#TTYPath=/dev/console` — and `/dev/console` is the primary console, which
`console.toml` makes the screen. That default is the whole cause.

So: `image/usr/lib/systemd/journald.conf.d/20-the-journal-goes-to-the-reader.conf`
sets `TTYPath=/dev/ttyS0`, after the base's `10-centos-bootc-persistent.conf`
(both are wanted — persistence is what let this be diagnosed at all, by
mounting the written disk and reading the journal the console never carried).

**Watched working before landing.** Built, written and booted: every one of
the four probes present, where the two boots before it had none, and
`systemd[1]:` lines up from 122 to 325. `alo-shipping`'s skip message — the
first finding in this document — is now readable on the serial port, where
before it took mounting the disk to see.

**And the recipe had to be told to copy it.** `image/Containerfile` copies
`usr/lib/systemd/system/` and `usr/lib/bootc/kargs.d/` by name, not
`usr/lib/systemd/` wholesale, so the drop-in would have been written,
committed, checked and absent from every image. Found by reading the recipe
for the copy rather than assuming a tree was copied.

### And the check that should have caught it

`a_reader_can_keep_the_boot()` asks whether a serial console exists.
`the_whole_journal_is_forwarded()` asks whether forwarding is on. **Both were
true of this image.** The gap between two passing checks and an unreadable
boot is what a third check is for:

`the_journal_reaches_a_reader()` — forwarding on, **and** the forwarded path
is one of this image's own serial lines. An unnamed path is refused, because
unnamed means the primary console means the screen.

This is the same shape as the fault the Mac lane found in
`a_reader_can_keep_the_boot` on 2026-10-10 by running it with controls: a
predicate testing something weaker than its name claims. So the new one is
tested the same way — a case where both older predicates answer yes and the
new one answers no, four paths that are named but are not this image's lines,
a named path with forwarding off, and the shipped file itself. Plus the
mutation: taking the path back out fires `TheJournalGoesToTheScreenAndNotTheReader`
**and neither of the other two**, which is what makes it a third check rather
than a second spelling of one that existed.

## What a reader should take from this

The two questions were not waiting on evidence. One was unanswerable as
asked, and the other had an answer that no amount of waiting would have
produced, because the unit that would answer it was skipped before it ran.

And the instrument built to answer them was itself broken in a way that two
green checks certified as working. That is three faults today of the same
shape — #624's edge drawn on one road and not the other, #626's controls that
could be drawn and not clicked, and this — where everything tested passed and
the thing nobody had thought to test was the thing that was wrong.
