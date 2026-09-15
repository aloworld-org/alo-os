# ADR 0044 — A private IPv4 departure is held to the network it was found on

**Status:** accepted, 2026-09-15, as task 25 of
`docs/autonomy/v0-5-the-local-network-plan.md`, which requires the decision to be
written before it is code. It narrows what a paired machine's departure permits,
leaves a provider's exactly as it was, and contradicts no accepted ADR.
**Date:** 2026-09-15
**Context:** [ADR 0003](0003-the-network-is-not-authority.md) (the network is not
authority), [ADR 0007](0007-the-cpu-is-the-default.md) (loopback is not checked),
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md) (the daemon holds no
capability), [ADR 0020](0020-a-question-is-carried-out-inside-the-turns-boundary.md)
(a question is put inside the turn's boundary, its addresses registered first),
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md) (the
kernel-boundary crates are lane A's), [ADR 0041](0041-a-link-local-departure-names-its-interface.md)
(a link-local departure names its interface, and names this as the next question),
tasks 22 and 24 of the local-network plan, `crates/alo-bounding-map`,
`crates/alo-bounding-kernel`, `crates/alo-asking`, `crates/alo-agentd`

## The question in one line

**The studio was found at `192.168.1.20` on the wired network. The Wi-Fi's router
hands out `192.168.1.20` too. Which of the two does a turn shown *the studio
machine* get to reach?**

## What was true before this decision

- Since task 22 a machine on two networks looks on each, and a machine it heard
  is written down at the address it answered from. For IPv4 that is the address
  alone: `alo_nearby::HeardFrom` kept an interface only for a link-local IPv6
  address.
- Since ADR 0041 a departure carries an interface — for the addresses the kernel
  will not dial without one. **No IPv4 address is one of them**, so an IPv4
  departure was an address and a port, and ADR 0041 named that as a limitation,
  not a decision.
- A turn's question to the studio connected from a socket held to nothing, so it
  left by **whatever the route said at the moment it connected**. Two equal routes
  to one range: the first one added wins (measured, `docs/quirks.md`). The boundary
  permitted the address on every network, so a question the person was shown going
  to the studio could reach whoever holds the same address on the other network —
  after a route changed while the question was open, or on a machine whose route
  never pointed at the studio at all.
- Private ranges repeat. `192.168.0.0/24`, `192.168.1.0/24` and `10.0.0.0/24` are
  what most home and office routers hand out, and a laptop docked on a wired
  network with its Wi-Fi up is on two of them.

## The options

Each is costed against the other kind of departure — **a provider**, whose address
is on no one network and which a turn reaches through whatever route this machine
has.

### A — Hold the departure to the interface the machine was found on, and check it against the socket

The daemon writes down the interface each IPv4 network was heard on; a question to
a paired machine found there connects from a socket **held to that interface**
(`SO_BINDTOIFINDEX`); the departure carries the interface; the programme reads the
interface the socket is held to (`skc_bound_dev_if`, which it already reads for
ADR 0041) and permits the departure only on that one.

- *What it holds:* a socket held to an interface leaves by it whatever the route
  says, and a process without `CAP_NET_RAW` cannot move a held socket afterwards
  (measured). `alo-agentd` has no capabilities (ADR 0018).
- *What it costs a provider:* nothing, if a departure held to no interface keeps
  permitting any — which is what every IPv4 departure did.
- *What it costs:* one read of kernel memory per IPv4 destination inside a turn;
  a connector of our own for the corridor, because the HTTP client cannot hold a
  socket; and one road around the socket to close (below).

### B — Decide by the route the kernel would take

The programme asks which interface the route to the address goes out of, and
compares that with the interface the machine was found on.

- *Not buildable where the decision is made.* `bpf_fib_lookup` is offered to XDP
  and traffic-control programmes, not to the LSM hooks that return `EACCES` at
  `connect`. A per-packet `cgroup_skb/egress` programme could drop packets by
  device, but a TCP handshake reaches the other machine before any data does, a
  refusal is a dropped packet rather than an answer, and it is a second enforcement
  mechanism to keep true beside the first.
- *And not what the person was shown.* A route is a fact about the moment; the
  plan's own case is a route that changes while the question is open. Checking the
  route at `connect` would permit a connection that the next route change moves.

### C — Deliberately not

Keep the IPv4 departure an address and a port, and say so.

- Rejected. It is law 1 failing quietly on the most common network shape there is:
  the indicator names *the studio machine* while the boundary permits the address
  on every network. ADR 0003 makes pairing deliberate precisely so that sharing a
  network is not authority; permitting the paired address on a network nobody
  paired on undoes that.

## The decision

**Option A.** In detail:

1. **An IPv4 departure keeps the interface it is given.**
   `alo_bounding_map::keeps_its_interface` is true for every IPv4 address (and, as
   before, for the IPv6 addresses `needs_an_interface`). The interface travels in
   the bits ADR 0041 opened, so the map is not a byte wider, and an IPv4 departure
   held to none has exactly the words it always had.
2. **Held to an interface, it permits that interface; held to none, it permits any.**
   `Departure::permits` is the one rule both halves use. The private ranges are not
   singled out: a public address on an office network and a carrier's shared range
   are the same question, and a departure nobody held still permits what it did —
   so keeping the interface narrows only a departure somebody held. **Held to none
   is IPv4's alone**: a link-local IPv6 departure with no interface still permits
   nothing (ADR 0041).
3. **The programme reads the interface from the socket, for every IPv4
   destination** — a `connect`, a `sendto` and a joined socket's peer — from
   `skc_bound_dev_if`.
4. **A message carrying control messages is held to no interface.** On IPv4 an
   `IP_PKTINFO` control message sends a datagram out of the interface it names,
   past the one the socket is held to, and the kernel does not check they agree
   (measured; IPv6's `IPV6_PKTINFO` is refused by the kernel in the same case). The
   programme does not parse control messages: it reads `msg_controllen` — one more
   offset from the running kernel's type information, the offsets map growing from
   eighteen slots to nineteen so two stay spare — and decides an IPv4 message with
   any as held to none. Such a message reaches only a departure held to none. A
   provider's question is TCP and carries none.
5. **Discovery measures the interface honestly.** Each IPv4 network is asked from a
   socket held to its interface, so an answer counted as heard there arrived there
   even when this machine's own address is the same on both networks, and every
   machine it heard is written down on that interface
   (`alo_nearby::HeardFrom::on_the_network`). The interface is never spelled in the
   address and never crosses the wire.
6. **The corridor dials from a socket held to that interface, and the turn registers
   the departure on it.** `alo_asking::DownTheCorridor::on_the_network`,
   `alo_turn::Bounding::carrying_out_a_departure_on` — which refuses by default and
   **never falls back to registering the address with no interface**, because that
   would permit it on every network. `alo-agentd`'s `ByTheKernel` is the
   implementation.
7. **What a person reads does not change.** The indicator and the record name the
   paired machine by the name its person gave it and print no address or interface,
   as ADR 0041 decided for both families.

## What it costs

- **One read of kernel memory more** for each IPv4 destination inside a turn, and
  one for each message a turn sends. Every process that is not a turn costs what it
  cost.
- **A dependency, `socket2`**, for `SO_BINDTOIFINDEX` and, in a test, a datagram
  carrying `IP_PKTINFO`: `rustix` and `std` have no spelling for either, and a
  `setsockopt` of our own is `unsafe`, which the workspace forbids.
- **A connector and a transport of our own for the corridor** (`alo-asking`'s
  `held_to.rs`), because `ureq`'s TCP transport is private. It serves plain HTTP
  only, which is all the corridor speaks; a provider's question keeps the client's
  own connector.
- **Lane A's crates are edited by the local-network lane**, as for ADR 0041: both
  are the same Claude Code supervisor lane in the same checkout, and the
  kernel-enforcement plan's open tasks touch neither file this changes.

## What would change this decision

- **A turn that runs with `CAP_NET_RAW`.** Such a process could move a connected
  socket to another interface after the check; alo OS runs no turn with it.
- **A kernel that starts checking `IP_PKTINFO` against the held interface**, as
  IPv6 does: the control-message rule would be narrower than it needs to be, and
  could be relaxed to the messages that name another interface.
- **Networks that are the same interface but not the same network** — the Wi-Fi
  moving from one access point's range to another's without the interface going
  down. The interface is held, not the network behind it; a question is short, and
  the next look measures again. Holding to a network identity rather than an
  interface would be a new mechanism and a new ADR.
- **A certified machine using VRF (`l3mdev`) devices**, as ADR 0041 notes.
