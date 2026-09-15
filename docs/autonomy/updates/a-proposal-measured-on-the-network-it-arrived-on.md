# A proposal from a private IPv4 address is measured on the network it arrived on

**Date:** 2026-09-16
**Workstream:** the local network (v0.5), task 26 of
`docs/autonomy/v0-5-the-local-network-plan.md`
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner
**Status:** ready for integration.

## What changed, for a person

A machine plugged into two networks that hand out the same private range — a
desk with a cable and Wi-Fi, which is most offices and most homes — has two
different machines at `192.168.1.20`. When one of them asked to pair, alo OS
checked *is there really a machine at that address* by asking down whichever
network the machine happened to be routing through at that second. It could ask
the other network, and be answered by the other machine. Now the question is
asked on the network the request arrived on, and nowhere else; and when the
machine cannot tell which network a request arrived on, it asks nobody and
refuses the request rather than guessing.

## What changed, in the code

- **`crates/alo-agentd/src/arrived_on.rs`** (new). One responsibility: which of
  this machine's networks a connection arrived on. `the_network_it_arrived_on`
  reads the accepted connection's local address with `getsockname` and matches it
  against the interfaces the kernel reports **at that moment** (nothing is kept,
  for `crate::looking`'s reason). `the_network_of` is the rule with no kernel in
  it: an IPv6 or loopback address names its own network; an IPv4 address exactly
  one reported interface owns is that interface's network; an address two
  interfaces own, or none does, is `ArrivedOn::NothingCouldSay`.
- **`crates/alo-agentd/src/looking.rs`**: `found_at` takes the `ArrivedOn` and is
  what the measuring socket is **held to** — `ArrivedOn::TheNetwork(i)` asks from
  a socket held with `SO_BINDTOIFINDEX` to `i` (`held_to`, already there from
  task 25) and writes everything found down on `i`
  (`alo_nearby::HeardFrom::on_the_network`), so a question to the machine it
  found is held there in turn; `ArrivedOn::ItsOwnNetwork` asks as before;
  `ArrivedOn::NothingCouldSay` asks nothing at all and answers *nothing found*.
- **`crates/alo-agentd/src/hearing.rs`**: reads the arriving network off the
  connection for a proposal, and for nothing else — a confirmation needs no
  measurement and still gets none.
- **`crates/alo-agentd/src/lib.rs`**: the module and its three items.
- **`docs/quirks.md`**: a new entry for what the kernel really does with an
  unheld listener and with `getsockname` on an accepted connection.
- **`docs/autonomy/v0-5-the-local-network-plan.md`**: task 26 marked done, and
  task 27 written, which is the finding below.

Nothing crosses the wire that did not before; `alo-nearby`, `alo-bounding*` and
`alo-shell` are untouched, and so is `image/`.

## The decision, and why

The task left the reading open. Three were available:

1. **The interface that owns the accepting socket's local address** — chosen.
   `getsockname` on the accepted connection is measured by the kernel, is there
   for every connection with nothing set beforehand, and changes nothing about
   how `crate::wire` reads a message. Where this machine's own address is the
   same on both networks, two interfaces own it, the reading is ambiguous, and
   the honest answer is `NothingCouldSay`: measured nowhere, refused as not
   found, never measured by the route.
2. **`IP_PKTINFO` read back from the accepted socket** — rejected. It would name
   the interface even where the address is shared, but reading it means either
   `recvmsg` inside `crate::wire`'s single reader — the one file whose single
   reader is why three wires can share a port — or an `IP_PKTOPTIONS`
   `getsockopt` that neither `rustix` nor `socket2` spells, which would mean an
   `unsafe` block. The laws forbid the second outright, and the first is a change
   to how a message is read that a later task can make on its own merits.
3. **The route to the source address** — rejected, because it is exactly what
   ADR 0044 refused: the route is not what the person was shown, and it changes
   underneath an open connection.

Loopback is `ItsOwnNetwork` rather than an interface to hold to: ADR 0020 leaves
loopback the only unchecked destination, there is one loopback network, and a
connection that arrived at `127.0.0.1` arrived from this machine. IPv6 is
`ItsOwnNetwork` too — a link-local address already carries the interface it was
heard on (ADR 0041) and a global one names its own network.

## What the kernel actually does (measured, 2026-09-16)

Probed before the code, on `6.18.33.2-microsoft-standard-WSL2`, with two `veth`
cables in a user namespace: `10.66.0.1/24` on one and `10.66.0.3/24` on the
other, a machine at `10.66.0.2` at each far end, one `TcpListener` on
`0.0.0.0:7610` held to nothing.

- **A connection completes only from the network the route points at.** The far
  end of the routed cable connected to both `10.66.0.1` and `10.66.0.3`; the far
  end of the other cable timed out on both. For an unheld listener `ireq->ir_iif`
  is zero, so `inet_csk_route_req` looks the SYN-ACK's route up unconstrained and
  the reply leaves by the route, to the wrong machine.
- **`getsockname` on an accepted connection is the address that was dialled**,
  not one belonging to the interface the packet arrived on — Linux's weak host
  model.
- **An established connection follows the main routing table when it changes**:
  moving the route while a connection was open sent its next reply out of the
  other cable. The end-to-end test therefore moves only where *questions* go,
  with an `ip rule ipproto udp` and a table of its own.

The first two are why the chosen reading is exact today rather than approximate:
while the wire listens on one socket held to nothing, the only connections that
complete arrived on the route's network. All three are in `docs/quirks.md`.

## The finding, and the task it became

The first measurement is a defect this task did not fix and could not fix inside
its own scope: **on two networks carrying one private range, a machine on the
network the route does not point at cannot reach this one's port at all** — the
studio on the cable proposes and its handshake never completes, because the reply
left by the Wi-Fi. The fix is one held IPv4 listener per network in
`crate::wire`, which also supplies a better reading (the interface of the
listener that accepted, immune to the weak host model). That is written into the
plan as **task 27, "A machine on two networks with one private range is
reachable on both"**, with the measurements and the acceptance it needs. It was
not folded into this task because it replaces the wire's listener, the poll set
that waits on it and the joining that follows network changes, none of which this
task's acceptance names.

## Verification

Run from `/mnt/c/dev/alo-os-claude` in WSL Ubuntu as root, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`. Every command
below was executed and passed; nothing here is predicted.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, zero warnings |
| `cargo test -p alo-agentd` | pass |
| `cargo doc -p alo-agentd --no-deps` with `RUSTDOCFLAGS=-D warnings` | clean |

The workspace suite was **not** run here; the supervisor runs it
(`docs/autonomy/LOOP.md`). No physical acceptance is claimed: the end-to-end test
runs on a real kernel in namespaces on this machine, not on the certified
workstation, and what two physical machines on a cable measure is still owed.

### One line per acceptance criterion

- *The reading is decided in the crate that measures, and written up with the
  reason* — `crates/alo-agentd/src/arrived_on.rs` module documentation, the
  section above, and the plan's own entry for task 26.
- *A proposal arriving over IPv4 is measured from a socket held to the interface
  it arrived on, and what is found is written down on that interface* —
  `alo_agentd::looking::tests::what_was_measured_on_a_network_is_written_down_on_it`
  and, over cables, step 2 of
  `a_proposal_measured_on_the_network_it_arrived_on::tests::a_proposal_from_a_private_address_is_measured_on_the_network_it_arrived_on`.
- *A connection whose arriving interface cannot be read is measured nowhere and
  refused as not found, never measured by the route* —
  `alo_agentd::arrived_on::tests::an_address_on_two_interfaces_says_nothing`,
  `…::an_address_no_interface_owns_says_nothing` and
  `alo_agentd::looking::tests::a_connection_whose_network_could_not_be_read_is_measured_nowhere`
  (which asserts that the machine that would have answered was not asked), and
  step 5 of the end-to-end test.
- *Two machines at the same private address on two networks, one of them
  proposing, are told apart end to end* —
  `a_proposal_measured_on_the_network_it_arrived_on::tests::a_proposal_from_a_private_address_is_measured_on_the_network_it_arrived_on`:
  both far ends answer discovery **as the studio**, the studio proposes over the
  cable, the proposal goes through `crate::hearing` and is judged against the
  studio while somebody else is asked nothing, and the same measurement held to
  nothing reaches somebody else instead — which is the failure, witnessed.

## Limitations

- The end-to-end test runs in network namespaces on one host. Two physical
  machines on two cables are still owed.
- `ArrivedOn::NothingCouldSay` refuses a proposal on a machine whose own address
  is identical on two networks. That is deliberate and tested; closing it needs
  the interface from the listener, which is task 27.
- `alo_nearby::Waiting` still keeps where the other machine answers as a bare
  `SocketAddr`, so a **confirmation** sent back to a machine at a private IPv4
  address is still dialled by the route. Out of this task's scope and worth a
  task of its own after 27, which changes the same call sites.

## Proposed shared-document updates

Only the integration owner edits these (`docs/autonomy/SHARED_MAIN.md`).

- **CHANGELOG.md**, under the local network: "A machine on two networks that hand
  out the same private range now checks a request to pair on the network it
  arrived on, so the machine that asked is the machine that is checked — and
  refuses the request rather than guessing when it cannot tell which network it
  arrived on."
- **QUEUE.md / STATE.md:** task 26 of the v0.5 local-network plan is done;
  task 27, "A machine on two networks with one private range is reachable on
  both", is written in the plan and ready, and carries a defect this task
  measured. Reference this report.
- **ROADMAP.md:** nothing moves. No v0.5 gate is ticked by this.
