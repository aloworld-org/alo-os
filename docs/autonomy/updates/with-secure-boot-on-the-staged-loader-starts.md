# With Secure Boot on, the staged loader starts

**Date:** 2026-09-16
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 9)
**Contributor:** Claude Code worker in `C:\dev\alo-os-shell`, for the owner
**Status:** **not finished, and no handoff was written.** The page fault is
found, explained and gone, and a second fault found on the way is fixed; the
task's acceptance — the install test passing with Secure Boot on — **does not
pass**, and where it now stops is below. Everything here gates.

## What was wrong

Task 2 left the environment unable to start under Secure Boot. With QEMU's q35,
OVMF's Secure Boot build and Microsoft's enrolled certificates, the firmware
page-faulted (`#PF`, `W:1 P:1`) starting the staged loader from the installer's
partition, before Linux, and hung.

The plan's steps were followed in order, each run booting the **same staged
partition** so that only one thing changed at a time.

### 1. The files are not the problem

Booted with Ubuntu's firmware **without** Secure Boot (`OVMF_CODE_4M.fd`, plain
`q35`), the same shim, the same loader and the same kernel start, systemd comes
up in the initramfs, and `alo-installing` says its sentences on the serial line:

```
BdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
[    0.000000] Linux version 6.19.14-101.fc42.x86_64 ...
alo OS is being installed on this computer. Each step is written here as it happens
```

So the files are startable. The difference is the firmware.

### 2. It is not Secure Boot, and not the machine's SMM

| what was started | firmware | machine | result |
|---|---|---|---|
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd` | `q35` | boots |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.fd` + `OVMF_VARS_4M.fd` | `q35,smm=on`, secure flash | boots |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35,smm=on`, secure flash | `#PF` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + `OVMF_VARS_4M.ms.fd` | `q35` | `#PF` |
| shim → loader → kernel | Ubuntu `OVMF_CODE_4M.secboot.fd` + **`OVMF_VARS_4M.fd`** (no keys, Secure Boot **off**) | `q35,smm=on`, secure flash | `#PF` |

Row 5 is the one that settles it: the Secure Boot *build* faults with no
certificates enrolled at all, so the fault is not signature verification. Rows 3
and 4 fault alike and rows 1 and 2 boot alike, so it is not the machine's System
Management Mode either. It is the firmware build.

### 3. It is the loader, not shim

| what was started | result |
|---|---|
| **shim alone**, `grubx64.efi` removed | *Failed to open \EFI\BOOT\grubx64.efi - Not Found*, **no fault** |
| **the loader alone**, `grubx64.efi` copied over `BOOTX64.EFI` | `#PF`, the same registers to the byte |

Shim starts and runs. The loader faults on its own, with or without shim in
front of it. Reading the registers against the loader's own PE headers
(`SectionAlignment 0x1000`, `.text` `+0x1000`, `.data` `+0x1b000`, `mods`
`+0x2c000`), the write runs from inside `.data` into `mods` — pages the stricter
firmware build has marked read-only.

### 4. The second firmware build names it

The plan's third step. Fedora's own firmware, `edk2-ovmf-20250812-21.fc42`,
starting **the same partition** on the same machine, says what is happening and
carries on:

```
PageFaultExitBoot: Page fault fixups needed (NX: 0, RW: 1).
PageFaultExitBoot: The guest OS boot chain is not NX clean.
PageFaultExitBoot: Applying global page table fixup (shim is older than v16).
[    0.000000] secureboot: Secure boot enabled
[    0.000000] Kernel is locked down from EFI Secure Boot mode
```

Both versions, named: **Ubuntu `ovmf` 2025.11-3ubuntu7** faults; **Fedora
`edk2-ovmf-20250812-21.fc42`** starts it with **Secure Boot enabled**. The boot
chain is the pinned base's own — `shim-x64-15.8-3.x86_64` and
`grub2-efi-x64-2.12-32.fc42.x86_64` — and the remedy the message names, shim 16,
is upstream's to ship, not ours to build (ADR 0011).

### 5. And a second fault, found on the way: the choice was never read

Every successful boot above ended with *No disk was chosen for alo OS, so
nothing was changed* — correct behaviour, on a choice that never arrived. The
loader's entry sourced `${cmdpath}/chosen.cfg`. Asked in a machine, the base's
loader answers:

```
ALODIAG cmdpath=[]
ALODIAG prefix=[(hd0,gpt5)/EFI/fedora]
ALODIAG config_directory=[(hd0,gpt5)/EFI/BOOT]
```

`cmdpath` is empty. Fedora's build has `/EFI/fedora` baked in as its prefix,
finds no configuration there, falls back to the directory it was loaded from,
and records that fall-back in `config_directory` alone. With `cmdpath`, **no
install could ever have succeeded**, under any firmware — the environment would
have refused every time for want of a disk. This was invisible until the loader
started at all.

## What changed

- `image/installing/grub.cfg` — the entry sources `${config_directory}/chosen.cfg`,
  with the measurement and the reason above it.
- `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`
  - the Secure Boot firmware is **taken out of the same pinned base the
    environment is built from** (`dnf install edk2-ovmf` in a container of it,
    once, cached in the work directory), rather than from whatever the host
    packages. `Firmware::code`/`variables` return paths; `what_is_missing` no
    longer asks the host for a Secure Boot firmware, and still fails naming
    everything else it needs.
  - new, ignored in the suite and run by name:
    `a_loader_the_firmware_does_not_trust_is_refused` — one byte of the staged
    loader changed, and the firmware answers *Access Denied -- rejected probably
    by Secure Boot*, starts no kernel, says none of the environment's sentences,
    and leaves the first disk byte-for-byte unchanged.
- `crates/alo-installing/tests/what_the_environment_carries.rs` —
  `wrong_with_the_entry` refuses an entry that does not read the staged choice
  from `${config_directory}`, and refuses one that mentions `${cmdpath}` at all;
  `an_entry_that_chooses_a_disk_itself_is_caught` gains that substitution as a
  refusal case.
- `crates/alo-installer/tests/the_installer_checks_consents_and_stages.rs` — the
  Windows program's staged path is held to the same variable, so the two halves
  of the choice cannot drift apart again.
- `docs/quirks.md` — two entries: *EDK II's strict image protection page-faults
  the base's signed loader, and Fedora's own firmware fixes it up* (hardware and
  firmware) and *GRUB — the base's signed loader leaves `cmdpath` empty, and sets
  `config_directory`* (pinned engines).
- `docs/booting.md` — what the virtual-machine test now uses and what it shows.

**User-readable change description (proposed for CHANGELOG):** *The small
environment a computer restarts into to install alo OS now starts with Secure
Boot switched on, and it reads the disk the person chose before the restart —
which a mistake in the loader's configuration meant it never had.*

## Decisions

- **The test's firmware comes out of the pinned base, not the host.** Two
  firmware builds disagree about the boot chain the base ships, and the one that
  matters for a test of that chain is the one built for it. Taking it from the
  base also means the firmware moves when the base moves, with no second pin to
  keep in step. Secure Boot stays enforced; it is never switched off (ADR 0033
  §4).
- **The enforcement is measured, not assumed.** Swapping a firmware to make a
  Secure Boot test pass is exactly the shape of quietly weakening a gate, so the
  new refusal test changes a byte of the staged loader and shows this firmware
  refusing it. A firmware that started anything would now fail that test.
- **No shim or loader of ours.** The fault is in an upstream loader and the
  remedy upstream names is shim 16. Building or patching either is forbidden
  (ADR 0011) and was not done. What the certified laptop's own firmware does is
  task 6's question, at the machine; if it turns out to be as strict as Ubuntu's
  build, the way through is a base whose shim is newer — a decision record and a
  pin, never a patch and never Secure Boot off.
- **`${config_directory}`, not a fallback chain.** An entry that tried
  `cmdpath` and then `config_directory` would have hidden the next such
  difference. One variable, measured, with a test that refuses the other.

## Verification

Platform: Windows 11 Pro 10.0.26200 checkout; everything below in WSL Ubuntu,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`, QEMU 10.2.1,
KVM.

### The acceptance, run by name — **failed**, and where

```
$ cargo test -p alo-installing --test installed_in_a_virtual_machine -- \
    --exact the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service \
    --include-ignored --test-threads 1
running 1 test
test the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service ... FAILED

thread '...' panicked at crates/alo-installing/tests/installed_in_a_virtual_machine.rs:699:13:
the console never said, after what came before it: alo OS is installed. This computer restarts in a few seconds

test result: FAILED. 0 passed; 1 failed; 0 ignored; 3 filtered out; finished in 908.01s
real	15m14.041s
```

What the machine's serial line said before it, in order — **this is the part
task 9 was about, and it works**:

```
[    0.000000] secureboot: Secure boot enabled
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Checking that virtio-alo-target is safe to install onto
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being
  replaced. This takes a while, and this screen will say when it is done
Still installing alo OS. Leave the computer on
  (four times)
alo OS could not be installed onto virtio-alo-target. That disk may now hold part of
  alo OS; nothing else on this computer was changed. Restart to try again
You can turn this computer off or restart it now
```

The firmware started the staged shim and loader **with Secure Boot enabled**, the
loader handed over the disk the person chose — which no run before this one did —
and the signature verified. `bootc install` then ended without installing. **Why
is not on the console**, because the environment says its own sentence and not the
installer's. Making it say them is the next worker's first step, and it is a gap
worth closing for its own sake: the person watching has no other window.

### The other runs, each on its own

- The elimination table in *What was wrong* — nine boots of the same staged
  partition, by hand in WSL with QEMU, each comparing one thing: recorded in
  `docs/quirks.md` with both firmware versions.
- The loader's variables, asked in a machine (`ALODIAG` above).
- A staged loader with one byte changed, under Fedora's Secure Boot firmware:
  *BdsDxe: failed to load Boot0002 … Access Denied -- rejected probably by Secure
  Boot*, *No bootable option or device was found* — no kernel, nothing of ours.
  This is what `a_loader_the_firmware_does_not_trust_is_refused` asserts; the
  test itself has **not** been run since it was written, because the host ran out
  of disk (below).

### The gates

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -p alo-installing -p alo-installer -- -D warnings`
  — `Finished`, no warnings.
- `cargo test -p alo-installing -p alo-installer` — all green:
  `alo-installer` lib 44, `the_installer_checks_consents_and_stages` 23;
  `alo-installing` lib 36, `the_boot_environment_installs` 14,
  `what_the_environment_carries` 5, `installed_in_a_virtual_machine` 1 passed
  and 3 ignored as intended in the suite.

  The first run of that gate **caught the new assertion being wrong**: the
  Windows crate's test refused any mention of `${cmdpath}` in the loader's
  entry, and the entry's own comment explains why `${cmdpath}` is not used. It
  now reads the entry without its comments, which is what
  `wrong_with_the_entry` already did.

Not run: the workspace suite (the supervisor's); any Hyper-V machine; any
physical machine; the Windows-only parts of `alo-installer`, which are the
plan's own paste and are untouched by this task beyond one test's assertion.

## What the next worker starts from

1. **Make the environment say why an install failed.** It says *alo OS could not
   be installed onto <disk>* and nothing else; the installer's own words go
   nowhere a person or a test can read them. The environment reports every step
   to the console because the person watching has no other window, and a failure
   is the step that most needs it.
2. **Then read what `bootc install to-disk` objected to.** The first worker's
   `docs/quirks.md` entry on what `--source-imgref` needs from the system it runs
   on is the place to start, and the machine has 3 GB of memory with the
   container store inside the initramfs.
3. Everything before that step is measured working under Secure Boot and needs no
   revisiting: the loader starts, the choice is read, the release verifies.

## Remaining limitations

- **Nothing here ran on the certified laptop, or on any physical machine.** Two
  firmware builds disagree about this boot chain; only the laptop can say which
  of them its own firmware resembles. That is task 6, at the machine.
- **Nothing here ran in Hyper-V.** The account the tests run under still cannot
  manage Hyper-V (task 2's report); the machine is QEMU with OVMF, named as such.
- **The host's disk is the binding constraint on this machine.** The virtual
  machine's disks are written inside WSL, whose virtual disk grows on Windows'
  `C:`, and `C:` filled to zero bytes during the first acceptance run — WSL
  itself began returning I/O errors while reporting 822 GB free inside. The
  supervisor's 12 GiB reserve measures the filesystem the build directory is on,
  which is the one *inside* WSL, and so cannot see this. `C:` has about 1 GB
  free after this task's cleanup; the WSL virtual disk is 153 GB on a 474 GB
  disk and cannot be compacted without shutting WSL down, which is a shared
  action needing an idle handoff with the other lane (`SHARED_MAIN.md`). This is
  for the owner, not something a worker may fix alone.

## Proposed updates for the integration owner

- CHANGELOG: the sentence above.
- STATE: *installer task 9 — the Secure Boot page fault is the base's signed
  loader writing to pages EDK II's stricter build marks read-only; the test now
  uses the firmware from the pinned base, which enforces Secure Boot and starts
  the chain, and a new test measures that enforcement. A second fault found on
  the way: the loader's entry read `${cmdpath}`, which the base's loader leaves
  empty, so the staged choice was never read.*
- QUEUE/ROADMAP: nothing beyond the plan's own status.
