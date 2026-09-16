# The Windows the walk is measured in

**Date:** 2026-09-16
**Workstream:** the installer plan's task 10 — *The installer, walked on a real
Windows in a virtual machine and killed at every step*
**Contributor:** Claude Code, lane B (`C:\dev\alo-os-shell`)
**Status: not done — the machine the walk needs does not yet hold a Windows,
and nothing was handed over.** No `.kernel-loop/handoff.toml` was written,
because the task is not finished and a handoff is a statement that it is. What
follows is what this session measured, so the next worker starts where this one
stopped rather than where the last one did.

## Why there is a report and no code

Task 10 asks for a test that installs a Windows into a virtual machine and then
walks `crates/alo-installer` to each of the seven steps in `staging.rs`, killing
it at every one. Everything in that sentence past *installs a Windows* waits on
a Windows being there, and this repository has never had one: the previous
worker on this task left 117 MB of a half-downloaded ISO under
`/root/alo-installer-vm/` and nothing else.

This session went after the first half — the machine, its media and its
Windows — and reached a Windows Setup that starts and then ignores the answer
file. That is genuine ground gained and it is written down below, but it is not
an acceptance criterion, so nothing is claimed as done and nothing was published.

## What was measured

### The Windows, and why it is Windows Server

There is no Windows a test may install unattended without a licence, with one
exception: Microsoft's evaluation editions, which download without an account,
without a key, and without a form. The client evaluation (Windows 11
Enterprise) is only reachable through a page that hands out a session-bound
link; the server evaluation is a plain redirect, and it is the one a test can
fetch on its own.

| | |
|---|---|
| Fetched from | `https://go.microsoft.com/fwlink/?linkid=2293312` |
| Which is | Windows Server 2025 evaluation, English (United States) |
| Size | 6 014 152 704 bytes |
| SHA-256 | `d0ef4502e350e3c6c53c15b1b3020d38a5ded011bf04998e950720ac8579b23d` |
| Fetched at | about 21 MB/s, four and a half minutes, 2026-09-16 |
| Kept at | `/root/alo-installer-vm/windows.iso` (root's WSL home, not the checkout) |

Its `sources/install.wim` carries four images, read out of the WIM's own XML
resource rather than guessed:

| Index | Name |
|---|---|
| 1 | Windows Server 2025 SERVERSTANDARDCORE |
| 2 | Windows Server 2025 SERVERSTANDARD |
| 3 | Windows Server 2025 SERVERDATACENTERCORE |
| 4 | Windows Server 2025 SERVERDATACENTER |

**Index 2 is the one to install**: Standard with the Desktop Experience, which
is the only pair of the four that has a desktop session for *shows Windows
starts to its desktop session* to mean anything. The Core images have no shell.

**What a server Windows does and does not show.** `Resize-Partition`,
`New-Partition`, `Format-Volume`, `Get-Disk`, `Get-Partition`,
`Confirm-SecureBootUEFI`, `Get-Tpm` and `bcdedit` are the same programs on
Server 2025 as on the client Windows 11 the certified laptop ships with — they
are the storage and boot stack, not the edition's shell — so the three things
task 10 exists to show (that the cmdlets do what `program.rs` asks, that
`naming.rs` makes the name the environment finds, and that a copy of
`{bootmgr}` is an entry the firmware starts) are all showable on it.
`Get-BitLockerVolume` is the one that is not installed by default on Server,
and the installer already has a decided answer for a check that cannot be
answered — *could not be found out*, never read as *off* — so a run there
refuses rather than proceeds, which is a refusal path worth walking on purpose.
**The client Windows stays the acceptance a person performs at the laptop**,
where ADR 0033 §5 already puts it; the virtual machine is where the code is
walked, not where the product is accepted. An evaluation Windows also expires
after 180 days, so the machine is a thing a test builds and throws away, never
a thing kept in the repository.

### The machine

QEMU q35 with OVMF, driven from the Linux the gates run in, the same shape
`crates/alo-installing/tests/installed_in_a_virtual_machine.rs` already uses,
because Hyper-V is still out of reach of the account the loop runs under (task
2's report). What it needs beyond that test's list is nothing new: KVM, OVMF,
`sfdisk`, `mtools` and about 60 GB. `xorriso`, `genisoimage` and `mkisofs` are
**not** installed and no package was installed to get them, which is why the
answer file travels on a disk rather than on a second CD-ROM.

Two SATA disks behind one AHCI controller, each with a serial QEMU is told to
report:

| Disk | Serial | What it is for |
|---|---|---|
| `ahci.0`, 48 GB | `ALOWINDOWS1` | the Windows the walk shrinks |
| `ahci.1`, 24 GB | `ALOTARGET1` | the empty disk alo OS would replace |

The serials are the point of the choice. Windows reports an AHCI disk with bus
`SATA`, model `QEMU HARDDISK` and that serial, and `naming.rs`'s SATA rule then
makes `ata-QEMU_HARDDISK_ALOTARGET1` — which is exactly the name udev gives the
same disk under `/dev/disk/by-id/`. So the SATA row of `naming.rs`'s table is
checkable from both sides in this machine without a physical disk, which is the
second of the three things task 10 exists for. NVMe and the Hyper-V SCSI names
are not: QEMU's NVMe device reports no NGUID or EUI-64 by default, and there is
no Hyper-V here at all. Those two rows stay unmeasured, and `naming.rs`'s
rustdoc already says so.

**Secure Boot is off in this machine**, and the OVMF build is the one without
Secure Boot — because the shipped installer refuses to run with Secure Boot on
(ADR 0033 §4) and this test runs the installer. That is the constraint the plan
already names, not a weakening of task 9's, which is a different machine
enforcing Secure Boot on a different boot chain.

### Three things about the firmware and Setup that cost this session its hour

1. **The firmware's boot prompt has to be typed through.** The Windows ISO's
   UEFI boot image is the one that says *press any key to boot from CD*, and
   OVMF times it out in about five seconds and falls through to PXE — the whole
   machine then sits at `>>Start PXE over IPv4` forever. A key has to be held
   down at the QEMU monitor from about one second in; sending one every second
   and a half starting at eight seconds is too late, and that is what two
   attempts here died of. `press.py` in `/root/alo-installer-vm/` sends
   `sendkey ret` every 250 ms for a given number of seconds and reports how many
   it sent, and a run that sent none is a run whose monitor socket was never
   there — which is a different failure wearing the same face.
2. **`qemu-system-x86_64` does not answer to its own name.** Linux truncates
   `comm` to fifteen characters, so `pkill -x qemu-system-x86_64` matches
   nothing and leaves a machine holding the write lock on `target.qcow2`; the
   next start then dies with *Failed to get "write" lock*. `pkill -x
   qemu-system-x86` works. `pkill -f` on any part of the command line is worse
   than useless here: it matches the shell that is running the `pkill`, which
   kills the session doing the killing.
3. **Windows Server 2025's Setup ignores an answer file on an attached disk,
   partitioned or not.** A 64 MB image formatted by `mformat` with no partition
   table, attached as `usb-storage`, got the new Setup as far as *Select
   language settings* and no further — it never read `autounattend.xml`. The
   image was then rebuilt as an MBR disk with one type `0c` partition at 1 MiB
   (`sfdisk`, then `mformat -i unattend.img@@1M`), which a Windows of any older
   generation would certainly have mounted, and **the second run stopped on the
   same screen**. Both runs are screen-dumped; the second is the measurement
   that matters, because it rules out the media's layout.

   What this leaves is Server 2025's *new* Setup, which is the 24H2 engine and
   does not look at removable media at the language stage the way the old one
   did. Two roads out, neither of them taken here: put `autounattend.xml` in
   the **ISO's own root**, which needs an ISO builder this machine does not
   have and no package was installed to get; or drive Setup's own escape —
   Shift+F10 opens a command prompt, and `X:\setup.exe /unattend:D:\autounattend.xml`
   starts it with the answer file by hand. The second is automatable exactly
   the way the boot prompt already is, through QEMU's monitor: `sendkey
   shift-f10`, then the command a key at a time. **That is the cheapest next
   step and it is where the next session should start.**

The answer file itself is written and is at `/root/alo-installer-vm/autounattend.xml`:
`windowsPE` wipes **disk 0 only** — never disk 1, because an answer file that
wiped the target disk would destroy the thing under test — lays out EFI, MSR and
an NTFS `C:`, installs index 2, and `oobeSystem` sets an administrator password,
signs in automatically, and at first sign-in copies `at-every-start.cmd` off the
answer disk and puts it in `HKLM\...\CurrentVersion\Run`. That script writes
*alo-walk: windows started to its desktop session* to `COM1`, which QEMU has
pointed at a file, and then runs `alo-walk.cmd` from whichever drive carries
one. That is the whole of how the walk will drive the guest and how it will see
that the guest reached a desktop: the console file, not a screenshot.

## What the next session does, in order

1. Bring the install to an installed Windows: start it with `install.sh`, hold
   the key with `press.py 25`, then send `shift-f10` and type
   `X:\setup.exe /unattend:D:\autounattend.xml` through the monitor. Read the
   screen with the monitor's `screendump` — a PPM, which twelve lines of Python
   turn into a PNG (`/root/shot.sh`) — when the console file stops telling you
   anything. The console carries only the firmware; Windows Setup itself says
   nothing there.
2. Shut it down and keep `windows.qcow2` as the pristine base. **Every walk
   runs on a `qemu-img create -b` overlay of it**, never on the base: that is
   what makes eight kills cost eight boots instead of eight installs, and it is
   what makes *the Windows partition byte-for-byte what it was* a comparison
   against a file that cannot have changed.
3. Then task 10 proper, and it wants splitting when it is written: the harness
   (needs, media, machine, console) is one file, the walk's seven kill points
   are another, and the kill points should be **derived from the sentences
   `staging.rs` says** rather than listed by hand, so a step added to the
   staging without a kill point fails a test instead of quietly going unwalked.

## Limitations

- Nothing was walked, nothing was killed, and no Windows partition was hashed.
  Every acceptance criterion of task 10 is open.
- No Rust was written this session, so there is nothing to gate and nothing to
  publish beyond this report and the plan's status.
- The ISO, the images and the scripts live under `/root/alo-installer-vm/` in
  the WSL root's home. None of it is in the checkout and none of it should be:
  it is 6 GB of somebody else's Windows.

## Proposed shared-document changes

None. No code changed, so there is nothing for `CHANGELOG.md`; `ROADMAP.md`'s
installer line is untouched; `docs/autonomy/QUEUE.md` and `STATE.md` should
record that task 10 was attempted and is still open, with this report as the
reason it is cheaper the next time.
