# A machine on two networks is found on each of them

**Date:** 2026-09-15
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 22)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, for a person

A laptop docked at the office is on the wired network and on Wi-Fi at once, and a
GPU box often has two ports. Until now an alo machine was found on only one of its
networks — whichever the kernel picked — and was silently missing from the other,
so a colleague on the other network was left typing an address. Now a machine is
found on **every** network it is on, it says exactly the same thing on each, and a
machine heard on two networks is shown as one machine. A network plugged in after
the machine started is joined as soon as the kernel reports it, with no restart.

## What was built

**`crates/alo-agentd`**

- `src/networks.rs` (new) decides which interfaces discovery is joined on. The
  function is `discovery_networks`, and its only input is what the kernel reports.
  An interface is joined when it is up and running (`IFF_UP` and `IFF_RUNNING`),
  carries multicast and has an IPv4 address, and never when it is loopback (by flag
  or by address). `joined_on` joins each one. A network that refuses the join is one
  sentence in the service log, and the rest are still joined.
- `src/route_messages.rs` (new) reads the kernel's routing messages: the
  `RTM_GETLINK` and `RTM_GETADDR` dumps, turned into `Interface`s. It uses no
  `unsafe`. It refuses a message that runs past what arrived and an error from the
  kernel, and it steps over attributes it does not read. `IFA_LOCAL` is used ahead
  of a point-to-point peer's `IFA_ADDRESS`.
- `src/joining.rs` (new) is `Joining`. It joins the discovery socket on every
  network when the service starts, and holds a routing socket subscribed to
  `RTMGRP_LINK | RTMGRP_IPV4_IFADDR`. Whenever the kernel writes to that socket it
  asks for the interfaces again, forgets networks that have gone and joins new ones.
  `EADDRINUSE` counts as already joined.
- `src/unix.rs` gains `a_route_dump`, `told_when_networks_change` and `emptied`.
  This file stays the only one that names `rustix`.
- `src/wire.rs`: `Wire::bound` now joins through `Joining` instead of
  `join_multicast_v4(group, UNSPECIFIED)`. It also gains
  `Wire::networks_waiting_on`, `Wire::networks_changed` and `Wire::joined`.
  `Wire::bound` no longer fails when the group will not join, because one network
  refusing is a line in the log, not a machine that does not start.
- `src/serving.rs` waits on the routing socket alongside everything else, and calls
  `networks_changed` when it is ready.
- `src/looking.rs`: a look at the discovery group asks on every network at that
  moment, from each network's own address, at the same time. The results go through
  `Around::heard_on_each`. A look at one address (tests, loopback, a proposal's
  source) is one socket and one window, as before.
- `src/listing_workspaces.rs`: a workspace is shown under its host's name only when
  the host answered from the same address on **every** network the workspace was
  heard on.
- `src/opening_workspaces.rs`: tests only. The behaviour is unchanged because the
  merge keeps task 18's refusal true.

**`crates/alo-nearby`**

- `src/heard_on_each.rs` (new) is `Around::heard_on_each`. It merges each network's
  results into one `Found` per identity: the first network gives the address and
  port, and each further network at the same port adds its address to
  `Found::also_at`. A workspace heard from one address on each network becomes one
  `FoundWorkspace` with an address per network. A workspace heard twice on one
  network stays two entries.
- `src/presence.rs`: `Found` gains `also_at` and `addresses()`. `src/workspace.rs`:
  `FoundWorkspace` gains the same, filled only by the merge.
- `src/proposals.rs`: a proposal is checked against every address discovery
  measured for the asking machine.

**Documents:** `docs/contracts/local-network-wire.md` gains *A machine on more than
one network* (additive). `docs/quirks.md` gains an entry on how the Linux kernel
handles multicast membership and source-address routing. The plan marks task 22
done and adds task 23.

## Decisions

1. **An interface that appears after start is picked up from the kernel's
   notification, not at the next start.** The plan left this open. The laptop that
   motivates the task is usually docked *after* it boots. Reading interfaces once
   would leave it missing from the wired network all day until someone restarted a
   service they have never heard of, which is the same silent absence this task
   exists to remove. The cost is one more descriptor in the service's `poll`, with
   no timer: a machine whose networks never change never wakes for them. If the
   routing socket will not open, that is a log line and the machine stays joined on
   the networks it had at start.
2. **Looking asks on each network from a socket bound to that network's address.**
   It does not use `IP_MULTICAST_IF`. Linux sends a multicast datagram out of the
   interface that owns the bound source address (recorded in `docs/quirks.md`), so
   only `std` is needed. Each network gets its own window, and the windows run in
   parallel inside `std::thread::scope`, so a look still takes `WHILE_LOOKING`
   whatever the number of networks.
3. **Networks are read again at every look**, not taken from the wire's joined
   list. It is the same rule as the rest of `crate::looking`: nothing kept, nothing
   stale.
4. **A machine heard on two networks keeps the first network's address as
   `address`**, which is what a pairing dials, and puts the others in `also_at`. A
   pairing is proven by its key (ADR 0031), so the address only decides the route,
   never who is trusted. If the same identity answers on another network with a
   different port, that answer is not merged in. This follows the rule a single
   window already keeps: the first answer for an identity wins.
5. **Workspaces merge only when each network heard exactly one claim.** Task 18
   refuses to open a workspace that answered from two places. Without this rule,
   every workspace on a two-network host would be refused. With a plain merge, two
   claims on one network would be hidden. With this rule, a host on each network is
   one workspace, and two claims on one network still reach the refusal.
6. **`advertised` (task 21) is unchanged.** The same bytes go out on every network,
   so there is nothing new to say, and the answer does not list networks. A list of
   networks would be the first step towards a per-network setting, which ADR 0003
   rules out.
7. **The integration test does not take `alo_bounding::Waited::on_this_kernel()`.**
   Both namespaces sit inside a user namespace the test created, the `veth` pairs
   disappear with them, and nothing on the host network changes. The lock is for
   state that is shared across the kernel, and none is touched here.

Nothing here adds a setting, a grant or an `unsafe` block. No promise in
`docs/features.md` is narrowed, and ADR 0003 is followed throughout.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| Joined on every up, multicast, IPv4 interface | `alo-agentd lib networks::tests::every_network_the_machine_is_on_is_joined` |
| Not on a down interface | `alo-agentd lib networks::tests::an_interface_that_is_down_is_not_joined` |
| Not without multicast | `alo-agentd lib networks::tests::an_interface_without_multicast_is_not_joined` |
| Not without an address | `alo-agentd lib networks::tests::an_interface_without_an_address_is_not_joined` |
| Not on loopback | `alo-agentd lib networks::tests::loopback_is_not_joined` |
| A network that cannot be joined is a log line; the others are joined | `alo-agentd lib networks::tests::a_network_that_cannot_be_joined_is_a_line_and_the_others_are_joined` |
| Interfaces read from what the kernel reports; a cut-short or refused report is refused | `alo-agentd lib route_messages::tests::interfaces_and_their_addresses_are_read_out_of_the_answers`, `...::a_message_cut_short_is_refused`, `...::the_kernel_refusing_is_refused_and_an_acknowledgement_is_not`, `...::the_kernel_on_this_host_reports_its_interfaces` |
| A machine heard on two networks is one machine with an address on each | `alo-nearby lib heard_on_each::tests::a_machine_heard_on_two_networks_is_one_machine_with_an_address_on_each` |
| Two claims on one network stay two | `alo-nearby lib heard_on_each::tests::two_claims_for_one_workspace_on_one_network_are_not_merged_away`, `alo-agentd lib opening_workspaces::tests::two_claims_on_one_of_two_networks_are_still_not_opened` |
| A workspace on a two-network host opens under its name | `alo-agentd lib opening_workspaces::tests::a_workspace_heard_on_each_of_two_networks_is_opened_under_its_name` |
| No door chooses a network | `alo-agentd lib networks::no_door_chooses_a_network::no_request_on_either_door_chooses_a_network` |
| On a real kernel with two networks: heard on one before joining, joined on both after the kernel's notification, one machine and one workspace with an address on each, the same bytes on each network, `advertised` unchanged byte for byte | `alo-agentd a_machine_on_two_networks a_machine_on_two_networks_is_found_on_each_of_them` |

## Verification

Run on Windows 11 through WSL2 Ubuntu (kernel `6.18.33.2-microsoft-standard-WSL2`),
as root, with `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test -p alo-nearby`: every target passed (155 unit tests plus the
  integration tests and doc tests).
- `cargo test -p alo-agentd`: every target passed, including 367 unit tests,
  `a_machine_on_two_networks` (1 passed, 2 inner tests ignored by design and run by
  it), and the existing kernel-bounded tests.

**Not run:** the full workspace suite, which the supervisor runs. **Not measured:**
two physical machines on a real office network with a real switch and Wi-Fi. The
test uses `veth` pairs between namespaces on one host, which exercises the same
kernel paths but is not the certified machine.

## Remaining limitations

- Discovery is IPv4 only. A network with no IPv4 address (two machines on one cable
  with no DHCP, or an IPv6-only office) is still not joined. Task 23 is written for
  it.
- The service log is the only place a network that failed to join is reported. The
  person's `advertised` answer does not list networks, by decision 6.
- A machine whose routing socket will not open follows no later changes, and says so
  once in the log.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "An alo machine on more than one network (a docked laptop on
  wired and Wi-Fi, a box with two ports) is now found on each of them and says the
  same thing on each. A network plugged in after start is joined as soon as the
  kernel reports it. A machine heard on two networks is shown as one."
- **ROADMAP.md / QUEUE.md:** v0.5 *machines find each other with zero configuration*
  gains multi-network discovery. Physical two-machine acceptance is still owed.
  Next in the plan: task 23, discovery over IPv6 link-local.
- **STATE.md:** reference this report.
