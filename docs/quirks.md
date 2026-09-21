# Quirks

Where reality and the specification disagree.

An operating system meets three kinds of reality that no document describes
correctly: hardware and firmware, applications being driven through their own
automation, and pinned upstream engines behaving unlike their manuals. When you
lose an afternoon to one of them, write it down here. The next person should
inherit the knowledge, not the debugging session.

## How to write an entry

One entry per quirk, newest first within each section. Every entry says: **what
it is, what version, what actually happens, what we do about it, and the date.**
A quirk with no version and no date is a rumour.

Keep the accommodation and the reason together. Six months from now the code
will look wrong to somebody, and this file is the only thing standing between
them and reintroducing the bug.

The rule this file serves: **strict in what we do, tolerant in what we accept.**
We behave correctly; we cope with hardware and applications that do not.

---

## DRM mapping cleanup is an upstream panic boundary (2026-09-07)

Pinned drm-rs 0.14.1 `map_dumb_buffer` issues MAP_DUMB and a shared read/write
mmap using its private allocation length. The shell now checks the returned
slice covers pitch times height before clearing all bytes, including padding
and allocation tail, and drops the mapping before framebuffer registration.
MAP_DUMB/mmap errors are returned and unwind buffer ownership. `DumbMapping::drop`
calls munmap with `expect`, so an unmap failure can panic rather than become a
ResourceError. The wrapper also constructs its slice before the shell can inspect
the hidden capacity. These source-inspected limits are not malformed-kernel
containment or recovered unmap failures. No engine patch or unsafe repository
code was added. Successful DRM mapping/unmapping and production recovery remain
unmeasured; fake memory tests and actual MAP_DUMB ENOTTY only prove their stated
paths. See `autonomy/updates/scanout-buffer-initialization.md`.

## DRM allocation wrappers assume valid kernel handles (2026-09-07)

Pinned drm-rs 0.14.1 `create_dumb_buffer` and `add_framebuffer` unwrap the
conversion of returned kernel IDs to nonzero handles. `DumbBuffer` exposes
size/format/pitch but keeps the allocation length private. The shell validates
exposed metadata before registration. The subsequent initialization component
now maps and checks the returned slice (see above); it cannot validate the
hidden capacity before upstream constructs that slice or contain a zero-handle panic
inside the wrapper. Mode is a transparent wrapper over the kernel timing struct
and is passed intact to upstream blob creation. These are source-inspected
limits, not reproduced malformed-kernel failures. No engine patch or unsafe
repository code was added. Explicit resource release reports every failed ioctl;
Drop can only attempt best-effort cleanup. A cleanup failure requires retiring
the device, with final file-description close as the remaining cleanup boundary.
Real allocation/destruction still needs a DRM-equipped environment. See
`autonomy/updates/direct-display-resource-ownership.md`.

## Atomic DRM property parsing trusts raw kernel metadata (2026-09-07)

Direct-display schema discovery rejects missing/duplicate properties, wrong
types, flags and unusable rectangle ranges after parsing. Pinned drm-rs 0.14.1
`control::Device::get_property` indexes raw range/object metadata and uses
C-string conversion internally. Invalid kernel response lengths/termination may
panic or violate upstream assumptions before our schema checks run. This is
source-inspected, not a reproduced kernel failure. Our code avoids its UTF-8
hashmap unwrap and enum-index helper, but does not claim to contain all malformed
kernel responses. No engine patch or lint exemption was introduced. Successful
DRM-device discovery and production failure recovery still need device evidence.
See `autonomy/updates/atomic-display-property-discovery.md`.

## Smithay libseat notification ordering and failure limits (2026-09-07)

Smithay 0.7.0 `backend/session/libseat.rs` forwards libseat callbacks through a
calloop channel. The seat fd's dispatch can queue notifications for a subsequent
readiness pass. `DirectSession` performs two nonblocking passes before lending
its discovery descriptor; a real calloop forwarding test verifies pause/activate
ordering and preserved cleanup error. Device access also checks backend activity.
Kernel revocation during an ioctl still requires ordinary error handling.

Source inspection also shows `disable()` acknowledged before the shell receives
`PauseSession`, and internal unwraps in initial/ongoing dispatch, disable and
notifier registration. These failure paths were not induced on a real seat here;
the ENOENT connection refusal is the only real libseat diagnostic measured.
Our discovery-stage owner closes descriptors on pause and reports errors the
backend returns. It cannot promise orderly renderer teardown before upstream's
disable acknowledgement, or recover from every internal backend panic. Before
production scanout, integrate an unpatched interface supporting the required
ordering and fallible handling; an engine source patch requires an ADR. No patch
or panic-catching workaround is included. Smithay also ignores requested device
open flags in this backend; libseat owns device-open semantics.

## Smithay nested pointer leave notifications (2026-09-07)

Smithay 0.7.0's `backend/winit/mod.rs` consumes Winit `CursorEntered` and
`CursorLeft` without forwarding them; `WinitEvent` exposes focus, input, resize,
close and redraw only. The nested shell can cancel input on deactivation/close,
but cannot observe a pointer leaving an otherwise active parent window. Do not
mistake focus-loss tests for leave-only acceptance. The next backend component
must expose that lifetime through an owned event loop or another unpatched
upstream interface before this backend can claim full pointer support. Current
development input remains explicitly limited; no pinned engine patch is made.

## Smithay final drag release retains the old pointer focus (2026-09-07)

Smithay 0.7.0's implicit pointer grab ends when its final button is released,
but the protocol handle still reports the drag recipient until another motion.
The real-client pointer-release popup refusal test exposed this: drag outside,
release, return inside, then request a popup with that release serial. Merely
checking the handle's focus allowed the menu. `alo-shell` now re-hits the scene
at final release before retaining popup authority. It still delivers the release
to the original drag recipient, and does not patch Smithay. The unchanged refusal
test passes; this is Unix-socket development evidence, not physical input testing.

## Hardware and firmware

### EDK II's strict image protection page-faults the base's signed loader, and Fedora's own firmware fixes it up
**Version:** the boot chain `quay.io/fedora/fedora-bootc:42@sha256:077182b6…`
ships — `shim-x64-15.8-3.x86_64` and `grub2-efi-x64-2.12-32.fc42.x86_64` — under
QEMU 10.2.1 (`1:10.2.1+ds-1ubuntu3.2`), KVM, `-machine q35,smm=on` with
`-global driver=cfi.pflash01,property=secure,value=on`, in WSL Ubuntu on Windows
11 Pro 10.0.26200. Two firmwares: **Ubuntu `ovmf` 2025.11-3ubuntu7**
(`/usr/share/OVMF/OVMF_CODE_4M.secboot.fd`) and **Fedora
`edk2-ovmf-20250812-21.fc42`** (`/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd`).
2026-09-16.
**Behaviour:** started by Ubuntu's Secure Boot build, the staged loader dies
before Linux and the machine hangs:

```
BdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
!!!! X64 Exception Type - 0E(#PF - Page-Fault) ... W:1 P:1 ...
RIP - 0000000070AB1CA0 ... RDI - 0000000070ACF000, RDX - 0000000070B19000
!!!! Find image based on IP(0x70AB1CA0) (No PDB) !!!!
```

A **write** (`W:1`) to a **present** (`P:1`) page. Located by elimination, each
run booting the same staged partition:

| what was started | firmware | machine | result |
|---|---|---|---|
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd` | `q35` | boots to `alo-installing` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd` | `q35,smm=on`, secure flash | boots to `alo-installing` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35,smm=on`, secure flash | `#PF` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35` | `#PF` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.fd` (**no keys, Secure Boot off**) | `q35,smm=on`, secure flash | `#PF` |
| **shim alone** (`grubx64.efi` removed) | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35,smm=on`, secure flash | *Failed to open \\EFI\\BOOT\\grubx64.efi — Not Found*, **no fault** |
| **the loader alone** (`grubx64.efi` as `BOOTX64.EFI`) | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.fd` | `q35,smm=on`, secure flash | `#PF`, byte for byte the same registers |
| shim → loader → kernel | **Fedora `OVMF_CODE.secboot.fd` + `OVMF_VARS.secboot.fd`** | `q35,smm=on`, secure flash | boots, Secure Boot **enabled** |

So it is neither Secure Boot itself (row 5 faults with no keys enrolled), nor
the machine's SMM (rows 3 and 4 fault alike, rows 1 and 2 boot alike), nor shim
(row 6), nor alo OS: it is **the loader, and the firmware build's memory
protection**. With the image base read back from the registers, the loader is
writing from its own `.data` into its `mods` section — pages the stricter build
has marked read-only.

Fedora's firmware says what it is, on the same partition, and carries on:

```
PageFaultExitBoot: Page fault fixups needed (NX: 0, RW: 1).
PageFaultExitBoot: The guest OS boot chain is not NX clean.
PageFaultExitBoot: Applying global page table fixup (shim is older than v16).
[    0.000000] secureboot: Secure boot enabled
```

The remedy the message names is **shim 16**, which negotiates memory protection
with the firmware; Fedora 42 ships shim 15.8, and building our own shim or
loader is forbidden (ADR 0011).
**Our response:** the virtual machine the installer is measured in takes its
Secure Boot firmware **out of the pinned base itself** (`dnf install edk2-ovmf`
in a container of it), not from whatever the host packages — the firmware built
for the boot chain the base ships. Secure Boot stays on and is never switched off
to make a test pass (ADR 0033 §4); that the firmware really enforces it is
measured by `a_loader_the_firmware_does_not_trust_is_refused`, which changes one
byte of the staged loader and shows the firmware answering *Access Denied --
rejected probably by Secure Boot* and starting nothing. **What only the certified
laptop can answer** is whether its own firmware is as strict as Ubuntu's build;
that is task 6 of the installer plan, at the machine, and if it is, the way
through is a base whose shim is 16 or newer, never a patched loader and never
Secure Boot off.
**Date:** 2026-09-16.

### OVMF without SMM saves its variables onto a FAT disk when its flash is SMM-only
**Version:** OVMF 2025.11-3ubuntu7 (`/usr/share/OVMF/OVMF_CODE_4M.fd`, the build
without Secure Boot and without SMM) with `OVMF_VARS_4M.fd`, under QEMU 10.2.1
(`1:10.2.1+ds-1ubuntu3.2`), KVM, in WSL Ubuntu on Windows 11 Pro 10.0.26200.
2026-09-15.
**Behaviour:** `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`
started the boot environment's refusal with that firmware on the machine the
Secure Boot test uses — `-machine q35,smm=on` and
`-global driver=cfi.pflash01,property=secure,value=on`, which lets only code
running in System Management Mode write the variable flash. This build has no
SMM code, so none of its writes to the flash take, and it falls back to keeping
its variables in a file, `NvVars`, in the root of the first FAT file system it
finds. On the test's first disk that is the staged `ALO-INSTALL` partition. The
refusal said *so nothing was changed*, correctly about alo OS, and the first
disk changed at three mebibytes of that partition: its boot sector and FATs, and
the cluster holding `NvVars`.

Measured with the firmware alone, no kernel, 45 seconds, on a 700 MiB disk with a
100 MiB data partition and a 590 MiB `ALO-INSTALL` FAT holding one file:

| firmware | machine | disk bytes changed | variable flash bytes changed | root of the FAT |
|---|---|---|---|---|
| `OVMF_CODE_4M.fd` | `q35` | 0 | 6176 | `EFI` |
| `OVMF_CODE_4M.fd` | `q35,smm=on`, secure flash | 967 (MiB 101: 10, MiB 102: 957) | **0** | `EFI`, `NvVars` (1523 bytes) |
| `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35,smm=on`, secure flash | 0 | — | `EFI` |

The second row is the whole of it: the flash refused every write, and the disk
received them. Neither alo OS nor a real laptop is involved — a laptop's firmware
has a variable store it can write, and the same firmware on a machine that lets
it write its flash (row 1) or the SMM build on the SMM machine (row 3) writes
nothing to any disk.
**Our response:** the test names a firmware and its machine as one value
(`Firmware` in that file): SMM and SMM-only flash for the Secure Boot build, a
plain `q35` for the build without it. The refusal test asserts, besides the
whole first disk being byte-for-byte unchanged, that the firmware's own variable
flash *did* change — so a machine that can no longer write its flash fails as
that, rather than as a write to a disk. `a_firmware_is_given_only_flash_it_can_write`
holds the pairing in the suite. Anyone starting OVMF by hand: never give
`OVMF_CODE_4M.fd` (or any build without `SMM` in its description) flash with
`secure=on`.
**Date:** 2026-09-15.

### Windows names an NVMe disk by its bus, and puts an identifier where the serial number is
**Version:** Windows 11 Pro 10.0.26200, `Get-Disk` from Windows PowerShell 5.1, on
the development machine (a Dell with an SK hynix PVC10 512 GB NVMe disk), read by
`crates/alo-installer/tests/reading_this_windows.rs` on 2026-09-15.
**Behaviour:** the installer has to tell the boot environment which disk to write
by the name Linux gives it under `/dev/disk/by-id/`, and it only has what Windows
reports. For this disk Windows reports `FriendlyName` as `NVMe PVC10 SK hynix
512GB` — its own bus name in front of the model — `SerialNumber` as
`FD5B_42CE_BC8F_9D54_ACE4_2E00_5113_F94B.`, which is not a serial number but a
128-bit identifier in groups of four ending in a full stop, and `UniqueId` as
`eui.ACE42E005113F94B`. A name made from the model and serial the way udev makes
`nvme-<model>_<serial>` would name no disk.
**Our response:** `crates/alo-installer/src/naming.rs` names an NVMe disk only by
its identifier, `nvme-eui.` and the grouped identifier joined and lower-cased (or
the `eui.` unique identifier when there are no groups), and never by its model.
Every name carries an identifier, so a wrong one names no disk and the environment
refuses with *not connected, so nothing was changed*. **Not yet seen from the Linux
side**: which of the two identifiers the kernel's `wwid` is for this disk (it
prefers an NGUID to an EUI-64) is unmeasured until the environment runs on a
machine with an NVMe disk.
**Date:** 2026-09-15

### `Get-Tpm` without an administrator answers with a sentence, not an error
**Version:** Windows 11 Pro 10.0.26200, Windows PowerShell 5.1, unelevated, on the
development machine, 2026-09-15.
**Behaviour:** `Get-Tpm` with `$ErrorActionPreference = 'Stop'` does not throw. It
returns the string *Administrator privilege is required to execute this command.*
as its output object, so `$tpm.TpmPresent` is `$null` and `[bool]` of it is
`false` — the first run of the installer's checks on this machine said *this
computer has no security chip (TPM)* about a computer whose TPM it had not been
allowed to ask about. `Get-PartitionSupportedSize` (*Access to a CIM resource was
not available*), `Get-BitLockerVolume` and `Confirm-SecureBootUEFI` do throw.
**Our response:** the installer's TPM script throws unless the answer has a
`TpmPresent` property, so an unelevated answer is *could not be found out*; the
installer also asks for an administrator's rights before any check, and
`reading_this_windows.rs` holds every one of these reads to *not known* when
unelevated.
**Date:** 2026-09-15

### A machine without a boundary runs no turn, and a development machine is no exception
**Version:** `alo-agentd` and `alo-bounding` from 2026-09-12, measured on
`6.18.33.2-microsoft-standard-WSL2` by
`crates/alo-bounding/tests/a_turn_without_a_boundary_does_not_run.rs`.
**Behaviour:** the boundary is twenty-three pinned links and two pinned maps under
`/sys/fs/bpf/alo`, made once at boot by `alo-boundaryd` (ADR 0018), and
`alo-agentd` opens the one map it may write. Three things can happen to that
arrangement under a running service, and until 2026-09-12 the service noticed
none of them:

- **a pin is removed.** Removing a link's pin is the one thing on the machine
  that detaches its hook. A service that had opened the map went on writing
  entries into it and running turns, and the kernel decided at one hook fewer
  than it had — a turn could still, say, send where it could not open.
- **the map's pin is removed, or the whole directory is taken away.** The
  service holds a descriptor, so its writes still land somewhere; whether any
  programme still reads that somewhere is the next point.
- **the loader is run again** — `pinned.taken_away()` and a fresh
  `Imposed::once`, which is what an operator restarting `alo-boundaryd` by hand
  does. Every pin is new. The programme now on the hooks reads a map the
  service never opened, and what the service writes into the map it holds is
  read by nothing. **Measured:** a turn under that arrangement ran, and opened
  a private key beside the one file it was granted — `Went { control: Opened,
  granted: Opened }` — on a machine whose pin listing showed nothing wrong and
  whose record said the turn was bounded.

Two smaller things came out of measuring it. **Detaching is asynchronous**:
the kernel releases a pinned link from a work queue, so for a moment after
`rm /sys/fs/bpf/alo/file_open` the old programme is still refusing, and a
test that removed a pin and opened a file in the same breath saw `EACCES`
where a second later it would have seen the file. What is stable is the pin's
absence, and that is what the service asks about. And **the daemon may see a
pin and may not open one**: the pins are `0600 root:root` in a `0750`
directory the agent's group may enter, so a `stat` from the service answers
and an open does not — which is right, because a descriptor on a link is
enough to detach it (`BPF_LINK_DETACH` checks nothing about how the descriptor
was opened), and a mode that let the service read a pin would let the person's
own daemon take the machine's boundary off.
**Our response:** the service asks the machine before every turn, and at
start — `alo_bounding::Boundary::in_place`, called first thing in
`Turns::doing`: is the map of turns still pinned, is every one of the twenty-three
hooks still held, and is the map at the pin the map this service holds, as the
kernel numbers its maps. Any *no* refuses the turn before its first verb, with
nothing made and nothing to undo; the refusal is written down in the record as
the machine's own (`not-bounded`, `docs/contracts/record-file.md`), the person
reads one sentence saying the machine and not they are at fault, and the
service log carries the machine's account — which pin, which two map numbers.
A machine with its boundary in place is unaffected, and the same file measures
that beside every refusal.

**There is no override, and there is not going to be one.** `docs/features.md`
promises *a refusal, not a warning*, and an environment variable that let a
turn run unbounded on a development machine would be the warning with a name;
`nothing_in_this_crate_reads_the_environment` in the same test file reads
`alo-bounding`'s source and fails the day one appears. What a development
machine gets instead is the same refusal and this entry. To find out which of
the three states it is in:

```
ls -l /sys/fs/bpf/alo                    # twenty-three links, bounds, fields — all present?
systemctl status alo-boundaryd           # did the loader run, and once?
journalctl -u alo-agentd | grep boundary # which pin, or which two map numbers
```

A missing directory or map is a loader that never ran, or one whose work was
taken away: `docs/hardware.md`'s five checks say why a loader refuses. A
missing link is a pin somebody removed; the remedy is the loader's pins made
afresh **and the service restarted**, because of the third state. Two map
numbers in the log is the third state exactly: the loader was run again, and
`alo-agentd` has to be restarted so that it opens the map the programme now
reads. Nothing here is repaired by the service on its own, because a service
that re-opened a boundary while running would be one that decided for itself
which boundary it was under.
**Date:** 2026-09-12

### Hyper-V refuses to start a machine rather than start it small, and the figure it refuses at is the host's, not the guest's
**Version:** Hyper-V on Windows 11 Pro 10.0.26200, a 15.5 GB host, measured
2026-09-11 starting the `alo-os` generation-2 machine `docs/booting.md`
describes.
**Behaviour:** with a browser, an editor, WSL and two build loops holding the
host's memory — about 1.1 GB *available* — `Start-VM` refused **4096 MB, then
1024 MB, then 768 MB** in turn: *Not enough memory in the system to start the
virtual machine*. It reserves overhead beyond the machine's own allocation and
refuses rather than starting with less. Nothing in that sentence is about the
disk. Once the browser was closed (2 GB available), the same 768 MB startup
with dynamic memory — 512 MB floor, 2 GB ceiling — booted to the text console,
and the guest's heartbeat came up at 92 seconds.
**What we do:** `docs/booting.md` says 8192 MB *if the host has it to give*
and says what refusal looks like. A person who cannot start the machine closes
what holds the memory; nothing in the image can be changed to help, because the
refusal happens before the image runs.

### Fedora's shim passes Hyper-V Secure Boot under the Microsoft UEFI Certificate Authority template
**Version:** the image built from `quay.io/fedora/fedora-bootc:42` at the
digest `image/Containerfile` pins, on Hyper-V (Windows 11 Pro 10.0.26200),
2026-09-11.
**Behaviour:** a generation-2 machine with Secure Boot **on** and the template
set to *Microsoft UEFI Certificate Authority* boots the disk; the default
template (*Microsoft Windows*) trusts only Windows' own signer. This was the one
step in `docs/booting.md` written before anybody had watched it, and it holds.
**What we do:** the document tells a person to pick that template rather than
to turn Secure Boot off. Turning it off would have been the softer test, and a
certified laptop ships with it on.

### `struct file`'s `f_path` is inside an anonymous union, and a search over named members does not find it
**Version:** `6.18.33.2-microsoft-standard-WSL2`, measured 2026-09-04 by reading
`/sys/kernel/btf/vmlinux` on the machine the boundary would not load on.
**Behaviour:** the kernel's type information describes `struct file` with
nineteen members, and three of them **have no name**. `f_path` is not one of the
nineteen — it is a member of the unnamed union that is, sixty-four bytes in,
sharing its bytes with a second member called `__f_path`:

```
struct file  vlen 19  size 184
  ...
  f_cred     at 48
  f_owner    at 56
  <unnamed>  at 64   union { struct path f_path; ... __f_path; }
  <unnamed>  at 80   union { struct mutex f_pos_lock; u64 f_pipe; }
  f_pos      at 112
```

This is ordinary C — an anonymous struct or union's members belong to the
structure around it — and the format keeps the source's shape rather than
flattening it. What makes it a trap is the failure: `alo-bounding`'s reader
searched the named members only, found nothing, and refused to impose the
boundary with *this kernel has no `file.f_path`, so the boundary has nowhere to
look*. **That sentence is a true statement about the search and a false one about
the kernel**, and it points whoever reads it at their machine rather than at our
code. Asked directly, the same BTF is 6,677,359 bytes and contains `f_path`,
`f_inode`, `dentry`, `d_name` and `mnt_root`; `bpf_lsm_file_open` is in
`kallsyms`. Everything the message doubted was there.

Kernel 6.6 kept `f_path` as a plain member, so this appeared as a kernel upgrade
breaking a boundary that had never run.
**Our response:** `crates/alo-bounding/src/btf.rs` implements the rule rather
than the case — a member with no name is walked into, and what is found inside
it comes back at the outer member's offset plus its own. The descent is bounded
by the same `PATIENCE` the width lookup uses, because this file is read from
`/sys` rather than written by us, and it is never itself an answer: asking for
`""` finds nothing. The fixture in `testing.rs` now keeps `f_path` where 6.18
keeps it, so every test of the reader is a test of the walk into it, and a fourth
fixture whose anonymous member leads back to the structure it is in asserts the
bound. Nothing was special-cased for `file` or for a version.
**Date:** 2026-09-04

### The BPF LSM is compiled into the WSL2 kernel and does not start
**Version:** `6.6.87.2-microsoft-standard-WSL2`, Ubuntu under WSL2 on Windows 11,
measured 2026-09-03 by reading the kernel's own config and its own list of
running security modules.
**Behaviour:** the kernel has `CONFIG_BPF_LSM=y`, so every account of it that
stops there says the BPF LSM is available. It is not. `CONFIG_LSM` is
`"landlock,lockdown,yama,loadpin,safesetid,integrity,selinux,apparmor,tomoyo"`
with no `bpf` in it, `/proc/cmdline` carries no `lsm=` parameter to replace that
list, and the kernel's own answer — `/sys/kernel/security/lsm` — is
`capability,landlock,yama,safesetid,selinux`. A security module that is not in
that list never registered its hooks, so nothing it would have decided is asked
of it. **Compiled in and started are two different questions**, and only the
second one matters.

Two smaller things go with it, both of which cost time before the answer
appeared. `securityfs` is **not mounted** on this kernel, so
`/sys/kernel/security/lsm` reads as a missing file rather than as an answer until
`mount -t securityfs securityfs /sys/kernel/security` is run — an empty result
here means *you have not asked yet*, not *no modules*. And `bpftool` is not
installed in the Ubuntu image, so `bpftool feature probe` returns nothing at all
and exits `0`, which reads exactly like a clean probe that found no LSM support.
Neither absence is an answer; both look like one.
**Our response:** the measurement is the finding, and no work around it was
attempted. `docs/autonomy/QUEUE.md` items 26 and 27 — the whole of ADR 0015 —
are blocked on a kernel that starts the BPF LSM, and `docs/hardware.md` now
states the requirement as two checks in order rather than one, because checking
only `CONFIG_BPF_LSM` is how this kernel passes. WSL2 can be given
`lsm=…,bpf` through `kernelCommandLine` in a `.wslconfig`, and the build loop
deliberately did not: that is a change to somebody's own machine, made outside
this repository, that restarts every distribution running on it, and it is the
machine owner's to make rather than a loop's.

**Resolved on this machine, 2026-09-04, by the owner making that change.**
`kernelCommandLine = lsm=capability,landlock,yama,safesetid,selinux,bpf` in
`.wslconfig`, then `wsl --shutdown`. The kernel now answers
`capability,landlock,yama,safesetid,selinux,bpf`, and that survives a cold boot.
Three things came out of doing it that the first measurement could not show:

- **`lsm=` replaces the built-in list, it does not add to it.** `CONFIG_LSM`
  still reads `"landlock,lockdown,yama,loadpin,safesetid,integrity,selinux,apparmor,tomoyo"`
  and is simply not what this kernel used. Every module that must keep enforcing
  has to be named again on that line: `lsm=bpf` alone would have started the BPF
  LSM and silently stopped the five that were already running. A boot parameter
  that turns a protection on can turn four others off in the same breath.
- **`securityfs` is not mounted at boot, and `systemd=true` does not mount it.**
  This distribution has systemd enabled and the mount was still absent, so it
  went into `/etc/fstab`, which WSL does process. Without that step the fix looks
  like it failed, because the file that would report success is the one missing.
- **The remedy `docs/hardware.md` predicted is now measured rather than
  supposed** — a boot parameter, on the same kernel, with no kernel of our own.

The certified machine inherits the requirement and not this workaround: what it
needs is a kernel that boots with `bpf` in its active list, however its image
arranges that.
**Date:** 2026-09-03; resolved 2026-09-04

### The kernel starts the BPF LSM and still cannot attach a program to it
**Version:** `6.6.87.2-microsoft-standard-WSL2`, Ubuntu under WSL2 on Windows 11,
measured 2026-09-04 by attaching the programme in `crates/alo-bounding-kernel`
and then reading `dmesg`.
**Behaviour:** the two checks in `docs/hardware.md` both pass — `CONFIG_BPF_LSM=y`
and `bpf` in `/sys/kernel/security/lsm` — and the attach never returns. The
thread goes into **uninterruptible sleep in `bpf_trampoline_get` and stays
there**: it cannot be killed, `SIGKILL` leaves the process a zombie with that
thread still in the kernel, and every later BPF attach on the machine blocks
behind the same mutex. Nothing in userspace reports anything; there is no error
because there is no return.

The kernel says why, in its own log, every ten seconds:

```
tasks_rcu_exit_srcu_stall: rcu_tasks grace period number 13 (since boot)
  gp_state: RTGS_POST_SCAN_TASKLIST is 634853 jiffies old.
Please check any exiting tasks stuck between calls to
  exit_tasks_rcu_start() and exit_tasks_rcu_finish()
```

Attaching a BPF LSM programme builds a trampoline, and that waits on an
RCU-tasks grace period. On this machine grace period 13 has never completed: at
the first stall message it was already 634853 jiffies old, which at
`CONFIG_HZ=250` is about forty-two minutes, on a machine that had been up for
forty-three. **The grace period stalled about a minute after boot and more than
an hour before any of this repository's code ran**, so the boundary did not
cause it and cannot avoid it — a `synchronize_rcu_tasks()` that will never
return is a `synchronize_rcu_tasks()` that will never return.
**Our response:** the measurement is the finding, and it is a **third**
requirement that neither ADR 0015 nor `docs/hardware.md` had: a kernel whose
RCU-tasks grace periods complete. It is the same shape as the two before it —
`CONFIG_BPF_LSM=y` is true and useless on a kernel that does not start the
module; a started module is true and useless on a kernel that cannot attach to
it — and it is worse than both, because the failure is a hang rather than an
answer. The check is one line and belongs before any attach:
`dmesg | grep -c tasks_rcu_exit_srcu_stall`, where anything but zero means no
BPF LSM, fentry or fexit programme will attach on that machine until it is
rebooted.

`crates/alo-bounding/tests/the_kernel_refuses.rs` was written, is correct, and
**has never run**: on this machine it hangs at the first attach, and no claim
about the kernel refusing anything is made anywhere in this repository. Queue
item 26 is not ticked.

**Reboot tested, and it reproduces — this kernel is out.** The machine was
restarted and measured again the same day. The stall is **deterministic, not bad
luck**: same grace period number (13), first stall message at **31 seconds** of
uptime with the period already 2507 jiffies (10 seconds at `CONFIG_HZ=250`) old,
so it stalls about **21 seconds after boot**, before anything of ours can run.
The attach was attempted again under a deadline and hung again, leaving the same
unkillable remnant. Two boots, same result on that kernel version.

**Fixed by a kernel upgrade the same day, and the scope of the finding was
wrong.** `wsl --update` took WSL from 2.6.1.0 to 2.7.12.0 and the kernel from
`6.6.87.2` to `6.18.33.2`. On the new kernel the stall count is **zero** past the
blind window, and the attach that had hung twice now returns in **0.08 seconds**.
So the sentence this entry originally carried — *WSL2 cannot host a BPF LSM* —
was **too broad by one word**: it was true of that kernel and false of WSL2. A
finding measured twice on one version is still a finding about that version, and
naming the platform instead of the build is how a temporary fact becomes a
permanent belief. The requirement in `docs/hardware.md` is unchanged and is what
should be quoted; this entry is the worked example of a kernel that failed it.

The `.wslconfig` `lsm=` line and the `securityfs` entry in `/etc/fstab` both
survived the upgrade, and the new kernel starts `bpf` — with `ima` alongside it,
which the old one did not.
**Superseded:** 6.18.33.2 passes all three checks.

One correction worth keeping, because it nearly became a rule. The entry above
reads as though the first stall message arrives about forty minutes in, which
would give the `dmesg` check a forty-minute blind window and make a zero
meaningless on a fresh machine. That is wrong: messages repeat every ten seconds
from the moment of the stall, and the forty-three-minute figure was simply the
**oldest message still in the ring buffer** when it was read, not the first one
emitted. The real blind window is about **thirty seconds**. So the one-line check
in `docs/hardware.md` is sound, with one qualification: *ask it on a machine that
has been up for more than a minute.*
**Date:** 2026-09-04, reboot-tested the same day

### `bpf_get_current_cgroup_id` answers with a *threaded* cgroup, and nothing says it will
**Version:** `6.18.33.2-microsoft-standard-WSL2`, measured 2026-09-04 by
`crates/alo-bounding/tests/a_turn_is_this_thread.rs`.
**Behaviour:** the helper's documentation says it returns "the cgroup id of the
current task", and the whole of the boundary in `crates/alo-bounding` rests on
which cgroup that is when a process's threads are in different ones. cgroup v2
allows that inside a *threaded subtree*: `cgroup.procs` moves a whole process,
`cgroup.threads` moves one task, and a process can have one thread in
`…/turn-1` while its siblings are in `…/home`.

The helper reads the task's own default cgroup, so for a thread in a threaded
cgroup it answers with **that thread's** cgroup rather than with the resource
domain above it or with the thread group leader's. That is what makes a turn a
thread rather than a process, and it is the difference between a boundary that
covers one enumerated verb and one that also covers the record being written
about it.

It is a fact about the implementation and not a documented promise, in exactly
the way `bpf_get_current_cgroup_id` answering with the directory's inode number
already was (`cgroup.rs`).
**Our response:** measured rather than assumed, and the measurement is a test
that fails loudly if it stops being true — `the_other_threads_of_this_service_
are_not_in_the_turn` asserts both halves at once, that the working thread is
refused a file and that a sibling thread of the same process opens it at the same
moment. If a kernel ever answered with the resource domain instead, the sibling
would be refused too and that test is what would say so. Nothing is
special-cased for a version.
**Date:** 2026-09-04

### Writing `0` into `cgroup.procs` or `cgroup.threads` means *whoever is asking*
**Version:** cgroup v2, any Linux; used by `crates/alo-bounding/src/turns.rs` and
`inside.rs`.
**Behaviour:** the documented way to move a task into a control group is to write
its number into one of those files, and every example does exactly that. A zero
also works and means the calling task — for `cgroup.procs` the calling process,
for `cgroup.threads` the calling **thread**, not its group leader. The kernel's
own admin guide does not mention it.
**Our response:** used deliberately, because the alternative is worse than
verbose. A thread's own identifier is `gettid`, which the standard library does
not expose, so writing the number would mean renting a crate to ask the kernel
who this thread is in order to tell the kernel to move this thread. And a number
written by hand is a number that can be somebody else's: a bug that moved another
task into an agent's boundary would be silent, and a zero cannot name the wrong
task.
**Date:** 2026-09-04

### The `inode_rename` hook has four arguments, and its destination usually names nothing
**Version:** Linux 6.18.33.2, BPF LSM; found 2026-09-07 building the rename half
of `crates/alo-bounding-kernel`
**Behaviour:** two things about this hook are not what reading the kernel's own
source first suggests.

`security_inode_rename` takes **five** arguments — the two directories, the two
directory entries, and `flags`. The **hook** takes four: `flags` is not passed
on to the security modules. A BPF LSM program is called with the hook's
arguments and then the previous module's decision, so the decision is argument
*four* here and argument *one* on a one-argument hook like `file_open`. Reading
it from the wrong slot does not fail loudly; it reads a pointer as an `i32` and
returns it, which is a boundary that refuses almost everything for reasons
nobody can see.

And the **destination entry usually has no inode**. A rename to a name nothing
is at — which is every no-clobber rename, and most ordinary ones — is handed a
*negative* directory entry: it names a place in a folder rather than a file.
Asking it which inode it is gives nothing.
**Our response:** the four arguments are written down where the hook is
declared, and the walk asks the two entries different questions, which
`deciding.rs` argues at length: the **source** is asked about the entry itself,
because the file is what a call named; the **destination** is asked about the
entry's **parent**, because a negative entry has no place to be asked about and
the folder is what the call named anyway. Asking the source's parent instead
would refuse every legitimate move, since the folder a move takes a file out of
is not a place its call names.

**The same trap, worse, one hook over.** `inode_link` is
`(struct dentry *old_dentry, struct inode *dir, struct dentry *new_dentry)` —
the directory sits **between** the two entries, where a rename has both entries
second and fourth. Reading a link's arguments in a rename's order takes the
destination folder's *inode* for the new *entry*, which is a pointer to the
wrong kind of structure entirely: the walk then reads whatever is at a
`dentry`'s offsets inside an `inode`, and refuses everything for reasons that
look like a broken boundary rather than a transposed argument. `inode_unlink` is
`(struct inode *dir, struct dentry *dentry)`, so its entry is second and the
previous module's decision is third. Four hooks, four different answers to
*which argument is which*, and none of them guessable.
**Date:** 2026-09-07, extended the same day for `inode_link` and `inode_unlink`

<!--
### <Machine or component> — <one-line summary>
**Version:** firmware / kernel / driver version the behaviour was seen on
**Behaviour:** what actually happens, as opposed to what is documented
**Our response:** what we do, and why this rather than something else
**Date:** YYYY-MM-DD, and who saw it
-->

## Application automation

Applications driven through adapters (`docs/contracts/app-adapters.md`) change
their automation surfaces between versions, sometimes silently. This is where
that gets recorded: which version, what changed, and what the adapter now does.

### GTK 3.24.41 — a hidden entry answers *what is your text?* with one bullet per character
**Mechanism:** accessibility
**Behaviour:** a `GtkEntry` with `visibility` off is exposed as a password field
(`ATSPI_ROLE_PASSWORD_TEXT`), and asked `org.a11y.atspi.Text.GetText(0, -1)` it
answers with its invisible character repeated — `●●●●●●●●●●●●●●●●●●●●●●●●●●●●`
for a 28-character password. The password is not sent, **its length is**. Nothing
in the accessibility specification requires a toolkit to withhold even that much,
and a toolkit or an application that exposes its own widgets can answer with the
text itself.
**Our response:** the fallback never asks a password field for its text at all
(`crates/alo-adapters/src/walking.rs`), and drops any text handed back beside one
(`crates/alo-adapters/src/shown.rs`). `the_accessibility_fallback_on_a_real_application.rs`
holds it against this GTK, reading the bus rather than the crate: no `Text` or
`EditableText` call reaches the field, and no answer carries the password. What a
toolkit *would* have answered is therefore never the question.
**Date:** 2026-09-16, seen on Ubuntu 24.04's `libgtk-3-0t64` under WSL.

### GTK 4.14 — no accessibility tree on the Broadway display
**Mechanism:** accessibility
**Behaviour:** GTK 4 chooses its accessibility back end by display: AT-SPI is
connected on X11 and Wayland, and on Broadway it silently falls back to a back end
that publishes nothing. `gtk4-builder-tool preview` on Broadway never appears in
the registry. GTK 3 connects its bridge regardless of the display.
**Our response:** the fallback's real-application test shows its windows with
GTK 3's `gtk-builder-tool` on Broadway, because Broadway needs no screen on a
build machine. The wire the fallback speaks is at-spi2's, the same for both
toolkits. A GTK 4 application on a certified machine, on its Wayland display, is
the on-machine acceptance in `docs/autonomy/updates/the-accessibility-fallback.md`.
**Date:** 2026-09-16.

### GTK 3.24.41 — an accessible object does not exist until something walked to it
**Mechanism:** accessibility
**Behaviour:** GTK 3's bridge creates an object's path the first time an answer
names it. Asking `/org/a11y/atspi/accessible/4` directly on a fresh window fails
with `UnknownObject`, and the numbers a window's objects get depend on the order
they were first reached — the header bar's buttons first if they were walked
first.
**Our response:** the fallback never keeps or guesses an object path: every read
and every press walks from the application's root through `GetChildAtIndex`, in
that moment, and a thing that vanishes mid-walk is left out rather than retried.
**Date:** 2026-09-16.

<!--
### <Application> <version> — <one-line summary>
**Mechanism:** api | accessibility | dbus | synthetic
**Behaviour:** what the API or the accessibility tree actually does
**Our response:** what the adapter does about it
**Date:** YYYY-MM-DD
-->

## Pinned engines

The kernel, Mesa, systemd, the model runtime and the fine-tuning stack are
configured, never patched. When one of them behaves unlike its documentation,
the accommodation lives in our configuration and the reason lives here.

An entry here that says "we patched it" is a bug in the process: a source patch
to an engine requires an ADR first.

### `systemd-cryptenroll` — there is no way to hand it the secret it is enrolling, and `--password` waits forever
**Version:** `systemd-cryptenroll` 257.13-1.fc42 in the pinned base
(`quay.io/fedora/fedora-bootc`, local image `b035260f985f`), measured 2026-09-19.
**Behaviour:** it has `--unlock-key-file=` for the secret that *opens* the volume, and no
option at all for the secret being *enrolled*. `--password` asks for the new passphrase
through `systemd-ask-password`, and closing standard input does not make it fail — it
waits. A container running `systemd-cryptenroll --unlock-key-file=… --password disk.img
< /dev/null` sat on it for two and a half minutes and was still waiting when it was
killed. An installer that took this road would hang with nothing on the screen.
**Our response:** `systemd-cryptenroll` is used for exactly two things, which are the two
it is the only tool for: `--recovery-key`, whose key it generates and prints to stdout,
and the chip (`--tpm2-device=`, `--tpm2-with-pin=`, `--tpm2-pcrs=`), whose PIN it does
take non-interactively. **The person's passphrase — ADR 0054's option B, the road a
machine with no usable chip takes — is enrolled with `cryptsetup luksAddKey`**, which
takes both the unlocking key and the new one as files. Measured working on a 64 MiB
LUKS2 virtual disk in the same base on the same day: added, the installer's first key
removed, and the volume then opened by the passphrase and by the recovery key and by
nothing else. See ADR 0056 measurement 6.

### `bootc` — `--block-setup tpm2-luks` is the chip alone, with no PIN and no recovery key
**Version:** `bootc 1.15.1` in the pinned base (`quay.io/fedora/fedora-bootc`, local image
`b035260f985f`), read on 2026-09-17.
**Behaviour:** the flag is documented as *Bind unlock of filesystem to presence of the
default tpm2 device*, which reads like the encrypted install alo OS wants and is not.
Read for everything it names around LUKS, the binary holds exactly three strings:
`luksFormat`, `systemd-cryptenroll` and `--tpm2-device=auto`. There is no
`--tpm2-with-pin`, no `--tpm2-pcrs` and no `--recovery-key` anywhere in it. So the disk
it makes opens for anybody who presses the power button on a machine whose Secure Boot
is off — which is every machine alo OS can be installed on today (ADR 0033 §4) — and a
person whose motherboard is replaced has nothing at all to recover it with.
**Our response:** the flag is not used. ADR 0054 decides the enrolment instead:
`cryptsetup luksFormat`, `systemd-cryptenroll --recovery-key`, then
`systemd-cryptenroll --tpm2-device=auto --tpm2-with-pin=yes --tpm2-pcrs=7`, and
`bootc install to-filesystem --root-mount-spec` onto the opened volume — whose own help
is what sends anything with LUKS in it to `to-filesystem` in the first place.
`crates/alo-encrypting` holds the order, and *the recovery key before the way the person
unlocks* is the part of it that is a decision.
**Date:** 2026-09-17.

### `systemd-cryptenroll` — the recovery key is on stdout, and the English is not
**Version:** `systemd-257.13-1.fc42` with `cryptsetup 2.8.4`, in the same base, measured
on a 64 MiB LUKS2 file on 2026-09-17.
**Behaviour:** `systemd-cryptenroll --recovery-key` prints two different things to two
places. **stdout** gets exactly 72 bytes: the key, as eight groups of eight characters
from the alphabet `cbdefghijklnrtuv` separated by `-`, and a newline. **stderr** gets
*A secret recovery key has been generated for this volume*, four more lines of English —
and, when stdout is not a terminal, a blank indented line where the key would be. The
key opened the volume; one character changed was refused; the passphrase the volume was
formatted with still opened it. A `systemd-recovery` token appears in the header beside
the new keyslot.
**Our response:** whatever enrols encryption reads the key off stdout and says its own
sentence about it, in the person's own language. The rented tool's English reaches a
service log and never a screen, which is the only way a machine promising 24 languages
can use a tool that speaks one. `alo_encrypting::RecoveryKey::as_printed` is what reads
those bytes and refuses anything that is not them; its unit tests are built on the key
this measurement produced.
**Date:** 2026-09-17.

### `cryptsetup` — a secret that opens nothing is exit 2, and a key file it can read is a complaint
**Version:** `cryptsetup 2.8.4` in the pinned base
(`quay.io/fedora/fedora-bootc:42@sha256:077182b6…`), measured on a 64 MiB LUKS2 file
under `podman --privileged` on 2026-09-20.
**Behaviour:** two things, both of which decide code rather than being trivia.

**One.** Every way a secret can fail to open a volume is the same answer: **exit 2**, with
*No key available with this passphrase* on stderr. Measured three ways on one volume — the
installer's own first key after `luksRemoveKey` wiped it, a secret that was never this
disk's, and the person's old secret after it had been replaced. `cryptsetup`'s other
numbers are 1 for arguments it did not understand, 3 for out of memory, 4 for a device that
is not the kind it was told, and 5 for a volume already open or busy; **only 2 is a fact
about a secret**, and anything else read as one would tell a person their correct key was
wrong.

**Two.** `systemd-cryptenroll` complains in as many words about a key file anybody but its
owner could read — *`…/first.key` has 0644 mode that is too permissive, please adjust the
ownership and access mode* — and carries on anyway. A warning from a rented tool about the
way we handed it a key is a bug of ours, not a line to filter out of a log.

**Our response:** `alo_encrypting::TheDiskRefused::from_how_it_ended` reads exit 2 as
*what was given does not open it* and every other non-zero answer as *the rented tool
refused and did not say why* — which is deliberately vague, because guessing would mean
telling somebody at a machine that will not open that their recovery key was wrong.
`alo_encrypting::ONLY_ITS_OWNER_MAY_READ_IT` is `0o600` because of the second, and the
key files live under `/run`, which is memory rather than the disk being encrypted.
`crates/alo-encrypting/tests/the_sequence_against_a_virtual_disk.rs` is where all of it
is measured again on every run.
**Date:** 2026-09-20.

### GRUB — the base's signed loader leaves `cmdpath` empty, and sets `config_directory`
**Version:** `grub2-efi-x64-2.12-32.fc42.x86_64` as
`quay.io/fedora/fedora-bootc:42@sha256:077182b6…` ships it, started by
`shim-x64-15.8-3`, from a FAT partition's `EFI/BOOT/`, under QEMU with Fedora's
`edk2-ovmf-20250812-21.fc42` and Secure Boot on. 2026-09-16.
**Behaviour:** the boot environment's entry is the same file on every machine,
and the disk the person chose is staged beside it as `chosen.cfg` for the entry
to source. GRUB's documented variable for *the directory this loader was started
from* is `cmdpath`, and `image/installing/grub.cfg` used it. It is **empty**.
Asked in a machine, the loader answers:

```
ALODIAG cmdpath=[]
ALODIAG prefix=[(hd0,gpt5)/EFI/fedora]
ALODIAG config_directory=[(hd0,gpt5)/EFI/BOOT]
```

Fedora's build has `/EFI/fedora` baked in as its prefix, finds no configuration
there, falls back to the directory it was itself loaded from — and records that
fall-back in `config_directory` only. So `[ -f "${cmdpath}/chosen.cfg" ]` tested
`/chosen.cfg` on the root device, found nothing, and every install refused *no
disk was chosen for alo OS, so nothing was changed*: correct behaviour on a
choice that never arrived.
**Our response:** `image/installing/grub.cfg` sources
`${config_directory}/chosen.cfg`. `wrong_with_the_entry` in
`crates/alo-installing/tests/what_the_environment_carries.rs` refuses an entry
that reads `${cmdpath}` at all, and
`crates/alo-installer/tests/the_installer_checks_consents_and_stages.rs` holds
the Windows program's staged path to the same variable — the two halves of the
choice cannot drift apart again without a test saying so.
**Date:** 2026-09-16.

### bootc 1.15.1 — in the boot environment, the image deploys and the bootloader's probe dies in `bwrap`'s `pivot_root`
**Version:** bootc 1.15.1 and bootupd 0.2.31 as
`quay.io/fedora/fedora-bootc:42@sha256:077182b6…` ships them, and the `bwrap` the
base ships, inside `image/installing/`'s initramfs; kernel
`6.19.14-101.fc42.x86_64`; QEMU q35 with Fedora's `edk2-ovmf-20250812-21.fc42`,
**Secure Boot enabled**, processor emulated (TCG — this host's `/dev/kvm` has
nothing behind it, see *WSL on a VMware guest shows `/dev/kvm`*); 3 GB of memory;
release `0.0.1` pulled by its pinned digest. 2026-09-16.
**Behaviour:** `bootc install to-disk --source-imgref registry:…` partitions the
chosen disk, makes its filesystem and deploys the release, and then fails
installing the bootloader. Until this run the environment said only its own
sentence, so *why* was on no screen and in no log anybody could reach. With the
installer's complaint noted on the serial line (`TheMachine::note`), the run's
console — `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`,
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`,
its `installing.log` — says, in order:

```
[    0.000000] secureboot: Secure boot enabled
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: virtio-alo-target
Checking that virtio-alo-target is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. …
[   67.591114]  vdb: vdb1 vdb2 vdb3
[   70.670731] EXT4-fs (vdb3): mounted filesystem c7c658c1-… r/w with ordered data mode.
Still installing alo OS. Leave the computer on            (37 times, one a minute)
/usr/bin/bootc: mke2fs 1.47.2 (1-Jan-2025)
/usr/bin/bootc: Deploying container image...done (3 minutes)
/usr/bin/bootc: error: Installing to disk: Installing bootloader: Probing bootupd --filesystem support: Subprocess failed: ExitStatus(unix_wait_status(256))
/usr/bin/bootc: bwrap: pivot_root: Invalid argument
alo OS could not be installed onto virtio-alo-target. That disk may now hold part of alo OS; nothing else on this computer was changed. Restart to try again
You can turn this computer off or restart it now
```

So the Secure Boot chain, the choice, the signature and the deploy all work; what
stops the install is `bootupctl`'s probe, which bootc runs inside the new
deployment through `bwrap`, and `bwrap`'s `pivot_root(2)` answering `EINVAL`. The
2026-09-15 entry above met the same step as *No such file or directory*, before
`bwrap` and `bootupctl` were carried; carrying them moved the failure one call
further, to here.
**Located from a run, 2026-09-16:** the installer runs from the kernel's initial
root file system, and `pivot_root(2)` refuses any caller whose root mount has no
mount above it. `bootc` 1.15.1 starts the probe as `bwrap --bind <root> / --proc
/proc --dev-bind /dev /dev --tmpfs … bootupctl backend install --help` (the
arguments are in its binary beside *Running bootupctl via bwrap in*), and
`bwrap` 0.10.0 always pivots. The kernel's `do_pivot_root` answers `EINVAL` when
the current root is the *absolute root*, a mount that is its own parent — and the
environment never switches out of the initramfs, because there is no root to
switch to. The pinned kernel has no option that changes this (its `config`
names none). Measured by booting the base's own kernel with an initramfs made by
the base's own `dracut` from `image/installing/alo-installing.conf` plus two
diagnostic units, run one after the other, each printing `/proc/self/mountinfo`
and then running that same `bwrap` line (QEMU q35, direct kernel boot, TCG, no
disks attached):

```
ALO-DIAG-bare-BEGIN                       (a unit on the initramfs's own root)
1 1 0:2 / / rw shared:1 - rootfs rootfs rw,size=1401480k,…
bwrap: pivot_root: Invalid argument
ALO-DIAG-bare-END
ALO-DIAG-rooted-BEGIN                     (RootDirectory=/run/alo/installing/root)
122 110 0:2 / / rw shared:44 master:1 - rootfs rootfs rw,size=1401480k,…
Usage: bootupctl backend install [OPTIONS] <DEST_ROOT>
ALO-DIAG-rooted-END
```

Mount `1`'s parent is `1`: the absolute root, and `bwrap` is refused. Mount
`122`'s parent is `110`: the same files, bound onto a directory and moved onto
`/` by systemd in the unit's own namespace, and the same `bwrap` pivots and
starts `bootupctl`. Secure Boot plays no part in this: the refusal is the
kernel's, whatever started it.
**Our response:** none to the engines (ADR 0011). The environment binds itself
again, whole and with its mounts, at `/run/alo/installing/root`
(`image/installing/run-alo-installing-root.mount`, `Options=rbind,rslave`), and
`alo-installing.service` takes that as its root (`RootDirectory=`), requiring and
waiting for the mount. `crates/alo-installing/tests/what_the_environment_carries.rs`
holds both units to that and refuses the service back on the initramfs's root,
a plain `bind`, a shared bind, and a root bound anywhere else. Earlier, the
environment began noting the last lines any failing program complained of on the
machine's serial lines and its log — never on the screen, whose sentences name
no machinery (`docs/features.md`) — which is how this was read from the console
rather than found by elimination. Whether the install then finishes and the disk
boots to `alo-agentd` with Secure Boot on is the installer plan's task 13: not
yet run, because one emulated run needs a machine with room for its disks.
**Upstream:** not theirs. The kernel documents the refusal, and `bwrap` needs a
root it can pivot from.
**Date:** 2026-09-16.

### bootc 1.15.1 — `bootc install` ends "No such file or directory" when the environment lacks `fstrim`
**Version:** bootc 1.15.1 as `quay.io/fedora/fedora-bootc:42@sha256:077182b6…`
ships it, inside `image/installing/`'s initramfs under the bound root of the entry
above; util-linux 2.40.4-10.fc42; QEMU q35 with Fedora's
`edk2-ovmf-20250812-21.fc42`, **Secure Boot enabled**, processor emulated (TCG);
release `0.0.1` pulled by its pinned digest. 2026-09-16.
**Behaviour:** with the bootloader's sandbox pivoting, the install ran further than
it ever had, and then ended with an error that names no program and no path. The
run's console (the installer plan's task 13, kept on the third PC as
`C:\dev\setup\task13-install-run.log`):

```
[    0.000000] secureboot: Secure boot enabled
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. …
Still installing alo OS. Leave the computer on            (46 times, one a minute)
/usr/bin/bootc: mke2fs 1.47.2 (1-Jan-2025)
/usr/bin/bootc: Deploying container image...done (3 minutes)
/usr/bin/bootc: error: Installing to disk: No such file or directory (os error 2)
alo OS could not be installed onto virtio-alo-target. That disk may now hold part of alo OS; nothing else on this computer was changed. Restart to try again
You can turn this computer off or restart it now
[ 2894.875975] EXT4-fs (vdb3): unmounting filesystem 8c600da7-….
```

**Located:** bootc attaches a context to every step it names — *Installing
bootloader*, *Querying for bootupd*, *Writing aleph version*, *Opening deployment
dir* — and anyhow prints every context in the chain, so an error carrying only the
outermost one, *Installing to disk* (`install_to_disk`), came from a call inside it
that adds none. In bootc 1.15.1's `crates/lib/src/install.rs` those are, after the
deploy: the bound images (none in this release), the image store's labels
(SELinux is off in the environment), and **`finalize_filesystem`**, which for each
file system it made runs `Task::new("Trimming root", "fstrim")`, then `mount -o
remount,ro`, then `fsfreeze -f` and `-u` — and `install_to_disk` then runs `umount
-R`. `Task::run` starts its program with a bare `cmd.spawn()?`, so a program that is
not there is exactly `No such file or directory (os error 2)` with nothing around it.
Of the four, the initramfs's list (`alo-installing.conf`) named `mount` and
`fsfreeze`; dracut's own `base` module brings `umount`; and **nothing brought
`fstrim`**: `grep -rlw fstrim /usr/lib/dracut/` in the pinned base finds no module
that installs it. The ext4 unmount on the console's last line is the kernel
tearing down the mounts bootc left when it exited, which fits a failure at the
step before its own `umount`.
**Our response:** none to the engines (ADR 0011). `alo-installing.conf` carries
`/usr/sbin/fstrim` and, named rather than left to a dracut module, `/usr/bin/umount`;
`lsinitrd` of the rebuilt initramfs lists `usr/bin/fstrim`, `usr/bin/mount`,
`usr/bin/fsfreeze` and `usr/bin/umount`.
`crates/alo-installing/tests/what_the_environment_carries.rs` holds the list to
all four (*what the installer finishes with*) and refuses a list without each one.
With them carried, the same test on the same machine, Secure Boot on, said:

```
[   71.936408] EXT4-fs (vdb3): mounted filesystem 70b9e529-… r/w with ordered data mode.
Still installing alo OS. Leave the computer on            (44 times)
[ 2777.778647] EXT4-fs (vdb3): re-mounted 70b9e529-… ro.
alo OS is installed. This computer restarts in a few seconds
[ 2795.317208] EXT4-fs (vdb3): unmounting filesystem 70b9e529-….
```

— the read-only remount is `finalize_filesystem`'s second step, after `fstrim`, and
the installed disk then booted with Secure Boot on (the installer plan's task 13
report,
`docs/autonomy/updates/the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`).
The same run found a fault in the test rather than the environment: after a failed
install the environment waits for a person and never powers off, and the test
waited out its whole deadline, 71 minutes past the failure; it now ends the moment
the environment says *You can turn this computer off or restart it now*.
**Upstream:** a missing program named in the error would have saved a run; bootc's
`Task` could say which program it could not start. Not filed from here.
**Date:** 2026-09-16.

### systemd 257 — systemd mounts the BPF filesystem so only root can pass through it
**Version:** systemd 257.13-1.fc42, kernel 6.19.14-101.fc42, selinux-policy
42.24-1.fc42, as release `0.0.1` (`ghcr.io/aloworld-org/alo-os@sha256:d3f05b60…`)
carries them; QEMU q35 with Fedora's `edk2-ovmf-20250812-21.fc42`, **Secure Boot
enabled** under Microsoft's certificates, processor emulated (TCG). Compared with
WSL 2 (`6.18.33.2-microsoft-standard-WSL2`, systemd 255 as PID 1). 2026-09-16.
**Behaviour:** the disk installed under Secure Boot booted, `alo-boundaryd` loaded
and pinned the boundary, and `alo-agentd` stopped a second later saying there was
no boundary. The installed machine's own account, printed to the serial line by the
installer test's watching unit:

```
[  253.815658] fedora alo-boundaryd[809]: alo-boundaryd: the boundary is on this kernel, pinned at /sys/fs/bpf/alo, with 60989 to write it and nobody else; this process holds nothing and is done
[  254.917004] fedora alo-agentd[1124]: alo-agentd did not run: a turn's work cannot be bounded on this machine: there is no boundary at /sys/fs/bpf/alo/bounds: alo-boundaryd loads one at boot, and until it has, this machine cannot bound a turn — …
× alo-agentd.service - alo OS agent service: the door an agent knocks on
     Active: failed (Result: exit-code) since Thu 2026-09-17 00:55:27 UTC; 32s ago
    Process: 1124 ExecStart=/usr/bin/alo-agentd (code=exited, status=1/FAILURE)
/sys/fs/bpf:
drwx-----T.  3 root root      0 Sep 17 00:53 .
drwxr-x---.  2 root alo-agent 0 Sep 17 00:55 alo
none [integrity] confidentiality
Enforcing
uid=1000(alo) gid=1000(alo) groups=1000(alo),60989(alo-agent)
```

**Located:** the boundary *was* there. The loader made `/sys/fs/bpf/alo` `0750
root:alo-agent` exactly as ADR 0018 asks, but systemd, which mounts the BPF
filesystem early in PID 1, mounts it with `mode=0700`, so its root is `1700
root:root` and nobody but root can pass through it to the directory beneath.
`alo-agentd` runs as the person, holding nothing, so it could not reach the pin.
`alo_bounding::Boundary::opened` asks `Path::exists()`, which answers `false` for a
path it is not allowed to look at as well as for one that is not there, so the
refusal said *there is no boundary* when there was one it could not reach. On WSL,
where every boundary test in this repository had run, the same mount point is
`drwxrwxrwt root:root`, the kernel's own default. The mount's source there is
`bpffs`, not systemd's `bpf`: WSL's init mounts it before systemd starts, and
systemd leaves a mount that is already there alone. So no test could have seen
this.
Neither Secure Boot's lockdown (`integrity`) nor SELinux (`Enforcing`) played any
part: with the passage below given, the same disk under the same firmware ran
`alo-agentd`.
**Our response:** none to systemd (ADR 0011). `image/usr/lib/tmpfiles.d/alo.conf`
adjusts the mount point, and makes nothing: `z /sys/fs/bpf 0710 root alo-agent -`.
The agent's group may pass through and do nothing else there. It can't list what is
pinned, it can't write, and nobody else may pass at all. The same release, written
to a disk by its own `bootc install to-disk` and booted under the same Secure Boot
firmware with only that line added through systemd's `tmpfiles.extra` credential,
said:

```
Id=alo-agentd.service
ActiveState=active
SubState=running
/sys/fs/bpf:
drwx--x---.  3 root alo-agent 0 Sep 17 01:02 .
drwxr-x---.  2 root alo-agent 0 Sep 17 01:04 alo
/run/alo/1000:
srw-rw----. 1 alo  alo-agent  0 Sep 17 01:04 agentd.sock
```

`crates/alo-image/src/reaching.rs` holds the line to the mode, to root, and to the
group the loader's unit runs in, and refuses an image without it, a directory made
there instead, every wider mode, another owner or group, and a loader that does not
wait for the mount. The credential was a diagnosis on a scratch disk. The
installer's own test changes nothing on the disk it installs: the line reaches a
machine in the next release the owner signs (ADR 0036), and release `0.0.1` boots
with `alo-agentd` failed as above.
**Upstream:** systemd's `mode=0700` is deliberate (pins are privileged objects), and a
distribution that wants an unprivileged reader is expected to arrange its own
passage, which is what this is. Not a bug to file.
**Date:** 2026-09-16.

### The Linux kernel — a link-local IPv6 address is only an address beside its interface, a new one cannot be used for a moment, and a development machine may have IPv6 off where a fresh namespace has it on
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`), rustix 1.1.4; measured on 2026-09-15 by
`crates/alo-nearby/tests/asked_and_answered_over_ipv6.rs` and
`crates/alo-agentd/src/two_machines_with_no_ipv4.rs`.
**Behaviour:** RFC 6762 §3 gives mDNS an IPv6 group, `ff02::fb`, and RFC 4007 says
a link-local address needs a zone. What that means on Linux, for discovery on a
network with no IPv4 address:

- **The WSL2 kernel's first network namespace has IPv6 switched off**
  (`net.ipv6.conf.all.disable_ipv6 = 1`: no `::1`, no link-local addresses), and a
  network namespace made with `unshare --net` starts with it **on** — a `veth`
  brought up inside one had an `fe80::/64` address within a couple of seconds.
  Measured. So a test that needs IPv6 cannot assume the host has it, and the
  IPv6 tests here run inside a namespace of their own rather than on the host.
- **The scope is the interface a datagram arrived on, and it is what makes the
  address reachable.** A question sent to `ff02::fb` with `sin6_scope_id` naming
  one end of a `veth` left on that interface, and the answer's source came back
  as a link-local address with the **asking** interface's index as its scope;
  dialling that scoped address reached the other machine and paired. Measured.
  The same address with no scope names no interface, and a datagram or connection
  to it has nowhere to go (the kernel's documented behaviour; not measured here —
  `alo-nearby` refuses such an answer before anything would try).
- **A new link-local address is *tentative* while duplicate address detection
  runs** (`IFA_F_TENTATIVE`, RFC 4862 §5.4), and nothing can be bound to it until
  that ends; the kernel reports the flag in the address dump and sends
  `RTM_NEWADDR` again to `RTMGRP_IPV6_IFADDR` when it clears (documented kernel
  behaviour). The tests wait for the flag to clear before binding. The flags byte
  in `ifaddrmsg` holds only the low eight flags; `IFA_FLAGS` carries all of them
  where the kernel sends it.
- **A listener bound to `[::]` with `IPV6_V6ONLY` cleared accepts IPv4 as well**,
  and reports an IPv4 peer as `::ffff:a.b.c.d`; whether it is cleared by default
  is `net.ipv6.bindv6only` (documented kernel behaviour, not measured here).

**Our response:** `alo-agentd` joins `ff02::fb` on every interface that is up and
running, carries multicast, has a link-local address the kernel has finished
checking and is not loopback (`crate::networks::link_local_networks`, addresses
read without tentative or failed ones in `crate::route_messages`), follows
`RTMGRP_IPV6_IFADDR` so an address that finishes its check is joined then, opens
the IPv6 discovery socket with `IPV6_V6ONLY` set and the port's listener with it
cleared — both explicitly, never left to a sysctl — and reads a peer's address
back through `alo_nearby::HeardFrom`, which keeps a link-local address's scope and
turns a mapped IPv4 address back into IPv4. A kernel with no IPv6 is a line in the
service log and discovery over IPv4 alone. What two physical machines on a cable
with no DHCP server hear is still owed to two machines.

### The Linux kernel — a link-local destination takes its interface from the address or else from the socket, the same link-local address on two interfaces is two destinations, and `/sys/class/net` answers for another namespace
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`),
iproute2 6.19.0 (`dummy`, `veth`), `http` 1.5.0 and `ureq` 3.4.0; measured on
2026-09-15 by `crates/alo-bounding/tests/a_link_local_departure_names_its_interface.rs`
and `crates/alo-agentd/src/a_paired_machine_over_link_local.rs`.
**Behaviour:** RFC 4007 says a link-local address needs a zone; ADR 0041 made the
zone part of a departure, and to decide one the programme has to take the interface
from where the kernel does:

- **The same link-local address on two interfaces is two reachable places.** Two
  `dummy` interfaces in one namespace, each given `fe80::a1/64` with `nodad`: a
  connection to `fe80::a1%<first>` and one to `fe80::a1%<second>` were both accepted
  by one listener on `[::]`, and a datagram to either was sent. Measured — and
  measured again as the reason for the decision: with the interface dropped from the
  departure (a mutation run of the test), a turn shown the first interface reached
  the second by all four roads.
- **Where a scope is read from.** `connect(2)` and `sendto(2)` take the interface
  from `sin6_scope_id` when it is not zero; with it zero, from the interface the
  socket is held to (`sk_bound_dev_if`), which a scoped `connect` and a `bind` to a
  scoped link-local address set. Measured for the three a Rust program can make: a
  scoped `connect` and the writes after it, a scoped `sendto`, and a `sendto` naming
  no scope from a socket bound to `fe80::a1%<interface>` — each reached or refused
  by the interface it named or was held to. That the kernel reads a scope only from
  an address of the full `sizeof(struct sockaddr_in6)`, twenty-eight bytes, and
  that `IPV6_UNICAST_IF`, `IPV6_MULTICAST_IF` and an `IPV6_PKTINFO` control message
  choose an interface only when neither of those did, is the kernel's source
  (`tcp_v6_connect`, `udpv6_sendmsg`), not measured: `std` always passes twenty-eight
  bytes and sets none of the three.
- **`/sys/class/net/<name>` answers for the namespace `sysfs` was mounted in.** Inside
  `unshare --net`, with `/sys` inherited, the interfaces the namespace made were not
  there, and `/proc/net/if_inet6` — which answers for the reader's namespace — listed
  them with their indexes. Measured. A `dummy` interface also carries no
  `IFF_MULTICAST` (the driver's source, not measured), so
  `crate::networks::link_local_networks` rightly never lists one.
- **A zone inside brackets is a valid authority to `http`, and `ureq` dials the
  address it was handed.** `http://[fe80::…%3]:7611/v1/chat/completions` parsed, and
  the question reached the studio at the registered scoped address through
  `alo_asking`'s resolver of registered addresses. Measured end to end. `std`'s
  `(host, port).to_socket_addrs()` does not parse a `%` itself and hands such a host
  to the system resolver (the standard library's source).
- **A threaded `home` group left by a test that panicked makes its parent "domain
  threaded"** and the next `Turns::of_this_service` from that scope fails with
  `EEXIST` — measured once while building the end-to-end test, on
  `/sys/fs/cgroup/init.scope/home`, removed by hand once `cgroup.threads` read
  empty. The same cascade *A boundary fixture takes a control group away while its
  process is still leaving* records.

**Our response:** `alo_bounding_map::Departure::on` keeps an interface exactly where
`needs_an_interface` (the kernel's `__ipv6_addr_needs_scope_id`) says an address
needs one, and `Departures::holds` refuses a destination that needs one and has
none. `alo-bounding-kernel`'s `departing.rs` reads `sin6_scope_id` only when the
caller's length is twenty-eight bytes or more, else `skc_bound_dev_if` — both
offsets for the socket side read out of the running kernel's type information — and
for a joined socket's peer reads `skc_bound_dev_if`; a destination that names no
interface either way is refused. `alo-agentd` registers a scoped address on its
interface and refuses to register a link-local one with none. `alo-turn` parses a
scoped literal itself rather than handing it to the resolver. The tests read
interface indexes from `/proc/net/if_inet6`, and the end-to-end test gives the
control group subtree back whether or not it fails. What two physical machines on a
cable measure is still owed to two machines.

### The Linux kernel — a socket held to an interface holds an IPv4 connection there, `IP_PKTINFO` sends a datagram past it, and one IPv4 address on two interfaces of one namespace is loopback's
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`, `setpriv`), iproute2 6.19.0 (`veth`, `dummy`), Python 3 for the probes;
measured on 2026-09-15 by probes run before the code and by
`crates/alo-bounding/tests/a_private_ipv4_departure_is_held_to_its_network.rs` and
`crates/alo-agentd/src/a_paired_machine_on_two_networks_with_one_address.rs`.
**Behaviour:** ADR 0044 holds a paired machine's IPv4 departure to the interface of
the network it was found on, which rests on what the kernel does with a socket held
to an interface — and on one place it does not do what the IPv6 half does:

- **Two equal routes to one private range: the first added wins, and a socket held
  to an interface ignores both.** A namespace with `10.9.0.1/24` on two `veth`
  cables, each to a namespace of its own at `10.9.0.20`: an unheld `connect` reached
  the far end of the cable whose address was added first, and one from a socket
  held (`SO_BINDTODEVICE`) to either cable reached that cable's far end. Measured,
  and again end to end: with the corridor's hold removed (a mutation run), the
  question to the studio went by the route to somebody else.
- **An unprivileged process can hold a socket once, and never move it.** As `nobody`
  with no capabilities (`setpriv --reuid=65534 --inh-caps=-all --bounding-set=-all`),
  `SO_BINDTODEVICE` on a fresh socket succeeded and connected there; the same option
  on a socket already held, after it connected, failed with `EPERM`. As root it
  succeeded. Measured. The kernel's source says why: since 5.7 an unheld socket
  needs no `CAP_NET_RAW`, a held one does.
- **On IPv4, an `IP_PKTINFO` control message sends a datagram out of the interface it
  names, past the one the socket is held to.** A datagram socket held to the first
  cable, `sendmsg` to `10.9.0.20` with `IP_PKTINFO` naming the second: the datagram
  arrived at the second cable's far end, as root and as `nobody`. Measured. An
  `IP_UNICAST_IF` on an unheld socket did the same, which is its purpose.
- **On IPv6 the kernel refuses the same thing.** A socket held to one cable,
  `sendmsg` to a link-local address with `IPV6_PKTINFO` naming the other interface:
  `EINVAL`, with the address scoped and unscoped, and nothing arrived. Measured —
  `ip6_datagram_send_ctl` checks the two agree; `ip_cmsg_send` has no such check.
- **One IPv4 address on two `dummy` interfaces of one namespace is local, and
  loopback answers it.** `10.64.0.20/24` on two `dummy` interfaces: an unheld
  connection and one held to the first reached a listener on `0.0.0.0`; one held to
  the second timed out. Measured. So the kernel test puts each copy of the address at
  the far end of a `veth` cable in a namespace of its own, where IPv6's link-local
  test could use two `dummy`s.
- **`struct msghdr`'s `msg_controllen` is eight bytes and is what the programme is
  handed at `socket_sendmsg`.** The width is checked against the running kernel's
  type information when the programme is loaded, and a datagram carrying
  `IP_PKTINFO` was refused by it — measured; that `____sys_sendmsg` has already
  copied the control messages into the kernel and kept their length is the kernel's
  source.

**Our response:** `alo_bounding_map::Departure` keeps an IPv4 departure's interface,
and `Departure::permits` holds one with an interface to that interface while one with
none permits any, as before. `alo-bounding-kernel`'s `departing.rs` reads
`skc_bound_dev_if` for every IPv4 destination, named or joined, and decides a message
whose `msg_controllen` is not zero as held to no interface — so `IP_PKTINFO` and
`IP_UNICAST_IF` reach only a departure held to none. `alo-agentd` asks each IPv4
network from a socket held to its interface and writes what it heard down on it,
and `alo-asking` dials a paired machine found there from a socket held to the same
interface; no process in a turn can move it afterwards, because `alo-agentd` holds
no capability (ADR 0018). **What is not closed:** a turn running as root could move a
connected socket to another interface; alo OS runs no turn as root.

### The Linux kernel — an unheld TCP listener answers every handshake by the route, and the address a connection was made to is the address that was dialled
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`veth`), Python 3 for the probe; measured on
2026-09-16 by a probe run before the code and by
`crates/alo-agentd/src/a_proposal_measured_on_the_network_it_arrived_on.rs`.
**Behaviour:** a namespace with two `veth` cables, `10.66.0.1/24` on one and
`10.66.0.3/24` on the other — two routers handing out one private range — with a
machine at `10.66.0.2` at each far end, and one `TcpListener` on `0.0.0.0:7610`
held to no interface:

- **A connection completes only from the network the route points at.** With
  `10.66.0.2/32` routed over the first cable, the machine at the far end of that
  cable connected to **both** `10.66.0.1` and `10.66.0.3`; the machine at the far
  end of the other cable timed out on both. Measured. The kernel's source says
  why: for an unheld listener `ireq->ir_iif` is zero, so the route lookup for the
  SYN-ACK (`inet_csk_route_req`) is unconstrained and the reply leaves by the
  route, to the wrong machine.
- **`getsockname` on the accepted connection is the address that was dialled**,
  not one belonging to the interface the packet arrived on: the connection made
  to `10.66.0.3` over the first cable was accepted with local address
  `10.66.0.3`. That is Linux's weak host model.
- **An established connection follows the main table when it changes.** Moving
  `10.66.0.2/32` to the other cable while a connection was open sent that
  connection's next reply out of the other cable, and the machine that had
  connected never received it. Measured — the end-to-end test therefore moves
  only where *questions* go, with an `ip rule ipproto udp` and a table of its own.

**Our response:** `crate::arrived_on` reads the network a connection arrived on
from the interface that owns the accepted socket's local address, and refuses —
[`ArrivedOn::NothingCouldSay`], measured nowhere, *not found* — when two
interfaces own it or none does. While the wire listened on one socket held to
nothing, the first measurement is what made that reading exact: the only
connections that completed arrived on the route's network. Since
`crate::listeners` the wire listens on one held listener per IPv4 network, so a
connection from the network the route does not point at completes — and the
reading is taken from the listener that accepted rather than from the address
dialled, which is the entry below.

### The Linux kernel — two TCP listeners on `0.0.0.0` coexist when they are held to different interfaces, an unheld one beside them is refused, and holding a socket to an interface by index needs no capability
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`veth`), socket2 0.6, Python 3 for the probe;
measured on 2026-09-16 by a probe run before the code and by
`crates/alo-agentd/src/listeners.rs` and
`crates/alo-agentd/src/a_machine_reachable_on_both_networks.rs`.
**Behaviour:** the entry above says an unheld listener answers every handshake by
the route, which on two networks carrying one private range makes this machine
unreachable from the network the route does not point at. Four things decided how
`crate::listeners` answers it:

- **Two listeners bound to `0.0.0.0` at one port coexist when each is held to a
  different interface** (`SO_BINDTOIFINDEX` set **before** `bind`). The kernel's
  bind-conflict check treats a different `sk_bound_dev_if` as a different binding.
  Measured with `lo` and `eth0`.
- **A listener at the same port held to *nothing* beside them is refused
  `EADDRINUSE`.** Measured. So the wire cannot keep its old listener in both
  families and add held ones beside it — that listener binds IPv4 too. It binds
  the held ones and one **IPv6-only** listener (`IPV6_V6ONLY` set), which takes no
  IPv4 address and conflicts with none of them; that combination binds. Measured.
- **A held listener answers its handshakes out of the interface it is held to.**
  `inet_csk_route_req` looks the SYN-ACK's route up with `ireq->ir_iif` as the
  output interface, and `inet_request_bound_dev_if` takes that from the listener's
  `sk_bound_dev_if`. Measured over two `veth` cables carrying one private range,
  with the route pointing at the other cable: a connection from the far end of the
  first cable to an **unheld** listener never completed, and to a listener **held
  to that cable** completed at once.
- **`SO_BINDTOIFINDEX` needs no `CAP_NET_RAW`.** Measured as uid 65534 with no
  capabilities, which is the shape `alo-agentd` runs in (ADR 0018). `SO_BINDTODEVICE`,
  which names the interface rather than numbering it, is the one that needs it.
- **`SO_BINDTOIFINDEX` takes an index no interface has.** `sock_bindtoindex_locked`
  refuses only a negative index; it does not look the device up. Measured: a
  listener held to interface `99999`, which this host does not have, bound and
  listened without complaint, and nothing can ever reach it. So an interface that
  goes between the kernel's report and the bind is **not** a refused bind — it is a
  listener nobody reaches, let go of at the next notification, when its index is no
  longer reported. What really refuses a network's bind is somebody else already
  holding the port there (`EADDRINUSE`), and that is what
  `listeners::tests::a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound`
  arranges. A connection dialled from a socket held to such an index is refused at
  `connect`, which is what `alo-nearby`'s dialling relies on.

**Our response:** `crate::listeners` binds one IPv4 listener per IPv4 network the
kernel reports — loopback among them, multicast not asked for — each held to that
network's interface, beside one IPv6-only listener held to nothing; they follow the
kernel's network notifications as `crate::joining`'s joins do, and a network that
will not bind is a line in the service log. `crate::arrived_on::what_a_listener_held_to`
takes the network a connection arrived on from the listener that accepted it, so no
address of this machine's is read and the weak host model cannot misattribute a
connection. A machine whose interfaces cannot be read binds one listener held to
nothing, says so, and reads a connection's network the old way.
**What was not closed then, and is now:** a discovery answer left by the route as
well, because the socket `crate::wire` answered *who is here* on was one socket
held to no network. The entry below is that hold, and it is why
`crates/alo-agentd/src/a_machine_reachable_on_both_networks.rs` still carries an
`ip rule ipproto udp` and a table of its own — it was written while discovery had
no hold, so that what it measured was the handshake, and it is left as it is
rather than rewritten. The fixture with **no rule at all** is
`crates/alo-agentd/src/a_discovery_answer_leaves_on_the_network_it_arrived_on.rs`.
**Date:** 2026-09-16.

### The Linux kernel — held datagram sockets on one port coexist, a question is delivered only to the socket held to the interface it arrived on, and the answer leaves by that interface
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`veth`), socket2 0.6, Python 3 for the probe;
measured on 2026-09-16 by a probe run before the code and by
`crates/alo-agentd/src/responding.rs` and
`crates/alo-agentd/src/a_discovery_answer_leaves_on_the_network_it_arrived_on.rs`.
**Behaviour:** the entries above hold the *port* to the network a connection
arrived on. Discovery is datagrams, and its answer is sent rather than routed by a
handshake the kernel completes, so it needed its own measurement. A namespace with
two `veth` cables carrying `10.67.0.0/24`, a machine at `10.67.0.2` at each far
end, the route to that address pointing at the cable the asking machine is **not**
on, and one datagram socket per interface bound to `0.0.0.0:5399` with
`SO_BINDTOIFINDEX` set before the bind:

- **Three held datagram sockets at one port coexist**, held to `lo`, to the first
  cable and to the second. Measured — for UDP the bind-conflict check treats a
  different `sk_bound_dev_if` as a different binding, as it does for TCP.
- **A socket held to *nothing* at the same port binds beside them**, which is
  where UDP differs from TCP: `SO_REUSEADDR` is enough, and nothing refuses it.
  Measured. So held sockets do not *displace* an unheld one the way held listeners
  do — the machine has to stop binding one, which is what `crate::responding`
  does.
- **A multicast question that arrived on one cable is delivered to the socket held
  to that cable and to no other**, and so is a unicast question to that cable's
  address. Measured, both.
- **The answer that socket sends leaves by the interface it is held to**, though
  the route to the asking machine's address points at the other cable: the machine
  that asked heard it, and the machine at the same address on the other network
  heard nothing. Measured.
- **An unheld socket in the same place answers by the route**: it heard the
  question from the cable perfectly well, and its answer went out the other
  network — the machine that asked heard nothing. Measured, which is the failure
  this was built for.

**Our response:** `crate::responding` answers discovery on one datagram socket per
network the kernel reports — loopback among them, as `crate::listeners` has it —
each held to that network's interface and joined to the group on the networks that
carry multicast, and binds **no** unheld socket beside them. The IPv6 socket stays
held to nothing, because a link-local address carries the interface it was heard on
in its own scope and the kernel answers back out of that one. A network the kernel
numbers zero, and a machine whose interfaces cannot be read at all, are answered on
**no** network rather than by the route — where `crate::listeners` binds one
listener held to nothing, because an unheld handshake merely fails while an unheld
answer reaches a machine that did not ask.
**What is not closed:** `IP_PKTINFO` would name the arriving interface per
datagram and needs no list of interfaces, but no crate in this workspace parses a
`cmsghdr` safely and `CLAUDE.md` forbids `unsafe`; so a question arriving on an
interface this machine cannot enumerate is answered by nobody rather than answered
exactly.
**Date:** 2026-09-16.

### The Linux kernel — a socket held to an interface that has gone stays open and silent, and a cable comes back as a new interface or the same one
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`), socket2 0.6; measured on 2026-09-16 by
`crates/alo-agentd/src/a_cable_pulled_and_plugged_in_again.rs`.
**Behaviour:** the entries above say what a socket held to an interface does while
that interface is there. What happens when a cable is pulled and plugged in again
while the service runs is not written down anywhere, and it is not one thing:

- **A cable pulled at the far end is an interface that goes.** Ending the last
  process in the far end's network namespace destroys its `veth`, and the peer in
  this namespace goes with it — a moment later, not in the same instant, because
  a namespace is torn down asynchronously. The routing socket says so. **The same
  cable laid again is a new interface with a new index**, so nothing matched by
  index survives it.
- **A cable pulled by its link going down is the same interface.** `ip link set
  down` keeps the index and the IPv4 address and clears `IFF_UP`; the far end
  loses its carrier. Set up again, it is the same index. Both changes are said on
  the routing socket.
- **A socket held to an interface that has gone stays open, and `poll` says
  nothing about it** — no error, no hang-up, and no datagram will ever arrive.
  Measured: discovery's answering thread, asleep on the responders it had taken
  before the colleague's cable went, stayed asleep on the dead one while the
  colleague's new cable was answered on by a socket it was not waiting on, and the
  colleague never heard an answer. So nothing about a gone interface wakes a
  thread that is not also reading the routing socket.
- **A TCP listener held to an interface that is down binds**, at the port the
  service listens at, and a second listener held to that interface at that port is
  then refused `EADDRINUSE` when the link comes up — which is how the fixture
  makes a network refuse the service's bind without a bug to make it.
- **A `veth` set up just before a service starts may not be running yet when the
  service reads its interfaces**; the service then answers on it only through the
  notification that follows. Measured: the first run of the fixture, before the
  fix below, failed at its first step for exactly that reason.

**Our response:** the listeners, the responders and the IPv6 joins already let go
of a network whose index is no longer reported and bind one that is new. What was
missing was the thread that answers discovery beside the service: it waits on the
responders and cannot read the routing socket the service already reads (two
readers take each other's messages). The responders now say when they move
(`crate::told_of_a_move`, one byte into a pair of sockets), and that thread waits
on it beside them and takes the responders again. Nothing wakes on an interval.
**Date:** 2026-09-16.

### The Linux kernel — an IPv6 membership outlives a deleted interface on the socket that joined it, and a join there then says `EADDRINUSE`
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`); measured on 2026-09-16 by
`crates/alo-agentd/src/two_machines_with_no_ipv4_find_each_other_again.rs`.
**Behaviour:** RFC 3493 says `IPV6_JOIN_GROUP` joins a group on an interface, and
nothing about what becomes of the membership when the interface goes. On a cable
carrying link-local IPv6 only, between two machines each running the service, with
a probe socket of the studio's own joined to `ff02::fb` beside the service's:

- **A link set down keeps the membership, on the socket and on the interface.**
  With the studio's end set down, a second join from the probe at that interface's
  number answers `EADDRINUSE`, and `/proc/net/igmp6` still lists the interface in
  `ff02::fb`. Reception's end, whose carrier went, stops being reported running,
  so the service no longer counts it among its networks. Set up again, both ends
  come back at the same index and the same link-local address.
- **A link deleted takes the interface's membership and leaves the socket's.**
  With the cable deleted, `/proc/net/igmp6` lists `ff02::fb` on no interface at
  that number — and a join from the probe at that number **still answers
  `EADDRINUSE`**, because the kernel checks the socket's own list of memberships
  before it looks for the interface. The socket holds it until it leaves it or
  closes.
- **So an interface given that number again is not in the group, and a join says
  it is.** `ip link add … index N` in a namespace gives a re-laid cable the number
  it had. Measured: with `crate::joining` reading `EADDRINUSE` as *already joined*
  and not leaving a network that went, the re-laid cable was counted joined on both
  machines and reception never found the studio.
- **A cable re-laid without a number is a new index at each end and a new
  link-local address** (a `veth` takes a random MAC), and a TCP connection to the
  old scoped address is refused by the kernel before anything is sent.

**Our response:** `crate::joining` leaves `ff02::fb` on a network that is no longer
reported, and a join refused `EADDRINUSE` is left and joined again rather than
assumed; with both removed the fixture fails at its last step, and with either one
alone it passes. A proposal is never dialled at a kept address: the machine is
looked for at the moment (`crate::looking`), so after a re-lay it is found and
dialled with the new interface.
**Date:** 2026-09-16.

### The Linux kernel — an IPv4 membership also outlives a deleted interface on the socket that joined it, while a socket held to the number keeps working
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`), procps `kill`; measured on 2026-09-16
by `crates/alo-agentd/src/a_cable_re_laid_between_two_readings.rs`.
**Behaviour:** RFC 3376 and `ip(7)` say `IP_ADD_MEMBERSHIP` joins a group on an
interface, and nothing about the membership when the interface goes. The IPv6 half
of this is the entry below; over IPv4, on a `veth` whose far end holds a probe
socket joined to `224.0.0.251` at its own address:

- **A link deleted and laid again at the same number (`ip link add … index N`) is
  not in the group, and the socket still says it is.** `/proc/net/igmp` lists no
  `224.0.0.251` on the re-laid interface, and a second join from the probe at that
  address answers `EADDRINUSE` — the kernel matches the socket's own list of
  memberships by the interface's number before it asks whether the interface is in
  the group. Leaving and joining again puts the interface in the group.
- **A datagram socket and a TCP listener held to that number with
  `SO_BINDTOIFINDEX` keep working on the re-laid interface.** The binding is a
  number compared with the interface a packet arrived on; reception's listener,
  never replaced, was reached on the re-laid cable, and a responder's socket
  answers there once its membership is taken afresh.
- **The kernel says so on the routing socket even when nobody reads in time.**
  With reception held still by `SIGSTOP` across the deletion and re-laying, a
  dump of the interfaces taken when it goes on shows only the same number under
  the same name, and the `RTM_DELLINK` queued meanwhile is what the service
  finds out from: the fixture passes on that message alone.
- **The kernel counts a group's users per interface, not per socket.** A socket
  leaving a group — or closing, which leaves every group it joined — takes one
  user off whatever interface has the number now. A socket that held a dead
  membership and closes *after* a new socket joined the re-laid interface would
  take that membership away; leaving the dead one before the new join does not.
  This one is read from `net/ipv4/igmp.c` (`ip_mc_leave_group`,
  `ip_mc_drop_socket`) and not measured on its own; the unit test
  `responding::tests::a_join_refused_as_already_held_is_taken_afresh_and_a_responder_let_go_of_leaves`
  measures that a responder let go of leaves the group at once, with its socket
  still open.

**Our response:** `crate::interfaces_that_went` reads `RTM_DELLINK` out of what the
service already reads, and `crate::responding` lets go of a responder whose
interface went even where its number is reported again — leaving its group at that
moment, before a new socket joins — and never reads `EADDRINUSE` as joined. With
the reading removed, the fixture fails at *reception never followed its cable to
40 … the interface is not in the discovery group*; with `EADDRINUSE` read as
joined, it fails at the probe. The listeners are unchanged. `crate::joining` follows
the same reading over IPv6. Where another process on the same machine had joined
the same group on the re-laid interface, leaving the dead membership takes one user
off theirs; nothing on alo OS joins `224.0.0.251` but this service, so it is
written down rather than worked around.
**Date:** 2026-09-16.

### The Linux kernel — an adapter laid again with its hardware address is, in a dump, the interface that went
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`), procps `kill`; measured on 2026-09-17
by `crates/alo-agentd/src/a_link_local_cable_re_laid_with_its_hardware_address.rs`.
**Behaviour:** RFC 4862 derives a link-local address from the interface identifier
and says nothing about an interface that goes and comes back. On a `veth` with no
IPv4 address, laid with `ip link add … index N address M … peer … index P address
Q`, deleted, and laid again with the same arguments while the machine at one end is
held still with `SIGSTOP`:

- **Everything a dump reads comes back the same.** The number, the name, the
  hardware address and the `fe80::` address (the default `addr_gen_mode`, EUI-64)
  are equal at both ends before and after; a service reading `RTM_GETLINK` and
  `RTM_GETADDR` dumps alone cannot tell the two interfaces apart. The only thing
  that says one went is the `RTM_DELLINK` queued on the routing socket meanwhile.
- **Duplicate address detection runs in a namespace whose only process is
  stopped.** Both link-local addresses stop being `tentative` before the stopped
  service is let go, so the first dump it reads already has the address usable —
  nothing in it arrives later to move the network.
- **The membership is the IPv6 one `two_machines_with_no_ipv4_find_each_other_again.rs`
  measured**: with the service's
  socket still holding `ff02::fb` at that number, `/proc/<pid>/net/igmp6` lists no
  group on the re-laid interface.

**Our response:** `crate::joining` leaves and joins afresh every network whose
interface the kernel said was deleted, whether or not it is reported again
(`crate::interfaces_that_went`). With that reading removed, the fixture fails at
*reception never followed its cable to 40: joined at `40`, and the interface is not
in the discovery group* — the service counting itself joined on an interface nobody
on the cable can reach it through.
**Date:** 2026-09-17.

### The Linux kernel — a routing socket nobody reads overflows quickly, says so once, and drops everything after, deletions included
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip -batch`, `veth`), procps `kill`; measured on
2026-09-17 by `crates/alo-agentd/src/a_machine_that_missed_what_the_kernel_said.rs`.
**Behaviour:** `netlink(7)` says a socket whose receive buffer is full gets
`ENOBUFS` and that messages are lost; it does not say how soon, or how a test can
tell it happened. With the service's routing socket (`RTMGRP_LINK |
RTMGRP_IPV4_IFADDR | RTMGRP_IPV6_IFADDR`, default receive buffer) in a process held
still with `SIGSTOP`:

- **One batch of 64 `veth` pairs made and deleted in its network overflows it.**
  The kernel counted 164 messages dropped after that batch.
- **The kernel counts the drops per socket, where a test can read them.** The
  `Drops` column of `/proc/<pid>/net/netlink`, on the row whose inode is the
  socket's (`/proc/<pid>/fd`) and whose `Groups` is `00000111`. It was `0` before
  the burst.
- **Everything after the overflow is dropped too, while nobody reads**, including
  `RTM_DELLINK` for a cable that is really gone and the `RTM_NEWADDR` that says a
  re-laid link-local address finished duplicate address detection. Deleting and
  re-laying two cables raised the count from 164 to 180.
- **What the reader then gets is one `ENOBUFS` and whatever was queued before the
  overflow.** The service said its log line for dropped messages exactly once.
- **Multicast memberships behave as with nothing dropped.** Neither re-laid
  interface was in `224.0.0.251` or `ff02::fb`, although the service's sockets
  still held memberships at both numbers.

**Our response:** `crate::interfaces_that_went::Went::lost` is *any interface may
have gone*: every responder is let go of and made again and every IPv6 join is
taken afresh, and `crate::joining` says `DROPPED` in the service log once. There is
no larger receive buffer: a buffer only moves the size of the burst that overflows
it. With `lost` read as nothing having gone, the fixture fails at *reception never
followed its cables … 40 in the IPv4 group: false; 44 in the IPv6 group: false*.
**Date:** 2026-09-17.

### The Linux kernel — a multicast group joined "anywhere" is joined on one interface, and a question to the group leaves by the default route unless it is sent from an interface's own address
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3 (`unshare`,
`nsenter`), iproute2 6.19.0 (`ip`, `veth`), rustix 1.1.4; measured on 2026-09-15 by
`crates/alo-agentd/tests/a_machine_on_two_networks.rs`.
**Behaviour:** RFC 6762 says a responder answers on every link it is on and
says nothing about how a process arranges that. Four things are true of Linux
that decided how `alo-agentd` does:

- **`IP_ADD_MEMBERSHIP` with `INADDR_ANY` joins one interface**, the one the
  route to the group picks — which is what `std`'s `join_multicast_v4(group,
  UNSPECIFIED)` asks for. A machine on a wired network and on Wi-Fi was a member
  on one of them, and a question arriving on the other was dropped before any
  socket saw it. Measured: with the studio joined on the wired `veth` only, a
  question from reception on the second `veth` got no answer, while the same
  question on the first did. Joining again with each interface's own address
  (`crate::joining`) makes the machine a member on each; joining twice on one
  interface answers `EADDRINUSE`, which is read as *already joined*.
- **A datagram to a multicast group leaves by the interface that owns the
  socket's bound source address** when no `IP_MULTICAST_IF` is set — the
  kernel's route lookup has a documented special case for exactly this
  (`net/ipv4/route.c`, *"direct multicasts … via necessary interface without
  fiddling with IP_MULTICAST_IF"*). So `crate::looking` binds one socket per
  network to that network's address and needs no socket option `std` lacks.
  Measured: reception's answers on each `veth` came back from the studio's
  address on that same network. (That a socket bound to `0.0.0.0` sends by the
  default route instead is the kernel's documented behaviour, not measured here.)
- **`IP_MULTICAST_ALL` is on by default**, so a socket bound to `0.0.0.0:5353`
  receives the group's traffic on every interface where *any* socket on the
  machine has joined — another mDNS responder's membership can make this daemon
  hear a network it never joined. That is harmless (the answer is the same bytes
  on every network) and is why the test above runs with no other responder in
  its namespaces, so that what it measures is this daemon's own joins.
- **The kernel says when a network changes**, on a routing socket bound to
  `RTMGRP_LINK | RTMGRP_IPV4_IFADDR`, and a membership goes with its interface.
  Several notifications arrive for one change (the link, then its address,
  then the link running), and the first can come before the interface is
  joinable, so the service asks the interfaces again on each notification
  rather than reading its content, and the test asks again until the second
  network is joined.

**Our response:** discovery is joined on every interface that is up and running,
carries multicast, has an IPv4 address and is not loopback, read from the
kernel's routing messages (`crate::route_messages`, parsed without `unsafe`),
and joined again whenever the kernel says a network changed. The two-network
test makes its networks inside a user namespace with `unshare --map-root-user
--net`, so it touches nothing on the host and takes no kernel-wide lock. What a
second physical machine on a real office network hears is still owed to two
machines.

### The Linux kernel — an office with no route out can be made on a development machine, and `OutNoRoutes` counts every packet that tried to leave it, once each
**Version:** `6.18.33.2-microsoft-standard-WSL2`, util-linux 2.41.3
(`unshare`), iproute2 6.19.0 (`ip`), rustix 1.1.4 and ureq 3.4.0, measured on
2026-09-13 by
`crates/alo-asking/tests/an_office_that_cannot_connect_still_has_working_ai.rs`.
**Behaviour:** `docs/features.md` promises *the whole of it works with no
internet at all*, and the plan asked for that measured with the unreachability
enforced rather than assumed. Four things turned out to be true of making that
measurement here, and none of them is in a manual:

- **A network namespace cannot be made from Rust under this repository's
  rules.** rustix 1.1.4 marks its safe `thread::unshare` `#[deprecated]` — the
  call was never sound — and the replacement, `unshare_unsafe`, needs an
  `unsafe` block, which this repository does not add for a test. util-linux's
  `unshare --map-root-user --net` makes the namespace instead, `ip link set lo
  up` brings loopback up in it (a fresh namespace's `lo` is down, and
  `127.0.0.1` is not routable until it is up), and a child of the test binary
  started from inside inherits it. `--map-root-user` is there so the same
  command works as root and as an unprivileged user on a machine that allows
  user namespaces; this machine's supervisor is root.
- **The kernel counts a refused route, once per attempt, before a byte
  goes anywhere.** `/proc/net/snmp`'s `Ip: OutNoRoutes` (and `Ip6OutNoRoutes`
  in `/proc/net/snmp6`) is per namespace, and increments by exactly one when a
  TCP `connect` or a UDP `sendto` is refused with `ENETUNREACH` — which reaches
  Rust as `io::ErrorKind::NetworkUnreachable`. Measured: one stream and one
  datagram addressed at `192.0.2.1` from inside the namespace read `2`; five
  questions bound for a provider there read `5`; a working day of discovery,
  pairing and eight questions down the corridor read `0`. `ip route` inside
  the namespace prints nothing at all.
- **ureq 3.4.0 makes the count exact rather than a lower bound.** Its
  connector retries the next resolved address only on `ConnectionRefused`
  and a per-address timeout; every other socket error — `ENETUNREACH`
  included — is returned at once as `Error::Io` with the kind intact. So one
  attempt is one packet, and `alo_asking::openai::what_went_wrong` can read
  *no route* off the error and off nothing else.
- **A search waits out its patience.** `alo_nearby::Looking::found` keeps
  listening until its patience ends even after a machine has answered,
  because a search cannot know how many will; the day therefore takes the
  five seconds it was given to look, not the milliseconds the road takes.

**Our response:** the measurement runs where a namespace can be made — Linux,
with `unshare --net` permitted — and **fails rather than skips** anywhere else
on Linux, naming util-linux as what it needs, the way the boundary tests fail
on a machine without a boundary. On Windows the file is compiled out
(`#![cfg(target_os = "linux")]`), so a green Windows suite says nothing about
this promise and the report says so. The one production change is the
mapping: only `NetworkUnreachable` and `HostUnreachable` become
`WentWrong::NoWayThere`; a refused connection, a reset, a timeout and a name
that did not resolve stay what they were, held by
`a_far_end_that_was_reached_and_refused_is_not_said_to_be_out_of_reach`.
**Upstream:** nothing to report; none of the four is a defect.
**Date:** 2026-09-13

### `logind` will open a session for something that is not `pam_systemd`, and the refusal for anybody else has two different wordings
**Version:** measured three ways. `systemd` 257 (257.13-1.fc42) on **the pinned
base** — `quay.io/fedora/fedora-bootc:42`, run as the alo OS image built from it
under `podman --systemd=always` — on 2026-09-11; `systemd` 259 on Ubuntu 26.04
under WSL2 on 2026-09-10 (the measurement at the foot of ADR 0024) and again on
2026-09-11 through `alo-sessiond`'s own call rather than through `busctl`.

**Behaviour:** ADR 0024 turns on whether `org.freedesktop.login1.Manager.CreateSession`
can be called by something that is not `pam_systemd`. It can, and the boundary is
privilege:

| Caller | What came back |
|---|---|
| root, well-formed arguments, an implausible leader PID | `Leader PID is not valid` (Fedora 42 / systemd 257); `Invalid leader PID` (Ubuntu / systemd 259) |
| uid 1000, the same call | `Access denied` |

The first row is the finding: the call was **authorised** and only its contents
were rejected, on both systemds. The method is on the interface, there is no
policy rule against a privileged caller, and `CreateSessionWithPIDFD` is on the
pinned base's interface too (`uhsssssussbssta(sv)`), so there is a newer spelling
to move to without a decision to retake.

Two things differ between the two machines and neither is a difference in the
answer:

- **The sentence for the root case is not the same string.** systemd 257 says
  `Leader PID is not valid` and systemd 259 says `Invalid leader PID`. Anything
  that recognised the *wording* would be reading prose that upstream rewords.
- **The refusal for an unprivileged caller does not always come from `logind`.**
  Asked from uid 1000 with `busctl` on Fedora, `logind` itself answers `Access
  denied`. Asked from uid 65534 on Ubuntu through `alo-sessiond`, the **bus
  policy** refuses the call before `logind` sees it, and the message is
  `Rejected send message, 2 matched rules; type="method_call" …`. Completely
  different sentences, and the same D-Bus error name in both cases:
  `org.freedesktop.DBus.Error.AccessDenied`. `busctl` prints the name's friendly
  form, which is what made the two look alike in the first measurement.

**Our response:** `crates/alo-sessiond` carries the D-Bus **error name** beside
the sentence (`NotOpened::Refused { named, why }`) and decides on the name;
the sentence is for whoever is reading a service log. Its test
(`logind_answers_the_way_the_adr_measured`) asserts the name and never the
wording. Nothing here matches on the root case's message at all: the opener
passes its own real process id, so *leader PID is not valid* is a refusal only
the measurement ever sees.

The privileged half is deliberately **not** run by `cargo test`. A test that
were root would not be refused — it would open a real session on whoever's
machine was running the suite, with the test process as its leader — so it skips
itself and says so, and the privileged answer stays this by-hand measurement.
**Date:** 2026-09-11

### A service that runs as a person cannot make a control group of its own, and `%U` in a system unit is 0
**Version:** `systemd` 259 on Ubuntu 26.04, kernel `6.18.33.2`, measured
2026-09-04 by starting `alo-agentd.service` under a real systemd. The same two
failures had stopped the first booted image, on `systemd` 257 on Fedora 42.
**Behaviour:** two separate things, both of which end in `EACCES` and neither of
which is documented as a refusal anywhere a person building an image would look.

- **The unit's own control group belongs to root.** `alo-agentd` makes a cgroup
  subtree under its own (ADR 0015: a turn is a control group), and a service
  with `User=` still gets a cgroup directory owned by root — so `mkdir` inside
  it is refused. `Delegate=` is what changes it: systemd then chowns the unit's
  cgroup directory and its `cgroup.procs`, `cgroup.threads` and
  `cgroup.subtree_control` to the service's user. Measured: with it, the daemon
  makes `home`, makes it threaded *while its own process is still in the parent*,
  and moves in; `cgroup.subtree_control` stays empty, so no domain controller is
  enabled above a threaded subtree and nothing conflicts.
- **`%U` is not the user the service runs as.** `RuntimeDirectory=alo/%U` in a
  unit with `User=alo` expands to **`alo/0`** — `systemctl show` says so — because
  the specifier is resolved when the unit is loaded and `User=` is resolved when
  the process is forked. It makes a directory for root, silently, and the
  service stops on a directory it cannot make.

**Our response:** `image/usr/lib/systemd/system/alo-agentd.service` says
`Delegate=yes` and `RuntimeDirectory=alo/1000` with the number written out, and
`crates/alo-image` holds that number to the one `/etc/alo/agentd.toml` names —
because two files agreeing is a test here and a specifier that reads as the
person is not. `Delegate=yes` rather than a controller list: what is needed is
the hierarchy, and naming a controller would enable one where a threaded subtree
is about to be made.
**Upstream:** not reported; both are documented behaviour read the wrong way
round by us.
**Date:** 2026-09-04

### systemd-sysusers does not fail on a number somebody else has — it takes a different one, or their group
**Version:** `systemd` 257 on `quay.io/fedora/fedora-bootc:42`, found 2026-09-04
by building `image/Containerfile` for the first time and reading the log.
**Behaviour:** every example in this repository — the machine description
contract, `alo-agentd`'s own tests, ADR 0001's prose — uses **989** as the
agent's login and group. On the pinned base, gid 989 is **systemd-resolve's**.
`systemd-sysusers` did not refuse the file. It said this, and carried on:

```
Suggested group ID 989 for alo-agent already used.
Creating group 'alo-agent' with GID 977.
Suggested user ID 989 for alo-agent already used.
Creating user 'alo-agent' (alo OS agent) with UID 976 and GID 989.
```

Read carefully, that is alo OS's agent **put into the resolver's group** — and
the machine description would still have said `group = 989`, so the daemon and
the machine would have agreed on a number that meant somebody else. Nothing
would have failed. `alo-agentd` would have handed its socket to a group the
resolver is in.

Two things follow, and the second matters more than the first. **A `u` or `g`
line is a request, not a declaration**: sysusers falls back, and it falls back
by taking the *existing* group when the number it was asked for is one. And
**every number in the system range is somebody's eventually** — Fedora allocates
system logins downward from 999, so a number that is free on the base today is
one a base update can take.

**What we do about it.** The agent's login and group are **60989**, in the range
systemd's own uid documentation leaves allocated by nothing (60578–61183), so a
base update cannot move it. And `image/Containerfile` runs `systemd-sysusers` at
build time and then asserts the three numbers with `test`, so a fallback is a
build that fails with the number in front of somebody rather than an image that
ships. The person stays at 1000, which is the first ordinary user on any Linux
and the one number no system package will ever ask for.

### bpf-linker 0.11.0 — the LLVM it was built against must be the one rustc emits
**Version:** `bpf-linker` 0.11.0, `rustc` nightly, Ubuntu 26.04, found
2026-09-04 by building `crates/alo-bounding-kernel`.
**Behaviour:** `bpf-linker` does not link objects the way a linker does — it
reads the LLVM bitcode `rustc` produces and runs LLVM's own passes over it. So
the LLVM it was built against has to be the LLVM the compiler emits, and when it
is not, **nothing says so**. A programme with a single BPF map in it makes the
linker die of a segmentation fault:

```
error: linking with `bpf-linker` failed: signal: 11 (SIGSEGV)
  PLEASE submit a bug report to https://github.com/llvm/llvm-project/issues/
  1. Running pass "sroa<modify-cfg>" on function "file_open"
```

Every part of that message points somewhere else. It names LLVM's bug tracker,
so it reads as an LLVM bug; it names our own function, so it reads as our code;
and it names an optimisation pass, so it reads as an optimiser problem. The
actual cause is two version numbers that are never printed together. A
programme with no map in it links perfectly, which is what makes the first hour
of this go into the code rather than into the toolchain.
**Our response:** the version is pinned in the repository rather than left to
whichever nightly a machine has.
`crates/alo-bounding-kernel/rust-toolchain.toml` names the compiler, and
`crates/alo-bounding/build.rs` starts the nested build **in that directory** so
the file is what decides — naming a channel on the command line would silently
overrule it. Whoever builds alo OS needs a `bpf-linker` built against the LLVM
that compiler emits: `rustc +<channel> -vV` says which, and
`cargo install bpf-linker --no-default-features --features llvm-<n>` is how it is
built against it. `docs/autonomy/LOOP.md` has what that took on this machine.
**Date:** 2026-09-04

### ureq 3.4.0 — `send_json` puts a pretty-printed body on the wire
**Behaviour:** the request body is `serde_json::to_writer_pretty`-shaped —
indented, with a newline after every field — where the obvious assumption is the
compact form. It is documented nowhere either way. Observed on a real socket by
`alo-asking`'s stub, which reads what actually arrives.
**Our response:** nothing is configured, because nothing is wrong: a provider
parses either, and the few hundred extra bytes on a question that is already
kilobytes of somebody's text are not worth a hand-built body. What changed is
the **test**: `alo-asking` asserts on the request body *parsed* rather than on
its text, so it says *these three fields and nothing else* — which is the
promise worth keeping (nothing of the person's leaves except the question) and
is also the assertion that does not break the next time ureq changes its
whitespace.
**Since 2026-09-13:** `alo-asking` serialises the body itself, compactly, and
sends the bytes — because ADR 0031's proof down the corridor is made over the
exact bytes on the wire, and a body the client re-serialises on its own terms
is not those bytes. The test assertion stays parsed, for the reason above.
**Upstream:** not reported; it is not a defect.
**Date:** 2026-09-03

### Ollama — a question has two APIs, and 404 means the model rather than the address
**Version:** Ollama's documented HTTP API as of 2026-09-03. **Not observed
against a running Ollama on any machine**; `alo-models`' tests drive a stub on a
real socket, and a run against the real runtime is owed with the rest of the
hardware verification (`ROADMAP.md` carries it as the model stack's machine
half).
**Behaviour:** the runtime answers questions two ways — its own `/api/chat`, and
an OpenAI-compatible `/v1/chat/completions` that speaks the shape a hosted
provider does. They are not the same reply: the native one puts the answer at
`message.content` and the compatible one at `choices[0].message.content`. And on
either, a model the runtime does not hold comes back **404 on an endpoint that
exists**, which at the protocol level is indistinguishable from an address that
is wrong — the same ambiguity a hosted provider has, recorded below.
**Our response:** `ollama.rs` uses `/api/chat`, the runtime's own. ADR 0006 says
Ollama's API is not our API and that one file may know what it is; using the
surface that imitates somebody else's would make the adapter's shape a guess
about a provider rather than a fact about the runtime, and the two could drift
apart in a release without anything here noticing. The 404 becomes
`RuntimeError::NotInstalled`, which names what was needed rather than what to
fix, and a person who typed the endpoint wrongly reads *that model is not
installed* — wrong, and wrong in the direction that costs them the least,
because a runtime alo OS ships is not an address anybody typed.
**Upstream:** not reported; both are documented behaviour.
**Date:** 2026-09-03

### Ollama 0.34.0 — `ollama create` leaves the source weights in the store, and nothing references them
**Version:** Ollama 0.34.0, the runtime `image/Containerfile` pins, run inside
the image's own weights stage; measured 2026-09-11 by building the recipe and
reading the store on the image it produced.
**Behaviour:** `ollama create NAME -f Modelfile` with `FROM /weights.gguf` does
two things where the documentation describes one. It copies the source file into
the blob store under that file's own sha256 — the log says
`copying file sha256:8a83c7fb…` — and then, under `parsing GGUF` and
`verifying conversion`, it writes a **second** blob of exactly the same length
and a different digest. The manifest it writes names only the second:

```
layers: [ {model,  sha256:01ec9e67…, 2393231072},
          {template, sha256:79bcc381…, 94},
          {params,   sha256:901ce025…, 98} ]
```

`sha256-8a83c7fb…` — 2,393,231,072 bytes, the artefact `THE_MODELS_SHA256` pins
and the recipe verifies before anything reads it — stays in `blobs/`, referenced
by no manifest. So a store built this way is **twice the size of the model in
it**: 4.5 GiB for 2.23 GiB of weights, and the image is 8.38 GiB where the
recipe's own comment says *2.23 GiB, carried once*.

It cannot be cleaned up afterwards on a machine. The store lands under `/usr`,
which is read-only on a bootc system (ADR 0011), so the runtime's own pruning —
whatever it would do — can never reach it.
**Our response:** written down on 2026-09-11 and not worked around, because the
build that found it passed and task 33 changed no decision to make one pass.
**Decided on 2026-09-12 (task 34): the source blob is removed in the weights
stage, after the import.** Of the three ways out, importing another way would
mean writing the runtime's store layout by hand, which is a second
implementation of a rented engine's format and exactly the drift ADR 0011
exists to refuse; carrying it deliberately costs 2.23 GiB on every machine and
across every network it is installed over, for a file nothing will ever open.
Removing a file the runtime wrote is the image's business and changes nothing
the runtime does — and the runtime agrees the blob is unused: **at every start
it tries to delete it itself** (`total unused blobs removed: 1`, and on a
machine `couldn't remove file … permission denied` against `/usr`). The digest
check is untouched, since it ran on the file before the import read it. Three
things are asserted before the store leaves the stage: the runtime can still
`show` the model off what is left, there is exactly one manifest, and every
blob in the store is named by it. `crates/alo-image` reads both lines
(`TheWeights::drops_the_source`, `TheWeights::holds_the_store_to_its_manifest`)
and refuses a recipe without either. The rebuild is measured in
`docs/autonomy/updates/the-weights-carried-once-and-a-runtime-that-does-not-call-home.md`.
**Upstream:** not reported.
**Date:** 2026-09-11, decided 2026-09-12

### Ollama 0.34.0 — the runtime asks its publisher two questions before anybody asks it anything
**Version:** Ollama 0.34.0, the runtime `image/Containerfile` pins, started out
of the built alo OS image as `alo-model` with the image's own store; measured
2026-09-11, once with an ordinary network and once with none.
**Behaviour:** within eight milliseconds of starting, with no request made of it
and no model loaded, the runtime makes **two outbound HTTPS requests to
`ollama.com`**:

```
WARN model_show_cache.go:142 "model show cloud cache hydration failed"
  error="Get \"https://ollama.com:443/api/tags?ts=…\": …"
WARN model_recommendations.go:168 "model recommendations refresh failed"
  error="Get \"https://ollama.com/api/experimental/model-recommendations?ts=…\": …"
INFO model_recommendations.go:177 "model recommendations cache sleep scheduled"
  wait=4m37s consecutive_failures=1
```

and it keeps trying for as long as it is up — every ~4m37s while they fail, and
on a long schedule once one succeeds. Its own defaults name the destination:
`OLLAMA_REMOTES:[ollama.com]`, `OLLAMA_NO_CLOUD:false`. None of this is in the
serving documentation, and none of it is an inference call: it is a list and a
recommendations feed.

On a machine sold on the sentence *a working day produces zero inference egress,
measured at the network boundary*, the process holding the model reaching its
publisher on every start is the thing that sentence is about.
**Our response:** `alo-modeld.service` already refuses it, and this measurement
is why that line is not decoration. `IPAddressAllow=localhost` under
`IPAddressDeny=any` is a kernel-side filter on the service's own control group,
so these two requests do not fail politely on a machine — they do not leave. That
is the design task 32 argued for; what this entry adds is that **the thing it was
guarding against has now been watched happening**, rather than supposed.

Two honest limits, as written on 2026-09-11: the filter itself was **not** what
refused the requests in that measurement — `--network=none` on a container was
— and the runtime's own switch, `OLLAMA_NO_CLOUD`, was a documented setting the
unit did not set.

**Both closed on 2026-09-12 (task 34).** The unit was started by the image's
own systemd — under a container, not at a boot; the entry below says exactly
what that took — with the image's own store and a working network, and the
filter was watched at two counters. Inside the container, an `nftables` rule on
the output hook counted **16 packets** to port 443 from uid 60991, the model
service's login: the two requests, and the kernel's SYN retries, over the three
seconds the runtime waits. On the host side of the container's bridge, a rule on
the forward hook counted **0** packets from the container to port 443 in the same
window. A control request from an unfiltered process in the same container —
`curl https://quay.io/`, a host the build already talks to — put 19 packets
through the same host counter and was answered `200`. So the requests were made,
the network was there, and nothing left: that is `IPAddressDeny=any` refusing,
not an absent network. What the runtime logs in that state is worth knowing,
because it is not what a refused connection usually reads like:

```
"model show cloud cache hydration failed"
  error="Get \"https://ollama.com:443/api/tags?ts=…\": context deadline exceeded"
```

A cgroup egress filter drops the packet after the socket has sent it, so the
connect does not fail — it times out. `context deadline exceeded` from this unit
is the filter working, and `lookup ollama.com: Temporary failure in name
resolution` (which the same measurement produced first, with a resolver the
filter also refused) is the filter working one step earlier.

And `OLLAMA_NO_CLOUD=1` is now set in the unit, as a second lock beside the
filter rather than instead of it. Measured on the same container: with it the
runtime logs `Ollama cloud disabled: true`, makes **neither** request, schedules
no retry (`consecutive_failures=0` with nothing attempted), and the journal
carries no line naming the publisher. `crates/alo-image` holds the line beside
the two filter lines and refuses a unit that dropped it or set it to anything
but `1`. Still not a boot; `docs/autonomy/v0-01-evidence.md` keeps *arrives
ready to run* owed.
**Upstream:** not reported.
**Date:** 2026-09-11, measured under systemd 2026-09-12

### Ollama 0.34.0 — the runtime states no digest anywhere structured, and the one it prints is the GGUF's own
**Version:** Ollama 0.34.0, the release `image/Containerfile` pins, running on an
Apple M3, 2026-09-20.

**Why it was asked.** `data/catalogue.toml` pins a `sha256` for each artefact
somebody other than the publisher made, and rule 6 sells that pin as *the file
we graded and the file a machine fetches are the same file or the fetch fails*.
Nothing compared it with anything. Before a check could be written, one question
had to be answered against the real program: **does the runtime expose, for a
model pulled from `hf.co/…`, the `sha256` of that GGUF file?**

**Behaviour, and it is two findings.**

**There is no structured field.** `/api/show` answers with `capabilities`,
`details`, `model_info`, `modelfile`, `modified_at`, `parameters`, `template`
and `tensors`, and **not one of them states a digest**. What carries it is
`modelfile` — the text `ollama show --modelfile` prints — whose `FROM` line
names the blob on disk:

```text
FROM /Users/…/.ollama/models/blobs/sha256-2e8040ce…68c2d
```

**And that blob's name is the GGUF's own `sha256`.** Measured on one real pull
of `hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M` (105 MB), three ways that
agree:

| Asked of | Answer |
|---|---|
| the registry manifest's `application/vnd.ollama.image.model` layer | `2e8040ce…68c2d` |
| the manifest the runtime wrote on this disk | `2e8040ce…68c2d` |
| `sha256sum` of the blob file itself | `2e8040ce…68c2d` |

The runtime prints `verifying sha256 digest` during the pull, so it checks the
bytes against that digest itself; what it does not do is let anybody ask whether
the digest is the one they wanted.

**It carries to the entries that matter, and that cost nothing to check.** Both
catalogue entries that state a pin were compared with their registry manifests —
a few kilobytes each, no weights fetched:

| Entry | Catalogue `sha256` | The manifest's model layer |
|---|---|---|
| `eurollm-9b-instruct` | `785a3b28…806b` | `785a3b28…806b` |
| `teuken-7b-instruct` | `03fd13da…630b` | `03fd13da…630b` |

**Our response:** `src/ollama.rs` asks `/api/show` after a pull and refuses when
the digest is not the pinned one, naming both. Reading a **path out of generated
text** is a thin place to stand, so the reader is strict — sixty-four lowercase
hexadecimal characters after `sha256-`, on a line beginning `FROM `, or nothing
— and nothing is a **refusal** rather than a skip. If a later release stops
printing that line, the pin stops being checkable loudly instead of quietly,
which is the failure this whole entry exists to avoid.

**What the check cannot do:** refuse before the download. It is of what arrived,
so a re-pointed tag costs the bytes and then fails. Checking the registry
manifest first would refuse sooner, and would be checking the registry's promise
rather than the machine's goods — the same shape as a recipe that tests a file's
executable bit and calls the program working.
**Date:** 2026-09-20.


### A bootc image inspected as a container has no `/root`, and the error says `file exists`
**Version:** `quay.io/fedora/fedora-bootc:42`, the base `image/Containerfile`
pins; found 2026-09-11 running the built alo OS image under `podman run`.
**Behaviour:** `/root` on an ostree-derived base is a **symbolic link to
`var/roothome`**, and `/var/roothome` does not exist until a machine boots and
`tmpfiles` makes it. A program that creates its own state directory under `$HOME`
therefore walks into a dangling link, and Go's `MkdirAll` reports it as:

```
Error: could not create directory mkdir /root: file exists
```

which reads like a permissions bug in the program, or like an image that shipped
something where a directory should be. It is neither, and it happens only when
the image is inspected as a container rather than booted.
**Our response:** nothing in the image changes. `alo-modeld.service` never goes
near it — `User=alo-model` with `StateDirectory=alo-model` and
`HOME=/var/lib/alo-model` — and started that way inside the image, as that login,
the same runtime comes up and lists the model the machine arrived with. This is
written down because the failure costs twenty minutes and points at the wrong
file, and because inspecting the image under `podman run` is what anybody will do
next.
**Date:** 2026-09-11

### The image's own systemd starts under podman, and what it takes to start a `User=` service there is what a machine's init already has
**Version:** podman 5.7.0 on Ubuntu 26.04 under WSL2 (kernel
6.18.33.2-microsoft-standard-WSL2), running the built alo OS image
(`quay.io/fedora/fedora-bootc:42`, systemd 257) with `/sbin/init` as its
process; measured 2026-09-12 while watching `alo-modeld.service`'s filter.
**Behaviour:** `podman run --systemd=always … /sbin/init` boots the image's own
systemd in a container without `--privileged`: every unit of ours is enabled and
attempted, and the machine reads `degraded` only because `alo-boundaryd` finds no
`/sys/fs/bpf` to pin to, which is the expected answer for a container. But the
model service — `User=alo-model`, an empty `CapabilityBoundingSet=`, no
capability asked for anywhere — **fails before it runs**:

```
alo-modeld.service: Failed to keep CAP_SYS_ADMIN: Operation not permitted
alo-modeld.service: Failed at step USER spawning /usr/bin/ollama: Operation not permitted
```

That is not the unit wanting a capability. It is systemd's own bookkeeping while
it changes user and applies the unit's sandbox, which needs the *init* to hold
`CAP_SYS_ADMIN` for the moment before it drops everything for the child — and
podman's default container init holds only the eleven capabilities an ordinary
container gets, none of which is that one. Three additions to the container's
init — `--cap-add=SYS_ADMIN,BPF,NET_ADMIN`, the first for the user switch, the
other two for `IPAddressDeny=` to attach its BPF programme to the unit's control
group — and the service starts as `alo-model` with `CapEff`, `CapBnd` and
`CapAmb` all `0000000000000000` and `NoNewPrivs: 1`, exactly as the unit says.
On a machine, PID 1 holds every capability, so what the container's init was
given is a strict subset of what a booted image's init already has; **the unit
was not changed and holds nothing.** Two smaller things cost time on the way:
`systemd-resolved` runs inside the container and podman writes the host's
resolver into `/etc/resolv.conf` rather than the stub, so a `User=` service
under `IPAddressAllow=localhost` cannot resolve anything — `--dns=127.0.0.53`
plus `resolvectl dns eth0 <upstream>` and `resolvectl default-route eth0 yes`
gives the container the shape a machine has, where the filtered process asks
the stub on loopback and `resolved` does the upstream query from its own
control group. And name resolution from the runtime then leaves no packet at
all from the unit's login: glibc's `nss-resolve` talks to `resolved` over a
Unix socket, which is why a counter on loopback for that uid reads zero while
the name resolved perfectly.
**Our response:** used as the measurement rig, and reported for what it is —
the image's own systemd, its own units, its own store and its own logins,
**not a boot**: no firmware, no disk, no `bootc install`. It is enough to watch a
unit's filter refuse a request and not enough to tick anything under *On the
machine*. Nothing in the image changed to make it start.
**Upstream:** not reported; podman's default capability set and systemd's need
for `CAP_SYS_ADMIN` around a user switch are both documented behaviour.
**Date:** 2026-09-12

### A model runtime's door is a TCP port, and a TCP port has no owner and no mode
**Version:** Ollama 0.34.0, the runtime `image/Containerfile` pins; systemd 257
(257.13-1.fc42) on the pinned base.
**Behaviour:** every door alo OS had decided who may knock at before this one was
a Unix socket — `/run/alo/<uid>/…` is 0750 and the agent's group (ADR 0017),
`/run/alo-sessiond` is 0750 and the greeter's (ADR 0024) — and both are decided
by a `Group=` line and a mode, because the filesystem carries an owner and a mode
for a socket and the kernel checks them on `connect(2)`. The model runtime's door
is not a Unix socket: `OLLAMA_HOST` is a host and a port and the server listens
with `net.Listen("tcp", …)`, so **there is nothing to own and nothing to chmod**.
No directive in a systemd unit restricts which local uids may connect to a
listening TCP port; `IPAddressAllow=`/`IPAddressDeny=` filter by address, and
every process on the machine connects from the same one.

The mistake this is written down to prevent is reading `Group=alo-model` in
`alo-modeld.service` as the sentence the other two units' `Group=` lines are.
It says who **answers**. It does not say, and cannot say, who may **ask**.
**Our response:** the unit decides everything a unit can decide — a login and a
group of its own, no capability and both lines saying so, the store the weights
landed in, the one loopback address `crates/alo-models` knocks at, and
`IPAddressDeny=any` under an allow list naming this machine alone, which is a
kernel-side filter on the service's own control group and is what makes *it
reaches nothing off this machine* enforced rather than asserted.
`crates/alo-image` checks each of those and deliberately does not check who may
connect, saying so beside the check.
`docs/decisions/0027-who-may-ask-the-model-anything.md` is where the gap is
argued and what closing it would cost is priced: a door of ours in front of the
runtime, a shared network namespace, or a rule in the boundary the machine
already loads. **No engine was patched to add a Unix socket** — that would be a
source change to a rented component, which ADR 0011 refuses without an ADR of
its own, and it is not the cheapest of the three anyway.
**Upstream:** not reported; TCP-only serving is documented behaviour.
**Date:** 2026-09-11

### Ollama 0.33.3 — one library model's manifest cannot be pulled, and the error is `EOF`
**Version:** Ollama 0.33.3, the runtime installed on the box every grade in
`data/catalogue.toml` was made on. The image pins 0.34.0
(`image/Containerfile`), which this was not tried against.
**Behaviour:** `ollama pull granite3.3:2b` prints `pulling manifest` and then
fails with `Error: EOF`, repeatably, on a machine whose network is fine —
`granite3.2:2b`, `qwen3:1.7b` and `hermes3:3b` all pulled from the same registry
minutes either side of it. Asking for the explicit tag
`granite3.3:2b-instruct-q4_K_M` answers `file does not exist`, so the failure is
not a typo in the name. Fetched by hand, `granite3.3:2b`'s manifest differs from
its neighbours' in one visible way: its model layer carries a `"from"` key
naming a path on the machine that published it
(`/Users/ollama/.ollama/models/blobs/…`), which the others do not. That is a
plausible cause and **not a confirmed one** — nothing here read the client's
source.
**Our response:** the candidate this was wanted for was catalogued at
`granite3.2:2b` instead, which is the IBM release before it and pulls normally.
That is a smaller change than it looks: the entry names the artefact it was
measured against (`artefact` in `data/catalogue.toml`), so what a machine
fetches and what earned the grade cannot drift apart. **No client was patched
and no version was moved** — engines are configured, never patched, and a
runtime that will not serve one model is a reason to measure another rather
than a reason to fork Ollama.
**Upstream:** not reported.
**Date:** 2026-09-11

### bootc 1.15.1 — `install to-disk` from outside its own container runs six more programs than it names
**Version:** bootc 1.15.1, bootupd 0.2.31, podman 5.x and skopeo 1.22.2, all
from the pinned base `quay.io/fedora/fedora-bootc:42@sha256:077182…`.
**Behaviour:** `bootc install to-disk --help` says it *must be invoked inside of
the container, which will be installed*, and offers `--source-imgref` for the
other case. The boot environment is that other case — an initramfs with no
container store and too little memory to hold a 6.6 GB image — and with
`--source-imgref registry:…` the tool does pull straight onto the new disk.
What it needs of the system it runs on is not written anywhere, and each gap is
found only after the disk has been partitioned, one error at a time. Booted in a
virtual machine on 2026-09-15, in order:

1. `Failed to find ostree/prepare-root.conf in /usr/lib or /etc` — it reads the
   **host's** `/usr/lib/ostree/prepare-root.conf`, not the image's;
2. `Creating imgstorage: Initializing images: No such file or directory`, then
   `could not find a working conmon binary`, then `could not find "netavark"` —
   it initialises a container store on the new disk by running `podman`, which
   needs `conmon`, `crun` and `netavark` even to list images;
3. `Creating importer: skopeo spawn error: No such file or directory` with
   `skopeo` present — the pull runs `skopeo` through `setpriv --reuid nobody`,
   so `setpriv` and a name service that knows `nobody` (systemd's, in an
   initramfs whose `/etc/passwd` holds only root) are both needed;
4. after twelve minutes of pulling, `Installing bootloader: Probing bootupd
   --filesystem support: No such file or directory` — the boot loader's
   installer is run inside the new deployment through `bwrap`.

**Our response:** `image/installing/alo-installing.conf` lists every one of them
for the base's own dracut; nothing was patched. The list is what was measured,
not what was read in a source tree, and a future base that needs another
program will say so the same way.
**Upstream:** not reported; `--source-imgref` outside a container is documented
as supported and its requirements are not.
**Date:** 2026-09-15

### cosign 3.1.3 — a signature is a referrer, not a `.sig` tag
**Version:** cosign 3.1.3, as the owner signed release 0.0.1 with
`--use-signing-config=false --tlog-upload=false`; `ghcr.io`, 2026-09-15.
**Behaviour:** the registry holds no `sha256-<digest>.sig` tag for the release
(the manifest request answers 404). The signature is a Sigstore bundle
(`application/vnd.dev.sigstore.bundle.v0.3+json`) attached as an OCI referrer,
listed under the fallback tag `sha256-<digest>`. The `sigstoreSigned` policy in
`containers-policy.json(5)` — the way podman, skopeo and `bootc
--enforce-container-sigpolicy` verify — reads the `.sig` attachment, so it has
nothing to verify here. Fedora 42 does not package cosign.
**Our response:** the boot environment carries upstream's own cosign 3.1.3
binary, pinned by sha256 and checked before the environment is built
(`image/installing/Containerfile`), runs the same `cosign verify` as
`docs/booting.md`, and then pulls by the same digest. Container signature
policy is left as the base ships it.
**Upstream:** not reported; the bundle format is cosign 3's documented default.
**Date:** 2026-09-15

<!--
### <Engine> <version> — <one-line summary>
**Behaviour:** what it does, versus what is documented
**Our response:** the configuration we apply, and why
**Upstream:** issue link if reported
**Date:** YYYY-MM-DD
-->

## Filesystems and paths

A grant is over a place, and a path is only a name for one. Where the two come
apart, a capability check can be correct and still be wrong — so this is where
that gets written down rather than discovered.

### A socket already open, and a datagram sent without connecting, are inside the boundary
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-12;
`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`
**Behaviour:** `socket_connect` decides when a socket is *joined* to a
destination, and a socket is joined once and written on many times. Two things
never passed it, and both were reproduced moving bytes past a bound turn's
boundary: a socket joined **before** the turn began — the network's version of
an inherited descriptor — and a datagram sent with `sendto` on a socket joined
to nothing, which makes no `connect` at all. A third, a proxy on loopback, is
below and is not this entry's.

**Our response:** a sixth hook, `socket_sendmsg`, which runs on every message
the machine sends and decides where the bytes are going. Three things about it
are worth an afternoon to whoever reads the code next:

- **It is asked of the sending thread's control group**, not the socket's. That
  is what closes the inherited case rather than restating it: a socket the
  daemon opened outside any turn is, at the moment a turn writes on it, being
  used by the turn. A cgroup `skb` programme would have attributed those bytes
  to the daemon's cgroup, because a socket remembers the cgroup it was *made*
  in, and would have closed nothing here — which is why an LSM hook and not a
  second kind of programme.
- **A message has up to two destinations, and both are checked.** The address
  it names (`msg_name`, which `sendto` fills and `send` and `write` leave null)
  and the peer the socket is joined to (`skc_daddr`, `skc_v6_daddr` and
  `skc_dport` on the `struct sock`). The kernel picks which the bytes follow by
  protocol — a stream socket ignores the name, a datagram socket uses it — and
  a programme that guessed the protocol would be one somebody could arrange to
  guess wrong. A message on a network socket that names nothing and is joined
  to nobody has no destination the programme can read and is refused, as is one
  whose named address is of a family the programme cannot read: `AF_UNSPEC` on
  a datagram is read by the kernel as an IPv4 address, and here it is a
  destination that cannot be checked rather than one that is not egress.
- **The peer's fields are reached through a named member.** `struct sock`
  keeps everything about its peer inside `__sk_common`, and inside that the
  address and the port sit in unnamed unions holding unnamed structures. The
  loader's type-information reader now follows a dotted path through *named*
  members and adds the offsets up; the fixture puts `__sk_common` eight bytes
  in rather than first, where the real kernel keeps it, so the addition is
  measured rather than passing because every part of it was zero. Several of
  the six offsets are genuinely zero on this kernel — `skc_daddr` opens
  `sock_common`, `msg_name` opens `msghdr` — and the map is an array, so zero
  is read as zero and not as *missing*.

What it keeps: loopback exempt for ADR 0007's reason and with the same cost, a
family that is not a network address allowed because it is not egress — which
is what lets the daemon go on answering the person on its Unix socket from
inside a turn — and *decides and forgets*, with the same two maps and nothing
written down; `the_boundary_decides_and_forgets.rs` now sends datagrams outside
a turn beside its opens. What it closes beyond the two named: a connection kept
open past the withdrawal of its destination, which the connect hook could not
re-check and this refuses on the next message.

Measured on this kernel: an inherited connection to a destination nobody showed
is refused `EACCES` at the write and the server hears nothing; the same to a
destination the turn *was* shown carries on; a datagram to a destination nobody
showed is refused and nothing arrives, and the same datagram from a process
that is not a turn arrives; a datagram to a shown destination goes; a datagram
to loopback goes unshown; the proxy on loopback still carries a turn out; a
Unix socket is connected to and written on. Nothing reaches a network in any of
them. `alo-egress`'s accounting is untouched — a refusal here is `EACCES` from
the kernel and never an egress event — and on the production path a refused
message reaches the record through the same `ureq` error the refused connect
does, in the same words; `alo-asking`'s `openai.rs` holds that.

**What this does not close:** the loopback proxy, which is ADR 0021's. A file
descriptor opened before a turn began was the other half of the same fact and
was closed the same day by `file_permission`, the entry *A descriptor opened
before a turn began is decided about on every use* below. WSL is development
evidence and never certified-hardware acceptance.
**Date:** 2026-09-12

### Four hooks are not a filesystem: what a bound turn can still change
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-08 and
as it stands on 2026-09-13;
`crates/alo-bounding/tests/what_a_bound_turn_can_still_change.rs`
**Behaviour:** the boundary watches twenty-three hooks — `file_open`,
`file_permission`, `inode_rename`, `inode_unlink`, `inode_link`,
`inode_setattr`, `inode_setxattr`, `inode_removexattr`, `inode_set_acl`,
`inode_remove_acl`, `file_ioctl`, `inode_create`, `inode_mknod`,
`inode_mkdir`, `inode_rmdir`, `inode_symlink`, `inode_getattr`,
`inode_getxattr`, `inode_listxattr`, `inode_readlink`, `inode_get_acl`,
`socket_connect` and `socket_sendmsg` — and a filesystem has more verbs than
the twenty-one of those that are about one. The filesystem hooks were chosen for one property:
**none of the mutations they leave unwatched moves a byte of somebody's file
past a grant.** That is a narrower promise than *a turn cannot change anything
outside its bound*, and reading the second where the first is written is how
somebody audits this boundary and comes away believing more than it does.

So this is the list, each row run against the real loaded programme with a
refused open beside it proving the boundary was in force — and since
2026-09-13 the list is empty, because every row it held is a hook:

| Hook | What a bound turn can still do | Why no contents leave a grant | Release |
|---|---|---|---|

The heading is kept, because `docs/contracts/agent-verbs.md` and every report
since task 5 point at it, and because
`crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs` holds
this entry to the programme by name: an empty table under the heading is
accepted as the honest state, and the heading being gone is not. What the
table held, and when each row closed:

- **`inode_symlink`** — a turn could make a symbolic link in a folder somebody
  granted pointing at a file nobody did; a name is not contents, and opening
  through it is a `file_open` on the file it leads to, refused. Closed
  2026-09-13.
- **`inode_create`** — a turn could make a file in a folder nobody granted by
  opening with `O_CREAT`: the create was unwatched and the open that followed
  it was not, in that order, so the inode was made and the write refused, and
  what was left was an empty file with a name of the turn's choosing. Closed
  2026-09-13.
- **`inode_mknod`** — the same file made without opening it, so nothing
  refused anything; putting anything in it was an open, refused. Closed
  2026-09-13.
- **`inode_mkdir`** — a directory made in a place nobody granted; a directory
  holds no bytes of anybody's file. Closed 2026-09-13.
- **`inode_rmdir`** — an **empty** directory nobody granted, removed; one that
  was not empty needed its contents unlinked first, and `inode_unlink` was
  watched. Closed 2026-09-13.

The entry *What a turn makes is inside the grant* below has the five hooks,
the shape of each, and the measurement.

Two things are **not** on that list and belong beside it. **What is inside a
file already open** was not a hook at all until 2026-09-12: `file_open` decides
at the moment of opening and says nothing afterwards, so a file descriptor that
existed before the turn began stayed usable inside it. `file_permission` now
decides on every read and write — the entry *A descriptor opened before a turn
began is decided about on every use* below has the measurement, and what it
leaves is a mapping, which that entry names. The socket half was closed the
same day by `socket_sendmsg`, the entry above. And
**starting a program** is not a way round any of this: `execve` opens the file it
runs, `file_open` is watched, and a bound turn asking for `/bin/true` is refused
with `EACCES` like any other file outside its bound. That is a floor under law 2
rather than the law, which is `alo-capability`'s.

**Two rows left this table on 2026-09-12.** `inode_setattr` and
`inode_setxattr` were here — a turn could change the mode, owner, times and
extended attributes of a file nobody granted it, and was no better off for it
because the boundary decides by place — and the size half of the first was
the sharpest thing in the entry: `truncate(2)` reaches `inode_setattr` without
an open, so a bound turn could **empty** a file it was refused `open` on,
measured by hand on 2026-09-08 and not reproducible in Rust without a
descriptor. Both are refused now, with three hooks beside them, and the
truncation is reproduced through a descriptor opened before the turn began;
the entry *Attributes, ownership and size are inside the grant* below has the
measurement and what it leaves. **The last five left it on 2026-09-13**, and
the argument that had kept them — none moves a byte — is the argument that
had been made for attributes until the size broke it; the entry *What a turn
makes is inside the grant* says what each left behind instead.

**Our response:** every claim above is a test. `crates/alo-bounding-kernel/src/deciding.rs`
carries the list beside the code that decides, `crates/alo-bounding/src/lib.rs`
carries it where somebody auditing the crate reads, and
`crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs` holds
this table to the programme: a hook that appears in `kernel.rs` and is still
listed here as unwatched fails that test, as does a row with no release, a row
nobody reproduced, a release `docs/features.md` has never heard of, or a hook
that arrives with no document naming it. So the list cannot rot into a
description of a boundary this one stopped being — in either direction.
**Date:** 2026-09-08; the table emptied 2026-09-13

### Attributes, ownership and size are inside the grant
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-12;
`crates/alo-bounding/tests/the_kernel_refuses_an_attribute_change.rs`
**Behaviour:** until 2026-09-12 the boundary decided what a turn could open,
read, write, move, remove and link, and nothing about what a file *is*. A bound
turn could change the mode, owner, times and extended attributes of a file it
was refused `open` on — and could **empty** it, because `truncate(2)` reaches
`inode_setattr` without an open. That last one was measured by hand and
written up in the entry above; it could not be reproduced in Rust, because
`std` has no path truncate and `rustix` has only `ftruncate` on a descriptor.

**Five hooks close it, and they ask one question.** `inode_setattr` is a
file's size, mode, owner and times; `inode_setxattr` and `inode_removexattr`
are an extended attribute set and taken away; `inode_set_acl` and
`inode_remove_acl` are a POSIX access list set and taken away. Every one of
them is handed the directory entry of the file being changed and walks up from
it exactly as `inode_unlink` does — `decide_attribute` in
`crates/alo-bounding-kernel/src/deciding.rs` is the one function all five
call — so a change to a file inside the grant goes, and one outside it is
`EACCES` at the syscall before the attribute has moved. Outside a turn each is
one hash lookup and a miss, and `the_boundary_decides_and_forgets.rs` makes
every one of these changes outside a turn beside its opens and still finds
nothing written down.

**Three things the next reader should know, each worth an afternoon:**

- **The entry is the second argument, not the first.** Since Linux 6.9 every
  attribute hook begins with the mount's identity mapping —
  `inode_setattr(struct mnt_idmap *, struct dentry *, struct iattr *)` — so
  the entry is `arg(1)` and the previous module's decision comes after the
  hook's own arguments as it does everywhere else: `arg(3)` for `setattr`,
  `removexattr` and `remove_acl`, `arg(4)` for `set_acl`, `arg(6)` for
  `setxattr`. Read from this kernel's own BTF (`bpf_lsm_inode_setattr` and its
  siblings) rather than from a header, because the rename hook's trap in this
  file is what reading the wrong pointer looks like: everything refused, for
  no reason anybody can see.
- **An access list is not an extended attribute to the kernel.** The call is
  `setxattr` and the name begins `system.posix_acl`, but since Linux 6.2 the
  kernel takes it to `inode_set_acl` and `inode_remove_acl` before
  `inode_setxattr` is ever reached. A boundary that watched only the
  extended-attribute hooks would refuse `chmod` and allow the same change
  spelled as a list. Measured here with a hand-made list — version two, five
  entries, forty-four bytes — set on `tmpfs`, which accepts one.
- **The size is reached through a descriptor opened before the turn began**,
  the way the daemon's own descriptors are, and `ftruncate` on it inside the
  turn is `inode_setattr` on the file's own entry — the same call `truncate(2)`
  makes, and not a read or a write, so `file_permission` never sees it. That
  is how the truncation the entry above could only describe is in the
  committed suite: before the hook the file emptied; after it, `EACCES` and
  the file says what it said.

Measured on this kernel, every one with a refused `open` proving the boundary
was in force and the same change landing on a file inside the grant proving it
was not refusing everything, and every one made by a process that is not a
turn and refused nothing:

| Change | Outside the grant, before | Outside the grant, now | Inside the grant |
|---|---|---|---|
| size, through a descriptor opened before the turn (`ftruncate`) | the file emptied | `EACCES`, and the file holds what it held | emptied |
| a rewrite (`open` with `O_TRUNC`) | `EACCES` at the open, as always | `EACCES` at the open, as always | the open goes and so does the truncation it carries |
| mode (`chmod`) | changed | `EACCES`, mode unchanged | changed |
| owner (`chown`) | changed | `EACCES`, owner unchanged | changed |
| times (`utimensat`) | changed | `EACCES`, times unchanged | changed |
| an extended attribute set (`setxattr`) | set | `EACCES`, not there | set |
| an extended attribute taken away (`removexattr`) | gone | `EACCES`, still there | gone |
| an access list set (`setxattr` on `system.posix_acl_access`) | set | `EACCES`, not there | set |
| an access list taken away | gone | `EACCES`, still there | gone |
| **inode flags** (`ioctl` with `FS_IOC_SETFLAGS`, through a descriptor opened before the turn) | `nodump` set | `nodump` set until 2026-09-13; `EACCES` since, flags undisturbed — the entry *A file's inode flags are inside the grant* below | set |

**What this did not close, for one day.** A file's **flags** — `chattr`'s
`nodump`, `noatime`, `append-only` and `immutable` — are set with an `ioctl`
on a descriptor, which is `file_ioctl` and not a change to an inode by name,
so none of the five hooks sees it. This entry named it, reproduced it in the
same test file in the direction it behaved, and said a hook on every `ioctl`
on the machine was a decision about cost as much as a hook. The decision was
taken on 2026-09-13 and the entry *A file's inode flags are inside the grant*
below has the hook, the cost, and what the kernel had bounded on its own.

**Our response:** closed, and every row above is a test. `docs/contracts/agent-verbs.md`
no longer tells an adapter author that a bounded turn can change a file's
mode, owner and attributes, because it cannot. `alo-files` holds that a
refused truncation, `chmod`, `chown` or attribute reaches the record as the
one sentence every machine refusal gets, not five. WSL is development evidence
and never certified-hardware acceptance; nothing *on the machine* is ticked.
**Date:** 2026-09-12

### A file's inode flags are inside the grant
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-13;
`crates/alo-bounding/tests/the_kernel_refuses_an_attribute_change.rs`
**Behaviour:** until 2026-09-13 a bound turn holding a descriptor that was
open before the turn began could set `FS_IOC_SETFLAGS` on it, and the flag
landed on a file the same turn had been refused `open` on a moment earlier.
An `ioctl` is not a change to an inode by name, so none of the five attribute
hooks saw it, and `file_permission` sees reads and writes, which an `ioctl`
is neither of. The entry above reproduced it and left it standing.

**One hook closes it, and the hook reads the request before it reads
anything else.** `file_ioctl(struct file *, unsigned int cmd, unsigned long
arg)` — three arguments, the previous module's decision fourth, the file
first as `file_open` and `file_permission` have it — runs on every `ioctl`
on the machine, which is every terminal asked its size, every socket asked
its state and every device driven. So `decide_request` in
`crates/alo-bounding-kernel/src/deciding.rs` compares the request number
before it looks up a control group: `FS_IOC_SETFLAGS` (`0x40086602`), the
same request in its 32-bit width `FS_IOC32_SETFLAGS` (`0x40046602`), and
`FS_IOC_FSSETXATTR` (`0x401c5820`) are decided exactly as an open of the same
descriptor would be — the walk from the file's own entry — and every other
request is allowed before any map is read. A `TIOCGWINSZ` inside a turn costs
three comparisons and is never walked. Measured: a read of a file's flags
(`FS_IOC_GETFLAGS`) on a descriptor to a file outside the grant, inside a
bound turn, is answered.

**Three things the next reader should know:**

- **What the kernel bounded on its own is unchanged.** `append-only` and
  `immutable` need `CAP_LINUX_IMMUTABLE`, which `alo-agentd` does not hold,
  so before this hook what a turn could actually set was `nodump` and
  `noatime`, and `nodump` is what the test sets. Small, and it was still a
  row in the hardening table; it is not any more.
- **`FS_IOC_FSSETXATTR` is refused by the same arm and is not in the
  committed suite.** `rustix` has a safe `ioctl_setflags` and `ioctl_getflags`
  and no safe spelling of the `fsxattr` request; `rustix::ioctl::ioctl` is
  `unsafe`, and `unsafe` is forbidden outside the kernel package's one file.
  The request number is the kernel's own encoding of `_IOW('X', 32, struct
  fsxattr)` and was read from this machine's `linux/fs.h` and from
  `linux-raw-sys`, both of which agree; the arm is three constants in one
  `match`, so a test of one spelling is a test of the comparison. It is named
  here rather than assumed, the way the mapping is.
- **A 32-bit program's requests go somewhere else on this kernel.** Since
  Linux 6.8 a compat `ioctl` reaches `file_ioctl_compat`, a hook of its own
  that this boundary does not sit on; before 6.8 it reached `file_ioctl` with
  the 32-bit width, which is why that number is recognised. What bounds it is
  real: a turn is one thread of a 64-bit `alo-agentd`, a turn cannot start a
  program outside its grant (`execve` is a `file_open`), and every descriptor
  Rust's standard library opens is close-on-exec, so an inherited descriptor
  does not survive into a program a verb with a bug in it might start inside
  the grant. It is
  written down here rather than hooked, and a hook on it is one `#[lsm]`
  function calling the same `decide_request` the day it is wanted.

**Our response:** closed, and the row is a test — the reproduction that held
the gap open was run against the programme that morning and passed with the
flag landing, then flipped into the refusal it is now, beside a test that a
read of the same flags inside the same turn is let through unwalked.
`the_boundary_decides_and_forgets.rs` sets and clears a flag on every file of
its ordinary day and finds nothing written down; `a_turn_without_a_boundary_does_not_run.rs`
refuses a turn when the thirteenth pin is gone with no line of its loop
changed. WSL is development evidence and never certified-hardware acceptance.
**Date:** 2026-09-13

### What a turn makes is inside the grant
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-13;
`crates/alo-bounding/tests/the_kernel_refuses_what_a_turn_makes.rs`
**Behaviour:** until 2026-09-13 a bound turn could leave a name of its
choosing anywhere on the machine — an empty file, by an open with `O_CREAT`
that was refused *after* the inode was made, or by `mknod(2)`, which opens
nothing and was refused nothing; a directory; a symbolic link — and could
remove any empty directory. Every one was measured in
`what_a_bound_turn_can_still_change.rs` with a refused open beside it, and
the entry above kept them with the honest reason none moved a byte: putting
contents into any of them is an open, and an open is watched. It is the
argument that had been made for attributes until `truncate(2)` broke it, and
what it left was the same shape of remainder — nothing of somebody's
contents, all of somebody's filesystem, and a name a turn leaves outlives the
turn.

**Five hooks close it, and four of them are decided by the folder.**
`inode_create(struct inode *dir, struct dentry *dentry, umode_t mode)`,
`inode_mknod(struct inode *dir, struct dentry *dentry, umode_t mode, dev_t
dev)`, `inode_mkdir(struct inode *dir, struct dentry *dentry, umode_t mode)`
and `inode_symlink(struct inode *dir, struct dentry *dentry, const char
*old_name)` are each handed the directory entry for the name being made, and
that entry is **negative** — it names a place in a folder rather than a file,
and has no inode to be asked about. So `decide_making` in
`crates/alo-bounding-kernel/src/deciding.rs` reads the entry's parent and
walks upwards from the folder exactly as an open would, which is the answer
the rename hook already gives its destination for the same reason, and it is
the folder `alo_files::Reaching` already puts among a turn's places for
anything a verb creates — so the `O_CREAT` open that writing an archive is
lands inside the grant without any bound widening. `inode_rmdir(struct inode
*dir, struct dentry *dentry)` is the one that removes; the directory exists,
so it is decided by its own entry through `decide_delete`, as a file unlinked
is, and a grant over a single directory is a grant over removing it. The
previous module's decision sits after each hook's own arguments — third,
fourth, third, second and third — and none of the modes, the device number or
the link's target is read.

**Three things the next reader should know:**

- **The arguments were read from this kernel's BTF, not from a header.**
  `bpf_lsm_inode_create` and its four siblings, with a throwaway reader over
  `/sys/kernel/btf/vmlinux`, before a line of the programme was written: the
  entry is the second argument on all five and the folder's inode the first,
  which is `inode_unlink`'s shape and not `inode_link`'s. The rename hook's
  trap in this file is what guessing looks like.
- **A refused `O_CREAT` open looked the same before and after.** `EACCES`
  either way; what changed is whether the empty file is there afterwards. The
  test asserts the name's absence rather than the number, because the number
  alone would have passed on the morning the gap was open.
- **A link's target is not decided, and deliberately.** A turn may make a
  link inside its grant that points at a file outside it, and is no better off:
  opening through it is `file_open` on the file it reaches, and the walk starts
  there. The test makes exactly that link, allowed, and reads through it from
  inside the turn, refused — and reads through it from outside to prove the
  refusal is the boundary's and not a broken link's.

Measured on this kernel, every one with a refused `open` proving the boundary
was in force, the same thing made inside the grant and used, and every one
made by a process that is not a turn and refused nothing:

| Making | Outside the grant, before | Outside the grant, now | Inside the grant |
|---|---|---|---|
| a file, by `open(O_CREAT)` | `EACCES` at the open, and the empty file left behind | `EACCES`, and no file | made, and takes bytes |
| a file, by `mknod` | made | `EACCES`, and no file | made, and takes bytes |
| a directory | made | `EACCES`, and no directory | made, and takes a file |
| an empty directory removed | removed | `EACCES`, and still there | removed, and made again |
| a symbolic link | made | `EACCES`, and no link | made; reading through it to a file outside the grant is `EACCES` |

**Our response:** closed, and every row above is a test — the reproductions
that held the gap open were run against the programme at `16eba10` that
morning and passed with each name landing, then flipped into the refusals
they are now. `the_boundary_decides_and_forgets.rs` makes and removes a file
by opening, a file without opening, a directory with a file in it and a link,
twenty rounds over, outside any turn, and finds nothing written down;
`a_turn_without_a_boundary_does_not_run.rs` refuses a turn over each of the
five new pins with no line of its loop changed. What is left on the
filesystem is the mapping the entry below names. WSL is development evidence
and never certified-hardware acceptance.
**Date:** 2026-09-13

### What a turn reads about a file is inside the grant
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-13,
four hooks that morning and the fifth, `inode_get_acl`, the same day;
`crates/alo-bounding/tests/the_kernel_refuses_what_a_turn_reads_about_a_file.rs`
**Behaviour:** until 2026-09-13 every hook decided what a turn did *to* a
file, and none decided what it learned *about* one it could not open. Inside
a bound turn, `stat(2)` on a path outside the grant answered with its size,
owner, mode and times, and so did `fstat(2)` on a descriptor to such a file
that was open before the turn began; `getxattr(2)` returned the value of a
`user.*` extended attribute, which is somewhere a person's application keeps
bytes that are not the file's contents — a comment, an origin, a checksum;
`listxattr(2)` returned their names; and `readlink(2)` returned where a
symbolic link points. So a turn refused a folder's listing by
`file_permission` could still ask each name in it whether it existed and how
big it was. No byte of contents moved; what moved was what the machine knew
about files nobody granted, which is *context is offered, never watched*
failing by another road. Every one of the five was measured answered,
against the programme at `5836d9c`, before a hook was written.

**Four hooks close it, and the arguments were read from this kernel's BTF
first.** A throwaway reader over `/sys/kernel/btf/vmlinux`, checked against
four hooks whose shapes the programme already documents, printed:

| Hook | Arguments | What is walked from | Previous decision |
|---|---|---|---|
| `inode_getattr` | `(const struct path *path)` | the path's entry, one read in from `arg(0)` | `arg(1)` |
| `inode_getxattr` | `(struct dentry *dentry, const char *name)` | `arg(0)` | `arg(2)` |
| `inode_listxattr` | `(struct dentry *dentry)` | `arg(0)` | `arg(1)` |
| `inode_readlink` | `(struct dentry *dentry)` | `arg(0)` | `arg(1)` |
| `inode_get_acl` | `(struct mnt_idmap *idmap, struct dentry *dentry, const char *acl_name)` | `arg(1)` | `arg(3)` |

The last row is the fifth hook, added the same day by task 20 after the four
above found the read it decides; its shape was read from the same BTF on the
day it was written rather than copied from `inode_set_acl`, and the two
agree — the mapping *is* there for the access-list pair, where it is not for
`inode_getxattr` one row up.

Two of the first four are not what a reading of the attribute hooks predicts, and the
plan predicted one of them wrong: `inode_getxattr` carries **no mount
mapping** before the entry, where `inode_setxattr` and `inode_removexattr`
do. A programme written from `inode_setxattr`'s shape would have read a
`struct dentry *` as a `struct mnt_idmap *` and refused everything for
reasons nobody could see — the rename hook's trap, one entry along. And
`inode_getattr` is handed a `struct path` rather than an entry: the same
`f_path` an open reaches its entry through, on its own, so the entry is one
read further in through the `path.dentry` offset the map already holds.

`decide_asking` in `crates/alo-bounding-kernel/src/deciding.rs` is what
`inode_getattr` asks and `decide_question` what the other three ask; both
end in the walk every file hook makes, from the entry of the file being
asked about, because the file exists and a grant can be over a single file.
`decide_asking` steps aside from a socket and a pipe first, by the inode's
kind, for the reason `decide_use` does: neither holds contents of its own,
neither is a place a grant is over, and a copy in the standard library asks
the kind of both its ends before it moves a byte, so a refusal there would
break a verb copying a file inside its grant towards a process this service
already talks to.

**Three things the next reader should know:**

- **What it costs.** `inode_getattr` runs on every `stat`, `lstat`, `fstat`
  and `statx` on the machine, which is the busiest hook here after reads and
  writes; for a process that is not a turn it is one hash lookup and a miss,
  and for a turn it is the walk an open already pays. `inode_permission` is
  **not** hooked, deliberately: it runs on every component of every path the
  kernel resolves, so the walk would be paid for every open on the machine
  twice, and what it would add is refusing `access(2)`, which reveals only
  whether a name exists. A bound turn can still learn that one bit, and it
  is named here rather than closed.
- **A file's access list, read, was still answered for the rest of that
  day, and is closed — task 20.** Since Linux 6.2 a `getxattr` of
  `system.posix_acl_access` is routed to `inode_get_acl` and never reaches
  `inode_getxattr`, exactly as the write is routed to `inode_set_acl` past
  `inode_setxattr`; the programme with the four hooks on it did not sit on
  `inode_get_acl`, so a turn refused the names of a file's attributes was
  still answered its access list — measured, in what was then
  `a_files_access_list_is_not_yet_inside_the_grant`, in the direction it
  behaved: a bound turn read a five-entry list off a file it was refused
  `open` on. That reproduction was run against the programme at `6f72631`
  and passed in that direction before the fifth hook was written, and is
  `a_files_access_list_is_inside_the_grant` now: `EACCES` outside the
  grant, and inside it the forty-four bytes read back are the forty-four put.
  **An access list that says no more than the mode bits is not stored at
  all** — the kernel folds it into the mode and a read answers `ENODATA` —
  so both the reproduction and the ordinary day in
  `the_boundary_decides_and_forgets.rs` carry a list with a named user and a
  mask; the least list the kernel accepts would have asked the hook nothing.
- **`fstat` was a proof and is now a refusal.** `what_a_turn_inherits.rs`
  used `fstat` on an inherited handle to show it was still a handle after a
  refused write; that `fstat` reaches `inode_getattr` through the
  descriptor's own path and is refused now, so the proof is `fcntl`, which
  asks no hook this boundary sits on, and the `fstat` is measured refused in
  the new file beside the same `fstat` answered on a handle to a file inside
  the grant. What `alo-files` needs — `symlink_metadata` of every path it
  was given, and a file's size and link count through the descriptor it
  opened — is inside the grant by construction, because resolving happens
  outside the boundary and every resolved path is among the turn's places;
  it was measured answered before the hook was written and is asserted
  answered, with the right answer, on every run since.

Measured on this kernel, every one with a refused `open` proving the
boundary was in force, the same question answered inside the grant with the
right answer, and every one answered to a process that is not a turn:

| Question | Outside the grant, before | Outside the grant, now | Inside the grant |
|---|---|---|---|
| `lstat` by name | the size, mode, owner and times | `EACCES` | the size |
| `fstat` through a descriptor opened before the turn | the size | `EACCES` | the size |
| `getxattr` of `user.alo.origin` | the value | `EACCES` | the value |
| `listxattr` | the names | `EACCES` | the names, with the attribute among them |
| `readlink` | where the link points | `EACCES` | where the link points |
| `getxattr` of `system.posix_acl_access` | the list | the list until `inode_get_acl` landed the same day; `EACCES` since | the list that was put |

**Our response:** closed, and every row above is a test — the first five
reproductions were run against the programme at `5836d9c` that morning and
passed with each question answered, the sixth against the programme at
`6f72631` with the four hooks on it, and each was then flipped into the
refusal it is now. `the_boundary_decides_and_forgets.rs` asks every file of
its ordinary day its size by name and through a descriptor, an attribute's
value, its attributes' names, its access list and where a link beside it
points, outside any turn, and finds nothing written down;
`a_turn_without_a_boundary_does_not_run.rs` refuses a turn over each of the
five new pins with no line of its loop changed. What a bound turn can still
learn about a file outside its grant is one bit — whether a name exists, by
`access(2)` — named above with the reason `inode_permission` is not hooked.
WSL is development evidence and never certified-hardware acceptance.
**Date:** 2026-09-13


### A descriptor opened before a turn began is decided about on every use
**Version:** Linux 6.18.33.2, alo OS's own BPF LSM as loaded on 2026-09-12;
`crates/alo-bounding/tests/what_a_turn_inherits.rs`
**Behaviour:** until 2026-09-12 this entry was *A descriptor opened before a
turn began is inside no boundary*, and it was true. `file_open` decides at the
moment of opening and says nothing afterwards, and there was no hook on a read
or a write, so every file already open when a turn started stayed fully usable
inside it. A turn is one thread of `alo-agentd` and not a process of its own
(law 2 — `crates/alo-bounding/src/turns.rs` has the argument), so that meant the
daemon's whole descriptor table: the record `alo_keeping::Writing` holds open
for appending, the way out of a turn, the door and whoever is at it, standard
output and error. It was the one gap in this crate that moved contents past a
grant, and it was measured doing so: the same thread refused `open` on a
private key with `EACCES` and reading every byte of it through a descriptor
opened a moment earlier, then writing what it read into the folder somebody
*did* grant, where an `archive_folder` or a `move_file` would carry it onwards
and where the record would name only a granted path.

**Our response:** a seventh hook, `file_permission`, which runs on every read
and write on the machine — `read`, `write`, `sendfile`, `splice`, `getdents` —
and decides about the descriptor at the moment it is used. Four things about it
are worth an afternoon to whoever reads the code next:

- **It is asked of the thread doing the reading**, not of whoever opened the
  descriptor. That is what closes the inherited case rather than restating it,
  and it is the same reasoning `socket_sendmsg` uses one entry up: a descriptor
  the daemon opened outside any turn is, at the moment a turn reads through it,
  being used by the turn.
- **The walk is the one an open takes.** From the file's own directory entry
  upwards until a granted place is met or the top of the filesystem is, so a
  descriptor to a file *inside* the grant is untouched — read through, written
  through, listed through — and one to anywhere else is refused with `EACCES`
  before a byte has moved. A pipe, a terminal, a device, the cgroup filesystem
  and the record are all *anywhere else*: none is a place a grant is over, which
  is what `file_open` already answered for the same things opened by name.
- **A socket is left to the hook that can decide about it, and a pipe is left
  alone.** A socket is a file too and its directory entry meets no grant, so a
  hook that refused it by its place would refuse the daemon its answer to the
  person and a question its provider (ADR 0020). The hook reads the file's
  kind from the inode's mode — one more offset, `i_mode`, fourteen in the map
  now — and steps aside for a socket; `socket_sendmsg` then decides by where
  the bytes are going, and a Unix socket is not egress. A pipe is stepped
  aside from for the same reason: it holds no contents of its own, so nothing
  of a person's file is in it that a process outside the boundary did not put
  there. A terminal, a device and the cgroup filesystem are not stepped aside
  from, and are refused as they are by name.
- **It cost the turn its own way out, and that was the point.** Leaving a
  boundary was the turn's own write into `home/cgroup.threads` through a
  descriptor opened before the first turn ever ran — the gap, used on purpose.
  That write is refused now like any other, so a turn's thread cannot end its
  own boundary at all, and it is brought home by a thread of the service that
  was never in a turn, started beside it before it went in;
  `crates/alo-bounding/src/inside.rs` has the arrangement. Nothing is started:
  a thread is not a program, and `a_turn_is_this_thread.rs` still reads the
  crate's source and says so.

**Landlock was considered and is not the mechanism**, and the reason belongs
here because ADR 0013 names it as the filesystem primitive. Landlock decides at
`open`, as this boundary already did — its filesystem hooks are the open, the
path operations and truncation, and there is none on a read or a write — so a
descriptor opened before the restriction is exactly as usable after it, which
is the gap restated. And a Landlock ruleset is irrevocable for the thread it is
applied to, so a turn that is a thread of the daemon could never be released
from one: closing this with Landlock would have meant a turn that is a process,
which is a change to what a turn *is* and belongs in an ADR. The BPF LSM is
ADR 0015's mechanism, and it needed one hook more.

| What a turn inherits | What the boundary refuses it now | What it still permits | Reproduced in | Release |
|---|---|---|---|---|
| `a file open for reading` | the first byte: `read` fails with `EACCES` and nothing of the file reaches the folder the turn was granted, where a write is still allowed and writes nothing | the descriptor stays valid, and the same descriptor to a file **inside** the grant is read through exactly as it always was | `what_a_turn_inherits.rs` | v0.5 |
| `a file open for writing` | the write, before a byte lands: `EACCES`, and the file still says what it said; and since 2026-09-13 its size, because `fstat` asks `inode_getattr` through the descriptor's own path — measured in `the_kernel_refuses_what_a_turn_reads_about_a_file.rs` | `fcntl` on the descriptor, which asks no hook this boundary sits on — measured so that the refusal cannot be a stale handle; it was `fstat` until the `stat` hook arrived — and a write through a descriptor to a file inside the grant, which lands | `what_a_turn_inherits.rs` | v0.5 |
| `a file open for appending` | a line in the machine's own record from inside a turn, which no execution caused; the record is as the daemon left it, and opening it by name is refused as it always was | nothing about the record from inside a turn, and that is right: the service writes the record outside the turn, which is where `alo-turn`'s `carrying.rs` always wrote it | `what_a_turn_inherits.rs` | v0.5 |
| `a directory descriptor` | `openat` relative to it, which is an open and always was refused; and now `getdents` through it, so the names in a folder nobody granted are refused as well as its files | the handle stays valid, and a descriptor to a folder **inside** the grant lists as it always did — which is what a folder handle is for | `what_a_turn_inherits.rs` | v0.5 |
| `a socket already connected` | a message to a destination nobody showed, decided by `socket_sendmsg` on every write, as the entry above measures | a write or a read on a Unix socket — the daemon's door, and whoever is at it — which the read-and-write hook steps aside from by the file's kind, so answering the person from inside a turn is untouched | `what_a_bound_turn_can_still_reach.rs` | v0.5 |
| `a pipe` | nothing at the pipe itself, and on purpose: a pipe holds no contents of its own, so what comes through it a process outside the boundary put there, and what goes into it reaches a process this service already talks to — a Unix socket's reasoning, and it was found rather than designed, when every child-process test stopped hearing from its child | the read and the write, in both directions, measured with a pipe made before the turn began | `what_a_turn_inherits.rs` | v0.5 |
| `the way out of a turn` | the write of `0` into `home/cgroup.threads` through the descriptor the service holds it open with, so a verb with a bug in it cannot end its own boundary early; opening any `cgroup.threads` by name is refused as it always was | the turn still ends: a thread of the service that is not in a turn writes the turn's thread number into that file on its behalf, and `Turns::doing` returns | `what_a_turn_inherits.rs` | v0.5 |

**What this does not close, and why it is not in the committed suite.** A
mapping. `mmap` of a file is `mmap_file`, not a read: a file mapped into memory
is read by the processor rather than by a syscall, so a mapping of an inherited
descriptor made inside a turn is a way to its contents that `file_permission`
does not see. It is not hooked, and it is the one thing in this entry **not
reproduced**: `std` has no `mmap`, `rustix`'s is `unsafe`, so is every crate's
that wraps it, and `unsafe` is forbidden outside `alo-bounding-kernel`'s one
file — a rule rather than an oversight, the same one that kept `truncate(2)`
out of the suite until a descriptor opened before the turn reproduced it (the
entry *Attributes, ownership and size are inside the grant* above). A hook
nobody can show refusing is a hook nobody
can show working, so it was not added on belief. Whoever closes `mmap_file`
will have to measure it the way the truncation was measured, by hand and with
the measurement written here; the walk is the one `decide_use` already makes,
and the only new thing is the argument that an anonymous mapping — `file` is
null, and every allocation inside a turn is one — is not a file at all.

Measured on this kernel, every one through `Turns::doing` on the thread the
assertions are made from, with a refused open proving the boundary was in force
and an allowed open inside the grant proving it was not refusing everything:
the key refused at the read and nothing copied; a writable descriptor to the
key refused and the key undisturbed; the invoice's descriptor read and written
through inside the grant; the record refused a line and unchanged; the private
folder refused its listing and the granted folder listing; `/proc/self/fd/<n>`
refused for the key and allowed for the invoice; a pair of Unix sockets written
and read across from inside; `home/cgroup.threads` refused the byte that used
to end a turn, and the turn ending anyway.
`the_boundary_decides_and_forgets.rs` reads and writes its ordinary files
outside a turn beside its opens and still finds nothing written down. On the
production path a refusal here reaches the record through the same
`std::io::Error` a refused open does, in the same words — `alo-files`'
`failed.rs` holds that. `alo-boundaryd`'s capability set is unchanged and
`crates/alo-image` still holds it to two.
`crates/alo-bounding/tests/what_a_turn_inherits_is_written_down.rs` holds this
entry to the programme: it fails the day `file_permission` leaves `kernel.rs`,
the day `mmap_file` arrives while this entry still calls a mapping unwatched,
and the day a row loses its reproduction. Every row is **v0.5**, where
`docs/features.md` puts *for the length of one turn, everything outside the
grant is unreachable*; this is that sentence becoming true of a descriptor.
WSL is development evidence and never certified-hardware acceptance.
**Date:** 2026-09-12

### The device number `stat` reports is not the one the kernel keeps
**Version:** Linux, any; found 2026-09-04 while writing `crates/alo-bounding`.
**Behaviour:** `stat` reports a file's device in `st_dev`, and the kernel holds
the same device in `super_block->s_dev`. **They are different packings of the
same two numbers**, and nothing anywhere says so:

```
stat reports    minor & 0xff | major << 8 | (minor & ~0xff) << 12
the kernel has  major << 20 | minor
```

For an ordinary partition at major 8, minor 2, one is `0x802` and the other is
`0x800002`. A comparison between them does not fail loudly — it simply never
matches, so a boundary keyed on a device number would find every file to be
outside every grant while looking perfectly healthy, and the code doing it reads
like the obviously correct code.
**Our response:** the conversion is in one function,
`alo_bounding::as_the_kernel_keeps_it`, with the two packings written above it,
and it is the only place a device number crosses between the two. The test that
would catch a mistake in it is not its own unit test — it is
`a_turn_granted_a_folder_opens_a_file_inside_it`, because a wrong conversion
refuses the granted file rather than allowing an ungranted one, which is a
failure in the safe direction and therefore the failure nobody notices.
**Date:** 2026-09-04

### `Path::is_absolute` answers about the host, not about the path
**Version:** Rust 1.97 `std`, seen 2026-09-03 in `alo-saying` on Windows 11
26200 and Ubuntu under WSL2
**Behaviour:** `Path::new("/usr/share/alo/translations").is_absolute()` is
`true` on Linux and **`false` on Windows**, because a Windows absolute path
needs a drive or a UNC prefix and this one has neither. It is documented
behaviour and it is right — the question `std` answers is *would this resolve
without a working directory on the machine you are running on* — but it is not
the question a test about a path alo OS ships asks.
**Our response:** where a constant is a path on the machine alo OS runs on
rather than a path on the host the tests run on, the test is written against the
text (`starts_with("/usr/")`) and says so. Reaching for `is_absolute` gives a
test that passes on the loop's Linux half and fails on its Windows half, having
found nothing wrong with anything. The same caution applies to `Path::join`,
`parent` and `components` on any path this repository writes down for a machine
it does not run the tests on.
**Date:** 2026-09-03

### A PDF ends with `%%EOF`, and readers accept it anywhere in the last kilobyte
**Version:** PDF 1.7 (ISO 32000-1) §7.5.5 and PDF 2.0; the tolerance is Adobe's
published implementation note and every widely used reader's behaviour.
2026-09-14, `alo-opening`.
**Behaviour:** the specification puts `%%EOF` on the file's last line. Real
PDFs carry bytes after it — a trailing newline pair, padding from a mail
gateway, a signature appended by a scanner — and readers open them, because
Acrobat has always looked for the marker within the last 1024 bytes. The same
readers accept a `%PDF-` header that is not at offset zero, which the
specification also does not.
**Our response:** `alo-opening` calls a PDF damaged only when `%%EOF` is absent
from the last 1024 bytes — the rule that separates a download cut short from a
PDF with a tail. It does **not** accept a header past offset zero: a search for
`%PDF-` near the start would find it in a letter about PDFs and call the letter
a damaged document, and *damaged* sends a person back to whoever sent them the
file. A real PDF with bytes before its header is therefore *not recognised*
rather than opened; that is the refusal of the two that costs less.
**Date:** 2026-09-14

### An OpenDocument lists its macro libraries whether or not it has a macro
**Version:** OpenDocument 1.2/1.3 packages; 2026-09-14, `alo-opening`. Read
from the package layout, **not yet measured** against a suite's own output on a
certified machine — task 2 of the documents plan opens real files and is where
that measurement belongs.
**Behaviour:** a package keeps its macro libraries as `Basic/script-lc.xml`, a
`script-lb.xml` per library, and one file per module. The two listings can name
a library with no module in it — a document whose *Standard* library was created
and never written in. Reading *has a `Basic/` folder* as *carries macros* would
tell a person their plain letter has macros in it.
**Our response:** a macro is a finding only when the list of contents names a
module beside those two listings under `Basic/`, or a script under `Scripts/`,
with something in it. Nothing is decompressed to decide it. Where a kind keeps
its macros inside a stream this crate does not read — an older PowerPoint
presentation — `Macros::NoneSeen` says exactly that, rather than *none*.
**Date:** 2026-09-14

### A zip has nowhere to say which clock its timestamps came from
**Version:** the zip format as every reader implements it; seen 2026-09-02 in
`alo-files`, against Windows 11 26200's own reader
**Behaviour:** a zip keeps each file's time as a DOS date and time, which
carries no timezone and is **conventionally the local time of whoever wrote the
archive**. `std` cannot say what this machine's offset from UTC is, and every
crate that can does it either through a dependency whose local-offset lookup is
unsound in a threaded process or through code this repository forbids.
**Our response:** the moment written is **UTC**, consistently, and it is
documented where the archive is made rather than left to be discovered. A reader
on a machine two hours ahead of UTC shows a file archived at 20:04 as 18:04.
Seconds are also kept in two-second steps, which is the format and not us. The
alternative — a guessed offset, or a dependency to find the real one — would be
wrong more interestingly rather than less often.
**Date:** 2026-09-02

### Resolving a path does not defeat a hard link
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** `alo-files` resolves every path a verb names and asks the grants
about where it really leads, which stops a symbolic link out of a granted
folder. A **hard** link is not a link in that sense: it is a second real name
for the same file, so a hard link inside a granted folder to a file that also
lives outside it resolves to the granted name and passes the check.
**Our response:** nothing in the path layer, because there is nothing honest to
do there — the granted name genuinely is a real name for that file. Making a
hard link needs write access to the granted folder and read access to the
target, so it is not a way *in*; it is a way for somebody who can already write
to a granted folder to widen what an agent may read — and what an agent reads
can leave the machine, while the record of the turn names only a granted path.

**Since 2026-09-07 it is the policy this entry predicted**, and there was only
one place it could go: a file is asked **how many names it has**, of the open
handle rather than of the path, and more than one means it is not read.
`crates/alo-files/src/opening.rs` asks it, so both verbs that read bytes —
`read_file` and `archive_folder` — answer the same way about the same file. An
archive is refused rather than made without the file, which is `archiving.rs`'s
own rule about bounds. A person is told the file has other names, that it was
not read, and to copy it or grant the folder the other name is in.

**Nothing beneath it was going to catch this, and that was measured rather than
assumed.** `crates/alo-bounding/tests/a_hard_link_is_inside_every_boundary.rs`
binds a turn to one folder on a running kernel with the real programme loaded:
the private file under its own name is `EACCES`, and **the same file under its
second name inside the granted folder opens and is read**. That is the boundary
doing exactly what ADR 0015 describes — deciding by where a directory entry sits
— and this entry sits in a granted folder. So it is not a second answer to this
question, and the check above is the only one there is.

**The other end of it was closed on 2026-09-07**: the boundary now watches
`inode_link`, so a **turn cannot make** a hard link with a source or a
destination outside its bound. That does not replace the counting rule and the
sentence above still stands as written — a link made by somebody else, before
the turn began, is one no kernel hook can see the wrongness of, and refusing to
read a file with more than one name is what covers it. The two answer different
halves: one stops a turn creating the second name, the other stops a turn
reading through a second name somebody else created.

**What it costs, stated plainly.** A file cannot say *where* its other names
are; there is no way from a file to its names short of scanning every filesystem
it could be on. So *more than one name* is all that is known, and a file whose
second name sits harmlessly beside the first — two names in one granted folder —
is refused too. That is the safe direction rather than a free one, and it is a
real cost to whoever meets it.
`crates/alo-files/tests/a_file_with_another_name_is_not_read.rs` has a test that
says so out loud, rather than only testing the case that flatters the rule.

**Windows still cannot count.** `std` exposes no name count there, so the check
answers *not that we can tell* and a hard link on NTFS is not caught.
Directories are exempt everywhere: every directory has at least two names, its
own and the `.` inside it, so counting names on one would refuse every folder on
the machine.
**Date:** 2026-09-02, answered 2026-09-07

### A path checked and then opened by name can change in between
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** the real path is resolved, the grants permit it, and then the
file is opened by that name. Anything with write access to a folder on the way
can swap a link in between the two.
**Our response:** the check is where it can be, and the fix is not another
check. Whatever opens the file holds on to *what it opened* rather than
resolving the same name twice.

**On Linux this is closed for reads** (item 6b). `crates/alo-files/src/opening.rs`
opens with `openat2` and `RESOLVE_NO_SYMLINKS`, which is the kernel refusing a
link at **every** component of the path inside one syscall — not only the last,
which is all `O_NOFOLLOW` would give. There is no moment between the components
for a substitution to be made in, because the resolution is one syscall. A
kernel too old for `openat2` answers `ENOSYS` and the read is refused; nothing
asks the machine what it supports and quietly does it the old way.

Moving a name is closed **against collisions** by `renameat2` with
`RENAME_NOREPLACE`: one call that both refuses and moves, so *nothing is
replaced that was not named* is a property rather than a check with a gap after
it. A filesystem that does not implement the flag answers `EINVAL` and the move
is refused rather than done the replacing way.

**And the folders on the way to a rename are closed too** (item 6c).
`renameat2` has no `RESOLVE_NO_SYMLINKS`, so it takes **handles on the two
folders** instead — opened with `O_PATH` and `RESOLVE_NO_SYMLINKS`, which
reaches each folder by a path no component of which was a link and then holds it
rather than naming it. A folder exchanged after the grants said yes is therefore
not walked through, on the source side and the destination side alike. The final
component of each name is not followed, which is `renameat2`'s own behaviour and
the right one: a link put where the file was is moved as the link it is.

Why that is allowed to be done at all is the entry below, and it was measured
rather than assumed.

The first walk written for reads — open `/`, then each folder relative to the
handle before it — could not be used: every one of those opens is above what the
call named, and a bounded turn was refused its own granted file with `EACCES`.
`alo-agentd`'s `a_turn_is_bounded_by_the_kernel` caught it on a running kernel,
and since item 6c that test also carries out a granted **move** inside a real
boundary, so the same mistake cannot be made twice.
`crates/alo-files/tests/nothing_is_swapped_in_between.rs` holds all of it down.

**Windows and everything else keep what they had:** `File::open` and a check
before the rename, with the gap this entry is about. `std` has no better answer
there.
**Date:** 2026-09-02, extended 2026-09-02 by the acting half, closed on Linux
2026-09-07 by items 6b and 6c

### An `O_PATH` open is not on the `file_open` hook, and confers no reading
**Version:** Linux 6.18.33.2, measured 2026-09-07 with alo OS's own BPF LSM
loaded; `crates/alo-bounding/tests/what_an_o_path_handle_is.rs`
**Behaviour:** `O_PATH` produces a handle on a *place* rather than an open file.
Linux does not run `security_file_open` for one, so a boundary on that hook
never sees it — which reads like a way round a boundary and is why this was
measured rather than reasoned about. With a turn bound to one granted folder:

| | |
|---|---|
| `O_PATH` on an **ungranted** folder | opens |
| `O_PATH` on an ungranted file | opens |
| `openat` a file **through** that handle | `EACCES` |
| reopening it through `/proc/self/fd/N` | `EACCES` |
| `renameat2` between two `O_PATH` handles | works |
| a plain rename of an ungranted file | `EACCES`, since renames were enforced |

**Our response:** `alo-files` takes `O_PATH` handles on the two folders of a
rename, and nothing else does. The first two rows are what makes that possible;
**rows three and four are why it is not a hole**, and they hold for a structural
reason rather than a lucky one — the programme in `alo-bounding-kernel` walks
upwards from the *file's own* directory entry, so what decides is where a file
is and not which handle it was reached from. An `O_PATH` handle therefore
carries exactly the authority needed to move a name and none of the authority
the boundary exists to withhold, and **nothing about this widens what a turn may
reach**. The test asserts every row, so a kernel that changes any of them fails
loudly rather than downgrading a guarantee in silence.

**The last row was the finding, and it has since been fixed.** When this entry
was written a rename was on no hook at all — ADR 0015 named `inode_rename`
beside `file_open` and only `file_open` had been built — so moving a file nobody
granted was something the kernel did not stop. It is built now, and the row
above is what this kernel says today.

That changes nothing about the rest of this entry, which is why it is worth
saying: `alo-files` takes `O_PATH` handles because such a handle confers no
reading, and that argument never depended on renames being unwatched.
**Date:** 2026-09-07, last row answered the same day

### Windows returns a path spelled differently from the one it was given
**Version:** Windows 11 26200, Rust 1.97 `std::fs::canonicalize`
**Behaviour:** canonicalising `C:\Users\x\Temp\Invoices` gives
`\\?\C:\Users\x\Temp\Invoices`. The two are the same folder and compare as
different paths, component by component, because the verbatim prefix is a
component.
**Our response:** none in the comparison, which is right to be exact — a grant
that matched loosely would match more than the person picked. **A grant is made
over a resolved path**: the folder a person picks is resolved when they pick it,
so both sides of every later comparison are spelled the way the machine spells
them. Written into the contract and asserted in `alo-files`' integration test,
which grants a resolved folder for exactly this reason.
**Date:** 2026-09-02

## Clocks and moments

A record is evidence about when something happened, so what a moment means when
it is written down, read back and compared is this section's subject.

### `SystemTime` walks back past 1970, and how far is the platform's
**Version:** Rust 1.97 `std::time::SystemTime`, seen 2026-09-03 in
`alo-keeping` against Windows 11 26200
**Behaviour:** a retention rule is naturally written as *keep anything after
`now - 30 days`*, and `SystemTime::checked_sub` is the obvious way to say it.
On Windows a `SystemTime` is counted from 1601, so subtracting thirty days from
a machine whose clock says it is the first minute of 1970 answers with a moment
in **1969** rather than `None`. On a platform where the representation is a Unix
`timespec` the same call can answer `None` instead. Both are correct for the
type; they are not the same boundary.
**Our response:** the window is measured **from the epoch forwards**, not from
`now` backwards. `Keeping::oldest_kept` asks how far `now` is past the epoch,
subtracts the window from *that*, and answers `None` when it does not reach —
so a boundary before 1970 is *nothing is removed*, identically on every
platform. It matters because the case it covers is a machine whose clock is
wrong, and a wrong clock must never be a way to empty a record. The test that
says so is `a_wrong_clock_never_removes_more`, and it was the failing test that
found this.
**Date:** 2026-09-03

### A record is replaced while it is open for appending, and Windows allows it
**Version:** Rust 1.97 `std::fs::rename`, Windows 11 26200; seen 2026-09-03 in
`alo-keeping`
**Behaviour:** shortening a record writes the replacement beside the old file
and renames it over. On Windows that is `MoveFileEx` with
`MOVEFILE_REPLACE_EXISTING`, and replacing a file another handle has open is
the classic way to get *access is denied*. It succeeds here, because `std`
opens files with `FILE_SHARE_DELETE` among the share flags — which is `std`'s
choice rather than a documented guarantee of the platform.
**Our response:** the rename happens with the writer's own append handle still
open, and the handle is **reopened immediately afterwards** — an old handle
goes on writing into a file that is no longer the record, which is a lost entry
rather than an error. Shortening is therefore a method on the writer taking
`&mut self`, so nothing can append during it and nothing else is expected to be
holding the record open. If a filesystem ever refuses the replace, the answer
is to close the handle before renaming and not to copy over the old file in
place: nothing is removed until the replacement is whole on the disk.
**Date:** 2026-09-03

### A record whose folder has been removed goes on accepting writes
**Version:** Rust 1.97 `std::fs`, Windows 11 26200; seen 2026-09-03 in
`alo-turn`
**Behaviour:** a turn that cannot write down what it did stops doing anything
else, and the integration test for that wanted a real disk to refuse a real
write. Removing the folder the record lives in does not do it: on Windows
`remove_dir_all` **succeeds** with the record file open — `std` opens files
with `FILE_SHARE_DELETE` — and the open handle then goes on accepting writes
and syncing them, into a file no longer reachable by any name. The write does
not fail, so the turn never learns anything is wrong. There is no portable way
to make a filesystem refuse a write to a handle it has already given out.
**Our response:** the closing is tested against a `Kept` that refuses
everything (`alo-turn`'s
`a_turn_that_could_not_write_something_down_does_nothing_else`), and the
integration test asserts the half a real disk *can* answer: that every entry is
on the disk before the door that made it answers. The rest of it —
`NotKept::NotAddedTo` really arriving from a full or failing disk — is owed
with the hardware verification. It is the same share-flag behaviour as *a
record is replaced while it is open for appending* above, met from the other
side.
**Date:** 2026-09-03

## Languages and counting

A sentence with a number in it is the one string that cannot be translated
line for line. Where what a plural form is called and what it actually covers
come apart, write it here — because the person who would notice is the person
reading that language, and there is nobody here who reads all 24.

### A plural form's name says nothing about which numbers it covers
**Version:** CLDR cardinal rules, `common/supplemental/plurals.xml` from
`unicode-org/cldr`, read 2026-09-02. Not a disagreement with CLDR — CLDR is
right — but with what the names lead a reader to assume.
**Behaviour:** three assumptions that all look safe and are all wrong. **Every
language has `other`:** Polish does not, for a whole number — its `one`, `few`
and `many` cover every integer between them, and CLDR's Polish `other` has
decimal samples only. A file offering a Polish translator `one` and `other` asks
them for one sentence nothing will ever show and leaves out the two that most
numbers take. **`one` means one:** Croatian's `one` covers 1, 21, 31 and 101;
French's covers 0 as well as 1; Latvian's `zero` covers 0, 10, 11 and 20 alike.
A translation that spells the number out — *jedna datoteka* — is then shown to
somebody with twenty-one files. **A form is picked by the number:** it is picked
by the number *and the language*, so English's forms cannot be used to look up a
Polish sentence.
**Our response:** `alo-strings`' `cldr.rs` holds the rules as code with each
CLDR condition quoted beside the arm it became, and three things are refusals
rather than conventions. A translation into a form its own language never uses
is refused, naming the forms it does use. A form may leave the number out only
where `names_one_number` says exactly one whole number takes it. A countable
string translated into a language whose rules are not in the table is refused
outright, in words addressed to whoever is contributing that language — nothing
falls back to English's two forms, because a sentence wrong for most numbers in
a language nobody here reads is worse than one that has not arrived.
**Date:** 2026-09-02

### Half the keys on a keyboard are not printed with a word anybody translates
**Version:** physical keyboard layouts, EU national variants, observed 2026-09-02.
**Behaviour:** *what a key is called* looks like one list of strings and is two.
`Q`, `7`, `,` and `F1` are printed identically on every keyboard sold in the
union, and translating them is not translation at all — it names a **position**,
which is the model `alo-shortcuts` exists to reject, since `Super+Q` on a French
keyboard is the key marked Q and not the one where an English keyboard has Q. The
other sixteen print a *word*, and it is a different word almost everywhere: a
German keyboard prints **Entf** for Delete, **Einfg** for Insert, **Pos1** for
Home, **Strg** for Ctrl and **Bild ↑** for Page Up; a French one prints **Maj**
for Shift. A shortcuts panel translated from one English list would either name
keys that are not on the keyboard in front of the person, or invite a translator
to render `Q` as `Й`.
**Our response:** the two kinds are different questions in the code.
`Key::mark` answers for the fifty-three that print a mark and is not a string at
all; `Key::said` answers for the sixteen that print a word, each declared in
`alo-shortcuts`' `words` with a note naming what a keyboard in another country
prints; and `Key::shown` is what a panel draws for either. Declaring all
sixty-nine was the alternative and is worse twice over: it hands a translator
forty-one rows reading `A`, `B`, `C`, and it makes `Strings::unanswered` — *what
a release note has to count* — report fifty-three strings nobody should ever
translate.
**Date:** 2026-09-02

### A machine cannot punctuate a list it assembled
**Version:** Greek orthography; CLDR list patterns, `common/main/*.xml`, not
implemented here.
**Behaviour:** a sentence that names two or more things has to join them, and
the joining is not punctuation a program can pick. Greek writes `;` where
English writes a question mark and `·` where English writes a semicolon, so a
list joined with `"; "` reads as a row of questions to the people it is for. The
conjunction before the last item is a word — *and*, *und*, *et* — that would
have to be its own string, placed by a machine that does not know the sentence.
**Our response:** no sentence in this repository joins a list. Where one thing
is named it goes in a gap, as `alo-shortcuts`' *{chord} is already {action}*
does; where two or more are, the sentence says so and the things are handed over
to be drawn as rows — `Clash::said` names the chord and `Clash::actions` hands
over what wants it, each said in the reader's own language. If a sentence ever
genuinely needs a list inside it, the list patterns are CLDR data like the plural
rules and are read rather than recalled.
**Date:** 2026-09-02

### A deserialiser is required to have a sentence and has nobody to ask for one
**Version:** `serde` 1, `#[serde(try_from = "…")]`, observed 2026-09-02.
**Behaviour:** every value in this repository that a settings file holds is
checked again on the way in, because a settings file is a thing a person edits —
a colour, a screen's name, a rotation, a schedule, a text size, a time of day, a
key combination. `serde` implements that with `try_from`, and it requires the
error to have a `Display`, because it turns it into a message with
`de::Error::custom`. That is exactly the thing our rule forbids: a `Display` on a
user-facing refusal is an English sentence one `to_string()` away from a screen.
And the deserialiser is the one caller that genuinely cannot obey the rule the
other way either — it is handed a value and a format, never the language the
person in front of the machine reads, and there is no argument to give it one.
**Our response:** what a refusal writes at that point is the **key** of the
string rather than the string. `alo-appearance`'s `NotRead` is that, shared by
its six deserialisers, and `alo-shortcuts`' `Chord` has a private one of its own
from item 9c. Whoever reports a settings file that did not read looks the key up
and shows the same words a settings panel shows for the same refusal — one
rendering, in the reader's own language, rather than an English line in a log
beside a translated line on a screen. The refusal itself is unchanged: the same
files are refused as before, and `said(&Strings)` is still the only road to
words. What is given up is `std::error::Error` on ten types that were never
errors a programmer handles.
**Date:** 2026-09-02

### Two gaps in a translated sentence arrive in the language the code was written in
**Closed by item 9g on 2026-09-03.** Kept because the shape of the mistake is
worth recognising again, and because the fix cost a public surface change.
**Version:** `alo-capability` at item 9e, observed 2026-09-02.
**Behaviour:** a translated sentence is only as translated as what goes into
its gaps, and two here came from somewhere that had not moved.
`capability.call.missing` — *{verb} needs {argument} — {purpose}* — filled
`{purpose}` from what the verb was declared with, which is the source string
rather than the reader's; the crate that declared the verb had the translation,
and the crate that refuses the call did not, because a `Verb` carried the
declaration and not a key. `capability.answer.lapsed` quotes the approval
sentence, which `alo_capability::Call` rendered at the moment the call was made
and kept as a string. So a German machine could read a German sentence with an
English clause inside it, which is exactly the failure `alo-appearance` closed
for colour names in item 9d.
**Our response while it stood:** the note on each of those two words said so, in
the words a translator needs, so nobody spent an afternoon looking for the
string that would fix it. **What closed it:** item 9g. A verb is declared from
`alo_strings::Word`s, a `Call` carries the key of its sentence and the values
that fill it, and `CallError::Missing` carries the key of the argument's
purpose — so both gaps are looked up with the same vocabulary as the sentence
around them. It was never worked around, because working around it would have
meant a second copy of a declaration, and one string rather than two that agree
is the rule the whole 9-series is built on.
**Date:** 2026-09-02, closed 2026-09-03

### A translated error cannot be a `std::error::Error`
**Version:** Rust 1.x, `std::error::Error: Debug + Display`, met again at item
9f on 2026-09-02 and at item 9h on 2026-09-03.
**Behaviour:** `std::error::Error` requires `Display`, and `Display` takes no
argument but a formatter — so a type that can only say what it is when it is
handed the reader's language cannot implement it. Everything downstream of that
trait goes with it: `?` into a `Box<dyn Error>`, `#[from]`, `anyhow`, and the
`{e}` a programmer writes without thinking. It is the same collision the
deserialiser entry above describes, met from the other side, and item 9f is
where it reached a type in a **public trait's** signature — `ModelRuntime`
returns `RuntimeError`, and third parties implement `ModelRuntime`.
**Our response:** the types a person reads give up `Display` and answer
`said(&Strings)`, and the ones a programmer reads keep it. The line between
them is *who is holding the machine when this appears*: `CatalogueError`
refuses the catalogue this repository ships, `VerbError` refuses a verb
declaration, `alo-shortcuts`' `DefaultsError` refuses a release's own defaults —
all read by whoever is fixing the thing that failed, so all still English and
still `std::error::Error`. What an adapter author gives up is `?` into a boxed
error, and what they get is a refusal their user can read; `RuntimeError`'s own
documentation says so where they will look. Two doctests in `alo-models` had to
drop their `?` for this reason and were re-checked afterwards, because a
`compile_fail` that starts failing on a missing conversion has stopped testing
what it was written for.

Item 9h met it in the place where it costs the most and is still worth paying:
`alo_egress::NotPermitted` is what `Indicator::beginning` returns, so an egress
refusal no longer arrives as an `Error` a caller can `?` into a box. The person
holding the machine when that appears is the owner watching the indicator, so
the refusal gives up `Display` like the rest — and three doctests, one of them
in `alo-record`, dropped their `?`. The `compile_fail` beside them was
re-checked outside the doctest harness and still fails on **E0624, associated
function `new` is private**, which is what it was written to test.
**Date:** 2026-09-03

## Models

Open-weight models in the catalogue have their own personalities: refusing
formats they claim to emit, ignoring stated context limits, or answering in the
wrong language. Where a model in the catalogue misbehaves in a way that affects
the agents, record it here with the exact model and quantisation — "it was fine
for me" is usually a different quantisation.

### The weights a machine arrives with: carried, not fetched (2026-09-11)
**Version:** `image/Containerfile` as of 2026-09-11, against
`crates/alo-models/data/catalogue.toml` of the same day and the certified
machines in `docs/hardware.md`. The artefact is
`Phi-3-mini-4k-instruct-q4.gguf` from `microsoft/Phi-3-mini-4k-instruct-gguf` at
revision `a64113399c2f6b8ad3e11c394733a2ddadaa7f33`, **2,393,231,072 bytes**,
sha256 `8a83c7fb9049a9b2e92266fa7ad04933bb53aa1e85136b7b30f1b8000ff2edef`.
**Behaviour:** ADR 0025 left one thing open and asked for the answer here —
*whether the image carries the weights or fetches them at setup*. What decides
it is not a preference: **a machine that fetches at setup has not arrived ready
when it is offline at setup**, and the promise the ADR took on is about what is
in the box. The measurement that was owed beside it was *the smallest catalogued
model that clears the verb-driving bar*, and the honest answer is that **there is
no such model**: every entry anybody has put to `alo-driving` is graded `rarely`,
this one included, so no size of carried model makes the agent work today. What
carrying does buy is a machine that can load and answer with a model on its own
disk, offline, on arrival — which is the promise as written — and it buys it for
2.23 GiB of image and an update channel that moves those bytes whenever the pin
moves. The other answer costs the same bytes, paid by a person on their first
morning, in a place where nobody here can help them.

Which model is then decided by three catalogue facts and not by a preference:
measured by `alo-driving` at all, small enough for the ordinary business laptop
`docs/hardware.md` certifies first (16 GB, no card), and under a licence that
permits commercial use outright. That last one is easy to miss and is the sharp
one: **carrying weights in an image is redistributing them**, so a licence with
conditions — Gemma's terms, the Llama community licences — would attach those
conditions to everybody who receives a copy of alo OS. `phi-3-mini-instruct` is
MIT, and it is the largest entry that satisfies all three.
**Our response:** the recipe declares the model, the quantisation, the artefact,
the revision and the digest; the digest is checked in a step of its own **before
anything reads the file**, because weights are imported by the runtime rather
than unpacked by `tar` and a check afterwards would leave the wrong bytes in a
layer. `crates/alo-image/src/weights.rs` reads the declaration and
`everything_wrong_with` refuses each of those going wrong — including a model the
catalogue never heard of, one nobody measured, and one whose licence was not ours
to hand on. **Nothing here has been built**: no `docker build` of this recipe has
been run in this lane, and the import step in particular is a recipe rather than
a measurement until somebody builds it.
**Date:** 2026-09-11

### Two models trained for tool calls, put to the same ten requests
**Version:** `qwen3:1.7b` (Qwen3 1.7B, Q4_K_M, 1,359,279,776 bytes) and
`granite3.2:2b` (IBM Granite 3.2 2B Instruct, Q4_K_M, 1,545,296,256 bytes),
served by Ollama 0.33.3 on the 5,926 MB WSL2 guest with four CPUs — the box
every other grade here was made on. Two rounds of `alo_driving::THE_SET`,
twenty attempts each, 2026-09-11. They are `data/catalogue.toml`'s
`qwen3-1.7b` and `granite-3.2-2b-instruct`, and each entry names the artefact
above as the one its grade was earned against.
**Behaviour:** the five entries measured on 2026-09-04 are general chat models,
and all five failed at the *shape*. These two were chosen for the opposite
property: both publishers train them for **tool calls and structured output**,
Qwen3 with an agentic/function-calling mode and Granite with function calling
among its stated core capabilities. That is the hypothesis this run tested, and
**it did not hold at this size.**

| | `qwen3:1.7b` | `granite3.2:2b` |
|---|---|---|
| Drove | **3** of 20 | **1** of 20 |
| The door would not read it | 14 | 15 |
| A change through the read door | 2 | 2 |
| A verb nothing declares | 1 (`READ`) | 0 |
| A format alo OS does not have | 0 | 2 (`"format":2`) |
| Grade | `rarely` | `rarely` |

**They fail in different ways, and neither way is reasoning.** Qwen3 is the
only model measured here that has ever driven `list` twice and `read` once —
those three are the whole of its score — and the other seventeen are one of
three mistakes. It **drops the door**: `{"format":1,"asks":{"open_application":
{"application":"org.alo.Writer"}}}` puts the verb where `read` or `propose`
belongs, which is well-formed JSON that is not a message. It **shouts the
names**: `{"verb":"READ","given":[{"named":"FILE",…}]}` is the right shape with
the registry's identifiers upper-cased, and `alo-capability` matches exactly, so
it is `NoSuchVerb`. And once it **leaked a token of another language into the
structure** — `…"march.pdf"}]}}特に}` — which is a multilingual model's own
sampling arriving inside somebody's file operation.

Granite's single success is `find` in round two. Its failures are almost all
**punctuation**: braces one over or one short, a stray `"` after the closing
brace, `"given"` written as an object where the protocol has a list. Twice it
invented **`"format":2`** and wrote a message from a version of alo OS that does
not exist — `alo-protocol` answers `FromANewerAloOs { format: 2 }`, which is the
reader refusing a future it was told about rather than guessing, and it is the
first time any measured model has reached that branch.

**A third was measured and is deliberately not catalogued.** `hermes3:3b` —
Nous Research's Hermes 3 on Llama 3.2 3B, 2,019,373,888 bytes, whose Hugging
Face tags literally include *function calling* and *json mode*, which makes it
the strongest case for the hypothesis this run tested. It drove **0 of 20**,
inventing a field at every turn: `"reads":"folder"` beside `asks`,
`"parameters"` where the protocol has `given`, `"propose"` hoisted out of `asks`
to sit next to `format`, and a nested `""invoices.zip""`. It is left out of
`data/catalogue.toml` on rule 1 rather than on its grade: the publisher's own
metadata says `license: llama3` while the model's stated base is Llama **3.2**,
and those are two different Meta community licences with different version
lines. This catalogue states a licence read off the publisher, and the publisher
contradicts itself — so the entry cannot be written honestly, and picking the
licence we think they meant is exactly the harm rule 1 names. The grade is
recorded here so the run is not lost; the entry waits on Nous.

**Our response:** the two whose licences are unambiguous are catalogued, both
say `rarely`, and neither can be the agent. **Nothing in the method moved**: the prompt is the registry's as
`alo-driving` builds it, the scoring is `alo-protocol`'s reader and
`alo-capability`'s validation, the runtime's context window is the pinned
runtime's default, and the five-minute wait is `alo_models`'
`WHILE_A_MODEL_THINKS`. Qwen3 answers with its thinking enabled, which is what
the artefact does by default and therefore what a machine would get; turning it
off would have measured a different model. What the run settles is the useful
half: **the bar is not being missed for want of tool-call training**, so the
model task 10 waits on is not one more curated small entry, and the next
measurement worth making is a larger one on a machine with room.
**Date:** 2026-09-11.

### A 7B-class entry cannot be measured on the box every grade here was made on
**Version:** `mistral:7b-instruct-v0.3-q4_K_M` — Mistral AI's
Mistral-7B-Instruct-v0.3 at the quantisation `data/catalogue.toml` states, 4.4 GB
— served by Ollama 0.33.3 on the development box this lane runs on: a WSL2
Ubuntu guest of **4 CPUs and 5,926 MB of memory with 4 GB of swap**, which is
what `C:\Users\SBW\.wslconfig` gives it (`memory=6GB`, `processors=4`) on a
15.5 GB Windows host. 2026-09-11.
**Behaviour:** the plan that asked for this grade said the memory question was
answered — the host has 15.5 GB, so a 7B entry is runnable here. **It is not,
and the 15.5 GB is the wrong machine's number.** Every grade in the catalogue
was made inside the WSL guest, because that is where the pinned runtime is, and
the guest is capped at 6 GB by a configuration whose own comment says why: the
host pages about 20 GB as it stands, and WSL does not hand memory back. The
model was fetched and run there anyway, twice, and what happened is arithmetic
rather than bad luck:

- **It loads, and the load alone outlasts the wait.** 284.7 s the first time,
  447 s the second — against `alo_models`' `WHILE_A_MODEL_THINKS`, which is five
  minutes. So `alo-driving`'s own warm-up question fails with
  `RuntimeError::TookTooLong` before an exercise is ever put, which is the first
  run's whole result.
- **Loaded, it is 5.0 GB in a guest of 5.9 GB**, so it runs against swap:
  `llama-server` held 4.3 GB resident with 1.25 GB paged out and 464 MB of the
  guest left, and took 1.5 of the 4 cores because it was waiting on paging
  rather than on arithmetic.
- **At that speed the fixed set cannot be put.** With the model already warm,
  the harness's twelve-token warm-up took **101.7 s** — 0.41 tokens per second
  reading it, **0.25 tokens per second writing** twenty-one tokens back. The
  first exercise's prompt is **715 tokens** (2,607 characters, the ten verbs as
  the registry declares them); its first 512-token chunk took **138.32 s** at
  3.70 tokens per second, and the call passed five minutes with three tokens
  written. `TookTooLong` again, on the `list` exercise, and the run stopped.
- **And the run cost the machine the guest.** Between the two attempts the WSL
  VM went down — `uptime` back to zero, the runtime gone with it — while the
  host had about 700 MB of physical memory free. A measurement that can take a
  shared checkout's build down with it is not a measurement this box can be
  asked for.

**Our response:** `teuken-7b-instruct`, `mistral-7b-instruct` and
`qwen2.5-7b-instruct` stay `not-measured`, which is the true sentence about
them: nobody has run the measurement, and this box cannot. Nothing was
loosened to get a number out of it — not the prompt, not the scoring, not the
runtime's context window, and not the five-minute wait, which is the constant
that keeps a slow machine from being reported as a model that cannot answer.
Three ways out were considered and rejected, and they are written down because
the next worker will reach them too: **raising the guest's memory** needs
`wsl --shutdown`, which `docs/autonomy/SHARED_MAIN.md` forbids without an idle
handoff from both loops, and a 15.5 GB host that already pages cannot afford
12 GB inside a VM; **the runtime on the Windows side** is a package install on
a shared machine, a second 4.4 GB copy of the weights against a C: drive with
12.4 GB free and a 12 GiB floor, and a host with 700 MB of memory to spare; and
**a smaller quantisation than the entry states** would be a measurement of
different weights. What would settle it is a machine with room — 16 GB to the
runtime and more than four cores — and the run is a download and an hour, not a
purchase, once there is one.
**Date:** 2026-09-11.

**Since, on a machine with room — 2026-09-13.** Two of the three have been
measured, and the third has a reason instead. First `qwen2.5-7b-instruct` was graded on an **Apple M3 with 8 GB of unified
memory**, Ollama 0.34.0 (the pinned runtime) serving the weights on the GPU, and
earned `rarely`, 4 of 10. Not 16 GB, and it did not need it: with the Linux VM
stopped the model loaded and the fixed ten took thirty-three seconds, which is
the difference between memory the runtime can page against and memory it
cannot. The same prompt, the same scoring, the same five-minute wait.
Then `mistral-7b-instruct`, on the same Apple M3 the same evening: 0 of 10,
which is `rarely`. `teuken-7b-instruct` stays `not-measured` with the reason
`too-large-for-the-measuring-machine`: on 8 GB the runtime ran out of GPU memory
answering its first question (`kIOGPUCommandBufferCallbackErrorOutOfMemory`), and
the file also carries no chat template the runtime can use, which is its own
entry below. What the box above could not do is still true of the box above.

### A 7B model gets the reads right and addresses every change to the wrong door
**Version:** `qwen2.5:7b-instruct-q4_K_M` (`sha256:845dbda0…697e`, 4,683,087,332
bytes) under Ollama 0.34.0 on an Apple M3 with 8 GB of unified memory, macOS
26.5.2, one round of `alo_driving::THE_SET`. 2026-09-13.
**Behaviour:** four of ten drove the verbs, and the four and the six split
exactly along a line the grade cannot show. **Every read went through the right
door** — `list`, `read`, `open` and `focus` produced
`{"format":1,"asks":{"read":…}}` or `{"asks":{"propose":…}}` with the verb and
its arguments where they belong. **Every one of the six failures named the
verb where the door belongs**: `{"asks":{"rename_file":{"verb":"rename_file",…}}}`,
`{"asks":{"move_file":…}}`, `{"asks":{"close_application":…}}`. Two did worse
inside that: `find` put the door's name `read` in `verb` and the verb's name in
the door's place, and `archive` dropped the argument list for a bare object of
name to value, which is how `qwen2.5-3b-instruct` failed on 2026-09-04. One
answer carried a stray backtick after its closing brace. Every failure is the
same outcome, `NotAMessage(NotReadable)`: the daemon's door could not read the
envelope, so the registry was never reached.

This is a different failure from the small entries above, which lost the
punctuation or copied the prompt's placeholders. A 7B model holds the
envelope's syntax and loses its **grammar** — which key is a door and which is
a verb — and loses it precisely for the verbs that change something, which are
the ones the door exists to keep apart.
**Our response:** the grade is `rarely` and the bar did not move. The prompt
was not edited to name the doors more loudly, because a bar that moves to meet
the candidate is not a bar; the finding is recorded here because it says where
the next candidate's prompt-independent evidence should be looked for. The
report the grade came from,
`docs/autonomy/updates/one-catalogue-entry-graded-on-a-machine-that-can-hold-it.md`,
has the ten answers verbatim.
**Date:** 2026-09-13.

### Teuken has no first-party Q4_K_M, and the entry named the research release
**Version:** `data/catalogue.toml`'s `teuken-7b-instruct` as of 2026-09-11,
against what openGPT-X publishes on Hugging Face that day.
**Behaviour:** two things, found while looking for the weights this entry's
`upstream` names. **openGPT-X publishes the model twice** —
`Teuken-7B-instruct-research-v0.4` and `Teuken-7B-instruct-commercial-v0.4` —
and the two are published under different licences: the Hugging Face metadata
for the research release says `license: other`, and only the commercial release
says `license: apache-2.0`. The catalogue's entry named the **research** release
and stated `Apache-2.0`, `commercial_use = "permitted"` — so a business reading
this catalogue would have been told it may use commercially a release whose
publisher licensed it for research. That is the harm the file's own first rule
names: a licence stated wrongly is worse than a model omitted. **And neither
release has a first-party GGUF**: there is no `teuken` in the pinned runtime's
library at all, and every Q4_K_M of it is a third party's requantisation
(`mradermacher`, `KnutJaegersberg`, `bartowski` and others). So the entry's
`quantisation = "Q4_K_M"` names an artefact its `upstream` does not publish, and
measuring one would be measuring somebody else's requantisation while reporting
it as this entry's.
**Our response:** `upstream` now names the commercial release, which is the one
whose licence the entry already states, and the licence line is unchanged
because it was true of that release all along. The size is unchanged: it is the
same model at the same quantisation, and the two releases differ in their
licence rather than in their weights. The second half is **not** worked around:
nothing in this repository says which artefact an entry means when its publisher
ships none at the quantisation stated, and a grade measured against a stranger's
requantisation would carry this catalogue's authority for a file this catalogue
never chose. That question is written into the lane's plan as part of the task
that widens the catalogue, and until it is answered `teuken-7b-instruct` cannot
be measured on any machine, however much memory it has.
**Date:** 2026-09-11.

### Two entries kept a four-bit size after they stopped claiming a four-bit file
**Version:** `data/catalogue.toml`'s `eurollm-9b-instruct` and
`teuken-7b-instruct` as of 2026-09-11, against the file lists their `upstream`
repositories publish that day.
**Behaviour:** rule 4 made `quantisation` a claim an entry has to be able to
point at, and these two could point at nothing, so both now state none. Their
sizes were left where they were, and that is a worse state than the one rule 4
fixed. `eurollm-9b-instruct` said 5.6 GB and `teuken-7b-instruct` said 4.6 GB —
0.61 and 0.66 bytes per parameter, which is what a four-bit GGUF costs and
nothing else does. `min_vram_gb` and `min_ram_gb` came from the same assumption:
Teuken said ten gigabytes of system memory beside weights that are fifteen. So
the entries read as small models an ordinary laptop could hold, and what they
name — the only thing either publisher actually publishes — is nearly four times
the size. A person sizing a machine off `min_ram_gb` would have bought the wrong
one, and rule 2's *sizes are what the disk and the card actually lose* was being
honoured for a file neither entry claimed.

It is worth saying what it is **not**. Neither figure was invented: both were
true of the entries as they were first written, when each claimed `Q4_K_M`. What
happened is that one half of a pair was corrected and the other half was left,
which is the ordinary way a data file goes wrong — and why the fix is a rule
with arithmetic under it rather than two better numbers.
**Our response:** rule 5, and the publisher's own release as the road. Where no
first-party quantised artefact exists, an entry states the weights its publisher
publishes, read off that repository's file list: EuroLLM is four `bfloat16`
shards totalling 18_304_683_360 bytes over 9_152_319_488 parameters, Teuken is
four totalling 14_905_484_192 over 7_452_725_248. The alternative — naming a
stranger's requantisation — was refused for the reason the entry above gives:
this catalogue would be vouching for a file it never chose, and choosing one is
a decision with nobody's name on it. Teuken's `parameters_b` was 7.0, the
publisher's product name rather than the count in its own manifest, and is now
7.5; its `on_cpu` moves from `workable` to `slow`, which was true of the
four-bit download and is not true of this one. **No grade moved**: a size is not
a measurement of driving, and both entries stay `not-measured`.

The rule is arithmetic so that it cannot rot the same way again.
`Catalogue::parse` divides `download_bytes` by `parameters_b` and refuses an
entry whose answer sits on the wrong side of 1.5 bytes per parameter for what it
claims — every four-bit entry in the catalogue lands between 0.56 and 0.80, and
`bfloat16` lands at 2.0 — and refuses a `min_vram_gb` or `min_ram_gb` below the
size. The consequence is visible rather than hidden: these two entries are now
large, slow and out of reach of an ordinary laptop, which is the true sentence
about a model nobody has quantised for us.
**Date:** 2026-09-11.

### The two best-known requantisers of Teuken took the release nobody may rely on
**Version:** the Hugging Face uploads of `openGPT-X/Teuken-7B-instruct-*-v0.4`
and `utter-project/EuroLLM-9B-Instruct`, read 2026-09-11, against
`data/catalogue.toml`'s rule 6.
**Behaviour:** ADR 0026 decided that a third party's artefact may be named if
the entry says whose it is, which file exactly, and what a reader needs to know.
Paying that for the two European entries turned out to be a licence question
before it was a quality one. **openGPT-X publishes Teuken twice** — a research
release under `license: other` and a commercial one under Apache-2.0 — and the
requantisers split across the two. `bartowski` and `QuantFactory`, the two names
a person would reach for first, both published the **research** release
(733 and 544 downloads); the commercial release is requantised by
`mradermacher` (932 downloads, plus a weighted set at `-i1-GGUF`),
`KnutJaegersberg`, `tensorblock` and a handful of individuals. This entry's
licence line says Apache-2.0 with commercial use permitted, so naming the
popular upload would have put a research-licensed artefact behind a commercial
claim — the same harm the entry two above records, arriving through the new
door instead of the old one. It is a harm nothing mechanical here would have
caught: `Requantised` checks that a digest is a digest and that the requantiser
is not the publisher, and a research requantisation passes both.

The second finding is smaller and in the same direction. Of the four uploads of
a Q4_K_M of EuroLLM, three are near-identical in size —
5_582_838_496, 5_582_838_208 and 5_582_838_112 bytes — and differ in every byte
of their digests, because they are three separate conversion runs of the same
weights. There is no "the" Q4_K_M of a model, only somebody's, which is the
whole reason rule 6 asks for a name and a pin rather than a quantisation label.
**Our response:** `eurollm-9b-instruct` names `bartowski`'s upload and
`teuken-7b-instruct` names `mradermacher`'s, each with the `sha256` its own
repository publishes and a note saying how it was made. The grounds are stated
in the entries rather than implied: for Teuken it is the licence, and for
EuroLLM — where all four uploads carry the model's own Apache-2.0 — it is that
`bartowski` states the fullest recipe, naming the llama.cpp release and
publishing the importance-matrix calibration set the others do not describe. The
licence on each **uploader's** repository was read as well as the publisher's,
which is what surfaced the split. Rule 6 gained both lessons for the next
curator. **No grade moved:** this lane's box cannot hold a 7B model at four bits
(the entry above has the numbers), so both entries stay `not-measured` with an
artefact now named — which is the honest state, and a smaller distance from a
grade than they were at this morning.
**Date:** 2026-09-11.

### The carry-or-fetch measurement ADR 0025 owes: the catalogue has nothing to weigh
**Version:** `data/catalogue.toml` as of 2026-09-11 — fourteen entries, five
measured by `alo-driving` on 2026-09-04 and two more later the same day, seven
`not-measured` — against the bar
`alo_driving::measured::RELIABLY` writes down and
`alo_models::Driving::clears_the_bar` enforces.
**Behaviour:** ADR 0025 recommends carrying the weights on the certified image
and fetching only where an image cannot, and says in as many words that this is
a recommendation with a measurement owed — two numbers and a sentence, owed
before anybody puts weights aboard. The first number does not exist: **the
smallest catalogued model that clears the verb-driving bar is no model at all —
none of them clears the bar.** Every entry, off the catalogue rather than from
memory:

| Entry | Download bytes | Drives the verbs |
|---|---|---|
| `eurollm-9b-instruct` | 5_582_838_496 | `not-measured` |
| `teuken-7b-instruct` | 5_018_868_512 | `rarely` |
| `mistral-7b-instruct` | 4_370_000_000 | `rarely` |
| `mixtral-8x7b-instruct` | 26_400_000_000 | `not-measured` |
| `qwen2.5-7b-instruct` | 4_680_000_000 | `rarely` |
| `phi-3-mini-instruct` | 2_400_000_000 | `rarely` |
| `llama-3.1-8b-instruct` | 4_920_000_000 | `rarely` |
| `gemma-2-9b-instruct` | 5_760_000_000 | `not-measured` |
| `llama-3.2-3b-instruct` | 2_020_000_000 | `rarely` |
| `qwen2.5-3b-instruct` | 1_930_000_000 | `rarely` |
| `gemma-2-2b-instruct` | 1_710_000_000 | `rarely` |
| `smollm2-1.7b-instruct` | 1_060_000_000 | `rarely` |
| `qwen3-1.7b` | 1_359_279_776 | `rarely` |
| `granite-3.2-2b-instruct` | 1_545_296_256 | `rarely` |
| `qwen3-4b` | 2_620_774_592 | `rarely` |
| `qwen3-8b` | 5_225_374_496 | `sometimes` |

**Fourteen entries as of 2026-09-11, seven measured and all seven `rarely`.**
The table above was twelve rows and five grades when this measurement was
first made; the two entries at its foot were added later the same day, chosen
because their publishers train them for tool calls and constrained output, and
they graded the same as the five general chat models before them. That does not
move the verdict — it strengthens it, because the obvious next candidate class
has now been tried. The seven that were not measured are not candidates: ADR
0007 says the grade is measured by us and never claimed by the publisher, and
`Driving::NotMeasured` refuses the bar on purpose. An unmeasured entry is a gap
the ledger already carries, not a model that is probably fine — and the gap has
a shape: everything unmeasured wants ten gigabytes of system memory or more,
and the box every existing grade was made on has six.

**Two of those rows moved twice on 2026-09-11, and both moves are corrections
rather than changes of model.** `eurollm-9b-instruct` and `teuken-7b-instruct`
had kept the four-bit `download_bytes` they were added with, 5.6 GB and 4.6 GB,
after rule 4 had taken away the quantisation those figures belonged to. Under
rule 5 they first became their publishers' own `bfloat16` releases, 18.30 GB and
14.91 GB — honest, and out of reach of any laptop. Then rule 6 was paid for both
and each entry named the upload it means: 5_582_838_496 bytes of `bartowski`'s
imatrix Q4_K_M and 5_018_868_512 of `mradermacher`'s static one, each pinned by
the digest its repository publishes. So the rows are four-bit figures again, and
for the first time they are four-bit figures for a file this catalogue chose
rather than for one it had merely assumed. The verdict is untouched: neither
entry is measured, so neither was ever a candidate.

**The channel:** what the image and its update stream can honestly carry.
ADR 0011 makes the OS a bootable container image pulled from a registry we
operate, and its layers are content-addressed: a weights layer travels only
when its digest changes, and two deployments that share it store it once. So
carrying weights costs the channel one transfer per weights **change**, not one
per update — and a pinned model changes by a decision, the way the runtime's
version in `image/Containerfile` does, not with every rebase of the base.
`docs/features.md`'s *an upgrade cannot break a working stack* is bootc's
atomic deployment with rollback, and a carried layer rides inside what
`bootc rollback` restores where a setup-time fetch sits outside it — which is
an argument for carrying, not only a cost. On size: the CPU-class entries the
certified laptop would carry are 1.06–2.4 GB and the 7B class that names a
four-bit artefact is 4.4–5.6 GB, the same order as the pinned base and the
runtime artefact the image already moves, so a carried layer of that kind is
not structurally beyond this channel. **The two European entries used to be a
different order, and are no longer:** while they named no artefact they stated
14.91 and 18.30 GB of `bfloat16`, which is not a layer this channel carries
comfortably; naming a requantisation apiece brought them to 5.02 and 5.58 GB,
inside the same band as every other four-bit row. That is a consequence of a
curation act rather than of anything the channel learned, and it is worth
saying plainly: the question *can the stream move this* was answered by
choosing a smaller file, not by measuring the stream. What is
honestly bounded: no registry of ours, no update stream and no mirror is
running yet, so transfer time on the certified machine's network, hosting
cost, and how the registry behaves when a five-gigabyte layer changes have
not been measured on real infrastructure. The structural argument above is
the whole of what can be stated today, and it is stated as reasoning rather
than as a measurement.

**The sentence:** today, **no weights go aboard** — neither carried nor
fetched — because there is nothing to carry: the promise is a model that can
drive the verbs, and no catalogued entry is measured doing it. When one is,
the answer the channel half supports is ADR 0025's own recommendation —
carried on the certified image, fetched at setup only where an image cannot —
remembering the ADR's caution that a machine which fetches at setup is not
local by default when it is offline at setup. The weights task therefore waits
on the catalogue rather than on wishes, and the plan's next task is the grade
it waits on.

**Our response:** the numbers above are held to `data/catalogue.toml` by
`crates/alo-models/tests/the_carry_or_fetch_measurement.rs`: a table row that
disagrees with the catalogue fails, an entry missing from the table fails, and
the day a catalogued model clears the bar while this entry still says none of
them does, the test fails and sends whoever sees it back here — to name the
model, its size and its grade, and to turn the sentence into carry or fetch
for real.
**Date:** 2026-09-11.

### Phi-3 Mini gets the envelope right and loses the argument list
**Version:** `phi3:3.8b-mini-4k-instruct-q4_K_M` — Microsoft's Phi-3-mini-4k-
instruct at the quantisation `data/catalogue.toml` states — served by Ollama
0.33.3 on four CPU cores, 2026-09-04. Two rounds of `alo_driving::THE_SET`,
twenty attempts.
**Behaviour:** **none of the twenty produced a call this machine would have
acted on**, and every one of them failed at the same place: the daemon's door,
before the verb registry was ever reached. Not one attempt got as far as being a
real call with a bad argument. What it wrote is the useful part, because it is
not what *sentences they manage, structure they lose* sounds like it would be —
the outer structure is nearly always right and the innermost part is nearly
always wrong:

```
{"format":1,"asks":{"read":{"verb":"read_file","given":["/home/anna/Invoices/march.pdf"]}}}
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":true}]}}}
{"format":1,"asks":{"propose":{"verb":"rename_file","given":{"file":"…/scan001.pdf"}}}}
{"format":1,"asks":{"propose":{"application":"org.alo.Writer"}}}
```

The format number, the `asks` wrapper, the door and usually the verb name are
all correct. `given` is then a list of bare strings, or an object instead of a
list, or the argument's *value* put in the `named` field with `"is": true` after
it, or the verb dropped and its argument left standing where the verb was. Five
of the twenty also arrived inside a ```` ```json ```` fence or with a paragraph
of explanation after them, in spite of a prompt that says *no explanation, no
code fence, no second line* — so they were more than one message as well as the
wrong shape.
**Our response:** the grade is `rarely` and that is what the entry says. Nothing
in the prompt or the scoring was changed to help it: the shape it cannot produce
is the shape `alo-protocol` really reads, and a measurement loosened until a
model passes measures the loosening. The finding to carry forward is that the
six outcomes did not divide this model at all — every failure was
`NotAMessage`, so a report that only counted them would have said *it wrote
prose*, which is exactly what it did not do. A model this close to the shape is
also the strongest case yet for the caveat below, that an agent composing the
envelope around what its model emits may do better than the grade says.
**Date:** 2026-09-04, iteration 44.

### Four small models, four ways of failing, and the grade cannot tell them apart
**Version:** `llama3.2:3b-instruct-q4_K_M`, `qwen2.5:3b-instruct-q4_K_M`,
`gemma2:2b-instruct-q4_K_M` and `smollm2:1.7b-instruct-q4_K_M` — the four
entries in `data/catalogue.toml` small enough for a 6 GB box, at the
quantisations it states — served by Ollama 0.33.3 on four CPU cores,
2026-09-04. Two rounds of `alo_driving::THE_SET` each, eighty attempts.
**Behaviour:** all four graded `rarely`, and **three of the eighty produced a
call this machine would have acted on**: one from Qwen2.5 3B, two from Gemma 2
2B, none from the other two. The grade is the same word four times and the
failures are four different problems.

```
llama    {"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}
qwen     {"format":1,"asks":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}}
gemma    ```json {"format":1,"asks":{"read":{"verb":"focus_application",…}}} ```
smollm2  {"format":1,"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":["folder","/home/anna/Invoices"]}]}}}
```

**Llama 3.2 3B loses the punctuation and nothing else.** Twenty of twenty
unreadable, and the argument shape — a list of `{"named":…,"is":…}` pairs,
which is the part Phi-3 never once got right — is correct in most of them. What
is wrong is a brace short, a brace over, a `"` where a `}` belongs. It is the
closest any measured model has come to the shape while scoring zero.

**Qwen2.5 3B puts the door and the verb name where they belong** and then
collapses `given` into a bare object of name to value, dropping the wrapper
entirely. Nineteen of twenty unreadable, one drove.

**Gemma 2 2B is the only entry whose answers get past the daemon's door.** Its
twenty divide across four of the six outcomes rather than piling into one: two
drove, five were more than one message (three of those inside a ```` ```json
```` fence), two asked a *change* through the read door, and two invented a verb
called `CLOSE` and were refused by the verb registry. It is the first evidence
that the six outcomes divide a real model at all — Phi-3's twenty were all
`NotAMessage`, and iteration 44 could not tell from one model whether that was
a fact about small models or about the scoring.

**SmolLM2 1.7B copies the prompt instead of answering it.** `"verb":"NAME"`,
`"named":"ARGUMENT"`, `"is":VALUE` — the placeholders the prompt uses to
describe the shape, written back verbatim with the real path sometimes stuffed
in beside them. That is not the same failure as losing the argument list; it is
a model that has read the schema as the answer.
**Our response:** the grades are what the entries say, and nothing in the
prompt or the scoring was changed to help any of them. The finding worth
carrying is that **`drives_verbs` is deliberately blind to all of this**: a
machine deciding whether to hand somebody's files to a model does not care
which way the model was wrong, so one word is the right public property and
this file is where the rest goes. It also means a report that counted only
outcomes would describe these four as one thing.
**Date:** 2026-09-04, iteration 45.

### A model that answers in seconds can still run past the five-minute wait
**Version:** `smollm2:1.7b-instruct-q4_K_M` under Ollama 0.33.3, four CPU
cores, 2026-09-04.
**Behaviour:** the first run of `against_a_model_on_this_machine` stopped on the
seventh exercise with `RuntimeError::TookTooLong` — `WHILE_A_MODEL_THINKS`, five
minutes — from the smallest model in the catalogue, on a run whose other
answers took seconds each. The second run of the same model, the same set and
the same machine finished in 134 seconds with every exercise answered. Nothing
was changed in between. The likeliest cause is the model running away on one
generation rather than the machine being slow, and no measurement can tell those
apart from outside.
**Our response:** the run was made again and the grade is the second run's,
which is what the harness is built for — *a runtime that fails stops the
measurement rather than scoring a failure*, because a model blamed for a machine
is the one way a grade is worse than no grade. Saying so here is the price of
that rule: a re-run kept and a stopped run discarded looks like picking the
better of two results, so the stopped run is recorded rather than left out. What
would make it wrong is re-running until a *grade* improves, and the two runs
here did not produce different grades — the first produced none.
**Date:** 2026-09-04, iteration 45.

### `Ollama::load` cannot load a model on a machine with no graphics card
**Version:** `alo-models`' `ollama.rs` as of 2026-09-04, against Ollama 0.33.3.
**Behaviour:** loading is an empty generate with a non-zero keep-alive, and it
is sent with `QUICK_TIMEOUT` — ten seconds, on the reasoning that listing what
is installed is a local read and a runtime that has not answered in that long is
not well. Loading is not that kind of call. Measured on the development box,
`phi3:3.8b-mini-4k-instruct-q4_K_M` took **220 seconds** to come off the disk
against **2 seconds** to answer once it was there. So `load` returns
`RuntimeError::Unreachable` — *nothing was listening* — for a runtime that is
listening, is working, and is doing exactly what it was asked.
**Our response:** **not worked around, and not yet fixed.** It is a queue item
(29) rather than a line changed in passing, because the constant is not the
whole of it: what a person is shown while a model loads for four minutes is a
question about the shell as much as about the adapter, and `RuntimeError` has no
variant meaning *it is loading*. Nothing in alo OS calls `load` today, which is
why this has never been seen. `alo-driving`'s measurement therefore warms a
model with a throwaway *question* instead, which goes through
`WHILE_A_MODEL_THINKS` — five minutes — and works.
**Date:** 2026-09-04, iteration 44.

### What `drives_verbs` measures, and the two things it does not say
**Version:** `alo-driving` as of 2026-09-04, run for the first time against a
real model on 2026-09-04 — `phi-3-mini-instruct`, graded `rarely` — and against
the four smaller entries the same day, all `rarely` too, in the entries above.
The remaining seven in `data/catalogue.toml` still say `not-measured`: every one
of them wants ten gigabytes of system memory or more and the measuring box has
six, so what is missing is a machine with room rather than a method.
**Behaviour:** the measurement puts ten requests to a model and scores each
answer through `alo_protocol::FromAnAgent` and `alo_capability::Verbs::call` —
the daemon's own door and the same validation a real turn does. Two consequences
that a grade does not carry on its face.

**It is asked in English.** The prompt is not a string a person reads, so it is
not an `alo_strings::Word` and this crate declares no vocabulary; what follows
is that a grade says how a model drives the verbs *when it is asked in English*.
A model asked in Latvian may do worse, and nothing here would know. Measuring in
twenty-four languages is a real question, it is a different one, and pretending
the current grade answers it would be the kind of claim `Driving::NotMeasured`
exists to prevent.

**The envelope is part of what is measured.** A model has to produce the whole
message — `{"format":1,"asks":{"read":{…}}}` — rather than a bare verb and
arguments. That is deliberate: a lighter shape invented for the measurement
would be a second parser for one syntax, which is the failure item 9g removed
one level down. But a real agent composes the envelope around whatever its model
emitted, so **a model wrapped by such an agent may drive the verbs better than
its grade says**.

**Our response:** both errors fall the same way — toward not giving a model the
agent — which is the direction every other decision about this property takes,
and a machine that refuses says so in words naming the two places ADR 0008
leaves open rather than substituting one. Neither is worked around. Whoever
raises the measurement's coverage changes the fixed set, and a changed set means
every grade in the catalogue is stale: `alo_driving::THE_SET` is where that
version lives, and it is a `&'static` array in the source so it cannot drift
quietly.

**A third thing it does not say, learned by running it.** A grade is a property
of the weights, not of the machine — the first measurement was made on four CPU
cores and would have earned the same grade on a workstation, because what was
being asked is whether the model can produce a shape. What that machine *did*
decide is how long it took, which is why the entry it produced left `min_ram_gb`
and `on_cpu` alone: those are what a machine loses to a model, and one slow WSL
box is not the certified one.
**Date:** 2026-09-03, iteration 42; run for the first time and extended
2026-09-04, iteration 44; four more models 2026-09-04, iteration 45.

### A model copies the door of the example it is shown, not the sentence under it
**Version:** Qwen 2.5 7B Instruct, `qwen2.5:7b-instruct-q4_K_M`, under Ollama
0.34.0 on an Apple M3 with 8 GB unified memory; `alo-driving` as of 2026-09-14.
**Behaviour:** `alo_driving::HOW_TO_ANSWER` shows one example request, through
the `read` door, and says in a sentence underneath to use `propose` for a
change. Asked in the envelope through `alo-asking`'s door, the model drove 71 of
80; five of the nine failures were a change sent through `read`, and the small
models' wrong doors in task 10 and the five-bit weights' twenty-two in task 13
were `read` too. Shown the same text with a second example through `propose`
(`alo_driving::ONE_EXAMPLE_PER_DOOR`), with nothing else changed — exercises,
verbs, scoring, bar, weights, runtime, machine and door — the same weights drove
**80 of 80**, and every change went through `propose`.
**Our response:** [ADR 0034](decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md).
The first instructions are kept and named; every catalogue grade records the
SHA-256 of the instructions it was earned under; the new grade sits beside the
old in `[[model.also_under]]` and never over it; and the recommendation still
reads neither, because an agent turn is not yet shown these instructions.
Whoever writes the turn's prompt writes an example for each door it offers.
**Date:** 2026-09-14.

### A grammar that forbids the wrong door moves the failure, it does not remove it
**Version:** `llama.cpp` 0.4.0 (build 10809, commit 5266f24da) serving
`qwen2.5:7b-instruct-q4_K_M` on an Apple M3 with 8 GB unified memory;
`alo-driving` as of 2026-09-14.
**Behaviour:** the engine's server takes a GBNF grammar, so the whole call can be
held in the protocol's own key order — which the pinned runtime cannot do,
because it orders a schema's keys alphabetically (ADR 0032). Held that way, the
door a verb takes is decided by the grammar and a change through the read door is
unreachable. Under the first instructions, which show only a `read` example, the
model then produced **the wrong verb** instead: `read_file` for a rename five
times in eighty, `read_file` for a close five times, `find_in_folder` for a close
three times, and two more — 65 of 80, against 71 of 80 for the same weights
through the pinned runtime under the same instructions. Under ADR 0034's
instructions both were 80 of 80.
**Our response:** [ADR 0035](decisions/0035-the-wrapper-or-the-engine.md),
rejected on it. A constraint on the shape of an answer is not a fix for a model
reading the request wrongly: it relocates the failure from the door, where
`alo_capability::Authorised::read` refuses it, to the verb, where the call is
well-formed and a machine would act on it. `alo-driving`'s measurement catches a
wrong verb because every exercise names the verb a correct answer calls; nothing
at runtime would. What fixed this failure was the instructions (ADR 0034), not
the constraint.
**Date:** 2026-09-14.

### The engine's own server does not place a model that does not fit, and the wrapper does
**Version:** `llama.cpp` 0.4.0 (build 10809) and Ollama 0.34.0 on an Apple M3
with 8 GB unified memory, macOS 26.5.2.
**Behaviour:** `qwen2.5:7b-instruct-q5_K_M` is 5,444,831,648 bytes and does not
fit the graphics processor's working set on this machine. The pinned runtime
serves it anyway, splitting it — 5,959,592,178 bytes loaded, 4,563,287,407 of
them on the graphics processor — and it was graded twice that way.
`llama-server -ngl 99` loads, then fails every request with
*Insufficient Memory (kIOGPUCommandBufferCallbackErrorOutOfMemory)* and answers
500. Told by hand to keep 21 of its 28 layers on the processor it serves, at 256
seconds for one sixty-token answer, past the five minutes `alo_models` waits.
**Our response:** recorded against [ADR 0035](decisions/0035-the-wrapper-or-the-engine.md),
which was rejected with this as one of its reasons. The wrapper's model
placement is a feature of the wrapper, not an accident of it, and a machine sold
with 8 GB is exactly where it matters. Whoever proposes the engine again owns
the placement decision on the smallest certified machine, and the five-bit
weights are the test case.
**Date:** 2026-09-14.

### Asked in the envelope within seconds of being fetched, the runtime answers nothing usable
**Version:** Ollama 0.34.0 on an Apple M3 with 8 GB, macOS 26.5.2, 2026-09-14,
measuring `teuken-7b-instruct` (a 7B at Q4_K_M, 6.0 GB loaded).
**Behaviour:** `Ollama::fetch`, then `unload`, then a first `/api/chat` **held to
the envelope's schema** answers in about six seconds with something
`alo_models::RuntimeError::Unusable` refuses — a non-200 or an empty message, and
the crate deliberately does not repeat a backend's body, so which is not
recorded. It happened twice, on two separate fetches, and both times the same
weights answered the same question correctly once they were warm: the
fetch-and-measure run that failed at the warm-up graded 3 of 20 when the
measurement was run again against a loaded model. A question asked **not** held
to a schema right after the same fetch answered normally (*" I am ready."*), so
what is fragile is the first *structured* request rather than the first request.
**Our response:** nothing in the product is changed and no retry is added to it:
a turn that gets nothing usable is told so, which is the right answer for a
person. What changed is how a measurement is run — the weights are warmed before
the fixed set is put to them, which the harness already does with its own
throwaway question and which is not enough within seconds of a fetch. Whoever
sees this in a product setting should read it as *the model was just installed*,
and the measurement that would settle the cause is a packet capture of that first
request, which nobody has made.
**Date:** 2026-09-14.

### A nine-billion model loads on 8 GB and still cannot answer a real request
**Version:** Ollama 0.34.0 on an Apple M3 with 8 GiB, macOS 26.5.2, 2026-09-15,
with the development VM stopped: `eurollm-9b-instruct` (5.58 GB file) and
`gemma-2-9b-instruct` (5.76 GB).
**Behaviour:** both load and both answer. EuroLLM sits at 6,494,638,568 bytes
with 4,608,848,035 on the graphics processor; Gemma at 7,451,579,511 with
4,125,160,897. **Loading is not the limit — the prompt is.** A nineteen-token
question held to the envelope came back from EuroLLM in 168 seconds and from
Gemma not at all inside 300. `alo-driving`'s own prompt carries every verb the
machine declares, about 1,500 tokens, and EuroLLM did not answer one inside the
300 seconds `alo-models` waits in any of three runs — asked freely, asked in the
envelope, and asked in the envelope under the instructions a turn shows. So the
cost that decides is prompt evaluation against weights that are 1.9 GB (EuroLLM)
and 3.3 GB (Gemma) adrift of the processor, not the size of the file.
**Our response:** both entries keep `too-large-for-the-measuring-machine`, whose
own words are *does not have the memory to run the model inside the time
`alo-models` waits for an answer* — which is now what was measured rather than
what was inferred from `min_ram_gb`. The wait is not lengthened and the prompt is
not shortened: the prompt is what a real turn sends, and a person waiting five
minutes for one request has already been failed. What would lift it is memory —
these two want the 12 GB their entries state — and `iogpu.wired_limit_mb`, left
at its default here, would buy about a gigabyte of residency on this machine but
needs the owner's password and cannot be set by an agent.
**Date:** 2026-09-15.

### An 8B model on this 8 GB machine can take longer than the five minutes the product waits
**Version:** Ollama 0.34.0 on an Apple M3 with 8 GB, macOS 26.5.2, 2026-09-14,
measuring `llama-3.1-8b-instruct` (5.7–6.2 GB loaded, about 1 GB of it off the
graphics processor).
**Behaviour:** two runs of the fixed ten were abandoned at the second or third
exercise with `RuntimeError::TookTooLong` — `alo-models` waits 300 seconds for an
answer (`WHILE_A_MODEL_THINKS`). At the time the machine had 5.1 GB of its 6 GB
swap in use and about 60 MB of free pages, with an editor, two agent sessions and
macOS's own indexing resident. The same entry, on the same day, with the weights
already warm and nothing else started, answered all twenty and graded 10 of 20.
A single answer took 8 to 16 seconds when it worked.
**Our response:** the harness refuses to score a runtime failure as a model's
failure, which is why two runs produced no grade rather than a bad one, and that
is the behaviour being relied on rather than worked around. The 300-second wait is
not raised: a person waiting five minutes for one request has already been failed,
and lengthening it would hide exactly this. What it says about the product is that
**8 GB is the floor for an 8B entry and it is a floor with nothing above it** —
the catalogue's `min_ram_gb` for this entry is 10, and this machine is below it.
A certified machine's own measurement is the one that decides.
**Date:** 2026-09-14.

### Under the boundary on this VM, an ordinary program cannot set a file flag
**Version:** `alo-bounding`'s `the_boundary_decides_and_forgets.rs` as of
2026-09-14, on the Mac lane's Lima VM — Ubuntu on kernel `7.0.0-31-generic`,
aarch64, six processors, 4 GiB.
**Behaviour:** `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`
fails at line 364 — *"an ordinary program can set a flag on its own files:
Os { code: 95, kind: Unsupported, message: 'Operation not supported' }"* — where
the test sets `IFlags::NODUMP` on a file of its own in `/tmp`, outside any turn,
while the boundary is attached. It fails **deterministically**, alone and in the
suite, and it is the only test of 4,846 in the workspace that does. With no
boundary attached, `chattr +d` on a file in the same `/tmp` (ext4) succeeds, so
the kernel and the filesystem do support the flag on this machine. Whether the
`file_ioctl` hook refuses it, or the aarch64 kernel answers `ENOTSUP` for a
reason of its own, is **not established here**.
**Our response:** recorded for `alo-bounding`'s owner, not worked around and no
gate weakened. The measuring lane found it while gating an unrelated change and
does not own the crate; it is already in the loop's own gate log on this machine
(`~/alo-builds/gate-the_workspace's_tests.log`, 2026-09-14). The measurement
that would settle it is the same test with the boundary detached, and then with
the `file_ioctl` hook alone.
**Date:** 2026-09-14.
**Settled, 2026-09-14:** the boundary was never involved — see *Setting a
file's flags to exactly `nodump` asks ext4 to take its extents away* below.

### Setting a file's flags to exactly `nodump` asks ext4 to take its extents away
**Version:** Linux `6.18.33.2-microsoft-standard-WSL2`, ext4 `/tmp`, no
boundary loaded; the loop's gate machine, 2026-09-14.
**Behaviour:** `FS_IOC_SETFLAGS` *replaces* a file's flags; it does not add to
them. Every file ext4 lays out in extents carries `EXTENTS_FL` (`0x80000`, the
`e` in `lsattr`), so a request of `NODUMP` alone asks ext4 to convert the file
back to indirect blocks, and ext4 refuses that conversion with `EOPNOTSUPP`
for a file whose blocks are not settled. Measured with no boundary at all, on a
file of sixteen bytes opened write-only: written, then flags set to `NODUMP`
alone — accepted, and the file *silently lost its extents* (`0x40` afterwards);
written, cut to one byte with `ftruncate`, then `NODUMP` alone — `EOPNOTSUPP`;
the same after an `fsync` — accepted; the same with `NODUMP` added to the
flags the file had — accepted every time (`0x80040`). `chattr +d` reads the flags
first, which is why every probe with it succeeded. On `tmpfs`, which has no
extents, `NODUMP` alone was always accepted, and that is where `/tmp` was when
these tests were written.
**Our response:** the two `alo-bounding` tests that set a flag
(`the_boundary_decides_and_forgets.rs`, `the_kernel_refuses_an_attribute_change.rs`)
read the flags first and add `NODUMP` to them, the way an ordinary program
does; the first puts back exactly what it read. Nothing a hook decides changed:
the refusal inside a turn is still the `SETFLAGS` request's, because a read of
a file's flags is answered inside a turn even outside the grant. This settles
the entry above and *Setting a file flag under the boundary is refused as
unsupported on aarch64* — the shortening just before the flag was the step the
probes there did not take.
**Date:** 2026-09-14.

### A turn asks a model in English, whatever language the machine runs in
**Version:** `alo-instructing` as of 2026-09-14, the crate the words a model is
shown moved into ([ADR 0037](decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)).
**Behaviour:** the text a model is shown is how to answer, every verb in the
sentence the verb declared, and the request — and all of it is English. The
verbs' sentences are `alo_strings::Word`s with an English default, and this
crate asks for no translation of them; the instructions are a `&'static str`
with no vocabulary behind them at all. Until now that was a fact about a
measurement (recorded above, *it is asked in English*). Once a turn composes
what it shows a model from here, it becomes a fact about the product: a person
whose machine is in Latvian is served by an agent whose model was asked in
English. **Since 2026-09-15 it does** (`alo_turn::Turning::asking_for_the_next_request`):
an agent's next request is put to a model in this text, with the request itself
in whatever language the person wrote it, beneath English instructions and
English descriptions of the verbs.
**Our response:** recorded rather than worked around, and it falls the way the
measurement's version does — a model asked in a language it is weaker in drives
the verbs worse, and every grade in the catalogue was earned in English, so the
grade a machine reads is the grade for the way it asks. What a **person** is
told never passes through here: their words go to the model as they wrote them,
and every sentence they read back is an `alo_strings::Word` in their own
language. Asking in twenty-four languages needs the verbs' sentences translated
*and* a grade per language, which is a measurement nobody has made; it is not
work this crate can hide, and `Instructions` is where a second set would go.
**Date:** 2026-09-14.

### The small model answers in the language it was asked in, except where it drifts
**Version:** `qwen2.5:7b-instruct-q4_K_M` under Ollama 0.34.0 on an Apple M3 with
8 GB unified memory, 2026-09-16; `alo-instructing` as of the access plan's task 5.
**Behaviour:** asked the same question — *where is the invoice from Northstar?* —
in each of the 24 official languages, with the clause naming the language to
answer in, the model answered **23 of 24 in the language it was asked in**. The
exception is **Croatian**, where it begins in Croatian and drifts into Russian
mid-sentence: *«Preporučljivo je контактiranje службе поддержки Northstar-a ili
provjeriti вашу электронную почту»*. In an earlier run of the same question it
answered Croatian in Croatian and instead produced a stray Cyrillic letter inside
Latin words in Estonian and Slovak (`prieponе`), so the drift is not fixed to one
language. **Answering in a language is not answering well**: the Finnish and
Estonian answers are fluent and untrue — Northstar becomes a navigation network —
and one Slovene answer was Python code for finding a file.
**Our response:** recorded per language in the access plan's task 5 report, as
*answers in it*, *answers in another language* or *does not answer*, and not
smoothed. No larger model is run to improve it (the owner's rule of 2026-09-15):
what a bigger model would do is not what this machine does. The clause names the
language in its own word for itself — *Hrvatski*, not *Croatian* — which is what
this model was asked under.
**Date:** 2026-09-16.

### A language reader small enough to ship is wrong about its neighbours
**Version:** `alo_instructing::the_language_of` as first written, 2026-09-16.
**Behaviour:** the reader decides a language from the script and from short words
a language's neighbours do not use, because a request is one sentence and nothing
may leave the machine to read it. On the 24 requests it is written for it reads
**24 of 24**; on the 24 paragraphs the model answered with it reads **20**,
taking Finnish for Swedish, Swedish for Danish, and Slovak for nothing at all.
Its first version was worse in a way worth keeping: it read *any* Cyrillic letter
as Bulgarian, so an Estonian answer containing one stray Cyrillic character was
Bulgarian to it. It now asks that 40% of the letters be in that script.
**Our response:** the reader answers `None` where it cannot tell, and the clause
is then left out rather than a guess being put in front of a model — a model
answers in the language of the question by itself more often than not. Where the
report's numbers and the reader disagree, the report says both: what the model
did, read by a person, and what the reader scored. Nothing in the product decides
anything important on this reader; it chooses one sentence in a prompt.
**Date:** 2026-09-16.

### The obvious local fine-tuning tool cannot make an adapter, only a merged model
**Version:** `llama.cpp` 0.4.0 (build 10809, commit 5266f24da), read on
2026-09-17 on an Apple M3 and in the aarch64 gate VM.
**Behaviour:** `llama-finetune` is already on every machine that serves a model
here, and it looks like the way to fine-tune locally. It is not. It trains
**every weight** and writes a whole new model to `-o`; its `--lora` flag only
*loads* an adapter somebody else produced. There is no option that writes one.
So a fine-tune through it either rewrites the base weights or produces a second
copy of the model with the person's documents merged into it.
**Our response:** it is not the engine.
[ADR 0048](decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)
requires that what one granted folder taught be a file a person can delete on its
own, and merging forecloses that permanently — the bytes cannot be unmixed later.
The rented stack is `transformers` + `peft`, pinned in
`crates/alo-adapting/src/engine.rs`, which produces a LoRA adapter as a separate
file in the ordinary safetensors format. Recorded here because the next person
to look for a local trainer will find `llama-finetune` first, and its help text
mentions LoRA, which makes the wrong answer look like the right one.
**Date:** 2026-09-17.

### The pinned runtime will not read an ordinary LoRA adapter, only a GGUF of it
**Version:** Ollama 0.34.0 on an Apple M3, 2026-09-17, with a `peft` 0.18.0 LoRA
adapter over `Qwen2.5-0.5B-Instruct`.
**Behaviour:** given the adapter in the ordinary safetensors format — the one
every other tool reads — `/api/create` with an `adapters` map answers
`{"status":"converting adapter"}` and then `{"error":"unsupported architecture"}`,
for `Qwen2ForCausalLM`, whether or not the base model's `config.json` is handed
over with it. The same adapter converted by `llama.cpp`'s `convert_lora_to_gguf.py`
is accepted, served beside its base, and changes what the model says.
Two smaller things beside it: the `adapters` field is a map of name to **uploaded
blob digest**, not a filesystem path (a path answers `error getting blobs path`),
and it is `adapters` as a map rather than a list.
**Our response:** the safetensors adapter stays canonical — it is what a person
takes to another machine — and the GGUF is derived, regenerable, and deleted with
the adapter it came from
([ADR 0048](decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)).
The cost of keeping the portable format is one conversion step and a second copy
to track; we pay it on purpose. Whoever changes the pinned runtime should check
whether the new one reads LoRA safetensors directly, which would remove the
derived copy and a class of mistakes with it.
**Date:** 2026-09-17.

## Providers and their APIs

A provider somebody adds themselves is a service nobody here operates, behind an
address nobody here chose. Where the convention every provider claims to follow
turns out to be followed differently, record it here — with what the evidence
actually is, because a provider's documentation is not a run against it.

### There is no status that means "the account has run out", so there is a list
**Version:** the OpenAI-compatible convention and its largest publishers' error
documentation as of 2026-09-03. **Not observed against any live provider**;
`alo-asking`'s tests drive a stub on a real socket, and an account that has
really run out is owed alongside the rest of the hardware verification — it is
also the one condition in this file nobody can produce on demand without letting
a real balance empty.
**Behaviour:** HTTP has had `402 Payment Required` since 1997 and the large
providers do not send it; the services that do are gateways and resellers. What
the publishers document instead is an ordinary status carrying a
machine-readable name: `429` with `insufficient_quota`, `403` with a billing
name for an account they have stopped serving. So the two statuses a person
most needs told apart mean two things each — `403` is *your key was refused* or
*your account is empty*, `429` is *slow down* or *your account is empty* — and
the status alone cannot say which. Worse, the names collide across publishers in
the wrong direction: Google's `RESOURCE_EXHAUSTED` and everybody's
`rate_limit_exceeded` sound like running out and mean asking too fast.
**Our response:** `alo-asking`'s `ran_out.rs` holds a closed list of the
identifiers that mean an account has nothing left **and mean nothing else**, and
`openai.rs` reads the body of a `403` or a `429` — and of no other reply — to
compare against it. `402` is answered on the status alone. Three rules keep it
honest. The identifier is compared and dropped, so nothing a provider wrote
travels into a sentence a person reads. Spelling is not tracked: the letters are
matched, so `insufficient_quota` and `InsufficientQuota` are one entry. And
**when in doubt it has not run out** — a name that is not on the list leaves the
refusal exactly as it was, because a wrong *the account has run out* sends
somebody to pay for something that was never the problem, which is worse than
the number they would otherwise have been shown. `RESOURCE_EXHAUSTED` is
deliberately absent, and there is a test that says so.
**Date:** 2026-09-03

### What a provider's status code means when a *question* fails
**Version:** the OpenAI-compatible convention as documented by its publishers as
of 2026-09-03. **Not observed against any live provider**; `alo-asking`'s tests
drive a stub on a real socket, and checking this against a provider somebody
pays for is owed alongside the rest of the hardware verification.
**Behaviour:** the convention says what the *endpoint* is and says almost
nothing about which status a provider answers with when it will not answer a
question. In particular **404 means the model, not the address**: a provider
that does not offer the model somebody asked for answers 404 on an endpoint that
exists, which is indistinguishable at the protocol level from an address that is
wrong. 400 is used for a request the provider would not accept and 429 for one
it would have accepted later, and neither is a thing the person who asked can
do anything about.
**Our response:** `alo-asking`'s `hosted.rs` maps each status to the sentence a
person is actually told, and the mapping is written down there beside the
reasoning: 404 and 405 become *the model this question needed was not there*,
400 and 422 become *something answered, and not with an answer*, 401 and 403
become *the key was not accepted*, and everything else becomes *it answered
{status}, which is a problem at that end rather than yours*. What is
deliberately **not** done is guessing between "the model is gone" and "the
address is wrong" — both send a person to look at something, and only one of
them is worth their afternoon, so the sentence names what was needed rather than
what to fix.
**What this entry no longer covers:** two of those statuses have a second
meaning, and it is the entry above. `403` and `429` are read one step further —
the name inside the refusal, against a closed list — because *the account has
run out* is a third sentence and neither status carries it.
**Date:** 2026-09-03

### An OpenAI-compatible address is documented both with and without `/v1`
**Version:** documented behaviour as of 2026-09-02 — Mistral publishes an
address ending `/v1`, the pinned runtime's OpenAI-compatible surface is the bare
address with `/v1/…` beneath it. **Not yet observed against either live
service**; the tests in `alo-models` are against a stub on a real socket, and
checking this against a provider somebody pays for is owed alongside the rest of
the hardware verification.
**Behaviour:** there is no single spelling of "the address of the API". Half the
world writes `https://api.example.com/v1` in the settings field and half writes
`https://api.example.com`, and appending `/v1/models` to the first gives
`/v1/v1/models` and a 404.
**Our response:** `trying.rs` appends `/v1/models`, or just `/models` when the
address already ends `/v1`. It is one line and it is deliberately not cleverer
than that: a 404 here would be read by a person as *my address is wrong* when
their address was right, which sends them to change the one thing that was
correct. A provider that answers on neither is reported as one this system
cannot use, which is what it is.
**Date:** 2026-09-02

### Loopback is taken at face value, and one thing can therefore lie
**Version:** the whole of this repository as of 2026-09-03 — `alo-models`'
`Provider::source`, `alo-egress`' `Leaving::asking` and `alo-asking`'s three
doors. Reasoned rather than observed: there is nothing to observe, because it is
what the code believes rather than what a service does.
**Behaviour:** `Provider::source` answers `InferenceSource::ThisMachine` for a
loopback address, and everything downstream follows:
nothing leaves, law 1 shows nothing, the answer says *on this machine*, and
`SourcePolicy::ThisMachineOnly` permits it. That is right for a runtime and for
a service somebody runs on their own machine. It is **wrong for a proxy** — a
process listening on loopback that forwards the question off the machine would
be believed by every type in this repository, and a person reading their
indicator would see a quiet day.
**Our response:** nothing here, deliberately, and it is written down rather than
worked around. Deciding it in code would mean either refusing loopback (which
breaks the ordinary case ADR 0007 makes the default) or inspecting what is
listening (which is a guess about a process, and a guess this repository is not
in a position to make). The place it was expected to be caught was **egress enforcement at
the network boundary**, and **that expectation was wrong in a way worth
correcting here.** That enforcement exists since 2026-09-07 and it is
*turn-scoped*: `socket_connect` decides where a bound turn may connect, and a
proxy somebody else started is not a turn, so its own outward connection passes
untouched. A turn reaching loopback is allowed without having been shown it,
because `Leaving::asking` does not call that a departure and refusing it would
break the default ADR 0007 makes.

So the hole is still open, and closing it would need enforcement that is not
turn-scoped — a filter on everything this machine sends, which is a different
piece of work with a different blast radius and is not scheduled. Law 2 is what
keeps it small: an agent cannot start the proxy.

**It is now reproduced rather than only reasoned.**
`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs` builds the
proxy — eleven lines, on loopback, forwarding to this machine's own `eth0`
address — and drives a bound turn through it. The turn is refused that address
directly with `EACCES`, and reaches it through the proxy in the same breath. The
day something closes it, that test fails and says so.

**What this entry did not cover, and now does not need to:** *whether an address
is loopback at all* was decided by a prefix match until item 18b, so
`http://localhost.attacker.example` and `http://127.0.0.1@attacker.example/`
were this machine to every type here. That was a hole rather than a quirk and it
is fixed — `alo_models::address` parses the host — but it is worth recording
that the two questions look alike and are not: *is this loopback* is answerable,
and *is what is listening on loopback honest* is the one above.
**Date:** 2026-09-03

### Two addresses that really are this machine are treated as somewhere else
**Version:** `alo-models`' `address.rs` as of 2026-09-03. Reasoned rather than
observed, and both cases were checked against `std::net`'s parsers rather than
recalled.
**Behaviour:** `http://127.1` is loopback to curl, to browsers and to most of
libc, which read a short-form IPv4 address; `std::net::Ipv4Addr` refuses it, so
alo OS reads `127.1` as a **name**, which is not `localhost`, and therefore as
somewhere else. `[::ffff:127.0.0.1]` genuinely reaches loopback and
`Ipv6Addr::is_loopback` answers `false` for it, with the same result.
**Our response:** left as it is, because the consequences all fall the safe way.
An address alo OS reads as somewhere else is refused over `http://` (so no key
travels in clear), is shown on the indicator if it is asked at all, and cannot
become an `alo_asking::Served`. The cost is that a person who typed `127.1` is
told to write the address in full, which is a sentence about a keystroke; the
cost of guessing the other way is a question leaving with the indicator quiet.
Handling short-form IPv4 would mean writing an address parser more permissive
than the standard library's, in the one file where being wrong is law 1 failing.
**Date:** 2026-09-03

### The kernel has a name for *that was a symbolic link* and the standard library does not
**Version:** rustc as pinned by `rust-version = "1.97"`, checked by compiling.
**Behaviour:** `alo-agentd` opens its machine description with `O_NOFOLLOW` so
that the file it checks and the file it reads cannot be two different files. The
kernel answers `ELOOP` when the last part of the path is a symbolic link, and
`std::io::ErrorKind` still has no stable spelling for it: `ErrorKind::FilesystemLoop`
is behind the unstable `io_error_more` feature (rust-lang issue #86442), and
using it is a compile error rather than a warning.
**Our response:** the comparison is made against `rustix::io::Errno::LOOP` in
`crates/alo-agentd/src/unix.rs`, which is the file that already holds every
other question this crate asks the kernel, and it hands back a two-variant
`NotOpened` so that nothing outside that file compares a raw number. When the
variant stabilises, the change is one line in one file. Nothing else in the
workspace is affected — this is the only place alo OS asks the kernel to refuse
to follow a link.
**Date:** 2026-09-03

### A function that reads an environment variable cannot be tested
**Version:** edition 2024, as the workspace sets.
**Behaviour:** `std::env::set_var` and `remove_var` are `unsafe` in edition
2024, because another thread reading the environment while one is written is
undefined behaviour. `CLAUDE.md` forbids `unsafe` workspace-wide, so a test
cannot set `$XDG_RUNTIME_DIR` — and a function that goes and reads it is one
whose refusals cannot be exercised at all.
**Our response:** the decision is separated from the lookup. It was
`alo_agentd::session::where_it_runs` taking what the variable said, with
`from_the_environment` as the single line that went and looked; that file is
gone with ADR 0017, which took the socket out of the session, and
`alo_choosing::where_it_is` is the same shape still running — it takes
`$XDG_CONFIG_HOME` and `$HOME` as arguments and reads neither itself.
Every rule is a test over the first, the second has nothing in it to be wrong,
and the shape is worth reaching for anywhere else this workspace ends up reading
the environment.
**Date:** 2026-09-03

### A `poll` resumed after a signal cannot say how much of its timeout is left
**Version:** `rustix` 1.1, Linux, checked by reading `poll(2)` and the crate's
signature.
**Behaviour:** `poll` takes a length of time rather than a deadline, and a
signal ends it early with `EINTR`. There is no way to ask how much of the
timeout was left: `poll(2)` says outright that the remaining time is not
reported, and the portable answer is to read a clock before and after and work
it out. `alo-agentd` resumes the wait rather than reporting it, because on this
machine a signal *is* how a stop arrives, so a resumed wait starts the whole
timeout again.
**Our response:** left as it is, deliberately, and written down here rather than
worked around. The only thing a timeout decides in this service is when a record
is shortened; the interval is an hour (`alo_agentd::ageing::EVERY`) and the rule
it serves is counted in whole days, so an extra hour at the far end of a signal
is inside the granularity of the promise either way. The signal that causes it
is the one that ends the service. Working it out exactly would mean a second
clock read in `crates/alo-agentd/src/unix.rs`, which is the file whose whole
value is that it asks the kernel and decides nothing.
**Date:** 2026-09-03

### An agent that is a login of its own cannot reach a socket under `$XDG_RUNTIME_DIR`
**Version:** systemd-logind as shipped with Ubuntu 24.04, kernel 6.6, found by
running `alo-agentd` as two real users rather than by reading anything.
**Behaviour:** `logind` creates `/run/user/<uid>` with mode `0700`, owned by the
person. `alo_agentd::place` then makes `alo/` beneath it `0750` and hands it to
the group the agent is in, and the socket `0660` — all of which is correct and
none of which helps: reaching a path means traversing every directory above it,
and the agent is a **different user** with no `x` on the person's session
directory. Every connection from the agent's login is `EACCES` before the two
locks this repository designed are consulted at all. The person's own door works,
which is why item 21c's tests never saw it: telling the two doors apart takes two
logins and a test process has one.
**Our response:** written down and left when it was found, then moved. It was
never a bug in `place.rs` — the directory and the socket are exactly the modes
they should be — it was the socket being in the wrong place, and where it goes
instead was a decision with security in it: a directory outside the session has
to be made by something privileged, has to be per-person on a machine that may
have more than one, and has to disappear when they sign out, which is the whole
reason `$XDG_RUNTIME_DIR` was chosen. ADR 0017 took that decision and the socket
is now `/run/alo/<uid>/agentd.sock`: the parent is the image's through
`tmpfiles.d`, the person's directory is the daemon's and goes when the daemon
does. The three requirements are met by three different owners rather than by
one directory that met all of them and could not be entered.

**What has still not been done is the measurement that found this**: a
connection from a second real login. The code is built and unit tested, the
image entry does not exist yet (`docs/autonomy/QUEUE.md` item 28), and no claim
that the agent's door works on a real machine is made anywhere until somebody
runs the two users again.
**Date:** 2026-09-03, and the move on 2026-09-04

### On Ubuntu 24.04's aarch64 kernel the verifier refuses the boundary for its stack
**Version:** kernel `6.8.0-134-generic`, aarch64, Ubuntu 24.04 in a Lima 2.2.0
virtual machine on an Apple M3 with 8 GB, `bpf` in the started security modules,
lockdown `none`, JIT on. The Mac lane's first gate run, 2026-09-13.
**Behaviour:** every test that imposes the boundary failed with *"the kernel
would not attach the boundary to file_open"*, which reads as the LSM question
`docs/hardware.md` asks and is not. The source error, printed by a probe calling
`alo_bounding::Imposed::once` directly:
`the BPF_PROG_LOAD syscall returned Permission denied (os error 13). Verifier
output: combined stack size of 3 calls is 544. Too large` — `stack depth
0+304+168`, 62,760 instructions processed of a million. The programme's three
nested calls use 544 bytes against the verifier's 512 for a call chain. The
same tree loads on the development PC's x86_64 6.18 kernel, and on this machine
it loads on Ubuntu's HWE kernel `7.0.0-31-generic` (and `6.17.0-42`): newer
kernels give JIT-compiled sub-programmes stacks of their own and stop adding
them up.
**Our response:** the Mac's VM runs the HWE kernel; nothing in the programme
was changed, because the programme is not this lane's and the certified machine
is x86_64. Two things for `alo-bounding`'s owner: the refusal is **named** as an
attach when it happens at load, which sent this diagnosis to the wrong question
first; and 544 bytes is 32 over a limit that older kernels still enforce, so a
certified machine on an older kernel would refuse the boundary for the same
reason. `docs/hardware.md`'s kernel checks do not ask this.
**Date:** 2026-09-13.

### Setting a file flag under the boundary is refused as unsupported on aarch64
**Version:** kernels `6.17.0-42-generic` and `7.0.0-31-generic`, aarch64, the
same virtual machine; tree `b83d95e`, gates run as root.
**Behaviour:** `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` fails,
every time, with *"an ordinary program can set a flag on its own files: Os {
code: 95, kind: Unsupported, message: "Operation not supported" }"* at
`the_boundary_decides_and_forgets.rs:364`. What was ruled out, by measurement:
the kernel and filesystem accept the same `FS_IOC_SETFLAGS` (`0x40086602`,
`NODUMP` alone and added to the existing flags) on ext4 `/tmp` and on tmpfs with
no boundary loaded; and with the boundary attached and no turn, `chattr +d`,
`lsattr`, `FS_IOC_GETFLAGS` and an unrelated terminal `ioctl` all behave. So the
refusal needs something the test does and the probe did not — the child
program, the attributes and access list set just before, the write-only
descriptor — and which of them was not established.
**Our response:** recorded for `alo-bounding`'s owner and not worked around. It
fails on the untouched tree, it is the one test in 4,088 that does, and it is not
this lane's crate. The Mac lane publishes with it named in every report rather
than ignored or excluded.
**Date:** 2026-09-13.
**Settled, 2026-09-14:** not aarch64 and not the boundary — see *Setting a
file's flags to exactly `nodump` asks ext4 to take its extents away*.

### A check that takes "the first decision" takes whichever the filesystem lists first
**Version:** `crates/alo-citing/tests/every_decision_this_repository_points_at.rs`
at `b83d95e`, run in the Mac lane's virtual machine against the checkout on the
Mac's shared mount (virtiofs).
**Behaviour:** `a_real_decision_taken_off_the_real_list_is_refused` fails with
*"`0023-installed-from-the-machine-it-replaces.md` is linked to by name in this
repository and the check did not notice the file was gone"*. `the_decisions`
reads `docs/decisions/` with `fs::read_dir`, whose order is the filesystem's, and
the test removes `split_first()` of it. On the PC's ext4 that is a decision the
check finds linked by name; on the Mac's shared mount it is 0023, and the check
finds no link to 0023 by its filename among the files it reads (`docs/contracts/`
does name it, and whether those are read was not established) — so the
assertion describes a failure the check had nothing to notice.
It passed again on the same machine an hour later, on a tree with four more
files elsewhere in the repository — which is the finding rather than a relief:
the test's outcome depends on something no change to `docs/decisions/` made.
**Our response:** recorded for the crate's owner: sort the listing, or choose
the decision by a property the assertion needs (one that is linked to by name)
rather than by position. Not changed here, because the crate is not this lane's.
**Date:** 2026-09-13.

### The keyring fixture's bus can still start the machine's own keyring
**Version:** `crates/alo-keyring-fixture` at `b83d95e`; dbus 1.14.10 and
gnome-keyring on Ubuntu 24.04 aarch64.
**Behaviour:** the fixture points its private session bus at an empty
`XDG_DATA_DIRS` so that *"the only Secret Service on it is the one started
below"*. On this dbus that does not hold: a bus started exactly that way lists
`org.freedesktop.secrets` among `ListActivatableNames`, because
`<standard_session_servicedirs/>` adds the compiled-in `/usr/share/dbus-1/services`
as well. When the fixture's own keyring is slow to take its name — as it is in a
small virtual machine — the fixture's first client activates the machine's
keyring instead, the fixture's keyring logs *"another secret service is
running"*, and ten `alo-agentd` tests fail with *"the fixture's keyring never
offered a collection to store a secret in"*. On a faster machine the fixture wins
the race and the isolation merely looks as though it holds.
**Our response:** on the Mac's VM the package's activation file is set aside with
`dpkg-divert --local --rename` (reversible, and listed by `dpkg-divert --list`),
which makes the machine match what the fixture assumes. The fixture itself should
start its bus from a configuration naming no service directories, as its
`from_a_config` path already does; that is the owner's change.
**Then, on 2026-09-15:** made, by task 3 of
`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`. Every bus the
fixture starts is started from its own configuration, which names no
`<servicedir>` and no `<standard_session_servicedirs/>`; `--session` is no
longer used. `crates/alo-secrets/tests/one_keyring_behind_the_secret_portal.rs`
puts a decoy `org.freedesktop.secrets.service` under both `XDG_DATA_HOME` and
`XDG_DATA_DIRS`, shows a plain `--session` bus lists it, and finds the fixture's
bus lists nothing it could activate, with one owner of the name and that owner
the fixture's own keyring. Put back to `--session`, the same test fails naming
`org.freedesktop.secrets` among the activatable names. The `dpkg-divert` on the
Mac's VM is no longer needed for the fixture and is left in place; removing it
is the machine owner's call.
**Date:** 2026-09-13.

### Teuken's GGUF carries no chat template, and the runtime says so and answers anyway
**Version:** `hf.co/mradermacher/Teuken-7B-instruct-commercial-v0.4-GGUF:Q4_K_M`
(the blob is the pinned `sha256:03fd13da…630b`, 5,018,868,512 bytes) under
Ollama 0.34.0 on an Apple M3 with 8 GB. 2026-09-13.
**Behaviour:** loading it, the runtime logs `model is missing
tokenizer.chat_template and Go TEMPLATE support is unavailable; chat responses
may be poorly formatted`. Asked through `/api/chat` to answer with one word, it
answered `I'm ready.<|im_end|>` — the end-of-turn token written into the text,
because nothing told the runtime how a turn is framed for this model. So a
question alo OS puts to Teuken through the pinned runtime reaches the weights in
a shape the model was not trained on, and a grade made that way would be a
measurement of the missing template as much as of the weights.
**Our response, first:** Teuken carried `too-large-for-the-measuring-machine`,
which is what stopped the first run on this machine (the GPU ran out of memory
on the first question). **Then, on 2026-09-14 (task 8):** the catalogue entry
carries openGPT-X's own template, applied when the model is fetched; asked
through it, Teuken answered `" ready."` with nothing stray in the text, loaded on
a second try, and was graded — 0 of 20 freely, 1 of 20 in the envelope. Its
answers cut paths short and misspell them (`/home/anna/Invoic`,
`/home/anna/Invoicnes`), which is the model rather than the template. The template is the finding for
`docs/autonomy/v0-5-the-models-measured-plan.md`'s task 3 — whether the pinned
runtime accepts what alo OS sends — and it matters before anybody grades Teuken
on a larger machine: without a template the grade is not of the weights. A
`Modelfile` `TEMPLATE` for Teuken would be alo OS configuring the engine, which
is allowed; choosing that template is a decision with a name on it and is not
made here.
**Date:** 2026-09-13.

### What 8 GB of unified memory holds, measured with the runtime alo OS pins
**Version:** Ollama 0.34.0 on an Apple M3 with 8 GB, macOS 26.5.2, nothing else
large running and the Linux VM stopped. 2026-09-13.
**Behaviour:** the runtime reports `gpu memory … available="4.8 GiB"
free="5.3 GiB"` on this machine, and llama.cpp aims to leave 1 GiB of that free,
so whether a four-bit model runs on the GPU is decided in the last few hundred
megabytes:

| Artefact | Loaded, as `/api/ps` reports | On the GPU | Outcome |
|---|---|---|---|
| `qwen2.5:7b-instruct-q4_K_M` | — | all | measured, 33 s for the fixed ten |
| `mistral:7b-instruct-v0.3-q4_K_M` | 5.14 GB | 4.63 GB | measured, 36 s |
| `llama3.1:8b-instruct-q4_K_M` | 6.25 GB | 5.20 GB | first run: the GPU ran out of memory on the tenth exercise (`kIOGPUCommandBufferCallbackErrorOutOfMemory`, `llama-server terminated`); second run, nothing else loaded: measured, 66 s |
| Teuken 7B, Q4_K_M | 6.02 GB | — | the GPU ran out of memory on the first question, twice |
| `gemma2:9b-instruct-q4_K_M` | 7.45 GB | 4.13 GB | the rest on the processor; no answer to the first exercise inside five minutes |
| EuroLLM 9B, Q4_K_M | 6.49 GB | — | no answer to the second exercise inside five minutes |

The runtime answers the out-of-memory failure with an HTTP 500, which
`alo_models` reads as `RuntimeError::Unusable` and a person is told as *not with
anything this machine could use*. The measurement harness stops on it rather
than scoring it, which is right: it is the machine failing, not the model.
**Our response:** the three that did not fit carry
`too-large-for-the-measuring-machine`. Nothing was loosened: not the context
window, not the wait, not the quantisation. macOS lets the GPU's share of
unified memory be raised (`sudo sysctl iogpu.wired_limit_mb=<megabytes>`); that
is configuration of the machine rather than of the runtime or the model, and it
needs the owner's password, so it was not done here. With it, Teuken and perhaps
EuroLLM may fit; Gemma 2 9B at 7.45 GB will not on 8 GB.
**Date:** 2026-09-13.

### An image test uses the model this lane measured as its example of an unmeasured one
**Version:** `crates/alo-image/src/checking.rs`,
`weights_naming_a_model_nobody_measured_are_caught`, at `eb658f5`.
**Behaviour:** the test edits a copy of the image's recipe to carry
`mistral-7b-instruct` and expects `Wrong::TheWeightsWereNeverMeasured`, because —
as its comment says — *nobody has put it to `alo-driving`*. On 2026-09-13 the Mac
lane did: 0 of 10, `rarely`, on an Apple M3 with 8 GB under Ollama 0.34.0, with
the ten answers verbatim in
`docs/autonomy/updates/every-catalogue-entry-graded-or-refused-with-the-reason.md`.
Writing that grade into the catalogue makes this test fail with
`checking::tests::weights_naming_a_model_nobody_measured_are_caught ... FAILED`,
because the example has stopped being true, not because the check is wrong.
**Our response:** the grade was held out of `data/catalogue.toml`, named as the
one exception in the two tests that require every entry to be graded or to say
why, and task 2 of `docs/autonomy/v0-5-the-models-measured-plan.md` stayed open.
**Resolved the same evening** by `2cf6025`, which reads the example off the
catalogue; the grade was then written and both exceptions removed.
`alo-image` is not the measuring lane's crate. The change it needs is one line:
an example that is still unmeasured — `teuken-7b-instruct` is, and says why —
or better, an example read off the catalogue (the first entry whose
`drives_verbs` is `not-measured`), so the next grade cannot break it again.
**Date:** 2026-09-13.

### The pinned runtime refuses the Modelfile a brought file was handed over with
**Version:** Ollama 0.34.0 — the release `image/Containerfile` installs — on an
Apple M3 with 8 GB, against `alo-models`' `Ollama::bring` as of `dec0ef1`.
2026-09-13.
**Behaviour:** `bring` sent `POST /api/create` with
`{"model": <id>, "modelfile": "FROM /absolute/path.gguf", "stream": false}`, and
every test in the repository agreed with it, because every test answered with a
socket the repository wrote. The runtime answers that request
`400 {"error":"neither 'from' or 'files' was specified"}`: the `modelfile` field
is not part of its create API. So *point alo OS at weights you already have and
it runs them* was built, tested, and refused by the program it depends on.
What 0.34.0 answered to each road tried, all on loopback with a real GGUF from
this disk:

| Request | Answer |
|---|---|
| `create` with `modelfile: "FROM <path>"` | `400 {"error":"neither 'from' or 'files' was specified"}` |
| `create` with `from: "<absolute path>"` | `400 {"error":"invalid model name"}` |
| `HEAD /api/blobs/sha256:<digest>`, not held | `404` |
| `POST /api/blobs/sha256:<digest>` with the file | `201` |
| the same with the wrong digest | `400 {"error":"digest mismatch, expected …, got …"}` |
| `HEAD` again | `200` |
| `create` with `files: {<name>: "sha256:<digest>"}` | `200 {"status":"success"}` |
| the same, from bytes that are not weights | `500 {"error":"unexpected EOF"}` |
| `/api/chat` by the created id | `"Ready."` |

**Our response:** `bring` now hands the file to the runtime's store by digest and
creates the model from that blob (`crates/alo-models/src/handing_over.rs`); the
fixtures assert those requests, two more carry 0.34.0's listing and answer
verbatim, and `THE_PINNED_RUNTIME` in `alo-models` must equal the image's
`ARG THE_RUNTIME` or a test fails. The runtime was not upgraded. The cost is
stated where it is paid: the runtime keeps its own copy of a brought file, once,
and the digest is read off the whole file first. `from` naming a path being
refused is the good news in the table — there is no spelling of a create that
turns this road into a download.
**Date:** 2026-09-13.

### The runtime samples every answer, so one round of ten is a small sample
**Version:** Ollama 0.34.0 on an Apple M3 with 8 GB, `alo-models`' `/api/chat`
request as of `c6a65ba`, which sets no sampling options. 2026-09-13/14.
**Behaviour:** the runtime's own log prints the sampler for every request, and
for every measurement here it read `temp = 0.800`, `top_k = 40`,
`top_p = 0.900` — the runtime's defaults, because the request names none. So
the same weights answer the same prompt differently from one run to the next.
Measured, not supposed: the Qwen 2.5 7B GGUF (`sha256:2bada8a7…3730`) drove **4
of 10** asked by its catalogue name on 2026-09-13 and **3 of 10** asked by the
name of a file brought on 2026-09-14 — the same bytes, the same ten exercises,
the same door. Both are `rarely`; neither is a different model.
**Our response:** nothing was changed. `alo-models` asks a model the way a real
turn asks it, and a measurement taken at a temperature no turn uses would be a
measurement of a different machine. What follows for the catalogue is stated
rather than hidden: a grade from one round is a grade from ten samples, the bar
is nine of them, and a model near a boundary — five, or nine — deserves a second
round before anybody relies on the line it lands on. `ALO_DRIVING_ROUNDS` is
the harness's own way to take one. The best grade here, Qwen 2.5 7B's four, is
one short of `sometimes` and should get that second round before anybody writes
`sometimes` or `rarely` about it with confidence; none is anywhere near nine,
which is the line that decides the agent.

**The second round was run on 2026-09-14** (task 6): Qwen 2.5 7B drove **8 of
20**, still `rarely`, and the catalogue now refuses a one-round grade within one
attempt of a line.
**Date:** 2026-09-14.

### A boundary fixture takes a control group away while its process is still leaving
**Version:** `crates/alo-bounding/tests/what_a_turn_inherits.rs` on the Mac
lane's Lima VM (Ubuntu 24.04 aarch64, kernel 7.0.0-31-generic, 6 CPUs, 4 GiB),
tree `20bf483` and after. 2026-09-14.
**Behaviour:** tests in that file fail intermittently — under the whole
workspace's load and, less often, run alone — with *"a service can be put back
where it was: Cgroup { what: "cannot take away the control group at", path:
"/sys/fs/cgroup/alo-inherit-reopen-<pid>/home", why: Os { code: 16, kind:
ResourceBusy, message: "Device or resource busy" } }"*. Run alone twice in a row
on the same tree, `an_inherited_descriptor_cannot_be_reopened_through_the_name_the_kernel_gives_it`
passed once and failed once; `a_directory_opened_before_the_turn_began_is_not_a_key_to_what_is_in_it`
failed the same way under load and passed alone twice. `rmdir` on a control
group answers `EBUSY` while any process is still in it, and a child that has
been told to exit has not necessarily left by the time the fixture removes its
group.
**Our response:** recorded for `alo-bounding`'s owner and not worked around;
the crate is not the measuring lane's. The fixture wants to wait for the group's
`cgroup.procs` to be empty (or `cgroup.events` to say `populated 0`) before
removing it. `tools/kernel-loop` already runs a failing gate twice for this kind
of transient; the Mac lane's publish script does not, so it refuses a tree this
flakes on and is run again.

**Then it refused a task twice, and the waiting went into the crate.** On
2026-09-14 the supervisor's gate, in WSL (kernel 6.18.33.2), failed
`a_turn_without_a_boundary_does_not_run.rs`'s
`a_turn_runs_where_the_boundary_is_in_place` the same way on both runs, and the
group it could not remove read `populated 0` minutes later with nothing in it.
The in-place turn is the only one in that file that starts
`Turns::doing`'s keeper thread. Moving a process out through `cgroup.procs`
leaves behind a thread that has already begun to exit, and that thread counts
until it is gone — and it is still *listed* in `cgroup.threads` meanwhile, so
the list cannot tell it from a live one. `Cgroup::removed` now waits, on
`EBUSY` only and for five seconds at most, for `cgroup.events` to say
`populated 0` — the count `rmdir` itself asks — and asks once more; a group
that does not empty is refused with the kernel's first answer, as before. Not
reproduced on demand: the test passed alone three times without the change, and
a probe doing the fixture's sequence 400 times, idle and under eight spinning
processes, never saw `EBUSY`. The explanation is from the kernel's migration
path and the group left behind, not from a reproduction.

**It cascades.** A `home` group left behind in the session scope makes the next
run of `alo-agentd`'s boundary tests fail four at once — *"cannot make a control
group at /sys/fs/cgroup/user.slice/user-501.slice/session-4.scope/home …
AlreadyExists"* — with two pins (`alo-agentd-gone-<pid>`, `alo-agentd-test-<pid>`)
left by processes that no longer exist. On the Mac lane's VM, before every gate
run, a tidy step removes only debris whose process is gone (a pin whose pid is
not running, a group with no processes in it) and says what it removed; nothing
a live fixture holds is touched.
**Date:** 2026-09-14.

### The pinned runtime orders a JSON schema's keys alphabetically
**Version:** Ollama 0.34.0, `/api/chat` with `format` set to a JSON schema, on
an Apple M3 with 8 GB. 2026-09-14.
**Behaviour:** a model held to a schema writes every object's keys in
alphabetical order, whatever order the schema lists its `properties` in. Held to
the protocol's whole call, Qwen 2.5 7B answered
`{"asks":{"read":{"given":[{"is":"/home/anna/Invoices/march.pdf","named":"read_file"}],"verb":"read_file"}},"format":1}`
— `asks` before `format`, `given` before `verb`, and **`is` before `named`**. A
model generates left to right, so it is made to write an argument's value before
it has written which argument it is, and it guessed wrong in fifteen of twenty.
The same model asked freely drove 8 of 20; held to the whole call, 3 of 20.
**Our response:** ADR 0032 holds a local model to the envelope and the door only
— the version and one of `read`, `propose` or `ask` — and leaves the call to the
model, which then writes it in the order the prompt teaches: 35 of 40. The
whole call's schema is not used on this runtime, and the reason is in the ADR so
it is not tried again by default. A later runtime that keeps a schema's order is
a new measurement.
**Date:** 2026-09-14.

### `fetch` pulled the catalogue's id, which the registry does not know
**Version:** `alo-models`' `Ollama::fetch` as of `95b8509`, against Ollama 0.34.0
on an Apple M3 with 8 GB. 2026-09-14.
**Behaviour:** `fetch` sent `POST /api/pull {"model":"mistral-7b-instruct:latest"}`
— the catalogue's own id with the runtime's default tag — and the runtime
answered `{"status":"pulling manifest"}` then
`{"error":"pull model manifest: file does not exist"}`. The registry knows
`mistral:7b-instruct-v0.3-q4_K_M`, which is what the entry's `artefact` names and
what the catalogue's own rule 4 says is fetched. So no catalogued model could have
been fetched by alo OS, and every fixture agreed with the request, because every
fixture was written by this repository. Task 3 put three requests to the real
runtime; this was a fourth it did not.
**Our response:** `fetch` pulls the entry's `artefact`, then creates the model
under the catalogue's id from it — `/api/create {"model": <id>, "from":
<artefact>}`, with the publisher's `template` for the one entry whose file carries
none — so everything else reaches the model by the id a person saw. Walked on the
real runtime for `mistral-7b-instruct` (it answered by that id) and
`teuken-7b-instruct`, by
`the_pinned_runtime_accepts_what_alo_os_sends.rs`'s ignored
`a_catalogue_entry_fetched_answers_by_its_catalogue_id`. The one request that
failed cost a manifest lookup at `registry.ollama.ai`, 2026-09-14 01:32:21 UTC.
**Date:** 2026-09-14.

### WSL on a VMware guest shows `/dev/kvm` and has no KVM behind it
**Version:** WSL 2.7.14, kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu 24.04,
QEMU 8.2.2, in a VMware 7,1 guest running Windows Server 2022 (build 20348).
2026-09-15.
**Behaviour:** `/dev/kvm` is present (`crw-rw---- root kvm 10, 232`), so a check
that only tests for the device reports that virtual machines can be accelerated.
They cannot: `qemu-system-x86_64 -accel kvm -cpu host` fails with *Could not
access KVM kernel module: No such device*, and `/proc/cpuinfo` has no `vmx` or
`svm` flag. WSL's nested virtualisation needs Windows 11, and this host is not.
The node appeared after the distribution was restarted with the virtualisation
packages installed, and it had not been there before.
**Our response:** a virtual machine on this box runs under `-accel tcg`. It
works, and it is slow: Fedora Cloud 42 with 3 CPUs and 4 GB reached its login
prompt in 190 seconds. Probe for acceleration by starting QEMU with
`-accel kvm`, never by looking for the device. Nothing measured under emulation
is a timing for alo OS on a real machine.
**Date:** 2026-09-15.

### `dbus-daemon` 1.14.10 names a caller's process only by its number
**Version:** `dbus-daemon` 1.14.10 (Ubuntu 24.04 aarch64, the Mac lane's Lima VM),
kernel `7.0.0-31-generic`. 2026-09-15.
**Behaviour:** the D-Bus specification added `ProcessFD`, a process descriptor
for the connection's process, to `GetConnectionCredentials` in its revision 0.42
(2023-08-21). This daemon does not send it. Asked about a client connection, it
answers `ProcessID`, `UnixUserID`, `UnixGroupIDs` and `LinuxSecurityLabel`, and no
`ProcessFD`. A backend that reads `/proc/<pid>/…` for that number is reading
about whichever process has the number when it reads. If the caller has ended
and been reaped by then, that may be a different process.
**Our response:** `alo-portals` holds the caller by a descriptor either way
(`crate::caller`, `HeldProcess`). It uses `ProcessFD` when a daemon sends one.
Otherwise it opens a descriptor for the number, checks after reading the sandbox
that the process is still alive, and asks the bus again that the connection
still has the same number. A caller that fails any of these is not identified.
`crates/alo-portals/tests/a_caller_is_named_by_the_process_the_bus_holds.rs`
reproduces the caller ending while its sandbox is read. One window is left open
with a daemon like this one: the caller ends and its number is reused before the
descriptor is opened, **and** the daemon has not yet noticed the closed socket
when it is asked again. Only a daemon that sends `ProcessFD` closes that window.
Which other daemons and versions send it was not measured here.
**Date:** 2026-09-15.

### `bootc install` reads the signature policy of the image it is installing
**Version:** bootc 1.15.1, in `quay.io/fedora/fedora-bootc:42@sha256:077182b6…`
(the base `image/Containerfile` pins), podman 4.9.3 in WSL Ubuntu 24.04.
2026-09-15.
**Behaviour:** `bootc switch --enforce-container-sigpolicy` refuses to stage
anything unless `/etc/containers/policy.json` refuses by default. But an image
that ships such a policy can no longer be installed with the documented
`podman run … <image> bootc install to-disk`. The installer runs *inside* that
image, opens it from the local store under the image's own policy, and stops
before writing a byte: *Fetching manifest: … containers-storage:[…]
localhost/…@sha256:… is rejected by policy*. Nothing says the store is covered
by the policy the image carries for its updates.
**Our response:** the virtual-machine test in
`crates/alo-updating/tests/an_update_keeps_the_persons_things.rs` gives its
images a `containers-storage` scope that accepts anything, beside the default
refusal and the one registry it trusts. For the shipped image this is the
installer lane's decision, and it is named in
`docs/autonomy/updates/an-update-applied-and-the-same-machine-afterwards.md`.
The signed policy for `ghcr.io/aloworld-org/alo-os` and the way the installer
opens the image have to be decided together.
**Date:** 2026-09-15.

### systemd freezes the machine when its generators run past 45 seconds
**Version:** systemd 257.13-1.fc42, in the base `image/Containerfile` pins
(`quay.io/fedora/fedora-bootc:42`), booted by
`crates/alo-updating/tests/an_update_keeps_the_persons_things.rs` under QEMU 8.2.2
`-accel tcg` with 3 CPUs, on the lane machine with no hardware virtualisation (the
entry *WSL on a VMware guest shows `/dev/kvm` and has no KVM behind it*). 2026-09-15.
**Behaviour:** the first boot froze and stayed frozen. The console, with the
kernel's timestamps:

```
[   69.723629] systemd[1]: systemd 257.13-1.fc42 running in system mode (…)
[   76.147140] systemd[1]: bpf-restrict-fs: LSM BPF program attached
[   79.728676] zram_generator::config[612]: No configuration found.
[  121.317082] systemd[1]: Failed to fork off sandboxing environment for executing generators: Protocol error
[  141.425104] NET: Registered PF_VSOCK protocol family
[!!!!!!] Failed to start up manager.
[  141.582882] systemd[1]: Freezing execution.
[  164.914420] systemd-ssh-generator[598]: Failed to query local AF_VSOCK CID: Cannot assign requested address
```

**It is not a unit's sandboxing option.** No `PrivateDevices=`, `ProtectKernel*=`
or `SystemCallFilter=` is involved, and no unit had started. It is PID 1's
deadline for its generators, read from systemd's own source at `v257.9`:

1. `manager_run_generators` (`src/core/manager.c`) forks a child, `(sd-gens)`,
   with `FORK_WAIT | FORK_NEW_MOUNTNS`: the "sandboxing environment" in the
   message. It runs every generator through `execute_directories(…,
   DEFAULT_TIMEOUT_USEC, …)`.
2. `do_execute` (`src/shared/exec-util.c`) arms `alarm()` for that timeout and
   relies on `SIGALRM`'s default action to end the child. Fedora builds systemd
   with `-Ddefault-timeout-sec=45` (`systemd.spec`, `f42`), so all generators
   together get **45 seconds**.
3. The parent's wait sees a child killed by a signal and returns `-EPROTO`
   (`src/basic/process-util.c`), printed as *Protocol error*. That branch falls
   back to running the generators unsandboxed only for a privilege error or
   `-EINVAL`, so it returns the error instead. `manager_startup` fails, and
   `main.c` says *Failed to start up manager* and freezes.

The timestamps agree. The generators started with the manager's early setup at
about 76 s, and the failure came at 121.3 s, 45 s later. What was still running
was `systemd-ssh-generator`. It probes for a hypervisor socket to offer SSH over
it, which loads the vsock modules, and under emulation that took until 141 s
(*Registered PF_VSOCK protocol family*). It finished at 164.9 s, long after its
parent was gone. The same test on the same tree booted and passed an hour
earlier, and again (1050 s) when re-run on its own, so under emulation this
depends on how busy the host is.
**Real hardware or only emulation:** the deadline is compiled into systemd, so
the mechanism exists on every machine this base boots. Reaching it takes
generators that together run longer than 45 seconds. Under hardware
virtualisation or on bare metal they take milliseconds, and loading a module
does not take a minute. So in practice this is a fault of emulating a CPU in
software. It has not been measured on the certified machine, and a boot there
that took seconds in its generators would be worth writing down.
**Our response:** nothing in the product changes. The shipped image keeps its
generators and its sandboxing. The update test's own images mask
`systemd-ssh-generator` with a symlink to `/dev/null` under
`/etc/systemd/system-generators/`, as `systemd.generator(7)` documents, because
nothing the test measures uses SSH. The test also stops waiting as soon as the
console says *Freezing execution*, and fails with *the virtual machine did not
finish booting*, a phrase `tools/kernel-loop` treats as the machine's rather than
the work's.
**Date:** 2026-09-15.

### Going back to the build before does not bring back `/etc` as it is now
**Version:** bootc 1.15.1, in `quay.io/fedora/fedora-bootc:42@sha256:077182b6…`
(the base `image/Containerfile` pins), booted by
`crates/alo-updating/tests/back_to_yesterdays_machine.rs` under QEMU 8.2.2
`-accel tcg`. 2026-09-15.
**Behaviour:** `bootc rollback` reorders the deployments the base already has;
it makes no new one, so no `/etc` merge happens. Each deployment keeps its own
`/etc`, and the earlier build starts with the `/etc` it had when the update
replaced it. A file written under `/etc/alo/` on the newer build was **not
there** after going back, while every file under `/var` — the person's home,
`/var/lib/alo`'s grants, pairings, record and the last-known build — was byte
for byte as it was. `bootc rollback --help` says the same in its own words. So
accounts and passwords (`/etc/passwd`, `/etc/shadow`), `/etc/alo/accounts.toml`
and whole-machine configuration changed since the update do not come back with
the earlier build; they stay with the newer one, and going forward again brings
them back.
**Our response:** nothing is patched (ADR 0011). The sentence a person approves
before going back, `keeping-up.going-back.offered`, says that their files and
their own settings stay as they are and that accounts, passwords and settings
for the whole machine changed since the update go back to how they were. The
test holds the measurement, so if a later base carries `/etc` across a return,
the test fails and the sentence is changed to match. What alo OS itself must
never keep under `/etc` for this reason is the person's own data — which is
already the case: grants, pairings, the record and the indexes live under
`/var`.
**Date:** 2026-09-15.

### A walk from `/` can wedge for good on WSL's Windows-backed mounts
**Version:** WSL 2.7.14, kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu 24.04 on
a VMware guest running Windows Server 2022; `alo-measuring`'s
`naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so` at `f11fa4c`,
inside `cargo test --workspace` with two lanes gating at once. 2026-09-15.
**Behaviour:** the test walks `/`, stopping at each mount point, which on this
machine means it must look at the 9p mounts WSL makes of the Windows side —
`/mnt/c`, `/mnt/d` and `/usr/lib/wsl/drivers`. One `statx` on such a path stopped
answering. The thread sat in `p9_client_rpc` (kernel stack: `v9fs_vfs_lookup` →
`__lookup_slow` → `path_lookupat`) in **uninterruptible** sleep for an hour, writing
nothing and reading nothing; the test's other thread waited on it. `kill -9` does
not end a thread in that state, and `cargo test` has no timeout, so **the gate does
not fail — it never finishes**. Everything else on the same mounts answered
instantly throughout (`ls /mnt/c`, `stat -f /mnt/c`), so the mount was not down;
one request was lost.
**Our response:** `wsl --shutdown`, then start the distribution again; the wedged
thread goes with it, and the lanes re-run their gates. Nothing else clears it. The
same tests had passed many times that day on the same machine, so this is a stall
under load rather than a fault in the test, and the test is unchanged. What it
costs is the gate run it was in, so a lane whose gates have gone quiet for much
longer than usual is worth a look: `ps -eo stat,args | grep " D "` finds the
uninterruptible thread, and `/proc/<tid>/stack` names the filesystem.
**Date:** 2026-09-15.


### Fixed-length lists of crates break when two lanes each add a crate
**Version:** `crates/alo-saying/src/collecting.rs` (`EVERY_LIST`, `ONE_STRING_EACH`)
and `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`
(`WHO_DECLARES_THEM`), as of 2026-09-16.
**Behaviour:** each list is a Rust array with its length in its type, such as
`[&str; 41]`. When two lanes each add a crate in parallel, git merges both new
entries cleanly, because they are on different lines, and leaves the declared length
one short. The combined tree then does not compile (`expected an array with a size
of 41, found one with a size of 42`). Git reports no conflict, and the break appears
only when the combined tree is gated. It happened five times on 2026-09-16 across the
two files, each costing a gate run and a hand-fixed count.
**Our response, and the fix it needs:** the lengths were corrected from the entries
each time. The lasting fix is for these lists to be slices, `&[&str]`, whose length is
not written anywhere; the tests beside them already check every entry against the
workspace, so nothing a fixed length proves is lost. `alo-saying` and `alo-by-hand`
are not the lane's that found this, so the change is left to their owners.
**Date:** 2026-09-16.
**Fixed, 2026-09-17:** `EVERY_LIST` and `ONE_STRING_EACH` are `&[&str]` and
`&[(&str, &str)]`, with no length written on either, and the counts they were
written for are made by comparing one list against another instead of against a
literal. `WHO_DECLARES_THEM` did not become a slice where it was — it moved.
There were two copies of it, in `alo-by-hand`'s test and `alo-software`'s, and
both broke on the day a crate was added rather than only on the day two lanes
added one; they are now one list in `crates/alo-declared/src/shipped.rs`, held to
the workspace's own member list by that crate, with the names and the
`declare_into` calls in one file so neither can move without the other.
`tools/kernel-loop`'s `REGISTRATIONS` names that file now, so a plan adding such
a crate is not refused for registering it.

### Tests leave their folders in /tmp, and enough of them slow a walk of the machine
**Version:** the workspace test suite as gated on the third PC, 2026-09-16.
**Behaviour:** tests in several crates make folders under `/tmp` and leave them when
they finish. Names began `alo-printing-`, `alo-access-unit-`, `alo-keeping-said-`,
`alo-access-test-`, `alo-keeping-disagrees-`, `alo-remembering-writable-` and
`alo-agentd-…-naming-door`, among others. After a day of gate runs `/tmp` held 19,632
entries. `alo-measuring`'s
`naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so` walks `/`, and
with that `/tmp` it ran for over forty minutes, busy on the processor, before it was
stopped. With the old folders removed, the eight tests in its file passed in 23
seconds. A plain `find / -xdev` over the same 393,000 entries took 3.7 seconds, so it
is a directory with tens of thousands of direct children that the walk handles
slowly, not the size of the machine.
**Our response:** on the third PC, a sweep removes `alo-*` folders in `/tmp` older
than two hours, every hour, so gates there stay fast. Two things belong to the
crates' owners: tests that make a folder should remove it, pass or fail, and
`alo-measuring`'s walk should be measured against a folder with tens of thousands of
entries, because a person's machine can have one.
**Date:** 2026-09-16.



### A virtual source made by the media server's own loopback tool cannot be recorded from
**Version:** PipeWire 1.0.5 with WirePlumber 0.4.17, Ubuntu 24.04 aarch64, 2026-09-17.
**Behaviour:** `pw-loopback` will publish a node with `media.class =
Audio/Source/Virtual`, which appears in the graph and in `wpctl status` as an
ordinary microphone. `pw-record` cannot open it: the stream fails at once with
`no more input formats`, and the file it writes is a header and no sound. The node
offers only `F32P` — planar float, fixed at two channels — and no adapter converts
it for a client that asks for anything else. Adding `audio.format`, `audio.rate`,
`audio.channels` and `audio.position` to the playback properties changes nothing.
**Our response:** `alo-sound`'s on-a-machine tests use the **kernel's** loopback
sound cards (`snd-aloop`) instead, which the media server enumerates as ordinary
ALSA devices with adapters, and which a recording tool opens like any other
microphone. A test that must read what a stream carries — which is the only way
*muted means silence* is a measurement rather than a claim — needs a device the
rest of the machine treats as real.
**Date:** 2026-09-17.

### A sound card put in another profile comes back in its default one after a replug
**Version:** WirePlumber 0.4.17, Ubuntu 24.04 aarch64, 2026-09-17.
**Behaviour:** a card set to a different profile with `wpctl set-profile` keeps it
until the card goes away. Unplug it and plug it in again — here, unbinding and
binding the kernel driver — and it comes back in its **default** profile rather
than the one that was set, which means **different node names**: `alsa_output.platform-snd_aloop.0.pro-output-0` came back
as `alsa_output.platform-snd_aloop.0.analog-stereo`. Anything holding the first name
is holding the name of a device that no longer exists, and a first attempt at
`alo-sound`'s mid-call test waited ten seconds for a device that was never coming
back under that name.
**Our response:** worth knowing beyond a test, because it is the one case where a
device's identity does **not** survive a replug: the identity is stable across a
replug of the same card in the same profile, which is what a person's cable does,
and not across a profile that was set and then lost. A profile alo OS wants kept is
configured on the session manager rather than set once at runtime. The test picks
cards offering a single output, which are cards in their ordinary profile.

### A full WSL disk image cannot be shrunk by making it sparse
**Version:** WSL 2.7.14 on Windows Server 2022, and the development PC, 2026-09-17.
**Behaviour:** WSL's `ext4.vhdx` only grows. Files deleted inside the distribution
give nothing back to Windows, and `diskpart compact vdisk` reclaims nothing: the
development PC measured 155.5 GB before and after. The fix that works depends on how
much room the volume has left when you apply it.
- **With room left**, `wsl --manage <distro> --set-sparse true --allow-unsafe`, a
  restart, then `fstrim -av` inside gives the space back. The development PC did this
  with about 20 GB free. Deleting one stale 65 GB build directory afterwards took the
  volume from 2.8 GB free to 53.5 GB.
- **With a full volume**, the same steps give nothing back. On the third PC, D: held
  only the image, a non-sparse 99.7 GB file, and had 0.05 GB free. It was made sparse
  with WSL shut down, then trimmed twice, once of 996 GiB of free blocks. Afterwards D:
  still had 0.05 GB free, and `fsutil sparse queryrange` still showed all 99.7 GB
  allocated. Punching holes is itself a write, and there was no room to make it.
**Our response:** convert every distribution to sparse while its volume still has
headroom. Once the volume is full, export, unregister and re-import. On the third PC:
1. Removed the 200 GB sparse loop file that held the build directories, so the export
   would not write it out at full size.
2. `wsl --export Ubuntu C:\wsl-export\ubuntu.tar` wrote 7.9 GB. `tar -tf` read it to
   the end: 267,099 entries, including the pinned nightly, `bpf-linker`, LLVM and the
   configuration.
3. `wsl --unregister Ubuntu`, then `wsl --import Ubuntu D:\wsl\Ubuntu` from the tar.
4. `--set-sparse` with WSL shut down.
The image came back at 6.9 GB, D: had 92.9 GB free, and nothing had to be reinstalled.
The WSL filesystem had also gone read-only before the fix: when the image could not
grow, a compiler died with SIGBUS and WSL then would not start, error
`Wsl/Service/CreateInstance/E_FAIL`. Freeing 90 MB on D: was enough to start it again.

Two things keep it from happening again:
- The gates' build directories now live on an ext4 loop image with a fixed size
  (80 GB on a 100 GB volume). A build that outgrows it fails as a build, and the machine
  keeps working.
- A change that moves where builds happen doubles disk use while nobody is looking:
  every artefact is built again beside the old ones. The change that moves the
  directory has to delete the one it moves away from in the same step. Both machines
  filled their volume from that cause on the same day.

### `bootc` changes the machine only for root holding `CAP_SYS_ADMIN`
**Version:** `bootc 1.15.1` in the pinned base (`quay.io/fedora/fedora-bootc`,
local image `b035260f985f`), read on 2026-09-17.
**Behaviour:** the program's write commands (`switch`, `upgrade`, `rollback`) check
that they run as uid 0 **and** hold `CAP_SYS_ADMIN`. Its binary carries *This
command requires full root privileges (CAP_SYS_ADMIN)* and *Verified uid 0 with
CAP_SYS_ADMIN*. Being root is not enough. `alo-brokerd` runs as root with both
capability lines of its unit empty, so `alo-updating` run inside it would fail on
every machine. In a container `bootc` refuses first for not being a booted host,
so the capability refusal was read from the program rather than provoked. A booted
virtual machine is where it is confirmed. The same image carries `udisks2
2.10.91`, which needs no capability of its caller: it decides from the caller's
uid through polkit, as NetworkManager does.
**Our response:** the storage verbs are carried out by the broker through udisks2.
The update verbs are answered `not-carried`, and ADR 0053 (proposed) decides how
they are carried out without handing the broker a capability.
**Date:** 2026-09-17.

### No client can open a camera through the media server on WirePlumber 0.4
**Version:** PipeWire 1.0.5 with WirePlumber 0.4.17 (Ubuntu 24.04 aarch64), against the
kernel's own `vivid` video device, 2026-09-17.
**Behaviour:** the camera is in the graph and complete — `pw-dump` lists
`v4l2_input.platform-vivid.0` as an `Audio`-style node of class `Video/Source`, with its
formats, its device file and its serial. **Nothing can attach to it.** `pw-cat --record
--media-type Video --target <name>` fails with `no target node available`, and
`gst-launch-1.0 pipewiresrc` fails with `stream error: target not found` — by name, by
serial and with no target at all. Both are the media server's own tools talking to its
own node.
**Our response:** `alo-cameras` lists cameras and turns them off; the acceptance that
*a test opens the camera through the portal and finds it listed on the indicator* was
**not** taken here, because on this stack no program can open a camera through the
server at all. Ubuntu 24.04 ships WirePlumber 0.4.17 and the video policy people write
about is 0.5's, so a machine that has to do this needs 0.5 — which the certified image
should pin deliberately rather than inherit.
**Date:** 2026-09-17.

**Corrected 2026-09-19: WirePlumber 0.5 does not fix it, and the sentence above
guessed.** *The video policy people write about is 0.5's* was inference from release
notes, not a measurement, and it was repeated into a plan, a report and an image pin
before anybody ran it. Two 0.5 releases were built from upstream source and run against
this same PipeWire 1.0.5 and the same `vivid` device, each in its own prefix with the
packaged 0.4.17 left installed:

| WirePlumber | What happened |
|---|---|
| **0.5.17** (`13d1e445…`) | the V4L2 node is never created: *Failed to activate V4L2 node `v4l2_input.platform-vivid.0`: enum params id:2 (Spa:Enum:ParamId:Props) failed*. Version skew — 0.5.17 is two years newer than PipeWire 1.0.5 |
| **0.5.2** (`24ecc232…`), contemporary with PipeWire 1.0.5 | the node **is** created and is complete — `Video/Source`, `/dev/video0`, state `suspended`, a full YUY2 `EnumFormat` from 320×180 to the largest vivid offers — and **still nothing can attach** |

On 0.5.2, `gst-launch-1.0 pipewiresrc` fails with `target not found` for **every** way of
naming it — `path=<node id>`, `target-object=<serial>`, `target-object=<node name>`, and
with no target at all — and `pw-cat --record --media-type Video` fails the same way. The
node is healthy and addressable and the refusal is identical to 0.4.17's.

**So the blocker is not the session manager's version.** What remains, untested here: the
`xdg-desktop-portal` camera road, which is what the acceptance actually asks for (*an
application reaches a camera only through the portal and its grant*) and which is not
running on this machine; PipeWire 1.0.5's own camera path; and whether `vivid` differs
from a real camera in a way that matters. **A pinned floor of 0.5 in the image is still
right** — 0.4 is the old line and the image shipped no media server at all — but it is
right for those reasons, and *it unblocks the camera acceptance* was never measured and
is now known to be false.
**Narrowed the same day, by a control experiment.** In one session under
WirePlumber 0.5.2, with the same client library and the same `pw-cat` binary:

| What was asked for | Result |
|---|---|
| an **audio** source (`alsa_input.platform-snd_aloop.0.analog-stereo`) | attached and captured **1 908 780 bytes** |
| the **camera** (`v4l2_input.platform-vivid.0`) | *no target node available*, **0 bytes** |

So the session works, the client works, the addressing works, and the client's
permissions work — a client that can attach to one node in a graph is not being
denied by access control on another. **The refusal is video-specific**, and that
eliminates most of what it could have been: not the session manager's version,
not permissions, not the client, not the way the target is named.

**It also eliminates the portal.** `alo-portals` declares `Portal::Camera` with a
grant over `Facility::Camera`, and the obvious next guess was that a camera is
reachable only through a portal handing over a connection. A portal hands out a
*connection*, and connections demonstrably work — the audio capture above used
one. A portal cannot make a link that the graph will not make.

**What is left to test**, and neither has been: whether `vivid` differs from a
real camera in a way that matters — every camera measurement in this repository
is against that one fixture — and PipeWire 1.0.5's own V4L2 capture path. A
second fixture would separate them, and `v4l2loopback` is not it: the version
packaged here (0.12.7) does not build against this kernel.
**Date:** 2026-09-19.

**Both of those were eliminated on 2026-09-20, and neither was it.** Measured on
the **development PC** (Intel Core Ultra 7 155U) in a KVM guest — Ubuntu 24.04.5,
kernel `6.8.0-139-generic`, **x86_64**, PipeWire 1.0.5, WirePlumber 0.4.17. A
different machine and a different architecture from the aarch64 lane above, which
is worth saying because the symptom is byte-identical on both.

The experiment that settles it **takes the camera out**. A synthetic video
stream was published *into* the graph —

```
gst-launch-1.0 videotestsrc is-live=true ! video/x-raw,width=320,height=240 \
  ! pipewiresink mode=provide \
      stream-properties="props,media.class=Video/Source,node.name=lane-b-synth"
```

— so there is **no V4L2, no `vivid` and no kernel device** anywhere in the path.
`pipewiresrc target-object=lane-b-synth` failed against it with the identical
`stream error: target not found` and **0 bytes**, in a session where `pw-record`
captured **1 298 476 bytes** of audio. A refusal that survives the removal of the
camera is not about the camera.

| Candidate | Verdict, 2026-09-20 |
|---|---|
| `vivid` differs from a real camera | **eliminated** — a source that is not a camera at all is refused the same way |
| PipeWire 1.0.5's own V4L2 capture path | **eliminated** — same experiment; there is no V4L2 in the synthetic path |
| WirePlumber's version | already eliminated 2026-09-19; **0.4.17 also *creates* the node**, contrary to this entry's heading |
| permissions / access control | **eliminated from the server's side** — `pw-cli info` reports the client's permissions on the camera node as **`rwxm-`** |
| the node being incomplete | **eliminated** — full `EnumFormat`: YUY2 320×180 with fourteen framerates |
| the client giving no format | **eliminated, and it is a trap** — see below |

**The trap, because it will cost the next person an hour.** With
`PIPEWIRE_DEBUG=3` the client logs
`find_format(): no format given` **immediately before**
`error (-32) target not found`, which reads exactly like the cause. It is not.
Supplying the node's own advertised caps
(`! video/x-raw,format=YUY2,width=320,height=180,framerate=15/1`), addressed by
node id, by node name, and with no target, **failed all three times with the same
error and zero bytes**. `target not found` is a misleading message for whatever
this actually is, and `no format given` is a red herring.

**Also measured, and it is the control the rest needed:** raw V4L2 straight off
the same `vivid` device captured **13 824 000 bytes** at 4.99 fps. The device
works; the graph will not hand it to anyone.

**What is left** is one question inside the media server's own stream connection
— not hardware, not a version, not a fixture — and **no further camera
measurement in this repository needs a camera.**
**Date:** 2026-09-20.

### A video capture that names no kind is refused, and it is two faults not one
**Version:** PipeWire 1.6.2 with WirePlumber 0.5.13 under WSL 2 on the
development PC (Intel Core Ultra 7 155U), and the same pair in a KVM guest on
Ubuntu 26.04.1, kernel `7.0.0-31-generic`; measured against PipeWire 1.0.5 with
WirePlumber 0.4.17 in an Ubuntu 24.04.5 guest, kernel `6.8.0-139-generic`.
2026-09-20.

**Behaviour, and the correction it makes to the entry above.** The sentence
*what is left is one question inside the media server's own stream connection*
was right that the fault was ours, and wrong that there was one of them. Two
were found, and **only one of them is the camera's**.

**The first is a kind that was never named.** A client reading video announces
itself to the session manager, and if it does not say `media.class`, the stream
arrives as `Stream/Input/Unknown`. The linking policy searches by kind, an
unknown kind matches nothing, and `prepare-link.lua` refuses it with
`sendClientError (…, -2, "target not found")` — which is why the message names
a target when nothing is wrong with the target. Measured in one session with no
camera and no kernel device anywhere in it: a synthetic source published with
`gst-launch-1.0 videotestsrc ! pipewiresink mode=provide`, attached by node id,
by node name and with no target. Without the kind, **all three refused, zero
bytes**. With `media.class = Stream/Input/Video`, **all three succeeded,
768 000 bytes each** — 5 frames of 320×240 YUY2, exactly. The same held for a
source shaped like a screen cast (`Stream/Output/Video`) as for one shaped like
a camera (`Video/Source`).

**A downstream element hides it, which is why it looked intermittent.** The
same attach with `videoconvert` after it succeeds *without* the declaration,
because the caps negotiated downstream give the stream a kind by accident. With
`filesink` or `fakesink` directly it fails. A pipeline that happens to work is
not a pipeline that said what it wanted, so the kind is declared rather than
inferred. This is also what `find_format(): no format given` was: **not the
cause of the refusal, and not a red herring either — the same missing kind seen
from the other side.**

**The second fault is the tool's arguments, and it had never worked at all.**
`gst-launch-1.0` takes **each argument as one word** of the pipeline and does
not look inside an argument for spaces. `pipewiresrc path=3 num-buffers=1`
handed over as a single argument is one word, no element is called that, and
the answer is `erroneous pipeline: syntax error` before a frame is asked for.
`alo-capturing` wrote it that way, so **the screenshot road had never carried a
picture on any machine**. With every setting its own argument the same pipeline
returns a real PNG: `320 x 240, 8-bit/color RGBA`, 11 602 bytes.

**And the camera is still refused, which is the part that is not ours.** With
both faults fixed — the stream logged by the session manager as
`Lookup for 'alo-os-capture' (52) / 'Stream/Input/Video'`, so the declaration
demonstrably arrived — a real `vivid` camera refuses every attach on **both**
stacks: by id, by name and with no target, on WirePlumber 0.4.17 and on 0.5.13,
and `pw-cat --record --media-type Video` refuses it too (*no target node
available*). Adding `media.role = Camera` to the capture changes nothing. Raw
V4L2 off the same device in the same session captured 13 824 000 bytes on
24.04 and 4 608 000 on 26.04. `find-defined-target`, `find-default-target` and
`find-best-target` each run and each find no candidate.

**So the earlier conclusion that the portal is eliminated is withdrawn.** It
rested on *a portal cannot make a link the graph will not make*, and the
shipped session manager says otherwise:
`/usr/share/wireplumber/scripts/client/access-portal.lua` keeps an object
manager over exactly the nodes a camera is —
`Constraint { "media.role", "=", "Camera" }` with
`Constraint { "media.class", "=", "Video/Source" }` — and updates client
permissions on them from the portal permission store, for clients matching
`Constraint { "pipewire.access", "=", "portal" }`. A portal does not link; it
grants the permission without which no link is offered. That is a reading of
the engine's own script and **not yet a measurement**: what would settle it is
a client reaching the camera through `xdg-desktop-portal` on a machine with a
desktop session, which this lane has not run.

**Our response:** `alo_in_use::heard::VIDEO_OUT_OF_THE_GRAPH` and
`WHAT_KIND_IT_IS` hold the vocabulary, `alo-capturing`'s `announcing.rs`
declares the kind, and `the_screen_cast.rs` gives every setting its own
argument with a test that holds the shape rather than the joined text. The
camera acceptance stays untaken and its blocker stays, now pointed at the
portal instead of at the version.
**Date:** 2026-09-20.

### On WirePlumber 0.4.17 the camera is not a linkable at all, so no permission can link it
**Version:** PipeWire 1.0.5 with WirePlumber 0.4.17, Ubuntu 24.04.4 aarch64, in
the Lima VM that gates this repository, 2026-09-20. **Measured independently of
the entry above and after it**, by instrumenting WirePlumber's own Lua through
`/etc/wireplumber/scripts/`, which it prefers over the installed copies — a
config override, nothing installed patched (ADR 0011), removed again and the
ordinary fixture restored and verified with 954 412 bytes of captured audio.

**First, the entry above is confirmed on a second stack and by a second
mechanism.** A video client that declares no `media.class` is refused here too,
and on 0.4.17 it happens in a different place: `policy-node.lua`'s `canLink()`
rejects a candidate on its first test, `properties["media.type"] ~=
target_properties["media.type"]`, and an undeclared client's `media.type` is
`nil`, which equals nothing. The instrumented policy printing what it compares:

```
PROBE findDefinedTarget consumer.media.type=nil want.direction=output
PROBE   linkable name=synthetic-camera dir=output mtype=Video canLink=false
```

So the fault spans two WirePlumber major versions through two different code
paths. With `media.class=Stream/Input/Video`, `gst-launch-1.0 pipewiresrc`
attached to a synthetic `Video/Source` and captured **ten buffers, EOS in
0.7 s**.

**Second, and this is new: on this stack the camera is never a candidate,
because it is not a linkable.** The policy chooses a target only from
`linkables_om` — `findDefinedTarget`, `findDefaultLinkable` and
`findBestLinkable` all iterate or look up in it. Printing every member of it at
the moment of a failed attach gives the six loopback audio nodes, the synthetic
`Video/Source`, and the client itself. **The camera is not there.**

It is not there because WirePlumber never makes a session item for it.
Instrumenting `create-item.lua`'s `addItem` shows it called for every audio node
and for the synthetic `Video/Source` — `si-node`, `class=Video/Source` — and
**never** for `v4l2_input.platform-vivid.0`, in a run where that node is present
and complete in the graph.

**This is upstream of permissions, and that is what makes it worth separating
from the portal reading above.** A portal grants permission on a node; the
policy never considers this node at all. On this stack, no permission store
entry could produce a link, because nothing is choosing between candidates that
include the camera.

**The candidate mechanism, recorded as a reading and not a measurement.** The
one anomaly in that node's entire trace is that a parameter enumeration on it
fails where it succeeds on every audio node:

```
enum_params_for_cache_done: <WpNode:49> enum params failed:
    enum params id:2 (Spa:Enum:ParamId:Props) failed
```

| `pw-cli enum-params <node> Props` | Result |
|---|---|
| the camera (`v4l2_input.platform-vivid.0`) | **nothing at all** |
| an audio source (control) | a full `Props` object — volume, mute |

That the failure is *why* the node never reaches `create-item.lua` is **not**
measured, and one observation argues against the simplest version of it: an
object manager with the identical interest, run from `wpexec` in a separate
process against the settled graph, **does** see the node. So whatever excludes
it is inside the daemon's own handling rather than a property of the global.

**What would tell the two explanations apart**, cheaply and before anybody
arranges a desktop session: print `linkables_om`'s members on **0.5.13** at the
moment of a failed camera attach. If the camera is absent there too, the portal
cannot be the explanation on that stack either, and the question is why a V4L2
node never becomes a linkable. If it is present, the portal reading stands and
this entry is a 0.4-only quirk.

**Also worth knowing:** `vivid` was recorded as *eliminated* as a variable on
the reasoning that a synthetic source is refused identically. That control was
itself failing for the undeclared-kind fault, so the two agreed for a reason
unrelated to what was being tested — a correct measurement of the wrong thing.
With the kind declared they stop agreeing: the synthetic source links and
`vivid` does not.
**Date:** 2026-09-20.

### WSL's kernel has no `vivid`; a KVM guest with a stock distro kernel does
**Version:** WSL 2 kernel `6.18.33.2-microsoft-standard-WSL2` on the development
PC, against Ubuntu 24.04.5's `6.8.0-139-generic` in a KVM guest, 2026-09-20.
**Behaviour:** `modprobe vivid` answers *Module not found* under WSL — the
Microsoft kernel ships no `linux-modules-extra` and there is no package that
supplies one for it. In a guest,
`apt install linux-modules-extra-$(uname -r)` then
`modprobe vivid n_devs=1 node_types=0x1` gives `/dev/video0` immediately.
**Our response:** camera work does not happen under WSL directly; it happens in a
guest. Two details that cost time here and are not obvious: the node appears
**`root:root 0600`** because a cloud image has no udev rule for it, so a
`KERNEL=="video[0-9]*" … GROUP="video", MODE="0660"` rule plus
`usermod -aG video` is needed; and **the user's PipeWire session must be
restarted after the group is added**, because a `systemd --user` manager started
before the group change keeps the old supplementary groups and its WirePlumber
then enumerates **no camera at all**. A camera missing from `wpctl status` is far
more often this than anything about the media server.
**Date:** 2026-09-20.

### A guest can be given a TPM 2.0 without `vtpm_proxy`; the host cannot
**Version:** QEMU 10.2.1 and `swtpm` 0.10.1 on the development PC (Intel Core
Ultra 7 155U), WSL 2 kernel `6.18.33.2-microsoft-standard-WSL2`; guest Ubuntu
24.04.5 under OVMF, 2026-09-20.
**Behaviour:** the WSL kernel has `# CONFIG_TCG_VTPM_PROXY is not set`, so
`/dev/vtpmx` does not exist and `modprobe tpm_vtpm_proxy` answers *Module not
found*. That was read as *no TPM is reachable from this machine at all*, and it
is not: `vtpm_proxy` is how a software TPM becomes a device on the **host**. A
**guest** is given one by QEMU's `emulator` backend over a `swtpm` socket, which
needs no kernel module. With
`-tpmdev emulator,id=tpm0,chardev=chrtpm -device tpm-tis,tpmdev=tpm0` the guest
has `/dev/tpm0` and `/dev/tpmrm0`, and
`systemd-cryptenroll --tpm2-device=list` answers
`/dev/tpmrm0  MSFT0101:00  tpm_tis`.
**Our response:** two things to know before trusting either answer. **First**,
systemd 255 is built `+TPM2` but loads libtss2 at runtime, so with a chip
present and `tpm2-tools` not installed the same command says ***TPM2 support is
not installed*** — a sentence about userspace that reads like one about
hardware. **Second**, `/dev/tpm*` is `tss`-owned, so every one of these commands
needs `sudo` or membership of `tss`; without it `tpm2-tools` prints a wall of
TCTI errors ending in *No standard TCTI could be loaded*, which also reads like
a missing chip. And what the guest gets **says what it is**:
`TPM2_PT_MANUFACTURER` is **"IBM"**, `TPM2_PT_VENDOR_STRING_1` is **"SW"**, and
its lockout is the simulator's default (`MAX_AUTH_FAIL 0x3`,
`LOCKOUT_INTERVAL 0x3E8`). See ADR 0056: this is a chip for exercising a
sequence, never evidence about a real chip's lockout.
**Date:** 2026-09-20.

### KVM is real on the development PC, and the third PC's `/dev/kvm` is not
**Version:** QEMU 10.2.1 on the development PC (Intel Core Ultra 7 155U),
WSL 2 Ubuntu, 2026-09-20.
**Behaviour:** the entry *WSL on a VMware guest shows `/dev/kvm` and has no KVM
behind it* is about the **third PC** and must not be quoted as a fact about this
fleet. Here `/proc/cpuinfo` shows `vmx`, `/dev/kvm` works, and it **accelerates**
rather than merely initialising: the same Alpine 3.21 virt image reached a login
prompt in **12.4 s under `-accel kvm` against 27.7 s under `-accel tcg`**, and an
Ubuntu 24.04 guest under OVMF went cold start to SSH login in **23 s**.
**Our response:** the rule in that other entry still stands and is what produced
this measurement — *prove acceleration by booting a guest under `-accel kvm` and
timing it, never by looking for the device file*. What changes is that plans
citing *no machine in this fleet has hardware virtualisation* were citing one
machine. Corrected in `docs/autonomy/v0-5-the-installer-plan.md` and ADR 0056.
**The disk half of those plans' condition is a different matter and is NOT
cleared — see the next entry, which is the same mistake in the other
direction.**
**Date:** 2026-09-20.

### `df` inside WSL reports the virtual disk's size, not the host's free space
**Version:** WSL 2 Ubuntu on the development PC, 2026-09-20. **Written after
filling the host's C: drive to zero bytes by trusting the first number.**
**Behaviour:** `df -h /` inside WSL reported **1007 GB total with 805 GB free**.
That is the **ext4 vhdx's virtual size**. The vhdx is a sparse file on the host's
C:, which is **474 GB with about 13 GB actually free**. Writing ~10 GB of ISOs
and qcow2 images inside WSL grew the vhdx by ~10 GB and took C: to **0 bytes**,
at which point WSL refused to start a new instance
(`Wsl/Service/CreateInstance/E_FAIL`, *failure step: 2*), the agent harness could
not write its own temp files, and every lane sharing the machine was at risk —
not just the one that did it.
**Our response:** **the free-space figure for anything that writes inside WSL is
the Windows volume's, never `df`'s.** Check it from the Windows side
(`df -h /c` under Git Bash, or `Get-PSDrive C`) before downloading an image or
creating a virtual disk. Recovery, once it has happened: delete the files inside
WSL and run **`fstrim -v /`**, which returned the blocks to the host — C: went
from 3 GB free to 11 GB free — **because the vhdx is sparse**. Deleting alone
does nothing; the vhdx does not shrink on its own. Do **not** reach for
`wsl --shutdown` plus `Optimize-VHD` while another lane is working in WSL: it
kills their session, and `fstrim` does the job without it.
**Date:** 2026-09-20.

### A program that opens a camera directly does not appear on the in-use indicator
**Version:** `alo-in-use` as of 2026-09-17, PipeWire 1.0.5, Ubuntu 24.04 aarch64.
**Behaviour:** `alo-in-use` reads the media server's record and counts a **running
source** as a use. A program that opens `/dev/video0` itself never touches the media
server: with `v4l2-ctl --stream-mmap` pulling frames at five a second, the server's own
record still said the camera's node was `suspended`, so the indicator has nothing to
show. The same is true of any program that opens an ALSA device directly.
**Our response:** stated rather than fixed, because it is the honest shape of the
claim: **the indicator shows what goes through the machine's media server, and an
application on alo OS is sandboxed (ADR 0005) and has no other road.** A program a
person runs in their own terminal does, and that is deliberate — alo OS ships a
terminal, and a machine that did not trust its owner with one would be a toy. It is
also exactly why `alo-cameras` holds *off* below the door: with the camera switched
off the machine lets go of the device, so there is no device file for a program of
anybody's to open, asked or unasked.

### `/opt` is a real directory on the pinned base, not a link into `/var`
**Version:** the base pinned in `image/Containerfile`, built 2026-09-17.
**Behaviour:** bootc bases commonly ship `/opt` as a symbolic link into `/var`, and
the recipe replaced it with a real directory before installing the document
converter — because `/var` is machine state a bootc system does not update from the
image, so an engine installed through the link would stop being part of the
read-only image. The pinned base does not do this: `/opt` is already a real
directory. `rm -f` refuses a directory, so the step failed the whole build with
`rm: cannot remove '/opt': Is a directory` the first time the recipe was ever built.
The recipe had said this was unmeasured; it was, until a release needed it.
**Our response:** the replacement is conditional — `if [ -L /opt ]`, done where a
base ships a link and skipped where it does not — rather than assuming either
shape. The `test -x` on the installed engine at the end of the same step is what
proves the engine really landed somewhere it will be found, whichever shape `/opt`
had.

### A dozen networks in one burst: no limit bites, and a port taken on one interface is refused on that interface alone
**Version:** WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu 24.04;
`net.core.rmem_default` 212992, `net.core.optmem_max` 131072,
`net.ipv4.igmp_max_memberships` 20, 10240 open descriptors. Measured by
`crates/alo-agentd/src/a_dock_with_a_dozen_adapters.rs`. 2026-09-17.
**Behaviour:** twelve `veth` cables were laid in one `ip -batch` and addressed and
brought up in a second, while `alo-agentd` was held still with `SIGSTOP`. Six
carried IPv4 and six link-local IPv6 only. The burst **did not overflow** the
service's routing socket: the kernel's `Drops` count for it in
`/proc/<pid>/net/netlink` stayed 0, with every link-local address through duplicate
address detection. (Task 33 needed 64 pairs made and deleted to reach 164 drops.)
Once let go, the service joined, answered and listened on all twelve, and the
kernel refused nothing. The service holds one IPv4 datagram socket per network with
one membership each, so `igmp_max_memberships`, which counts per socket, never
comes near. Its one IPv6 socket held twelve `ff02::fb` memberships within
`optmem_max`.
A TCP listener bound to `0.0.0.0` at the service's port, held to one interface
(`SO_BINDTOIFINDEX`) and without `SO_REUSEADDR`, makes the service's own listener
refused with `EADDRINUSE` **on that interface only**. Its listeners at the same port
held to every other interface, its IPv6-only listener, and its discovery responder
on that same interface are all unaffected. The refusal comes again on every later
round that reads the interfaces, so the service log repeats it each time the kernel
reports a change.
With several IPv4 interfaces in one network, the fixture sends each question to
`224.0.0.251` with `IP_MULTICAST_IF` set to that cable's address rather than rely on
the route to pick the interface.
**Our response:** nothing is limited and nothing is patched. The service's
per-network failure lines already name the network (`crate::responding`,
`crate::listeners`, `crate::networks::joined_on`). The fixture now holds them to
that, and fails if a line is lost or a network is silently skipped. More than
thirteen networks is not measured here; a limit met on certified hardware is written
here and said in the service log where it bites.
**Date:** 2026-09-17.

### A person's service with no capabilities hears every TCP socket in its network destroyed
**Version:** WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu 24.04,
`CONFIG_INET_DIAG=y`, `CONFIG_INET_TCP_DIAG=y`. Measured with a scratch probe as
root, as uid 1000 with `CapEff` 0 in the initial namespaces, and as mapped root in
a user namespace; then by `crates/alo-agentd/src/told_of_a_port_let_go.rs` and
`crates/alo-agentd/src/a_port_another_program_let_go_of.rs`, the latter with the
service under `setpriv --bounding-set=-all`. 2026-09-17.
**Behaviour:** a `NETLINK_SOCK_DIAG` socket may bind to the multicast groups
`SKNLGRP_INET_TCP_DESTROY` (1) and `SKNLGRP_INET6_TCP_DESTROY` (3) with **no
capability at all**; the kernel does not ask for `CAP_NET_ADMIN`, as it does for
most netlink groups. From then on it is sent one `inet_diag_msg` for every TCP socket
destroyed **in its own network namespace**, whoever owned it, with both addresses and
ports — a listener closed, a bound socket that never listened, a connection ended.
The message comes after the socket has left the bind tables: a listener bound at the
same port on hearing it is not refused. A listener closed arrives with `idiag_state`
7 (`TCP_CLOSE`), so the state does not tell a listener from a connection; a socket
whose bind failed arrives with port 0. A classic socket filter attached with
`SO_ATTACH_FILTER` (no capability either) runs on these broadcasts, so a filter
loading the sixteen bits at offset 20 and keeping only one port drops every other
message in the kernel.
**Our response:** `alo-agentd` joins both groups to learn that a program let go of
the port presence advertises, and attaches that filter before it joins, so it never
reads what any other socket on the machine was doing. A message is a reason to try
the port again, not proof it is free. The refusal line for a network whose port is
taken is now said once, when it is first refused, and a line says when it binds.
This changes what the entry above records about the refusal repeating. The
certified image's kernel is not measured here: a kernel without `CONFIG_INET_DIAG`
refuses the join, and the service log then says a taken port is tried again only
when the machine's networks change.
**Date:** 2026-09-17.

### A record from the media server arrived as more than one list, once
**Version:** PipeWire 1.0.5, Ubuntu 24.04 aarch64, 2026-09-17, under a full gate run.
**Behaviour:** `alo-in-use` read `pw-dump`'s record and refused it with *the record is
not readable: trailing characters at line 42599 column 1* — a complete JSON list,
followed by something else. The same machine, asked again a moment later and six times
after that while its graph was changing under it, answered a single list that parsed
every time. **What produced it was not caught**, so nothing here claims to know; the
line number was fifty past the length of an ordinary record, which is consistent with a
second list of whatever changed while the first was being written, and that is as far as
the evidence goes.
**Our response:** the three crates that read that record — `alo-in-use`, `alo-sound`
and `alo-cameras` — now read it as a **stream** of lists rather than as one, taking an
object listed twice as it was listed last. A single list, which is every other reading
there has ever been, goes through unchanged. The reason for tolerating it rather than
insisting: *this machine answered something unreadable* takes the in-use indicator off
a screen while a camera may be on, and that is the one answer it must never give for a
reason nobody can act on.
**Date:** 2026-09-17.

### A machine with the media server's tools installed and no server running
**Version:** `alo-in-use` as of 2026-09-17.
**Behaviour:** `pw-dump` is installed, no session is running, and the tool exits
non-zero with `can't connect: Host is down`. That was read as *what handles sound and
video did not answer* — a server that is broken — when what is true is that there is no
server: an ordinary build host with the package on it, and every machine whose session
has not started yet.
**Our response:** a failure whose text says it could not connect is now
`NothingHandlesSoundAndVideo`, which is what an indicator should say and what makes the
on-a-machine test skip itself rather than fail. It reads the tool's own wording, which
is a thin thread: where that changes, it falls back to *did not answer*, which is still
a refusal and still not an empty indicator.
**Date:** 2026-09-17.

### A dual-stack listener on `[::]` refuses every IPv4 listener held to an interface, and a kernel can bind IPv6 with IPv6 off everywhere
**Version:** WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Ubuntu 24.04. Measured
with a scratch probe in a user network namespace, and by
`crates/alo-agentd/src/listeners.rs` and
`crates/alo-agentd/src/a_port_held_over_ipv6_at_start.rs`. 2026-09-17.
**Behaviour:** two things the specification leaves implicit. **First**, a TCP listener
on `[::]` without `IPV6_V6ONLY` holds the IPv4 port too, and an IPv4 listener at that
port held to an interface with `SO_BINDTOIFINDEX` — loopback included — is refused
`EADDRINUSE` beside it, both with `SO_REUSEADDR`. An IPv6-only listener on `[::]`
refuses none of them. So a program holding the port *dual-stack* takes every network
from the service at once, and one holding it *IPv6-only* takes IPv6 alone.
**Second**, this host's initial network namespace has `net.ipv6.conf.all.disable_ipv6
= 1` and no `::1` on `lo`, yet an `AF_INET6` socket still opens and binds `[::]` at a
port: the kernel has IPv6, the interfaces have none. A connection to `::1` there fails
`EADDRNOTAVAIL`. A fresh network namespace has IPv6 on and `::1` on `lo` once `lo` is
up.
**Our response:** the IPv6-only listener is tried again when the kernel says the port
was let go of only after `EADDRINUSE`, never after anything else
(`crate::listening_over_ipv6`). The unit test that connects over `::1` does so only
where `::1` is there, and the fixture that measures the behaviour connects over
link-local in namespaces of its own, where it always is. A dual-stack holder at start
still stops the service, because nothing at all binds; that is task 37 of the local
network plan.
**Date:** 2026-09-17.

## A Unix socket alone does not authenticate CUPS administration

**Observed:** CUPS 2.4.7-1.2ubuntu7.14 in this machine's Ubuntu WSL, 2026-09-18.
A request to /admin/ over the local socket returned 401 without authentication.
The same request with PeerCred root returned 200 from root with an empty
capability bounding set. CUPS verifies that claimed name against SO_PEERCRED;
a different UID claiming root is not root.

**Response:** the broker uses PrintingService::for_the_broker, which sends only
that fixed identity over a Unix socket. Ordinary and TCP clients do not gain
administrative credentials. The ignored the_real_printing_service acceptance
starts private CUPS queues and the upstream IPP Everywhere emulator: setup,
default selection and removal succeed with all Linux capability sets empty;
an unauthenticated request and a different UID are refused without a queue
change. Discovery input remains the protocol fixture. No physical-printer or
installed-image certification is inferred. See printers-through-the-broker.md
under docs/autonomy/updates for the exact test and recovery history.

### Two test fixtures under one prefix can hand each other's folders to each other
**Version:** `alo-power`, and every crate that copied the shape, 2026-09-18.
**Behaviour:** a test fixture that makes a temporary folder from a process id and a
counter of its own is unique **only against itself**. `alo-power` had two — the battery
fixture in `battery.rs` and the settings fixture in `keeping.rs` — each with a
zero-based counter and both spelling the folder `alo-power-<process>-<counter>`. In one
test process they select the same folder, and since both remove their folder when the
test ends, one test deletes the files another is still using. It fired in a publication
gate as `NotFound` on a battery fixture write and `NotWritable(NotFound)` on a settings
write, and **blocked two other lanes' publications** — neither of which had anything to
do with either crate.
**Our response:** the prefix was separated (`alo-power-battery-`), and then the
dependence on prefixes was removed: every fixture in `alo-sound`, `alo-cameras`,
`alo-power` and `alo-portals` now makes its folder with `create_dir`, which refuses one
that already exists, and tries the next number when it does. Two fixtures can then never
hold one folder however their names are spelled. **The general rule for anybody writing
one: a temporary folder must be made, not made-if-needed** — `create_dir_all` is what
turns a name collision into two tests sharing a directory, and the cleanup that follows
is what turns sharing into data loss.
**Date:** 2026-09-18.

### A release signed with cosign 3 is refused by the policy `bootc` stages under
**Version:** cosign 3.1.3 (the signer named in `image/pinned.toml`), against
skopeo 1.22.2 and containers-common 0.67.0 in
`quay.io/fedora/fedora-bootc:42@sha256:077182b6…` (the base
`image/Containerfile` pins), which is the `containers/image` the base's
`bootc` 1.15.1 stages a build with. 2026-09-19.
**Behaviour:** `cosign sign` 3.x publishes a signature as an **OCI 1.1
referrer**. On a registry without the referrers API — `ghcr.io` among them —
that lands under the fallback tag `sha256-<the build>`, with nothing after it,
and holds an index whose one manifest is
`artifactType: application/vnd.dev.sigstore.bundle.v0.3+json`. A real
signature, and the place holds it. `containers/image` does not look there: a
`sigstoreSigned` policy fetches the **attachment** `sha256-<the build>.sig`,
which cosign 3 does not write unless it is told to. So a build signed by the
owner exactly as ADR 0036's procedure describes is refused with *Source image
rejected: A signature was required, but no signature exists* — measured against
release `0.0.3`, the one `image/pinned.toml` pins, and against `0.0.4`, which
nothing at the place vouches for at all. The two are indistinguishable from
the machine's side, which is the part that matters.
**Our response:** nothing in this workspace lowers what counts.
`alo_looking::vouched_for` reads the name **the base would read** — the `.sig`
attachment — so this machine says *nobody has vouched for it* about a build
its own base refuses, rather than promising an update it cannot install
(`crates/alo-looking/src/vouching.rs`, measured by
`crates/alo-updating/tests/an_offer_a_person_can_act_on.rs`). Bringing the
signature back to where the policy looks is the release process's, which is
the installer lane's under ADR 0036, and it is handed over in
`docs/autonomy/updates/an-offer-a-person-can-act-on.md`. Until it lands, no
alo OS release can be staged under `--enforce-container-sigpolicy`, which is
the safe direction to fail in and is not a direction anybody should be left in
for long.
**Date:** 2026-09-19.

### `bootc` gives a signature refusal the same exit code as every other failure
**Version:** bootc 1.15.1 and containers-common 0.67.0 in the base
`image/Containerfile` pins. 2026-09-19.
**Behaviour:** a build the signature policy will not have and a download that
stopped halfway both come back as an unsuccessful exit with the same code.
Nothing in the status, the code or a separate stream tells them apart; the only
difference is the text, where a policy refusal is `containers/image`'s *Source
image rejected: …*.
**Our response:** `crates/alo-updating/src/genuine.rs` reads what the base
said, for whole clauses only and never for the word *signature* — the base
prints *Getting image source signatures* on the way to a **successful** stage,
so a word match would read a success as a refusal. What it does not recognise
it does not guess at: that stays *the update could not be prepared*, which is
true of everything. This is a weaker thing to depend on than an exit code and
is written down as such; if the base ever gives a distinct code, the reading
should move to it.
**Date:** 2026-09-19.

### `bootc install --filesystem btrfs` makes no subvolume of its own
**Version:** bootc 1.15.1 out of the pinned release 0.0.5
(`ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1`),
btrfs-progs v6.19.1, kernel 6.19.14-101.fc42.x86_64. 2026-09-21.
**Behaviour:** asked for btrfs, the installer runs one `mkfs.btrfs` and nothing
else — no `--subvol`, no layout — and everything afterwards is an ordinary
directory inside the top-level subvolume. The install says so itself:

    podman run --rm --privileged --pid=host --security-opt label=type:unconfined_t \
      -v /var/lib/containers:/var/lib/containers -v .:/output \
      ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1 \
      bootc install to-disk --via-loopback --wipe --filesystem btrfs \
        --karg console=ttyS0,115200n8 /output/disk.raw
    # Creating root filesystem (btrfs) on device /dev/loop0p3 (size=20.9 GB)
    # > mkfs.btrfs -U 91ebb896-8cd6-4a20-8eba-2e97d84614b6 -L root /dev/loop0p3

and the machine it made says the same, on its first start:

    btrfs subvolume list -a -p -u /sysroot
    # (nothing)
    grep btrfs /proc/self/mountinfo
    # 79 82 0:35 /boot     /boot    rw,… - btrfs /dev/vda3 …,subvolid=5,subvol=/
    # 83 82 0:35 /ostree/deploy/default/deploy/ea3e…0.0/etc /etc rw,… subvolid=5,subvol=/
    # 84 82 0:35 /          /sysroot ro,… - btrfs /dev/vda3 …,subvolid=5,subvol=/
    # 35 82 0:35 /ostree/deploy/default/var /var rw,… - btrfs /dev/vda3 …,subvolid=5,subvol=/

`/boot`, `/etc`, `/sysroot` and `/var` are four bind mounts of four directories
in subvolume 5, the filesystem's own root; `/` itself is a read-only composefs
overlay, not btrfs at all.
**Our response:** btrfs is what ADR 0045's undo needs and it is what the
installer names (`alo_image::THE_ONLY_FILESYSTEM`), but the *subvolume* half of
that decision is nobody's yet by default — no home is a subvolume because the
base makes none. Whoever creates a person's account creates the subvolume; this
entry exists so that nobody reads *installed on btrfs* as *has somewhere to
snapshot*. Nothing of ours partitions or lays out subvolumes at install
(ADR 0011): the only argument passed is `--filesystem`.
**Date:** 2026-09-21.

### A person's home lands in `/var/home`, which is a directory and not a subvolume
**Version:** the same install and release. 2026-09-21.
**Behaviour:** the image's login says `/home/alo`, `/home` is `/var/home`, and
on a machine nobody has signed into yet `/var/home` is empty. It is an ordinary
directory, so there is nothing there a snapshot can be taken of:

    getent passwd alo
    # alo:x:1000:1000:alo OS:/home/alo:/bin/bash
    ls -la /var/home
    # total 0
    # drwxr-xr-x. 1 root root   0 Sep 21 00:57 .
    # drwxr-xr-x. 1 root root 266 Sep 21 00:57 ..
    btrfs subvolume show /var/home
    # ERROR: Not a Btrfs subvolume: Invalid argument

A home *made* as a subvolume behaves as ADR 0045 needs, on the same machine:

    btrfs subvolume create /var/home/person
    # Create subvolume '/var/home/person'
    btrfs subvolume snapshot -r /var/home/person /var/lib/alo/undo/before
    # Create readonly snapshot of '/var/home/person' in '/var/lib/alo/undo/before'
    rm -f /var/lib/alo/undo/before/a-file
    # rm: cannot remove '…': Read-only file system

**Our response:** the installer's argument is the half that cannot be changed
later and it is landed; making each home its own subvolume is the accounts
lane's, as ADR 0045 assigns it, and until that lands every undo answers *not yet
on this machine* — which it already does
(`crates/alo-keeping-up/src/putting_back.rs`). A machine installed before this
change is on ext4 and is not converted: it keeps that answer for good, which is
why the argument had to land before the certified laptop was installed.
**Date:** 2026-09-21.

### Taking a read-only snapshot needs no capability; removing one needs `CAP_SYS_ADMIN`
**Version:** kernel 6.19.14-101.fc42.x86_64, btrfs-progs v6.19.1, on the disk
the pinned release installs. 2026-09-21.
**Behaviour:** the two halves of ADR 0045's undo do not cost the same. Taking a
snapshot is ordinary filesystem permission — write access to the directory it
lands in — and no capability at all:

    capsh --drop=cap_sys_admin -- -c 'btrfs subvolume snapshot -r /var/home/person /var/lib/alo/undo/taken'
    # Create readonly snapshot of '/var/home/person' in '/var/lib/alo/undo/taken'
    # take_without_sys_admin_exit=0
    runuser -u alo -- btrfs subvolume snapshot -r /var/home/alo/home /var/home/alo/kept/three
    # Create readonly snapshot … → person_snapshot_exit=0
    runuser -u alo -- btrfs subvolume snapshot -r /var/home/alo/home /var/lib/alo/undo/four
    # ERROR: Could not create subvolume: Permission denied → exit 1

Removing one is not, and the person who took it cannot:

    capsh --drop=cap_sys_admin -- -c 'btrfs subvolume delete /var/lib/alo/undo/taken'
    # ERROR: Could not destroy subvolume/snapshot: Operation not permitted
    # WARNING: deletion failed with EPERM, you don't have permissions …
    # remove_without_sys_admin_exit=1
    btrfs subvolume delete /var/lib/alo/undo/taken
    # remove_with_sys_admin_exit=0
    findmnt -no OPTIONS /var
    # rw,relatime,seclabel,discard=async,space_cache=v2,subvolid=5,subvol=/
    # user_subvol_rm_allowed_present=0

`bootc install` sets no `user_subvol_rm_allowed`, and we do not add mount
options of our own, so deletion is root's on every machine this repository
installs. A read-only snapshot also cannot be cleared with `rm -rf` — that
answers *Read-only file system* — so one left behind stays until something
privileged removes it with `btrfs subvolume delete`.
**Our response:** written down here because ADR 0045's accepted terms turn on
it. Taking the bracket is cheap and needs no privilege the turn does not
already have; **expiring it does** — the seven-day window, the oldest-go-first
under disk pressure and *forgetting is one act* all need a privileged remover,
which is the broker's verb rather than something `alo-turn` can do on its own.
A design that assumed the same authority for both halves would find out on a
machine that had filled with snapshots nothing could delete.
**Date:** 2026-09-21.

### A btrfs snapshot into a destination that already exists is made *inside* it, and says `Read-only file system`
**Version:** btrfs-progs v6.19.1, kernel 6.19.14-101.fc42.x86_64. 2026-09-21.
**Behaviour:** `btrfs subvolume snapshot -r SRC DEST` where `DEST` already
exists does not refuse. It treats `DEST` as a directory and makes the snapshot
at `DEST/$(basename SRC)` — and where `DEST` is itself a read-only snapshot,
that write is refused with `ERROR: Could not create subvolume: Read-only file
system`, which reads exactly like a filesystem mounted read-only.
**Our response:** it cost this task two measuring boots and a wrong conclusion:
a run that left a snapshot behind made the next run's *taking a snapshot needs
`CAP_SYS_ADMIN`* look measured, when what had happened was a second snapshot
landing inside the first. Anything that takes a bracket names a destination
that does not exist yet and checks the result, rather than reading `EROFS` as a
mount problem; and any measurement of this is made on a destination nothing
has touched.
**Date:** 2026-09-21.
