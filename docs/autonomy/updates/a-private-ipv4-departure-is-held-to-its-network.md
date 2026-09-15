# A private IPv4 address on two networks is still one destination the kernel bounds

**Date:** 2026-09-15
**Workstream:** the local network (v0.5), task 25 of
`docs/autonomy/v0-5-the-local-network-plan.md`; edits the kernel-boundary crates
owned by the kernel-enforcement lane (ADR 0028), as task 24 did
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner
**Status:** ready for integration

## What changed, for a person

Most home and office routers hand out the same few address ranges, so a laptop
docked on a wired network with its Wi-Fi up can see `192.168.1.20` on both — and
those are two different machines. Until now, when an agent asked the paired studio
machine at `192.168.1.20` a question, the kernel boundary around the turn allowed
that address **on every network**, and the connection left by whichever network
the routing table preferred at that moment. The indicator said *the studio
machine*; the question could reach somebody else.

Now the studio is remembered on the network it answered discovery on, the question
connects from a socket held to that network's interface, and the boundary allows
the address only there. The same address on the other network is refused with
`EACCES`, and so is any attempt to send by the routing table instead. A question to
a hosted provider is unchanged. What the indicator and the record say is unchanged.

## Decision

[ADR 0044](../../decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md),
written before the code as the plan requires. Three options:

- **A — hold the departure to the interface the machine was found on, and check it
  against the socket's held interface.** Chosen. A socket held to an interface
  leaves by it whatever the route says, and an unprivileged process cannot move it
  afterwards (measured); `alo-agentd` holds no capability (ADR 0018).
- **B — decide by the route the kernel would take.** Rejected: `bpf_fib_lookup` is
  not available to the LSM hooks that answer `EACCES` at `connect`, a per-packet
  egress programme lets the TCP handshake reach the other machine first, and a route
  is a fact about a moment the plan itself says can change mid-question.
- **C — deliberately not.** Rejected: law 1 failing quietly on the commonest network
  shape there is.

What was decided inside option A, and why:

1. **Every IPv4 departure keeps the interface it is given; held to none, it permits
   any interface.** A provider's departure is therefore byte-for-byte and
   behaviour-for-behaviour unchanged. The private ranges are not singled out: a
   public address on an office network and a carrier's shared range are the same
   question, and keeping the interface narrows only a departure somebody held.
2. **A message carrying control messages is decided as held to no interface.** The
   probe before the code found that on IPv4 an `IP_PKTINFO` control message sends a
   datagram out of the interface it names, past the one the socket is held to, while
   IPv6 refuses the same with `EINVAL`. Rather than parse control messages in the
   programme, it reads `msg_controllen` and treats any as held to none — so such a
   message reaches only a provider-style departure. A mutation run proved the rule is
   what refuses those datagrams.
3. **Discovery asks each IPv4 network from a socket held to its interface.** Without
   it, where this machine's own address is also the same on both networks, an answer
   could be counted as heard on the wrong one. The interface is written down beside
   the address (`HeardFrom::on_the_network`) and never spelled in it or sent.
4. **The corridor dials through a connector of its own** (`alo-asking/src/held_to.rs`),
   because `ureq`'s TCP transport is private and the client cannot hold a socket.
   It serves the corridor's plain HTTP only; a provider keeps the client's connector.
5. **`Bounding::carrying_out_a_departure_on` refuses by default and never falls
   back** to registering the address with no interface, which would permit it on
   every network. Existing implementations elsewhere in the workspace compile
   unchanged; only the corridor, which only `alo-agentd` builds, calls it.
6. **IPv6 is untouched**: a link-local address keeps ADR 0041's rule, and the corridor
   is held only for an IPv4 address.
7. **`socket2` joins the workspace dependencies** for `SO_BINDTOIFINDEX` (and, in a
   test, a datagram with `IP_PKTINFO`): `std` and `rustix` 1.1.4 cannot set it, and a
   `setsockopt` of our own would be `unsafe`, which is forbidden. Already in the
   local registry at 0.6.5; no source patch.

**Coordination (ADR 0028):** `alo-bounding`, `alo-bounding-map` and
`alo-bounding-kernel` are lane A's. As with task 24, both workstreams are this
supervisor's lane in this checkout, and the kernel-enforcement plan's open tasks
touch none of `departure.rs`, `field.rs`, `departing.rs`, `fields.rs` or the fixture.

## Source

- `crates/alo-bounding-map/src/departure.rs` — `keeps_its_interface`,
  `Departure::permits`, `Departures::holds` through it; tests for held, unheld and
  the map round trip.
- `crates/alo-bounding-map/src/field.rs`, `lib.rs` — `Field::MessageControlLength`.
- `crates/alo-bounding-kernel/src/departing.rs`, `kernel.rs` — the interface read for
  IPv4, the control-message rule, nineteen offset slots.
- `crates/alo-bounding/src/fields.rs`, `testing.rs`, `imposing.rs`, `Cargo.toml`;
  `crates/alo-bounding/tests/a_private_ipv4_departure_is_held_to_its_network.rs` (new).
- `crates/alo-nearby/src/heard_from.rs` — `on_the_network`, `interface`.
- `crates/alo-asking/src/held_to.rs` (new), `corridor.rs`, `openai.rs`, `lib.rs`,
  `Cargo.toml`.
- `crates/alo-turn/src/bounding.rs`, `asking.rs`, `testing.rs`, `down_the_corridor.rs`.
- `crates/alo-agentd/src/looking.rs`, `corridor.rs`, `bounding.rs`, `testing.rs`,
  `lib.rs`, `Cargo.toml`;
  `crates/alo-agentd/src/a_paired_machine_on_two_networks_with_one_address.rs` (new).
- `Cargo.toml`, `Cargo.lock` — `socket2`.
- `docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md`
  (new), `docs/contracts/local-network-wire.md` (one additive line), `docs/quirks.md`
  (one entry), the plan (task 25 done, task 26 written).

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The decision is in an ADR before it is code, with the options and what each costs a provider | ADR 0044 |
| A question from a turn to a paired machine at a private IPv4 address is reached on the network it was found on | `alo-agentd` `a_paired_machine_on_two_networks_with_one_address::tests::a_question_to_a_paired_machine_at_a_private_address_reaches_only_the_network_it_was_found_on` |
| …and refused with `EACCES` on another network carrying the same address, on a real kernel under `Waited::on_this_kernel()`, two interfaces in a namespace of their own, a listener that would have answered on either | `alo-bounding` `a_private_ipv4_departure_is_held_to_its_network::a_private_ipv4_departure_reaches_its_network_and_not_the_same_address_on_another` and `…::shown_on_the_other_network_the_cable_is_the_one_refused` |
| A provider's departure is unchanged | `alo-bounding` `a_private_ipv4_departure_is_held_to_its_network::a_departure_held_to_no_interface_is_reached_as_it_always_was`; `alo-bounding-map` `departure::tests::an_ipv4_departure_held_to_no_interface_permits_what_it_always_did` |
| What the indicator and the record name is unchanged in shape | the `alo-agentd` end-to-end test above (step 2: `Happened::Left { destination: PairedMachine { machine } }`, indicator quiet); `alo-turn` `down_the_corridor::tests::a_machine_found_on_one_network_is_bounded_and_dialled_on_that_network` |
| Refusal paths | default deny on both networks (`…::a_turn_shown_nothing_reaches_neither_network`); a boundary that cannot hold an interface runs nothing (`alo-turn` `bounding::tests::a_boundary_that_cannot_hold_a_request_to_an_interface_refuses_it_and_runs_nothing`, `down_the_corridor::tests::a_boundary_that_cannot_hold_a_departure_to_an_interface_asks_nothing`); a held departure refuses an unheld socket and another interface (`alo-bounding-map` `departure::tests::an_ipv4_departure_held_to_an_interface_is_permitted_there_and_nowhere_else`, `alo-agentd` `bounding::tests::a_private_ipv4_address_is_registered_on_the_interface_it_was_found_on`) |

Two mutation runs, reverted before the gates: disabling the control-message rule in
`departing.rs` made both `IP_PKTINFO` attempts reach; removing the corridor's hold
made the end-to-end question go by the route to the other machine at the same
address, and the studio counted nothing.

## Verification

Executed on Windows 11 with WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`
with the BPF LSM, `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`,
from `/mnt/c/dev/alo-os-claude`, all in the foreground:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- in `crates/alo-bounding-kernel`: `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-bounding-map -p alo-nearby -p alo-asking -p alo-turn -p alo-agentd -p alo-bounding` — clean.
- `cargo test -p alo-bounding-map` — 34 + 5 doctests passed.
- `cargo test -p alo-nearby` — all targets passed (163 unit).
- `cargo test -p alo-asking` — all targets passed (112 unit).
- `cargo test -p alo-turn` — all targets passed (113 unit).
- `cargo test -p alo-agentd` — all targets passed (385 unit, 6 ignored children).
- `cargo test -p alo-bounding` — every target passed, exit 0, including tasks 24's
  and 25's real-kernel files.
- `cargo test -p alo-boundaryd` — passed (it loads the programme with the new
  offset).

Not run here, by instruction: the whole workspace suite (the supervisor runs it).
Not measured: two physical machines on two physical networks; every network here
is `veth` in namespaces.

## Numbering

The decision was first written as ADR 0042. The software lane published its own
ADR 0042 (installing an application) and then 0043 (the terminal) while this task
was being gated, and `alo-citing`'s
`every_decision_this_repository_points_at_exists` refused two files claiming one
number. This decision is the later of the two to reach `main`, so it is the one
renumbered: it is **ADR 0044**, and every citation this task wrote — rustdoc,
comments, `Cargo.toml` notes, the plan, the wire contract, `docs/quirks.md` and this
report — follows the rename. The installing ADR and the citations of it are
untouched.

## Remaining limitations

- A turn running with `CAP_NET_RAW` could move a connected socket to another
  interface; alo OS runs none.
- The interface is held, not the network behind it: a Wi-Fi interface that moves to
  another access point's same-numbered range without going down is the same
  interface. Named in ADR 0044.
- A connection that **arrives** from a private IPv4 address is still measured from a
  socket held to nothing — written as task 26.

## Proposed shared-document updates

- **CHANGELOG.md:** "A question to a paired machine found at a private IPv4 address
  now reaches only the network it was found on: the same address on another network
  — which is often another machine — is refused by the kernel, and the question
  never leaves by whichever network the routing table prefers. Questions to hosted
  providers are unchanged."
- **ROADMAP.md / QUEUE.md:** v0.5 local network — task 25 done; task 26 (a proposal
  from a private IPv4 address is measured on the network it arrived on) ready.
- **STATE.md:** reference this report.
