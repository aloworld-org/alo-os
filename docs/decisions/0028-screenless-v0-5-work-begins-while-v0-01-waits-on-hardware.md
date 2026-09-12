# ADR 0028 — v0.5's screenless work begins while v0.01 waits on hardware

**Status:** **ACCEPTED, 2026-09-12**, under the owner's standing delegation, on
the owner's instruction to keep building while the certified machine is bought.
**Date:** 2026-09-12
**Proposed by:** the v0.01 delivery workstream
**Context:** `CLAUDE.md` (*scope is gated: nothing gets built that isn't in
`docs/features.md` with a tier, inside the current phase*), `ROADMAP.md` (the
v0.01 exit gate), `docs/autonomy/v0-01-evidence.md`,
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md),
[ADR 0024](0024-what-a-person-signs-in-at.md), `docs/hardware.md`

## The question in one line

**May work on v0.5 promises start before v0.01's exit gate is passed, when
everything left in v0.01 is waiting on a machine and on a lane that is away?**
The constitution says scope is gated to the current phase. It says so to stop
a release convincing itself it is finished by starting the next one. That is
the harm to avoid, and it is avoidable without leaving two build lanes idle for
a week.

## What is true today

- **v0.01's remaining items cannot advance from this machine.** Two promises
  have no evidence and both need the certified laptop (*the GPU works on first
  boot*; *boots on one certified machine, firmware to sign-in*). The sign-in
  screen is `crates/alo-shell` and the desktop lane's, away until about
  2026-09-16; its two halves that are not drawing — the session opener and the
  greeter's logic — are built. The image boots in a VM on the development PC
  and the owner has watched it, twice.
- **The laptop is being bought.** `docs/hardware.md` carries the specification.
- **v0.5 has a block of promises that need no screen and no hardware**, and it
  is the deepest part of the product: the grant becoming a boundary the kernel
  imposes (ADR 0013), the kernel taught what a turn is (ADR 0015), the record
  becoming an observation rather than a claim, a boundary that cannot be applied
  refusing to run — and, in the other lane's territory, the provider and model
  rules: https-only unless local, test before saving, *unknown* never satisfies
  a region, weights a person brings themselves.
- **Two lanes with nothing to do is a cost**, and it is paid daily.

## The options

### A — Stop until the laptop arrives

Keeps the phase gate literally. Costs a week of two lanes, and the only thing
it protects against — v0.01 being called done because v0.5 started — is
protected by something stronger already: the evidence ledger, which a test
holds to `docs/features.md`, and the roadmap's two boxes, where *on the
machine* moves only for a machine.

### B — Start v0.5's screenless promises, and say so

The work that needs no screen and no hardware begins, in the same shape the
v0.01 plan used — *everything that needs no screen, built first* — on plans of
its own, partitioned by crate between the lanes exactly as before. Nothing
about v0.01 moves: no box in `ROADMAP.md`, no line in the ledger, no wording in
`docs/features.md`. A v0.5 item built this way is ticked *built* and never *on
the machine*.

### C — Have this lane draw the sign-in screen

Rejected. It is the fastest route to the demo and it puts a second editor on
`crates/alo-shell`, the desktop lane's, days before that lane returns. The
partition is what makes two lanes safe; tonight it failed twice on a shared
*number* and cost twenty minutes each time. Failing it on a *crate* costs the
work.

## The decision

**Option B.** With these terms, which are the whole of what makes it not a
drift:

- **v0.01 stays v0.01.** Its exit gate is unchanged, its ledger is unchanged,
  and the two promises that need the laptop are evidenced on the laptop or not
  at all.
- **Two plans, two partitions.** The kernel-boundary work
  (`alo-bounding`, `alo-bounding-kernel`, `alo-bounding-map`, `alo-boundaryd`,
  `alo-egress`) is lane A's, on `docs/autonomy/kernel-enforcement-plan.md`,
  which already exists for it. The provider and model rules (`alo-models`,
  `alo-choosing`, `alo-asking`, `alo-secrets`, `alo-telling`) are lane B's, on
  `docs/autonomy/v0-5-lane-b-plan.md`. Neither lane touches the other's crates.
- **Numbers are checked before they are taken.** Tonight two tasks and two
  decisions were numbered the same by two lanes an hour apart. Before a worker
  or a person writes the next task or decision: pull, list, then number.
- **When the laptop arrives, v0.01 comes first.** The lanes are stopped, the
  disk that booted in the VM is written to the machine, and the two remaining
  promises are measured before any v0.5 work resumes.

## Consequences

- Both lanes have a week of real work that ends in tests rather than in a
  screen nobody can look at yet.
- The evidence ledger gains no entries and loses none. `ROADMAP.md`'s v0.5
  section gains *built* ticks only.
- A reader six months from now sees v0.5 work dated before v0.01's hardware
  acceptance and finds this decision explaining why, rather than a phase gate
  quietly ignored.
