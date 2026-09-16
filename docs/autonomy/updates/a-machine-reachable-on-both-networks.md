# A machine on two networks with one private range is reachable on both

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 27)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, in words a person outside this repository can read

An alo machine plugged into two networks — a docked laptop on the wired LAN and
on Wi-Fi, a GPU box with two ports — could only be reached from one of them.
Where both networks hand out the same private range, which is the commonest
office and the commonest home, a colleague on the other network could see the
machine and never connect to it: their computer's request arrived, and the reply
left by the wrong cable. Their person was left typing an address, which is the
step the promise removes.

Now the machine answers its port on **every** network it is on. Two machines on
the network the computer's own routing does not favour can propose a pairing to
each other and pair, and a different machine that happens to have the same
address on the other network is never spoken to. Nothing about what the machine
says on the network changed, and there is still nothing to configure: which
networks it answers on is what it is plugged into.

## Why, and what the kernel really does

Until this change `alo-agentd` bound **one** listener for the port presence
advertises, held to no interface. Measured on this kernel and written into
`docs/quirks.md`:

- **An unheld TCP listener answers every handshake by the route.** For a
  listening socket `ireq->ir_iif` is zero, so `inet_csk_route_req` looks the
  SYN-ACK's route up unconstrained. A connection from the network the route does
  not point at therefore never completes at all.
- **`getsockname` on an accepted connection is the address that was *dialled*,**
  not one belonging to the interface the packet arrived on (Linux's weak host
  model). So once such a connection *can* complete, task 26's reading — the
  interface that owns the accepted socket's local address — would attribute it to
  the wrong network. The two are one task, which is why this one does both.

Three more measurements decided the shape (`docs/quirks.md`, 2026-09-16):

- Two listeners bound to `0.0.0.0` at one port **coexist** when each is held to a
  different interface (`SO_BINDTOIFINDEX` set before `bind`); the kernel's
  bind-conflict check treats a different `sk_bound_dev_if` as a different binding.
- A listener at the same port held to **nothing** beside them is refused
  `EADDRINUSE`. So the held listeners **replace** the old listener-in-both-families
  rather than joining it: that listener bound IPv4 too. An **IPv6-only** listener
  (`IPV6_V6ONLY` set) takes no IPv4 address and coexists with all of them.
- `SO_BINDTOIFINDEX` needs no `CAP_NET_RAW` — measured as an unprivileged user,
  which is the shape `alo-agentd` runs in (ADR 0018). `SO_BINDTODEVICE`, which
  names the interface rather than numbering it, is the one that does.

## The code

New: **`crates/alo-agentd/src/listeners.rs`**

- `Listeners::bound` binds one IPv4 listener per IPv4 network the kernel reports,
  each held to that network's interface, beside one IPv6-only listener held to
  nothing. Every failure on the way — IPv6 missing from the kernel, one network
  refusing the bind, the interfaces unreadable — is a line in the service log;
  listening **nowhere at all** is the one refusal.
- `Listeners::changed` follows the kernel's network notifications exactly as
  `crate::joining`'s joins do, called from `Wire::networks_changed`: a laptop
  docked an hour after it started is otherwise unreachable on the wired network
  all day. Listeners are matched to networks by the interface's **index**, so an
  address changing on an interface leaves the listener and its backlog alone.
- `Listening::of` takes a handle onto each listener for one round of the service
  and nothing is locked while that round waits — which may be for ever. A listener
  a network change took away closes when the round already waiting on it ends.
- A machine whose interfaces cannot be read binds one listener held to nothing,
  says so in the service log, and reads a connection's network the old way. It
  does not begin following the kernel later, because that one unheld listener
  holds the port against every held listener that would replace it.

Changed:

- **`crates/alo-agentd/src/networks.rs`** — `listening_networks`, a pure function
  of what the kernel reports: up and running with an IPv4 address, **loopback
  included** (a connection to `127.0.0.1` is answered today and goes on being
  answered) and **multicast not asked for** (a handshake needs none, so a
  point-to-point tunnel is a network this machine answers on).
- **`crates/alo-agentd/src/arrived_on.rs`** — `what_a_listener_held_to`: the
  network a connection arrived on is the listener that accepted it, because the
  kernel would not have given it that socket otherwise. Loopback stays *its own
  network*. `the_network_it_arrived_on` is kept and is what a listener held to
  nothing reads by.
- **`crates/alo-agentd/src/unix.rs`** — `an_ipv6_only_listener_on`, and
  `ready_and`, which waits on the caller's fixed things and on a list whose length
  is not known until it is asked. `a_listener_in_both_families_on` is gone; the
  held listener is `socket2`'s, in `listeners.rs`, beside `crate::looking`'s.
- **`crates/alo-agentd/src/wire.rs`** — the wire holds `Listeners`;
  `Knocked::arrived` carries the network; `Wire::listening`, `Wire::listened_on`.
- **`crates/alo-agentd/src/serving.rs`** — one round waits on every listener and
  accepts from the first that spoke.
- **`crates/alo-agentd/src/hearing.rs`** — a proposal is measured on
  `Knocked::arrived` rather than on a reading taken there.
- **`crates/alo-nearby`** — `dialling::put` dials **held to the network the other
  machine was heard on**; `Found::on_the_network` and
  `Waiting::where_the_other_was_heard` carry it; `crossing::propose` and
  `crossing::confirm` pass it. `socket2` joins this crate's dependencies for the
  one socket option `std` cannot set, reached from `dialling.rs` alone.

## Decisions I made, and why

1. **The held listeners replace the dual-stack listener rather than joining it.**
   Forced by the kernel: an unheld listener beside them is `EADDRINUSE`, and the
   old listener bound IPv4. Hence IPv4-per-network plus one IPv6-only listener.
2. **Loopback is one of the networks listened on, and is measured as *its own
   network*.** The plan asks for loopback to keep working. Reporting it as an
   interface to hold to would have written `%1` onto every measurement a test on
   one host makes; *its own network* is what `arrived_on` already documents for
   loopback, and it keeps every existing test honest rather than merely passing.
3. **Multicast is not required of a listening network**, unlike a discovery join.
   A join without multicast carries nothing; a handshake does not care.
4. **Listeners are matched to networks by interface index, not by the whole
   network.** A DHCP renew or a second address would otherwise tear a listener
   down and drop the connections waiting in its backlog.
5. **The listeners are held behind a mutex that only the service's thread takes,
   and a round holds a reference-counted handle rather than the lock.** The wire is
   shared with the thread that answers discovery (`crate::answering_discovery`), so
   it must stay `Sync`; a round that polled while holding the lock would hold it
   for as long as the machine is quiet, which is for ever.
6. **A machine that cannot read its own interfaces falls back to one unheld
   listener** rather than refusing to start or listening nowhere. It is what every
   alo machine did before this change, the log says so, and a service that will not
   start is worse than one reachable where the route points.
7. **`alo-nearby` dials a proposal and a confirmation held to the network the
   machine was heard on.** This was not spelled in task 27's acceptance, and the
   acceptance cannot be met without it: the asked machine's confirmation goes to
   the address it measured, and on two networks with one private range an unheld
   dial reaches whoever the route reaches — so two machines could find each other,
   propose, and never pair. It is not a new decision, it is ADR 0044 applied to the
   pairing wire, which `alo-asking`'s corridor already obeys for a question.

## What is **not** done, and why it is task 28

**A discovery answer still leaves by the route.** The socket the machine answers
*who is here* on is one per family, held to no network, and the answer is a
unicast datagram to the asking machine. On a machine on two networks carrying one
private range it therefore goes to whichever of the two the route picks. A
machine on the other network can now *reach* this one's port and still not *find*
it.

This is outside task 27's acceptance, which names the listeners and the reading
and nothing about discovery's own sockets. It is measured, written into
`docs/quirks.md` and into the contract, and written into the plan as **task 28 —
A discovery answer leaves on the network the question arrived on**, with its own
acceptance including the end-to-end this one could not yet write.

Because of it, the end-to-end test here keeps discovery's answers on the cable
with an `ip rule ipproto udp` and a table of its own — the same technique task
26's fixture used, and for the same reason: so that what this test measures is the
handshake, which is what task 27 changed, and not a second thing nothing in this
change touched. The main routing table still points at the other network, which is
the machine the task is about, and every other thing reception does reaches the
studio because the **code** holds it: the look is held per network (task 22), the
measurement is held to the network the connection arrived on (task 26), and the
confirmation is held to the network the studio was heard on (this change).

## Acceptance, and where each part is tested

| Acceptance | Where |
| --- | --- |
| One IPv4 listener per IPv4 network, held to its interface, beside one IPv6-only listener; loopback among them | `alo-agentd` `listeners::tests::the_port_is_listened_on_once_per_network_and_loopback_is_one`, `networks::tests::every_network_the_machine_is_on_is_listened_on_including_loopback` |
| Measured: two held listeners coexist and an unheld one beside them is refused `EADDRINUSE` | `alo-agentd` `listeners::tests::an_unheld_listener_beside_the_held_ones_is_refused`; `docs/quirks.md` |
| The listeners follow the kernel's network events | `alo-agentd` `listeners::tests::a_network_that_goes_is_let_go_of_and_one_that_comes_is_listened_on` |
| A network that will not bind is a line in the service log and the others are still bound | `alo-agentd` `listeners::tests::a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound` |
| `accept_one` answers with the interface of the listener that accepted, and `hearing` measures a proposal on that | `alo-agentd` `arrived_on::tests::a_held_listener_says_its_own_network`, `…::a_listener_held_to_loopback_says_its_own_network`, `…::a_listener_on_no_interface_at_all_says_nothing`; the end-to-end below asserts `Knocked::arrived` is the cable |
| The old reading is kept where there is no held listener | `alo-agentd` `listeners::tests::a_listener_handed_in_is_the_only_one`, `wire::tests::a_message_is_read_once_and_a_non_message_is_carried_as_one` |
| End to end: the machine on the network the route does not point at proposes and is paired, and a machine at the same address on the other network is asked nothing | `alo-agentd` `a_machine_reachable_on_both_networks::tests::a_machine_on_two_networks_with_one_private_range_is_reachable_on_both` |

### The refusal paths, beside the legitimate ones

- An **unheld** listener is never reached from the network the route does not
  point at — asserted in the end-to-end test beside the held one that is, so the
  contrast is the measurement rather than a claim.
- An unheld listener bound beside the held ones is refused `EADDRINUSE`.
- Accepting where nothing was ready is refused rather than blocking the service
  (`listeners::tests::accepting_where_nothing_was_ready_is_refused`).
- A listener on an interface the kernel numbers zero says *nothing could be said*,
  so a connection it accepted is measured nowhere.
- A dial held to a network that is not there reaches nobody rather than going by
  the route (`alo-nearby` `dialling::tests::what_is_put_held_to_a_network_that_is_not_there_reaches_nobody`).
- Somebody else at the same private address on the other network is asked nothing
  and connected to never — witnessed by the machine that would have answered,
  which counts both.

## Picked up again on 2026-09-16, and what the third worker changed

This task was built, passed every gate, and was committed locally by the
supervisor (`e6515ee`), which was then interrupted while rebasing onto an
advanced `main`; the commit was never published and the plan still said *ready*.
The third worker applied that commit onto the current `main` (it applied without
conflict), reviewed it against the acceptance line by line, and found **one
criterion with no test behind it**: *a network that will not bind is a line in
the service log and the others are still bound*. The table above had pointed at
the code that writes the line, and every listener test panicked on any line.

Writing that test measured something new. The obvious fixture — a network whose
interface index nobody has — **binds**: `SO_BINDTOIFINDEX` does not look the
device up. So an interface that goes between the kernel's report and the bind
leaves a listener nothing reaches, let go of at the next notification, rather
than a line. That is harmless and now written down (`docs/quirks.md`, the
`listeners.rs` module documentation); what really refuses a network's bind is
somebody else holding the port on it, and
`listeners::tests::a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound`
arranges exactly that on loopback: the service still starts, says one line naming
the network and why, keeps every other network's listener, and listens on
loopback — and answers there — once the port is free and the kernel next says a
network changed. It also found that `::1` is not configured on this WSL host, so
the test proves the port is answered over IPv4 loopback rather than assuming IPv6.

No other code changed. `a-machine-reachable-on-both-networks-gate-refusal.md` is
the second worker's diagnosis of the earlier host refusal and is published
unchanged beside this report.

## Verification

Run from `/mnt/c/dev/alo-os-claude` in WSL Ubuntu (kernel
`6.18.33.2-microsoft-standard-WSL2`) with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`.

- `cargo fmt --all` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings, on the
  tree rebased onto current `main`.
- `cargo test -p alo-agentd` — passed.
- `cargo test -p alo-nearby` — passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd -p alo-nearby --no-deps` — clean.

Each was run again by the third worker after the added test, on the rebased tree.

The workspace suite is the supervisor's, per the task's instructions.

Not run here: anything on the certified machine. This is network-namespace work
on one host, and the fixtures make and destroy their own `veth` cables inside a
user namespace, so they take no kernel-global state and do not need
`alo_bounding::Waited::on_this_kernel()`.

## Proposed updates to the shared documents

`CHANGELOG.md`, under the local network:

> A machine plugged into two networks now answers its port on both of them, so a
> colleague on either can reach it — including where two routers hand out the
> same addresses, which is most offices and most homes. A machine at the same
> address on the other network is never spoken to by mistake. Nothing about what
> the machine says on the network changed, and there is still nothing to
> configure.

`docs/autonomy/QUEUE.md` and `STATE.md`: task 27 of the v0.5 local-network plan is
done; task 28 — *A discovery answer leaves on the network the question arrived on*
— is written into the plan and is ready.

`ROADMAP.md`: no gate is ticked by this on its own.
