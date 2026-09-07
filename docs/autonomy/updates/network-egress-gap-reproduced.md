# Network egress gap reproduced

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Reproducing unrestricted network access from inside a bound turn
- Status: ready for integration

## What changed and why

One test, and no enforcement. `crates/alo-bounding/tests/what_a_turn_can_reach_on_the_network.rs`
puts the gap on a running kernel with the real programme loaded, so that the
work which closes it has something that fails when it lands.

The boundary watches four things a turn does to files and **nothing** it does to
a socket. That is the v0.01 requirement named in the egress indicator's own
remaining half — *the enforcement at the network boundary, without which all of
this describes only the code that asked* — and the policy for it was written in
the previous task.

## What the kernel said

One turn, bound to one folder:

| The same child, in the same turn | Kernel |
|---|---|
| opens a file nobody granted | **`EACCES`** |
| opens a socket to an address nobody granted | **allowed** |

**The file refusal is the control and it is not decoration.** *A bound turn
opened a socket* is also exactly what a turn with **no** boundary would do, so a
fixture that quietly failed to bind anything would have reported the gap it was
written to find. The test throws the socket result away unless the file was
refused first.

The test asserts what this machine does **today**. When task 3 of the plan lands
it will fail, and its message says where to come and what to change — the same
arrangement that made the rename gap tell the next contributor it had closed.

## Shared-machine safety

**Nothing here reaches a network.** The address is a `TcpListener` the test owns,
on `127.0.0.1`, on a port the operating system chose. No name is resolved, no
packet leaves the machine, and no host-wide networking is touched. The connect
has a five-second timeout so a hung attempt fails rather than hanging a run.

The test takes the existing `on_this_kernel::one_at_a_time()` lock and pins under
a path named for its own process, as the six existing files in that directory do.
No other worker's process was stopped, no pin removed, nothing unloaded.

## Acceptance criteria and actual verification

Acceptance was a test that binds a turn, has the child attempt an outbound
connection to an address nothing granted, and records what the kernel does today.
That is what it does, with the control above.

Verified on Ubuntu/WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`,
by the supervisor:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — every existing filesystem and loader protection
  retested alongside the new test
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps`
- the BPF target's `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  on the pinned nightly

**Windows:** not run. `alo-bounding` is `#![cfg(target_os = "linux")]` and
compiles to nothing there, so this test has no Windows behaviour. Said rather
than implied.

**WSL is development evidence and never certified-hardware acceptance.**

## Decisions and approvals

No approval needed: a test that records existing behaviour changes no grant, no
capability, no contract and no ADR.

One decision worth recording: **the assertion is written as what the machine
does now, not as what it should do.** An `#[ignore]`d aspirational test would sit
green and silent; this one is the tripwire that fires when the gap closes.

## Remaining gaps and hardware obligations

- The gap itself. Task 3 of the plan is the programme that closes it.
- Everything else in the plan's *Incomplete* table, with its release named.
- A failure path still not induced: a genuine mid-attach kernel refusal.
- All physical acceptance; no *On the machine* box is affected.

## Proposed shared-document updates

**CHANGELOG.md** — nothing; no user-visible behaviour changed.

**ROADMAP.md** — no box changes. The v0.01 egress line's remaining clause now has
a measurement behind it rather than an assumption.

**docs/autonomy/QUEUE.md** — no new item; this workstream's list is the plan.

**docs/autonomy/STATE.md** — reference this report. The fact worth carrying: a
bound turn is refused a file and not refused a socket, measured on a running
kernel, and there is now a test that fails the day that changes.
