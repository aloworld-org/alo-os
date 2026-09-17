# A link-local cable re-laid with the same hardware address between two readings is still joined

**Date:** 2026-09-17
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 32)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository's owner
**Status:** ready for integration

## What changed

A USB network adapter pulled out and pushed back in comes back as the same
interface in every way a machine can read: the same number, the same name, the
same hardware address, and so the same `fe80::` address. If that happens between
two of the service's readings of its interfaces, the readings match exactly. But
the kernel dropped the interface's discovery-group membership when the old one was
deleted. If the service only compared readings, it would count itself joined on a
network where nobody can find it until it restarts. Task 31 made the IPv6 joins read
the kernel's own *this interface was deleted* message, but only a unit test on a
hand-made list covered it. This task tests it on a real kernel.

- `crates/alo-agentd/src/a_link_local_cable_re_laid_with_its_hardware_address.rs`
  (new) — the fixture. **The studio** runs in a network namespace of its own, and
  **reception** runs as a separate process in a namespace nested inside the
  studio's. Both serve as `src/main.rs` does. One `veth` with link-local IPv6 only
  joins them, and it is laid every time at numbers 40/41 with hardware addresses
  `02:a1:0c:00:00:40`/`…41`.
  1. Laid: each finds the other over link-local with its interface, and the studio
     proposes and they pair.
  2. Reception is held still with `SIGSTOP`, and the cable is deleted. The studio
     finds nothing, and a proposal to reception is refused before anything is
     sent.
  3. The cable is laid again with the same arguments while reception is still
     held. The number, hardware address and link-local address at both ends are
     asserted equal to before. Both addresses have finished duplicate address
     detection before reception is let go. Reception's re-laid interface is not in
     `ff02::fb` even though its socket joined at that number, and nothing answers.
  4. Reception is let go with `SIGCONT`. Once its service is joined and the kernel
     lists the interface in the group, the studio's **first** question finds
     reception on the same interface at the same address. Both questions are
     answered with the same bytes as in step 1. The studio proposes and they pair
     on that interface.
- `crates/alo-agentd/src/lib.rs` — the fixture registered (test-only, Linux).
- `crates/alo-agentd/src/joining.rs` — rustdoc only: names the new fixture as the
  measurement behind the reading of what went.
- `docs/quirks.md` — new entry: *an adapter laid again with its hardware address
  is, in a dump, the interface that went*.
- `docs/contracts/local-network-wire.md` — one bullet added under *A cable deleted
  and laid again before the machine looks*.
- `docs/autonomy/v0-5-the-local-network-plan.md` — task 32 marked done; task 33
  written (below).

**No product code changed.** The behaviour task 31 added already holds it; this
change is the proof on a real kernel.

**Change description, for the changelog:** A network adapter unplugged and plugged
back in faster than alo OS looks at its networks — coming back with the same name,
hardware address and link-local address — is now shown on a real kernel to be joined
afresh. The other machine on that cable finds this one with its first question, and
the two can pair there, with no restart.

## Decisions

- **Which machine is held still.** Reception, the machine that is asked, because
  the criterion is about that machine's joins. The studio is not held and follows
  its own end in separate rounds (the fixture waits for it). So only reception is
  between two readings, and a pass cannot come from the studio's side.
- **How the ordering is made certain.** Task 31's method, plus one extra wait:
  `SIGSTOP`, then wait until every thread is reported stopped, delete, re-lay,
  **wait until both link-local addresses are no longer tentative**, then `SIGCONT`.
  Without that wait, reception's first dump could see the interface with no usable
  address yet. It would drop the network and join again on a later round, and the
  test would pass even without the reading it exists to measure.
- **How "followed the kernel" is read.** From `/proc/<pid>/net/igmp6`: only the
  service joins `ff02::fb` in reception's network, together with the service's own
  `Wire::joined`. Nothing is asked until both agree, and the next question is
  the one that must be answered.
- **Why the fixture is a unit-test module and not an integration test.** It needs
  `crate::looking`, `crate::serving` and `crate::testing`, like the task 30 and 31
  fixtures it builds on.
- **The refusal paths.** Tested inside the same fixture:
  - while reception is held with its cable gone, a proposal is refused with
    `NO_SUCH_MACHINE_ON_THE_NETWORK` and no pairing is left waiting;
  - while it is held with the cable back, nothing answers the studio, which also
    shows the ordering is real and not a race won.

## Where it fails without the reading

Mutation run, 2026-09-17: in `joining.rs::kept_and_gone`, the condition
`!went.includes(network.index())` was replaced by a condition that is always true,
so every network still reported stays joined. The fixture failed at step 4:

```
reception never followed its cable to 40: joined at `40`, and the interface is not in the discovery group
```

Reception counted itself joined at 40, and the kernel had no interface there in the
group. The studio's question could never reach it. The line was restored before
the gates below.

## Verification

Platform: WSL2 Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`, run from
`/mnt/c/dev/alo-os-claude` with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`. All executed on
2026-09-17:

- `cargo test -p alo-agentd --lib a_link_local_cable_re_laid_with_its_hardware_address`
  — 1 passed, 2 ignored (the inner halves, run by the outer test), 23.7 s.
- The mutation run above — failed as described.
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-agentd --no-deps` — clean.
- `cargo test -p alo-agentd` — every target passed: 439 passed / 22 ignored in the
  library tests, plus all integration tests and doctests.

Not run by this worker: the full workspace suite, which the supervisor runs.

**Host note:** `C:` had about 1.8 GB free during this task, and WSL failed to start
once (`Wsl/Service/CreateInstance/E_FAIL`) before starting on retry. Looking for
room, I ran a delete of this checkout's own test binaries older than a day
(it matched none) and `fstrim /` inside the distro (it freed nothing on `C:`).
Nothing else was touched, and nothing was actually deleted. The owner may want to free space before the workspace suite
runs (see the SIGBUS/IO-error symptoms written up for task 31).

## Limitations

- The kernel behaviour is measured on `veth`, not on a physical USB adapter.
  `veth` is where the index and hardware address can be set exactly, and the kernel
  path (`RTM_DELLINK`, membership dropped, EUI-64 address) is the same one. A
  physical re-plug belongs to hardware acceptance on the certified machine.
- `addr_gen_mode` is the namespace default (EUI-64). Under `stable-privacy` the
  address is still identical for the same name and prefix. That is not measured
  here, and the reading does not depend on it.

## The second pass: a program busy being written

The first handover was refused twice by the workspace suite, not on this work but
on `alo-updating`'s `with_no_proxy_the_program_is_told_there_is_none`, which
could not start its program: `Text file busy (os error 26)`. That test and
`the_program_the_base_starts_really_receives_the_proxy` each write a small shell
program and start it, on two threads of one test process. While one thread still
holds its program open for writing, a child the other thread forks inherits that
descriptor until the child starts its own program, and in that moment the kernel
refuses to start the first program (`ETXTBSY`). It shows only when the machine is
loaded, which the whole workspace suite is.

The fix is in the test file alone: both tests take one lock
(`STARTING_A_PROGRAM`) for as long as they write and start a program, so no fork
can copy a program that is still being written. The lock guards no data, so a
poisoned lock is taken anyway rather than failing the next test. No product code
changed, and no test was removed or weakened. `cargo test -p alo-updating` and
`cargo test -p alo-agentd` pass, and the proxy test binary passed eight runs in a
row.

## Proposed updates for the integration owner

- **CHANGELOG.md:** the change description above.
- **ROADMAP.md:** none. No box moves; this is evidence under v0.5's *machines find
  each other with zero configuration*.
- **QUEUE.md / STATE.md:** task 32 done, with this report. Task 33, *A machine that
  missed what the kernel said about its networks is still found on every one*, is
  ready: it covers the routing socket overflowing (`ENOBUFS`) on a real kernel,
  which is so far held only by `interfaces_that_went`'s unit test.
