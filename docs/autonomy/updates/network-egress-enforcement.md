# Network egress enforcement

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-bounding-map`, `alo-bounding-kernel`, `alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Socket attribution and default-deny for a bound turn
- Status: ready for integration

## What changed and why

The v0.01 requirement named in the egress indicator's own remaining half — *the
enforcement at the network boundary, without which all of this describes only
the code that asked* — now exists. A fifth BPF LSM hook, `socket_connect`,
decides where a bound turn may connect.

The sentence it enforces:

> **A turn opens no socket unless the person has been shown that it is about
> to.**

### The correction that shaped it

The policy task had said a turn *with a departure written* may open sockets.
**That was wrong and would have shipped a hole**: one displayed departure would
have become permission for every address there is, and somebody shown *asking
alo, in Frankfurt* would have authorised the internet. A departure is now **one
address and one port**, and there is a test whose whole job is that a second
destination is refused while the first is still permitted.

### The four parts, specified

- **Destination binding** — a departure permits one address and one port.
  Another address is refused; so is another port on the same address.
- **Lifetime** — the turn. Destinations live in the same map entry as the places
  a turn may reach, so they are gone when the turn ends, by the revocation that
  already existed and is already tested.
- **Withdrawal** — the daemon rewrites the entry without that destination. It
  takes effect on the next `connect`, because the programme reads the map on
  every call and holds nothing between them. Tested.
- **Connection reuse** — `socket_connect` fires when a connection is *made*. An
  established socket is **not** re-checked, so a destination withdrawn while a
  connection to it is open stays reachable over that connection until it closes.
  A gap, stated rather than discovered.

### Coverage, and the gaps by name

| | |
|---|---|
| TCP over IPv4 | enforced |
| TCP over IPv6 | enforced — the entry holds 128-bit addresses |
| UDP with `connect` | enforced; it reaches the same hook |
| **UDP with `sendto`** | **not enforced.** An unconnected datagram never reaches `socket_connect`. It would need `socket_sendmsg`, a hook on every message rather than every connection |
| **A socket already open, or inherited into a turn** | **not enforced** — the same class as already-open descriptors |
| Address families that are not network addresses | **allowed**, deliberately: a Unix socket is not egress, and refusing it would enforce something no policy claims |
| A family whose address cannot be read | **refused** — a destination that cannot be checked |
| **Loopback** | **not checked** — see below |

### Loopback, and a forward reference in `docs/quirks.md` that this corrects

`127.0.0.0/8` and `::1` are allowed without having been shown. ADR 0007 makes a
model on this machine the default, and `alo-egress`' `Leaving::asking` does not
call a question answered here a departure at all — so nothing is ever shown for
it and nothing would ever be written. Refusing it would break the ordinary case
the product is built around.

`docs/quirks.md`'s *Loopback is taken at face value, and one thing can therefore
lie* says a proxy listening on loopback would be believed by every type in this
repository, and that **the place it is caught is egress enforcement at the
network boundary**. That forward reference pointed here and **this is not that
place**: the enforcement is turn-scoped, so a proxy somebody else started is not
a turn and its own outward connection passes untouched. The quirk is corrected
rather than left reading as answered.

## Decisions and approvals

**No approval is requested.** The task named three things that would have needed
one and none happened:

- **No third map.** Destinations went into the existing `BOUNDS` value, whose
  layout `alo-bounding-map` already owns. A third map would have changed what
  `the_program_has_nowhere_to_write_what_it_sees` asserts.
- **No hook that sees message contents.** `socket_connect` sees an address.
- **`Departing` untouched.** The userspace provider-and-region policy is exactly
  what it was.

Nothing widened: no grant covers more, no agent capability was added, and a turn
can reach exactly the destinations somebody was shown — which is fewer than
before, never more.

**Cgroup attribution is not an audit record and this does not claim it is.** The
programme writes nothing down. What a person is shown is what `alo-egress` shows
them. Kernel-sourced recording remains the separate architectural decision
recorded in the plan: unscheduled, unbuilt, and not to be started without an
ADR.

### The guard that fired, and why it was not widened

`the_program_has_nowhere_to_write_what_it_sees` asserts the programme's maps are
exactly `["BOUNDS", "FIELDS"]`. It failed: a `.rodata.cst32` section had appeared,
which the compiler emits for 128-bit constants.

There was an honest case for adding it to the list — `.rodata` is constant data,
frozen read-only by the verifier at load, and genuinely not somewhere a programme
can put what it saw. **That case was not taken.** The constants were written so
that none is emitted, and the guard passes with its list unchanged. A tripwire
widened for a good reason is still a tripwire that has moved.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, stable Rust 1.98.0 and the pinned
`nightly-2026-06-01` for the BPF half, own `CARGO_TARGET_DIR`.

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero.
- `cargo test --workspace` — **114 test binaries, all ok, no failures** (was 112).
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — clean.
- BPF target, pinned nightly: `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  — clean.

Against the real loaded programme, `the_kernel_refuses_a_departure.rs`, seven
cases: default-deny; the shown destination permitted; **a second destination
refused while the first is permitted**; another port refused; withdrawal
effective next connect; loopback unchecked; a non-turn unaffected. Plus
`what_a_turn_can_reach_on_the_network.rs`, whose assertion this change inverted:
a bound turn is now refused a file *and* a destination nobody showed it, with
the file refusal as the control that proves the boundary was applied at all.

**Every existing protection retested**: the four filesystem hooks, the loader's
attach/refuse/take-away tests, `the_boundary_decides_and_forgets`, and
`alo-agentd`'s whole-journey turn — all pass.

**Shared-machine safety.** No test reaches a network. Addresses that must be
*checked* cannot be loopback, and must not be reachable either, so they are from
`192.0.2.0/24`, which RFC 5737 reserves for documentation and nothing routes.
What is asserted is what the kernel said before a packet was sent: `EACCES` for a
refusal, anything else for a permission. Loopback tests use a listener the test
owns on a port the OS chose. No host-wide networking changed, no pin removed, no
other worker's process stopped.

**Windows:** not run and not applicable — `alo-bounding` is
`#![cfg(target_os = "linux")]` and compiles to nothing there. `alo-bounding-map`
is portable and its tests run in the workspace suite on any host; only Linux was
exercised here.

**WSL is development evidence and never certified-hardware acceptance.**

## Remaining gaps and hardware obligations

- **UDP `sendto`, already-open and inherited sockets** — named above, unbuilt.
- **The loopback proxy** — not closed by turn-scoped enforcement, and the quirk
  corrected to say so.
- **Nothing writes destinations yet in production.** The daemon code that signs
  somebody in, fetches a model or checks for an update *does not exist* —
  `ROADMAP.md` says so on the same line as this requirement. What is delivered is
  the enforcement and the interface a daemon will use; the wiring is a task for
  when there is a daemon to wire.
- Everything in the plan's *Incomplete* table with its release named; the v0.5
  items are unbuilt and are not this release's.
- A failure path still not induced: a genuine mid-attach kernel refusal.
- **All physical acceptance.** No *On the machine* box is affected.

## Proposed shared-document updates

**CHANGELOG.md**, under Unreleased:

> **An agent can no longer connect anywhere you were not shown.** The indicator
> has always told you when something was about to leave your machine; now the
> machine itself refuses a connection you were never shown. Being shown one
> destination permits that one — not another address, and not another port on
> the same one — and it lasts for that piece of work and no longer. A model
> running on your own computer is unaffected, because nothing leaves for it to
> show you. Everything that is not your agent is untouched.

**ROADMAP.md** — the *Egress indicator, and no telemetry* line's **On the
machine** clause lists three things still owed; **the third, "the enforcement at
the network boundary", is now built** and can be struck from that clause. The
box stays unticked: the indicator surface and the daemon code are still owed,
and physical acceptance with them.

**docs/autonomy/QUEUE.md** — no new item; this workstream's list is the plan.

**docs/autonomy/STATE.md** — reference this report. Two facts worth carrying:
the boundary is now five hooks and six pins; and `docs/quirks.md`'s loopback
entry pointed at network-boundary enforcement as the place its hole would be
caught, which turn-scoped enforcement does not do — the correction is in
`deciding.rs` and in the quirk.
