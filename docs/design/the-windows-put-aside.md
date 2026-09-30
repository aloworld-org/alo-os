# The windows a person put aside

The panel at the right edge that holds minimised windows: what it is, what already
exists, and which half of it this repository can finish.

**Status:** a design note, 2026-09-29. It settles one contradiction in
`docs/features.md`, surveys what is already built, and splits the owner's behaviour
contract into what a crate can decide and what is the compositor's. **It builds nothing
and decides nothing the owner has not already decided.**

## First, the contradiction, because it has to go before anything is built

`docs/features.md:421` ends:

> Arranging is placing; nothing is stacked, **so nothing is minimised**.

Three lines below it, `:424` keeps minimising *and records that this exact refusal was
withdrawn*:

> This line used to say *nothing is swallowed into a bar*, refusing the second
> outright; the owner's direction of 2026-09-26 keeps both, because a tile that stays
> visible and a thing put away for later are two different wishes and a person has both
> in a day.

The correction was made in one line and not in the other. **This is not two opinions and
it is not a reading error — it is a partially applied edit**, which is worse than
either: the document holds the contradiction *and* the evidence that somebody already
knew the answer, three lines apart.

`:421` is corrected to say what it was protecting, which survives: arranging is placing,
and **nothing is stacked** — windows do not pile up behind one another, which is the
claim that made *so nothing is minimised* look like a consequence of it. Putting a
window aside deliberately is not stacking, and the person asked for it.

## What the panel is, in one paragraph

A person's windows live on an endless canvas, where they put them. The **Dock** at the
bottom answers *which applications are open* and takes you to the window you last used.
The **right panel** answers a different question — *what did I put aside* — and it holds
**individual windows**. Three minimised Browser windows are three named previews, not
one Browser icon. It sits in the viewport rather than on the plane, so it does not move
when the canvas pans or zooms.

`docs/design/the-alo-dock.md` already settles the division: the Dock shows the
application is open, the right panel owns the individual windows, and it records that
this is *not the duplicate the owner refused* — the Dock's list is per application and
appears on demand; the panel is across every application and permanent.

## What already exists, which is much more than expected

Surveyed by reading the files, not by asking anyone.

### The architecture rule the panel needs is already enforced

`crates/alo-canvas` is built on exactly the panel's first contract rule:

> There is a **plane**, which carries the frames a person's applications are in, and a
> **viewport**, which carries the dock, the status area, the indicators and a window's
> controls. A pan or a zoom moves the plane under the viewport. It does not move the
> viewport, and the viewport does not correct for it.
>
> **Nothing in the viewport layer may read the camera to correct itself. If it has to,
> it is in the wrong layer.**

So *the right panel is a viewport control, never a child of the scrolling or zooming
canvas* is not a new decision. It is the existing architecture, and the panel is a
viewport surface by construction if it never imports a `Camera`.

**And that crate has already written the test-design warning this note would otherwise
have had to discover.** From `camera.rs`:

> A dock that subtracted a pan to stay still would be drawn in the plane and
> compensating, which the plan's constraint names as the wrong layer — **and it would
> pass a test that only checked where the dock ended up.**

That is the whole of how to test rule 1. A test asserting *the panel is still at x=1720
after a pan* passes for a panel that is in the wrong layer and compensating correctly.
**The thing to hold is the dependency, not the outcome**: the panel's position must not
be a function of the camera, and the way to prove it is that the type cannot see one.

### The window model

In `crates/alo-dock/src/`:

| what the panel needs | where it already is |
|---|---|
| a window keeps its position while put aside | `window.rs` — `Window` holds `at: Patch` *kept whatever state it is in* |
| the state itself | `window.rs` — `HowItSits::PutAside`, *it keeps its patch and the right panel holds its preview* |
| restore travels to the window | `clicking.rs` — *go to the window you used last, bring it back if it was put away, and move the view if you could not already see it* |
| a click never moves a window | `clicking.rs`, stated as a rule with its reason |
| *Previous view* | `travelling.rs` — *Back returns a view, not an application*, and it goes after the next deliberate action |
| peek that leaves the canvas still | `travelling.rs` — `Peeking`, reached by keyboard through the same door |
| named per-window previews | `previews.rs` — one list, three doors, and the one used last *marked* rather than merely first |
| the edge reveal under full screen | `revealing.rs` — a state machine, no timers, and the pointer can travel onto it |

**`revealing.rs` is the one to read before writing anything.** Its subject is the clause
that reads like a detail: *the pointer can travel onto the revealed Dock without it
disappearing on the way*. The panel needs exactly that at the right edge, with no
timers, for the reason that file gives — a timer makes the behaviour depend on how fast
somebody moves, which is what a person with a tremor or a trackball cannot control.

**A second state machine for the same problem would be the drift this repository keeps
finding.** Whether `Revealing` is generalised to an edge or the panel gets its own is a
real question; it is not one to answer by writing the second one first.

## Which plane the panel speaks, decided elsewhere

This note asked the question and **[`one-plane-two-vocabularies.md`](one-plane-two-vocabularies.md)
answers it**, written by the lane that owns `alo-dock` after the panel's survey found
two vocabularies for one plane. The decision is not restated here.

What the panel takes from it: **it speaks `alo-canvas`**, and the reconciliation is to
move the plane **out** of `alo-dock` rather than to teach the panel both. That move is
that lane's and is deliberately not being made at six in the morning. **The panel is
built against `alo-canvas` from the start either way** — a new surface should not wait
on a cleanup, and it should not learn the vocabulary that is leaving.

**A second copy of that reasoning is what this note must not become.** Two documents
deciding one thing is the shape this repository spent a night finding: they agree until
one is edited, and then the reader who opens the wrong one is not told there is another.
So the table and the argument live there, and the pointer lives here.

**The sentence worth carrying across**, because it generalises past the plane:
`crates/alo-dock/src/on_the_canvas.rs` has carried *Nothing here knows about a dock, a
screen or a display* as a heading the entire time. **A module can describe its own
misplacement clearly and go on being misplaced** — because a docstring is read as a
description of what the code does, never as a claim about where it lives.

**And the panel gets its own crate** rather than growing inside `alo-dock`, depending on
it for `WindowId`, `AppId` and `HowItSits`. A panel of windows put aside is a different
reason to change from a dock, which is law 4 at the scale of a crate. If a third surface
later wants those same types, they can be extracted then — **two users is not yet a
shared crate.**

## One place where the older design note and the newer direction differ

`docs/design/the-alo-dock.md` already answers what happens when a restored window's
saved place is taken:

> Choosing one restores that window at its own place; if something now overlaps it, the
> restored window **comes forward** and **nothing is rearranged**.

The owner's specification answers the same question with more in it:

> If another window now occupies the old position, it must not stack invisibly behind
> it: **show the intended placement, offer a sensible nearby position, let the person
> accept or drag**, and keep the original position in History.

**They agree on the half that was decided and the newer one decides more.** *Comes
forward* and *must not stack invisibly behind it* are the same refusal, and *nothing is
rearranged* survives — offering a nearby position that the person takes is not the
system rearranging anything. What is new is that the person is **shown** the collision
and given the choice, where the older sentence quietly resolved it for them.

Recorded rather than silently superseded, because the older sentence is not wrong, it is
short — **and a reader who finds only one of the two will build a restore that is
defensible and is not what was asked for.** `the-alo-dock.md` is another lane's surface,
so this note does not edit it.

## What is missing

- **A Place.** A window knows its patch and not which Place it belongs to. The
  specification says minimising saves *location, size, Place and zoom context* and that
  the panel may group by Place, so a Place has to exist where a preview can read it.
  (`alo-dock`'s `places.rs` is a different word: it is where an icon sits on the bar.)
- **The panel itself** — its three presentations, its own scrolling, its header and
  count, and a collapse choice that persists per Place.
- **Everything about alo in a minimised window**: the named task, the scope, progress,
  last confirmed action, Stop, and *Requires you*.
- **Privacy**: `Preview hidden` with a safe identifying name, previews made locally.
- **Multiple selection**, and the proposed arrangement shown before it happens.
- **Dragging out**, with the collision shown before the drop.

## Three findings from the lane that owns the frames, which are this panel's problem too

Relayed 2026-09-29 from the lane working `crates/alo-shell`'s frame edges. All three are
about **a region that is not where it was declared to be**, and a preview laid over the
edge of a canvas is exactly that: a shell-owned region sitting where a client draws.

**1. A band yields wherever a client actually drew.**
`Server::resize_from_the_edge_under` asks `pointer_target(at).is_some()` and returns
false *before* it consults a band. In their words: *a shell-owned region is measured
from window geometry, a client may draw outside it, and where the application drew the
press is the application's.*

**This is the panel's test, not an analogy.** The panel sits over the canvas at the right
edge; peek draws over the canvas; a dragged preview crosses it. The specification already
says peek *must not accept clicks meant for the canvas*.

**And the sharper half is not *ask who drew here* — it is *which question you ask*.**
That lane used `pointer_target`, the hit test `pointer_motion` and the renderer already
use, rather than comparing rectangles itself. **A second opinion about where clients are
is how two answers drift**, and an answer that agrees today is not the same thing as the
answer. So the panel asks the shell's existing question rather than computing its own —
which is the same rule as *one definition of the gates, three consumers*, and the same
rule as this note refusing to write a third vocabulary for the plane.

**2. Shell regions surround the frame and its name, not the frame alone.** Their top
resize band and the name band were both exactly as wide as the frame, so one sat inside
the other's last six pixels and **whichever hit test ran first decided silently**. That
is the same class as the specification's own rule that **animations must not change hit
targets midway through a pointer move**.

**3. More specific wins, and the order is the search order.** Corners before edges,
first hit returns, because a 24px corner overlaps the last 24 of two 6px edges by
construction.

**And their two tests are worth copying before their rules are.**
`no_part_of_a_frames_name_resizes_it` **sweeps the whole band** rather than testing one
point, because an overlap at one end is how that bug comes back; and
`a_real_press_on_a_band_begins_a_resize_and_reaches_no_client` goes through
`pointer_button` rather than calling the compositor directly — the test that would have
caught the original gap, because **the bands had no production caller at all.** Eight
regions and four cursors: drawn, documented, tested, and unreachable by any mouse,
because every test called `Server::begin_a_resize` directly and none entered by the road
a person uses. It looked like part of the product for two commits.

**A test that enters by the road a person uses is the only one that can tell you the
road exists.** For the panel that means a press routed through the compositor's real
input path, not `panel.peek_at(…)` — and it is the second test that matters more, which
is the opposite of the order these things are usually written in.

## The owner's behaviour contract, and which half this repository can finish

The contract has **ten** rules. (Thirteen was used in conversation and was never
counted; it is recorded here because a number that travels unchecked is how a claim
becomes furniture, and this one crossed a lane boundary twice before anybody counted.)

**model** — decidable in a crate, testable with no display. **compositor** — the
`On the machine.` half that law 3 ticks on a certified machine.

| | the rule | which half |
|---|---|---|
| 1 | the panel is a viewport control, never a child of the canvas | **model, and already architectural** — `alo-canvas` forbids a viewport surface reading the camera; the test holds the dependency, not the position |
| 2 | each preview maps to one window, not one application | **model** — `previews.rs` is already per-window |
| 3 | clicking restores at the **saved** position and moves the camera to it | **model** — the patch is kept and `clicking.rs` already decides this |
| 4 | dragging a preview moves the window to a new position | **both** — where it would land and whether it collides are model; the lift, the drag and the edge pan are compositor |
| 5 | peek leaves the window minimised and the canvas unchanged | **model** — `travelling.rs::Peeking` says exactly this |
| 6 | collapse changes the panel, not the windows | **model** — a presentation flag, and a test proving it touches no window state |
| 7 | panel scrolling never pans or zooms the canvas | **both** — which surface a scroll belongs to is a crate's decision; that the events arrive separated is the compositor's |
| 8 | true full screen conceals the panel until deliberately revealed | **model** — `revealing.rs`'s state machine at another edge |
| 9 | alo activity is named, scoped, stoppable and recorded | **model**, and the record is `alo-record`'s |
| 10 | closing with unsaved work asks; minimising never discards work | **model** for *minimising never discards*, which is a property of the state change; **compositor** for the dialogue |

**Seven of ten are wholly or partly decidable here, and rule 1 is already decided.** The
panel is not blocked on a display, and the parts that are blocked were always going to
be.

## The owner's acceptance test, and what it can and cannot earn

> Open many windows, pan and zoom far away, minimise several, scroll the panel, restore
> one, and then return to the previous view. The canvas alone should move; the panel and
> Dock should remain exactly where the person expects them.

**This is a real-build test and no unit test earns it.** What a crate earns is every
decision inside it: that restoring names the saved patch and not the current view, that
the panel's position is not a function of the camera, that a scroll over the panel is
routed to the panel, that *previous view* returns a view rather than a window.

What it cannot earn is *remain exactly where the person expects them*, which is pixels
under a moving camera and belongs to the `On the machine.` box.

Naming that split before building is what printing taught this repository this week:
what was owed there was never more code, it was paper out of a real printer, and the
roadmap already had a box for it.

## The prototype is evidence of intent, not of behaviour

The Figma file shows twelve frames — before minimise, expanded previews, location cue,
peek, restored in place, drag preview, collapsed rail, private preview, multi-select,
restore together, alo working, empty handle.

**Dragging, hold-to-peek, independent panel scrolling and camera movement are operating
system behaviours that Figma illustrates as outcomes rather than executing.** The
prototype settles what things look like and what a person should be able to do, and
settles nothing about whether any of it works. Citing it as though it demonstrated
behaviour would be the fault ADR 0080 is about: an artifact that cannot tell *this was
drawn* from *this was done*.

## Where the code goes, which is a question and not a decision

The panel shares a window model with `alo-dock` and needs a camera from `alo-canvas`.
That argues for its own crate depending on both, which also keeps law 4's
one-responsibility rule honest at the crate scale.

**But `alo-dock` is being read and extended by another lane**, so what is added to it and
by whom is a coordination question before it is a design one. Nothing here touches it.

## What is asked of the owner

*This list had a first item — **which vocabulary the plane speaks** — and it was answered
by the lane that owns `alo-dock` while this note sat in the merge queue. It is gone from
here and settled above, rather than left standing: a record that goes on asking a
question somebody has answered reads exactly like a record with an open one, and nothing
would ever say otherwise. That is instance 5 of
[ADR 0080](../decisions/0080-a-signal-that-cannot-be-wrong-tells-you-nothing.md), and it
was very nearly committed here by the lane that wrote it.*

1. **The deep teal.** `#0F6B72` is reserved for alo activity and selection uses a
   human-control outline instead. `docs/design/palette.toml` and
   `the-palette-foundations-states.md` hold this product's colours; whether the panel
   introduces anything new or only uses what is there is worth one sentence from whoever
   owns the palette.
2. **Grouping by Place** is allowed *without hiding individual windows behind an app
   icon*. Whether it is the default when there are many, or always a choice, is not
   decided.
3. **`docs/features.md:421`** is corrected here on the same direction of 2026-09-26 that
   `:424` already cites. If that reading is wrong, this is the line to say so on.
