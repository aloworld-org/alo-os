# The installer, walked on a real Windows and killed at every step

**Date:** 2026-09-21
**Workstream:** the installer plan's task 10
**Contributor:** Claude Code, lane A (`/root/alo-os`, WSL Ubuntu on the
development PC, Intel Core Ultra 7 155U, 15.5 GB)
**Status: not done — most of it measured, three things still owed, and nothing
is ticked.** What was measured is below with its evidence; what was not, and
what it needs, is under *What is still owed*.

## In one paragraph

A Windows 11 installed itself into a QEMU machine with no hand on the keyboard
and reached a desktop session. The installer, built for Windows with a genuine
environment beside it, ran elevated there, typed the name it itself showed,
and was killed after each of `staging.rs`'s seven steps; the landing was read
back from Windows' own tools and six of the seven landed exactly (step 4's
boundary is shorter than the walk can react and was never landed). After every
kill Windows restarted to its desktop session. Every file of the Windows
partition and of its start partition was hashed from the host with the machine
off and held to two controls; steps 2, 3 and 6 changed nothing beyond them, and
steps 1, 5 and 7 left one or two user-profile cache files each that the
controls do not explain. `Resize-Partition`, `New-Partition`, `Format-Volume`,
`Add-/Remove-PartitionAccessPath` and `bcdedit` did exactly what
`crate::program` asks. On the installer's own restart the firmware started the
area's loader, and — on Fedora's firmware — the environment came up, looked for
`ata-QEMU_HARDDISK_ALOTARGET1`, found it, checked it and partitioned it: the
SATA name, seen from both sides. Found on the way: entries `bcdedit` copies
from `{bootmgr}` carry Windows' optional data, which shim misreads and falls
back past; and once, after a kill at step 7 and a *shutdown*, the firmware's
entry pointed at Windows' own partition instead of the area.

## The machine, and why it is QEMU

QEMU 10.2.1 `q35` under KVM, not Hyper-V: the account the tests run under on
this PC cannot manage Hyper-V (task 2's report). UEFI (OVMF), GPT, a `swtpm`
TPM 2.0 per machine, 3072 MiB, and two disks on one AHCI controller with
serials the test gives them — a 64 GB Windows disk `ALOWINDOWS1` and an empty
32 GB second disk `ALOTARGET1`. The walk's instructions and the download arrive
on a CD, deliberately: a third *disk* would be a disk `Get-Disk` reports and
the installer decides from. Every destructive step is inside a machine the
walk made; the host's disks are never touched, and every Windows disk the
walk used is an overlay of one installed image.

**The Windows:** Windows 11 Enterprise Evaluation 25H2 (26200.6584),
`https://go.microsoft.com/fwlink/?linkid=2334167`, 7 092 807 680 bytes, no key,
no form. The unattended install took **32 minutes** from the first key to the
desktop (08:06–08:38). The answer file wipes disk 0 only.

## Secure Boot is off in this machine, and only in this machine

ADR 0033 §4: the shipped installer **refuses to run with Secure Boot on**, and
this walk runs it. The firmware is OVMF's Secure Boot capable build with
nothing enrolled, so it is off *and answerable*: the installer said *Secure
Boot is off*. The build without Secure Boot could not answer
`Confirm-SecureBootUEFI`, which `deciding.rs` refuses as firmly as *on*.

## What was measured

Every result below comes from a script file inside WSL whose first line printed
`probe_should_be_7=7`.

### Windows, before anything was done to it

- **Settled first.** An overlay of the Windows as Setup left it spent
  1 011–1 015 s on its first start (the first PowerShell alone took four
  minutes); after one settling start of the installed disk itself (hibernation
  off, ten minutes, a shutdown Windows asked for) an overlay's first start took
  **141 s**. Base before: `0cc8f02a…`; settled: `05451f58…`.
- **The installed Windows, read from the host:** 250 141 files on the Windows
  partition, 146 on the start partition, **0 unreadable, 0 unlistable**, in
  593 s.
- **Two controls, each read after one start and after two:** the same Windows
  with the installer never run; and the installer run the same way and
  **refused at the consent** — it said *You stopped the installer, so nothing
  was changed*, exited 1, the start files were unchanged around it, and its
  partition table was the installed one.

### The seven kills

The kill lands on the step's *effect*, not a sentence: the installer is frozen
(`NtSuspendProcess`) while its own PowerShell child finishes steps 1, 2, 3 and
6, terminated when the fourth PowerShell child appears (step 5, all four
`bcdedit` calls behind it), and terminated the moment `chosen.cfg` exists
(step 4) or the next start is set (step 7). Afterwards the facts are read from
Windows' own tools and the landing is said, never assumed.

| step | killed at | landed | what Windows' tools showed after the kill | restarted to the desktop | Windows partition beyond both controls |
|---|---|---|---|---|---|
| 1 shrink | 73.3 s | exactly | `C:` 67 496 837 120 → 66 423 095 296 bytes (−1 GiB); no area | yes, 140 s | 1 cache file |
| 2 area made | 91.1 s | exactly | partition 4 at 66 872 934 400, 1 073 741 824 bytes, basic data, no file system | yes, 128 s | nothing |
| 3 prepared | 87.7 s | exactly | FAT32 `ALO-INSTALL`, letter `D:`, no file in it | yes, 169 s | nothing |
| 4 copy | — | **never** | after step 6; after step 5; and twice in the middle of step 5 (the entry copied, not yet pointed) | yes, after the first two | not compared |
| 5 entry | 120.3 s | exactly | entry `alo OS` complete and listed last; letter still there | yes, 123 s | 1 temp file |
| 6 letter taken | 97.0 s | exactly | entry complete; no letter; no next start | yes, 99 s | nothing |
| 7 next start | 144.2 s | exactly | `bootsequence {d781f2cb…}` | first restart: the firmware started `alo OS` — from Windows' partition (below); second: yes, 94 s | 1–2 cache files |

The *restarted* column is the time from power-on to the walk having signed in
and read its state. Every one of these boots printed `console alo … Active` and
`explorer.exe … Console` — seventeen in all, controls included. The start
partition changed beyond the controls after no step. **Resets:** every boot in
the table needed none. One earlier attempt at step 2 — a fresh overlay before
the installer had run — did not sign in after two resets; QEMU itself had
stopped (monitor silent, every thread on a futex, 56 MB resident), so it was
killed and step 2 walked again from a fresh overlay. The host slept 16:33–18:03
and again near 20:36; the second voided one step 4 run, whose kill boot never
signed in.

**The paths the controls do not explain**, all of them:
step 1 `Users/alo/…/StartMenuExperienceHost…/CryptnetUrlCache/Content/B76BE…`;
step 5 `ProgramData/Microsoft/Windows/WER/Temp/afd2f54d-…`;
step 7 `Users/alo/…/Client.CBS…/TempState/TileCache_100_4_PNGEncoded_Data.bin`
and `…/AC/Temp/e5c2c870-….tmp`. Per-user shell caches and error reporting's
temp directory — nothing under `Windows\`, `Program Files` or the start
partition, and nothing any program of `crate::program` writes. They are not
waved away: the test holds them as failures.

**How the rule was reached, since it was changed twice:** against the plain
control alone, step 1 left 32 paths outside the noise — the WMI repository,
Defender's scan history, the TPM's key cache — Windows' bookkeeping of any
program having run. So the refusal control was added. Then the TPM key cache
sits under a directory named by each chip's key hash, different on every
machine by construction, so identifier-shaped path components (40-hex hashes,
`{GUID}`s) are compared as `<id>`. Nothing else is rewritten, and no path is
listed by hand.

### The three things task 3 said only a running Windows could show

1. **The storage cmdlets and `bcdedit` do what `crate::program` asks** — yes,
   read back after every step above, to the byte offset and size.
2. **The names `naming.rs` makes are the names the environment finds** — **for
   SATA, yes.** Windows: bus `SATA`, model `QEMU HARDDISK`, serial
   `ALOTARGET1`; the installer wrote `set alo_installing_to=ata-QEMU_HARDDISK_ALOTARGET1`;
   the environment said *Looking for the disk you chose:
   ata-QEMU_HARDDISK_ALOTARGET1*, *Checking that … is safe to install onto*,
   and the kernel then showed `sdb: sdb1 sdb2 sdb3` on the 32 GiB disk. NVMe
   and Hyper-V SCSI: not seen (below).
3. **A copy of `{bootmgr}` with a `device` and `path` is an entry the firmware
   starts** — **yes, with two findings.** On the installer's own restart the
   firmware printed `starting Boot0002 "alo OS" from
   HD(4,GPT,…,0x7C8F800,0x200000)/\EFI\BOOT\BOOTX64.EFI` — the area, exactly
   (both firmware builds). But:
   - **it carries Windows' optional data** (136 bytes beginning `WINDOWS`, read
     from the variable itself), which shim takes as a file to start: *Failed to
     open \EFI\BOOT\䥗䑎坏S*, then *falling back to default loader*. On Fedora's
     `edk2-ovmf` 20250812-21 the fallback reached GRUB and Linux with
     `alo.installing.to=ata-QEMU_HARDDISK_ALOTARGET1`; on Ubuntu's OVMF 2025.11
     it page-faults — task 9's firmware fault, not this one.
   - **once it pointed at the wrong partition.** After the kill at step 7 and
     a `shutdown /s`, the firmware's `Boot0002` held
     `HD(1,…)/\EFI\BOOT\BOOTX64.EFI` — Windows' own partition, whose fallback
     loader is Windows — and the next start honoured the next-start choice and
     came up in Windows. Probes on another machine: `bcdedit` wrote the area
     for `device`-then-`path`, `path`-then-`device` and `device` twice, and it
     stayed so a minute later and across a full shutdown; a volume path left it
     on Windows' partition; a load option written directly with
     `SetFirmwareEnvironmentVariableEx` held the area with no optional data.
     The machines after steps 4, 5 and 6 and both roads held the area. What
     rewrote step 7's is **not explained**.

### The road, run once — twice, on two firmware builds

The installer, not killed: it said *Everything is ready. This computer
restarts in a few seconds*, exited 0, and restarted Windows itself. Nothing
said what to start. The firmware started the `alo OS` entry from the area.
On Fedora's firmware the environment started and **found the disk by the name
the installer wrote**; `bootc` then stopped with
*error: Installing to disk: Creating rootfs: No such file or directory (os
error 2)*, and the environment said *That disk may now hold part of alo OS;
nothing else on this computer was changed*. The install that finishes is task
12's; this is new ground for it (task 9 stopped at `bwrap: pivot_root`).

## What is still owed

- **The test, run end to end, with its run pasted.** The measurements above
  were made by the shell harness under `/root/t10` running the same guest
  scripts this change commits (`tests/walking/guest/`); the Rust test was
  written to the same design, compiled, and its non-ignored test passes, but
  the three ignored tests have not been run by name. One end-to-end run is
  about six hours on this PC.
- **Step 4's boundary.** Between the copy's last write and the first `bcdedit`
  the installer takes less time than the walk can see and act on: polling the
  effect landed after step 6; freezing then listing children landed after
  step 5; a busy wait on `chosen.cfg` and an immediate kill landed in the
  middle of step 5, `bcdedit /copy` already done (and so did the probe run).
  Landing it needs the walk to stop the installer *on* the write — a debugger
  break on the file's close, not a poll.
- **Byte-for-byte after steps 1, 5 and 7.** The four cache files above; more
  control runs would show whether they are run-to-run noise.
- **The step 7 entry pointing at Windows' partition**, unexplained, one
  machine.
- **NVMe and Hyper-V SCSI names** from the Linux side: an NVMe device that
  reports an identifier, and a Hyper-V machine.
- **The release's own MSVC build.** The walk cross-builds
  `x86_64-pc-windows-gnu` (imports only `KERNEL32`, `msvcrt`, `ntdll` and one
  API set); that the MSVC build behaves the same is not shown.

## Limitations and costs

- The host's C: went from 62 GB free to 39 GB and back and forth with each
  overlay; at the end it had **39 GB** free (40 GB mid-run). `/root/alo-os/target`
  holds 71 GB of a stale build cache that is not this task's.
- A Windows guest beside a release build starved WSL's relay for an hour and
  lost the first install (`docs/quirks.md`).
- `docs/quirks.md` gained nine entries from this walk.
