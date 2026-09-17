# Storage through the broker, and the decision updates wait on

**Date:** 2026-09-17
**Workstream:** v0.5 the broker and the disk: task 4 of
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, *Updates and storage,
through the broker* (`ROADMAP.md`: ★ *System verbs through the privileged broker*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os-2`
**Status:** ready for integration. **Storage is built** and tested in WSL.
**The updates are not built.** They wait on ADR 0053, proposed here, and on two
additive changes from other owners. The plan moves them, with their acceptance
unchanged, to a new task 8. Nothing has run on certified hardware, and the image
carries neither the broker nor udisks2's unit. Each is said below.

## What changed, for a person

alo OS can now mount a USB stick, a memory card or a disk in a USB enclosure, and
eject one, through the one part of the system allowed to change the whole machine.
A drive is always mounted for the person signed in, in the place their own session
finds their drives, and never for the assistant. Plugging a drive in gives the
assistant nothing. It can read what is on a drive only once the person grants a
folder on it, as with any other folder.

The machine is told which drive by what the drive itself reports: its maker, model
and serial number, and the filesystem's own identifier. It is never told by a
device name like `/dev/sdb1`, which names whichever drive was plugged in second.
A drive that is part of the machine is never mounted or ejected this way. Ejecting
never forces anything: if a file on the stick is still in use, the stick stays
switched on and the person is told it was not ejected. **Nothing here can format,
repartition or erase a drive.**

A drive's health — whether it reports that it is failing — can be read by the
person's own side without asking anybody, because the disk service answers it to
anyone. A drive that keeps no health report, as most USB sticks do, is "not known",
never "good".

**Updates and going back are not carried out yet.** This task found that the
system's update program refuses to run without a privilege the broker is built never
to hold. A decision record sets out the ways forward and recommends one. Until it is
accepted, asking the broker to apply an update or go back is refused and written
down, and nothing runs.

## What changed, in the repository

**`crates/alo-drives`** (new). What a drive is, as udisks2 reports it:

| File | What it holds |
|---|---|
| `src/reported.rs` | `Drive` (identifier, `Plugged::{Removable, BuiltIn}`, `Health::{Good, Failing, NotKnown}`, filesystems), `Filesystem` (UUID, mounted), `TheDrives`; `as_reported()` and `filesystem_as_reported()` are the bytes an identity is digested from, each after its own prefix; names with a control character, empty or too long are not reported |
| `src/login.rs` | `LoginName` (one shape), and `of_user`: the one account in `/etc/passwd` with a user number, or nobody |
| `src/service.rs` | `Drives` (what there is now, health included) and `DriveService` (mount, eject: two changes and no third); `NotAnswering`, `NotDone` in English for logs |
| `src/bus.rs` | udisks2's published names, and `METHODS`: the five methods the client may call, enforced in `call` |
| `src/udisks.rs` | `UDisks` over `unix:path=/run/dbus/system_bus_socket` (never an environment variable): one `GetManagedObjects` for a consistent picture; removable when the drive or medium is removable or attached over USB/SD **and** no block holds the system hint; health from the ATA self-assessment when supported, enabled and read, or the NVMe critical-warning bits when read; mount with the single option `as-user`; eject unmounts without options, then powers off or ejects; `OnThisMachine` connects afresh per request |

**`crates/alo-brokerd`**:

| File | What changed |
|---|---|
| `src/storage.rs` (new) | `Storage<D>`: `storage.mount` and `storage.eject` against what the disk service reports now; exactly one match; only a drive a person plugged in; never an already-mounted filesystem; mounted for the login `/etc/passwd` gives the door's user (`THE_ACCOUNTS`) |
| `src/carrying.rs` | `Carriers<S, D>` takes the storage carrier; `updates.apply` and `updates.roll-back` answer `not-carried` naming ADR 0053 |
| `src/lib.rs`, `src/main.rs`, `Cargo.toml`, `alo-brokerd.service` | the storage carrier wired to `alo_drives::udisks::OnThisMachine`; documentation (the unit's lines are unchanged, only its comments) |
| `tests/only_the_network_approved_is_changed.rs` | the new carrier signature, with a disk service that panics if asked to change anything |

**`crates/alo-broker`**: documentation only (`src/lib.rs`, `src/keeping.rs`,
`src/verbs.rs`), recording that health needs no broker verb and that the updates
wait on ADR 0053. `src/` is 2,208 lines, within its ceiling of 2,450. Its dependency
list is unchanged.

**Registrations:** `Cargo.toml` members and `Cargo.lock` (one crate). `alo-drives`
declares no words and no verbs, so neither `alo-saying` nor `alo-by-hand` changes.

**Documents:** `docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md`
(new, **proposed**). `docs/contracts/agent-verbs.md`: the storage identities and
verbs carried out, and why the update verbs are not. `docs/quirks.md`: `bootc`
needs `CAP_SYS_ADMIN`. The plan marks task 4 done, writes what was built and what
was not, adds task 8 for the updates, and makes task 7's walk depend on it.

## Decisions, and why

1. **The updates are a decision, not code.** Measured in the pinned base
   (`quay.io/fedora/fedora-bootc`, local image `b035260f985f`): `bootc 1.15.1`'s
   write commands require *uid 0 with CAP_SYS_ADMIN*. The broker runs with both
   capability lines empty, held by task 3's test. Separately,
   `alo_keeping_up::Staging::of` needs a `Ready`, which only a check shown on the
   indicator can make, so no process told thirty-two bytes can ever decide one.
   The only ways to carry an update out would be to widen the broker's privilege,
   to add a privileged component, or to edit `alo-keeping-up`, which this plan
   reads and never edits (and which the loop refuses). None of those is a worker's
   decision. So ADR 0053 compares four options — capability for the broker, a unit
   per verb, rpm-ostree's daemon, and nothing in v0.5 — and recommends a unit per
   verb that the broker starts through systemd. The update verbs answer
   `not-carried` until then, and the test that holds that fails once the ADR is
   accepted.
2. **Task 4 is marked done, and the updates move to a new task 8 with their
   acceptance word for word.** The prompt for this task asks for a finished handoff
   and the task marked done. Storage is complete, and the update half's finished
   work is the decision. Moving the unbuilt half into its own blocked task, rather
   than leaving task 4 open, keeps the loop from sending the next worker at storage
   that is already done, and narrows nothing: the acceptance is copied, not
   rewritten.
3. **A new crate, `alo-drives`**, although the plan names only `alo-broker` and
   `alo-encrypting`. This follows task 3's `alo-networks`: what a drive is must be
   shared by the privileged side (mount, eject) and the person's side (listing,
   health). It must not live in the door, which stays auditable. Flagged for the
   integration owner.
4. **A drive is named by udisks2's `Id` and a filesystem by that and its UUID**,
   each digested after a prefix so a drive and a filesystem can never share an
   identity. A filesystem's identity includes its drive's, so the same card in
   another reader, or a copied filesystem, is not what was approved. A device node
   is never an identity. A drive or filesystem with no such name is not reported
   at all, because nothing could approve it.
5. **Removable means a person plugs it in, and never the system.** Removable or
   attached over USB/SD, and no block device carrying udisks2's system hint. So a
   machine booted from a USB disk cannot have its own system ejected through the
   broker. The broker checks this and so does the client.
6. **Mounted *as* the person, by `as-user`, with no other option.** The broker has
   no name to go on, so the name is the one `/etc/passwd` gives the uid the machine
   description says the door is for, and only when exactly one account has it.
   `as-user` is udisks2's own road to *mount for this person where their session
   looks*. Mount options, a filesystem type and a mount point are never passed.
7. **Eject never forces.** Unmount has no options. A busy filesystem stops the
   eject before the drive is switched off, so nothing is powered off under an open
   file.
8. **Health is not a broker verb.** udisks2's D-Bus policy lets anybody read its
   properties, so health needs neither privilege nor approval. Making it a broker
   verb would have turned a read into a change (ADR 0001 §5). `Health` has three
   values and no score, and `NotKnown` is never read as good.
9. **The client names five methods and enforces them.** `bus::call` refuses any
   other method name. A test holds the list to five entries and reads the source
   for 27 udisks2 methods that write, lock, unlock or test a disk. The crate
   depends on `zbus` alone and listens for nothing, so plugging a drive in cannot
   start anything in it and nothing in it can grant.
10. **ADR number 0053.** First written as 0050, and renumbered twice: 0050 and 0051, then 0052, were published on `main` while this task was being gated. It is the next unused number on `main` and on every local
    branch. The held printers branch carries 0047, which is already taken on
    `main` and is that branch's to renumber.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The storage verbs take the drive by its stable identity, as the disk service reports it now | `alo-brokerd` `only_the_drive_approved_is_mounted_or_ejected` `a_removable_drive_mounts_for_the_signed_in_person_and_ejects`; refusals: `a_drive_not_reported_now_or_reported_twice_is_not_touched` |
| …against udisks2's own interface, rented | `alo-drives` `the_disk_service_on_a_bus` `each_change_is_made_against_exactly_what_the_disk_service_reported`, `the_disk_service_reports_each_drive_its_filesystems_and_its_health`; refusals: `nothing_is_changed_that_was_not_reported_or_is_part_of_the_machine`, `a_filesystem_that_will_not_unmount_leaves_the_drive_on` |
| Never format, repartition or erase anything; no verb writes to a partition table | `alo-drives` `a_drive_is_only_ever_mounted_or_ejected` `the_client_can_ask_for_five_things_and_none_writes_a_disk` |
| Only a drive a person plugged in | `alo-brokerd` `only_the_drive_approved_is_mounted_or_ejected` `a_drive_that_is_part_of_the_machine_is_neither_mounted_nor_ejected` |
| A removable drive mounts for the signed-in person only | `alo-brokerd` `only_the_drive_approved_is_mounted_or_ejected` `a_removable_drive_mounts_for_the_signed_in_person_and_ejects`; refusals: `a_person_the_accounts_do_not_name_has_nothing_mounted`; `alo-drives` lib `login::tests::an_account_that_is_not_exactly_one_login_is_nobody` |
| No grant is made to an agent by plugging a drive in | `alo-drives` `a_drive_is_only_ever_mounted_or_ejected` `plugging_a_drive_in_grants_nobody_anything`; `alo-brokerd` `only_the_drive_approved_is_mounted_or_ejected` `the_broker_can_make_no_grant` |
| One approval, one execution, under a genuine token | `alo-brokerd` `only_the_drive_approved_is_mounted_or_ejected` `a_drive_is_changed_only_under_a_genuine_approval_and_once` |
| Check a disk's health | `alo-drives` `the_disk_service_on_a_bus` `the_disk_service_reports_each_drive_its_filesystems_and_its_health` |
| The update verbs: **not met, decided instead** (ADR 0053); nothing runs and the refusal is recorded | `alo-brokerd` `the_updates_wait_on_their_decision` `neither_update_verb_is_carried_out_while_its_decision_is_proposed`, `nothing_in_the_brokers_process_runs_the_base` |
| The broker stays auditable | `alo-broker` `small_enough_to_audit_in_an_afternoon` `the_broker_is_no_longer_than_an_afternoon` |
| Still root holding no capability | `alo-brokerd` `the_unit_is_the_process` `the_broker_runs_as_root_in_the_persons_group_holding_no_capability` |

## Verification

Run in WSL Ubuntu on this machine, as root, with target
`$HOME/alo-builds/alo-os-2-72aa7fda7f7de151`:

- `cargo fmt --all` then `cargo fmt --all --check`: clean.
- `cargo clippy -p alo-drives -p alo-brokerd -p alo-broker --all-targets -- -D warnings`: passed.
- `cargo test -p alo-drives -p alo-brokerd -p alo-broker`: all passed (alo-drives
  6 unit, 2 source, 4 on a bus; alo-brokerd 2 unit and 6 + 5 + 6 + 5 + 3 + 2
  integration; alo-broker all).
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-drives -p alo-brokerd -p alo-broker`: passed.
- `cargo test -p alo-citing` (not edited; it checks the new ADR's citations and the
  contract's): passed. **An environment finding:** its test binary was first
  reused from a build of another checkout (`/mnt/c/dev/alo-os-fix`) that had
  shared this target directory. It carried that checkout's path and failed with
  *the repository is where this crate says it is*. Updating the timestamps of two
  of its files (content unchanged) rebuilt it from this checkout, and it passed.
  Cargo's fingerprint for a path package does not include the checkout's absolute
  path, so a shared target directory can run another checkout's `env!` paths.
- In the pinned base image, measured with `podman run`: `bootc --version`, `bootc
  upgrade|switch|rollback --help`, the host status schema (`downloadOnly`), the
  CAP_SYS_ADMIN strings in `/usr/bin/bootc`, and `rpm -q udisks2 NetworkManager
  rpm-ostree`.

**The first handoff was refused, and why.** The supervisor's
`cargo clippy --workspace --all-targets` stopped in
`alo-changing-network`'s `the_network_changes_only_through_the_broker`, which
also builds the broker's `Carriers` and still passed two carriers, not three.
The per-crate gates above never built that crate. The second worker gave the
test a disk service that reports no drives and fails the test if a network
change ever mounts or ejects one. The same stand-in is in `alo-brokerd`'s
network test. `alo-drives` became a dev-dependency of `alo-changing-network`.
No library code changed. Then, the same way:

- `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test -p alo-changing-network -p alo-brokerd -p alo-drives -p alo-broker`:
  all passed, including `the_network_changes_only_through_the_broker`'s 7 tests.

**Not run:** the whole-workspace suite, which the supervisor runs. **Nothing ran on
certified hardware.** The disk service in every test is a stand-in: in memory for
the broker's tests, and a `zbus` service with udisks2's name, objects and
interfaces on a private `dbus-daemon` for the client. It is not udisks2. **Not
measured:**

- whether udisks2 lets root holding no capability call `Mount` with `as-user`
  (`filesystem-mount-other-user`), `Unmount` and `PowerOff`. Polkit authorises
  uid 0, but the booted image is where that is confirmed;
- what `ConnectionBus`, `Removable` and `HintSystem` a real stick, card reader and
  USB enclosure report on the certified laptop;
- `bootc`'s capability refusal on a booted host (read from the program in a
  container, where it refuses first for not being booted).

## Limitations and follow-ups

- **The updates.** ADR 0053 is for the owner. If it is accepted as proposed, then
  before task 8 can be built: **proposed for the machine-keeps-itself lane**, a
  way in `alo-keeping-up` to decide a `Staging` from an approved `{from, to}`
  checked against the base's status now; **proposed for `alo-egress`'s owner**, an
  additive errand for fetching an update.
- **No turn offers the storage verbs, and no agent verb declares them.** An agent
  verb crate would have to register in `alo-software`'s terminal test, a crate
  this plan never edits. **Proposed:** task 7, or the turn lane, declares
  `mount_drive`, `eject_drive` and a `drive_health` read, and hands their approvals
  to the door.
- **The shell** shows drives, their health, and mount and eject in Settings and the
  status area through the same verbs (ADR 0009). **Proposed** for the shell lane.
- **The image.** **Proposed** for the image lane: udisks2 is in the base already.
  The lane should add a polkit rule so that the agent's own login cannot call
  udisks2's mount, unmount, power-off or eject itself, going around the broker, as
  task 3's report owes for NetworkManager.
- **Carried from tasks 1 and 3:** the kernel cannot tell `alo-agentd` from another
  program the person runs, and the broker is not yet in the image.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "alo OS can now mount and eject USB sticks, memory cards and
  external disks through the one part of the system allowed to change the whole
  machine. A drive is always mounted for the person signed in, never for the
  assistant, and plugging one in gives the assistant nothing. A drive is named by
  what it reports about itself, never by a device name. Nothing can format or
  erase a drive this way. A drive's health can be read without asking anybody.
  Applying an update and going back through the broker are not built yet: the
  update program needs a privilege the broker is built never to hold, and a
  decision record proposes how to proceed. Nothing has run on a real drive yet."
- **ROADMAP.md:** under ★ *System verbs through the privileged broker*, storage is
  done (the code); updates wait on ADR 0053 (proposed) and on changes from the
  machine-keeps-itself and egress owners.
- **QUEUE.md / STATE.md:** broker plan task 4 done (storage; ADR 0053 proposed).
  New task 8 is blocked on ADR 0053 and two additive changes. New items: declare
  and offer the storage verbs from a turn; Settings' drives pane; a polkit rule
  keeping the agent's login off udisks2; measure udisks2 `as-user` from a
  capability-less root and `bootc`'s capability set on a booted image. Environment
  note: target directories shared between checkouts can run stale `env!` paths.
