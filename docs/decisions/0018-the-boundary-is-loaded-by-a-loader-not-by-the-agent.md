# ADR 0018 — The boundary is loaded by a small privileged loader, never by the agent's daemon

**Status:** accepted — answers the question
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md) left open and
[ADR 0001](0001-the-capability-model.md) §2 already had the shape of
**Date:** 2026-09-04
**Context:** `docs/autonomy/QUEUE.md` item 26e, found by 26d wiring the daemon to
ADR 0015; `crates/alo-agentd`, `crates/alo-bounding`, `crates/alo-bounding-kernel`;
the image (queue item 28), which is where the unit file lives

## The decision in one line

A **small privileged loader** installs the BPF LSM programme once at boot and
pins its maps; `alo-agentd` keeps running as the signed-in person with **no new
capabilities**, and writes grants into the pinned map. alo OS gains one
privileged component, and the agent's daemon is not it.

## The question, and why it could not be dodged

Item 26d implemented ADR 0015 faithfully and thereby asked something nobody had
answered. Loading a BPF LSM programme needs `CAP_BPF` and `CAP_SYS_ADMIN`.
`alo-agentd` runs as the signed-in person. So as of 26d **the daemon refuses to
start on any machine where it cannot get them** — correct behaviour, and not yet
a machine anybody can boot.

Two answers were on the table, and the queue was right that they are different
products: give the service the capabilities through `systemd`'s
`AmbientCapabilities=`, or have something privileged impose the programme once
at boot.

## Why the first answer is not available

ADR 0001 §2 does not only say *never as root*. Its second clause is the one that
decides this:

> `alo-agentd` runs as the signed-in person. Never as root, **never with
> capabilities the person does not have.**

`CAP_BPF` and `CAP_SYS_ADMIN` are emphatically capabilities a person does not
have. `AmbientCapabilities=` is therefore not a careful reading of ADR 0001 §2 —
it is the thing ADR 0001 §2 is written to forbid, wearing a systemd directive.

The security argument is worse than the textual one. **`CAP_BPF` is not the
power to load our programme; it is the power to load any programme.** A daemon
holding it can attach a BPF program to every syscall on the machine, which is
precisely the capability [ADR 0015](0015-the-kernel-learns-what-a-turn-is.md)
identifies as its one dangerous property and which queue item 27 wrote a test to
prove we do not use. That test asserts *our* programme records nothing. It says
nothing about a second programme loaded by a compromised daemon.

So the trade is: the agent's daemon is the largest, most network-exposed,
most-likely-to-be-compromised component in the system, and the first answer
proposes giving exactly that component the ability to watch every syscall on the
machine. **A boundary that requires the bounded thing to hold kernel-wide power
is not a boundary.**

## Why the second answer is the one alo already chose

ADR 0001 §2 does not leave privileged work unaddressed. It names the pattern in
the sentence immediately after the prohibition:

> The genuinely privileged operations — printer configuration, network settings,
> system updates, storage — sit behind a **separate broker** with its own fixed
> verb list and no free-form parameters. The broker is small enough to audit in
> an afternoon, and that is a design constraint on it, not an aspiration.

Loading the boundary is a genuinely privileged operation. It gets the treatment
ADR 0001 §2 already prescribes for genuinely privileged operations. This ADR
adds no new architectural idea; it applies an existing one to a case the
original list did not enumerate.

ADR 0015 also already said it, and item 26e is right that it can be read
literally: the programmes are *"verified by the kernel before they are allowed to
run, **loaded at boot**"*. Loaded at boot means loaded by something that runs at
boot, which is not a per-person session daemon.

## The shape

| | runs as | holds | does |
|---|---|---|---|
| `alo-boundaryd` | root, at boot, then idle | `CAP_BPF`, `CAP_SYS_ADMIN` | loads the one programme, pins its maps, sets their ownership |
| `alo-agentd` | the signed-in person | **nothing new** | opens the pinned map by path, writes one grant per turn |

- **It loads exactly one programme**, the one built from
  `crates/alo-bounding-kernel` and shipped in the image. It takes no path, no
  name and no argument that selects what to load. There is no verb here at all,
  which is a stronger version of ADR 0001 §2's *fixed verb list*.
- **It exits, or idles holding nothing.** The capabilities are needed to load
  and pin; they are not needed afterwards.
- **The interface between the two is the pinned map**, not an API. `alo-agentd`
  opens it by a known path and writes `{cgroup id → grant}`. That is the whole
  contract, and it is the one ADR 0015 already describes.
- **Map ownership is how the person is let in**: the pinned map is owned by the
  agent's group, mode `0660`, the same shape as the socket in
  [ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md). Writing a
  grant needs no capability, only permission on the map.

## What this costs, stated plainly

**alo OS now has one privileged component, where it had none.** That is a real
loss and it should be named rather than buried: every privileged component is a
thing that can be got at, and "no privileged components" was a cleaner sentence
to sell than "one, and it is very small".

What makes it acceptable is the size of what it is trusted with. `alo-boundaryd`
takes no input, makes no decision, and can do exactly one thing. It is auditable
in an afternoon by the standard ADR 0001 §2 sets for the broker, and unlike the
broker it has no verbs to get wrong. The alternative was a *large* privileged
component — the whole agent daemon — which is the same loss with none of the
containment.

## What we rejected

**`AmbientCapabilities=` on `alo-agentd`.** Forbidden by ADR 0001 §2's second
clause, and it hands kernel-wide observation to the most exposed process in the
system. Rejected on both grounds independently.

**setuid or a capability on the binary.** Same objection, worse: the capability
then belongs to the file, so every invocation has it, including ones nobody
started deliberately.

**Running the daemon as root.** Rejected by ADR 0001 §2's first clause, and it
would make the agent's authority over the person's files unlimited — the exact
thing the grant model exists to prevent.

**Shipping without the boundary until this is easier.** That is ADR 0015's
guarantee turned off by default, which item 26a already refused for the same
reason: the tempting way to satisfy the letter of a security rule is a component
that satisfies nothing.

## Consequences

- A new crate, `alo-boundaryd`, and a second unit file in the image. Queue item
  28 is unblocked and gains it: the image ships two units, not one.
- `alo-agentd` no longer loads anything. Its `Boundary` becomes an open of a
  pinned map, and its refusal to start changes from *cannot load* to *the
  boundary is not present on this machine*, which is a different and more
  accurate sentence for a person to read.
- `docs/hardware.md`'s kernel requirements are unchanged, but the question *who
  loaded it* now has an answer for whoever certifies a machine.
- The image's `tmpfiles.d` work (ADR 0017) and the pinned map's directory are the
  same kind of thing and should be written together.
