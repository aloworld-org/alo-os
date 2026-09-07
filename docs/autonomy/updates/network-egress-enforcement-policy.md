# Network egress enforcement policy

- Date: 2026-09-07
- Workstream: kernel enforcement (`alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Establishing the network egress policy the kernel can actually enforce
- Status: ready for integration

## What changed and why

No enforcement, and deliberately none: this is the policy statement that has to
exist before a hook is chosen, because the obvious reading of *enforce the
egress policy in the kernel* does not survive contact with the kernel.

The statement is in `crates/alo-bounding/src/lib.rs`, under **What this boundary
can decide about the network, and what it cannot**, and is summarised in
`docs/autonomy/kernel-enforcement-plan.md` beside the task it closes.

### The finding

**`alo-egress`'s policy cannot be enforced in the kernel.** It decides by
*provider* and *region* — a question may be answered in the building, on this
machine, or in a named part of the world. A programme on a socket sees a control
group, a protocol and an address. A provider is a name somebody resolves through
DNS; a region is a fact about a company. Neither is visible where the
enforcement would sit.

A kernel-side approximation — an address list, a guess from a hostname seen
earlier — would be a *second* policy, disagreeing with the first in ways nobody
could predict, and the disagreement would be invisible until it mattered. So the
statement says plainly: **the kernel is not the policy engine and this crate will
not make it one.**

### What is enforceable, and it is stronger

What ADR 0013 actually names is *which sockets may be opened, and attribution of
every one to the turn that caused it.* Expressed as a sentence:

> **A turn opens no socket unless the person has been shown that it is about
> to.**

`alo-egress` already makes that a thing with a type: an `alo_egress::Departing`
cannot be obtained without `Indicator::beginning` having asked the policy and
shown the person. Today it is a promise the daemon keeps. The enforcement is the
same promise with the kernel behind it — the daemon writes a turn's permission to
leave where a programme can read it, exactly as it writes the places a turn may
reach.

Three consequences, and they are the whole policy:

- **Default deny, and for turns only.** A bound turn with no departure written
  opens no socket. Every other process — a person's browser, their mail client,
  the service's own errands, which are not turns — is unaffected, exactly as for
  files.
- **Attribution is the same question as permission.** The kernel is asked *which
  turn is this*, and it already knows: the control group. No second mechanism.
- **The programme still writes nothing down.** ADR 0015's *the LSM decides and
  forgets* is not relaxed for sockets.

## Acceptance criteria and actual verification

Acceptance was a written statement naming what the kernel can decide, what it
cannot, and which half of law 1 each layer keeps, with an explicit statement that
this does not make the kernel the policy engine. All four are in the text.

Verified on Ubuntu/WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`,
by the supervisor, which records what each gate said:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps`
- the BPF target's `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  on the pinned nightly

This change is documentation inside a crate, so the tests it must not break are
every test in the workspace, and they were all run.

**Windows:** not run. `alo-bounding` is `#![cfg(target_os = "linux")]` and
compiles to nothing on Windows, so there is no Windows behaviour for this change
to have. Said rather than implied.

**WSL is development evidence and never certified-hardware acceptance.**

## Decisions and approvals

**No approval was needed and none is requested.** No grant widens, no agent
capability is added, `Departing` keeps its meaning, and a turn can still open
exactly the sockets the person was shown. The change only names what a later one
may and may not do.

The decision recorded is the negative one: **the provider-and-region policy stays
in userspace**, and any future proposal to approximate it in the kernel has to
argue against this text first.

## Remaining gaps and hardware obligations

- Nothing is enforced yet. The reproduction is task 2 of the plan and the
  programme is task 3.
- Everything else in the plan's *Incomplete* table, with the release that owns
  each.
- All physical acceptance. No *On the machine* box is affected.

## Proposed shared-document updates

**CHANGELOG.md** — nothing; no user-visible behaviour changed.

**ROADMAP.md** — no box changes. When the v0.01 egress line is next read, its
remaining *On the machine* clause names network-boundary enforcement, and the
policy for it is now written down rather than assumed.

**docs/autonomy/QUEUE.md** — no new item; this workstream's list is
`docs/autonomy/kernel-enforcement-plan.md`.

**docs/autonomy/STATE.md** — reference this report. The fact worth carrying: the
egress policy alo OS sells is a userspace policy and cannot be enforced in the
kernel, so what the kernel will enforce is *nothing leaves a turn that the person
was not shown*, which is the same guarantee from the other side.
