# A discovery answer leaves on the network the question arrived on

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 28)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, in words a person outside this repository can read

Task 27 made an alo machine plugged into two networks **reachable** on both. It
could still not be **found** on both. When another computer on the network asked
*who is here*, this machine's answer went out whichever cable its own routing
preferred — so on the commonest office and the commonest home, where two routers
hand out the same range of addresses, the colleague on the other cable asked and
heard nothing. They could not pair with a machine they could not find, even though
its port was now open to them.

Now the answer goes back the way the question came. A machine asking on the wired
network is answered on the wired network; a machine that happens to have the same
address on the Wi-Fi is sent nothing at all. Two machines on the network the
computer's own routing does not favour now find each other and pair with nothing
configured anywhere. What the machine says about itself is unchanged, byte for
byte, on every network and in both families — and there is still nothing to set:
which networks it answers on is what it is plugged into.

## Why, and what the kernel really does

`alo-agentd` answered discovery on one socket per family, held to no network, and
`alo_nearby::Answering::answer_one` replies to the asking machine's unicast
address. A probe run before the code, and the new fixture, measured what that does
(`docs/quirks.md`, 2026-09-16 — two `veth` cables carrying one private range, a
machine at `10.67.0.2` at each far end, the route pointing at the cable the asking
machine is *not* on):

- **An unheld socket answers by the route.** It heard the question from the cable
  perfectly well; its answer left by the other network, and the machine that asked
  heard nothing. This is the failure the task exists for, measured.
- **Held datagram sockets at one port coexist** — `lo`, the first cable and the
  second, each `0.0.0.0:5399` with `SO_BINDTOIFINDEX` set before the bind. For UDP
  as for TCP the bind-conflict check treats a different `sk_bound_dev_if` as a
  different binding.
- **An unheld socket at the same port binds beside them too**, which is where UDP
  differs from TCP: `SO_REUSEADDR` is enough and nothing refuses it. So held
  sockets do not *displace* an unheld one the way task 27's held listeners did —
  the machine has to stop binding one, deliberately.
- **A question is delivered only to the socket held to the interface it arrived
  on**, multicast and unicast alike; the other held sockets are not given a copy.
- **That socket's answer leaves by the interface it is held to**, against the
  route: the machine that asked heard it, and the machine at the same address on
  the other network heard nothing.

## The decision the task left open, and what it cost

The plan asked for the reading to be decided in the crate that answers and written
up with its costs. Three readings the kernel really gives were available.

**Chosen: one datagram socket per network, held to its interface.** It is the
reading the rest of this daemon already uses — `crate::looking::held_to` holds a
socket for asking, `crate::listeners::held_to` one for listening — it needs no new
dependency and no change to how a datagram is read, and the kernel measurements
above say it is exact for every question that reaches it.

**Not chosen: `IP_PKTINFO` read back with `recvmsg` and answered with `sendmsg`.**
It is the *more* exact reading — the arriving interface comes off each datagram, so
it needs no list of interfaces at all and would answer correctly even on a machine
whose own interfaces cannot be read, with one socket per family instead of one per
network. **It is not available in safe Rust here.** Control messages are `cmsghdr`
bytes: `rustix` has no `IP_PKTINFO` ancillary message, `socket2` hands out the raw
control buffer and nothing that parses it, and laying a kernel structure out of raw
bytes by hand is exactly the shape of thing law 2's companion rule against
`unsafe` exists for. A later task can make that change on its own merits, behind a
crate that parses control messages safely; it is written down in
`crates/alo-agentd/src/responding.rs` and in `docs/quirks.md` as what is not
closed.

**Not chosen: leaving the answer to the route**, which is what the code did and
what ADR 0044 refuses.

What the chosen reading costs, network by network — the plan asked for this
explicitly:

- **Loopback** is one of the networks answered on, so a question to `127.0.0.1` is
  answered exactly as it is today, by the person's own machine and by every test
  that puts two machines on one host. It is never **joined**, because a multicast
  question on loopback reaches this machine only. The set of networks answered on
  is therefore `crate::networks::listening_networks` — the same set the port is
  listened on — and the joins are the subset that carries multicast.
- **An interface that carries no multicast** (a point-to-point tunnel) is answered
  on and not joined, for the same reason.
- **Over IPv6 nothing is held.** An IPv6 discovery question comes from a link-local
  address that carries the interface it was heard on in its own scope (ADR 0041),
  and the kernel answers back out of that interface without being told to. So the
  IPv6 responder stays one socket held to nothing, joined per link-local network by
  `crate::joining`. What that leaves open is a question from a **global** IPv6
  address arriving on one of two interfaces, whose answer would go by the route —
  discovery never asks from one.
- **A question whose network cannot be read is answered on no network.** An
  interface the kernel numbers zero takes no socket, and a machine that cannot read
  its interfaces at all takes none — each a line in the service log. This is
  deliberately **not** what `crate::listeners` does, which binds one listener held
  to nothing rather than listening nowhere, and the two differ because the failures
  differ: an unheld handshake merely fails to complete, and nobody is told anything
  untrue, while an unheld *answer* is a datagram saying *this machine is here* sent
  to a machine that did not ask — and the machine that did ask finds nothing
  either way.

## The code

New: **`crates/alo-agentd/src/responding.rs`**

- `Responders::bound` binds one IPv4 datagram socket per network the kernel
  reports, each held to that network's interface and joined to `224.0.0.251` on the
  networks that carry multicast, with **no unheld socket beside them**. Every
  failure on the way is a line in the service log; a network that will not take a
  socket leaves the others answering.
- `Responders::on` is the one socket a test hands in, and `also_answering_on` the
  IPv6 one — both held to nothing and following nothing.
- `Responders::hosting` tells every responder, and every responder made later, the
  workspace this machine hosts, so a network answered on after a cable is plugged
  in says exactly what the ones answered on at start say.
- `Responders::changed` follows the kernel's network notifications as the listeners
  and the joins do, called from `Wire::networks_changed`; responders are matched to
  networks by interface **index**.
- `Responding::of` takes a handle onto each responder for one round and nothing is
  locked while that round waits — which may be for ever.
- `Responding::answer_the_first_that_speaks` is what a caller with nothing else to
  wait on uses: one responder is read directly, so a read timeout set on a socket
  handed in comes back; several are waited on at once.

Changed:

- **`crates/alo-agentd/src/wire.rs`** — the wire holds `Responders` instead of two
  sockets and two `Answering`s; `Wire::responding`, `Wire::answered_on`;
  `Wire::joined` is the IPv4 groups the responders joined plus the IPv6 ones;
  `networks_changed` moves the responders too.
- **`crates/alo-agentd/src/joining.rs`** — the IPv4 joins moved out to the socket
  that is held on each network, where the join belongs. What stays is the IPv6
  group and the kernel's notification, which the wire hands to the listeners and
  the responders as well; a machine with no IPv6 still follows its networks.
- **`crates/alo-agentd/src/answering_discovery.rs`** — the thread waits on every
  responder and on the end that stops it, taking the responders again each round
  because a machine plugged in or unplugged answers on a different set of sockets
  from the one the last round waited on.
- **`crates/alo-agentd/src/unix.rs`** — `a_shared_datagram_socket_on` and `ready`
  are now only a test's; the service binds held sockets and waits on lists whose
  length it does not know until it asks (`ready_and`).
- **`crates/alo-agentd/src/lib.rs`** — the new module, the new fixture, and
  `DiscoverySockets` gone from the re-exports.

## Decisions I made, and why

1. **The set of networks answered on is `listening_networks`, the same set the
   port is listened on** — loopback included, multicast not required — and the
   *joins* are the subset from `discovery_networks`. It keeps one invariant worth
   having: a machine is found and reachable on exactly the same networks, and a
   question that arrives anywhere it listens is answered.
2. **No unheld socket is bound beside the held ones.** UDP would have allowed it,
   unlike TCP. An unheld socket would answer by the route and race the held ones
   for a unicast question, which is the whole failure.
3. **A machine that cannot read its own interfaces answers nowhere**, where
   `crate::listeners` binds one listener held to nothing. The asymmetry is
   deliberate and argued above and in the module: silence is the true thing to say
   when this machine cannot tell which network it would be speaking on.
4. **The workspace is kept by `Responders` rather than only inside each
   `Answering`.** Otherwise a cable plugged in after the service started would
   answer *who is here* and say nothing about the workspace this machine hosts —
   two machines to whoever asked on both networks.
5. **A responder whose socket cannot be taken a second time when the workspace is
   told is let go of with a line**, rather than left answering something other than
   what this machine advertises.
6. **`alo-nearby` is untouched.** The crate that answers takes the socket it is
   handed and replies to whoever asked; which interface that is has always been the
   caller's to decide, and this change is the caller deciding it. The decision and
   its reasons are written where the sockets are made.

## Acceptance, and where each part is tested

| Acceptance | Where |
| --- | --- |
| How an answer is held is decided in the crate that answers and written up with the reason, and what each reading costs loopback and IPv6 | `crates/alo-agentd/src/responding.rs` module documentation; *The decision the task left open* above; `docs/quirks.md` |
| One socket per network held to its interface, each joined on that network; loopback answered on and not joined | `alo-agentd` `responding::tests::discovery_is_answered_once_per_network_and_loopback_is_one` |
| An answer to a question that arrived on one network leaves on that network — two cables, one private range, a machine at the same address at each far end; the machine that asked hears the answer and the other hears nothing | `alo-agentd` `a_discovery_answer_leaves_on_the_network_it_arrived_on::tests::a_discovery_answer_leaves_on_the_network_the_question_arrived_on` |
| A question whose network cannot be read is answered on no network rather than by the route | `alo-agentd` `responding::tests::a_network_that_will_not_take_a_socket_is_a_line_and_the_others_still_answer`, `responding::tests::a_machine_with_no_networks_answers_on_none` |
| What is said is unchanged — the same identity, port and workspace, byte for byte, on every network and in both families | `alo-agentd` `responding::tests::what_is_said_is_the_same_bytes_on_every_network_and_in_both_families` |
| A network that will not take a socket is a line in the service log and the others still answer | `alo-agentd` `responding::tests::a_network_that_will_not_take_a_socket_is_a_line_and_the_others_still_answer` |
| Two machines on a network the route does not point at find each other and pair with **no policy rule in the fixture** | `alo-agentd` `a_discovery_answer_leaves_on_the_network_it_arrived_on::tests::a_discovery_answer_leaves_on_the_network_the_question_arrived_on` |

### The refusal paths, beside the legitimate ones

- A network the kernel numbers **zero** takes no socket: it is a line in the log
  naming the network, the others go on answering, and a question arriving there is
  answered by nobody rather than by the route.
- A machine whose interfaces cannot be read answers on **no** network at all, waits
  on nothing, and refuses to answer rather than answering by the route.
- Nothing ready is nothing answered, and a packet that is not a question is stepped
  over rather than replied to
  (`responding::tests::nothing_ready_is_nothing_answered_and_a_non_question_is_stepped_over`).
- A network that goes is let go of, so a socket held to an interface that has gone
  is not kept
  (`responding::tests::a_network_that_goes_is_let_go_of_and_one_that_comes_is_answered_on`).
- In the fixture, **somebody else at the same private address on the other
  network** is the witness: it answers discovery *as the studio*, counts every
  question, every connection and every datagram at the port the studio asks from —
  which is where an answer that left by the route would land — and reports nothing
  on all three.
- **A mutation run proves the fixture measures the hold.** With
  `bind_device_by_index_v4` given `None` — the code as it was — the fixture fails
  at *the machine that asked did not hear this machine's answer*. Reverted.

### One thing measured structurally rather than over a socket

The byte-for-byte claim *on every network and in both families* is asserted for
every responder where the answers are made, and asked for real over IPv4 on
loopback. It is not asked for real on the other networks or over IPv6 on **this**
host, and cannot be: a datagram to this machine's own address on another interface
is delivered over loopback, and this kernel's first network namespace has IPv6
switched off altogether — both already in `docs/quirks.md`. The two cables carrying
one private range are the new fixture; the IPv6 answer asked for real is
`crate::two_machines_with_no_ipv4`'s and `crate::a_paired_machine_over_link_local`'s,
which run inside namespaces of their own and are unchanged by this task.

## Proposed changelog entry

> **A machine is found on every network it is on.** An alo machine plugged into
> two networks that hand out the same range of addresses — the commonest office
> and the commonest home — answered *who is here* out of whichever cable its own
> routing preferred, so a colleague on the other one asked and heard nothing.
> Discovery is now answered on one socket per network, held to that network, so an
> answer goes back the way the question came and nobody else is sent anything. Two
> machines on the network the routing does not favour now find each other and pair
> with nothing configured. What the machine says about itself is unchanged.

Roadmap and queue: nothing to tick that is not the plan's own task 28, marked done
in `docs/autonomy/v0-5-the-local-network-plan.md` in this change, with task 29
written after it (*A cable pulled is a network this machine is no longer found on,
and one plugged in is found at once*) — the real-kernel measurement no test in this
repository has yet made, and the one place a socket held to an interface that has
gone would show.

## Verification

Run from `/mnt/c/dev/alo-os-claude` in WSL Ubuntu (kernel
`6.18.33.2-microsoft-standard-WSL2`) with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`.

- `cargo fmt --all` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps` — clean.
- `cargo test -p alo-agentd` — passed: 415 unit tests (14 ignored — the halves
  the real-kernel fixtures re-run inside their namespaces), and every integration
  target green.
- Each evidence test run on its own with `--exact` — all eight passed; the
  end-to-end fixture in about ten seconds.
- The mutation (`bind_device_by_index_v4(None)` in `responding.rs`) was run again
  on 2026-09-16 before this handoff: the fixture failed at *the machine that asked
  did not hear this machine's answer*, and the hold was restored and the fixture
  passed again.

**How this change reached the tree.** The work was first built in a local commit
(`362754c`) that the supervisor parked on `parked/task-29-*` without publishing,
because the handoff named task 28 while the plan in that tree had already moved
on to 29. It was brought onto current `main` unchanged (the only overlap,
`docs/quirks.md`, merged cleanly), checked against the acceptance above and gated
again from scratch.

The workspace suite is the supervisor's, per the task's instructions. No other
crate's source was touched.

## Limitations

- A question arriving on an interface this machine cannot enumerate is answered by
  nobody rather than answered exactly; `IP_PKTINFO` would answer it, and needs a
  crate that parses a `cmsghdr` safely.
- A question from a **global** IPv6 address arriving on one of two interfaces would
  be answered by the route. Discovery asks from link-local addresses only, so
  nothing this product does reaches that path.
- The responders follow the kernel's notifications, and that following is tested by
  handing them a list rather than by pulling a cable on a real kernel. That test is
  task 29.
