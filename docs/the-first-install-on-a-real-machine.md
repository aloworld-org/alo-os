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

Run the installer on Windows. It reads the machine before it offers anything:
the disks, the firmware, Secure Boot, and Windows' own Fast Startup setting.

**Look for, and write down:**

- **the disk it names**, and whether it is the one you expected;
- **the size it says it needs** — 24 GB is the floor, and a machine with no
  empty disk that size is refused in words with nothing changed;
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
panel's own resolution, or a fallback? The rest depend on the check below,
which cannot be performed — see the note there.

### The boundary attaches on this kernel

Three questions in this order, and the third is the one that gets missed:

```
zcat /proc/config.gz | grep -E '^CONFIG_(BPF_LSM|LSM)='   # how it was built
mount -t securityfs securityfs /sys/kernel/security       # if not already mounted
cat /sys/kernel/security/lsm                              # what actually started
```

**Look for:** `bpf` in the output of the last line. `CONFIG_BPF_LSM=y` on its
own is a true answer to the wrong question — a kernel can have the BPF LSM
compiled in and never start it.

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

**This check cannot be performed on 2 October, and that is a finding rather than
a step to improvise.**

Nothing in the image asks a model. The image ships `alo-agentd`, `alo-boundaryd`,
`alo-brokerd`, `alo-desktop`, `alo-compositor` and the installers; there is no
model binary and no command that pulls or runs a catalogue entry.
`hardware.md`'s definition of *the GPU works on first boot* has "pulling and
running a model from the catalogue is one command" as one of its four parts, and
that command does not exist yet.

**So on the day:** write *not shown — no command in the image asks a model*, and
leave both this box and the GPU box open. Do not substitute a hand-run Ollama:
what the roadmap promises is the machine doing it, and a model run by hand
proves the CPU can do arithmetic, not that alo OS can ask it anything.

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
