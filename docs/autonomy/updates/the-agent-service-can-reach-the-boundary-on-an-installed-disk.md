# The agent service can reach the boundary on an installed disk

**Date:** 2026-09-16
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 14,
*The disk installed under Secure Boot runs the agent service*)
**Contributor:** Claude Code worker in `C:\dev\alo-os` on the third PC (`AGAI01`), for
the owner
**Status:** ready for integration, for task 14 as split. **Why `alo-agentd` failed
on the disk installed under Secure Boot is located from the machine's own account,
recorded, and fixed in the image, with its own tests.** The named virtual-machine
test still does not pass, and no worker can make it pass: it installs the pinned
release `0.0.1`, which does not carry the fix, and only the owner builds, signs and
pins a release (ADR 0036). That run is task 15, written into the plan in this change
and marked blocked on the owner's release.

Follows `updates/the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`
(task 13), which is not edited here.

## What the installed machine said

Task 13's run ended with `alo-agentd.service` *failed* and no reason. Rather than
spend another hour on the emulated install to read the new `ALO-WHY` lines, I wrote
the same release to a disk natively and booted that disk under the same firmware:

1. `podman pull ghcr.io/aloworld-org/alo-os@sha256:d3f05b60…` (the pinned digest, 4
   minutes), then `bootc install to-disk --via-loopback --wipe --filesystem ext4`
   from that image in `podman --privileged`, which is the command
   `crates/alo-installing/src/writing.rs` runs, without `--source-imgref`. 7 minutes,
   *Installation complete!*
2. That disk booted under QEMU q35 with the Secure Boot firmware and variables the
   test takes from the base (`OVMF_CODE.secboot.fd`, Microsoft's certificates),
   emulated, with the test's two credentials: the watching unit and the drop-in that
   starts `user@1000.service`. The first boot printed nothing past the shim's
   *PageFaultExitBoot* lines for 29 minutes, because the installed machine's console
   is not the serial line. I stopped it and added `console=ttyS0,115200` to **this
   scratch copy's** boot entry, and the next boot reached a login prompt in 2.5
   minutes and the watching unit shortly after.

The machine's own account (the watching unit's output, `ALO-WHY` section, trimmed):

```
Id=alo-agentd.service
ActiveState=failed
SubState=failed
× alo-agentd.service - alo OS agent service: the door an agent knocks on
     Active: failed (Result: exit-code) since Thu 2026-09-17 00:55:27 UTC; 32s ago
   Duration: 274ms
    Process: 1124 ExecStart=/usr/bin/alo-agentd (code=exited, status=1/FAILURE)
● user@1000.service - User Manager for UID 1000
     Active: active (running) since Thu 2026-09-17 00:53:45 UTC; 2min 15s ago
[  253.815658] fedora alo-boundaryd[809]: alo-boundaryd: the boundary is on this kernel, pinned at /sys/fs/bpf/alo, with 60989 to write it and nobody else; this process holds nothing and is done
[  254.848280] fedora alo-agentd[1124]: alo-agentd: no translations were loaded: /usr/share/alo/translations could not be read — No such file or directory (os error 2). This machine speaks English until it is fixed
[  254.917004] fedora alo-agentd[1124]: alo-agentd did not run: a turn's work cannot be bounded on this machine: there is no boundary at /sys/fs/bpf/alo/bounds: alo-boundaryd loads one at boot, and until it has, this machine cannot bound a turn — docs/quirks.md, *A machine without a boundary runs no turn*, says what to check; alo-agentd will not run without a boundary, because a turn that cannot be bounded does not run (ADR 0015)
/sys/fs/bpf:
drwx-----T.  3 root root      0 Sep 17 00:53 .
drwxr-x---.  2 root alo-agent 0 Sep 17 00:55 alo
/var/lib/alo:
drwx------.  2 alo  alo  4096 Sep 17 00:55 .
-rw-r--r--.  1 alo  alo    13 Sep 17 00:55 record.jsonl
lockdown,capability,yama,selinux,bpf,landlock,ipe,ima,evm
none [integrity] confidentiality
Enforcing
uid=1000(alo) gid=1000(alo) groups=1000(alo),60989(alo-agent)
```

## What was found

**The boundary was there. The person's service could not pass the door in front of
it.** systemd 257 mounts the BPF filesystem with `mode=0700`, so `/sys/fs/bpf` is
`1700 root:root`. The loader made `/sys/fs/bpf/alo` `0750 root:alo-agent` exactly as
ADR 0018 asks, but `alo-agentd` runs as uid 1000 and could not pass through the
directory above it. `alo_bounding::Boundary::opened` checks for the pin with
`Path::exists()`, which answers `false` for a path it may not look at, so the
refusal said *there is no boundary*.

Why no earlier test saw it: in WSL, where every boundary test in the repository runs,
`/sys/fs/bpf` is `drwxrwxrwt root:root` (the mount's source is `bpffs`, from WSL's
init, not systemd's `bpf`).

It is not Secure Boot's lockdown (`integrity`) and not SELinux (`Enforcing`). The
second boot shows that.

**Shown, not inferred.** The same scratch disk, booted again under the same Secure
Boot firmware with only `z /sys/fs/bpf 0710 root alo-agent -` handed to systemd as a
`tmpfiles.extra` credential:

```
Id=alo-agentd.service
ActiveState=active
SubState=running
Result=success
● alo-agentd.service - alo OS agent service: the door an agent knocks on
     Active: active (running) since Thu 2026-09-17 01:04:49 UTC; 1min 2s ago
   Main PID: 1068 (alo-agentd)
     CGroup: /system.slice/alo-agentd.service
             └─1068 /usr/bin/alo-agentd
/sys/fs/bpf:
drwx--x---.  3 root alo-agent 0 Sep 17 01:02 .
drwxr-x---.  2 root alo-agent 0 Sep 17 01:04 alo
/sys/fs/bpf/alo:
-rw-rw----. 1 root alo-agent 0 Sep 17 01:04 bounds
-rw-------. 1 root alo-agent 0 Sep 17 01:04 fields
-rw-------. 1 root alo-agent 0 Sep 17 01:02 file_open
… (twenty-three hook links in all, each -rw------- root:alo-agent)
/run/alo/1000:
srw-rw----. 1 alo  alo-agent  0 Sep 17 01:04 agentd.sock
/var/lib/alo:
-rw-r--r--.  1 alo  alo    32 Sep 17 01:04 machine-id
none [integrity] confidentiality
Enforcing
```

So on the release as published, under Secure Boot, with SELinux enforcing, that one
line is the whole difference between `alo-agentd` failing and running with its door
open.

## What changed

- `image/usr/lib/tmpfiles.d/alo.conf`: `z /sys/fs/bpf 0710 root alo-agent -`, with
  the reason beside it. `z` adjusts the mount that is already there and makes
  nothing. Configuration only; systemd is the base's, unmodified (ADR 0011).
- `crates/alo-image/src/reaching.rs` (new): the way from the agent's service to the
  boundary. Two checks inside `everything_wrong_with`:
  - the loader waits for `/sys/fs/bpf` (`RequiresMountsFor=`);
  - the image adjusts that mount to exactly `0710`, owned by root, in **the group
    the loader's unit runs in**. So a loader moved to another group without this
    line moving with it is caught.
  - Tests: `the_image_lets_the_agents_group_reach_the_boundary` (the shipped image,
    and the whole image still clean); refusals
    `a_boundary_behind_roots_door_is_caught` (release 0.0.1's image),
    `a_directory_made_where_the_mount_belongs_is_caught` (`d` in place of `z`),
    `a_way_through_wider_than_passing_is_caught` (`0711`, `0750`, `0730`, `1777`),
    `a_way_through_given_to_somebody_else_is_caught` (the person's group, the
    agent's group as owner), `a_loader_moved_to_another_group_is_caught`,
    `a_loader_that_does_not_wait_for_its_file_system_is_caught`, and
    `the_sentences_name_the_mode_the_group_and_the_decision`.
- `crates/alo-image/src/making.rs`: reads a `z` line as an adjustment (`ADJUSTED`,
  `Made::is_adjusted_at`), never as a directory made, with
  `an_adjustment_and_a_directory_are_not_each_other`.
- `crates/alo-image/src/image.rs`: `Image::adjusted_at`.
  `crates/alo-image/src/service.rs`: `Service::waits_for_mounts`.
  `crates/alo-image/src/wrong.rs`: `TheLoaderDoesNotWaitForItsFileSystem`,
  `TheBoundaryIsOutOfTheAgentsReach`, `TheWayToTheBoundaryIsNotWhatWasDecided`.
  `crates/alo-image/src/checking.rs` calls the new module; `lib.rs` exports
  `THE_BPF_FILESYSTEM`, `THE_PASSAGES_MODE` and `ADJUSTED`.
- `docs/quirks.md`: *systemd mounts the BPF filesystem so only root can pass through
  it*, with both consoles.
- `docs/autonomy/v0-5-the-installer-plan.md`: task 14 marked done for its part, with
  the quicker road to an installed machine written down; task 15 written (blocked on
  the owner's release); task 11 now depends on 15.

**User-readable change:** on a machine installed from the next release, the agent
service can reach the security boundary loaded at startup and starts. Before, it
stopped straight after boot saying no boundary existed.

## Decisions

- **Group search only, not `0711`.** The agent's service needs to *pass through*
  `/sys/fs/bpf` to one name it already knows. `0710 root:alo-agent` gives exactly
  that to exactly the group ADR 0018 already gives the map. Nobody else gains
  anything, and the group can't list what else is pinned on the machine. Every pin
  keeps its own mode, which the loader sets.
- **In the image's `tmpfiles.d`, not in `alo-boundaryd`.** The loader could `chmod`
  the mount, but that would put a machine-wide permission decision inside a
  privileged binary, where `crates/alo-image` can't read it. A declarative line is
  visible to anyone reading the image and held by a test. It is applied by
  `systemd-tmpfiles-setup.service`, which `alo-agentd.service` already starts after.
- **Checked against the loader's `Group=`**, not a spelled-out `alo-agent`, so the
  two cannot drift apart.
- **Split rather than claimed.** The acceptance's test installs the pinned release,
  and no worker may make a new one (ADR 0036). Running that test for an hour against
  `0.0.1` would only reproduce the failure already recorded. The fix is shown instead
  on the same release under the same firmware, and the named run is task 15, marked
  `blocked` so the loop does not select it before the owner's release exists.
- **The credential was diagnosis only.** Task 14's constraint says nothing on the
  installed disk is changed by the test to make a service start. The test is
  unchanged, and task 15's constraint names the credential explicitly as not a way
  to pass it.
- **Not the sign-in stand-in.** The stand-in started `user@1000.service` correctly
  (`active (running)`, its bus up). Nothing about a real sign-in differs at the step
  that failed.

## Verification

All on the third PC (`AGAI01`), Windows Server 2022 with WSL 2 Ubuntu. No hardware
virtualisation (`qemu-system-x86_64 -accel kvm`: *failed to initialize kvm: No such
device*), so every machine was emulated (TCG). 893 GB free on the distribution's
filesystem before the runs.

**Gates, executed in WSL** (`CARGO_TARGET_DIR=/root/alo-builds/alo-os-88e6ebddb0cab76e`),
after the last code change:

```
cargo fmt --all -- --check                              FMT-OK
cargo clippy -p alo-image --all-targets -- -D warnings  exit 0
cargo test -p alo-image                                 exit 0
  src/lib.rs                               261 passed
    making::tests::an_adjustment_and_a_directory_are_not_each_other ... ok
    reaching::tests::a_boundary_behind_roots_door_is_caught ... ok
    reaching::tests::a_directory_made_where_the_mount_belongs_is_caught ... ok
    reaching::tests::a_loader_moved_to_another_group_is_caught ... ok
    reaching::tests::a_loader_that_does_not_wait_for_its_file_system_is_caught ... ok
    reaching::tests::the_sentences_name_the_mode_the_group_and_the_decision ... ok
    reaching::tests::the_image_lets_the_agents_group_reach_the_boundary ... ok
    reaching::tests::a_way_through_given_to_somebody_else_is_caught ... ok
    reaching::tests::a_way_through_wider_than_passing_is_caught ... ok
  tests/how_the_image_is_published.rs        6 passed
  tests/how_the_installer_is_released.rs     6 passed
  tests/what_the_image_owes_the_daemons.rs  25 passed
```

`cargo fmt --all` was also run from the Windows checkout. `alo-image` is the only
crate touched. The workspace suite was not run here; the supervisor runs it.

**The machine runs, executed:** the two boots above, on a disk written from the
pinned digest. Their whole serial lines are kept on this PC as
`C:\dev\setup\task14-installed-agentd-failed.log` and
`C:\dev\setup\task14-installed-agentd-running.log`.

**Not executed:**
- `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`:
  it installs release `0.0.1`, which would reproduce the recorded failure. It is
  task 15's acceptance against the next pinned release.
- The image was not rebuilt with the new line (`docker build` of the full recipe is
  over an hour and carries the weights). Its effect was shown by the credential on
  the published release. `bootc container lint` does not read `tmpfiles.d` types.
- The laptop: nothing here ran on it.

**After the runs:** the scratch disk (`/root/alo-builds/diag`, 24 GB sparse) and the
pulled release image were removed, and `podman image prune -f` ran. `podman images`
shows only the two pinned bases, no `qemu` is running, and 884 GB is free.

## Remaining limitations and found on the way

- **A machine installed from `0.0.1` still boots with `alo-agentd` failed.** The fix
  reaches machines only in a release the owner builds from a commit carrying it
  (task 15).
- **`alo-bounding` says *there is no boundary* when it cannot reach one.**
  `Boundary::opened` uses `Path::exists()`, which folds a permission refusal into
  absence, and that sent this diagnosis toward the loader first. `try_exists()`,
  with a refusal of its own naming the directory it could not pass, would have said
  it. `alo-bounding` is not this plan's crate, so this is proposed rather than made.
- **The image carries no `/usr/share/alo/translations`**, so every start logs *no
  translations were loaded*. No translation exists in the repository yet, so nothing
  is missing in practice. Whether the image should ship the empty directory is for
  the owner of `alo-saying`.

## Proposed updates for the integration owner

- **CHANGELOG.md:** *On a machine installed from the next release, the agent service
  starts: the image now lets it reach the security boundary loaded at boot, which
  systemd's default permissions had kept out of its reach.*
- **ROADMAP.md:** no box moves. Nothing ran on the laptop, and the named install
  test has not passed.
- **QUEUE.md / STATE.md:** installer plan task 14 done for its part, as split; task
  15 (*A release that carries the way to the boundary, installed under Secure Boot
  to the agent service*) blocked on the owner's release under ADR 0036; task 11 now
  depends on 15. Proposed to `alo-bounding`'s owner: tell *cannot reach the
  boundary* apart from *no boundary*.
