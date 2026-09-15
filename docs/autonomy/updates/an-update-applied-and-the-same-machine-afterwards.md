# An update applied, and the same machine afterwards

**Date:** 2026-09-15
**Workstream:** v0.5 the machine keeps itself — task 2 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` (`ROADMAP.md`: *atomic
updates with rollback*, *updates that never interrupt*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os`
**Status:** ready for integration. Tested in an emulated virtual machine; not on the certified machine. Calling it at boot and from the shell is the image and shell lanes' work (see limitations).

## What changed, for a person

When a person chooses to apply an update, alo OS now prepares it so it starts
the next time they restart. Their files, settings, grants, pairings, history and
search indexes stay exactly as they were. This was measured in a virtual machine,
file by file, not assumed. After the restart, the machine's history says that it
started on an updated version. At any moment the machine can say which version
of its system it is running. If an update can't be prepared, the person is told
that nothing changed and that the next restart starts the machine as it is now.

## What changed, in the repository

**`crates/alo-keeping-up`** (decisions; still no clock, thread, socket, file or
process, and still four dependencies — task 1's test holds it to that unchanged):

| File | What it holds |
|---|---|
| `src/deployments.rs` | `Deployments`: the booted, staged and rollback builds read from `bootc status --format json` (format version 1). Only `status.{booted,staged,rollback}.image.imageDigest` is read, and each goes through `Digest`'s check. `running()` refuses a booted deployment with no image (`NotRunningABuild`) |
| `src/source.rs` | `Source`: the repository updates come from, `host[:port]/path`. A tag or digest inside it is refused, and so are capitals, spaces, a leading dash (a flag), `..`, and empty segments. `Source::at(&Digest)` joins on the build |
| `src/staging.rs` | `Staging::of(ready, deployments_now, source, when)`: the one instruction, `switch --enforce-container-sigpolicy <source>@<digest>`, plus `--apply` only for `NowBecauseThePersonAsked`. `NotStaged`: `NotRunningABuild`, `TheMachineMovedOn`, `AlreadyWaiting` |
| `src/since.rs` | `Since::between(last_known, deployments)`: `FirstKnown`, `Unchanged`, or `Updated { from, to }` |
| `src/words.rs` | Six new sentences, each with a note: waiting for the restart, changed since it was found, already waiting, running not known, not prepared, not written down |
| `src/standing.rs`, `src/testing.rs`, `src/lib.rs` | A `cfg(test)`-only `Ready` for unit tests; fixtures; exports and crate docs |

**`crates/alo-updating`** (new; the doing):

| File | What it holds |
|---|---|
| `src/the_base.rs` | `Base` trait; `TheBase` runs `/usr/bin/bootc` with argument vectors, no shell, stdin null, and keeps the last 2 KiB of stderr |
| `src/status.rs` | `deployments` / `running`: the base is asked every time and nothing is cached |
| `src/applying.rs` | `apply(base, ready, source, when)`: read status now, decide `Staging`, run it |
| `src/last_known.rs` | The build last seen, one digest per line: written to `.new`, synced, renamed; read strictly |
| `src/restarted.rs` | `after_a_restart(base, last_known_at, record, at)`: entry first, last-known second |
| `src/refusing.rs` | `NotAnswered`, `NotRead`, `NotApplied`, `NotRecorded`, each `said()` in the vocabulary |
| `tests/applying_an_update.rs` | Legitimate and refusal paths against a stand-in base that records every question |
| `tests/an_update_keeps_the_persons_things.rs` | The virtual-machine test, plus a guard that its base is `image/Containerfile`'s |

**Other crates' surfaces, additive:**

- `crates/alo-record`: `Happened::Updated { from, to }` and `Entry::updated`,
  with no agent, not egress, and no errand. It is added to every accessor's
  nobody arm, with a unit test for the JSON shape `"updated":{"from":…,"to":…}`.
- `crates/alo-recounting`: `Outcome::Updated` and the word
  `recounting.outcome.updated`, in `EVERY_WORD` and `EVERY_OUTCOME`.
- `docs/contracts/record-file.md`: the `updated` tag is documented. Entries
  without an agent are now five, not four.
- `Cargo.toml` workspace member, `Cargo.lock`.
- `docs/quirks.md`: *`bootc install` reads the signature policy of the image it
  is installing*. The first run hit this; it matters to the installer lane.

## Decisions

- **The doing is a second crate, not a relaxation of the first.** Task 1's
  test says `alo-keeping-up` names no process, file or clock. Loosening it would
  narrow a promise that was just published. So `alo-updating` runs the program,
  and every argument it passes comes from `Staging`.
- **`switch` to a digest, never `upgrade` to a tag.** The build staged is the
  one the person was told about. A tag can move between the offer and the fetch.
- **`--enforce-container-sigpolicy` on every instruction.** ADR 0036 says an
  unsigned build is never staged. With this flag, bootc refuses to stage under a
  policy that accepts anything by default, and no choice or offer can remove it.
  **Finding for the installer lane:** the shipped image must carry a
  `/etc/containers/policy.json` that requires the owner's key for
  `ghcr.io/aloworld-org/alo-os`. Until it does, `apply` on the real image is
  refused by the base (the person is told nothing changed). That is the safe way
  to fail, and this lane may not edit `image/`.
- **The record entry is written at the first start on the new build, not at
  staging.** An update nobody restarted into has not happened. So the machine
  keeps one fact across restarts, `/var/lib/alo/last-known-build`, and compares.
  The entry is written **before** that fact advances. A crash in between
  duplicates the entry rather than losing it, and a test forces exactly that.
- **A last-known build that can't be read is refused, not treated as unknown.**
  Treating it as unknown would silently skip the entry for a real update.
- **Editing `alo-record` although the plan says it only reads it.** The
  acceptance asks for an entry "from which digest to which, as an errand with no
  agent", and no existing kind holds two builds. `LeftOnItsOwn` is a departure
  shown on the indicator, and starting on a build is not one. The record contract
  already allows a new `happened` kind as additive (`format` stays `1`), so I
  added one. `alo-recounting` matches exhaustively and needed its clause. Both
  edits are minimal and are named here for the owners of those crates.
- **"Errand" is read as *the machine's own act with no agent*, not as an
  `alo_egress::Errand`.** `Happened::errand()` answers `None` for `updated`,
  because nothing left the machine when it started. Fetching the build is the
  base's pull. Putting that pull on the indicator as `CheckingForAnUpdate` is
  the caller's job, and is still open (see limitations).
- **`Base` is a trait** so refusal paths can be shown to have asked the base
  nothing. Only `TheBase` exists outside tests, and a stand-in changes who
  answers, never what is asked.
- **A different build already waiting is replaced** by the one chosen. The same
  build waiting is refused, because one approval is one execution.
- **The test's images are the shipped base plus the test**, not the full alo OS
  image. That image carries 4.5 GB of weights, and the claim under test is the
  base's behaviour on alo OS's paths under alo OS's instructions. The guard test
  fails if `image/Containerfile` moves to another base.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| Applying stages the new digest through `bootc`, touching no path | `alo-keeping-up` lib `staging::tests::every_instruction_enforces_the_signature_policy_and_names_no_path`; `alo-updating` `applying_an_update::an_update_the_person_chose_is_staged_by_digest_and_nothing_else_is_asked` |
| In a VM: known files written, update applied, reboot, each named thing byte for byte | `alo-updating` `an_update_keeps_the_persons_things::an_update_applied_in_a_virtual_machine_keeps_every_named_thing_byte_for_byte` |
| The record gains an entry, from which digest to which, no agent | `alo-updating` `applying_an_update::the_record_says_the_machine_updated_from_which_build_to_which_with_no_agent`; `alo-record` lib `entry::tests::an_update_is_recorded_from_one_build_to_another_with_nobody_behind_it` |
| The running digest is readable at any moment | `alo-updating` `applying_an_update::the_running_build_is_the_bases_answer_at_the_moment_it_is_asked` (and in the VM, before and after) |
| Applied only when the person chooses; refusals change nothing | `applying_an_update::a_machine_that_moved_on_is_refused_and_the_base_is_never_told_to_stage`, `…an_update_already_waiting_is_not_staged_a_second_time`, `…a_base_that_will_not_stage_the_build_is_said_as_nothing_changed`, `…a_machine_whose_running_build_cannot_be_read_is_not_updated` |
| The record is never left with a hole | `applying_an_update::an_update_whose_build_could_not_be_kept_is_written_again_rather_than_lost`, `…a_last_known_build_that_cannot_be_read_is_refused_and_not_written_over`, `…a_base_that_cannot_be_asked_writes_nothing_down` |

## Verification

Run on 2026-09-15 in WSL Ubuntu 24.04 on the Windows Server gate machine, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
| `cargo test -p alo-keeping-up` | 28 unit + 10 integration passed |
| `cargo test -p alo-updating` | 6 unit + 10 (`applying_an_update`) + 1 (`an_update_keeps_the_persons_things`, 2 ignored) passed |
| `cargo test -p alo-record` | 73 unit + 2 integration + 2 doctests passed |
| `cargo test -p alo-recounting` | 55 unit + 16 integration + 6 doctests passed |
| `cargo test -p alo-saying` / `-p alo-collected` | 63 + 4 + 1 / 8 + 11 passed |
| `cargo test -p alo-shell --lib record_` | 30 passed (they read `EVERY_OUTCOME`) |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-updating -p alo-keeping-up -p alo-record -p alo-recounting --no-deps` | exit 0 |
| `cargo test -p alo-updating --test an_update_keeps_the_persons_things -- --exact --ignored an_update_applied_in_a_virtual_machine_keeps_every_named_thing_byte_for_byte` | **passed, 948.79 s** |

**The virtual machine run.** QEMU 8.2.2, `-accel tcg` (no KVM on this host),
q35, OVMF, 3 CPUs, 3 GB. The run used bootc 1.15.1 from the pinned base. The
first attempt was refused before anything was written; that is the quirk above,
and the fix was the test image's policy. On the second attempt the machine's
console said, in order:

- at 201 s, running `sha256:020b162fa4aa7a7dbca92c858de254f5db1feffc3a50eb1131b814efcbc55076`;
- the base staged the second build (`ostree-finalize-staged` at 403 s); applying
  it again was refused as already waiting;
- at 563 s, `before the update: passed`, and at 612 s the person's restart;
- on the new build, byte for byte as written: the settings, the appearance
  settings, the grants, the pairings, the file index, the list of indexed
  folders, a letter, a 3 MiB photograph, and the record;
- `the record says the machine updated from sha256:020b162f… to
  sha256:13fab9d8aa6c89ac615ae2430794528648a0f2d034307d9a58df622fa76e34e1`, and a
  second start on the same build added no entry;
- `after the restart: passed`, then the machine powered off.

Not run: the full workspace suite, which the supervisor runs. Nothing was
measured on the certified laptop.

## Remaining limitations

- **Nothing calls these at boot or from the shell yet.** `after_a_restart` needs
  a oneshot unit in `image/`, which belongs to the installer lane. `apply` needs
  the shell's update surface, which belongs to the shell plan. Proposed unit:
  `alo-updated.service`, `Type=oneshot`, `After=local-fs.target`, running a
  binary that calls `after_a_restart` with `THE_LAST_KNOWN_BUILD` and the record
  path from `/etc/alo/agentd.toml`.
- **The signature policy file** for the shipped image (see Decisions).
- **The base's pull of the build is not yet on the indicator.** The caller of
  `apply` should hold an `Underway` for `Errand::CheckingForAnUpdate` (or a new
  fetching-an-update errand; that choice belongs to `alo-egress`'s owner) around
  the call.
- **A rollback currently reads as `updated`** at the next start, because it is
  also a different build booting. Task 3 decides how a rollback is recorded and
  must tell the two apart.
- Measured in an emulated VM, not on the certified laptop.

## Proposed shared-document updates

- **CHANGELOG.md:** "Applying an update now keeps everything a person has:
  their files, settings, grants, pairings, history and search indexes were
  checked byte for byte across an update and restart in a virtual machine. The
  machine's history records when it starts on an updated version, and it can
  always say which version it is running."
- **ROADMAP.md:** v0.5 *atomic updates with rollback*: applying through bootc
  and the record entry are in (`crates/alo-updating`). Boot-time wiring,
  signature policy in the image, and rollback (task 3) remain.
- **QUEUE.md / STATE.md:** machine-keeps-itself plan task 2 done; task 3 next.
