# A refusal writes nothing the installer does not own

**Date:** 2026-09-15
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 8)
**Contributor:** Claude Code worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration.

## What was wrong

Task 2's refusal test, `a_release_signed_by_another_key_writes_nothing_and_says_so`,
said *this download is not a genuine alo OS, so nothing was changed* and then
found the first disk changed at three mebibytes inside the staged `ALO-INSTALL`
FAT. The report for task 2 named two experiments, in order. **The first one found
the writer**, so the second (the refusal without the appended initramfs archive)
was not needed.

**The writer was the test's virtual machine.** The refusal started
`OVMF_CODE_4M.fd`, the firmware build without Secure Boot *and without System
Management Mode*, on the machine made for the Secure Boot build: `q35,smm=on` with
`-global driver=cfi.pflash01,property=secure,value=on`, which lets only SMM code
write the variable flash. None of that firmware's variable writes took, so it
did what OVMF does when it has no variable store it can write. It saved its
variables as a file, `NvVars`, in the root of the first FAT it found: the
installer's partition. That accounts for the boot sector, the FATs and one data
cluster.

It was neither alo OS nor a laptop's firmware. A laptop has a variable store its
firmware can write. So neither of the plan's two branches fits exactly: the
writer is not ours, and the firmware-only assertion would not be true either. The
right result is the stricter one. The machine is fixed, and the test holds that
**the whole first disk** is byte-for-byte unchanged.

## Evidence

Measured in WSL Ubuntu, QEMU 10.2.1 (`1:10.2.1+ds-1ubuntu3.2`), OVMF
2025.11-3ubuntu7, KVM. The firmware ran alone for 45 seconds: no kernel, no
network, nothing of ours. The disk was 700 MiB, with a 100 MiB data partition and
a 590 MiB `ALO-INSTALL` FAT32 holding `EFI/BOOT/chosen.cfg`. Before and after
were compared with `cmp -l`, and the FAT was listed with `mdir`.

```
== plain: changed bytes: 0                    (OVMF_CODE_4M.fd, -machine q35)
Directory for ::/
EFI          <DIR>     2026-09-15  19:49

== exact: changed bytes: 967                  (OVMF_CODE_4M.fd, q35,smm=on, secure flash)
Directory for ::/
EFI          <DIR>     2026-09-15  19:49
NVVARS            1523 2026-09-15  19:50  NvVars
changed bytes by MiB:  10 at MiB 101, 957 at MiB 102

== secboot+smm: changed bytes: 0              (OVMF_CODE_4M.secboot.fd + VARS .ms, q35,smm=on, secure flash)

variable flash bytes changed:  plain 6176,  exact 0
```

In the second run the flash took nothing and the disk took the writes. This is
written into `docs/quirks.md`, *OVMF without SMM saves its variables onto a FAT
disk when its flash is SMM-only*.

## What changed

- `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`
  - `Firmware` (`SecureBoot`, `Plain`) holds the firmware's code, its
    variables and **the machine it is started in** as one value. Nothing can
    start a firmware on the other one's machine again. `a_machine` and
    `variables` take it.
  - The refusal now starts the plain firmware on a plain `q35`.
  - The refusal test also asserts that the firmware's own variable flash
    changed. If a machine ever again cannot write its flash, the test fails
    saying so, instead of reporting a write to a disk.
  - New in the suite (not ignored): `a_firmware_is_given_only_flash_it_can_write`.
  - The Secure Boot install test's machine is unchanged: it already paired the
    SMM build with SMM-only flash. That configuration was measured writing
    nothing to the disk (row 3 above).
- `docs/quirks.md`: the entry above.
- `docs/booting.md`: the refusal passes; the install still does not (task 9).
- `docs/autonomy/v0-5-the-installer-plan.md`: task 8 marked done. Tasks 9 and 10
  follow it.

**The sentence was not touched.** Nothing of ours wrote, so no code in the
environment changed. The machine the test runs was wrong, and that is what changed.

**User-readable change description (proposed for CHANGELOG):** *The installer's
test of its refusal no longer uses a virtual machine that writes to the disk by
itself. The refusal is now shown to leave every byte of the computer's disk as it
was.*

## Decisions

- **The stricter assertion, not the firmware exception.** The plan allowed
  "the installer's own partition changes only as a firmware booting from it
  changes it" if the writer was the firmware. The writer was a firmware, but only
  on a machine no laptop is. Taking that exception would have written a
  virtual-machine fault into the product's promise, so the test holds the
  whole disk.
- **Measuring the flash, not just pairing the arguments.** A test of the
  argument list only guards against this exact mistake. The flash assertion
  measures the property that matters, which is that the firmware had somewhere of
  its own to write.
- **The second experiment was not run.** The first explained every changed byte.
  A second would have tested a cause already excluded by the firmware-only run,
  which changed the disk with no kernel present at all.

## Verification

Platform: Windows 11 Pro checkout; gates in WSL Ubuntu,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`.

Executed, passing:

- The acceptance, run by name in a virtual machine (recipe built, first disk
  staged, refusal booted, disks compared):

  ```
  $ cargo test -p alo-installing --test installed_in_a_virtual_machine -- \
      --exact a_release_signed_by_another_key_writes_nothing_and_says_so \
      --include-ignored --test-threads 1
  running 1 test
  test a_release_signed_by_another_key_writes_nothing_and_says_so ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 208.00s
  real	3m36.115s
  ```

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -p alo-installing -- -D warnings`: `Finished`, no
  warnings.
- `cargo test -p alo-installing`: lib 36 passed; `installed_in_a_virtual_machine`
  1 passed (`a_firmware_is_given_only_flash_it_can_write`), 2 ignored as intended;
  `the_boot_environment_installs` 14 passed; `what_the_environment_carries` 5
  passed.
- The three firmware-only boots in *Evidence*, run by hand with a script in the
  worker's scratch directory and a throwaway `/root/alo-task8` work directory in
  WSL.

The Windows-only `alo-installer` crate is not touched, so the plan's paste of
`cargo test -p alo-installer` does not apply to this task.

Not run: the workspace suite (the supervisor's); the Secure Boot install test
(task 9's acceptance, which this change does not claim); any Hyper-V or physical
machine.

## Proposed updates for the integration owner

- CHANGELOG: the sentence above.
- STATE: *installer task 8 done — the refusal's first-disk write was the test VM's
  non-SMM OVMF saving `NvVars` under SMM-only flash; refusal VM test passes with
  the whole first disk unchanged.*
- QUEUE/ROADMAP: nothing beyond the plan's own status.
