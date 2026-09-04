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
ADR 0015 names two things, they are not the same thing, and a third was found by
trying to use a machine that satisfied both:

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

The three checks are in the order the failures happen in, and each one is
invisible to the one before it: built in, started, and attachable.

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
