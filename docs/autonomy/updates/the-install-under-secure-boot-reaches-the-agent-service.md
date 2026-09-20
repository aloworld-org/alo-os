# The install under Secure Boot finishes, and the installed disk boots to the agent service

**Date:** 2026-09-20
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`, task 15,
*A release that carries the way to the boundary, installed under Secure Boot to the
agent service*)
**Contributor:** Claude Code lane B in `/root/alo-os-lane-b` on the **development PC**,
for the owner
**Status:** **done.** The named test passed whole, with Secure Boot on, against the
release pinned now — `0.0.4` at
`sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858`, revision
`b41b4b5e`. The run is pasted below.

Follows `updates/the-install-under-secure-boot-does-not-fit-a-workers-window.md`
(the 2026-09-18 attempt on the third PC), which is not edited here: what it measured
was true of that machine, and this run does not contradict it.

## The machine, which is the whole reason this run finished

| | Third PC (`AGAI01`), 2026-09-18 | Development PC, 2026-09-20 |
|---|---|---|
| Processor | — | Intel Core Ultra 7 155U, **12** processors to WSL |
| Hardware virtualisation | absent; `-accel kvm` refuses | **present**; the test's own probe started a machine with `-accel kvm` and it stayed up |
| How the guests ran | `-accel tcg -cpu max` (emulated) | `-accel kvm -cpu host` |
| The install | **about 124 MB a minute**, 4.7 GB in 38 minutes, stopped unfinished | **whole, in about 11 minutes** |
| The named test | stopped after ~90 minutes, unfinished | **`ok`, in 1303.36 s — 21.7 minutes, including building the environment** |

The previous report's finding stands and is now narrowed by measurement: the run does
not fit a worker's window **on an emulated machine**. On a machine with working
hardware virtualisation it fits inside a worker's window with room to spare, and the
plan's instruction to check `uptime` and `pgrep -af cargo` first is still right — this
run was started with lane A's `cargo test --workspace` beside it, load average 8.3 on
twelve processors, and it still finished.

## The run

```
running 1 test
test the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 1303.36s
```

Exit 0. The machine was started by a firmware with Secure Boot on — not a flag the
test asserts from the outside, but the guest kernel's own account of itself, twice,
in `installing.log`:

```
[    0.000000] secureboot: Secure boot enabled
[    0.014016] secureboot: Secure boot enabled
```

and the arguments that put it there, `-machine q35,smm=on` with
`-global driver=cfi.pflash01,property=secure,value=on` over the `OVMF_CODE.secboot.fd`
taken out of the pinned base rather than from this host.

### What the environment said, in order, with nothing refused

Every sentence the acceptance asks for, from `installing.log`:

```
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: virtio-alo-target
Checking that virtio-alo-target is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto virtio-alo-target. Everything that was on that disk is being replaced. This takes a while, and this screen will say when it is done
Still installing alo OS. Leave the computer on          (ten times)
alo OS is installed. This computer restarts in a few seconds
```

*This is a genuine alo OS* is the owner's signature over the pinned digest, verified
**inside** the machine against `signing/alo-os.pub`. None of the four refusals —
*not genuine*, *not reachable*, *not installed*, *restart when ready* — appears
anywhere in the log; the test asserts each one's absence by name.

The install began at guest monotonic 20.7 s and the environment shut down at 659.0 s.

### The first disk

The test hashes the Windows disk before the install and compares it twice: *the first
disk is unchanged during the install*, and *the first disk is unchanged when alo OS
booted*. **Both passed.** The second is the assertion no run had reached before this
one, because no run had ever got the installed disk to boot.

### The installed disk, booted on its own under Secure Boot

`installed.log`, the machine's own answer:

```
Id=alo-boundaryd.service
ActiveState=active
SubState=exited

Id=alo-agentd.service
ActiveState=active
SubState=running
```

and at length:

```
● alo-agentd.service - alo OS agent service: the door an agent knocks on
     Loaded: loaded (/usr/lib/systemd/system/alo-agentd.service; enabled; preset: disabled)
     Active: active (running) since Sun 2026-09-20 14:00:49 UTC; 30s ago
   Main PID: 1137 (alo-agentd)
     CGroup: /system.slice/alo-agentd.service
             └─1137 /usr/bin/alo-agentd
```

The boundary was loaded first and got out of the way, which is ADR 0018's shape:

```
alo-boundaryd: the boundary is on this kernel, pinned at /sys/fs/bpf/alo, with 60989
to write it and nobody else; this process holds nothing and is done
```

So the passage task 14 added in `image/usr/lib/tmpfiles.d/alo.conf` — the agent's
group through `/sys/fs/bpf` — **is in release 0.0.4 and works through the installer**,
which is the thing task 15 existed to show. Nothing on the installed disk was changed
by the test to make a service start: no `tmpfiles.extra` credential, no patched unit.
The one credential the test passes is the reporting unit that reads `systemctl` out
onto the serial line.

## Two lines in the logs that are not faults, and one of them was nearly written up as one

**A released alo OS machine says it loaded no translations, and that is correct.**
On the installed disk, at first boot:

```
alo-agentd: no translations were loaded: /usr/share/alo/translations could not be
read — No such file or directory (os error 2). This machine speaks English until it
is fixed
```

This is the first time the line has been seen on a machine installed from a **released**
image rather than from a scratch disk, so it is worth recording that it survived into
0.0.4 — but it is **not** a defect and it is not being filed as one.
`updates/the-agent-service-can-reach-the-boundary-on-an-installed-disk.md` already
recorded it on 2026-09-16 with the reason: **no translation exists in this repository
yet.** Checked again here — there is no `.toml` translation anywhere in the tree —
so there is nothing for the image to carry, `alo-saying` is correctly reporting an
empty case, and shipping the directory empty would change the log line without
changing what the machine can say.

This report drafted that finding as *a European product whose machine only speaks
English*, and that was wrong, so the correction is left visible rather than deleted:
the catalogue is absent because nobody has written a translation, not because the
image drops one. The open question is unchanged and is still somebody else's — whether
the image should ship the empty directory, which task 14 put to the owner of
`alo-saying`. Nothing is ticked here and nothing goes to `docs/quirks.md`: that file
is for hardware and applications that misbehave, and this is our own component
correctly saying it found nothing.

A second line, from the firmware rather than from us, and not a fault:

```
PageFaultExitBoot: Applying global page table fixup (shim is older than v16).
```

## Disk, as the plan requires

15 GB free was checked before the run: C: had 47.3 GB. The run's disks and images were
removed when it ended — by the test's own `Leftovers` guard, which drops them whether
it passes or fails — and `fstrim -v /` then returned **20.5 GiB** to Windows, leaving
45.0 GB free. Deleting inside the vhdx alone returns nothing; the trim is the step that
does.

Kept: the two serial logs this report quotes, at `/root/lane-b-logs/serial/` on the
development PC.

## What is not claimed

- **This is one virtual machine, not a laptop.** Task 6 (the certified laptop,
  firmware to the daemon) and v0-01 delivery task 12 still need hardware that does not
  exist, and neither is ticked by this run.
- The guest's disks are virtio, its firmware is OVMF, and its Windows disk is a
  fixture. *Alongside Windows* (task 4) and *replace Windows* (task 7) are their own
  tasks and are untouched here.
