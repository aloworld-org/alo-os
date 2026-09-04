# Hardware

**Certified first, compatible later.** A machine model bought twice and working
completely — that is the standard. A compatibility list grows outward from
there.

Since ADR 0007 that standard is held **twice**: an ordinary business laptop and
a GPU workstation, with the laptop first. This line said "one machine model"
until the ADR landed, two paragraphs above the section that already said two.

"Supports PCs" is not a claim anyone can honour, and hardware support is where
operating system projects die. The discipline of refusing scope here is the only
mitigation there is; if it slips, that risk is unmitigated.

## Status

**Nothing is certified yet.** alo OS is pre-v0.01 and this table is the shape
the answer will take, not an answer.

## Certified

A machine is **certified** when everything in `docs/features.md` for the current
release works on it, verified on a physical unit by someone who owns one — not
inferred from a chipset. That includes the unglamorous parts: suspend and resume,
external displays, and printing.

| Machine | GPU / VRAM | Status | Verified | Notes |
|---|---|---|---|---|
| _(none yet)_ | | | | |

**Two machines are certified, not one** (ADR 0007), and the first matters more.

**An ordinary business laptop** — no discrete graphics, 16 GB of memory — running
its agents on the CPU. This is the machine that decides whether this project has
a market: the Windows 10 fleet it exists to catch has almost no discrete GPUs in
it, and a system those machines cannot run agents on is a system they cannot
adopt.

**A GPU workstation** with **24 GB VRAM or more**, for alo OS AI. The 24 GB floor
is what makes a large model run at a useful speed and makes fine-tuning
practical. It is acceleration, not an entry price.

**alo OS Desktop** — the non-GPU SKU — inherits the CPU default and is where
most machines will land. Starting with one recent business-class model, then the
Windows 10 fleet by generation.

## Compatible

A machine is **compatible** when it boots and works, but is not something we
verify on every release. Community reports are welcome and go here with the
reporter and the date, so a reader can judge how stale the claim is.

| Machine | GPU / VRAM | Works | Doesn't | Reported by | Date |
|---|---|---|---|---|---|
| _(none yet)_ | | | | | |

## What "the GPU works on first boot" means

It is a promise, so it needs a definition. On a certified machine, from a fresh
image:

- the display comes up at native resolution without configuration;
- the GPU is available to the model runtime with no driver installation, no
  CUDA or ROCm archaeology, and no virtualenv;
- pulling and running a model from the catalogue is one command;
- an upgrade cannot break that stack — the model runtime is versioned together
  with the drivers it needs, and a bad deployment rolls back.

If any of those is false on a machine, that machine is not certified, whatever
else works.

## What the kernel must be able to do

A certified machine's kernel has to be able to enforce a grant, which is a
requirement about how the kernel was **configured** and not about the silicon.
ADR 0015 names two things, they are not the same thing, a third was found by
trying to use a machine that satisfied both, and a fourth is what the boundary
reads rather than what it attaches:

- **`CONFIG_BPF_LSM=y`** — the BPF LSM is compiled into the kernel.
- **`bpf` present in the list of security modules that actually start**, which
  is `CONFIG_LSM` unless an `lsm=` boot parameter replaces it.

The second is the one that gets missed, because the first is the one people
check. A kernel can have the BPF LSM compiled in and never start it, and then
`CONFIG_BPF_LSM=y` is a true answer to the wrong question.

Ask the machine both, in this order:

```
zcat /proc/config.gz | grep -E '^CONFIG_(BPF_LSM|LSM)='   # how it was built
mount -t securityfs securityfs /sys/kernel/security       # if not already mounted
cat /sys/kernel/security/lsm                              # what actually started
```

The last line is the answer. It lists the security modules this kernel is
running, and `bpf` is either in it or the grant cannot be enforced by the kernel
on this machine.

**A worked counterexample, measured rather than supposed.** The WSL2 kernel this
repository can reach — `6.6.87.2-microsoft-standard-WSL2`, 2026-09-03 — had
`CONFIG_BPF_LSM=y` and answered `capability,landlock,yama,safesetid,selinux`: a
kernel that passes the first check, fails the second, and is no use for proving
ADR 0015. `docs/quirks.md` has the entry.

**And then the remedy, on the same kernel, 2026-09-04.** The machine's owner set
a boot parameter and the same kernel now answers
`capability,landlock,yama,safesetid,selinux,bpf`. Nothing was rebuilt and nothing
was patched, which is the point: *a kernel that fails the second check is a
kernel that was booted wrongly, not a kernel that cannot do it.* Ask a machine
that fails what it was booted with before concluding anything about what it can
run.

One trap belongs next to the fix, because it turns a protection into a
regression. **`lsm=` replaces the built-in list; it does not add to it.** Writing
`lsm=bpf` starts the BPF LSM and silently stops every module that was enforcing
before it. The parameter has to name the modules the kernel already ran *and*
`bpf` — which is why the line above has six names in it and not one. Read
`/sys/kernel/security/lsm` before and after, and compare the two, rather than
checking only that `bpf` appears in the second.

**There is a third requirement, and it was found by a machine that passed both
of the above.** A BPF LSM programme is attached through a *trampoline*, and
building one waits for an RCU-tasks grace period to complete. On a kernel where
that machinery has stalled the attach never returns, in uninterruptible sleep,
and cannot be killed — no error, no timeout, and every later BPF attach on the
machine queues behind it until a reboot. So:

- **The kernel's RCU-tasks grace periods must complete.**

```
dmesg | grep -c tasks_rcu_exit_srcu_stall                 # must be 0
```

Anything but zero means no BPF LSM, `fentry` or `fexit` programme will attach on
that machine, however the first two checks answer. **Ask it on a machine that has
been up for more than a minute**: the stall is reported about ten seconds after
it begins, so a zero read at the login prompt is a question asked too early
rather than an answer. That is the only qualification this check needs — the
window is seconds, not the tens of minutes an earlier reading of the ring buffer
suggested.

The WSL2 kernel above is the worked counterexample for this one too: measured
2026-09-04, `6.6.87.2` starts the BPF LSM and cannot be attached to, because a
grace period stalls about twenty seconds after boot and never finishes. It was
rebooted and measured again and reproduced exactly — same grace period number,
same timing — so it is a property of that kernel rather than a bad run.

**And it is a property of that kernel, not of the platform.** The same machine
upgraded to `6.18.33.2` the same day passes all three checks: zero stalls past
the blind window, and an attach that returns in 0.08 seconds where it had hung
twice. The lesson is worth more than the fix — *a kernel that fails one of these
is a kernel to upgrade before it is a machine to replace*, and the first
question to ask of any machine that fails the third check is what it is running.
`docs/quirks.md` has both measurements.

**There is a fourth, and it is about what the kernel says rather than what it
does.** alo OS attaches no module compiled against a kernel version (ADR 0015),
so the boundary asks the running kernel where its own structures are and refuses
to load on one that will not say. That answer is published only by a kernel built
with `CONFIG_DEBUG_INFO_BTF=y`:

- **The kernel publishes its own type information.**

```
test -s /sys/kernel/btf/vmlinux                           # must exist, and not be empty
```

A machine that fails this fails at start-up with a sentence naming the file,
which is what somebody needs in order to work out that a config option is off.
Every distribution kernel worth certifying has it on, and it is here because a
missing answer is one of the four ways this boundary declines to be imposed.

**And there is a fifth, which is about where the boundary is kept rather than
about the kernel that runs it.** Since
[ADR 0018](decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
the programme is loaded once at boot by `alo-boundaryd` and **pinned**, so that
`alo-agentd` — which runs as the signed-in person and holds no capability — can
open one of its maps by path. Pinning is a `bpffs` operation and there is nowhere
to pin without one:

- **A BPF filesystem is mounted at `/sys/fs/bpf`.**

```
mount | grep '/sys/fs/bpf'                                # type bpf, not sysfs
```

`systemd` mounts it on any ordinary machine, which is why this is last: it is the
check that is almost never the answer, and it is here because a machine without
it fails at start-up naming a directory rather than a kernel. A machine that
fails it is told so by `alo-boundaryd` before anything is loaded.

The five checks are in the order the failures happen in, and each one is
invisible to the one before it: built in, started, attachable, self-describing,
and somewhere to keep it.

### The image's own kernel, asked three of the five

**ADR 0015's *the Fedora-derived base ships both* was read and not measured
until 2026-09-04.** The paragraph below it said whoever certifies the first
machine would find out. Queue item 28 built the image, so it can be asked now —
of a kernel alo OS chose rather than one it borrowed.

| Kernel | Built in | Started | Self-describing | Measured |
|---|---|---|---|---|
| `6.19.14-101.fc42.x86_64`, from `quay.io/fedora/fedora-bootc:42` | `CONFIG_BPF_LSM=y` | `bpf` is in `CONFIG_LSM` | `CONFIG_DEBUG_INFO_BTF=y` | 2026-09-04, from `/usr/lib/modules/<version>/config` in the built image |

```
CONFIG_BPF_LSM=y
CONFIG_LSM="lockdown,yama,integrity,selinux,bpf,landlock,ipe"
CONFIG_DEBUG_INFO_BTF=y
```

**The second column is the one that was in doubt**, and it is the one that
usually fails: a kernel can have the BPF LSM compiled in and never start it. This
one starts it in its built-in list, with **no boot parameter of ours** — so the
`lsm=` trap two paragraphs above is a trap alo OS's image does not have to walk
into, and ADR 0015's sentence is measured rather than hopeful.

**Two of the five were unanswered because nothing had booted the image.** Whether
grace periods complete and whether `bpffs` is mounted are questions about a
kernel that is *running*, and what the table above reads is the configuration
file the image ships beside its kernel.

### The image booted, and the other two answered themselves

**2026-09-04, in QEMU with KVM**, from a disk `bootc install to-disk` wrote:
UEFI → GRUB 2.12 → the ostree deployment → the kernel. What the console said:

```
LSM support for eBPF active
landlock: Up and running
bpf-restrict-fs: LSM BPF program attached
tasks_rcu_exit_srcu_stall — 0 occurrences

[  OK  ] Finished alo-boundaryd.service
[  OK  ] Started  alo-agentd.service
```

- **Grace periods complete.** Zero RCU-tasks stalls. This is the requirement
  that cost a morning and made the WSL2 kernel useless for any BPF work: there,
  a grace period stalled twenty seconds after boot on every boot and no
  programme could ever attach. On this kernel it does not happen.
- **The LSM machinery is live, and not only ours.** `systemd` attached its own
  `bpf-restrict-fs` programme during boot, which is independent corroboration:
  the mechanism works here rather than merely being compiled in.
- **Both units came up, in order.** `alo-boundaryd` finished — it is
  `Type=oneshot`, so *finished* is success — and `alo-agentd` then started,
  which it could not have done if the boundary had failed, because its unit says
  `Requires=`. [ADR 0018](decisions/0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
  is now a thing that has happened rather than a thing that was decided.
- One unit did fail: `systemd-ssh-generator`, on `Failed to query local AF_VSOCK
  CID`. That is systemd looking for a QEMU guest channel the machine was
  deliberately not given. Nothing of ours.

**`ROADMAP.md`'s image line keeps its machine box empty, and this is why.** A
virtual machine is not the certified machine law 3 asks for: no firmware of a
model somebody bought, no suspend, no external display, no printer. What this
boot settles is that the image is *bootable* and the units are *correct* —
which is what the five kernel questions were for. It does not settle that alo OS
runs on hardware, and ticking the box on a VM would be exactly the optimistic
tick `ROADMAP.md`'s own preamble forbids.

**Who loads it now has an answer, and whoever certifies a machine should know
it.** Not the agent's daemon: ADR 0018 gives the `CAP_BPF` and `CAP_SYS_ADMIN` a
BPF LSM programme needs to one small service that runs at boot, takes no
argument, and exits. On a certified machine `alo-boundaryd.service` runs before
`alo-agentd.service` and the boundary is on the kernel before any person signs
in — so *is the boundary in force* is a question about the boot, answered by
`ls /sys/fs/bpf/alo`, rather than a question about whichever session is open.

**All four were run and the boundary was proved on 2026-09-04**, on
`6.18.33.2-microsoft-standard-WSL2`: a process in a turn's control group, granted
one folder, opened a file inside it and was refused `EACCES` by the kernel when
it reached for a private key beside it —
`crates/alo-bounding/tests/the_kernel_refuses.rs`. That is a development machine
rather than a certified one, and it settles the mechanism rather than the
hardware.

**This is a configuration expectation, not a patch.** `CLAUDE.md`'s *engines are
configured, never patched* holds: what a certified machine needs is a kernel
built and booted with the BPF LSM on, and if the base of ADR 0011 does not ship
that, the answer is a boot parameter in the image rather than a kernel of our
own. Whoever certifies the first machine runs the three lines above and writes
the answer into the table, because *the Fedora-derived base ships both* is at
present something this repository has read and not something it has measured.

## Reporting hardware

Tell us the machine, the GPU and VRAM, the image version, what worked, and what
didn't. Firmware quirks, driver misbehaviour and anything where reality
disagrees with the specification belong in `docs/quirks.md` — with the version
and the date, so the next person inherits the knowledge rather than the
debugging session.
