# Two machines with no IPv4 address between them still find each other

**Date:** 2026-09-15
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 23)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, for a person

Two alo machines joined by one cable with no router between them, an office whose
router is down for the afternoon, or a network run on IPv6 alone have no IPv4
address in common. Until now the machines were strangers there, and the person was
left typing an address. Now they find each other over the IPv6 address every
network interface gives itself, pair exactly as they do anywhere else, and say
exactly the same thing about themselves as they do over IPv4. A machine heard both
ways on one network is shown once.

While building the end-to-end test, a second problem came to light that affected
every network, not only these: **two real alo machines could not pair at all**. The
machine proposing was busy waiting for the other machine's answer at the very moment
that machine asked whether the proposer existed, so the proposal was always refused.
That is fixed too.

## What was built

**`crates/alo-nearby`**

- `src/heard_from.rs` (new): `HeardFrom`, an address measured off a packet or a
  connection. For a link-local IPv6 address (`fe80::/10`) it keeps the interface
  the address was heard on (the scope id). Without that, a link-local address names
  no network and cannot be dialled. For any other address it drops the scope, and
  it reads an IPv4-mapped address back as the IPv4 address it is. `at(port)` gives
  the scoped socket address.
- `src/reading.rs`: `a_machine_heard` and `a_workspace_heard` read an answer
  together with the full socket address it came from. An answer from a link-local
  address with no scope is refused as `NotNearby::NamesNoNetwork`. `a_machine_in` and
  `a_workspace_in` keep their signatures and go through the same check.
- `src/presence.rs`, `src/workspace.rs`: `Found::address`, `Found::also_at` and the
  matching fields of `FoundWorkspace` are now `HeardFrom`, so `where_it_answers()`
  is scoped. `Found::seen` keeps its signature, and `Found::heard` is added.
  `HeardFrom` compares equal to a bare `IpAddr` with the same address.
- `src/looking.rs`: adds `THE_IPV6_ADDRESS` (`ff02::fb`). `Looking` reads answers
  with their scope.
- `src/refusing.rs`: adds `NamesNoNetwork`, counted as a stranger's packet.
- `src/heard_on_each.rs`: the documentation says how the families merge, and one
  test is added. The merge rule is unchanged, so a machine heard in both families
  is one machine.

**`crates/alo-agentd`**

- `src/networks.rs`: `Interface` gains `ipv6`, `Network::address` is an `IpAddr`,
  and two functions are added:
  - `link_local_networks` picks every interface that is up and running, carries
    multicast, has a link-local address and is not loopback.
  - `every_discovery_network` returns every IPv4 network first, then the IPv6 ones.
- `src/route_messages.rs`: the address dump asks for every family. An IPv6 address
  that is still tentative, or whose duplicate address detection failed, is skipped.
  The flags come from `IFA_FLAGS` when the kernel sends it, and from the header
  otherwise.
- `src/joining.rs`: `Joining` takes `DiscoverySockets` (an IPv4 socket and an
  optional IPv6 one) and joins `ff02::fb` on each link-local interface. If the
  machine has no IPv6 socket, IPv6 networks are not considered.
- `src/unix.rs` gains four things. It is still the only file that names `rustix`.
  - `a_shared_ipv6_datagram_socket_on`, with `IPV6_V6ONLY` set.
  - `a_listener_in_both_families_on`, with `IPV6_V6ONLY` cleared.
  - A subscription to `RTMGRP_IPV6_IFADDR`.
  - A `#[cfg(test)]` helper that shuts a socket for sending.
- `src/wire.rs`: `Wire::bound` binds the port in both families. If IPv6 is not
  there, it logs a line and binds IPv4 only. It also opens the IPv6 discovery
  socket; if that fails, it logs a line and carries on. `answering_over_ipv6_on`
  adds a second `Answering` with the same presence and the same workspace.
  `hosting` applies to both answerers. Also added: `discovery_ipv6_waiting_on` and
  `answer_discovery_over_ipv6`. `Knocked::from` is now a `HeardFrom`.
- `src/looking.rs`: each link-local network is asked from its own scoped address at
  `ff02::fb` on its interface. `found_at` asks in the family and scope the
  connection came from, and asks nothing for an unscoped link-local address.
- `src/hearing.rs`: passes the scoped address to `found_at`, and the bare address to
  the two wires' `Arrived::carried` (unchanged).
- `src/answering_discovery.rs` (new): discovery, in both families, is answered on a
  thread beside the service for as long as it runs; see decision 1.
  `src/serving.rs` runs its rounds inside `beside` and waits on the thread's
  failure signal instead of the discovery socket. A failing discovery socket still
  stops the service with `NotServed::TheWire`.
- `src/two_machines_with_no_ipv4.rs` (new, test only): the end-to-end test.
- `tests/a_machine_on_two_networks.rs` (task 22's test): its `veth` networks now
  also carry link-local IPv6, which is now answered too, so its assertions count the
  IPv4 addresses and networks. What it proves is unchanged.

**Documents:**

- `docs/contracts/local-network-wire.md` gains *A network with no IPv4 address*
  (additive): the IPv6 group, the same-bytes rule, the scope rule, the port in both
  families, and which address is dialled.
- `docs/quirks.md` gains an entry on link-local scope, tentative addresses, and the
  WSL2 kernel having IPv6 off in its first namespace.
- The plan marks task 23 done and adds task 24.

## Decisions

1. **Discovery is answered on its own thread, because two daemons otherwise cannot
   pair.** The deadlock went like this:
   - The person's `pair` request is handled inside one round of `Serving`, and
     `crossing::propose` dials the other machine and waits up to ten seconds for
     its reply.
   - The asked machine judges the proposal against a measurement made at that
     moment: `found_at` asks the proposer over discovery and waits two seconds.
   - The proposer answered discovery only between rounds, so it heard that question
     only after the reply had come back as a refusal (`NotFromWhereItWasFound`).
   Every earlier pairing test put a hand-made responder on one side, so no test had
   ever run two `Serving`s against each other. I found it by reading the code path
   while designing the end-to-end test. I did not re-run the old code to watch it
   fail.

   Options I weighed:
   - Answer pending discovery questions while dialling. That puts discovery inside
     `alo-nearby`'s dialling code.
   - Hand the proposal to a later round. That changes a request/reply door into an
     asynchronous one.
   - **A thread that does nothing but answer discovery.** This is the one I chose.
     It touches no grant, record, connection or turn. It says the same bytes
     whatever the loop is doing, which ADR 0003's *presence never says what a
     machine is doing* already required. It ends through a socket-pair hangup, not
     a timer.
   `serving.rs`'s module documentation no longer claims "no threads" without
   qualification: it names this one thread and says why it exists.
2. **When both families answered, the IPv4 address is written first, and a pairing
   dials it.** The plan left this open.
   - Machines that already had IPv4 keep dialling what they dialled before IPv6 was
     asked.
   - An IPv4 address needs no interface to be dialled.
   - It is the address people already recognise.
   - The link-local address stays in `also_at`, and a proposal from it is still
     judged correctly, because proposals compare against every measured address.
   Where only IPv6 answered, the scoped link-local address is dialled. The order is
   a property of `every_discovery_network`, which is tested.
3. **Link-local only, not every IPv6 address.** `ff02::fb` is a link-local group,
   so a question to it leaves on one interface. The interface's own link-local
   address is the one that always exists without configuration. A global IPv6
   address on the same interface adds nothing to discovery.
4. **The scope lives in `HeardFrom`, not in a new field beside `Found::address`.**
   A separate `scope` field would have had to be kept in step with `also_at` by
   hand. Also, `fe80::1` on two interfaces is two addresses, and only a type that
   holds both parts can say so. To keep the change small for other crates,
   `HeardFrom` compares equal to an `IpAddr`, and `Found::seen`,
   `reading::a_machine_in` and `a_workspace_in` keep their signatures. No crate
   outside these two needed a change.
5. **One listener in both families, not a second listener.** The service waits on
   one port descriptor, and a second one would have meant a second door through
   `Serving`. `IPV6_V6ONLY` is cleared explicitly so the behaviour does not depend
   on `net.ipv6.bindv6only`. IPv4 peers are turned back into IPv4 addresses, so
   `Proposals::arrived` and `confirmation_arrived` compare as before. If IPv6 is not
   in the kernel, the port is bound over IPv4 and a line says so.
6. **Tentative addresses are skipped, and the service joins when the kernel says
   the check finished.** Nothing can be bound to a tentative address. Joining it and
   then failing every look would be a silent partial state. Subscribing to
   `RTMGRP_IPV6_IFADDR` costs nothing while nothing changes.
7. **Still no setting (ADR 0003).** `no_request_on_either_door_chooses_a_network`
   now also sends `ipv6 off`, `discover-on family`, `advertised family`, and a `pair`
   carrying a `family`, and every one is refused on both doors.
8. **The end-to-end test is a unit-test module, not a file in `tests/`.** A real
   pairing through the person's door needs `Serving` with a person's door, which
   needs the crate's test fixtures (`Pretending`, `NothingIsBounded`). Those are
   `pub(crate)` and cannot be reached from `tests/`. It uses the same namespace
   technique as task 22's test and takes no kernel-wide lock, for the same reason.

No gate was weakened. No grant was widened, no `unsafe` block was added, no promise
in `docs/features.md` was narrowed, and no ADR is contradicted.

## Acceptance criteria and evidence

| Criterion | Test (workspace `.`) |
|---|---|
| `alo-nearby` asks and answers over IPv6 as over IPv4, with the same closed advertisement | `alo-nearby asked_and_answered_over_ipv6 asked_and_answered_over_ipv6_as_over_ipv4`, `alo-nearby lib reading::tests::an_answer_over_ipv6_is_read_as_one_over_ipv4_with_its_interface` |
| A packet saying more than presence is refused whichever family carried it | `alo-nearby lib reading::tests::saying_more_than_presence_is_refused_over_ipv6_as_over_ipv4` (and the namespace test's second part) |
| `ff02::fb` joined on every up, multicast, link-local, non-loopback interface | `alo-agentd lib networks::tests::every_interface_with_a_link_local_address_is_joined_over_ipv6` |
| Not when down / without multicast / without a link-local address / on loopback | `alo-agentd lib networks::tests::an_interface_that_is_down_is_not_joined_over_ipv6`, `...::an_interface_without_multicast_is_not_joined_over_ipv6`, `...::an_interface_without_a_link_local_address_is_not_joined_over_ipv6`, `...::loopback_is_not_joined_over_ipv6` |
| What the kernel reports, read; an unusable IPv6 address is not one | `alo-agentd lib route_messages::tests::an_ipv6_address_is_read_once_the_kernel_has_finished_checking_it` |
| An interface that cannot be joined is a log line and never a stopped service | `alo-agentd lib networks::tests::a_network_over_ipv6_that_cannot_be_joined_is_a_line_and_the_others_are_joined` |
| A found machine's link-local address carries its interface, because without it the address names no network | `alo-nearby lib heard_from::tests::a_link_local_address_keeps_the_interface_it_was_heard_on`, `...::a_link_local_address_with_no_interface_names_no_network`, `...::an_address_that_names_its_own_network_keeps_no_scope`, `alo-nearby lib reading::tests::an_answer_from_a_link_local_address_with_no_interface_is_refused`, `alo-agentd lib looking::tests::a_link_local_address_with_no_interface_is_looked_for_nowhere` |
| A machine heard over IPv4 and IPv6 on one network is one machine with an address in each family | `alo-nearby lib heard_on_each::tests::a_machine_heard_in_both_families_is_one_machine_with_an_address_in_each`, `alo-agentd lib networks::tests::an_interface_with_both_families_is_two_networks_ipv4_first` |
| The same identity, port and workspace answer in both families, byte for byte; two machines with no IPv4 between them find each other and pair through task 12's request end to end | `alo-agentd lib two_machines_with_no_ipv4::tests::two_machines_with_no_ipv4_address_between_them_find_each_other_and_pair` |
| Two daemons can pair: discovery is answered while the service is busy, and a failing discovery socket still stops it | `alo-agentd lib answering_discovery::tests::discovery_is_answered_while_the_service_is_busy`, `...::a_discovery_socket_that_fails_wakes_the_service_with_why` |
| No setting: no IPv6 on/off and no family chosen on either door | `alo-agentd lib networks::no_door_chooses_a_network::no_request_on_either_door_chooses_a_network` |
| Task 22 still holds with IPv6 on its networks | `alo-agentd a_machine_on_two_networks a_machine_on_two_networks_is_found_on_each_of_them` |

## Verification

Run on Windows 11 through WSL2 Ubuntu (kernel `6.18.33.2-microsoft-standard-WSL2`),
as root, with `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-nearby -p alo-agentd --no-deps`:
  clean. Fixing it corrected four intra-doc links in `heard_on_each.rs`. With
  `--document-private-items`, five warnings remain, all in files this task did not
  touch (`deliberating.rs`, `pairing.rs`, `answering.rs`).
- `cargo test -p alo-nearby`: every target passed, including 162 unit tests,
  `asked_and_answered_over_ipv6` (1 passed, the inner test ignored by design) and
  the doc tests.
- `cargo test -p alo-agentd`: every target passed:
  - 379 unit tests, with 2 inner namespace tests ignored by design.
  - `a_machine_on_two_networks`, `a_question_is_bounded_by_the_kernel` (5),
    `a_turn_is_bounded_by_the_kernel`, `a_turn_is_refused_when_the_boundary_is_gone`,
    `two_doors_on_one_socket` and `what_a_machine_says_about_itself`.
- Each of the 22 evidence tests above, run on its own with
  `-- --exact --include-ignored`: 1 passed each.

**Not run:** the full workspace test suite, which the supervisor runs. **Not
measured:** two physical machines on a real cable or a real IPv6-only office
network. The tests use `veth` pairs between user-namespaced network namespaces on
one host. That exercises the same kernel paths, but it is not the certified machine.

## Remaining limitations

- A departure to a scoped link-local address has not been shown to be bounded by the
  kernel and named by the indicator. The boundary's `Departure` is an address and a
  port, with no interface. Task 24 is written for this.
- A machine that cannot open an IPv6 socket (IPv6 compiled out) is discovered over
  IPv4 only and says so once in the service log. The person's `advertised` answer
  does not list families, for the same reason task 22 kept networks out of it.
- mDNS on IPv6 conventionally uses a hop limit of 255; questions here leave with
  the kernel's default multicast hop limit of 1. That is correct for a link-local
  group and is noted in case another responder turns out to expect 255.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "Two alo machines with no IPv4 address between them — one cable
  and no router, an office whose router is down, an IPv6-only network — now find
  each other over IPv6 link-local and pair as they do anywhere else, saying exactly
  the same thing in both families. A machine heard over both is shown once and
  reached over IPv4. Fixed: two alo machines could not pair with each other, because
  the proposing machine could not answer the other machine's check while it waited
  for that machine's reply; discovery is now answered beside the service."
- **ROADMAP.md / QUEUE.md:** v0.5 *machines find each other with zero configuration*
  gains discovery on networks with no IPv4 address. Physical two-machine acceptance
  is still owed. Next in the plan: task 24, a link-local departure bounded by the
  kernel and shown by the indicator.
- **STATE.md:** reference this report.
