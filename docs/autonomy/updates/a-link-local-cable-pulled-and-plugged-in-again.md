# Two machines with no IPv4 address between them find each other again when the cable comes back

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 30)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, in words a person outside this repository can read

Two alo machines joined by a cable with no router and no DHCP server between them
now find each other again after that cable is unplugged and plugged back in, with
nothing restarted. That includes a cable swapped for a new one, or a USB network
adapter pulled and pushed back in. While the cable is out, neither machine offers
the other, and asking to pair with it is refused before anything is sent. Once the
cable is back, each finds the other where it is now, and a new pairing request
goes to the new connection, never to the old one.

Measuring this found a real bug, and this change fixes it. When a network adapter
disappears, Linux quietly leaves a record of the old group membership on the
machine's discovery socket. If a replacement adapter then got the same internal
number, alo OS saw that record and assumed it was already listening there. It
wasn't, so the two machines never found each other again until the service
restarted. alo OS now drops the membership when a network goes away, and never
assumes it is still listening.

## What was built

- `crates/alo-agentd/src/two_machines_with_no_ipv4_find_each_other_again.rs`
  (new): the measurement on a real kernel, described below.
- `crates/alo-agentd/src/joining.rs`: `follow` now **leaves** `ff02::fb` on every
  network that is no longer reported, not only forgets it. `join` no longer reads
  `EADDRINUSE` as *already joined*: it leaves and joins again, and returns the
  second answer. `left` is new. The module documentation says why, and four unit
  tests are new: a membership already held is still held after joining again; a
  network that went is left; leaving a network never joined is nothing; and a join
  at a number no interface has is refused.
- `crates/alo-agentd/src/lib.rs`: the new test module registered.
- `docs/quirks.md`: *an IPv6 membership outlives a deleted interface on the
  socket that joined it, and a join there then says `EADDRINUSE`* (new).
- `docs/contracts/local-network-wire.md`: *A cable with no IPv4 address, pulled
  and plugged in again* (new, additive; nothing on the wire changes).
- `docs/autonomy/v0-5-the-local-network-plan.md`: task 30 marked done; task 31
  written.

## The measurement

Two daemons, each running `Serving` over `Wire::bound` as `src/main.rs` does, with
a person's door. Reception runs in a user and network namespace, the studio in a
network namespace nested inside it, and one `veth` runs between them with no IPv4
address. The studio does what reception tells it on standard input. It hosts a
workspace, so both discovery answers are compared byte for byte. It also holds a
probe socket of its own, joined to `ff02::fb` beside the service's, to measure
what the kernel does with a membership.

1. **Cable laid:** each finds the other at its link-local address, with its
   interface, and they pair through task 12's request.
2. **The studio's link set down:** reception's end stops being joined, and
   neither machine finds the other. A proposal to the studio is refused with
   *no machine by that identity answered on this network just now*, and no
   proposal is left waiting. Both doors still answer, with the pairing intact.
   The probe's membership is kept on the socket and on the interface.
3. **Set up again:** each finds the other at the same index and the same
   address, with no restart, and the answers are the same bytes.
4. **Cable deleted:** neither machine finds the other, and both doors answer.
   `/proc/net/igmp6` lists the group on no interface at that number, yet the
   probe's second join there still answers `EADDRINUSE`.
5. **Re-laid:** there is a new index at each end and a new link-local address.
   Each machine finds the other with the new interface, and the answers are the
   same bytes. A TCP connection to the studio's address at the old index is
   refused, and `found_at` there finds nothing.
6. **A proposal to the paired machine** pairs again. The studio shows a code only
   after its service has measured reception at the address the connection came
   from, which carries the new interface because no other exists.
7. **Deleted again, and re-laid at the numbers it first had**
   (`ip link add … index N`): each finds the other.

**Step 7 failed before the fix**, with *reception never found the studio*.
Mutation runs:

- both halves of the fix removed (the code as it was): fails at step 7;
- only the leave on a network that went removed: passes;
- only the fresh join after `EADDRINUSE` removed: passes.

Either half alone covers this fixture. Both are kept, for different reasons:

- **Leaving** stops dead memberships from piling up, one for every cable pulled.
- **Joining afresh** covers a cable deleted and re-laid between two readings of
  the interfaces. The service never saw that network go, so nothing was left.

## Decisions

- **The far end's namespace is not ended.** The plan asks for the far end's
  namespace to end, and also for neither service to restart. The studio is the
  far end, and its namespace lasts exactly as long as the studio's service does,
  so both cannot happen. The cable is deleted instead, from the studio's side,
  which is what a namespace ending does to a `veth`. Task 29 measured that, in a
  fixture where the far end had no service to keep. Every other part of that
  pull is kept: a new interface at each end, a new index, a new address.
- **A direct `veth`, not a bridge between two cables.** A bridge in a third
  namespace would let a namespace really end with both services running. It
  would also put a switch between the machines, and the plan names one cable.
  Its multicast snooping would add behaviour this task is not about.
- **The fix lives in `crate::joining`, not in `crate::networks`.** The rule for
  which interfaces to join did not change. What changed is what a join and a
  leave mean on the socket, which is `joining.rs`'s one responsibility.
- **A proposal after a re-lay needed no change.** Nothing keeps an address:
  `crate::pairing` looks the machine up at the moment of proposing, and the
  machine asked measures the proposer at the connection's own scoped address.
  The test shows it, and step 5 shows that the old index could not have been
  dialled anyway.
- **Task 31 is written** for the IPv4 side. `crate::responding` and
  `crate::listeners` match a network by interface index alone, and the IPv4 join
  also reads `EADDRINUSE` as joined. A cable re-laid at the same index between
  two readings is the same failure in the other family, and nothing has measured
  it yet.

## Verification

Run on Windows 11 through WSL2 Ubuntu (kernel `6.18.33.2-microsoft-standard-WSL2`),
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`, from
`/mnt/c/dev/alo-os-claude`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean, exit 0.
- `cargo test -p alo-agentd`: exit 0. The library has 428 passed and 18 ignored
  (the ignored ones are namespace inner halves); every integration target passes.
- `cargo test -p alo-agentd --lib two_machines_with_no_ipv4_find_each_other_again`:
  1 passed, in about 39 s.
- `cargo test -p alo-agentd --lib joining::`: 4 passed.
- The three mutation runs above, executed and then reverted.

Not run: the full workspace suite (the supervisor runs it), and anything on
certified hardware. A `veth` in a namespace is not a physical NIC, and a real
adapter's carrier and renumbering behaviour is still to be measured on the machine.

## Remaining limitations

- The IPv4 responders and listeners have the same index-reuse gap (task 31).
- The fresh join after `EADDRINUSE` is covered on a real kernel only together
  with the leave. No fixture forces a delete and re-lay into one reading of the
  interfaces. Task 31 asks for that ordering to be made certain in its test.

## Proposed shared-document updates

- **CHANGELOG.md:** "Two alo machines on a cable with no IPv4 address find each
  other again after the cable is unplugged and plugged back in, including when
  the adapter comes back under the same interface number. A leftover kernel group
  membership used to hide them from each other until the service restarted."
- **QUEUE.md / STATE.md:** v0.5 local network task 30 done (this report); task 31
  ready.
- **ROADMAP.md:** no change.
