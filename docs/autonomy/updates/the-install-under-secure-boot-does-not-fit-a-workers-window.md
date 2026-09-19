# The install under Secure Boot pulls and verifies release 0.0.2, and does not fit a worker's window

**Date:** 2026-09-18
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 15,
*A release that carries the way to the boundary, installed under Secure Boot to the
agent service*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-3` on the third PC (`AGAI01`),
for the owner
**Status:** **blocked — task 15 is not done, and no handoff was written.** The named
test did not finish inside this worker's ninety-minute window and was stopped
deliberately rather than left running unwatched. What the run did show, and the
measurement that says what a machine needs to hold it, are below and in the plan.
**Nothing here claims the acceptance.**

Follows `updates/the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`
(task 13) and `updates/the-agent-service-can-reach-the-boundary-on-an-installed-disk.md`
(task 14). Neither is edited here.

## What this task is

Task 14 found why `alo-agentd` failed on a disk installed under Secure Boot —
systemd mounts `/sys/fs/bpf` so that only root may pass through it to the boundary
pinned beneath — and fixed it in `image/usr/lib/tmpfiles.d/alo.conf`. It could not
show the fix *through the installer*, because the installer pulls what
`image/pinned.toml` pins, and only the owner builds, signs and pins a release
(ADR 0036). Release `0.0.2` is that release: digest
`sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9`, built
from revision `8d2619d`, verified against `signing/alo-os.pub`.

So this task holds no design. It is one run:
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`,
whole, with Secure Boot on, against the release pinned now. That is why there is no
code in this report — and why, the run not having finished, there is nothing to hand
over as done.

## The run

Started 2026-09-18 11:44 (PDT) on the third PC (`AGAI01`), Windows Server 2022 with
WSL 2 Ubuntu. **No hardware virtualisation** — this is a VMware guest and
`qemu -accel kvm` refuses here (`docs/quirks.md`, *WSL on a VMware guest shows
`/dev/kvm` and has no KVM behind it*) — so every machine was emulated (TCG) on the
four processors WSL has.

**Before the first virtual machine started: 31.7 GiB free on C:**, the drive the
distribution's disk lives on, above the plan's 15 GB floor. Said here because the
plan asks a worker to say it.

| | |
| --- | --- |
| The environment built from `image/installing/Containerfile` | 11:44 – 12:00 (16 min) |
| The Secure Boot firmware taken out of the pinned base | within that |
| First disk written Windows-shaped and staged, hashed, copied; second disk made | 12:00 – 12:01 |
| The install machine started, Secure Boot on | 12:01 |
| Stopped, deliberately, with the install still writing | 12:39 |

### What the machine said

```
BdsDxe: loading Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
BdsDxe: starting Boot0002 "UEFI Misc Device" from PciRoot(0x0)/Pci(0x3,0x0)
PageFaultExitBoot: Page fault fixups needed (NX: 0, RW: 1).
PageFaultExitBoot: The guest OS boot chain is not NX clean.
PageFaultExitBoot: Applying global page table fixup (shim is older than v16).
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: virtio-alo-target
Checking that virtio-alo-target is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. This takes a while, and this screen will say when it is done
Still installing alo OS. Leave the computer on
…
```

Every sentence the acceptance asks for before the install itself, in order, under
Secure Boot, against release `0.0.2`: the choice read, the disk found and checked,
the network reached, and **the owner's signature over the pinned digest verified
inside the machine** — *This is a genuine alo OS*. No refusal sentence appeared.
The firmware line the test asserts, `Secure boot enabled`, is on the serial line.

This is the first run to reach that point against `0.0.2`. It says the new pin,
the new digest and the committed key agree with each other and with the environment
built from this checkout — which is worth having, and is **not** the acceptance.

### Why it did not finish

In thirty-eight minutes the install wrote **4.7 GB** to the second disk — **about
124 MB a minute**, steady throughout:

| Time | Written to the second disk |
| --- | --- |
| 12:05 | 442 MB |
| 12:08 | 819 MB |
| 12:10 | 1.2 GB |
| 12:13 | 1.6 GB |
| 12:16 | 1.9 GB |
| 12:20 | 2.3 GB |
| 12:23 | 2.7 GB |
| 12:27 | 3.3 GB |
| 12:31 | 4.0 GB |
| 12:35 | 4.6 GB |
| 12:39 | 4.7 GB (stopped) |

The pinned image is **5.79 GB compressed over 80 layers** —

```
skopeo inspect --raw docker://ghcr.io/aloworld-org/alo-os@sha256:8f9c36e0…
  80 layers, 5.79 GB
```

— and what lands on the disk is larger than that, because it lands decompressed. At
124 MB a minute the install alone needs an hour or more from the machine starting,
and the installed disk's own boot — the part that shows `alo-agentd` running, which
is the whole reason this task exists — follows it. Sixteen minutes of the window had
already gone on building the environment before the machine started.

**What differs from task 13's 46-minute install on this same PC is the machine, not
the release.** Another lane was running `cargo test --workspace` and an `apt-get`
throughout, and the load average was 8.5 on four processors. The plan's line that
this PC *holds one emulated install run inside a worker's limit* is true of a quiet
PC and not of a shared one. That is recorded in the plan as the finding the plan's
own rule asks for.

## What changed

- `docs/autonomy/v0-5-the-installer-plan.md`: task 15's status keeps **ready** — it
  is not done — and now carries the measurement above and what a machine has to be
  for a worker to hold the run: a machine no other lane is gating on, or a
  supervisor running it outside a worker's window the way task 13's was run.

Nothing else. No product code was touched, because nothing in this task called for
any: the fix is already in release `0.0.2`, and the only thing left to do was to see
it work through the installer.

## Decisions

- **The run was stopped rather than left running.** A virtual machine abandoned
  when this worker ends is one whose `Leftovers` guard never runs: the plan's rule
  since 2026-09-15 is that a virtual-machine test removes its disks *pass or fail*,
  and a run killed with the session leaves about a dozen gigabytes behind on a drive that
  had 31.7 GiB. Stopping it deliberately, in time to clean up and check, is the only
  ending that obeys that rule.
- **No handoff was written, and task 15 is not marked done.** The acceptance is the
  whole test, and the whole test did not run. A handoff here would be a claim that
  the work is finished, which is the one thing a worker may never write.
- **The crate was built in a directory of its own**,
  `/root/alo-builds/this-machine-installing`, rather than the shared
  `$HOME/alo-builds/this-machine`: another lane held that one for the whole session
  with a workspace suite, and Cargo's lock would have made this run wait for it.
  It holds 694 MB and is named here rather than removed, which is what
  `tools/kernel-loop/src/where_it_builds.rs` asks of build directories — whether it
  goes is the owner's decision, and it holds an hour of compilation the next worker
  on this task can use.
- **The test itself was not changed.** It is the acceptance; editing it while it was
  the thing being measured would make the measurement worthless.

## Verification

Executed, on the third PC, in WSL 2 Ubuntu:

```
cargo test -p alo-installing --test installed_in_a_virtual_machine -- --exact \
  --include-ignored --nocapture \
  the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service
```

with `CARGO_TARGET_DIR=/root/alo-builds/this-machine-installing`. It compiled, the
environment built, the machine started under Secure Boot and reached *This is a
genuine alo OS* and the install. **It was stopped before it returned**, so there is
**no pass to report** and no exit code to quote. The serial line as far as it got is
kept on this PC as `C:\dev\setup\task15-install-run.log`.

**Not executed:** the gates. `cargo fmt --all`, `cargo clippy --all-targets` with
warnings denied and `cargo test -p alo-installing` were not run, because this change
is one paragraph of a plan document and no handoff is being written for it. The next
worker on this task runs them with its own change.

**After the run**, and this is the plan's rule rather than tidiness. The test was
stopped with `SIGTERM`, which does not unwind, so its `Leftovers` guard never ran and
the disks were removed by hand:

```
rm -rf environment staged.img windows.raw windows.before target.raw installing-vars.fd
podman rm --force alo-installing-vm-firmware alo-installing-vm-stager
podman image prune -f
```

What is left in the work directory is `firmware/` (2 MB — the Secure Boot firmware
taken out of the pinned base, which the test keeps between runs on purpose and does
not list among its leftovers) and `installing.log`, the log this report names.
`podman images` shows only the two pinned bases and `podman ps -a` nothing. The
distribution's filesystem went from 62 GB used to **51 GB**, and C: is back where it
started at 31.7 GiB free — the WSL disk image does not shrink, so the room is
returned inside it rather than to the host.

## Remaining limitations

- **Task 15 is untouched as work.** Nothing here shows `alo-agentd` running on an
  installed disk. That is still the acceptance and still unmeasured.
- **The environment build is 16 minutes of the window**, every time, because
  `podman build` starts from nothing on this machine. A worker that had the
  environment already built would have most of an hour more for the run. Whether
  that build should be kept between runs — it is a directory in
  `CARGO_TARGET_TMPDIR`, removed by the `Leftovers` guard with everything else — is
  a real question for this plan and is not decided here.
- **`docs/quirks.md` says of the BPF passage that *release `0.0.1` boots with
  `alo-agentd` failed*.** That is still accurate. It will want a sentence when a run
  shows `0.0.2` booting with it running, and not before.

## Proposed updates for the integration owner

- **CHANGELOG.md:** nothing. No behaviour changed.
- **ROADMAP.md:** no box moves.
- **QUEUE.md / STATE.md:** installer plan task 15 remains open, with the plan
  carrying the measurement of why an emulated install does not fit a worker's window
  on a PC that is also gating another lane. Worth the owner's attention: three tasks
  in this plan (12, 13, 15) have now been split or lost to the length of one
  emulated run, and task 13's run was in the end made by the supervisor rather than
  a worker. That may be the right home for this one too.
