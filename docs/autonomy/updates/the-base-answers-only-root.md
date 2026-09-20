# The base answers only root

**Date:** 2026-09-20
**Workstream:** v0.5 — the machine keeps itself
(`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 11, *The check on a
machine that is not this one*)
**Contributor:** Claude Code worker in `/root/alo-os-lane-b` on the development
PC, for the owner
**Status:** ready for integration. Task 11 is complete and marked in the plan.
Nothing in `crates/` changed: this task was two measurements, and what they found
is written as a decision and as task 12 rather than as code.

## The machine

Everything below was measured on **one real bootc machine**, named here so that
no line of it can be mistaken for a test with a stand-in behind it.

| | |
|---|---|
| Name | `alo-lane-b-bootc` |
| What it is | the pinned image installed to a 20 GiB disk with `bootc install to-disk --via-loopback --wipe --filesystem ext4`, booted under KVM (`-accel kvm -cpu host -smp 4 -m 4096`, OVMF) on the development PC |
| Image | `ghcr.io/aloworld-org/alo-os:0.0.4`, pulled **by the pinned digest** `sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858` |
| Base | Fedora Linux 42 (Adams), kernel 6.19.14-101.fc42.x86_64, `bootc` 1.15.1 |
| The person | `alo`, uid 1000, gid 1000, in `alo-agent` (60989) — the image's own `sysusers.d`, unedited |

The check program is `crates/alo-looking-once`, built from this tree for
`x86_64-unknown-linux-musl` (static-pie, 2 994 936 bytes) and copied onto the
machine. It is the unit's own program; nothing about it was changed to be
measured.

## The first half: the base refuses the person

**`bootc status` refuses an unprivileged caller, in every spelling.**

| asked as | how | exit | stdout | stderr |
|---|---|---|---|---|
| root | `bootc status --format json --format-version 1` | 0 | 972 bytes of the booted host | — |
| uid 1000 | the same, through `runuser -u alo` | 1 | **0 bytes** | `error: Status: Preparing for write: Querying root privilege: This command must be executed as the root user` |
| uid 1000 | the same, in a login shell (`su - alo -c`) | 1 | 0 bytes | the same sentence |
| uid 1000 | the same, `systemd-run --uid=1000 --gid=1000` — the way the unit runs it | 1 | 0 bytes | the same sentence |
| uid 1000 | `--format yaml`, the human form, and `--booted` | 1 | 0 bytes | the same sentence |

`bootc` is `0755 root root` with no file capabilities and is not setuid, so there
is nothing on the machine to widen: the refusal is the program's own, and it is
taken on the *write* path — `Preparing for write: Querying root privilege` —
before it reads anything, which is why no read-only form of the question gets
past it.

**And the unit fails, as the unit, on the machine.** Run as the person:

```
alo-looking-once: this machine did not find out whether there is an update: the
base would not say which build this machine is running: NotAnswered(SaidNo {
program: "/usr/bin/bootc", code: Some(1), said: "error: Status: Preparing for
write: Querying root privilege: This command must be executed as the root user"
}) — nothing left this machine
EXIT=1
```

Nothing was kept and nothing left the machine. Run as root on the same machine a
minute later, the same program asked `ghcr.io` and kept an answer. **Task 10's
central decision — that the check runs as the person — is right about privilege
and wrong about its source of truth**: as written, *updates that never interrupt*
is a unit that fails at every boot on every real machine.

## What the base will tell the person, without being asked to

The answer is not to make this component root, and it does not have to be. The
base already writes down, world-readable, exactly what the check needs — and both
of these were read **as uid 1000** on this machine:

| read as the person | what came back |
|---|---|
| `cat /ostree/deploy/default/deploy/*.0.origin` (`0644 root root`) | `container-image-reference=ostree-unverified-registry:ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…` |
| `ostree admin status` | `* default f9ce6166…0` — the `*` is the booted deployment |
| `ls /sysroot/ostree/deploy/default/deploy/` | both the deployment and its `.origin` |

So *which build is this machine running* is answerable to an unprivileged caller
today, on a stock base, with no new component and no widened grant. **ADR 0001 §2
does not fire and no ADR is owed for privilege.** What it costs is one sentence of
ADR 0011 — *the base is spoken to through its own command* — which stays exactly
as it is for `upgrade`, `switch`, `rollback` and everything else that changes the
machine, and is narrowed only for the one question the base will not answer to
the person at all. Writing that narrowing, and the code, is task 12.

## The thing nobody was looking for

On this machine the two digests disagree, and the one the check compares is the
wrong one:

```
spec.image.image                  ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…
status.booted.image.image.image   ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…
status.booted.image.imageDigest   sha256:2e7ecd95…
the origin file                   …@sha256:48bd5f31…
```

`alo_keeping_up::Running` reads `imageDigest`; `Standing::between` compares it to
what the registry answers. So the answer this machine kept, running exactly the
pinned build, was:

```json
{"about":"sha256:2e7ecd95…","offered":"sha256:48bd5f31…","vouched_for":false,
 "because":"this-machine-started","at":1789906688}
```

— *a newer version of this machine's system is available*, on a machine that is
already it. Both digests were measured for the same image: `podman` reports config
id `74a4aa1563c0…` for both, the local container store's manifest digest is
`sha256:2e7ecd95…` and the registry's is `sha256:48bd5f31…`, and this machine was
installed from the local store, which is where its `imageDigest` came from.

**Not measured, and named rather than guessed:** whether a machine installed
straight from the registry records the registry's digest and does not show this
at all. That is one install away and it is task 12's first line. Worth noting that
the origin file carries the digest that *does* match what the registry offers, so
the fix above closes this too, whichever way the install turns out.

## The second half: the company network

A real proxy, not a test's argument: a process listening on the host at
`10.0.2.2:3128` that speaks `CONNECT` and appends every request it is given to a
log, named to the machine in its own `/etc/alo-proxy/proxy.json` — `manual`,
`10.0.2.2:3128` for both roads, `set_by: an-organisation`, `0644`, exactly the
shape `docs/contracts/machine-proxy-file.md` fixes. Departures were counted at the
boundary two ways at once: by the proxy's own log, and by `nftables` counters in
the machine's own `output` hook, one for the proxy and one for anything going
straight out to port 443.

| the proxy is | the check said | through the proxy | straight out to 443 | the boundary's log |
|---|---|---|---|---|
| set and reachable | kept the answer, exit 0 | **98 packets, 7 890 bytes** | **0 packets** | 6 × `CONNECT ghcr.io:443` |
| set, nothing listening on the port | *there is no way out of this machine to the place its updates come from*, exit 1, nothing kept | 0 | **0 packets** | nothing: one SYN at the dead port, 60 bytes, never answered |
| there, and not a setting this machine keeps | *the machine's proxy at /etc/alo-proxy/proxy.json could not be read, so nothing was asked … — nothing left this machine*, exit 1 | 0 | **0 packets** | nothing |

Six departures is task 10's six — two questions, each answered `401` and asked
again with a token — arriving at the proxy as `CONNECT` rather than as connections
of the machine's own. **A proxy that is set and cannot be reached refuses, and a
machine on a company network does not go around its own proxy**: the straight-out
counter is zero in all three rows, including the one where the only road was
broken and going around would have worked.

## What is handed over

- **The code that reads the origin file**, and the narrowing of ADR 0011 it needs
  — task 12, written into the plan in this change.
- **The registry-install digest question** above — task 12's first line.
- **The image installation** of `alo-looking-once` is still the installer lane's,
  exactly as task 10 handed it over; this measurement copied the program onto the
  machine by hand, and says so.

## Verification

No crate changed, so there is nothing for the suite to say about this that it did
not say yesterday; the gate was run on the tree anyway and its result is in the
pull request. What this change contains is two documents.
