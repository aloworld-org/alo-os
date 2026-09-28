# The first install on a real machine

This is the page you follow at the machine. `docs/booting.md` is for somebody
building the image; this is for the person standing in front of a computer that
still has Windows on it, with a phone in the other hand to photograph the
screens nothing can capture.

It is written to be followed once, on the testing PC, and then again on the
certified laptop without changing. Task 6 of
[the installer plan](autonomy/v0-5-the-installer-plan.md) is what it serves, and
[`autonomy/v0-01-evidence.md`](autonomy/v0-01-evidence.md) is where what you saw
is written down afterwards.

**The machine is the testing PC.** Not the certified laptop, which stays clean
for the certification run after this, and not the development PC, which is the
only machine here with hardware virtualisation — losing it would cost the lane
and the ability to diagnose whatever this walk finds. The testing PC can be
reinstalled as many times as the first attempt needs, which is the property that
matters most.

## Before the day

### Two things that must be true, or the day ends at step 2

Both are read out of the installer rather than assumed, and the installer refuses
rather than asking, so neither can be fixed once the walk has started.

- **Secure Boot must be OFF.** `deciding.rs` returns `Refusal::SecureBootOn` when
  it is on, and `Refusal::SecureBootNotRead` when it cannot be read — and ADR
  0033 §4 says the installer **never suggests the setting be changed**, so the
  program will refuse and will not tell you what to do about it. Turn it off in
  the firmware before the day, and write down that you did: the walk is then a
  walk with Secure Boot off, and the roadmap box says so.
- **The machine needs a second, empty disk of at least 24 GB.** Not free space on
  Windows' disk — a separate whole disk. `lib.rs` is plain about it: *the
  environment replaces one whole empty disk; putting alo OS into the same disk
  beside Windows is the installer plan's task 4* — and task 4 is **in progress**.
  A single-disk machine cannot be walked at all, so if the testing PC has one
  disk, that has to be solved before Friday rather than discovered on it.

### The machine's own facts, written down first

A walk on an unnamed machine produces evidence nobody can repeat. Fill this in
and add it to [`hardware.md`](hardware.md) **before** anything is installed,
because most of it cannot be read afterwards.

| | |
|---|---|
| Make and model | |
| Memory | |
| Firmware | UEFI / legacy BIOS |
| Secure Boot | on / off |
| TPM present | yes / no, and the version it reports |
| Windows | the original installation / a reinstall |
| Reachable from the development PC | yes / no, and at what address |

On Windows: **System Information** (`msinfo32`) gives make, model, memory,
firmware type and Secure Boot state. **`tpm.msc`** gives the chip and its
version. If `tpm.msc` says *Compatible TPM cannot be found*, write that — it is
an answer, and it decides what the machine asks for at first boot.

**Reachability matters more than it looks.** The Windows side of the install
should be read as it happens rather than transcribed off a screen afterwards.
The firmware, shim and GRUB moments cannot be captured on any machine and are
watched by a person — those are the ones to photograph.

### What to have ready

- The machine, plugged into power, on a wired network if it has a port.
- A phone, for the firmware and boot-menu screens.
- The disk name you expect alo OS to land on, so a wrong one is obvious when
  the installer says what it found.
- Somewhere to write timings. How long the pull takes on real hardware is a
  number nobody has yet.

## The walk

### 1. Download from the Release

The published image is pinned by digest and signed. Download the installer from
the Release page, on the machine itself.

**Look for:** the file arriving whole. Nothing else is checked here by you — the
installer checks the digest itself at the next step, and that is the check that
counts.

### 2. Run it, and read what it found

**Start it as an administrator.** `administrator.rs` checks for those rights
*before it asks anything else*, deliberately, so that nobody is shown a list of
what their computer is and told at the end that nothing could be done.

**Windows will warn you that the publisher is unknown.** The executable is
**unsigned** — there is no code-signing certificate, and ADR 0046 decides the
release ships unsigned and says so rather than an agent obtaining a certificate
that would make the publisher of alo OS somebody nobody chose. Expect
SmartScreen, choose *More info* then *Run anyway*. Do not confuse this with the
installer's own refusal, which is a different sentence: *this download is not a
genuine alo OS, so nothing was changed*. That one has no way past it, and means
the download really is not genuine.

It then reads the machine before it offers anything: administrator rights,
whether the environment beside it is genuine, UEFI, Secure Boot, TPM, BitLocker,
free space, memory and the disks — each from Windows' own tools, and each said
aloud to you.

**Look for, and write down:**

- **the disk it names**, and whether it is the one you expected;
- **the size it says it needs** — 24 GB is the floor, and a machine with no
  empty disk that size is refused in words with nothing changed;
- **what it says about BitLocker.** A volume BitLocker is still encrypting or
  decrypting is refused; a volume simply encrypted is not the same case. Write
  down which of the two it said.
- **what it says about memory.** It decides nothing — less memory is a slower alo
  OS, not a broken computer — but it says a second sentence below what
  `docs/hardware.md` designs around, and that sentence is worth recording.
- **whether it asks about Fast Startup.** It asks only when Fast Startup is on,
  after the consent and before anything is changed. If it asks, answer *turn
  off*: Windows' hibernation image would otherwise make the disk unsafe to
  share. The setting is journalled and put back if a later step fails.

**If it refuses:** that is a result, not a failure of the day. Photograph the
sentence and write it down verbatim. A refusal that says why is the behaviour;
a refusal nobody can read is the bug.

### 3. Type the consent

It asks for the name of the disk alo OS replaces, typed out. Nothing before this
point has changed the machine.

**Look for:** that the name it asks for is the name it showed you in step 2.

### 4. Reboot, and watch the environment install

The machine restarts into the boot environment, which pulls the image and
installs it.

**Look for, and time it:**

- the pull starting, and whether it says anything while it runs;
- **how long it takes** — this is the first measurement on real hardware and on
  a real network;
- `Creating rootfs`, which is where a known stall has been seen before, so if it
  stops there for more than a few minutes say so rather than waiting silently;
- the sixth step after the install: the firmware entry named **alo OS**, Windows
  Boot Manager directly behind it, and the installer's own staging area taken
  away.

**If the download stops arriving**, the install should end in words rather than
saying *Still installing* for ever. If it does not, that is the finding, and it
is worth more than a clean run.

### 5. The boot menu

The machine restarts again and shows the menu.

**Look for:** **alo OS** and **Windows Boot Manager**, both named, with alo OS
first. Photograph it. Choose alo OS.

### 6. Reaching `alo-agentd`

**Look for:** the console reaching `alo-agentd`. That is the end of the install
and the beginning of everything else.

Write down anything it prints on the way that looks like an error, even if it
carries on afterwards.

### 7. Go back to Windows, and come back again

Both directions matter, and the way back is the one that decides whether this
machine is safe to leave with somebody.

- Restart, choose **Windows Boot Manager** in the menu, and confirm Windows
  comes up.
- From inside Windows, start the installer with the switch's word: it sets the
  firmware's **next start** only and restarts. Confirm alo OS comes up, and
  that the start *after that one* is Windows again — the default should never
  have moved.

## The four checks

Each of these is a promise the roadmap carries with an on-the-machine box, and
each box moves only for what you actually saw.

### The GPU works on first boot, where there is one

[`hardware.md`](hardware.md) defines it, and all four parts must hold:

- the display comes up at **native resolution** without configuration;
- the GPU is available to the model runtime with **no driver installation**, no
  CUDA or ROCm archaeology, no virtualenv;
- pulling and running a model from the catalogue is **one command**;
- an upgrade cannot break that stack.

**On the day you can answer the first.** Look at the display: is it at the
panel's own resolution, or a fallback? The rest depend on the check below, which
depends in turn on whether a command of ours has landed by then — see the note
there.

### The boundary attaches on this kernel

**Ask the machine, not the kernel config.** `alo-boundaryd` is in the image, and
its whole job is to load the boundary and pin it. Its own source is explicit that
on a kernel with no BPF LSM it **ends in failure rather than in success**, because
a process that exited cleanly would be telling a supervisor that a machine had
been given its boundary. So it cannot report success falsely, which is what makes
it the check:

```
systemctl status alo-boundaryd     # Type=oneshot, RemainAfterExit=yes
journalctl -u alo-boundaryd        # its reasons, in English
ls /sys/fs/bpf/alo                 # the pin the boundary hangs on
```

**Look for:** the unit succeeded, and the pin exists. If it did, the boundary
attached on this kernel and this promise is shown — by the machine doing it,
which is the only evidence the promise was ever about.

**If it failed**, the log says why in English, and *then* these three reads find
out which of the three requirements is missing:

```
zcat /proc/config.gz | grep -E '^CONFIG_(BPF_LSM|LSM)='   # how it was built
mount -t securityfs securityfs /sys/kernel/security       # if not already mounted
cat /sys/kernel/security/lsm                              # what actually started
```

**`zcat /proc/config.gz` may itself fail**, with *No such file or directory*: it
needs `CONFIG_IKCONFIG_PROC` compiled in, and nothing in this repository
establishes that the image's kernel has it. That is a gap in our knowledge rather
than a fault in the machine — write down which it was, and do not read it as an
answer about the BPF LSM.

**And `bpf` in that last line is not the whole question.** It shows the module
*started*; it does not show a programme can be *attached*. `hardware.md`'s third
requirement is that an attach hangs for ever, uninterruptibly and with no error,
on a kernel whose RCU-tasks grace periods have stalled — so a machine that passes
both reads can still be one where the boundary does nothing. That is exactly why
the daemon, which actually attaches, is the check and the reads are the
diagnosis.

Then, **on a machine that has been up more than a minute**:

```
dmesg | grep -c tasks_rcu_exit_srcu_stall                 # must be 0
```

Anything but zero and no BPF LSM programme will attach on that machine, however
the first two answered. Ask it late rather than at the login prompt: the stall
is reported about ten seconds after it begins, so an early zero is a question
asked too soon.

**If `bpf` is missing:** do not conclude the machine cannot do it. Ask what it
was booted with. A kernel that fails the second check was usually booted wrongly
rather than built wrongly. And if a boot parameter is used to fix it, `lsm=`
**replaces** the built-in list rather than adding to it — the parameter must
name every module the kernel already ran *and* `bpf`, or it silently stops the
ones that were enforcing. Read `/sys/kernel/security/lsm` before and after and
compare them.

### The pinned model answers on this CPU

**The model is on the machine and serving. What may be missing on the day is a
command of ours to ask it with.**

*Corrected 2026-09-28. This page said the check could not be performed because
nothing in the image asks a model. That was established by grepping `[[bin]]`
sections in `Cargo.toml` files rather than by asking what the image installs, and
it missed `alo-modeld` entirely.*

The image **does** ship the model: the runtime at `/usr/bin/ollama`, 2.23 GiB of
qwen3 weights under `/usr/share/alo/models/`, and `alo-modeld.service` running
`ollama serve`. ADR 0025 decided all of it, and that unit's own comment records
that until it existed nothing started either half — `alo-models` knocked at the
loopback address a runtime listens on and found nothing there, which is the same
answer a machine with no model at all gives.

**So first, ask the machine whether it is serving:**

```
systemctl status alo-modeld
```

What is narrower, and may still be owed on the day, is a command of **ours**.
`alo-models` is a library with no binary, so the only way to ask the model today
is to type the name of the thing we rented — and `docs/features.md` promises *a
person never learns the name of anything we rented*, which
`alo-modeld.service`'s own comment restates. A lane is adding that command; if it
has landed by 2 October this page carries it and the check is performable.

**If it has not landed:** write *the model is served, and no command of ours asks
it*. That is a different and much smaller finding than the one this page used to
record, and it leaves the GPU box open for one stated reason rather than two.

Do **not** substitute `ollama run`. What the roadmap promises is the machine doing
it; a model run by hand proves the CPU can do arithmetic, not that alo OS can ask
it anything — and typing the rented name is the thing the promise forbids.

### Boots on one certified machine, firmware to the daemon

This is the walk itself. It is shown if steps 1 to 6 completed and you reached
`alo-agentd`, and it is **not** shown if you helped the machine at any point in
a way the page does not describe. Write down any help you gave.

## The security chip, if this machine has one

Separate from the four, and only if `tpm.msc` found a chip. Which road the
machine takes at first boot depends on it:

```
systemd-cryptenroll --tpm2-device=list
```

**Look for:** a device named. A chip whose manufacturer is **IBM / SW** is a
simulator rather than a machine, and does not count.

The three promises only a chip can keep are not part of this walk — they need
their own run — but recording whether the chip is present and ready tells the
next person which of the two roads this machine is on.

## What is knowingly missing on the day

These are not failures of the walk, and the report should say so rather than
letting silence imply more:

- **no video plays** — the image carries no pipeline;
- **no update can be applied** — the image ships no signature policy;
- **the camera and microphone are refused**;
- **the three promises only a chip can keep are unmeasured**;
- **there is no sign-in screen** — it is the desktop lane's and is not in the
  image. *Firmware to sign-in* stays open.

The walk tests **the install and the first start**. That is all it claims.

## Writing down what you saw

Into [`autonomy/v0-01-evidence.md`](autonomy/v0-01-evidence.md), by hand, with:

- the date;
- the machine, by the name it was given in `hardware.md`;
- the Secure Boot state it was walked under;
- for each of the four checks, what was seen — or that it was not shown, and
  why.

The roadmap's on-the-machine boxes move **only** for what was seen. A box ticked
from a successful install that did not demonstrate the thing the box names is
the failure this whole document exists to avoid.
