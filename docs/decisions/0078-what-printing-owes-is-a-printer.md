# ADR 0078 — What printing owes is a printer, not a release

**Status:** proposed, 2026-09-29. The owner decided printing is not a priority, and
then said which part moves: **"We move the printing remaining tasks to version 2"** —
*the remaining* tasks, twice, in two separate messages. This record does that, and
finds that the remaining work is **one** promise rather than three.
**Date:** 2026-09-29
**Context:** `CLAUDE.md` law 3 (*done means the machine still works* — *an OS that
boots but cannot print is not a released OS*); `ROADMAP.md`'s tiers and its **Moved
here from v0.5 on 2026-09-26** list; `docs/features.md` lines 285, 302 and 344;
`crates/alo-printing`, `crates/alo-changing-printers`; `alo_reconciling::tier::Tier::V2`.

## The question

**Printing is not a priority. Which printing promises move, and what does that do to
law 3, which says an OS that cannot print is not a released OS?**

## The answer

**One promise moves. Two stay, because their work is done and only its proof on
hardware is owed — and that proof is the box law 3 exists to tick.**

    moves to v2   Files and printers shared between paired alo machines   (302)
    stays at v1   ★ Printers, solved.                                    (285)
    stays at v1   Printing. Unglamorous, and it decides public-sector…    (344)

`:344` is *corrected* to `[v1]` rather than left alone: it said `[v0.5]`, which had
been wrong since 2026-09-26. See **What was already wrong** below.

`ROADMAP.md` loses `- [ ] Files and printers shared between paired machines` from its
v1 list and keeps `- [ ] **Printing**`, because **v2 promises have no roadmap
section**: there is no `## v2` in that file, and the v2 promise already there —
*Screen recording* — lives in `docs/features.md` alone. This follows that precedent
rather than inventing a section.

## Why only one moves, measured rather than assumed

**`Files and printers shared between paired alo machines` is wholly unbuilt.** No
crate does it, no test names it, `docs/autonomy/v0-5-evidence.md` has no entry for it,
and `ROADMAP.md` carried it unchecked. It is remaining work in every sense the word
has, so it moves.

**`Printers, solved.` and `Printing` are built.** `crates/alo-printing` finds a printer
over IPP, sets it up, prints to it, and says what is wrong when it stops —
`finding_printers.rs`, `changing_a_printer_set_up.rs`, `printing_a_document.rs`,
`a_printer_that_stopped.rs`, `the_real_printing_service.rs`,
`printing_reaches_only_this_machines_printing_service.rs`, and
`crates/alo-changing-printers/tests/printers_change_only_through_the_broker.rs` — all
against a real CUPS rather than a stand-in, and all through the nine gates.

**What they owe is paper.** No physical printer has ever been attached. That is not
remaining *work*; it is remaining *proof*. Moving a built thing to a later version
would say it is not built, which is the one thing the documents must not say about it.

## Law 3 is satisfied, and not by an argument about tiers

`ROADMAP.md` gives every elaborated promise two boxes:

    - [x] **The code.**        the half this repository can finish today
    - [ ] **On the machine.**  ticked only under law 3, on the certified machine

**What printing owes is precisely the second box**, and the roadmap already says that
box is law 3's to tick on the certified machine. Printing is therefore in the same
position as every other promise whose code is finished and whose hardware has not been
met. There is nothing to narrow, qualify or except, and nothing for the owner to
approve in `CLAUDE.md`.

**Two earlier drafts of this record got there by the wrong road, in opposite
directions, and both are recorded rather than replaced** — the sequence is the useful
part:

The **first** was written when the destination was **v1**, and argued there was no
conflict because *a developer preview that cannot print is not a released OS that
cannot print* — v1 being the release, printing being in it. True, and the wrong
reason.

The **second** was written when the destination became **v2**, and concluded the
conflict was real: printing after the release means the released OS cannot print,
which law 3 forbids. Also true *given that reading*, and it offered three ways out,
the first of which was amending a law.

**Both were answering a question about which tier the promise sits at. The real
question was which half of it is unfinished.** The owner's *remaining* is what made
that visible: a promise whose code is done does not move, a promise with no code does.
A peer lane and this one had between them come within one instruction of proposing an
amendment to one of the four laws, because both had read a scope question as a tier
question. Recorded because the mistake is cheap to repeat: **the law was never the
subject.**

## What was already wrong

**`docs/features.md:344` said `[v0.5] Printing`** for three days after `ROADMAP.md`
moved printing to v1 on 2026-09-26, in a section that states the reason — the owner
cut v0.5 to what a developer preview needs, and *an organisation buying the product
will want them, and a developer trying it will not*. **That change of direction is the
fact that moved printing**, and it is recorded here because a move with no reason
recorded is a move somebody re-argues in a month.

The scope gate and the order disagreed, and `CLAUDE.md` makes the scope gate binding —
*nothing gets built that isn't in `docs/features.md` with a tier*. `:344` now says
`[v1]`.

**And the repository already had a checker that was not obliged to fire.**
`alo_reconciling::the_gate::tiers_that_disagree` exists to find a promise sitting at
one tier in the definition and another in the roadmap. Its test asserts only that the
finder still finds *Camera and microphone* — a liveness check on the instrument rather
than a gate on the repository. The disagreement was discoverable the whole time and
nothing had to discover it. Worth knowing; not this record's to fix.

## Nothing is ticked

Writing `- [x] **The code.**` for printing would be a claim, and the roadmap's own rule
is that *a code box is a claim, so its clause names the crate* — which needs a survey
of what these two crates cover against what the promise says. The evidence above is
strong enough to say the code exists and weak enough that nobody should tick from it
without looking. **Printing has no boxes at all**, like most of v1, and an unelaborated
promise claims nothing — the right state for something nobody is working on.

## The disclosure is a release note, not a flag

Printing ships in v1 with no physical printer behind it, and there are two ways to
say so. **A feature flag defaulting off** hides the gap, and a lane and I both reject
it for the same reason: code that ships disabled is code nobody exercises, so we would
meet the first real printer at the same moment a developer does — and law 3's
*integration test on real hardware* would be owed against a surface that had gone cold.
**An honest release note** — printing is implemented and tested against CUPS, no
printer model is certified yet, `docs/hardware.md` names the list — keeps the path
warm and tells the truth in the place people look for it. `CHANGELOG.md` is the
integration owner's, so this is a proposal to them and not an edit.

## Three promises that mention printers and did not move

Named because *the printing tasks* could be read to include them, and moving them would
carry unrelated work along:

- **`docs/features.md:26`** — *Settings, as one place: network, display, sound,
  **printers**, storage, keyboard, accounts, privacy, updates.* One of nine items.
- **`docs/features.md:120`** — the portal list, where **print** is one of seventeen.
- **`docs/features.md:165`** — *System verbs through the privileged broker:
  **printers**, network, updates, storage.*

Each promises a *surface* that includes printers rather than printing itself, and each
stays at v0.5.

**And `docs/features.md:324` was deliberately not touched.** Egress attestation is *a
signed, **printable** statement* — the word is about the artifact, and a search for
*print* catches it.

## What the evidence ledger does now

`docs/autonomy/v0-5-evidence.md` held printing's entry while the promise read
`[v0.5]`. The promise is `[v1]`, so the entry is not that gate's to hold, and
correcting the tier orphaned it: `every_v0_5_promise_is_reconciled_against_evidence_that_runs`
failed on the next run and said why — *the ledger has an entry for `Printing…` and no
promise in `docs/features.md` contains those words.* The documents said the change was
done; the crate that judges them said otherwise.

The first repair renamed the entry to *Printing — moved to v1*, and the check failed
again for the same reason: **it reads every `###` heading as a promise of this tier**,
so a heading saying *moved* is still an entry claiming to be one. The pointer is in the
ledger's preamble instead.

The entry's substance is carried here, so nothing is withdrawn:

> **Shown by:** `crates/alo-printing/tests/printing_a_document.rs`,
> `crates/alo-printing/tests/finding_printers.rs`,
> `crates/alo-printing/tests/changing_a_printer_set_up.rs`,
> `crates/alo-printing/tests/a_printer_that_stopped.rs`,
> `crates/alo-printing/tests/the_real_printing_service.rs`,
> `crates/alo-printing/tests/printing_reaches_only_this_machines_printing_service.rs`,
> `crates/alo-changing-printers/tests/printers_change_only_through_the_broker.rs`,
> `docs/autonomy/updates/printers-found-set-up-and-said-what-is-wrong.md`,
> `docs/autonomy/updates/printers-through-the-broker.md`
>
> **Still owed:** **a printer.** Everything is here — found, set up, printed to, and
> told what is wrong when it stops, against a real CUPS rather than a stand-in, with
> printing reaching only this machine's own service. No paper has come out of
> anything.

**Not a new `v1-evidence.md`, on purpose.** `alo-reconciling` reads
`docs/features.md`, `ROADMAP.md`, `v0-01-evidence.md` and `v0-5-evidence.md`. A ledger
it does not read would be an evidence file nothing checks — the dead-citation fault
rebuilt as a new file. An ADR *is* read by `alo-citing`, so the seven tests above are
held to existing.

## What is asked of the owner

1. Whether the three printer-mentioning promises at v0.5 — Settings, the print portal,
   the broker verb — should stay where they are. Each ships a surface that includes
   printers, and `alo-printing` exists, so each can do something real; what is missing
   is a printer to point it at.
2. Whether **a printer model** should be named in `docs/hardware.md`, so printing's
   `On the machine.` box has a stated bar rather than an implied one. *Certified before
   compatible* applies to printers as much as to machines: one printer that works
   beats a compatibility list nobody can honour.
