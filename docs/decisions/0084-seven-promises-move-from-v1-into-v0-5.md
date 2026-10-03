# ADR 0084 — Twelve promises move into v0.5

**Status:** **accepted, 2026-10-03**, by the owner. Asked whether v1 could fold
into v0.5 except printing, keeping what a developer needs every day and what a
demonstration rests on.

## The whole release does not move, and the reason is a measurement

v0.5 holds 31 roadmap promises, **none complete**, and **28 of them waiting only
on a machine**. The constraint on that release is a certified laptop nobody has
plugged in, not scope. Folding 35 more promises into it changes what the release
is called and not when any of it works.

And v1 is mostly not demonstration material. Of its 35 promises, roughly 18 are
organisation and compliance work — identity providers, SIEM export, key escrow,
fleet policy and enrolment, update rings, certification groundwork, audit,
support and SLA. A developer does not meet them in a day's work and none of them
raises an eyebrow in a demonstration. That is what the release's own name says:
*an organisation can buy it*.

So seven move, and the rest stay.

## What moves, and why each one

| Promise | Why it belongs in the release a person uses |
|---|---|
| **"Make this machine like my old one"** | the strongest demonstration in the whole roadmap: a replacement machine that is actually yours |
| **Ask for appearance changes** — *use dark after six* | the agent doing something ordinary, instantly, at the lowest possible stakes |
| **Portals: USB, registered shortcuts, dynamic launchers, remote desktop** | **USB not working is noticed within an hour** of ordinary use |
| **The shell in all 24 official EU languages** | the European argument, visible in one screen |
| **Guided fine-tune, the dataset never leaving the machine** | the sovereignty claim made concrete rather than stated |
| **Cross-machine agent work, under a grant made on the target machine** | genuinely novel; nothing else on a desktop looks like it |
| **Multi-user on one machine, with per-person grants** | an ordinary expectation of any operating system |

Printing stays at v1, as the owner asked.

## An eighth was proposed and was already here

*Devices: audio with mid-call switching, Bluetooth, camera, microphone* reads
`[v1]` in `ROADMAP.md` and `[v0.5]` in `docs/features.md`, at three separate
lines. `CLAUDE.md` binds building to `docs/features.md`, so the roadmap was the
stale one. Corrected there rather than moved.

## This makes the release further from done, deliberately

Before: 89 `[v0.5]` promises in `docs/features.md`. After: 96. The seven added
are, as far as anybody has checked, unbuilt — so a release that was 28-waiting-on-hardware
now also carries seven that are waiting on work. **That is the intended trade.**
The alternative was a demonstration with nothing in it a person recognises.

## And five canvas promises, decided the same day

Asked afterwards whether the canvas work above this release could come with
them, the owner said all five. They are a different argument from the seven: not
*what a demonstration rests on* but **what the canvas is**, which the owner had
already ruled on once — *the complete agreed canvas experience belongs in v0.01;
do not defer parts of it to v1.1*.

| Promise | Was | What kind of move this is |
|---|---|---|
| *A Place remembers time* | `[v1.1]` | a widening. The only canvas promise above this release, named rather than moved by the lane that found it because it is not among the five completion requirements and rests on undo's snapshots |
| *Every screen is a view onto the canvas* | `[v1]` | a widening |
| *A panel out of view costs nothing* | `[v1]` | a widening, and a cost property rather than a feature — what shows it is a measurement |
| *Every canvas also answers as a list* | `[v1]` | **a correction.** Built at v0.5: task 7 of `docs/autonomy/the-smallest-canvas-worth-showing.md`, *Done, 2026-09-29, all three thirds*, six tests in `crates/alo-shell/tests/every_frame_answers_as_a_list/mod.rs`. A promise built below its own tier |
| *Give it to alo* | `[v1]` | **overrides a standing decision** |

**On Give it to alo.** Its tier was kept deliberately when the Stop carve-out put
*alo working in a window you put aside* at `[v0.01]` and left its neighbours
alone — that was recorded at the time as the reason not to move it. The owner
chose on 2026-10-03 to move it anyway. Written here so the earlier reasoning is
not read as still current by somebody who finds it.

## Two things the roadmap was wrong about, found by making this change

**None of the five was on `ROADMAP.md` at all.** `docs/features.md` carried them
and the roadmap listed them nowhere, in v1 or outside it. `CLAUDE.md` makes the
roadmap the only order work gets built in, so these were promises the scope gate
held with no order to be built in. They are added rather than moved.

**And a sentence in the canvas background was wrong when written.** It says
*Tidy this canvas, Every screen is a view onto the canvas and A panel out of
view costs nothing stay at `[v1]`*. `docs/features.md` has carried *Tidy this
canvas* at `[v0.01]` throughout. Two of its three moved today and the third was
never `[v1]`, so the paragraph is corrected rather than half-updated.

## What this makes the release

89 `[v0.5]` promises before today, 101 after. Of the twelve added, one is built
and eleven are not, and none has a task in any plan. **The next step for all
eleven is the same**: a task in a plan, which needs no hardware and is therefore
work a machine with no laptop can do.

## What each entry owes, and why it points here

`every_v0_5_promise_is_reconciled_against_evidence_that_runs` refuses a ledger
entry that names no test, no report, and no place the work lives: *a promise that
is missing and points nowhere is one the next reader derives again from scratch*.
None of these seven has a task in any plan, because none was expected this side
of v1.

So each entry points at this record, and this record says the next step plainly:
**each of the seven needs a task in a plan before anybody can start it**, and
that is a separate piece of work from deciding the tier. Writing those tasks is
the thing a machine with no hardware can do, and the third PC has nothing else —
measured the same day, its five plans hold four finished and three installer
tasks that all wait on a machine.

## What was deliberately not claimed

Asked what already exists for the seven, the counts are substantial and
worthless: appearance in 136 files, pairing in 118, USB in 40, LoRA in 24. None
of that is evidence a promise is shown; it is evidence somebody has used the
word. Each ledger entry therefore says the evidence has not been assessed, names
where to look, and offers nothing — rather than claiming a crate shows a promise
because its name appears nearby.

## Alternatives rejected

**Move the whole of v1.** Rejected above: it does not accelerate anything, and it
empties the meaning of both releases.

**Move nothing and demonstrate v0.5 as it stands.** Rejected because v0.5's
person-facing surface is thin without these: no language but English, no USB
portal, no appearance by request, and nothing a visitor has not seen before.

**Move the seven and leave the ledger alone.** Not available, and correctly so —
the gate refuses a promise nothing asks about, which is how six were once missed
one at a time.
