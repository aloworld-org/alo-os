# The Windows installer program: check, say, consent, stage, restart

**Date:** 2026-09-15
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 3 —
*The installer program: check, say, consent, stage, reboot*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-shell`, for the owner
**Status:** **ready for integration** — the program and every decision in it, with
the one part of the acceptance no machine here can run split to the plan's new
task 10 (below, *What is not done*).

## What changed

**User-readable change description (proposed for CHANGELOG):** *The installer a
person downloads is written. On Windows it checks the computer — how it starts,
Secure Boot, TPM, BitLocker, free space, memory, disks — and says what it found;
refuses a computer with Secure Boot on and says why, without suggesting anyone
change it; says exactly what it will do; asks the person to type the name of the
disk alo OS will replace; then makes Windows 1 GB smaller, puts the installing
environment in that space, adds an "alo OS" start-up entry for the next start
only, and restarts. If any step fails, it puts everything back. It has not yet
been run against a Windows it may change.*

- `crates/alo-installer` — new. One responsibility per file:
  - `sequence.rs` — the order: administrator, genuine download, checks, decision,
    what will happen, typed consent, staging, restart.
  - `program.rs` — **every program it runs**, as enumerated variants with typed
    arguments: eight reads and fourteen changes, through Windows PowerShell's
    storage cmdlets, `bcdedit`, `whoami` and `shutdown` at whole paths beneath
    `%SystemRoot%\System32`. Scripts are constants with numbers put into them and
    are handed over as `-EncodedCommand` (`encoded.rs`), so no character is
    re-read by a command line. No IOCTL, no `unsafe`.
  - `identities.rs` — the only values of the machine's that reach an argument: a
    disk number, a partition number, a drive letter, a braced GUID.
  - `administrator.rs`, `starting.rs`, `security_chip.rs`, `bitlocker.rs`,
    `memory.rs`, `windows_volume.rs`, `disks.rs`, `entries.rs` — each reads one of
    Windows' answers; an answer that is not the answer is *not known*, never a
    default.
  - `checking.rs` (runs the reads), `found.rs` (says each finding), `deciding.rs`
    (the offer or the first refusal), `consent.rs` (the typed disk name),
    `staging.rs` (the steps, the journal, putting back), `ended.rs` (endings and
    their sentences), `environment.rs` (the staged files held to the release),
    `naming.rs` (the disk's name after the restart), `sizes.rs`, `machine.rs` (the
    seam), `on_windows.rs` (the seam on Windows), `words.rs` (63 sentences, all
    with translator notes).
  - `tests/the_installer_checks_consents_and_stages.rs` — the whole program
    against a scripted Windows; every refusal asserts **no program that changes
    the computer ran**.
  - `tests/reading_this_windows.rs` — `#![cfg(windows)]`: the eight reads against
    the Windows the test runs on, through a machine that refuses every change.
- `crates/alo-saying` — collects `alo-installer`'s words (38 lists).
- `Cargo.toml` — the new member.
- `docs/booting.md` — *The program that stages it: `alo-installer`, on Windows*:
  what it does in order, what it refuses, the download layout and the release's
  list, how disk names are made, and what is and is not measured. The
  environment's own list is now headed *What the environment does, in order*.
- `docs/quirks.md` — two entries measured on this machine (below).
- `docs/autonomy/v0-5-the-installer-plan.md` — task 3 marked done and split; task
  10 written; tasks 4 and 6 now wait on 10 as well.

## Decisions, and why

1. **A console program, not a window.** The acceptance is about what is checked,
   said and consented to, and a console says all of it in the vocabulary. A
   graphical window is a second large dependency in the most dangerous program
   the product ships; it belongs with the Release (task 5) if the owner wants it,
   and the sequence does not change for it — `TheMachine::say`/`ask` are the only
   two methods a window would replace.
2. **Asks for an administrator's rights first, and does not elevate itself.** A
   person is never shown everything their computer is and then told nothing could
   be done. Self-elevation (`Start-Process -Verb RunAs`) would add a program that
   starts programs; the sentence names Windows' own *Run as administrator*.
3. **Genuine = the environment matches a list whose SHA-256 is compiled into the
   release.** ADR 0036 (as accepted) puts the public key inside what a person
   downloads and has the environment verify the image signature itself. What this
   program must add is that the environment it stages is the one this release
   carries. `ALO_INSTALLER_ENVIRONMENT_SHA256` is set by the release build (task
   5); each file is read once, compared, and **the same bytes** are written, so
   nothing changes between check and copy. A non-release build has no list and
   refuses as not genuine — which is true of it. Nothing shows or offers the check.
4. **The typed consent is the disk's name, and it is also the choice.** One
   question, so the choice and the consent cannot disagree. Case and spacing do
   not matter; anything else does. Two disks with the same maker's name are shown
   and typed with Windows' number after the name.
5. **Only an empty second disk of at least 24 GB is offered.** Task 2's
   environment replaces one whole disk and refuses a disk holding Windows. Offering
   the Windows disk would be restarting a person into a refusal. **So a one-disk
   laptop — the certified one, and this development machine — is refused today**
   with *no empty disk … beside the one Windows is on*; installing beside Windows
   on the same disk is task 4, and the plan now says so under task 3. Any
   partition at all makes a disk *in use*, stricter than the environment's list of
   Windows types, because a Linux or a camera card is as missable.
6. **The area is 1 GB, and Windows keeps 16 GB free.** The environment is a
   kernel, an initramfs and signed loaders — hundreds of MB; a download whose files
   exceed the area (less FAT's own) is refused. 16 GB free is more than a Windows
   feature update needs, so installing alo OS never leaves Windows unable to update.
7. **Staging order puts the one step that changes what starts last.** Shrink;
   make the area *at the offset the shrink freed*; check it is still there and
   format it; copy and read back; copy `{bootmgr}` into an entry named alo OS,
   point it at `\EFI\BOOT\BOOTX64.EFI`, list it last; take the letter away (and set
   `NoDefaultDriveLetter` so Windows does not re-letter it); set the firmware's
   one-time next start (`bcdedit /set {fwbootmgr} bootsequence`); restart. A run
   killed anywhere before the last step leaves a smaller Windows, an extra
   partition and an unused entry — all of which start Windows as before.
8. **Putting back is a journal, and it never over-claims.** Each change is
   journalled once it succeeded; a failure undoes the journal newest first. A shrink
   that *reports* failure is re-read rather than believed, and journalled if the size
   moved. Format and removal check the partition's offset first, so a renumbered
   partition is never touched. If putting back fails, the ending is
   `NotPutBack` and the person is told exactly what remains — never *nothing was
   changed*.
9. **BitLocker is read and said; only a volume mid-conversion is refused.**
   Shrinking an encrypted volume is Windows' own tool's job and it does it; the
   installer never reads the volume or touches its protectors. A state Windows did
   not report decides nothing (the shrink is the same tool that refuses a
   conversion in progress). No sentence names a recovery key — the plan forbids
   naming keys at all. Whether a firmware entry change asks a BitLocker machine for
   recovery on some firmware is **unmeasured** and is task 10's to see.
10. **Disk names after the restart are made only from identifiers.** NVMe:
    `nvme-eui.` + the identifier; SATA: `ata-<model>_<serial>`; SAS/SCSI (Hyper-V):
    `wwn-0x<NAA>`; everything else (USB above all) is not offered. A wrong name names
    no disk and the environment refuses it as not connected.
11. **Memory and TPM decide nothing.** Less than 16 GB is said with *slow with
    less*; the TPM is said. Neither is required by installing beside Windows.

## What this machine showed that the design had wrong

Running the read-only checks on this Windows 11 Pro 10.0.26200 laptop (unelevated)
found two things, both now in `docs/quirks.md` and in the code:

- **`Get-Tpm` unelevated returns a sentence, not an error**, so the first run said
  *this computer has no security chip (TPM)* about a TPM it was not allowed to ask
  about. The script now throws unless the answer has `TpmPresent`.
- **Windows names an NVMe disk by its bus and puts an identifier where the serial
  is**: `NVMe PVC10 SK hynix 512GB`, serial `FD5B_42CE_BC8F_9D54_ACE4_2E00_5113_F94B.`,
  unique id `eui.ACE42E005113F94B`. The first rule (`nvme-<model>_<serial>`) would
  have named no disk. NVMe is now named by identifier only; which of the two
  identifiers Linux's `wwid` is here stays unmeasured until the environment runs on
  an NVMe machine.

Also confirmed: `Get-PartitionSupportedSize`, `Get-BitLockerVolume` and
`Confirm-SecureBootUEFI` need an administrator and throw without one.

## Acceptance, criterion by criterion

| Criterion | Where it is shown |
|---|---|
| Checks UEFI, Secure Boot, TPM, BitLocker, free space, memory, from Windows' own tools, each with a sentence | `program.rs` reads; `the_installer_checks_consents_and_stages::a_computer_that_can_take_alo_os_is_checked_told_asked_staged_and_restarted` (every finding said before the question); `checking_the_computer_changes_nothing`; per-reader unit tests; **run against this Windows** (`reading_this_windows`, pasted below) |
| With Secure Boot on, refuses and says why, never suggesting the setting change | `with_secure_boot_on_it_refuses_says_why_and_suggests_nothing`, `secure_boot_that_could_not_be_read_is_refused`, `words::tests::the_secure_boot_refusal_never_suggests_changing_the_setting` |
| Says exactly what will happen, and takes a typed consent naming the disk | the whole-road test (the four *will* sentences before the question); `the_consent_is_the_disks_name_and_nothing_else`; `consent::tests::*` |
| Shrinks through Windows' tooling, stages the environment, adds the UEFI entry, restarts | whole-road test (exact order and arguments, files and `chosen.cfg` on the area); `the_next_start_is_the_last_change_before_the_restart`; `the_choice_is_what_the_environments_loader_reads`; `program::tests::*` |
| Every step before the restart is reversible | `a_failure_at_each_step_puts_back_everything_before_it` (all seven steps), `what_could_not_be_put_back_is_said_exactly`, `a_shrink_that_reported_failure_is_asked_about_not_believed`, `an_area_made_in_the_wrong_place_is_never_formatted`, `a_copy_that_does_not_read_back_is_put_back` |
| **…and tested so in a Hyper-V VM with a real Windows, killed at each step, Windows still boots** | **Not done — task 10.** See below. |
| `unsafe_code = "forbid"` holds; Windows' own programs behind safe wrappers | workspace lint unchanged; `std::process::Command` only |
| A person never sees a key; not genuine is said in the plan's words, no way past | `a_download_that_is_not_genuine_is_refused_in_the_plans_words`, `a_download_with_a_file_missing_is_incomplete`, `environment::tests::a_list_that_could_misplace_a_file_is_not_a_list`, `words::tests::nothing_here_names_the_machinery_a_key_or_a_signature` |
| Every refusal changes nothing and says so | each refusal test asserts no changing program ran; `words::tests::every_refusal_says_nothing_was_changed` |

## What is not done, and why it is a task of its own

The walk in a VM with a real Windows, killed at each step. **This machine cannot
run it:** the account cannot manage Hyper-V (task 2's report found the same), no
Windows installation media or unattended-install tooling is in the repository, and
the only Windows here is the development machine's own, which the plan forbids any
destructive test on. Writing hundreds of lines of a VM test nobody could run would
be a test in name only. Following task 2's precedent, task 3 is marked done for
what is proven and **task 10** now holds exactly that acceptance, with what it must
show that the scripted machine cannot: the cmdlets and `bcdedit` doing what
`program.rs` asks on a running Windows, the disk names matching `/dev/disk/by-id/`,
and a copied `{bootmgr}` entry being one the firmware starts. Tasks 4 and 6 now
wait on it.

## Verification

Platform: Windows 11 Pro 10.0.26200 (this checkout), rustc 1.97.1; and Linux under
WSL (Ubuntu, the supervisor's build directory for this checkout).

Executed, all passing:

- `cargo fmt --all` — clean (`--check` passes).
- `cargo clippy -p alo-installer -p alo-saying --all-targets --no-deps -- -D warnings`
  — clean on Windows and on Linux. (Without `--no-deps`, Windows clippy stops in
  `alo-remembering`, which this change does not touch: four `dead_code` warnings in
  `src/names.rs` at `HEAD`, whose users are Linux-only. Linux is unaffected.)
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-installer --no-deps` — clean.
- `cargo test -p alo-installer` — Windows and Linux (Linux: 44 + 23; the Windows-only
  read test is compiled out).
- `cargo test -p alo-saying` — 63 + 4 + 1, Windows and Linux.
- `cargo test -p alo-collected` — 8 + 11 (the new crate's words are collected).
- `cargo test -p alo-image` — 209 + 6 + 25 (it reads `docs/booting.md`).
- Each of the 36 evidence tests run alone with `--exact`: one test, passing.

`cargo test -p alo-installer -- --nocapture` on this Windows PC:

```
     Running unittests src\lib.rs
running 44 tests
test bitlocker::tests::an_unnamed_state_is_not_known ... ok
test disks::tests::no_answer_is_no_disks ... ok
test consent::tests::the_name_typed_is_the_disk_chosen ... ok
test encoded::tests::base64_is_the_rfcs ... ok
test disks::tests::an_area_an_earlier_start_left_is_found ... ok
test disks::tests::a_read_only_or_partitioned_disk_is_refused ... ok
test disks::tests::only_the_empty_nameable_disk_is_for_alo_os ... ok
test entries::tests::a_name_that_only_contains_it_is_not_it ... ok
test administrator::tests::elevated_is_the_high_levels_identifier ... ok
test consent::tests::anything_but_an_offered_disks_name_is_refused ... ok
test consent::tests::nothing_typed_is_a_person_who_stopped ... ok
test ended::tests::every_ending_is_said_whole ... ok
test bitlocker::tests::each_state_is_read ... ok
test disks::tests::two_disks_with_one_name_are_told_apart ... ok
test entries::tests::an_entry_an_earlier_start_left_is_found ... ok
test environment::tests::no_released_list_is_nothing_genuine ... ok
test encoded::tests::a_script_is_utf16_little_endian ... ok
test found::tests::memory_is_said_as_it_was_sold ... ok
test environment::tests::a_complete_list_is_read ... ok
test environment::tests::the_choice_is_one_line_naming_the_disk ... ok
test identities::tests::an_entry_is_exactly_a_braced_guid ... ok
test identities::tests::a_letter_is_one_letter ... ok
test naming::tests::a_disk_whose_name_cannot_be_made_has_none ... ok
test program::tests::formatting_or_removing_checks_the_area_is_still_where_it_was_made ... ok
test environment::tests::a_list_that_could_misplace_a_file_is_not_a_list ... ok
test program::tests::the_tools_are_windows_own_at_whole_paths ... ok
test program::tests::reads_and_changes_are_told_apart ... ok
test memory::tests::bytes_are_read_and_nothing_else_is ... ok
test naming::tests::each_known_bus_is_named_as_udev_names_it ... ok
test security_chip::tests::each_answer_is_read_and_no_answer_is_not_read ... ok
test sizes::tests::had_rounds_down_and_needed_rounds_up ... ok
test starting::tests::an_unanswered_question_is_not_known_never_off ... ok
test starting::tests::uefi_and_secure_boot_are_read ... ok
test program::tests::every_program_is_whole_arguments ... ok
test windows_volume::tests::an_answer_that_does_not_add_up_is_no_answer ... ok
test windows_volume::tests::the_shrink_gives_up_the_area_right_after_windows ... ok
test windows_volume::tests::too_little_space_is_refused ... ok
test words::tests::a_download_that_is_not_genuine_is_said_in_the_plans_words ... ok
test words::tests::every_key_is_one_of_this_crates_and_names_one_string ... ok
test words::tests::every_refusal_says_nothing_was_changed ... ok
test words::tests::every_word_carries_a_note ... ok
test words::tests::nothing_here_names_the_machinery_a_key_or_a_signature ... ok
test words::tests::the_secure_boot_refusal_never_suggests_changing_the_setting ... ok
test words::tests::the_list_declares_and_nothing_is_replaced ... ok
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src\main.rs
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\reading_this_windows.rs
running 1 test
administrator: false
This computer starts with UEFI, which alo OS needs
Whether Secure Boot is on could not be found out
Whether this computer has a security chip (TPM) could not be found out
This computer has 15 GB of memory. alo OS is made for 16 GB or more, and is slow with less
decided: Err(SecureBootNotRead)
test this_windows_answers_every_check_and_nothing_is_changed ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.52s
     Running tests\the_installer_checks_consents_and_stages.rs
running 23 tests
test checking_the_computer_changes_nothing ... ok
test a_copy_that_does_not_read_back_is_put_back ... ok
test a_computer_with_no_empty_second_disk_is_refused ... ok
test an_area_made_in_the_wrong_place_is_never_formatted ... ok
test a_restart_that_did_not_happen_is_said ... ok
test a_download_with_a_file_missing_is_incomplete ... ok
test a_bios_computer_or_an_unread_one_is_refused ... ok
test a_windows_disk_that_is_not_gpt_is_refused ... ok
test a_download_that_is_not_genuine_is_refused_in_the_plans_words ... ok
test bitlocker_part_way_through_is_refused ... ok
test a_computer_that_can_take_alo_os_is_checked_told_asked_staged_and_restarted ... ok
test the_choice_is_what_the_environments_loader_reads ... ok
test not_an_administrator_is_refused_before_anything_is_read ... ok
test the_next_start_is_the_last_change_before_the_restart ... ok
test a_shrink_that_reported_failure_is_asked_about_not_believed ... ok
test secure_boot_that_could_not_be_read_is_refused ... ok
test what_could_not_be_put_back_is_said_exactly ... ok
test with_secure_boot_on_it_refuses_says_why_and_suggests_nothing ... ok
test too_little_space_is_refused_with_the_numbers ... ok
test what_an_earlier_start_left_is_refused_and_not_removed ... ok
test what_could_not_be_read_is_refused ... ok
test the_consent_is_the_disks_name_and_nothing_else ... ok
test a_failure_at_each_step_puts_back_everything_before_it ... ok
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The real read ran **unelevated**: this shell cannot start an elevated process, so
Secure Boot, the TPM, BitLocker, the shrinkable size and the firmware list were
*not known* — the behaviour the test holds, and exactly what a person who did not
choose *Run as administrator* is refused before. The same read elevated is not yet
run; task 10 runs everything elevated inside a VM.

**Not executed:** the installer changing any Windows; any VM; the environment
finding a disk by a name this program made; the full workspace suite (the
supervisor runs it).

## Remaining limitations

- One-disk computers are refused until task 4.
- English only: no translation is carried in the download yet (task 5 ships them).
- Wi-Fi is not carried across the restart (unchanged from task 2's contract).
- The disk-name rules are unmeasured from the Linux side (task 10).
- A second run after a killed one refuses *already started* and does not clean up;
  removing alo OS, and what an interrupted start left, is task 4's *remove alo OS*.

## Proposed updates for the integration owner

- **CHANGELOG:** the change description above.
- **ROADMAP:** *Installer* — the Windows program is written and tested against a
  scripted Windows; not yet run against a Windows it may change.
- **QUEUE/STATE:** installer plan task 3 done (this report); task 10 added and
  ready; tasks 4 and 6 also wait on 10.
