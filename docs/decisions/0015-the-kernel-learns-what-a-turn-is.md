# ADR 0015 — The kernel learns what a turn is

**Status:** accepted — the mechanism behind
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md), and the answer to
*how do we make the kernel AI-native*
**Date:** 2026-09-03
**Context:** [ADR 0001](0001-the-capability-model.md) (the capability model),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base),
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md), `crates/alo-agentd`,
`docs/features.md`, `CLAUDE.md` (engines are configured, never patched)

## The decision in one line

alo OS teaches the Linux kernel **one new noun — the agent turn** — by attaching
our own programs to the kernel's security hooks with **BPF LSM**, so that a
grant is enforced, a socket is attributed and a record is written by the kernel
itself; **written in Rust, with no kernel patched and no fork maintained.**

## What "an AI-native kernel" actually means here

Not a model in kernel space. That has no memory protection, no fault isolation,
and a crash takes the machine down — it is rejected outright and is not the idea.

The kernel already understands processes, users, cgroups and files. **What it
does not understand is that a handful of syscalls belong together because a
person approved one sentence**, and that everything outside that sentence is
forbidden for the next four seconds.

That missing concept is the whole of it. Teach the kernel the *turn* and four
things stop being promises our own code makes about itself:

| | Today | With the turn in the kernel |
|---|---|---|
| **The grant** | `alo-capability` refuses politely, and a bug in a verb gets past it | the path is *unreachable*; an overreaching verb gets `EACCES` from the kernel |
| **The record** | what `alo-agentd` reports it did | **what the kernel watched happen** |
| **Egress** | what our code chose to tell the indicator | every socket attributed to the turn that opened it |
| **Undo** | best-effort, reconstructed from the record | exact, from a snapshot taken when the turn began |

The second row is the reason this is worth doing. An honest program's account of
itself is an **audit log**; the kernel's account is a **guarantee**, and those are
sold to a security team in completely different conversations.

## The mechanism

A **BPF map** is shared memory between our daemon and programs running inside
the kernel. The daemon writes; kernel-side programs read.

```
turn begins
  ├─ alo-agentd creates a cgroup for this turn
  ├─ writes  { cgroup_id → grant }  into a BPF map
  └─ runs the verb's work inside that cgroup

every syscall from inside it
  ├─ a BPF LSM program on file_open / inode_rename / socket_connect
  ├─ looks up cgroup_id in the map
  └─ returns 0 (allowed) or -EPERM (refused, by the kernel)

turn ends
  ├─ the entry is removed
  └─ authority is gone — not revoked later, gone
```

These are the same security hooks SELinux and AppArmor use. Our programs are
**verified by the kernel before they are allowed to run**, loaded at boot, and
require no module compiled against a kernel version.

**Rust on both sides.** `aya` compiles Rust to BPF bytecode, so the kernel-side
programs and the daemon that loads them are one language and `CLAUDE.md`'s
two-languages rule survives intact. The C route (`libbpf`) would put a third
language in this repository, which that rule calls a bug.

## Four depths, and only one of them is a fork

- **0 — Linux's own controls, used properly.** Landlock, seccomp, cgroup v2 with
  eBPF, namespaces, `SO_PEERCRED`. Decided in ADR 0013; item 21c shipped the
  last of them.
- **1 — a BPF LSM that knows about turns.** *This ADR.* Kernel-level enforcement,
  our policy, no fork.
- **2 — an out-of-tree kernel module.** A full LSM in C, if a BPF hook turns out
  to be insufficient. More power, and it recompiles against every kernel version
  forever. **Only with a named capability depth 1 could not express**, written
  into a new ADR.
- **3 — patching the kernel and keeping a fork.** Rejected. Every upstream
  release becomes a merge and every CVE becomes ours, and it buys nothing depths
  0–2 do not. **If a hook is genuinely missing, the answer is to propose it
  upstream** — which is how Landlock itself arrived.

`CLAUDE.md`'s *configured, never patched* holds at depths 0 and 1. Depth 2 does
not patch either, but it is a maintenance burden and needs its own decision.

## The rule that keeps this from becoming surveillance

**A BPF LSM sits on the security hooks, which means it sees everything by
construction.** The mechanism that gives us enforcement would also give us total
surveillance — the capability is identical and only the discipline differs. This
is the single most dangerous thing in this repository, and it is worth saying so
in the document that introduces it.

**So: the LSM decides and forgets.** A syscall outside an agent turn is checked
and leaves **no trace** — not a log line, not a counter, not a timestamp. A
person's editor, browser and terminal pass through and are forgotten. **Only
turns are recorded, because only turns had an agent in them.**

This is not a promise, it is a test: run ordinary programs under the LSM and
assert the record is **empty**. The day somebody adds *just a little telemetry*,
that test fails and names what it caught. Without the test the rule would erode
in a year, one reasonable-sounding feature at a time.

It also keeps ADR 0001 §4 true at the depth where it would be easiest to break:
**a background reader is a bug in this product, not a feature request.**

## Consequences

- **A new crate**, and it is `alo-agentd`'s floor: everything it does happens
  before the first verb of a turn runs.
- **The base gains a requirement.** `CONFIG_BPF_LSM=y`, and `bpf` present in
  `CONFIG_LSM`. The Fedora-derived base of ADR 0011 ships both — this is a kernel
  **config** expectation, not a patch, and it belongs in `docs/hardware.md` as
  something a certified machine must have.

  **Two more of these were found by building it, and both are written into
  `docs/hardware.md` rather than left here.** The list above is *how the kernel
  was built*; what decides is *what it started* — a kernel can have
  `CONFIG_BPF_LSM=y` and never register the module — and beyond that, *whether a
  programme can be attached at all*, because attaching one waits on an RCU-tasks
  grace period and a kernel whose grace periods have stalled hangs the attach
  forever in uninterruptible sleep. Three checks, in the order they fail in, and
  each invisible to the one before it.

- **There is `unsafe` in the half that runs in the kernel, and this ADR is where
  that was decided.** *Rust on both sides* means writing a programme whose input
  is a raw pointer the kernel hands over and whose only way to dereference
  anything is a helper call; there is no safe spelling of that and no crate to
  rent one from, because the safety comes from the kernel's verifier refusing to
  load a programme that would read out of bounds — a check stricter than the
  compiler's and made after it. So `crates/alo-bounding-kernel` denies
  `unsafe_code` at its root and lifts it in exactly one file, `kernel.rs`, which
  is the shape `alo-agentd`'s `unix.rs` and `signalling.rs` already have. Four
  things need it: the exported symbol, the hook's argument, a read of kernel
  memory, and a map lookup. The half that runs on this machine,
  `crates/alo-bounding`, has none and is a member of the workspace that forbids
  it.
- **`aya` is a rented dependency** and is named in one file, as ADR 0006 does
  with the model runtime and item 21c did with `SO_PEERCRED`.
- **v0.01's owed line is answered.** *Enforcement at the network boundary,
  without which all of this describes only the code that asked* is a BPF program
  on the turn's cgroup, and it now has a mechanism rather than an intention.
- **A turn whose boundary cannot be applied does not run.** If the LSM cannot
  load, that is a refusal and not a warning — the same rule `alo-egress` already
  follows when a policy cannot be evaluated.
- **Ordered behind `alo-agentd` and the turn.** There is no turn to enforce yet.
  This is written now so the turn is built with a boundary in mind rather than
  retrofitted into one.

## Alternatives rejected

**Leave enforcement in userspace.** Rejected: it is the current state, and it
makes the record only as trustworthy as the program writing it.

**A model, or agent logic, in kernel space.** Rejected on every ground —
stability, security, and no benefit.

**An out-of-tree C LSM now.** Not rejected, deferred: it costs a permanent
rebuild treadmill and a third language, and nothing yet names a thing BPF LSM
cannot express.

**A kernel fork.** Rejected, and the reasoning is
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)'s: building
what already exists buys a worse copy of something free, and the agent never
touches the part a fork would let us change.
