# A cable pulled is a network this machine is no longer found on, and one plugged in is found at once

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 29)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What changed, in words a person outside this repository can read

An alo machine now keeps up with its cables while it runs. Unplug a network and
the machine stops being found on it, while staying found and reachable on every
network it is still on. Plug it back in, and the machine is found by the first
computer that asks there, with no restart and nobody needing to reboot after
docking.

Measuring this found a real bug, and this change fixes it. The part of the
service that answers *who is here* went on listening on the connections it had
before a cable was plugged in. A colleague on a newly plugged-in cable asked and
heard nothing. They were only answered once somebody on a different network
happened to ask something. On a quiet network that meant never, and the symptom
was exactly *it only works if I restart it after docking*. What the machine says
about itself is unchanged, byte for byte, throughout.

## What was built

- `crates/alo-agentd/src/a_cable_pulled_and_plugged_in_again.rs` (new): the
  measurement on a real kernel, described below.
- `crates/alo-agentd/src/told_of_a_move.rs` (new): `ToldOfAMove`, a pair of
  non-blocking connected sockets. `tell` writes one byte, and a full pair counts as
  already told. `waiting_on` is the end to poll, and `heard` empties it.
- `crates/alo-agentd/src/responding.rs`: `Responders` holds a `ToldOfAMove`
  (`Responders::moved`) when it follows the kernel. `answer_on` tells it when a
  responder is let go of or added, and not when nothing changed or when a network
  refused its socket. A pair that cannot be made or written to is a line in the
  service log, never a refusal to start. `hosting` carries it across.
- `crates/alo-agentd/src/wire.rs`: `Wire::answering_moved`.
- `crates/alo-agentd/src/answering_discovery.rs`: each round empties the pair,
  takes the responders, and waits on the pair beside them and beside the end of
  the service.
- `crates/alo-agentd/src/lib.rs`: the two modules registered.

## The measurement

Reception runs the whole service over `Wire::bound`, as `src/main.rs` does, in a
user-namespaced network namespace. Two `veth` cables run to far ends in namespaces
of their own: the studio at `10.69.1.2` and a colleague at `10.69.2.2`. Each far
end asks the group *who is here* and prints the raw bytes of reception's answer.
It also connects to reception's port and reports "reached" only if the service
itself replied with `not-for-this-wire`, so somebody else holding the port does
not count.

1. **Both cables up:** both far ends find and reach reception, and its person's
   door answers.
2. **The colleague's cable pulled by its far end going away:** its namespace ends
   and the `veth` goes. Reception answers, listens and is joined on the studio's
   cable and not the colleague's. The studio still finds and reaches it, and the
   door answers.
3. **The same cable plugged in again**, with a new index. With no restart, the
   colleague's **first** question is answered and its first connection is answered.
4. **The studio's cable set down at reception's end:** the studio finds and
   reaches nothing, the colleague still finds and reaches reception, and the door
   answers.
5. **A failure on the way back:** a listener somebody else holds at the port on
   the studio's cable while it is down. When the cable comes up, discovery is
   answered there, the port is not listened on, and the service log says
   `the port presence advertises could not be bound on cable0`. The outer test
   reads the log from reception's stderr. The door and the colleague are still
   answered. Once somebody else lets go and the cable is re-seated, the studio
   finds and reaches reception again.

Every answer either far end heard is compared with the first, byte for byte.

**Mutation:** with `Wire::answering_moved` made to return nothing, and the cables
left to come up before the service starts, the test fails at step 3 with *the
colleague, again never found reception*. The very first run, before the fix,
failed at step 1 with *the studio never found reception*. The cables became
running a moment after the service read its interfaces, so they were bound only
through the notification, which is the same bug.

## Decisions

- **How the answering thread hears of a move: a byte into a socket pair, written
  by the responders.** That thread cannot also read the kernel's routing socket.
  The service's thread already reads it, and two readers take each other's
  messages. A second routing socket per thread would work, but it would repeat the
  kernel read and race the service's own rebind: the thread could wake before the
  new responder exists and go back to sleep on the old set. Telling *after* the
  set has moved closes that race. The pair is emptied *before* the responders are
  taken, so a move between the two leaves a byte and wakes the next wait at once.
  Nothing wakes on an interval, as the plan's constraint requires.
- **Told only when the set really moved.** A notification that changes nothing
  (an address lease renewed, a notification for an interface discovery does not
  use) wakes nobody, and neither does a network that refused its socket. Tested.
- **The listeners and the IPv6 joins needed no change.** The service's own thread
  waits on the routing socket beside the listeners, so it is never asleep on a
  stale set, and `Joining` reads the notification itself. The measurement shows
  both follow a pulled and re-laid cable. Step 5 also shows that a listener that
  would not bind is bound after the next notification once the port is free.
- **What "at once" means, tested:** the first question after the wire counts the
  network as answered on is answered. The socket is bound and joined before it is
  counted, so a question asked the moment after is queued on it and read when the
  thread wakes.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| Both cables up: found and reachable on both; a cable pulled leaves it found and reached on the other and on nothing at the first, service running, door answering; plugged in again, found and reached without a restart; what is said unchanged byte for byte; a failure on the way is a line in the service log, never a stopped service | `alo-agentd` lib `a_cable_pulled_and_plugged_in_again::tests::a_cable_pulled_is_a_network_no_longer_found_on_and_one_plugged_in_is_found_at_once` |
| The responders say when they move, and only then | `alo-agentd` lib `responding::tests::the_responders_say_when_they_move_and_only_then` |
| A network that will not take a socket is not a move | `alo-agentd` lib `responding::tests::a_network_that_will_not_take_a_socket_is_not_a_move` |
| A socket handed in never moves | `alo-agentd` lib `responding::tests::a_socket_handed_in_has_nothing_to_say_about_moving` |
| Hosting a workspace keeps the responders' voice | `alo-agentd` lib `responding::tests::hosting_a_workspace_keeps_saying_when_the_responders_move` |
| The pair: nothing said wakes nothing; said wakes and heard quiets; a full pair neither blocks nor fails | `alo-agentd` lib `told_of_a_move::tests::*` (three tests) |

## Verification

Platform: WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, from
`/mnt/c/dev/alo-os-claude` with this checkout's own `CARGO_TARGET_DIR`.

- The end-to-end test was run three times in a row and passed each time, in about
  7.4 s. The mutation run failed as described above.
- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps`: clean.
- `cargo test -p alo-agentd`: lib 423 passed and 16 ignored (the ignored ones are
  the inner halves of the namespace fixtures, which their outer tests run).
  Integration targets all passed: `a_machine_on_two_networks` 1,
  `a_question_is_bounded_by_the_kernel` 5, `a_turn_is_bounded_by_the_kernel` 1,
  `a_turn_is_refused_when_the_boundary_is_gone` 1, `two_doors_on_one_socket` 4,
  `what_a_machine_says_about_itself` 12. Doc-tests: 0.
- The full workspace suite was not run here. The supervisor runs it.

## Contract and quirks

- `docs/contracts/local-network-wire.md`: *A cable pulled, and plugged in again*
  (new, additive). Nothing on the wire changes; it says what a reader can rely on.
- `docs/quirks.md`: *a socket held to an interface that has gone stays open and
  silent, and a cable comes back as a new interface or the same one*.

## Limitations

- Measured over IPv4 only. Over IPv6 link-local, a link loses its address when set
  down, and a re-laid cable changes the interface that address belongs to. That is
  task 30, now written in the plan.
- A machine whose interfaces cannot be read at start follows nothing, as before.

## Proposed updates to the shared documents

- **CHANGELOG.md:** "An alo machine keeps up with its cables: unplugged from a
  network it stops being found there, and plugged back in it is found by the first
  machine that asks, with no restart. This fixes discovery staying silent on a
  network plugged in after the service started."
- **ROADMAP.md:** none. It is part of v0.5's *machines find each other with zero
  configuration*.
- **QUEUE.md / STATE.md:** local-network plan task 29 done. Task 30 (IPv6
  link-local across a pulled cable) is ready.
