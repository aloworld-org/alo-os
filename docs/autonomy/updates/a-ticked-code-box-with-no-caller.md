# Proposed to the integration owner: a ticked code box with no caller

- **Contributor:** the Mac lane
- **Date:** 2026-10-08
- **Subject:** `ROADMAP.md`'s v0.5 gate for *Multi-monitor, scaling, hotplug*
- **Why this is a proposal and not an edit:** `docs/autonomy/SHARED_MAIN.md`
  line 492 — *"Only the designated integration owner edits these shared
  documents"* — and `ROADMAP.md` is one of the four. Other contributors
  *"include proposed changes to these documents in their own task report"*,
  which is what this file is.

## What the document says

`ROADMAP.md:2205`, under **v0.5 — a person can work on it all day**:

```
- [ ] Multi-monitor, scaling, hotplug
  - [x] **The code.**
        `alo-displays` — which screens these are over the whole set so two
        identical monitors are told apart, where each goes, how large each
        draws with what the machine can actually draw applied last, and an
        arrangement remembered per **set** of screens so docking at the
        office restores the office's layout. Hotplug both ways
        (`plugged_in`, `unplugged`) with what was open on a screen that went
        following it and coming back ...
```

## What is true, measured 2026-10-08

**Every clause of that is true about `alo-displays` in isolation**, and none of
it is disputed here. The crate is good and its tests are real —
`three_sets_of_screens_remembered_apart` genuinely proves the per-set memory.

**What is not true is the reading a person takes from the tick.** Three
measurements, each confirmed independently by the desktop lane:

| what | measured |
|---|---|
| nothing builds an arrangement | `direct_desktop.rs:155` — `fn the_screens_of(..) -> Option<Screens> { None }`, the default. One caller. **No override anywhere**: `alo-desktop` does not appear in a workspace-wide search for the name |
| hotplug has functions and no behaviour | `plugged_in` and `unplugged` have **no caller outside `alo-displays`**. Every other hit is `alo-agentd` on network cables, a different subject |
| discovery happens once | `discover_every_atomic_output` is called at `direct_desktop.rs:287` and nowhere else. `direct_desktop.rs:726` carries a comment saying it *is what will make it* — **future tense, in the source** |

So the box reads *this half is whole*, and the half that is whole is a library
nothing calls.

## Why this is worth changing rather than leaving

`ROADMAP.md`'s own header says **a page of ticked code boxes means the work is
ready for phase 8.** This tick defeats that reading: phase 8 cannot show
multi-monitor on a certified laptop, because there is nothing for it to show. A
tick that promises readiness for a phase that cannot use it is worse than an
unticked box, and it is in the one document a reader trusts for *what is done*.

## What is proposed

**Not unticking it.** The code genuinely is written, and `[ ]` would be wrong
in the other direction and would discard real work from the record.

A clause inside the tick saying what it covers, in the gate language that
document already uses: that `alo-displays` is complete and tested, that
**nothing in production reaches it**, that hotplug has the two functions and no
watcher, and where the remaining work is — **tasks 10 to 12 of
`more-than-one-display-plan.md`** for the arrangement, **tasks 13 to 16 of
`more-than-one-display-plan.md`** (tasks 13 to 16) for hotplug, position and scale.

The integration owner may reasonably judge that the reachability table already
in `more-than-one-display-plan.md` is enough and the tick should stand. This
lane will not re-raise it.

## A gap this surfaced, which is not ours to close

**`SHARED_MAIN.md` line 492 restricts four documents to a designated integration
owner and designates nobody.** No document in `docs/autonomy/` names who holds
the role, and the history cannot answer it either, because every commit is
authored by the repository's owner as the attribution rule requires.

The nearest thing to a designation is two lines in `docs/autonomy/STATE.md`, and
read in place they are weaker than they look. Both are the **only** occurrences
of the phrase in that file:

```
## 2026-09-08 — Focused window layout command dispatch
Single desktop worker in C:\dev\alo-os; integration owner of the four progress
                                                                    (line 13432)
## 2026-09-08 - Native window control strip painting
Single desktop worker and shared-progress integration owner in C:\dev\alo-os.
                                                                    (line 13477)
```

Three things about them, each checked in place:

- **They are a lane's own description of itself**, at the top of an iteration
  entry, in a journal whose job is to record what happened. Evidence of who has
  been doing it; never of who may.
- **They are dated 2026-09-08** — a month old.
- **They name `C:\dev\alo-os`**, which is not the checkout any lane is working
  in today.

So the designation does not exist, and the only trace of one is stale and about
somewhere else.

**How this was found is worth more than the finding.** Each lane corrected the
other, and neither correction was disputed. The Mac lane cited
`SHARED_MAIN.md` line 492 and inferred from it that the document belonged to the
desktop lane, which is not what the line says. The desktop lane then reported
that *no document names who the owner is*, having searched three guessed
phrasings — `designated integration owner`, `integration owner is`, `am the
integration` — rather than the plain substring, which finds both hits at once.
**Tested the spellings expected instead of the words**, which is
`docs/misreadings/a-spelling-i-did-not-search-for-is-not-an-absence.md` and the
confident negative `a-negative-result-proves-nothing-about-the-search.md` was
written for three days earlier.

The consequence, measured today: **two lanes each verified this finding
independently and neither could land it** — one may not edit the document, and
the other cannot show it was designated. `SHARED_MAIN.md` reserves four
documents to a designated owner, the only trace of a designation is a month-old
self-description from a checkout nobody is using, and three lanes are running.

This file is the route that works without resolving any of that. The designation
itself is the owner's to make, and both lanes have raised it.
