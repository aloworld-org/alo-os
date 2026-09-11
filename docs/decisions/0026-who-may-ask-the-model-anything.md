# ADR 0026 — Who on this machine may ask the model anything

**Status:** **PROPOSED.** The recommendation is to accept the machine as it is
today and to close the gap when the thing that makes it matter arrives, which is
a decision about ordering rather than about code — and ordering is the owner's.
Nothing in this repository waits on it: `alo-modeld.service` ships either way,
and what changes if this is accepted is a line in `docs/features.md`'s v0.5
section rather than anything on a v0.01 machine.
**Date:** 2026-09-11
**Proposed by:** the v0.01 delivery workstream, from task 32 of
`docs/autonomy/v0-01-delivery-plan.md`
**Context:** [ADR 0001](0001-the-capability-model.md) §2 and §5 (the agent is a
login of its own, and never holds authority the person does not),
[ADR 0005](0005-applications-are-sandboxed-and-ask.md) (applications are
sandboxed and ask), [ADR 0006](0006-the-pinned-model-runtime.md) (the pinned
model runtime), [ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
(one privileged component, and what it is trusted with),
[ADR 0019](0019-a-runtime-is-found-not-configured.md) (a runtime is found at one
address, not configured), [ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md)
(the default is what a machine arrives able to do), `image/`,
`crates/alo-image`, `crates/alo-models`, `crates/alo-bounding`

## The question in one line

**The machine now serves a model. Which processes on it may ask that model
anything — and is *the unit file* the place that can be decided?**

## What is true today, measured rather than remembered

- **The model is served by `alo-modeld.service`**, added in the same change as
  this decision. It runs as `alo-model` (60991), a login and a group of its own
  that is neither the person's nor the agent's; it holds no capability and both
  lines say so; its store is the directory the weights landed in; and its IP
  access list is `IPAddressAllow=localhost` with `IPAddressDeny=any`, so it
  reaches nothing off this machine — enforced in the kernel on its own control
  group rather than written in a comment.
- **It answers on `http://127.0.0.1:11434`**, which is the one address
  `crates/alo-models` knocks at (ADR 0019) and the only one it has.
- **A loopback TCP port has no owner and no mode.** This is the finding. Every
  door alo OS has decided who may knock at so far has been a Unix socket:
  `/run/alo/<uid>/…` is 0750 and the agent's group (ADR 0017),
  `/run/alo-sessiond` is 0750 and the greeter's (ADR 0024). Both are decided by
  a `Group=` line and a mode, because the filesystem carries an owner and a mode
  for a socket and the kernel checks them on `connect(2)`. **A TCP socket
  carries neither.** No directive in any systemd unit restricts which local
  uids may connect to a listening port; `IPAddressAllow=`/`IPAddressDeny=` filter
  by *address*, and every local process connects from the same one. So
  `Group=alo-model` on that unit says who **answers**. It does not say, and
  cannot say, who may **ask**.
- **The pinned runtime offers no Unix socket.** `OLLAMA_HOST` is a host and a
  port; the server listens with `net.Listen("tcp", …)`. ADR 0011 forbids
  patching a rented engine to add one, so this is not a line we may write.
- **Today the machine's logins are `root`, `alo` (the person, 1000),
  `alo-agent` (60989), `alo-greeter` (60990) and `alo-model` (60991).** All five
  can reach the port. Four of them are processes we ship and one is the person.

## What is actually at stake, said plainly

Not much, today, and a great deal at v0.5. That distinction is the whole of this
decision and it is worth being exact about, because *a port anybody can reach*
reads alarming and the alarm points at the wrong year.

**It is not a grant boundary.** No agent verb touches the model runtime; the ten
verbs in `docs/contracts/agent-verbs.md` are six about files and four about
applications, and none of them is *ask a model*. Reaching the port does not
reach a file, a window or a device, and it never produces an approval or a
record entry, because nothing it does is a change to the machine. ADR 0001's
model is not in this path and is not weakened by it.

**It is not an egress.** The runtime reaches nothing off this machine, enforced
above. A local process asking it a question causes no packet to leave, which is
law 1's measurement unaffected.

**What it is: the owner's compute, and later somebody else's code.** Asking a
model is the most expensive thing a machine can do without asking anybody, and
at v0.01 the only things that could ask are four processes we wrote and the
person who owns the machine. At v0.5, ADR 0005 puts *sandboxed applications* on
this system, running as the person, and `docs/features.md` puts the portal in
front of them — which is exactly the promise that a sandboxed application asks
before it reaches anything. **An application that can open a TCP connection to
127.0.0.1 has reached the machine's model without asking, around the portal, in
the one way the sandbox does not cover.** That is the day this matters, and it
is the day it stops being theoretical.

## Options

### A. Leave it: every local process may ask, and say so

Nothing changes. The unit is what it is; `docs/features.md` gains no promise it
does not keep, and this ADR is where a reader finds out that the port is open to
the machine and why that was acceptable while it was.

- **Costs:** nothing at v0.01. At v0.5 it is the sandbox's hole, named above,
  and it would arrive as an application quietly using the model somebody paid
  for — which looks like nothing at all until a report of it, because there is no
  record and no indicator for a thing that never left.
- **Honest about:** this is the state the machine is in whether or not anybody
  writes it down. The only difference an ADR makes is whether the next reader
  inherits the reasoning or discovers the port.

### B. A door of ours in front of the runtime

Put something we wrote between `alo-models` and the runtime: a Unix socket at
`/run/alo-modeld/door`, 0750 and a group, forwarding to the loopback port, with
the runtime bound somewhere nothing else looks.

- **Costs:** a new component in the path of **every question the machine ever
  answers**, written by us, carrying somebody's words, which is the one kind of
  code ADR 0001 §7 is most careful about. It changes ADR 0019's address, so
  `crates/alo-models` changes with it. And it does not close the hole it was
  built for: the runtime is still listening on a TCP port, on the same machine,
  and an application that can find 11434 can find whatever we moved it to —
  unless the runtime is *also* confined, at which point the confinement is the
  answer and the proxy is not.
- **Verdict:** rejected. It is a component in the hottest path in the system
  bought in exchange for an obstacle rather than a boundary.

### C. A network namespace shared with what may ask

`PrivateNetwork=yes` on the model service and `JoinsNamespaceOf=` on whatever
may ask it: the loopback the runtime listens on exists only inside that
namespace, so nothing outside it can reach the port at all. Stronger than a
group, and entirely upstream systemd.

- **Costs:** whoever joins that namespace has **only** that namespace. The
  process that would join is `alo-agentd.service`, and it is the one process on
  this machine that talks to the outside world — hosted providers (ADR 0008),
  which are half of what alo OS offers and the half the egress indicator exists
  for. A private namespace with no route out would silently turn every hosted
  provider into unreachable, which is the worst available way to keep a
  promise: by breaking a different one quietly.
- **Verdict:** rejected on those grounds, and worth recording as rejected,
  because it is the first thing a reader reaches for.

### D. The kernel already on this machine says who may connect

`alo-boundaryd` loads a BPF LSM programme at boot and holds a map the daemon
writes (ADR 0018, ADR 0013, ADR 0015). The kernel's own `socket_connect` hook is
where *which process may reach which socket* is answerable, and it is the hook
family this repository already builds against in `crates/alo-bounding`. The rule
would be small and readable: a connection to the model service's port is
permitted from the person's own session and refused from anything inside a
sandbox.

- **Costs:** the one privileged component gains a second programme, and ADR
  0018's argument is that it is acceptable *because of how little it is trusted
  with*. That is a real price and it must be paid deliberately, in an ADR of its
  own, at the time. It is also not free to get right: the thing being identified
  is a cgroup rather than a uid, since a sandboxed application runs as the
  person.
- **And it is not this task's to build.** It needs the sandbox to exist to be
  testable against anything, and the sandbox is v0.5.

## Recommendation

**A now, D when the sandbox arrives** — and the second half is the part being
decided, because the first half is merely what is true.

At v0.01 every process that can reach the port is one alo OS ships or the person
who owns the machine, none of them can reach anything through it that the
capability model governs, and no packet leaves. Buying a boundary today costs
either a component in the hottest path (B) or the agent's egress (C), and D
cannot be measured against anything until there is something to keep out.

What this ADR asks the owner for is that **the portal work at v0.5 carries the
local model's port with it** — not as a note in a report, as a line in
`docs/features.md`'s v0.5 section under the sandbox: *a sandboxed application
reaches the local model through the portal or not at all.* The promise belongs
beside the sandbox because that is where it becomes keepable and where somebody
will be looking; written anywhere else it is a finding nobody inherits, which is
the failure `crates/alo-reconciling` and `crates/alo-citing` were both built
after.

`docs/features.md` is not changed by this ADR, because only the owner moves the
definition.

## Consequences if accepted

- `image/usr/lib/systemd/system/alo-modeld.service` is unchanged, and this file
  is what a reader finds when they ask why `Group=alo-model` does not gate the
  port.
- `crates/alo-image` continues to check what a unit can decide — the login, the
  group, the store, the address, the silence — and deliberately not who may
  connect, which it says in as many words beside the check.
- v0.5's sandbox work inherits one more thing to cover, named rather than
  discovered.

## Consequences if rejected

If the owner wants the port closed at v0.01, it is D, and it is its own ADR,
its own task and the second programme on the one privileged component. It is
buildable — the loader, the map and the tests for all three already exist — and
it is not a day's work, and the thing it would be keeping out does not ship
until v0.5.
