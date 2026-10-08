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
`more-than-one-display-plan.md`** for the arrangement, **tasks 1 to 4 of
`screens-and-desks-plan.md`** for hotplug, position and scale.

The integration owner may reasonably judge that the reachability table already
in `more-than-one-display-plan.md` is enough and the tick should stand. This
lane will not re-raise it.

## A gap this surfaced, which is not ours to close

**`SHARED_MAIN.md` line 492 restricts four documents to a designated integration
owner and designates nobody.** No document in `docs/autonomy/` names who holds
the role, and the history cannot answer it either, because every commit is
authored by the repository's owner as the attribution rule requires.

The nearest thing to a designation is `docs/autonomy/STATE.md:13432` and
`:13477`, where the desktop lane describes itself as *"single desktop worker and
shared-progress integration owner"*. **That is a lane's self-description in a
progress log, not a designation by the owner** — so it is evidence of who has
been doing it and not of who may.

The consequence, measured today: **two lanes each verified this finding
independently and neither could land it**, because one may not edit the document
and the other cannot show it was designated. This file is the route that works
without resolving that. The designation itself is the owner's to make, and both
lanes have raised it.
