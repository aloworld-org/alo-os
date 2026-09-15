# A paired machine reached over IPv6 link-local is a departure the kernel bounds and the indicator shows

**Date:** 2026-09-15
**Workstream:** the local network (v0.5), task 24 of
`docs/autonomy/v0-5-the-local-network-plan.md`; edits the kernel-boundary crates
owned by the kernel-enforcement lane (ADR 0028)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository owner
**Status:** ready for integration

## What changed, for a person

Since task 23 two machines on a network with no IPv4 address find each other and
pair, and one dials the other at an address like `fe80::20%3`: an address that
exists on every network cable, plus the number of the one it was heard on. When an
agent asks such a paired machine a question, that question now leaves exactly the
way a question over IPv4 does: the turn's kernel boundary allows it on the
interface the machine was found on and refuses the same address on any other with
`EACCES`, the egress indicator shows it, and the record keeps it under the name the
person gave the machine.

Before this change, a departure registered for "the studio on the cable" would have
let a bound turn reach `fe80::20` on **every** network the machine is on. That is
one place shown and several permitted.

## Decision

[ADR 0041](../../decisions/0041-a-link-local-departure-names-its-interface.md),
written before the code because the plan requires that a change to a departure's
shape be decided first. Three options were weighed: matching the address and port
only (rejected, since it widens a departure to every link), refusing link-local
departures entirely (rejected, since it removes task 23's feature), and **carrying
the interface for the addresses that need one**, which was chosen:

- `Departure` gains `interface: u32`, packed into bits of the map word that were
  zero. The map is no wider, and IPv4 values are byte-for-byte unchanged.
- `Departure::on` keeps an interface only where `needs_an_interface` says the
  address needs one. This is the kernel's `__ipv6_addr_needs_scope_id`: link-local
  unicast, plus interface- and link-scoped multicast. A scope on a global address
  is dropped, so it cannot turn one destination into two.
- A link-local destination with no interface names no network.
  `Departures::holds` refuses it whatever was shown, and the daemon refuses to
  register one before a boundary is entered, naming the address in the service log.
- The BPF programme takes the interface from the same places the kernel does, in
  the same order: `sin6_scope_id` when the caller's length is 28 bytes and the
  scope is non-zero, else the socket's `skc_bound_dev_if`. That order covers
  `connect` and `sendto`. For a joined socket's peer it uses `skc_bound_dev_if`.
- The indicator and the record name the paired machine by its given name in both
  families and print no address. The end-to-end test checks this.

Smaller choices made here:

- The network hooks moved out of `alo-bounding-kernel/src/deciding.rs` into a new
  `departing.rs`. ADR 0041 changed what a destination is and touched nothing a file
  hook reads, so the file had gained a second reason to change (law 4).
- `alo-turn` parses a scoped IPv6 literal itself instead of passing it to the
  system resolver, whose handling of `%` is its own.
- The IPv4 counterpart is a named limitation rather than a decision. A private
  IPv4 address on two networks is also two machines, but the kernel needs no
  interface to dial it. The next task, task 25 below, is written into the plan to
  decide it.

## Where

- `crates/alo-bounding-map/src/departure.rs`, `field.rs`, `lib.rs`: the shape,
  `needs_an_interface`, and two new fields (`SockBoundInterface`, `MessageNameLength`).
- `crates/alo-bounding-kernel/src/departing.rs` (new), `deciding.rs`, `kernel.rs`,
  `main.rs`: the decision in the programme. The offsets map grows from 16 to 18
  slots so that two slots stay spare.
- `crates/alo-bounding/src/fields.rs`, `imposing.rs`, `testing.rs`: the two offsets
  are found and width-checked, and the BTF fixture carries them.
- `crates/alo-agentd/src/bounding.rs`: a scoped address is registered on its
  interface, and an unscoped link-local address is refused.
- `crates/alo-turn/src/asking.rs`: a scoped literal is registered exactly as written.
- `docs/contracts/local-network-wire.md`: one additive line.
  `docs/quirks.md`: what the kernel does.

## Acceptance criteria and their tests

| Criterion | Test |
|---|---|
| Reached when registered, `EACCES` when not, real kernel, under `Waited::on_this_kernel()` | `alo-bounding` `a_link_local_departure_names_its_interface`: `a_link_local_departure_reaches_its_interface_and_not_the_same_address_on_another`, `a_turn_shown_nothing_reaches_no_link_local_destination` |
| Scope decided in the shape's crate, tested both ways | `alo-bounding-map`: `departure::tests::a_link_local_departure_is_held_on_its_interface_and_no_other`, `…a_link_local_address_with_no_interface_is_held_by_nothing`; on a kernel: `shown_on_the_other_interface_the_cable_is_the_one_refused`, `a_link_local_address_with_no_interface_is_refused_however_it_was_shown` |
| Daemon and turn register the interface, and refuse an unscoped address | `alo-agentd` `bounding::tests::a_link_local_address_is_registered_on_its_interface`, `…with_no_interface_is_refused_before_a_boundary`; `alo-turn` `asking::tests::a_paired_machine_at_a_link_local_address_is_registered_with_its_interface` |
| Indicator and record name it as for IPv4; a machine found only over IPv6 leaves nothing the indicator missed, end to end | `alo-agentd` `a_paired_machine_over_link_local::tests::a_question_to_a_paired_machine_found_only_over_ipv6_leaves_nothing_the_indicator_did_not_show` |

During development, a mutation that dropped the interface from the departure turned
every refusal on the other interface into a reach. `docs/quirks.md` records this.

## Verification

All commands ran on WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, as root,
from `/mnt/c/dev/alo-os-claude` with the checkout's own `CARGO_TARGET_DIR`:

- `cargo fmt --all` and `cargo fmt --check` in `crates/alo-bounding-kernel`: clean.
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel`: exit 0.
- `cargo test -p alo-bounding-map`: all passed.
- `cargo test -p alo-turn`: all passed.
- `cargo test -p alo-agentd`: all passed, including the end-to-end test.
- `cargo test -p alo-bounding`: all passed. The new test file was also run on its
  own.
- The full workspace suite was not run here. The supervisor runs it.

Not measured: two physical machines on a cable. As with task 23, that is owed to
certified hardware.

## Remaining limitations

- IPv4 departures carry no interface. See task 25.
- A kernel feature that takes a link-local scope from somewhere this programme does
  not read, such as VRF/`l3mdev` masters, would need the order revisited. ADR 0041
  names both cases.

## Proposed shared-document updates

- **CHANGELOG:** "A question to a paired machine found only over IPv6 is held by the
  kernel to the network the machine was found on, and is shown and recorded like any
  other."
- **ROADMAP / QUEUE:** local-network task 24 is done. Task 25 (a private IPv4
  address on two networks) is ready.
- **STATE:** reference this report and ADR 0041.
