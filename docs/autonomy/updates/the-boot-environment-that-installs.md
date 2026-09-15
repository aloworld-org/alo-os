# The boot environment that installs, tested in a virtual machine

**Date:** 2026-09-15
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`,
task 2 — *The boot environment that installs, tested in a virtual machine*)
**Contributor:** Claude Code workers in `C:\dev\alo-os-shell`, for the owner
(a first worker, stopped by the supervisor at 90 minutes; a second, who wrote
this report)
**Status:** **integrated as the written half, 2026-09-15.** No handoff was written,
and the report below is the second worker's, unchanged. After three workers reached
the supervisor's deadline, the task was split: what gates here — the environment,
the program and its refusals — is task 2 and is published; the two virtual-machine
failures under *What stops it* are tasks 8 and 9 of the installer plan, each with
this report's next experiments as its starting point.

## What changed

The installer's reboot half: a kernel and an initramfs made from the image's
own pinned base, which reads the disk a person chose, checks that the pinned
release is signed by the owner's key, writes it onto that one disk with `bootc
install to-disk`, and says every step on every console.

- `image/installing/` — the recipe (`Containerfile`), the initramfs list
  (`alo-installing.conf`), the one unit and target, and the loader entry
  (`grub.cfg`). The shim, loader and kernel are the base's signed files, copied.
  The signature checker is upstream cosign 3.1.3, pinned by sha256 and checked
  in its own stage.
- `crates/alo-installing` — the program inside the environment. Every decision
  is in `sequence.rs` and tested against a scripted machine; `running.rs` is
  the thin Linux side. Five enumerated programs (`program.rs`), no command
  strings. Refuses: no choice, two choices, a path or partition, a disk that
  never appears, disks that cannot be read, the installer's own disk (FAT label
  `ALO-INSTALL`), a disk holding Windows partition types, a disk in use or
  read-only, no network, not genuine (checker failed, or its answer names
  another digest), and a damaged environment. Every refusal before writing says
  *so nothing was changed*, never restarts, and ends with *you can turn this
  computer off or restart it now*. 23 words, collected by `alo-saying`.
- `crates/alo-image` — `installing.rs`/`installs.rs` hold the environment's
  recipe to the image's base and toolchain, to carrying the pin and the key the
  pin names, and to a checker that is one release checked by digest; three new
  `Wrong`s.
- `crates/alo-saying` — collects `alo-installing`.
- `docs/booting.md` — the environment, what the staging program (task 3) owes
  it, and how to watch it in a VM, now with an honest *neither passes yet*.
- `docs/quirks.md` — the first worker's two entries (what `bootc install
  to-disk --source-imgref` needs from the system it runs on; cosign 3's
  signature as an OCI referrer). **Not re-measured by the second worker**: the
  VM runs below never reached `bootc`, so the first entry is the first worker's
  account, from experiments this report cannot reproduce.

**User-readable change description (proposed for CHANGELOG, not yet):** *The
installer's second half — the small environment a computer restarts into to
install alo OS — is written and checked, but not yet shown working in a virtual
machine.*

## What the second worker changed in the first worker's draft

- `tests/installed_in_a_virtual_machine.rs`: staging mounted the FAT image at
  `/mnt/staged` inside a container of the bootc base, where `/mnt` is a link to
  a `var/mnt` that does not exist, so **both VM tests failed before starting a
  machine** — the test had never run to a VM. Now `/tmp/staged`.
- The same file: the first-disk check kept only a hash, so a change printed two
  hashes. It now keeps a copy and names every changed mebibyte and its
  partition (`the_first_disk_is_unchanged`).
- `src/lib.rs`: the rustdoc said the choice lives in `EFI/fedora/chosen.cfg`; the
  loader reads `${cmdpath}/chosen.cfg`, which is `EFI/BOOT/`, as `booting.md`
  and the test already said.
- `docs/booting.md`: removed the claim that the environment *pulled and
  installed the release* on 2026-09-15 — nothing this session ran shows it.
- The same file: its work directory moved from `/tmp` to Cargo's
  `CARGO_TARGET_TMPDIR` (see *Verification*, run 3).
- The plan: task 2 stays *ready*, with what was found, so the loop does not take
  it as done and the next worker starts from the two failures.

## What stops it

### 1. With Secure Boot on, the firmware page-faults starting the staged loader

`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`,
QEMU q35 (`smm=on`, secure pflash), Ubuntu's `OVMF_CODE_4M.secboot.fd` with
`OVMF_VARS_4M.ms.fd`, KVM. The serial line, in full after the screen clears:

```
BdsDxe: loading Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
BdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
!!!! X64 Exception Type - 0E(#PF - Page-Fault)  CPU Apic ID - 00000000 !!!!
ExceptionData - 0000000000000003  I:0 R:0 U:0 W:1 P:1 ...
RIP  - 0000000070AB1CA0 ... CR2 - 0000000070ACF000
!!!! Find image based on IP(0x70AB1CA0) (No PDB)  (ImageBase=0000000001059C58, ...) !!!!
```

A write (`W:1`) to a present page (`P:1`) from an image with no debug symbols —
the shape of the base's `shimx64.efi`/`grubx64.efi` writing to memory the
firmware has marked read-only. The machine then hangs until the test's hour
runs out. Not yet known: whether shim or grub faults; whether it is this OVMF
build's memory protection or the files; whether a laptop's firmware does the
same. **Next step:** boot the same partition with `OVMF_CODE_4M.fd` (no Secure
Boot) to split *the files* from *Secure Boot*; then shim alone with a trivial
second stage. Switching Secure Boot off is not a fix (ADR 0033 §4); the test
must keep it on.

### 2. The not-genuine refusal changes the first disk

`a_release_signed_by_another_key_writes_nothing_and_says_so` (kernel started
directly, firmware without Secure Boot). The console said, in order, *alo OS is
being installed…*, *Checking that virtio-alo-target is safe to install onto*,
*Checking over the internet that this download is a genuine alo OS*, *This
download is not a genuine alo OS, so nothing was changed*, *You can turn this
computer off or restart it now*; the second disk's allocated blocks stayed 0 —
and then **the first disk's hash differed**. That contradicts the sentence it
had just said, so it is the most important thing in this report.

Ruled out by two smaller boots of the same kernel and initramfs, each comparing
a copy of a Windows-shaped disk byte for byte (`cmp -l`), both **unchanged**:

| boot | path | first disk |
|---|---|---|
| random-filled partitions, no FAT, no choice | *No disk was chosen* | unchanged |
| the test's own staged FAT, choosing the installer's disk | lists the disks, *holds this installer* | unchanged |
| the same, choosing an empty disk, no network adapter | waits for a network | unchanged |

So booting, udev, `lsblk` and the staged FAT's presence do not write, in those
boots.

**Where it changed** (rerun with the new diagnostics, work directory on disk):

```
the first disk changed while the environment refused, at 3 mebibyte(s):
MiB 1653 (alo OS installer), MiB 1654 (alo OS installer), MiB 1871 (alo OS installer)
```

All three are inside the staged `ALO-INSTALL` FAT partition, which begins at
MiB 1653: its first two mebibytes (boot sector, reserved sectors and FATs) and
one cluster about 218 MiB in. Windows' four partitions and the partition table
are untouched. That is the shape of something opening the FAT for writing — a
dirty flag plus a directory or file cluster. The serial log shows no Linux
mount of it, and `systemd-gpt-auto-generator` reports no
`LoaderDevicePartUUID`, so it had nothing to mount an ESP from.

What differs between this boot and the unchanged diagnostic boots: this one
starts QEMU with `q35,smm=on` and `-global cfi.pflash01.secure=on` (the
diagnostic boots used plain `q35`), runs 4 CPUs/3 GB, brings the network up and
runs `cosign`, and boots an initramfs with an extra `newc` archive appended.
**Next experiments, in order:** (1) run the diagnostic "installer's own disk"
boot with the test's exact machine flags, to test the firmware's FAT driver
(EDK2 opens every FAT it finds while enumerating boot options) — if it is the
firmware, a real laptop does it too, the install test (which boots from that
partition) will show it as well, and the right assertion is *Windows' partitions
and the table are unchanged, and the installer's own partition changes only as
a firmware booting from it changes it*, written down in `docs/quirks.md`;
(2) if not, the not-genuine road with no extra archive (the owner's key and a
pin naming a digest the key did not sign).

## Decisions

- **QEMU/OVMF rather than Hyper-V.** Measured on this PC on 2026-09-15: `Get-VM`
  answers *You do not have the required permission*, and the account is not an
  administrator. OVMF with Microsoft's enrolled certificates is the same shape
  (UEFI, GPT, Secure Boot) and runs in the Linux the gates use. A Hyper-V run is
  a person's step; `docs/booting.md` gives its shape. This is a difference from
  the plan's wording, said here rather than hidden, and it does not mark the task
  done.
- **This machine can run KVM now.** Commit `a382fdc` measured no hardware
  virtualisation on a Server 2022 host; this PC (Windows 11 Pro) has `vmx` in the
  guest and `qemu -accel kvm` starts.
- **No handoff.** The acceptance is two VM tests and neither passes. A handoff is
  a statement that the work is finished.

## Verification

Platform: Windows 11 Pro checkout, gates in WSL Ubuntu,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`.

Executed, passing:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -p alo-installing -p alo-image -p alo-saying -- -D warnings`
  — `Finished`, no warnings (rerun for `alo-installing` after the test changes).
- `cargo test -p alo-installing -p alo-image` — alo-image 210 + 6 + 25 passed;
  alo-installing lib 36, `the_boot_environment_installs` 14,
  `what_the_environment_carries` 5 passed; `installed_in_a_virtual_machine`
  2 ignored (as intended in the suite).
- `cargo test -p alo-saying` — 63 + 4 + 1 passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-installing -p alo-image` — clean.

Executed, failing (the acceptance):

- `cargo test -p alo-installing --test installed_in_a_virtual_machine -- --include-ignored --test-threads 1`
  - run 1: both failed at staging (`mkdir: cannot create directory '/mnt': File exists`) — fixed;
  - run 2 (1716 s): the refusal failed on the first disk's hash; the install
    hung in the firmware (above) until it was stopped.
  - run 3 (refusal only): failed copying the first disk — `/tmp` in WSL is a
    3.9 GB tmpfs in memory, shared with other lanes, and was at 100%;
  - run 4 (refusal only, `TMPDIR` on disk, 335 s): the change located above.
    The test now uses Cargo's `CARGO_TARGET_TMPDIR`, which is on disk, so no
    `TMPDIR` is needed; clippy and the crate's tests pass after that change. It
    has not been run to a VM since.

Not run: the whole workspace suite (the supervisor's); any Hyper-V machine; any
physical machine.

## Proposed updates for the integration owner

- CHANGELOG, ROADMAP, QUEUE: nothing yet — the task is not done.
- STATE: *installer task 2 in progress; environment and program written and
  gating; VM acceptance blocked on an OVMF Secure Boot page fault and a
  first-disk write on the refusal path.*
