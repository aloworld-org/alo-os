# ADR 0071 — The name moves a frame, the edge resizes it, and the pointer says which

**Status:** accepted, 2026-09-27.

Written by the Mac lane. Amends [0065](0065-the-interface-a-world-its-places-and-the-objects-in-them.md),
which contradicted itself on one point; the owner settled it on 2026-09-27 and
specified it in `docs/design/the-shortcuts-and-the-edges.md`.

**What is implemented is recorded against the tasks in
`docs/autonomy/the-smallest-canvas-worth-showing.md`, not here.** This is a
decision, and a decision is not a claim that any of it has been built.

## The collision, quoted

ADR 0065 says both of these, two paragraphs apart. First, under *Handles, not
chrome*:

> A frame is the application's own picture with nothing around it. Selecting it
> shows handles; **dragging an edge or a corner resizes it**, and the application
> is told its new size as it happens, not after a border is let go.

Then, in the table headed *Where the title bar's work goes*, against the row
*gives something to grab*:

> **the name is the handle**, and so is the frame's edge. Inside the frame every
> click belongs to the application, so nothing is moved by accident.

So the edge is a resize handle in one place and a move handle in the other, on the
same pixels. `docs/autonomy/the-smallest-canvas-worth-showing.md` inherited the
ambiguity and split it across two tasks — task 3 *Dragging a frame* and task 4
*Resizing, from the edges and the corners* — which is how it was found: neither
task could build its gesture without deciding the other's.

## Decision

**The edge and the corners resize. The name above the frame moves it.**

The table is the loose sentence and the paragraph is the right one. Task 4 keeps
the edges and the corners; task 3 gets the name. *Inside the frame, every click
still belongs to the application* survives from 0065 unchanged, and is the clause
that matters most: it is what makes a frame safe to work in.

This is what Windows and macOS have always done, so it is what a person arrives
knowing — which is the standing rule in `docs/features.md` that **the habits
people arrive with still work**.

## And the part without which this is only an assertion about pixels

**The pointer is the affordance.** Over an edge or a corner the cursor becomes the
**double-headed arrow for that direction**; over the name it does not.

This is not a detail of the decision, it is the decision's justification. Without
it, *the edge resizes and the name moves* is a rule a person has to be told and
then remember, on a frame that deliberately shows them nothing — and a rule nobody
can see is a rule they discover by resizing a window they meant to move. With it,
the frame can keep 0065's promise of **no furniture at all** and still answer *what
does this part do* the moment somebody reaches it. A cursor is the cheapest
affordance there is: it costs no pixels until the pointer arrives, it covers every
direction without drawing eight grips, and it is the one piece of window-management
vocabulary every desktop already shares.

It is also why this amendment is not simply a note on 0065. 0065's argument for
chrome-lessness was *a frame is the application's own picture with nothing around
it*, and the honest objection to it is that a picture with nothing around it
cannot be grabbed. The cursor is the answer, and 0065 did not have it.

## Consequences

- **A frame needs a name, and this compositor did not read one.**
  `xdg_toplevel.set_title` is not read anywhere in `alo-shell`; smithay holds it
  in the toplevel's role attributes and nothing asked. A frame with no name has
  nothing to be grabbed by, so the name stops being decoration and becomes the
  move affordance. Where an application sets neither a title nor an app id, the
  name is the machine's own words for one — `alo_access::words::AN_APPLICATION`,
  the same phrase the accessibility tree already reads for a window, rather than a
  second vocabulary invented for this.
- **The name is also what a reader says.** Task 7 of the canvas plan — *every
  canvas answers as a list* — asks that every frame be reached, focused **and
  named**, and was blocked on the same missing capability. One name serves the
  grab and the reader, which is the constraint that task carries: *this is not a
  second interface.*
- **A client cannot move or resize itself. Both requests are refused, and
  refused outright.** A chrome-less frame has no non-content area a client can be
  pressed in, so a client asking to be moved is asking on behalf of a press that
  belongs to it. This reverses what `alo-shell` did until now, which honoured such
  a request from a press anywhere in a window. 0065 anticipated the case it costs
  and put it in v1 — *an application that insists on drawing its own window is
  given a frame holding that window* — which is a frame around a window, with the
  name above it, and not this.

  **Outright rather than only when the press was in the content**, because that
  test would depend on mapping a serial back to where its press began, and an
  implicit grab is exactly the case where that mapping stops being trustworthy: a
  client holding a button pressed inside itself receives the next press too,
  wherever the pointer has since travelled. A rule that depends on nothing beats a
  rule that depends on bookkeeping being right.

  **And `xdg_toplevel.resize` is refused for the same reason and one more.** The
  shell owns the edges and the corners exactly as it owns the name, so a resize
  request is the same kind of request. But the decisive argument is that the two
  cannot be separated: **resizing from the left edge and then from the right
  composes into a translation** — same size, moved — so honouring resize while
  refusing move would leave move reachable in two gestures. A refusal that can be
  composed around is not a refusal.

  What refusing `resize` does **not** do is freeze a window's size. That request is
  specifically an interactive, pointer-driven resize; a client may still commit
  whatever size it likes and be drawn at it. What it loses is taking over the
  pointer to do it.
- **The refusal cannot be tested on its own.** A frame nothing can move satisfies
  *a press inside the content never moves it* perfectly, so the refusal is only
  meaningful beside a working move: one test, in which the name moves the frame and
  a content press does not move the same frame.
- **What this costs, plainly: a dead title bar.** An application that draws its own
  title bar — anything that has not negotiated server-side decorations — has a
  title bar that looks draggable and does nothing, and resize grips that will do
  nothing once the resize half lands. That is not a reason to soften the rule; it is
  why decoration negotiation matters, and it is task 3 of
  `docs/autonomy/applications-people-already-use.md`, which names `xdg_decoration`
  as *whether the application or the shell draws the frame, which on a canvas is
  the shell*. Until that exists, a dead title bar is the visible symptom, and it is
  understood rather than unnoticed.
- **The two refusals ship in two changes, and the gap is named.** Nothing in the
  shell can resize yet — the only road is a client's own grips — so refusing
  `resize` before the edge gesture exists would leave a person unable to resize
  anything at all. So the move refusal ships with the name that replaces it, and the
  resize refusal ships welded to the edge gesture and the cursor, never separated
  from them. **For that one landing a client can still move itself by resizing from
  two edges in succession**, which is the hole this consequence exists to name
  rather than discover.
- **The double-headed arrow is a compositor-owned cursor**, like the arrow in
  `default_cursor.rs`, and its hotspot is its middle rather than its tip. Four
  shapes cover eight directions.
- **Nothing here settles what a selected frame shows.** 0065's *selecting it shows
  handles* stands; this says which pixels move and which resize, and that the
  cursor is what tells somebody so.

## The three habit collisions are not settled by this

`docs/design/the-shortcuts-and-the-edges.md` names three places where a key a
person arrives knowing means something else here — `⊞`+D is *show desktop* on
Windows and hands work to alo here, `⊞`+M minimises everything there and one
window here, and `⊞` alone opens the alo Bar where Windows opens its own menu. The
owner left all three open. **They are decisions about whether a familiar key keeps
its familiar meaning or earns a better one, and they are not resolved by building
the shortcuts**; an implementation that quietly picked one would settle it without
anybody deciding.
