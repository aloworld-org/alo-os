# A cable deleted and re-laid between two readings is still joined, over IPv4 as well

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 31)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, in words a person outside this repository can read

An alo machine whose network adapter disappears and comes straight back is now
found again by the machines on that network, with nothing restarted. This happens
when a dock re-enumerates its adapters or a virtual network is rebuilt. Before this
change, if the adapter came back under the same internal number before alo OS had
looked, the machine went on answering "who is here?" on a network that no longer
delivered the question to it. Nobody on that cable found it until the service
restarted, while connections to it still worked.

alo OS now listens for Linux saying an adapter was **deleted**, not only for what
the adapters look like afterwards. An adapter that was deleted is treated as new,
even when it comes back looking identical. The machine also no longer trusts
"already joined" from the kernel without checking: it leaves and joins the
discovery group again.

## What was built

- `crates/alo-agentd/src/interfaces_that_went.rs` (new): `Went`, the interfaces
  the kernel said were deleted in one round of the service. Messages the kernel
  dropped (`ENOBUFS`), or a message that cannot be read, count as *any interface
  may have gone*. The module documentation weighs the three readings. Three unit
  tests.
- `crates/alo-agentd/src/route_messages.rs`: `links_deleted_in` reads the index
  out of each `RTM_DELLINK`, in the unspecified family only (a bridge says
  `RTM_DELLINK` in `AF_BRIDGE` when a port merely leaves it). Two unit tests.
- `crates/alo-agentd/src/unix.rs`: `emptied` hands each datagram it reads, and
  "messages lost", to a closure instead of discarding them.
- `crates/alo-agentd/src/joining.rs`: `Joining::changed` returns the `Went` it
  read. A link-local network whose interface went is left and joined afresh even
  where it is reported again (`kept_and_gone`, one unit test).
- `crates/alo-agentd/src/responding.rs`:
  - `Responders::changed` takes `&Went`. A responder whose interface went is let
    go of even where its number is reported again, and that network is answered
    on by a new socket.
  - **Every responder let go of leaves its group at that moment** (`Responder::leave`).
  - `joined` takes a join refused `EADDRINUSE` afresh (leave, then join) instead
    of reading it as joined. It is `pub(crate)` so the fixture can measure it.
  - `left` is new.
  - Three unit tests are new: a refused join taken afresh, together with a
    responder let go of leaving at once while a round still holds its socket; a
    join where no interface is refused; and a responder whose interface went
    answered again on a new socket while the others keep theirs, plus a lost
    round replacing every socket.
- `crates/alo-agentd/src/wire.rs`: `networks_changed` passes what `Joining` read
  to the responders.
- `crates/alo-agentd/src/listeners.rs`: documentation only, explaining why the
  listeners need none of this (measured).
- `crates/alo-agentd/src/a_cable_re_laid_between_two_readings.rs` (new): the
  measurement. `lib.rs` registers the new modules.
- `docs/contracts/local-network-wire.md`: *A cable deleted and laid again before
  the machine looks* (new, additive; nothing on the wire changes).
- `docs/quirks.md`: *an IPv4 membership also outlives a deleted interface on the
  socket that joined it, while a socket held to the number keeps working* (new).
- `docs/autonomy/v0-5-the-local-network-plan.md`: task 31 marked done, and task 32
  written.

## Decisions

### How the ordering is made certain

The criterion asks for the cable to be deleted and re-laid **before the service
reads the interfaces again**, with the method decided and written up. A test that
races the service's thread gets that ordering only when it happens to win. So
reception runs as **a process of its own**, and the far end:

1. sends `SIGSTOP`;
2. waits until `/proc/<pid>/task/*/stat` reports every thread stopped (`T`);
3. deletes the cable and waits until neither end is reported;
4. lays it again (`ip link add … index N`, retried until the number is free);
5. sends `SIGCONT`.

Everything the kernel said in between is waiting on reception's routing socket,
and the next reading reception takes is of the re-laid cable. A stopped and
continued process is the same process, so the service does not restart.

*After the service has followed the kernel* is also read from the kernel, not
guessed. Nothing in reception's network namespace joins `224.0.0.251` except the
service, so the fixture waits for two things: `/proc/<pid>/net/igmp` listing the
re-laid interface in the group, and the wire reporting that it answers, listens
and is joined at the expected number. It then asks **once**.

### How a responder and a listener tell a re-laid interface from the one that went

Three readings were weighed. The choice is the first:

- **The kernel's `RTM_DELLINK`**, already arriving on the routing socket the
  service waits on. It is exact, costs nothing extra, and is sent at the moment
  the membership goes. A dump taken afterwards cannot see a same-number re-lay.
- **`/proc/net/igmp`** answers for the interface, not the socket. If another
  process had joined the group there, the interface would look fine. When that
  process later left, the responder would be deaf with no notification.
- **Taking every membership afresh on every notification** is always correct.
  But it costs a leave and a report per network every time any address anywhere
  changes, and an IPv6 router advertisement renewing a lifetime is one such
  change.

Lost or unreadable messages (`ENOBUFS`, a message cut short) are read as *any
interface may have gone*. Every responder is then answered again once.

**A responder whose interface went is replaced by a new socket rather than kept
and re-joined**, which is how a network plugged in has always been handled. What
makes this safe is leaving at the moment of letting go. The kernel counts a
group's users per interface, and a socket closing leaves on whatever interface
has the number by then. The answering thread may hold the old socket open for the
rest of its round, and a late close would take one user off the new socket's
membership on the re-laid interface.

**The listeners are unchanged.** `SO_BINDTOIFINDEX` is a number the kernel
compares with the arriving interface, and a TCP listener joins no group. The
fixture reaches the port on the re-laid cable through the listener that was
there before.

**`crate::joining` takes the same reading over IPv6.** Task 30 left one gap open:
a link-local cable that comes back identical (same number, name and address)
between two readings. The unit test holds the rule; a real-kernel measurement is
task 32.

## The measurement

On a real kernel. The far end runs in a user and network namespace; reception
runs in a network namespace nested inside it, serving as `src/main.rs` does with
a person's door, and driven over its standard input. One `veth` carries IPv4:
`10.72.1.1/24` at reception and `10.72.1.2/24` at the far end.

1. **Laid**, reception's end numbered 40: the far end finds reception and reaches
   its port, and the door answers.
2. **Deleted and re-laid at the same numbers, with reception held still.**
   - A probe on the far end's side measures the kernel. The re-laid `far0` is not
     in `224.0.0.251`, and a second join from the probe is refused `EADDRINUSE`.
   - `crate::responding::joined(&probe, …)` then succeeds with `far0` in the
     group.
   - Once reception has followed the kernel, the far end's **first** question is
     answered with the same bytes, the **first** connection reaches the service,
     and the door answers.
3. **Deleted and re-laid at new numbers (42), with reception held still:** the
   same results.

**Mutation runs:**

- The `went` check removed from `Responders::answer_on`: the fixture fails at
  *reception never followed its cable to 40: answered, listened and joined at
  `40 40 40`, and the interface is not in the discovery group*. The wire kept its
  socket and the kernel had nobody in the group. The port was reached throughout.
- `joined` reading `EADDRINUSE` as joined again: the fixture fails at *a join
  refused as already held was counted joined with the interface out of the
  group*, and so does the loopback unit test.

## Verification

Platform: WSL2 Ubuntu 24.04, kernel `6.18.33.2-microsoft-standard-WSL2`, util-linux
2.41.3, iproute2 6.19.0; `CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`.

**Executed:**

- `cargo test -p alo-agentd --lib -- a_cable_re_laid`: passed three times in a
  row.
- `cargo test -p alo-agentd --lib -- a_cable_pulled two_machines_with_no_ipv4 a_discovery_answer a_machine_reachable`:
  5 passed. These are tasks 29 and 30, whose code paths changed.
- `cargo test -p alo-agentd --lib -- responding:: joining:: route_messages:: interfaces_that_went:: unix:: wire::`:
  all passed.
- The two mutation runs above.
- `cargo fmt --all`: clean (`--check` exits 0).
- `cargo clippy --workspace --all-targets -- -D warnings`: exits 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps`: exits 0.
- `cargo test -p alo-agentd`: exits 0. The library has 438 passed and 20 ignored
  (the ignored ones are the halves each fixture runs inside its namespaces).
  `a_machine_on_two_networks` 1 passed, `a_question_is_bounded_by_the_kernel` 5,
  `a_turn_is_bounded_by_the_kernel` 1,
  `a_turn_is_refused_when_the_boundary_is_gone` 1, `two_doors_on_one_socket` 4,
  and `what_a_machine_says_about_itself` 12.

**Not run by this contributor:** the workspace suite, which the supervisor runs.

## Remaining limitations

- If another process on the same machine had joined `224.0.0.251` on the re-laid
  interface, leaving the dead membership takes one user off theirs. This is the
  kernel's per-interface counting. Nothing on alo OS joins that group except this
  service, so it is written down in `docs/quirks.md` rather than worked around.
- The IPv6 half of the reading has only a unit test. The measurement is task 32.

## Proposed updates to the shared documents

- **CHANGELOG.md:** "A network adapter that disappears and comes straight back,
  such as a dock re-enumerating, no longer leaves the machine unfindable on that
  network until the service restarts."
- **QUEUE.md / STATE.md:** v0.5 local-network task 31 done, with this report as
  its reference. Task 32, *A link-local cable re-laid with the same hardware
  address between two readings is still joined*, is ready.
- **ROADMAP.md:** no change.
