# ADR 0041 — A link-local departure names its interface

**Status:** accepted, 2026-09-15, as task 24 of
`docs/autonomy/v0-5-the-local-network-plan.md`, which requires that a change to
the shape of a departure be decided before it is built. The decision narrows
what one departure permits and widens nothing; it contradicts no accepted ADR.
**Date:** 2026-09-15
**Context:** [ADR 0003](0003-the-network-is-not-authority.md) (the network is not
authority), [ADR 0007](0007-the-cpu-is-the-default.md) (loopback is the ordinary
case and is not checked), [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0020](0020-a-question-is-carried-out-inside-the-turns-boundary.md) (a
question is carried out inside the turn's boundary, with the addresses it may
reach registered first), [ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(the kernel-boundary crates are lane A's), task 23 of the local-network plan,
`crates/alo-bounding-map`, `crates/alo-bounding-kernel`, `crates/alo-agentd`

## The question in one line

**A paired machine found only over IPv6 is dialled at `fe80::…%3`. Is the `%3`
part of the destination the kernel permits a turn to reach, or not?**

## What was true before this decision

- `alo_bounding_map::Departure` is an address, a port and a family. It has had
  an IPv6 shape since the boundary was first given destinations, and every test
  of it used IPv4.
- Task 23 made two machines with no IPv4 address between them find each other
  and pair. The address a pairing and a question dial is a **scoped link-local
  address**: `fe80::20%3`, the interface the other machine was heard on beside
  the address it was heard at.
- `fe80::/10` is on every link. Each interface gives itself an address in it
  with nobody configuring anything, so **the same `fe80::20` can be a different
  machine on each interface** — the studio on the cable, and whatever is on the
  Wi-Fi. The kernel will not dial a link-local address without an interface for
  exactly that reason.
- So a departure without the interface, registered for the studio on the cable,
  permitted a bound turn to reach `fe80::20` **on every link the machine is on**.
  One sentence shown, several machines permitted — which is the thing a
  departure exists to stop, on exactly the network the release just made work.

## The options

### A — Match the address and port only, and say so

No shape change. The kernel keeps comparing what it compares today.

Rejected. It widens a link-local departure to every interface, which the plan's
own constraint forbids in as many words (*no departure is widened to a prefix,
an interface or "the local network"*), and it is law 1 failing quietly: the
indicator names *the studio machine* while the boundary permits a machine on a
different link.

### B — Refuse link-local departures altogether

No shape change, and nothing ambiguous is ever permitted.

Rejected. A network with no IPv4 address is where task 23's promise lives, and
a turn on that network could then never ask the paired machine its person chose.
It trades a guarantee for a feature by removing the feature.

### C — Carry the interface in the departure, for the addresses that need one

A departure gains the interface an address is on, **kept only where the address
needs one** — the kernel's own `__ipv6_addr_needs_scope_id`: IPv6 link-local
unicast and interface- or link-scoped multicast — and zero for every other
address. The kernel half reads the interface from the same places the kernel
itself takes it from, in the kernel's order, and a link-local destination with
no readable interface is a destination nothing holds.

## The decision

**Option C.** In detail:

1. **The shape.** `Departure` gains `interface: u32`, packed into the high
   thirty-two bits of the word that already carried the family and the port.
   Those bits were zero, so the map value is not a byte wider, `WORDS` is
   unchanged, and every IPv4 destination and every destination written before
   this reads back exactly as it did.
2. **One door.** `Departure::on(family, address, port, interface)` keeps the
   interface only where `needs_an_interface(family, address)` says so.
   `Departure::of` is `on` with no interface. The daemon hands in whatever scope
   a resolved `SocketAddrV6` carried; the programme hands in whatever scope a
   `sockaddr_in6` carried; both get the same value for the same destination, and
   a scope on a global address — which the kernel ignores — cannot make one
   destination two.
3. **Where the programme reads the interface.** For a link-local address named
   in a `connect` or a `sendto`: the `sockaddr_in6`'s scope when the caller's
   length is the full twenty-eight bytes and the scope is not zero; otherwise
   the interface the socket is held to (`skc_bound_dev_if`), which a scoped
   `connect`, a bind to a scoped address and `SO_BINDTODEVICE` all set. For the
   peer of a joined socket: the interface it is held to, which its `connect`
   set. Two more offsets are read out of the running kernel's type information
   for it — `sock.__sk_common.skc_bound_dev_if` and `msghdr.msg_namelen` — and
   the offsets map grows from sixteen slots to eighteen, so two stay spare for
   `the_boundary_decides_and_forgets` to hold at zero.
4. **No interface is nowhere.** A link-local destination whose interface reads as
   zero from both places is refused whatever was shown: the kernel would choose
   its interface from `IPV6_UNICAST_IF`, `IPV6_MULTICAST_IF` or a control
   message, none of which a departure describes. `Departures::holds` answers
   false for such a destination even if the entry holds the same nothing, and
   the daemon refuses to register one — naming the address in the service log —
   before a boundary is entered.
5. **What a person reads does not change.** The indicator and the record name a
   paired machine by the name its person gave it, in both families, and never
   print an address: `%3` is not something a person can read, and a departure
   that looked different by family would read as two places.

## What it costs

- **Two reads of kernel memory more**, and only for a turn reaching a link-local
  address that names no scope. Every other destination, and every process that
  is not a turn, costs what it cost.
- **Lane A's crates are edited by the local-network lane.** ADR 0028 partitions
  them to the kernel-enforcement workstream, and the plan requires the edit be
  coordinated. Both workstreams are the same Claude Code supervisor lane in the
  same checkout; the kernel-enforcement plan's open tasks touch neither file
  this changes. The report for task 24 records it.
- **IPv4 is unchanged, and that is a limitation named rather than solved.** A
  private IPv4 address (`192.168.1.20`) can also be a different machine on each
  of two networks, and a departure to one permits it on whichever the route or a
  `SO_BINDTODEVICE` chooses. The kernel does not require an interface to dial
  one, so there is no scope to read, and deciding by the route a packet would
  take is a different mechanism. The next task in the local-network plan is to
  decide it.

## What would change this decision

- A kernel that reads a link-local scope from somewhere this programme does not
  (a new socket option honoured ahead of `skc_bound_dev_if`): the programme's
  order would have to follow it, and `docs/quirks.md` would say where reality
  moved.
- A certified machine using VRF (`l3mdev`) devices, where the kernel compares a
  scope against the master device rather than the interface itself. alo OS
  certifies no such configuration today.
