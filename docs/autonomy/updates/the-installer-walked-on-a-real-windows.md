# The installer, walked on a real Windows and killed at every step

**Date:** 2026-09-21, continued 2026-09-22
**Workstream:** the installer plan's task 10
**Contributor:** Claude Code, lane A (`/root/alo-os`, WSL Ubuntu on the
development PC, Intel Core Ultra 7 155U, 15.5 GB)
**Status: not done.** The Rust test now runs by name end to end; the install
and the road pass; the kill test fails on files Windows itself changes that the
controls do not yet explain. Nothing is ticked. What was measured is below with
its evidence; what was not, and what it needs, is under *What is still owed*.

## In one paragraph

A Windows 11 installed itself into a QEMU machine with no hand on the keyboard
and reached a desktop session, under the Rust test run by name. The installer,
built for Windows with a genuine environment beside it, ran elevated there,
typed the name it itself showed, and was killed after each of `staging.rs`'s
seven steps. The kill is landed on the step itself, and the landing is read back
from Windows' own tools: all seven landed exactly. After every kill Windows
restarted to its desktop session. The start partition never changed beyond the
controls. The Windows partition changed, after every kill, in per-user shell
and web caches, Defender's scan history, Windows Terminal's state and one WMI
file; the controls do not explain these, and the test holds them as failures.
The installer now writes its start-up entry itself, **with no optional data**,
read back from the firmware variable. On the installer's own restart the
firmware started it from the area, shim went straight to GRUB, and the
environment found `ata-QEMU_HARDDISK_ALOTARGET1`, the name the installer wrote.

## The machine, and why it is QEMU

QEMU 10.2.1 `q35` under KVM, not Hyper-V: the account the tests run under on
this PC cannot manage Hyper-V (task 2's report). UEFI (OVMF), GPT, a `swtpm`
TPM 2.0 per machine, 3072 MiB, and two disks on one AHCI controller with
serials the test gives them — a 64 GB Windows disk `ALOWINDOWS1` and an empty
32 GB second disk `ALOTARGET1`. The walk's instructions and the download arrive
on a CD, deliberately: a third *disk* would be a disk `Get-Disk` reports and
the installer decides from. Every destructive step is inside a machine the
walk made; the host's disks are never touched, and every Windows disk the
walk used is an overlay of one installed image. The guest has user-mode
networking (QEMU `-netdev user`), so it can reach the internet.

**The Windows:** Windows 11 Enterprise Evaluation 25H2 (26200.6584),
`https://go.microsoft.com/fwlink/?linkid=2334167`, 7 092 807 680 bytes, no key,
no form. The answer file wipes disk 0 only.

## Secure Boot is off in this machine, and only in this machine

ADR 0033 §4: the shipped installer **refuses to run with Secure Boot on**, and
this walk runs it. The firmware is OVMF's Secure Boot capable build with
nothing enrolled, so it is off *and answerable*: the installer said *Secure
Boot is off*. The build without Secure Boot could not answer
`Confirm-SecureBootUEFI`, which `deciding.rs` refuses as firmly as *on*.

## The Rust test, run by name

`cargo test -p alo-installer --test the_installer_walked_on_a_real_windows --
--ignored --exact --test-threads=1 --nocapture <name>`, detached from any
session through a `systemd-run` unit that ran a script file inside WSL. Every
result file begins and ends with `probe_should_be_7=7`. `uptime` at start and
end matches the run's seconds in each start, so **the host did not sleep during
any of them** (it is on the charger with sleep disabled).

| start | head | tests | result | why it stopped |
|---|---|---|---|---|
| 1, 02:09 | `caf26df4` | all three | 0 passed, 3 failed, 1 008 s | the desktop session was judged on the guest's first line, before the shell's line (fixed, `3956d51a`); the two others died on the poisoned lock |
| 2, 06:21 | `da3f4ebd` | all three | none | the second machine was refused its security chip: `swtpm` ends when a machine disconnects (fixed, `cdee38b6`) |
| 3, 06:45 | `cdee38b6` | all three | **install passed**; 2 failed; 3 222 s | an internet outage: the environment's container build could not resolve `index.crates.io` |
| 4, 08:07 | `cdee38b6` | kill, road | 0 passed, 2 failed, **17 261 s** | kill: 15 findings (below); road: the test's own reading (fixed, `b0dc191a`) |
| 5, 12:57 | `b0dc191a` | road | **passed**, 1 659 s | — |

```
probe_should_be_7=7
start=5 (see the note at the top of road-run5.sh)
head=b0dc191a25312de41cd59050a70b29e7cf74c3a4
dirty=0
base=16497442816 2026-09-22 07:20:14.999753291 +0200
started=2026-09-22T12:57:30+02:00
uptime_at_start=57464.91
host_free_before=77G
cargo_test=0 seconds=1660
finished=2026-09-22T13:25:11+02:00
uptime_at_end=59125.09
host_free_after=78G
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1659.22s
probe_should_be_7=7
```

The fourth start's two failures were two defects in the test's own reading. The
fifth start measured one of them again; the other is measured from the road:

- **The road's step 4 name.** The guest printed
  `223.1s the-name-written: ata-QEMU_HARDDISK_ALOTARGET1`. The test matched
  only lines that *begin* with the marker, and the walk stamps every line with
  its clock. The marker is now found anywhere in the line, and the fifth start
  passed with it.
- **Step 7's area.** The test read the area's first sector off the killed
  boot's console *after* the restart had started a fresh console in the same
  file, so it read `None`. The firmware itself had printed that it started
  `alo OS` from partition 4 at sector 130 611 200 (`0x7C8F800`). On the same
  installed Windows the road printed the area at byte offset 66 872 934 400,
  which is that sector. The test now reads the area first. The kill test has
  not been run again since.

## What was measured

### The start-up entry, without Windows' optional data

`bcdedit` cannot make an entry without the 136 bytes of optional data that
start with `WINDOWS`. Each probe was read back from the variable:

- a `{bootmgr}` copy with every boot-manager value deleted still carries 136
  bytes;
- `/create /application firmware` is refused;
- a copy of a firmware application keeps its original path whatever
  `device` and `path` say.

So the installer's step 5 (`WritingTheEntry`, `4d6914a1`) now writes the load
option itself with `SetFirmwareEnvironmentVariableEx` and reads it back byte for
byte. The load option holds the area's hard-drive node, with the slot taken from
the disk's own GPT, then `\EFI\BOOT\BOOTX64.EFI`, and nothing after it. It hands
`bcdedit` the identifier Windows lists the entry under, which `bcdedit` then
uses for ordering, the next start and removal. The installer does not copy
anything anywhere. The fifth start's guest read the variables back:

```
395.1s bootoption Boot0002: [alo OS] slot=4 start=130611200 file=[\EFI\BOOT\BOOTX64.EFI] optional-data=0 bytes
395.1s bootoption Boot0004: [Windows Boot Manager] slot=1 start=2048 file=[\EFI\Microsoft\Boot\bootmgfw.efi] optional-data=136 bytes
BdsDxe: starting Boot0002 "alo OS" from HD(4,GPT,25358A13-…,0x7C8F800,0x200000)/\EFI\BOOT\BOOTX64.EFI
```

The shell harness ran the road on both firmware builds. On **Fedora's
`edk2-ovmf` 20250812-21** there was no *Failed to open* and no *falling back*:
shim started GRUB directly, and Linux booted with the name. On **Ubuntu's OVMF
2025.11** the output was the same up to the #PF page fault. That fault is task
9's firmware fault, measured there without any entry. Windows rewrote a variable
written with its own partition number (5) to the GPT slot (4) within twenty
seconds. The installer writes the slot itself (`docs/quirks.md`).

### The seven kills

The kill lands on the step's *effect*, not on a sentence the installer prints:

- **Steps 1–3:** the installer is frozen (`NtSuspendProcess`) while its own
  PowerShell child finishes the step.
- **Steps 4–7:** the next program the installer starts is held. Image File
  Execution Options hand it to a stand-in, which writes down the command line it
  was given and never returns. Step 4 is held at the entry's PowerShell, step 5
  at the letter's PowerShell, step 6 at `bcdedit … bootsequence` and step 7 at
  `shutdown.exe /r`.

All of this is in the walk's guest scripts. None of it is in the installer
(`caf26df4`). A run in which the installer had begun putting back is never
counted as landed. In the shell harness and the Rust test alike, **all seven
landed exactly**, and Windows restarted to its desktop session after every one.
The fourth start printed `step N … walked` for all seven steps.

The kill test's own rule: a path counts as a finding when its content changed
from the base after a kill, or after the restart that follows it, and none of
these explains it:

- **the controls:** the installer never run, run and refused at the consent,
  and run and killed at the consent, each read after one start and after two;
- **the noise:** a directory in which the controls disagree with each other.

Identifier-shaped path components (40-hex, `{GUID}`) are compared as `<id>`.
Nothing is listed by hand.

The fourth start's controls changed 5 331 paths after one start and 5 424 after
two, and disagreed in 721 directories. **Nothing was unreadable.** Its 15
findings:

| after | start partition | Windows partition, beyond the controls |
|---|---|---|
| kill 1 / restart | nothing / nothing | 7 / 50 paths |
| kill 2 / restart | nothing / nothing | 21 / 46 |
| kill 3 / restart | nothing / nothing | 19 / 41 |
| kill 4 / restart | nothing / nothing | 51 / 60 |
| kill 5 / restart | nothing / nothing | 60 / 60 |
| kill 6 / restart | nothing / nothing | 60 / 60 |
| kill 7 / restart | nothing / nothing | 20 / 41 |

The fifteenth finding was step 7's area reading `None`, the test defect above.
The first fourteen are all in these places:

- `Users/alo/AppData/Local/Microsoft/Windows/INetCache/IE/<8 characters>/08b7573a…[1].xml`;
- `…/MicrosoftWindows.Client.CBS…/AC/INetCache/<8 characters>/th[1].svg`, and
  that package's `EBWebView` component data (`ZxcvbnData/3.2.0.0`, `Speech
  Recognition/1.15.0.1`, `Service Worker/CacheStorage`);
- `ShellExperienceHost…/Settings/settings.dat` and `roaming.lock`;
- Windows Terminal's `LocalState/state.json`, `settings.json` and `Helium`
  hives;
- `ContentDeliveryManager…/TargetedContentCache`;
- `OneDrive/StandaloneUpdater/*.json`;
- `ProgramData/Microsoft/Windows Defender/Scans/History/…`;
- once each, `Windows/System32/wbem/Performance/WmiApRpl_new.ini` and a lock
  screen image.

None is under `Windows\Boot`, `Program Files` or the start partition. None is
anything a program of `crate::program` writes.

**What the census shows, and what it does not.** Each of these kinds exists in
the base and in the controls too:

- `settings.dat`, `state.json`, `ZxcvbnData`, `TargetedContentCache` and
  `ECSConfig.json` are in all 21 readings of the shell harness;
- the IE-cache xml is in all six refusal and kill-at-consent readings, each time
  under a *different* randomly named 8-character directory;
- Defender's latency history is in every reading in which the installer ran,
  and in neither plain control.

So these are files Windows writes over time, not files the kills create. Two
things plausibly explain why the controls do not cover them:

- The IE cache's container directories are random names, like the TPM key hash
  that made the `<id>` rule necessary. The test does not normalise them, and it
  was not widened by hand to do so.
- The guest is online. Edge WebView's components and the content delivery cache
  arrive from Microsoft's servers when Windows chooses, and the controls ran
  hours before the later steps.

Neither is proved. The kill test still fails, correctly.

### The three things task 3 said only a running Windows could show

1. **The storage cmdlets and `bcdedit` do what `crate::program` asks.** Yes.
   This was read back after every step, to the byte offset and size, with the
   firmware variable read back for the entry.
2. **The names `naming.rs` makes are the names the environment finds.** **For
   SATA, yes**, and this time under the Rust test. The installer wrote
   `ata-QEMU_HARDDISK_ALOTARGET1` (the road's step 4). The kernel was handed
   `alo.installing.to=ata-QEMU_HARDDISK_ALOTARGET1`. The environment said
   *Looking for the disk you chose: ata-QEMU_HARDDISK_ALOTARGET1* and
   *Checking that ata-QEMU_HARDDISK_ALOTARGET1 is safe to install onto*. NVMe
   and Hyper-V SCSI were not seen (below).
3. **An entry the firmware starts.** Yes, now written without optional data
   (above). Both the kill after step 7 followed by a shutdown and the road
   started it from the area.

### The road

The installer was not killed. It said *Everything is ready*, exited 0 and
restarted Windows itself; nothing else told the firmware what to start. The
firmware started `alo OS` from the area, and the environment **found the disk
by the name the installer wrote**. `bootc` then stopped with *error: Installing
to disk: Creating rootfs: No such file or directory (os error 2)*. That is now
the plan's task 16 (status *ready*, with its output and a repro). The install
that finishes is task 12's.

## What is still owed

- **The kill test's findings.** 14 findings, each explained by the census but
  not by a control. The next measure is a control, not a rule. These would test
  the two explanations above:
  - cut the guest's network, since the installer needs none;
  - take a plain control boot next to each step, so that the controls see the
    same hours of Windows' own schedule.

  Either one means another run of about five hours. The step 7 area check has
  also not run again since its fix.
- **NVMe and Hyper-V SCSI names** from the Linux side.
- **The release's own MSVC build.** The walk cross-builds
  `x86_64-pc-windows-gnu`. That the MSVC build behaves the same is not shown.

## Limitations and costs

- The host's C: had 94 GB free before the first start and 78 GB after the fifth.
  The gates below run `fstrim` twice.
- A Windows guest beside a release build starved WSL's relay for an hour and
  lost the first install (`docs/quirks.md`). The runs since then built nothing
  beside a guest.
- The logs of every start are kept under `/root/t10/logs/overnight-*-try` and
  `/root/t10/logs/road`.
