# A dock with a dozen adapters is found on every one of them at once

**Date:** 2026-09-17
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 34)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository's owner
**Status:** ready for integration

## What changed

Until now, every fixture put a machine on one or two cables. A docking station or a
lab switch brings a dozen networks up in one moment. The service then holds one
responder, one listener and one IPv6 membership for each of them. If a socket, a
descriptor or a join ran out part of the way through, the machine would be found on
eleven networks of twelve, and nobody would know which one it missed. This task
measures that on a real kernel. It also measures what happens when one network does
fail.

- `crates/alo-agentd/src/a_dock_with_a_dozen_adapters.rs` (new): the fixture.
  **The far end** runs in its own user and network namespace. **Reception** runs as
  a separate process in a nested namespace and serves as `src/main.rs` does.
  1. **Twelve cables in one burst.** Reception is held still with `SIGSTOP`, and
     the fixture waits until the kernel reports every thread stopped. Twelve `veth`
     cables are then laid: `cable0`–`cable11` at 40–51 on reception's side and
     `far0`–`far11` at 60–71 on the far end's, each with a hardware address.
     `cable0`–`cable5` carry IPv4 (`10.74.N.0/24`) and `cable6`–`cable11` carry
     link-local IPv6 only. All pairs are made in one `ip -batch`, and reception's
     ends are addressed and brought up in a second. Every link-local address
     finishes duplicate address detection, and nothing is in a discovery group,
     before reception is let go. The fixture then waits until the service holds
     every cable and the kernel lists every cable in its groups. After that, the
     **first** question on each of the twelve cables is answered, all with the same
     bytes, and the port is reached on each.
  2. **Half unplugged in one batch.** Cables 3, 4 and 5 (IPv4) and 9, 10 and 11
     (link-local) are deleted in one `ip -batch`. Reception is found and reached on
     the other six, holds nothing at the six that went, and its door answers.
  3. **One network that fails.** `cable12` (IPv4) is laid while reception is held
     still. Before reception is let go, *the squatter* (the test binary again,
     run inside reception's network with `nsenter`) listens on the wire port held
     to that interface, without `SO_REUSEADDR`. After reception follows the kernel,
     it answers and joins on `cable12` but does not listen there. The first question
     there is answered with the same bytes, and the port is not reached there. It
     is still reached on every other IPv4 cable, and the door answers.
  4. **The outer test** checks that the service log names `cable12` in the line
     *the port presence advertises could not be bound on cable12 (10.74.12.1): …*,
     and that no per-network failure line names any other network. It checks that
     the log says `DROPPED` exactly when the kernel's drop count for reception's
     routing socket is above zero, and that reception stops cleanly when told to.
- `crates/alo-agentd/src/lib.rs`: the fixture is registered (test-only, Linux).
- `docs/contracts/local-network-wire.md`: new section *Many networks at once*,
  additive. Nothing on the wire changed.
- `docs/quirks.md`: new entry covering what the kernel did with a dozen networks in
  one burst, and how a port taken on one interface is refused there alone.
- `docs/autonomy/v0-5-the-local-network-plan.md`: task 34 marked done. Task 35 is
  written: *A port another program let go of on one network is listened on there
  again*.

**No product code changed.** The service already had what the task asks for. There
is one socket per network, no limit of its own, and a line naming the network for
every per-network failure (`crate::responding`, `crate::listeners`,
`crate::networks::joined_on`). What was missing was a measurement holding it to
that on a real kernel.

**Change description, for the changelog:** A computer running alo OS on a docking
station with many network adapters is now shown, on a real Linux kernel, to be
found and reachable on every one of a dozen networks brought up at once. When half
of them are unplugged it stays reachable on the rest, with no restart. If it cannot
serve one network, for example because another program holds its port there, its
log names that network, and it keeps working everywhere else.

## Decisions

- **How "in one burst" is made certain.** Reception is held still while every
  cable is laid, and the fixture waits for duplicate address detection before
  letting it go. The first thing reception reads is therefore all twelve networks
  together, not twelve separate changes.
- **Whether the burst overflowed is read, not required.** The kernel's `Drops`
  count for reception's routing socket is read from the one row in
  `/proc/<pid>/net/netlink` whose inode is among reception's descriptors (as in task
  33). **On this machine it was 0**: twelve cables laid in two batches do not fill a
  212992-byte receive buffer. Task 33 already measured the overflow on purpose, so
  this fixture does not force one. It passes either way, and it holds the service
  log to whatever the kernel counted.
- **How a network is made to fail.** A real refusal, not a fake one: another
  program holds the port on one interface, which is what a taken port looks like on
  a real machine. The listener was chosen over the responder because the responder
  shares its port (`SO_REUSEADDR`) by design. Making the listener fail also leaves
  the machine *found* on that network while unreachable there, which shows the
  failure stays on that one listener and spreads to nothing else. It is laid while
  reception is held still, so the squatter is certain to hold the port before the
  service tries to.
- **"Never a silent miss" is checked from both sides.** The fixture requires the
  service to hold exactly the expected networks, compared as strings for answering,
  listening, IPv4 joining and IPv6 joining. After that it requires a door answer,
  which is handled on the service's own round, so the round that followed the
  cables has finished. Only then is the state re-read. The outer test checks that
  every per-network failure line names only `cable12`.
- **Asking on six IPv4 cables from one network.** Each question is sent with
  `IP_MULTICAST_IF` set to the far end's address on that cable. The route alone
  would pick one of the six interfaces.
- **No limit.** None was imposed, and none bit at thirteen networks (see
  `docs/quirks.md`: one membership per IPv4 socket, so `igmp_max_memberships`, a
  per-socket count, never comes near; the IPv6 socket's twelve memberships sit well
  within `optmem_max`).
- **What this found and did not fix.** A listener refused because its port is taken
  is tried again only when the kernel says a network changed. A program letting go
  of the port is not such a change, so the log line's *until it can be* is not kept.
  Fixing that means choosing how the service learns the port is free, with no
  polling, which is a design decision outside this task's acceptance. It is written
  as task 35.

## Acceptance criteria

| Criterion | Test |
|---|---|
| Twelve cables laid in one batch while held still; once followed (per interface from `igmp`/`igmp6`), the first question on every cable answered, and the port reached on every IPv4 cable (and every link-local one) | `alo-agentd` lib `a_dock_with_a_dozen_adapters::tests::a_dock_with_a_dozen_adapters_is_found_on_every_one_of_them_at_once` |
| Every answer the same bytes | same test |
| Half the cables deleted in one batch: found on the other half, nothing at the deleted ones, service running | same test |
| Whether the burst overflowed read from the kernel's drop count, and the service log held to it | same test (0 drops on this machine) |
| A failure on one network is a line naming it, never a stopped service, never a silent miss | same test, plus the two mutation runs below |

## Verification

Platform: WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, run as root
against `/mnt/c/dev/alo-os-claude` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`.

- `cargo test -p alo-agentd --lib -- --exact a_dock_with_a_dozen_adapters::tests::a_dock_with_a_dozen_adapters_is_found_on_every_one_of_them_at_once`:
  **passed** (7.5 s). Far end's output: *twelve cables laid in one burst while it was
  held still: 0 messages dropped for its routing socket*.
- **Mutation run 1**: `crate::listeners`' per-network bind failure line silenced.
  The fixture **failed** with *the service log never named the network its port
  could not be bound on*. Reverted.
- **Mutation run 2**: `crate::responding::answer_on` skipped the last network it
  was given. The fixture **failed** with *reception never followed its cables: it
  holds `cable0,…,cable4|…` … and the kernel does not list ["cable5"] in their
  groups*. Reverted.
- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps`: clean.
- `cargo test -p alo-agentd`: **passed**: 443 unit tests (27 ignored, the
  namespaced halves of fixtures), and every integration target.
- Every acceptance criterion above is covered by the one fixture, so the handoff
  names that one test as its evidence. The two mutation runs are what show its
  refusal paths bite.
- The worker did not run the full workspace suite. The supervisor runs it.

## Remaining limitations

- Measured with `veth` cables in namespaces, on one kernel. A physical dock on
  certified hardware is still owed to that machine.
- Thirteen networks is the most measured. No limit is imposed, and larger counts
  are not claimed.
- A port refused on one network is not tried again once it becomes free, unless a
  network changes. That is task 35.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** local-network task 34 done, with this report as evidence.
  Task 35 (*A port another program let go of on one network is listened on there
  again*) is ready.
- **ROADMAP.md:** no change; no v0.01 box moves.
