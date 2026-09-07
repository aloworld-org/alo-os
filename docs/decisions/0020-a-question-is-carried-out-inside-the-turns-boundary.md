# ADR 0020 — A question is carried out inside the turn's boundary

**Status:** accepted — approved by the repository owner on 2026-09-07, recorded
in `docs/autonomy/updates/network-request-boundary-approval.md`
**Date:** 2026-09-07
**Context:** [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md) (the grant
is enforced by the kernel), [ADR 0015](0015-the-kernel-learns-what-a-turn-is.md)
(the kernel learns what a turn is), [ADR 0007](0007-the-cpu-is-the-default.md)
(the CPU is the default), [ADR 0008](0008-where-inference-happens.md),
`crates/alo-egress`, `crates/alo-asking`, `crates/alo-turn`,
`docs/autonomy/updates/end-to-end-network-enforcement.md`

## The decision in one line

A question a turn puts to a provider is **carried out inside that turn's
kernel boundary**, with the addresses it may reach registered before the
connection is made and withdrawn when the request is over — so *nothing leaves
silently* becomes something the machine refuses rather than something the daemon
remembers to do.

## What is actually wrong today

The boundary exists and the request does not go through it.

`socket_connect` refuses a bound turn any destination nobody was shown. What
makes a connection a *bound turn's* is the control group it is made from, and
the only thing that puts a thread into one is `Bounding::carrying_out` — which
wraps the file verbs and nothing else. `Turning::asking` reaches `alo-asking`
directly, on a thread in no control group, so the programme sees every provider
request as *not a turn*: the answer that allows everything.

`crates/alo-turn/tests/whether_a_question_runs_inside_the_boundary.rs` measures
it. One turn, one file verb, one question, and a `Bounding` that counts
executions carried out inside it: the count is one, and it was the file verb.

So law 1's *nothing leaves silently* is, at the network, exactly what ADR 0013
says an audit log is — an honest program's account of itself.

## The decision

**1. A question runs inside a boundary of its own.** `Bounding` gains a way to
carry out a network request, beside the one that carries out a file verb. It is
**additive and fails safe**: the new method has a default that refuses, so an
implementation written before this ADR cannot make a network request rather than
making one unbounded. `alo-turn`'s own rule — no library here ships a `Bounding`
that bounds nothing — is why the default is a refusal and not a passthrough.

**2. Resolution is separated from connection, and DNS is defined rather than
excepted.** The provider's host is resolved **before** the boundary is entered,
by the daemon, which is not a turn. The addresses that come back are registered,
the boundary is entered, and the request is made with a resolver that returns
*those addresses and no others* and asks no name server anything.

So there is **no DNS inside the boundary and no exception for it**. A turn does
not resolve; it connects to what was resolved for it and registered. That is the
narrowest answer available and it is why the question *what may a bounded turn
ask a name server* does not have to be answered at all.

**3. The connection is request-scoped.** The HTTP client is built for one
request and dropped with it, so its connection pool cannot outlive the
permission. A connection that survived would be authority that outlived what the
person was shown, which is the thing the destination binding exists to prevent.

**4. Original-hostname TLS is preserved.** Only the *resolver* is replaced. The
connector is the client's own, the URI keeps the provider's hostname, and
certificate verification is against that hostname exactly as before. Connecting
to an address while verifying a name is precisely what a resolver is for, and
substituting an address into the URL would have been the wrong way to do this.

**5. A turn that cannot be bounded does not ask.** ADR 0015's rule, unchanged:
if the boundary cannot be established, the question is not put. Failing open
here would be worse than not having built it.

## What this does not decide

The approval is scoped and so is this.

- **No new agent capability and no wider grant.** A turn may reach the addresses
  its own question resolved to, for the length of that question. Nothing else
  changes about what an agent may do.
- **The provider-and-region policy stays in userspace.** `alo-egress` decides;
  the kernel refuses what was not shown. ADR 0020 does not make the kernel a
  policy engine, and `crates/alo-bounding`'s own documentation argues why it
  cannot be one.
- **Loopback is still not checked**, so a model on this machine is untouched
  (ADR 0007) — and the loopback-proxy hole `docs/quirks.md` records stays open.
  Closing it needs enforcement that is not turn-scoped, which is a different
  decision.
- **The LSM still writes nothing down.** *Decides and forgets* is untouched, and
  kernel-sourced audit recording remains its own unmade decision.

## Consequences

- `Bounding` grows one method, defaulted to a refusal. Every implementation in
  this repository that should bound a request overrides it; anything that does
  not, refuses — which is the safe direction and needs no version bump.
- `alo-asking` accepts the addresses a request may use, and builds its client
  per request. Its redirect refusal (`max_redirects(0)`) and its key handling are
  unchanged.
- `alo-turn` resolves before it bounds, and the resolution happens where it
  always did — outside any turn.
- **What is still not covered is named rather than implied**: a socket already
  open or inherited, UDP sent without a connection, and the loopback proxy.
  `docs/autonomy/updates/end-to-end-network-enforcement.md` carries the
  assessment and this ADR does not claim otherwise.

## Alternatives rejected

**Bound the whole daemon.** Rejected: it would apply a turn's grant to every
process the person's own service runs, including the errands that are
deliberately not turns.

**Let the kernel enforce the provider-and-region policy.** Rejected, and argued
at length in `crates/alo-bounding/src/lib.rs`: a provider is a name resolved
through DNS and a region is a fact about a company, and a kernel-side
approximation would be a second policy disagreeing with the first unpredictably.

**Substitute the resolved address into the URL.** Rejected: it would connect to
the right place and verify the wrong name, which trades one guarantee for
another.

**Give a bounded turn a blanket exception for DNS.** Rejected on the approval's
own terms. Resolving before entering costs nothing and leaves no exception to be
widened later.
