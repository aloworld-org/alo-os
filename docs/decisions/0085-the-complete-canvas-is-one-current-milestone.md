# ADR 0085 — The complete canvas is one current milestone

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
2. **The status area's position.** `canvas_fixed_controls` records twice that the
   owner fixed it at the top-right; a real draw puts it at `x=905 y=575 367 × 36`
   on a 720-tall screen, immediately above the Dock's band at `y=619`. The draw and
   the record disagree and the two readings have opposite fixes.
3. **The corner ruling names three surfaces and the set has four.** The owner's
   ruling of 2026-09-30 settles the corner for the top controls, the right panel
   and the bottom Dock. The status area joined the fixed-control set on 2026-10-02,
   after it, and on a real draw its band and the panel's reserved column **share
   104 pixels** — so that ruling's own clause, *one pointer position cannot reveal
   two surfaces*, is false of a real screen for the surface it does not name.
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
