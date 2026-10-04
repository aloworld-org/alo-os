# ADR 0086 — The complete canvas is one current milestone

**This was 0085 until 2026-10-04.** Another lane's
[ADR 0085](0085-how-a-person-reaches-settings.md) landed on main while this one was
in flight, under the same number — two records claiming one identity, which the
citation gate caught from the far side: a link in
`docs/design/the-interface-in-the-file.md` named a decision that existed on no
branch anybody could fetch. Recorded here because the gate's own message is right
about why it matters — *a decision renamed leaves every link to it reading exactly
as it did before* — and because the collision is not a mistake either lane made.
Numbers are claimed by landing, and two lanes drafting at once will take the same
one; what was missing is the check at the point of writing, which is noted as owed
below.

**Status:** **accepted, 2026-10-04**, by the owner, in these words:

> Update `features.md`, `ROADMAP.md` and the canvas plan so every agreed canvas
> capability belongs to one current milestone: Complete canvas.
>
> Remove version-based deferrals for canvas work. This includes Places and World,
> window movement and resizing, selection and arrangement, zoom and pan, all four
> Dock positions, the minimized panel, full-screen controls, keyboard access,
> accessibility, recovery notices, persistence, and canvas-facing agent controls.
>
> Include the supporting integration needed to make those features work: renderer
> access, validated agent-status delivery, cancellation, and production
> save/restore paths.
>
> Keep implementation order and dependencies. Remove release-tier barriers. Work
> may be sequenced, but it must not be excluded merely because an older document
> labels it v0.5 or v1.
>
> Reconcile conflicting ADRs and scope records before applying their gates. Keep
> unrelated operating-system features on their existing roadmap.
>
> A canvas capability is complete only when it works through the production path
> and meets its acceptance criteria. A model, unused function, Figma state or
> isolated test is progress — not completion.
>
> Assign every remaining task an owner, continue independent work while
> dependencies are being resolved, and report concrete blockers rather than asking
> again whether already-approved canvas features are in scope.
>
> This authorizes completing the agreed canvas, not inventing unlimited new
> features.

Quoted whole rather than summarised, because three of its sentences are gates on
the work and one is a gate on *this lane's* behaviour, and a summary would be a
lane deciding which of them bound it.

## The milestone is written as `[v0.01]`, and that is the decision inside the decision

A new marker — `[canvas]`, `[Complete canvas]` — was the obvious reading and is
the wrong one. `crates/alo-reconciling/src/tier.rs` hard-codes four markers and
three roadmap headings:

```rust
Self::V0_01 => "- [v0.01]",   Self::V0_01 => Some("## v0.01 — it boots and the agent acts")
Self::V0_5  => "- [v0.5]",    Self::V0_5  => Some("## v0.5 — a person can work on it all day")
Self::V1    => "- [v1]",      Self::V1    => Some("## v1 — an organisation can buy it")
Self::V2    => "- [v2]",      Self::V2    => None
```

`promises_at`, `promises_with_no_box` and `tiers_that_disagree` all read those.
A promise carrying a marker the reconciler does not know is a promise **no gate
counts** — so a *Complete canvas* marker would not remove the barrier, it would
replace a barrier the gate enforces with one nothing enforces. That is weaker
than the tier it replaced, and it is the opposite of what the instruction asks
for.

So the canvas's promises move **into the current release**, which is what
*remove version-based deferrals* means operationally, and the milestone's **name**
lives where a person reads it: a `### Complete canvas` section inside
`ROADMAP.md`'s v0.01, gathering them with their boxes so the board counts them.

**This is the repository's own mechanism, used twice already.** The owner moved
the full canvas experience into v0.01 on 2026-09-30, and twelve promises into v0.5
in [ADR 0084](0084-seven-promises-move-from-v1-into-v0-5.md). Both times the
annotation says the same thing: *the tier moved rather than the scope gate being
crossed — `CLAUDE.md` binds building to what this file says, so the file is what
changed.*

## What moves

| Promise | Was | From the owner's list |
|---|---|---|
| **A Place remembers time** | `[v0.5]` | persistence |
| **Every screen is a view onto the canvas** | `[v0.5]` | Places and World |
| **A panel out of view costs nothing** | `[v0.5]` | the minimized panel |
| **Every canvas also answers as a list** | `[v0.5]` | keyboard access, accessibility |
| **Give it to alo** | `[v0.5]` | canvas-facing agent controls |

Four of the twelve capabilities the instruction names were **already** `[v0.01]`
and are not touched: *Every goal is a canvas* and *The World, and moving between
Places* (Places and World), *Frames, dragged and resized like a design canvas*
(window movement and resizing, zoom and pan, selection), *Where the Dock goes*
(all four Dock positions), *The top controls* and *Reaching the Dock over a
full-screen window* (full-screen controls), *The canvas is where they left it*
(persistence), and *Alo working in a window you put aside, and Stop*
(cancellation).

**One capability the instruction names had no promise at all**, and it is added
rather than borrowed: *recovery notices*. Telling a person the machine moved their
window is not the notifications portal and not calm notifications — both of which
stay where they are, because they are the ordinary notification system and the
instruction says to keep unrelated operating-system features on their existing
roadmap. What the canvas needs is narrower and is written as its own promise.

## What does not move, named so nobody widens it

The notifications portal, calm notifications, *Notifications are decisions*,
divide-the-screen, remember-a-split, touchpad gestures, keyboard layouts,
screenshots, the status area's own clock-battery-network-volume promise, Bluetooth,
the shell's 24 languages, sticky and slow keys, corporate proxy, the alo Bar,
History, Content is the interface, No dock by default, Zones, A frame simplifies
as it shrinks, Ghost previews, the agent's presence, project spaces, and
everything outside `## The interface`. **A canvas capability is one the canvas
cannot be complete without, not one a canvas touches.**

## What completion means, and it is now a rule rather than a habit

> A canvas capability is complete only when it works through the production path
> and meets its acceptance criteria. A model, unused function, Figma state or
> isolated test is progress — not completion.

This is the fault this repository has found **eight times across three machines**
in two days, written as a gate for the first time. Every instance was green:

| | |
|---|---|
| `canvas_never_lost`'s rule | written, tested, no caller on any path a person takes |
| `frames_the_controls_now_hide` | tested, called by tests alone |
| its mover | tested, called by tests alone |
| `bring_this_window_back` | drawable, pointable, revealable — and unreachable |
| `panel_is_revealed` | two of three mutations survived the whole suite |
| `panel_reserved` → a constant | seventeen tests passed |
| the status area removed from the rule | nine hundred and fifty tests passed |
| `alo-admitting` / `alo-reported` | an island: tested, clippy-clean, no way in |

Four of the eight were found by **removing the wiring and watching nothing fail**,
which is the only technique on that list with a measured hit rate. None was found
by its author re-reading their own work.

So *works through the production path* is to be read as a lane reads a gate: the
value a person's machine uses reaches the code from the draw, the seat or the disk
— not from a fixture that supplied it.

## Reconciliations this ADR requires before the milestone's gates apply

The instruction says to reconcile conflicting ADRs and scope records **before**
applying their gates. Four are open and each is recorded with what was measured:

1. **ADR 0076 against the four-edge promise.** `docs/features.md` promises *the
   bottom edge by default, and the person may choose bottom, left, right or top*;
   [ADR 0076](0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
   fixes the Dock to the bottom edge. *All four Dock positions* is in this
   milestone, so **0076 is superseded in that respect** and its own record must say
   so. Everything measured from the bottom edge becomes edge-relative: the Dock's
   band, the status area's position, the top controls' band, the reveal strips.
2. **~~The status area's position.~~ Two surfaces share one name, and this
   reconciliation was wrong when it was written.** It said the owner fixed the
   status area at the top-right while a real draw put it at `x=905 y=575 367 × 36`,
   immediately above the Dock — and called that a disagreement with opposite fixes.
   **It is not a disagreement.** Measured 2026-10-04:
   - The owner's **status area** — clock, battery, network, volume — is at the
     top-right, and **nothing draws it.** `crate::desktop_raster`'s own note says
     those four have no location in the picture.
   - `FixedControlsDrawn::status_area` carries `pictures.status.band`, which is the
     **egress indicator's** painted band. That indicator has been at the far end of
     the Dock since before ADR 0076, which moved where the answer comes from and
     explicitly *not the position*. `crate::egress_status_place` says so in its own
     words: *the rest of the status area is owed a location and does not have one …
     the egress indicator is not waiting on it, because it has a corner of its own
     and always did.*

   So the draw and the record describe **different surfaces**, one drawn and one
   not, and there was never a position to settle. What is owed is a **name**: a
   field called `status_area` that holds the egress band will be read as the status
   area by the next person, which is how this lane came to write a conflict into a
   decision record. **Recorded as this lane's own error rather than quietly
   rewritten**, because an ADR that silently drops a reconciliation leaves a reader
   wondering which of them was real — and because the mistake has a shape worth
   keeping: it was made by reading a field's *name* and taking it for a measurement,
   eleven days after the same lane did that with a filename and was corrected.
3. **The corner ruling names three surfaces and the set has four.** ✅ **Closed
   2026-10-04.** The owner's ruling of 2026-09-30 settles the corner for the top
   controls, the right panel and the bottom Dock. The egress indicator's band joined
   the fixed-control set on 2026-10-02, after it, and on a real draw its band and the
   panel's reserved column **shared 104 pixels** — so that ruling's own clause, *one
   pointer position cannot reveal two surfaces*, was false of a real screen for the
   surface it does not name.

   **And it was three surfaces, not one.** `egress_status_place::Place` is the one
   constructor that answers *where does a surface at this corner sit*, and it
   measured from the output's own width. Three things go through it: the egress
   indicator, *your camera is on* (`crate::in_use_raster`), and notifications at the
   other end (`Place::of_the_other_end`). **Only the first is in
   `FixedControlsDrawn`'s set**, so the other two were under the panel with nothing
   in the repository able to measure them.

   Fixed by construction rather than by agreement: the column is a **required
   parameter** of `Place::of`, so the corner's far edge *is* the column's near edge,
   and all three call sites had to answer for it to keep compiling. A
   `clear_of(panel)` to call afterwards was the obvious shape and the wrong one — a
   combinator is a thing a caller forgets, and this record's own consequences section
   is about correct, tested, unreachable code.
4. **`ROADMAP.md` and `docs/features.md` name the canvas differently.** Five of six
   roadmap canvas entries have no counterpart title in the definition, because the
   roadmap's titles were promoted from the plan's task headings. Ticking a code box
   on one of those five would record evidence against an entry no promise matches.
   Content is reconciled first and the gate's checks extended second — the reverse
   order makes every lane's build red for a disagreement none of them wrote.

## Consequences

**The v0.01 exit gate grows.** Five promises join a release whose gate counts
promises, and a sixth is new, so v0.01 cannot be declared complete until the
canvas is. That is the milestone's whole point and it is stated here rather than
discovered at the gate.

**`alo-reconciling` needs no change for this ADR**, which is why the marker
decision above matters: the reconciler reads `[v0.01]` today. The two gate holes
named in reconciliation 4 are separate work and are not closed by this change.

**The tiers of unrelated features are untouched**, so a reader comparing this file
with `ROADMAP.md` will find v0.5 and v1 still populated. They are not residue.
