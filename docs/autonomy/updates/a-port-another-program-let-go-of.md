# A port another program let go of on one network is listened on there again

**Date:** 2026-09-17
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 35)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository's owner
**Status:** ready for integration

## What changed

Task 34 made one network fail on purpose. Another program held the port presence
advertises on one interface, so the service's listener there was refused. The log
said a machine on that network could not reach this one *until it can be*. But
nothing tried again once the port was free. The listeners moved only when the kernel
said a network changed, and a program closing a socket is not a network change. So a
port taken for a moment at start-up (by an installer, say, or a service restarting)
left the machine found on that network but unreachable there until some cable
changed. The service now hears from the kernel that the port was let go of, and
listens there again.

- `crates/alo-agentd/src/told_of_a_port_let_go.rs` (new): how the service hears that
  a TCP socket at its port was destroyed. It opens a `NETLINK_SOCK_DIAG` socket that
  joins `SKNLGRP_INET_TCP_DESTROY` and `SKNLGRP_INET6_TCP_DESTROY`. Before joining,
  it attaches a classic socket filter (`only_the_port`) that keeps only messages
  whose local port is the wire's. The module doc sets out the readings that were
  weighed. Unit tests cover:
  - a listener let go of is heard, and the port binds on hearing it (IPv4 and IPv6);
  - a socket at any other port is never heard (a listener and a connection);
  - a connection at the port closing is heard while the listener still holds the
    port, which is why hearing is only a reason to try again.
- `crates/alo-agentd/src/listeners.rs`: the listeners open that socket **before**
  their first bind, and remember which networks were refused (by interface
  number). `let_go_of` empties the socket and, only if something was refused, tries
  every network again. A refusal is said **once**, when first refused, and it now
  says when the port is tried again, which depends on whether the kernel will say.
  A network that binds after being refused is said once: *the port presence
  advertises is bound on … now that nothing else holds it there*. A network that
  goes is forgotten. If the socket will not open, that is a line in the log, and
  refusals then say the port is tried again only when the networks next change.
  New unit tests cover:
  - a let-go that leaves the port held is still refused, and not said again;
  - a let-go with nothing refused changes nothing;
  - a listener handed in hears nothing.

  The existing refusal test now also checks that the refusal is said once, that
  the bind follows the let-go with no network change, and that the bind is said
  once.
- `crates/alo-agentd/src/wire.rs`: `let_go_waiting_on` and `port_let_go_of`.
- `crates/alo-agentd/src/serving.rs`: the service's round also waits on that socket,
  and calls `port_let_go_of` when it speaks.
- `crates/alo-agentd/src/a_port_another_program_let_go_of.rs` (new): the fixture on a
  real kernel, described below.
- `crates/alo-agentd/src/lib.rs`: both registered.
- `Cargo.toml`: the comment on `socket2` names the new file, which uses it for
  `SO_ATTACH_FILTER`. The dependency itself is unchanged.
- `docs/contracts/local-network-wire.md`: *A port another program lets go of*
  (new, additive). Nothing on the wire changed.
- `docs/quirks.md`: a new entry. A person's service with no capabilities hears every
  TCP socket in its network being destroyed, and a classic filter narrows that in
  the kernel.
- `docs/autonomy/v0-5-the-local-network-plan.md`: task 35 marked done, and task 36
  written (the IPv6 listener refused at start is never tried again).

**Change description, for the changelog:** A computer running alo OS whose network
port was held for a moment by another program, such as an installer or a service
restarting, now becomes reachable on that network as soon as the other program lets
go. It needs no restart and no change to its networks. It learns this from the Linux
kernel at the moment it happens, without checking on a timer and without any extra
privilege, and it never looks at other programs' connections to do so. Its log says
once that the port was taken, and once that it is reachable again.

## Decisions

- **How the service learns the port is free: the kernel's TCP socket destruction
  broadcast.** I expected this to need `CAP_NET_ADMIN`, which the unit forbids
  (`CapabilityBoundingSet=` is empty, ADR 0001 §2, ADR 0018). If it had, the task
  would have become an ADR. **I measured instead.** As uid 1000 with `CapEff` 0, in
  the initial namespaces, the join succeeded. A listener's close was delivered at
  once with its port, and a bind right after it succeeded. The fixture holds the
  service to the same condition: reception runs under `setpriv --bounding-set=-all`
  and asserts `CapEff` and `CapBnd` are zero. That way the test cannot pass on a
  user namespace's capabilities alone. Alternatives weighed and rejected:
  - **a `pidfd` on the holder:** the holder is found through another process's
    `/proc/<pid>/fd`, which a person's service cannot read for a root installer, and
    a program can close the socket and keep running;
  - **an interval:** forbidden, and it wakes a machine where nothing happened;
  - **knocking on the other program's port and waiting for the reset:** it delivers
    a connection into somebody else's service, and a port that is bound but not
    listening refuses the knock.

  **No ADR was written**, because a kernel reading exists without an interval.
- **The service reads only its own port.** Unfiltered, the group carries every TCP
  close in the network namespace, with addresses. A classic BPF filter (four
  instructions, offset 20 = `idiag_sport`) drops the rest in the kernel before it is
  queued. It is attached before the join. The unit test
  `a_socket_at_any_other_port_is_never_heard` is what holds this, and the mutation
  run below shows it bites.
- **Hearing is a reason, not proof.** The wire's own accepted connections are
  destroyed at the port too, and so are the other program's connections while its
  listener lives. So a message triggers a retry only when something was refused, and
  a retry that is refused again says nothing.
- **Said once, both ways.** The dock fixture's quirk entry recorded the refusal line
  repeating on every network change. A log that repeats a line on every retry is
  noise, and with let-go retries it would repeat on every connection the wire
  closed. The refusal is said when a network is first refused, the bind when it is
  first bound, and a network that goes and comes back refused is said again.
- **Subscribed before the first bind**, so a let-go between a refusal and the
  subscription cannot be missed. The subscription is always open (not only while
  something is refused), which avoids that race entirely. With nothing refused, the
  cost is emptying a socket that only speaks when a TCP socket at the wire's own
  port closes.
- **How *with no network changing* is made certain.** `ip -o monitor link address`
  runs in reception's network, subscribed to the same three routing groups as the
  service. The fixture waits until the kernel lists that monitor's routing socket
  (groups `00000111`, inode among its descriptors) before the squatter lets go.
  Every link-local address has already finished duplicate address detection. The
  monitor must print nothing between the let-go and the port being reached.
- **Where the code lives.** Hearing the let-go is its own file
  (`told_of_a_port_let_go.rs`), as `told_of_a_move.rs` and
  `interfaces_that_went.rs` are. Retrying stays in `listeners.rs`, which already
  owns *where the port is listened on*.

## Acceptance criteria

| Criterion | Test |
|---|---|
| With the port held by another program on one cable, it is not reached there and is reached on the other | `alo-agentd` lib `a_port_another_program_let_go_of::tests::a_port_another_program_let_go_of_on_one_network_is_listened_on_there_again` |
| Once that program lets go, with no network changing, the port is reached on that cable | same test (with `ip monitor` printing nothing) |
| The service log says so once | same test (exactly one refusal line and one bind line naming `cable1`, none for another network); `listeners::tests::a_network_that_will_not_bind_is_a_line_and_the_others_are_still_bound`; refusal path `listeners::tests::a_let_go_that_leaves_the_port_held_is_still_refused_and_not_said_again` |
| How the service learns the port is free, decided with a kernel reading | `told_of_a_port_let_go::tests::a_listener_let_go_of_is_heard_and_the_port_then_binds`; refusal path `told_of_a_port_let_go::tests::a_socket_at_any_other_port_is_never_heard` |
| What is said is the same bytes throughout | the fixture (both cables' answers equal, and equal before and after) |

## Verification

Platform: WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, run as root
against `/mnt/c/dev/alo-os-claude` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`. The fixture
itself runs in user and network namespaces, with reception holding no capabilities.

- **Probe before any code** (scratch script, not committed): joining the destroy
  groups succeeded as root, as uid 1000 with `CapEff` 0, and as mapped root in a
  user namespace. Each time, closing a listener at port 47123 was delivered with that
  port, and a bind right after it succeeded.
- The fixture run on its own (far end with `--nocapture`): **passed** in 5.2 s. The
  service log said, once each:
  - *the port presence advertises could not be bound on cable1 (10.75.1.1): Address
    already in use (os error 98); … tried again as soon as the kernel says a program
    let go of the port*
  - *the port presence advertises is bound on cable1 (10.75.1.1) now that nothing
    else holds it there; …*
- **Mutation run 1:** `Listeners::let_go_of` emptied the socket and did not try
  again. The fixture **failed** with *reception never held
  `cable0,cable1|cable0,cable1`: it holds `cable0,cable1|cable0`*. Reverted.
- **Mutation run 2:** the filter's last instruction kept every message. The unit
  test **failed** with *the destruction of a socket at another port reached the
  service*. Reverted.
- One run of the crate suite failed my own test
  `a_let_go_with_nothing_refused_changes_nothing`. It had asserted that nothing more
  arrived within 200 ms after emptying the socket. The kernel sends these messages
  from its own queue, so a socket closed earlier (the probe listener `a_free_port`
  binds) can arrive late. The assertion claimed something that is not true, so it
  was replaced with what the code promises: a second `let_go_of` with nothing
  waiting returns and changes nothing.
- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps`: clean.
- `cargo test -p alo-agentd`: **passed**. That is 451 unit tests (30 ignored, the
  namespaced halves of fixtures, including task 34's dock fixture with the refusal
  now said once) and every integration target.
- The worker did not run the full workspace suite. The supervisor runs it.

## Remaining limitations

- **The certified image's kernel is not measured.** The Fedora bootc kernel
  normally builds `CONFIG_INET_DIAG` as a module, which the kernel loads when the
  group is joined, but nobody has checked that on the image. A kernel without it
  refuses the join, the service log says so, and a taken port is then tried again
  only on a network change. Owed to the certified machine.
- **The IPv6-only listener is not covered.** If another program holds `[::]` at the
  port when the service starts, it is still never tried again. That is task 36,
  written in the plan.
- Measured with `veth` cables in namespaces on one kernel. Not measured on a
  physical adapter.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** local-network task 35 done, with this report as evidence.
  Task 36 (*A port another program held over IPv6 at start is listened on over IPv6
  once it is let go of*) is ready.
- **ROADMAP.md:** no change; no v0.01 box moves.
