# A refusal that is about the work, not about what was left in a build directory

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **four refusals in one day were this; none of them were the work.**

A gate reported `no method named numbers found for struct Accounts` while the
method sat in the tree and had done for half an hour. Every narrower check
passed on the same files — `cargo check -p` and `--workspace`, clippy with
warnings denied, `cargo test -p` for both crates — and only `cargo build
--workspace` failed. `cargo build --workspace -v` named it: the crate was handed
a cached `.rmeta` from before the method existed, and Cargo held that unit fresh,
so no `rustc` ran for it at all. `cargo clean -p` the two crates and the whole
workspace built in eighteen seconds.

It happened **four times** on 2026-09-11, on four different crates. Twice it
parked finished work. Once it stopped a run outright. Once it reported
`alo-driving` broken when all thirty of its tests pass on the same commit. Each
one cost about an hour, and two of them nearly ended with correct work discarded
as defective.

**Why a worker cannot see it and the supervisor can.** A whole-workspace build
resolves features differently from `cargo build -p <crate>`, so the two are
*different units of the same crate*: the `-p` unit gets rebuilt correctly while
the workspace unit stays stale. That is exactly the gap between what a worker
runs on its own crates and what the supervisor runs on everything. The checkout
also lives on `/mnt/c`, where every source mtime crosses drvfs from Windows, and
Cargo's freshness test is known to be delicate on that ground.

So before anything is gated, the units for **the crates the handoff names** are
dropped — and only those. A whole `cargo clean` would throw away an afternoon of
compilation belonging to forty crates nobody touched, to pay off a tax that
costs seconds per task. Files outside `crates/` name no crate and are skipped.

**It is best effort on purpose.** A clean that cannot run is not a reason to
refuse to gate: the gates are the check, and this is only an attempt to make
their answer be about the work. Whatever it says is dropped.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a gate's refusal is now about the change rather than
about what a previous build left behind.
