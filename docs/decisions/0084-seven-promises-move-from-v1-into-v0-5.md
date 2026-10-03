# ADR 0084 — Seven promises move from v1 into v0.5

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
