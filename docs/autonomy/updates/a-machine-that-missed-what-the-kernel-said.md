# A machine that missed what the kernel said about its networks is still found on every one

**Date:** 2026-09-17
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 33)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository's owner
**Status:** ready for integration

## What changed

The service follows its networks by reading what the kernel says on a routing
socket. If the service is not reading when a burst of changes arrives, that socket
overflows. A dock with a dozen adapters, or a container host rebuilding its bridges,
is enough. The kernel then throws away everything after the overflow, including the
message that a cable was unplugged, and says `ENOBUFS` once. The service already
treated that as *any network may have gone*, but only a unit test on a hand-built
value checked it. This task measures it on a real kernel. It also makes the
overflow visible in the service log, which before said nothing.

- `crates/alo-agentd/src/a_machine_that_missed_what_the_kernel_said.rs` (new) — the
  fixture. **The far end** runs in its own user and network namespace.
  **Reception** is a separate process in a nested namespace, serving as
  `src/main.rs` does. Two `veth` cables join them, laid every time at the same
  numbers and hardware addresses: `cable0`/`far0` at 40/41 carrying IPv4, and
  `cable1`/`far1` at 44/45 carrying link-local IPv6 only.
  1. Both cables are laid. The far end finds reception on each cable and reaches
     its port on each. The kernel's drop count for reception's routing socket is 0.
  2. Reception is held still with `SIGSTOP`, and the fixture waits until every
     thread is reported stopped. Batches of 64 `veth` pairs are made and deleted in
     reception's network until the kernel's drop count rises (164 after one batch
     on this machine). Both cables are then deleted and laid again identically. The
     link-local cable's numbers, hardware addresses and `fe80::` addresses are
     asserted equal to before, with duplicate address detection finished. The drop
     count must rise again (180), which proves the messages about the cables were
     dropped too. Neither re-laid interface is in its discovery group.
  3. Reception is let go. Once its service lists `40` for answering, listening and
     joining over IPv4 and `44` for joining over link-local, **and** the kernel
     lists both interfaces in their groups, the far end asks. Its **first** question
     on each cable is answered with the same bytes as in step 1, the port is
     reached on each cable, and the person's door answers.
  4. The outer test checks that the service log carries the `DROPPED` line
     **exactly once**, and that reception stops cleanly when told to.
- `crates/alo-agentd/src/interfaces_that_went.rs` — `Went` now tells messages the
  kernel **dropped** (`lost`, new `was_dropped`) from a message it **could not
  read**. Both still mean any interface may have gone. New unit test
  `only_messages_the_kernel_dropped_are_a_round_that_was_dropped`, and the existing
  lost/unreadable test is adjusted.
- `crates/alo-agentd/src/joining.rs` — `DROPPED` (the log line) and
  `said_if_dropped`, called once per round in `Joining::changed`. A routing socket
  that could not be read at all now becomes `Went::any_of_them()` rather than
  `lost()`, because it already has its own log line. New unit test
  `a_round_the_kernel_dropped_messages_in_says_so_once_and_no_other_round_does`.
  The module documentation is updated.
- `crates/alo-agentd/src/lib.rs` — the fixture is registered (test-only, Linux).
- `docs/quirks.md` — new entry: *a routing socket nobody reads overflows quickly,
  says so once, and drops everything after, deletions included*.
- `docs/contracts/local-network-wire.md` — the existing bullet *Where a machine
  cannot tell what went* is extended, additively. Nothing on the wire changed.
- `docs/autonomy/v0-5-the-local-network-plan.md` — task 33 is marked done, and task
  34 is written.

**Change description, for the changelog:** When many network changes happen at once
— a dock bringing up its adapters, or containers rebuilding their bridges — while
alo OS is busy, Linux may drop some of what it says about them. alo OS now shows,
on a real kernel, that it is still found and reachable on every network afterwards,
over IPv4 and over link-local IPv6, with no restart. The service log records once
that messages were dropped.

## Decisions

- **How the overflow is made certain.** The criterion asked for the kernel's own
  drop count, and that count is read. The row in `/proc/<pid>/net/netlink` is
  chosen by the **socket inode** among reception's open descriptors and by the
  service's groups (`00000111`), with exactly one such row required. A count read
  off some other socket would prove nothing. The count is read twice: after the
  burst, and after the cables are re-laid. The second reading shows that the
  cables' own messages were dropped, not just queued behind the burst. The burst is
  `ip -batch` in batches until drops appear, capped at 50 batches. If the cap is
  reached, the test fails; it is never skipped.
- **Order inside the held-still window.** Burst first, then the cables. Re-laying
  the cables first would queue their `RTM_DELLINK` before the overflow, and the
  service would read it. The test would then measure task 31 again, not the loss.
- **Both cables come back identical.** If the numbers or addresses changed, a plain
  comparison of dumps would move the networks, and the test could pass without the
  reading of a dropped round. With identical cables, only `Went::lost` can make the
  service join again, and the mutation run confirms that.
- **Why a log line, and why only for drops.** The criterion requires the log to say
  once what was lost. The kernel reports an overflow once, as `ENOBUFS`, so one
  line per round in which it was reported is one line per overflow. A datagram that
  cannot be parsed is not an event on the machine, so it gets no such line. A
  socket that cannot be read at all already has its own line. Keeping `lost`
  separate from *unreadable* in `Went` is what makes that possible.
- **No larger receive buffer**, as the constraint says: a bigger buffer only
  changes how big a burst has to be to overflow it.
- **What "listened on" means over link-local.** `crate::listeners` holds one
  listener per network with an IPv4 address. Link-local IPv6 is reached through
  another socket, so the fixture does not ask for a link-local listener. It checks
  that the port is actually **reached** over `fe80::…%45`.
- **The refusal paths.** Tested: with `Went::lost` read as nothing having gone, the
  machine is not found (mutation run below). A round with nothing dropped, a round
  that could not be read, and a message cut short never say `DROPPED`, while a
  dropped round says it exactly once (unit tests). The service is not stopped by
  the overflow (the fixture).

## Acceptance criteria

| Criterion | Test |
|---|---|
| Socket overflowed, as the kernel's drop count shows; both cables re-laid identically; first question on each cable answered and port reached; same bytes throughout; service log says what was lost once and the service keeps running | `alo-agentd` lib `a_machine_that_missed_what_the_kernel_said::tests::a_machine_that_missed_what_the_kernel_said_about_its_networks_is_still_found_on_every_one` |
| `DROPPED` said once for a dropped round, and never for any other round | `alo-agentd` lib `joining::tests::a_round_the_kernel_dropped_messages_in_says_so_once_and_no_other_round_does` |
| Dropped and unreadable are told apart, and both still mean any interface went | `alo-agentd` lib `interfaces_that_went::tests::only_messages_the_kernel_dropped_are_a_round_that_was_dropped` |
| With `Went::lost` read as nothing went, the test fails | mutation run, below |

## Verification

Platform: WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, run as root
against `/mnt/c/dev/alo-os-claude` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`.

- `cargo test -p alo-agentd --lib -- --exact a_machine_that_missed_what_the_kernel_said::tests::a_machine_that_missed_what_the_kernel_said_about_its_networks_is_still_found_on_every_one`: **passed** (5.2 s).
- **Mutation run.** `Went::lost` was changed so it no longer set `unknown`, which
  reads a dropped round as *nothing went*. The fixture then **failed** at step 3
  with *reception never followed its cables: answered, listened and joined over
  IPv4 and joined over link-local at `40 40 40 44`; 40 in the IPv4 group: false; 44
  in the IPv6 group: false*. By then the far end had already reported 164 drops
  after the burst and 180 after the re-laying, and the log had carried `DROPPED`
  once. The mutation was reverted.
- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps`: clean.
- `cargo test -p alo-agentd`: **passed** — 442 unit tests (24 ignored, the namespaced halves of fixtures), and every integration target.
- The full workspace suite was not run by the worker. The supervisor runs it.

## Remaining limitations

- Measured with `veth` cables in namespaces, on one kernel. A physical dock
  overflowing the socket on certified hardware is still owed to that machine.
- The drop counts (164, 180) are what this machine measured. The fixture requires
  only that the count rises, never a particular number.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** local-network task 33 done, with this report as evidence.
  Task 34 (*A dock with a dozen adapters is found on every one of them at once*)
  is ready.
- **ROADMAP.md:** no change; no v0.01 box moves.
