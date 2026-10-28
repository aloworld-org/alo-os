# The terminal window

**Status: measured off the design file, recorded before implementation**, which
is `CLAUDE.md`'s first standing rule: *a screenshot is not a specification —
take the numbers off the frames and record them, because a layout matched by eye
is a layout nobody can check.*

**Date:** 2026-10-14.

**The design:** file `nDxyF5Ho9oC4RObjVzwBNJ`, five roots, all committed in
`figma-snapshot/` and listed in its `ROOTS.md`:

| node | what it is | snapshot |
|---|---|---|
| `479:28965` | Terminal · Rich results by default | `479-28965.xml` |
| `485:28999` | Terminal · Current behaviour, source labels, autonomy, recovery | `485-28999.xml` |
| `493:29344` | Terminal settings · permission levels and model selection | `493-29344.xml` |
| `498:29475` | Terminal / Settings / Owner-approved gear | `498-29475.xml` |
| `500:29477` | Terminal settings · the detailed states, light and dark | `500-29477.xml` |

**Nothing in this document is built yet.** `alo-terminal` owns the pty and the
grid and knows no display; what draws the grid is the next change, and this is
the specification it has to meet.

## What the snapshot carries, and what it does not

**Geometry only.** Across all **1,300** named elements of `485-28999.xml` the
only attributes that occur at all are `id`, `name`, `x`, `y`, `width` and
`height` — 1,300 of each — plus `hidden` on 62 of them. Measured: **zero**
occurrences of `fontSize`, `fontFamily`, `fill`, `lineHeight`, `fontWeight` or
`style`.

So **type and colour cannot be taken from the snapshot** and were asked of the
live file instead, through its variable definitions. Both sources are named
against every figure below, because a figure whose provenance is not written
down is one the next reader has to re-derive.

## The window, which is one size in every view

Read from `479-28965.xml`. The section holds **three sibling views** — *01
Unstructured output / text fallback*, *02 Benchmark / automatic graph*, *03
Changes / automatic diff* — and all three contain the identical window. That the
three agree is the measurement; a figure taken from one frame could have been
that frame's accident.

| element | size | at | note |
|---|---|---|---|
| the view's own frame | 1440×1040 | — | the frame the window was drawn in |
| External window edge | 1040×44 | (200, 80) | **alo's existing edge**, not the terminal's |
| the window body | 1040×800 | (200, 124) | directly below the edge, same width |

So the terminal is **1040 wide** with a **44-tall** edge above it and an
**800-tall** body, drawn inside a 1440×1040 frame.

### Inside the body, in the body's own coordinates

| element | size | rows it occupies |
|---|---|---|
| Project and session identity | 1038×56 | 1 – 57 |
| Sessions | 1038×56 | 73 – 129 |
| Transcript / optional view explicitly requested | 1038×706 | 57 – 763 |
| Task scope note | 1038×28 | 763 – 791 |

**Children begin at (1, 1) and are 1038 wide in a 1040 body**, so the body
carries a **1px border** on every side. That is a derived figure and is stated
as such: it is `(1040 − 1038) / 2`, not something the file labels.

**And the body is fully accounted for**, which is the check that this list is
complete rather than merely long:

```text
   1  border
+ 56  identity strip
+706  transcript
+ 28  task scope note
+  9  below the note
= 800  the body
```

*The `Task scope note` was missing from the first draft of this table, and the
arithmetic above is what found it: the transcript ends at 763 in an 800 body, so
37 rows had no owner. It holds the text `zsh · UTF-8` — the shell and the
encoding — and the design's own reference text for these views says* preserve
original output and existing shell input, *so a shell input row is a separate
thing still to be located.*

The transcript's own content begins 20 rows below its top: its single text child
sits at y 20 inside it. So the transcript is inset **24 horizontally** (measured
twice, below) and **20 vertically at the top**.

## Sessions and the Transcript overlap, and the designer switched Sessions off

**The overlap is real.** Sessions occupies rows 73–129 and the Transcript
occupies rows 57–763, so Sessions sits **inside** the Transcript's own span.
Both cannot be drawn at once.

This was recorded here as a silence in the file, with *this document does not
guess* beside it. It is not a silence. The file answers it, in the one place
nobody had read: **the visibility flag the designer set.**

### The measurement

Every placement of the frame named `Sessions` in the captured snapshot, counted
by whether it carries `hidden="true"`:

| node | placements | hidden | visible |
|---|---|---|---|
| `479-28965.xml` | 3 | 3 | 0 |
| `485-28999.xml` | 31 | 31 | 0 |
| `493-29344.xml` | 14 | 14 | 0 |
| `500-29477.xml` | 68 | 68 | 0 |
| | **116** | **116** | **0** |

**One hundred and sixteen placements, and the designer drew none of them.**

### Three things that have to agree, and do

A visibility flag on its own could be an accident of one export. Three
independent readings point the same way:

1. **The flag**, above: hidden in 116 of 116.
2. **The arithmetic.** The body balances to exactly 800 — `1 + 56 + 706 + 28 +
   9` — **without** the sessions strip in it. A layout that meant to stack
   both would not add up without one of them.
3. **The nesting.** `Sessions` (`479:29023`) and the Transcript
   (`479:29032`) sit at the same depth — 3 — under the **same parent node**,
   `479:29012`: two sibling frames in one container, one on and one off. That is what a toggled state
   looks like in a design file. A stacked layout would have one inside the
   other, or neither overlapping.

Corroborating how `hidden` is used in this file at all: in `479-28965.xml` it
appears six times, and the other three are a text named `launch`. Both uses are
alternatives the designer parked, not deletions.

### What this answers, and what it still does not

**Answered: the default.** The transcript is what is drawn. The strip is not. So
the 1038×706 transcript is unconditional, which is what the code already
builds, and nothing in the window is waiting on this.

**Settled 2026-10-16: the condition is not a question yet, and this closes it
rather than answering it.** It was recorded here as *the owner's to settle when
the strip is wanted*, which read as a decision waiting on somebody. It is not.

*When does a second session appear in the strip* presupposes **several sessions
inside one terminal window**, and `docs/features.md` promises that nowhere —
searched for a sessions strip and for a second session, **nought hits**. So
there is no promise whose condition this would be, and naming one would be
deciding a feature rather than resolving an ambiguity. `CLAUDE.md`: *nothing
gets built that isn't in `docs/features.md` with a tier*, and *when the answer
is not obvious, it is scope*.

**The plural that is promised is a different plural.** *A project you come back
to is where you left it — its terminals, folders, launch commands and related
windows* is several terminal **windows** restored in a Place, which is the
canvas's doing and wants no strip inside a window. A reader meeting *terminals*
there and a hidden `Sessions` frame here could join the two into a feature
neither says; they are unrelated, and that is written down because the join is
the easy mistake.

**So the sessions strip is not built from this design**, and nothing is waiting
on anybody. Not because the file contradicts itself — it is consistent, and the
three readings above agree — but because the file shows the strip drawn nowhere
and the scope list carries no line for it. Building a surface the design
switched off in every frame it drew would be a promise nobody made.

**What would reopen it** is one line in `docs/features.md` with a tier, saying
that a terminal window holds more than one session. The condition becomes a real
question the moment that exists, and not before.

## The transcript is the protected region, and it is one size

Read from `485-28999.xml`. The element named **`Protected transcript /
terminal-owned source gutter`** is the Transcript frame of `479-28965.xml` under
its full name — the name that says what
[ADR 0113](../decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
promises about it.

Measured across every instance in that node:

| size | instances | reading |
|---|---|---|
| 1038×706 | 30 | the ordinary transcript, in the 1040-wide window |
| 718×706 | 1 | a narrower transcript, 320 less wide |
| | **31** | every instance in the node |

**The height never varies.** The one narrow instance is 320 narrower and exactly
as tall, which is what a side panel taking 320 columns would leave.

### A source region is inset, and its height is always a multiple of 22

| transcript width | source region width | inset each side |
|---|---|---|
| 1038 | 990 | 24 |
| 718 | 670 | 24 |

`990 = 1038 − 24 − 24` and `670 = 718 − 24 − 24`, so the inset is **24, and it
does not change with the width.**

And every `Terminal / Source region` measured is one of four heights, each a
whole number of 22s:

| height | lines | at width 990 | at width 670 | all |
|---|---|---|---|---|
| 22 | 1 | 60 | 1 | **61** |
| 44 | 2 | 42 | 0 | **42** |
| 66 | 3 | 14 | 2 | **16** |
| 88 | 4 | 4 | 0 | **4** |
| | | 120 | 3 | **123** |

**123 instances, every one a multiple of 22, with no exceptions.** So a line of
the transcript is **22 tall**, derived from geometry alone and before any type
information was asked for.

*The by-width columns are here because the first version of this table gave the
990-wide counts as the totals, and so said 60 and 14 where the answers are 61
and 16. The totals are what a reader checks against; the split is what makes
them checkable.*

## The gutter, and the four words in it

[ADR 0113](../decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
decides that a narrow column beside the transcript belongs to the terminal and
says who is speaking. The design draws it, and the file states the rule in its
own words — an annotation on `485-28999`:

> **Source gutter: terminal-owned, outside clipped program content. PROGRAM
> includes parsed rich results. SYSTEM means an attributed observation, with
> scope and time. ALO is interpretation. YOU is input. Colour is
> supplementary.**

*Outside clipped program content* is the load-bearing phrase. The gutter is not
an overlay drawn on top of the cells; it is room taken away from them before
they are laid out. That is the difference between a promise and a decoration,
and it is why `a_programs_room_inside` exists rather than a drawing order.

### It is 96 wide, and the second measurement is the one that matters

| read from | figure |
|---|---|
| `Rich test result / untrusted program data`, 894 wide at x **96** inside a 990-wide boundary | `96 + 894 = 990` exactly |
| a **plain** `Terminal / Source region`, rendered 1:1 and measured by pixel | gutter holds `you`; the content's ink resumes at x **97** |

The first alone would have been a measure of **rich results**, and a plain
transcript would then have drawn its cells straight through the gutter. The
second is a node with no rich result anywhere in it, and it has the same
column — so the gutter is unconditional.

### What is in it

| | |
|---|---|
| the label's box | **80** wide, **22** tall, at (0, 0) of the region |
| between label and first character | **16** — `96 − 80` |
| type | `Label/Medium`: **Manrope SemiBold, size 13, line height 18, letter spacing 0.1** |
| colour | `text/primary`, **#102a43** |
| the words | `you`, `alo`, `system`, `program` — **lowercase**, as drawn |

The type and the colour were asked of the live file **for the label node
itself** (`488:29286`) and not for its parent, because a parent's variables are
every variable used anywhere beneath it. Asking the parent would have been a
measurement of the subtree reported as a measurement of the label.

**Four readings off the rendered image agree with the file's own tokens**,
which is why these are recorded as measured rather than as read:

| measured from the 1:1 render | the file's token |
|---|---|
| ink `(16, 42, 67)` | `text/primary` `#102a43` |
| program ground `(238, 242, 244)` | `bg/cool` `#eef2f4` |
| gutter ground `(255, 255, 255)` | `bg/surface` `#ffffff` |
| x-height 7 + descender 3 ⇒ ~13 | `Label/Medium` size **13** |

### Labelled once per region, which the design draws plainly

**The command in the sample below is `alo`, and the command alo OS ships is
`alo-ask`.** That is a deliberate departure, decided by the owner on 2026-10-18
and recorded in
[ADR 0124](../decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md):
there is one command, it both answers and acts, and the name was not the
question. Everything else the sample draws — the proposal, the grant, the
approval before anything runs — is what that command becomes.

The transcript node renders as:

```
you      launch ❯ alo "run this project"
alo      Found one project: ~/Projects/launch…
         Proposed command: npm run dev
         Starts a local server. May write bui…
system   Grant: read this project; ask before…
         Recovery covers project files only i…
alo      Run this command? [y/N]
```

The `alo` region is three lines tall and carries **one** `alo`. That is ADR
0113's *each region is labelled once, not each line*, drawn rather than
described, and it is why `ARegion::labelled_at` exists as one function instead
of a condition each caller writes for itself.

### Where the design and ADR 0113 do not say the same thing

ADR 0113 writes the labels as `PROGRAM · cargo test`, `ALO · response`,
`SYSTEM · observation`, `YOU · submitted request`. The design draws the origin
word alone, in lowercase, and **96 wide cannot hold the longer form** — at the
label's own 13px, `PROGRAM · cargo test` is roughly twice the 80 the design
gives it.

The reading taken, and the reason:

- **The ADR governs which origins exist and that the gutter says them in
  words.** That is what it decided, and it is decided.
- **The design governs how they are drawn** — case, size, family, colour.
  Those are visual properties, and `CLAUDE.md` makes the design the visual
  truth. The uppercase in ADR 0113 is how its examples are written, not a
  stated typographic rule.
- **The qualifier is not dropped, and it is no longer owed — 2026-10-22.** The
  design puts the source detail inside the region as a caption — `Source: cargo
  test output · Not independently verified` — rather than in the gutter, and
  **that caption is drawn**: `the_terminals_cells_are_drawn.rs` fills what
  printed a result into the sentence and places it under the summary, in the
  program's region, which the third rule below asks for.
  *This bullet said it* belongs with rich results, which are not built *from the
  day it was written until 2026-10-22. Rich results were built on 2026-10-18 —
  #761, #803, #804 — and `docs/features.md` carried the same deferral, with the
  same premise, and went stale on the same day. **A note that defers to another
  promise has to name it**, because when that promise lands the deferral is the
  thing somebody has to come back and re-read, and nothing else points at it.*
  **Still to measure**: whether the caption sits at the inset this file records
  — 17 in, 862×22, 38 below the summary's row. The content and the region are
  right; the geometry has not been compared against the frame.

This is written down because it is a deliberate departure from one of two
documents that disagree, and `CLAUDE.md` asks for exactly that: *a deliberate
departure is written down where the next person looks rather than left in a
diff.*

## The note under the transcript, and what it says

The design's `Task scope note`: **1038×28 at the body's row 763**, holding one
text node — `zsh · UTF-8`, **66×16 at (24, 0)** inside the band.

Two things that reading gives which the band alone does not:

- **the text is inset 24**, the transcript's own inset, so the note lines up
  with the first column of output above it;
- **and sits at the top of the band**, not centred in it. The band is 28 and the
  text's line is 16, so the twelve are all below. Measured, because *centred*
  is what anybody would assume.

Its type and colour, asked of the live file for the note's own node
(`479:29034`) rather than for a parent — a parent's variables are every
variable used anywhere beneath it:

| | |
|---|---|
| type | `Label/Small`: **Manrope SemiBold, size 11, line height 16, letter spacing 0.3** |
| colour | `text/muted`, **#596b78** |

**Three sizes for three jobs.** The transcript is 14, the gutter's label 13,
this 11. What a program said, who said it, and what the terminal is — read in
that order, and the sizes are why.

`alo-appearance` already holds `Role::TextMuted` at the design file's own
`#596B78`, so the drawing reads the role rather than writing the value again.

### What it says, and where that comes from

**The shell's name is read from the process that is running** —
`/proc/PID/comm` of the pseudoterminal's child — and not from `$SHELL`. Those
can differ: `$SHELL` is a preference the environment carries, and it can be
stale, inherited, or set by something other than what actually started. The
note exists to tell a person what they are typing into, so a note naming a
shell that is not the one running would be worse than no note. **Where the
kernel will not say, the note is left out rather than guessed**, which
`with_no_shell_named_the_note_is_left_out` holds.

**The encoding is `UTF-8` and is a constant**, because the engine decodes
UTF-8; it is not a setting this terminal has.

**And none of it is in the vocabulary, deliberately.** `zsh · UTF-8` holds no
English — a program's own name, a separator, and the name of an encoding.
`CLAUDE.md` makes hardcoded English a bug and there is none here to
externalise. That is the opposite call from the gutter's four words, which are
English a person reads and needed entries of their own: the test is whether
there is anything to translate, not whether text is drawn.

## A recognised test result, and the four things in it

The gutter above was measured against a frame named **`Rich test result /
untrusted program data`** (`488:29287`), and that frame specifies the rich
result itself as well. It is in `485-28999`, which **is** the authoritative node
for the gutter and for provenance — so unlike the two rich views in
`479:28965`, this one is a specification rather than a rendering reference.

### What is in it, read off the frame

The frame is **894 wide at x 96**, height **262**, inside the 990-wide
transcript boundary. Its children, in reading order:

| | x | y | w | h |
|---|---|---|---|---|
| the summary — `Tests · 23 passed · 1 failed` | 17 | 17 | 862 | 22 |
| the provenance — `Source: cargo test output · Not independently verified` | 17 | 55 | 862 | 22 |
| the program's own text, at length | 17 | 93 | 862 | 88 |
| the actions row | 17 | 197 | 376 | 48 |

and the actions row holds three `Ask alo / Action` instances, at x **0** (100
wide), **108** (100) and **216** (160).

**The inset is 17 and the step between the two one-line rows is 38** — `55 − 17`
— which is the 22-tall row plus 16 of space. Those are the figures a layout
derives from; they are logical measures and go through the display's scale like
every other.

### Three rules the frame states in words, and all three are tests

The frame and its parent carry these as text, and each one is a rule rather
than a caption:

- ***Formatted output is not system evidence.*** So every accessor on
  `alo_terminal::ATestResult` is named `it_said_…`, and there is no method that
  reports a count as a fact.
- ***Text reveals original bytes.*** So a recognised result **never replaces**
  what the program printed — the program's own text is already on the screen
  above the two lines, which is why the table's third row needed no code.
- ***PROGRAM includes parsed rich results.*** So the two lines extend the
  **program's** gutter region, not alo's: a summary read out of a stranger's
  output is still that stranger's claim. A test asserts that alo's own remark
  moves down when a result appears above it, which is the one observable
  consequence of getting the region right.

A fourth, on the parent frame, is **not built and is written down here so it is
not mistaken for done**: *Keyboard focus stays at the shell until a view is
explicitly entered; Escape returns it.* Nothing is enterable yet, so nothing
takes focus — which is the honest state rather than a gap a surface papers over.

### What was built, 2026-10-17, and what was not

**Built:** the summary and the provenance, from the numbers, in the program's
region. The ignored count is shown **only when it is not nought**, because the
design's own summary reads `Tests · 23 passed · 1 failed` over a run with none.

**Not built:** the three `Ask alo / Action` instances. They are the agent's
road, which is a surface of its own, and binding them before it exists would be
the fault `alo-shell`'s `tests/the_road_every_action_takes.rs` refuses.

**Not specified anywhere, and so not built:** the other two rich views.
`479:29079` and `479:29140` are instances of `Terminal / View / Benchmark
comparison` and `Terminal / View / File diff`, **990×320**, and the components
themselves are in no snapshot in this repository — their insides are not
recorded anywhere a reader can check. `479-28965.xml` says so itself: *the
components themselves are not in this capture.* That is why a test result is
the rich view that exists: it is the one the design describes.

## The other two rich views, measured off the live file 2026-10-18

**Measured by the desktop lane, which has Figma access this one does not**, at
the owner's instruction. Recorded here beside the test-result view so the three
can be compared; the capture itself goes into `docs/design/figma-snapshot/` by
that lane, not this one.

Both are symbols, both **990x320**, and both sit in `479:28965` — so both are
**rendering references** rather than specifications, the same standing this
document already gives that node.

### `479:28967` — Terminal / View / Benchmark comparison

| | x | y | w | h |
|---|---|---|---|---|
| the summary — `Build time · 25% lower in this comparison` | 17 | 17 | 958 | 22 |
| the provenance — `bench-014 + bench-015 · Saved at 13:04 · Illustrative data` | 17 | 51 | 958 | 22 |
| the first row | 17 | 85 | 958 | 40 |
| the second row | 17 | 137 | 958 | 40 |
| the scale note — `Same linear scale · Lower is faster · Inspect values when focused` | 17 | 189 | 958 | 22 |
| the actions row | 17 | 223 | 958 | 48 |

**A row is a rule, not four positions**, and the rule is the thing to build to:

```text
label(110) | 16 | bar(value x scale) | 16 | value(110)
```

One 16 px gutter throughout. **The value label follows the bar and is not at a
fixed x** — measured: the label ends at 110 and the bar starts at 126; the
240 ms bar ends at 606 and its value sits at 622; the 180 ms bar ends at 486 and
its value sits at 502. *A value pinned at 622 would be right for exactly this
data and wrong for all other data, and would look right in review — which is the
dangerous kind of wrong.*

**The bars are a scale and not two numbers:**

| | value | bar |
|---|---|---|
| Previous | 240 ms | 480 |
| Current | 180 ms | 360 |

**Exactly 2 px per ms, from a shared origin at x 126**, and that is an
*absolute* scale rather than *the longest bar fills the track*. Two points
cannot tell those apart, so the track was measured: the fixed parts are
110+16+16+110 = **252** of the 958-wide row, leaving **706**. **The longest bar
is 480, not 706** — so it is not filling the track, 226 px are slack, and
2 px per ms fits both points exactly.

**So a bar is its value against the scale, never two constants**, which
`CLAUDE.md`'s *nothing is built to one screen size* requires anyway.

#### Where a tall rich view goes, which the design does not say

**A benchmark is seven of alo's lines** — a summary, a provenance, two rows of
two lines each, and the scale note — and **a diff is two plus a line per changed
line**, where a recognised test result is two.
alo's lines are drawn **beneath the grid**, after all the terminal's rows, and
on a transcript this size there is not room for seven.

**Measured on a nested compositor, 2026-10-18**, with the screen cleared first
so the chart had as much room as it ever gets: the summary, the provenance and
the first row draw, and the first bar measures **480 px wide and 16 tall** — the
design's own figures exactly. **The second row's bar is clipped at the
transcript's foot.**

The design draws its rich views *inside* the transcript at a fixed 990x320, not
beneath it. Where a view taller than the room below the grid should go — inside
the transcript, scrolled with it, or in a window of its own — is not answered
anywhere, and *a thing on your desk that outlives the scroll* in
`docs/features.md` suggests the third without specifying it.

**Recorded rather than chosen**, like the overflow above. What is drawn today is
correct as far as it reaches.

#### The gap this exposes, recorded rather than chosen

**At 2 px per ms a bar overflows once its value passes 353 ms** — 706 px of room
divided by two. The design shows 240 and 180 and nothing near the edge, so
**what a slow benchmark does is unspecified**: clamp, rescale both bars, wrap, or
something else.

*Left open deliberately. This is exactly the sort of thing that gets invented
once and then cited as the design.*

### `479:28986` — Terminal / View / File diff

| | x | y | w | h |
|---|---|---|---|---|
| the summary — `src/auth.ts · 1 removal, 1 addition` | 17 | 17 | 958 | 22 |
| the provenance — `Saved diff · Source: git diff · No changes applied by this view` | 17 | 51 | 958 | 22 |
| the diff body | 17 | 85 | 958 | 110 |
| the actions row | 17 | 207 | 958 | 48 |

**The body is one text node, not one per line**, rendering as five rows — 12,
13, −14, +14, 15 — with the signs in a gutter left of the numbers.

**The removal marker is not an ASCII hyphen**, and that is as far as the
measurement goes. What was observed is the glyph as the connector returned the
layer name and as it rendered: a long centred dash, aligned with the plus's
crossbar, visibly not `U+002D HYPHEN-MINUS` in that mono face.

**It was not byte-verified, and *exactly U+2212* is an inference rather than a
reading.** The lane that measured it said so unprompted: a connector hands back
text, and anything retyped through a person or an agent is a measurement of the
retyping. *So the thing worth holding in a test is that the two markers are
distinct and that neither is the ASCII hyphen* — not a codepoint nobody read off
the file. If the exact codepoint ever matters, read it from `479:28989` or from
the committed snapshot, and say in the test which of those it came from.

### The two sources disagree about the actions row, and nothing here settles it

| source | x positions | widths | row | contents end | layout |
|---|---|---|---|---|---|
| `485:28999`'s `Rich test result` (`488:29291`) | 0, 108, 216 | 100, 100, 160 | 376 | 376 | **hugs** |
| `479:28967` and `479:28986` (`479:28979`) | 0, 128, 256 | 120, 120, 176 | 958 | 432 | **fills** |

> **An earlier version of this section said the specification wins, and that was
> an over-reading.** *Corrected 2026-10-18 after the desktop lane pushed back on
> it.* `ROOTS.md` does say **`485:28999` is the contract; `479:28965` is only
> rendering** — and its very next sentence scopes that: the frames defer **for
> the source gutter, autonomy and recovery**, and *a reader taking **provenance
> rules** from the rendering node has taken them from the wrong one*. **Nothing
> in either text claims authority over a button's width.** The headline is
> broader than the body, two readers took it differently, and the body binds.

**They are the same three actions, not two sets.** `488:29287` renders *Text ·
Expand · Open on canvas*, identical to the 479 frames.

**And the difference follows no scale**, which is what rules out *a different
frame at a different width*:

```text
frame ratio 990 / 894 = 1.1074
  100 x 1.1074 = 110.74   measured 120   does not fit
  100 x 1.1074 = 110.74   measured 120   does not fit
  160 x 1.1074 = 177.18   measured 176   fits
```

Two of three do not follow the frame width. They are not hug-content either —
the labels are identical, so hugging would give identical widths, and 100 is not
120. **They are fixed widths set differently in two places, and one of them is
stale.** There is no rule to recover.

**What does agree, and is safe to build to:** the **8 px** gap between buttons
and the **48** height, in both.

#### The layout mode is the disagreement that changes code

The specification's row **hugs** — 216+160 = 376, exactly its contents, no
slack. The rendering's row **fills** — 958 wide with the buttons ending at 432
and **526 px of empty frame** after them.

**That is not a number, it is a behaviour**, and it is the one that matters:
a hugging row and a filling row do different things the moment a label is
translated into a longer language, which this repository promises for 24 of
them.

**Left for the owner.** Either lane could pick one and be plausible, and that is
the reason not to: this is better visible as an inconsistency than settled by
reading authority into a note that was scoped to something else.

### The three labels are in no layer name

**Text · Expand · Open on canvas**, left to right. Expanding the instance gives
one text child named `Quiet`, which is the Button's style variant and not its
label — so these were read off a render, the same road this document used for
the gutter's type and the walk used for step 10's summary.

**What the three actions *do* is not unblocked by having their words**, and the
reason changed on 2026-10-18.

*This paragraph said the actions needed* a road from the terminal to the agent
which nothing constructs yet *until 2026-10-21. **That road exists and is
constructed in product code.*** `WhoAsked` gained a second arm on 2026-10-18 —
`TheTerminal`, beside `ATaskWindow(WhichTask)` — and
`crates/alo-shell/src/direct_desktop.rs` asks on it at line 838 and handles the
answer at line 1092. Neither site is a test; measured 2026-10-21, it is the only
non-test file that names the arm. `what_a_person_asked_goes_to_the_agent.rs`
records why the arm could be added at all: the order had said widening it
*cannot be built alone* because nothing would ever construct a second arm, and
**the owner's decision that an explanation comes through the agent is what
constructs it** — `Ctrl`+`X` `Ctrl`+`A` travels this road today.

**What is still not decided is the row the buttons sit in**, which is the
disagreement two sections above: the specification frame's row hugs its contents
at 376 and the rendering frames' fills at 958 with the buttons ending at 432.
That is the live blocker, not the road, and it is a behaviour rather than a
number — **a hugging row and a filling row diverge the moment a label is
translated**, and `docs/features.md` promises all 24 official EU languages by
name. The three labels are not in the vocabulary yet, so there is nothing
translated to measure; what a longer label costs is therefore an estimate and
not a reading.

*Put to the owner on 2026-10-21 with a recommendation, and recorded here so the
next reader neither re-asks it nor guesses: **each button hugs its own label
while the row fills the width and keeps them left-aligned.** It keeps every
label readable in every language, which a fixed button width does not; it keeps
the row's position and height stable, which is the accessibility argument for
filling; and it makes no button width a constant. It leaves one thing open that
the owner was asked in the same message — what happens when the three together
are wider than a narrow window, where wrapping to a second line is what keeps
every label readable. **Unanswered as this was written**, so nothing is built
from it.*

## The type, asked of the live file

The snapshot has no type information, so these came from the file's own variable
definitions on 2026-10-14 — `479:28965` for the first and the transcript node
`487:29034` for the second.

| token | font |
|---|---|
| `Label/Medium` | Manrope, SemiBold, size 13, weight 600, line height 18, letter spacing 0.1 |
| `Terminal/Command` | **Geist Mono, Medium, size 14, weight 500, line height 22, letter spacing 0** |

**`Terminal/Command`'s line height is 22**, which is the figure the 123 source
regions were already measured at. **Two independent sources agree**, and that
agreement is the reason to trust the number: geometry alone could have been a
coincidence of the frames, and a token alone could have been unused.

### The weight is load-bearing, and it did not work

`Terminal/Command` is **Medium, weight 500**. Asked for exactly that, the
terminal's text was drawn in **Inter** and was not monospace — because fontdb
registers `GeistMono[wght].ttf` as one face at weight 400 and cosmic-text
honours a family only at an exact weight match. `docs/quirks.md` carries the
mechanism, and `fonts/GeistMono-Medium.ttf` is the static cut that fixes it.

**So a terminal drawn at `Attrs::new()`'s default weight would have looked
correct while being wrong at the design's own weight.** That is why this
document records the weight beside the size rather than only the size.

## The body's panel and hairline, measured 2026-10-28

**Why this was measured.** The boot walk of 2026-10-28 found the terminal's text
floating on the desktop with no window round it: the body was painted in
`Token::Cream`, which is `bg/canvas` — the canvas's own colour.

Asked of the live file for the body node itself, `479:29012` *Terminal only /
01 · Quiet light*, with `get_variable_defs`, and rendered with
`get_screenshot`:

| | token | value |
|---|---|---|
| panel | `bg/surface` | `#ffffff` |
| hairline | `border/default` | `#e7ebef` |
| corners | `radius/lg` | 18 |
| shadow | `Elevation/Floating` | drop shadow `#07131F24`, offset (0, 16), blur 40, spread −4 |

The render is 1112×872 for a 1040×800 node — 36 on each side, which is the
shadow's reach — and shows the transcript on the panel's own white with no
ground of its own, and no rule between the identity strip and the transcript.

**Built:** the panel in `bg/surface`, the hairline in `border/default` at
`A_BORDER` through the display's scale (never less than one pixel), and the
transcript on the panel's white — `crates/alo-shell/src/the_terminals_frame.rs`.

**Built 2026-10-28: the 18 corners and the shadow**, by the Dock's own
rasteriser. `dock_surface` used to be fixed to the Dock's shadow (0 8 24, 8%)
and took no display scale; it now takes a look — corner, fall, blur, spread and
darkness — together with the scale that look is drawn at, in
`crates/alo-shell/src/how_a_panel_is_raised.rs`. The terminal's body asks for
`Elevation/Floating` at its display's real scale, so at 200% the corner is 36
pixels and the shadow falls 32.

| | logical | at 100% | at 200% |
|---|---|---|---|
| corner, `radius/lg` | 18 | 18 | 36 |
| shadow fall | 16 | 16 | 32 |
| shadow blur | 40 | 40 | 80 |
| shadow spread | −4 | −4 | −8 |
| shadow colour | `#07131F` at `0x24` (36 of 255) | the same | the same |

**The blur is drawn the way the Dock's always has been** — the same distance
function, faded across the blur's width with a smoothstep — not a true
Gaussian; `dock_surface`'s header gives the reason. **The Dock still draws at
scale one**, because its band, slots and marks are laid out unscaled and
rounding its corner alone at a dense display's scale is the fault
`dock_raster` records. Its pixels are unchanged: `dock_surface`'s tests hold
a hash of every pixel of a Dock with every kind of shape on it, taken from the
rasteriser before this change.

**Output sits on white everywhere — the owner's decision, 2026-10-28.** Two
readings of the design disagree: the gutter section above measured a
`Terminal / Source region` in `485-28999` with the program's text on
`bg/cool` and the gutter on `bg/surface`, and this node's render draws the
whole transcript on `bg/surface`. The owner chose white, which is what was
built, and nothing changed. **A program's region has no ground of its own**;
`bg/cool` in `485-28999` is not followed, and this paragraph is where the
next reader learns that it was decided rather than missed.

**Also not built, each for a reason:**

- **the external window edge** above the body (1040×44, `479:28999`): it is an
  interactive surface — drag, minimise, maximise, close — and this terminal is
  not yet a window on the canvas that any of those could act on. Controls that
  reach nothing would look live.
- **the strip's menu** — *search, new session, settings*, the three dots at
  (936, 6), 44×44. Nothing it names exists for it to open, so it would look
  live and reach nothing. Its room is kept, so the words beside it do not move
  when it arrives.
- **the dark body**, which has not been read: the dark pair in `500:29477` is
  owed. The dark ground and ink are the ones drawn before.

## The strip across the top, built 2026-10-28

`Project and session identity`, `479:29013`, asked of the live file on
2026-10-28 for its own nodes. It is a row: 24 of padding each side, 6 above and
below, a gap of 12, items centred.

| node | text | where | type | colour |
|---|---|---|---|---|
| `479:29016` | `~/Projects/launch` | (24, 19), in a 740-wide box that clips | `Label/Medium`, Manrope SemiBold 13 on 18 | `text/muted` |
| `479:29018` | `Local · disan` | (854, 19), 70×18 | Manrope SemiBold **12** on 18, no style variable | `text/muted` |
| `479:29019` | the menu | (936, 6), 44×44 | — | — |

**What each says is the owner's decision of 2026-10-28**, because the file does
not say:

- **the folder is the one the shell is in now**, following the person as they
  change folder, with `~` for their home. It is read from the kernel's
  `/proc/PID/cwd` for the shell's own process **when the shell has written
  something** — a `cd` is always followed by a prompt — and at no other time.
  Nothing watches it, and no shell integration is needed: alo's integration
  marks (ADR 0123) carry no folder, and a strip that only worked with a snippet
  installed would be blank on most machines.
- **`Local` means the shell runs on this machine**, not on a paired one. It is
  a word in the vocabulary (`terminal.where.this-machine`), joined to the name
  by the design's own ` · `.
- **the name is the signed-in person's** — the name they typed when they made
  their account. It is in the account store the sign-in screen reads
  (`/etc/alo/accounts.toml`, root's, mode 0600), which the desktop must not
  read, so at every sign-in `alo-giving-a-name` copies it into
  `/var/lib/alo-names/<number>`, readable by that person alone, and the desktop
  reads it once
  ([ADR 0136](../decisions/0136-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md),
  `docs/contracts/the-persons-name.md`). **Where there is no file** — an account
  that has not signed in since, or a machine where the helper could not run —
  the words read `Local` alone, never the desktop's own login, `alo`, which is
  nobody's name.
- **A name too long for its room ends in an ellipsis.** *A reading, recorded
  so it can be disagreed with:* the design hugs `Local · disan` and says nothing
  about a longer name. The words keep the room they already had — from 24 in to
  114 before the edge — and are fitted with `crate::fitted`, the shell's rule
  for a window title: cut at a whole letter, ending in `…`, so `Local · ` stays
  whole and the folder gives way first.

**Where the words go at another width is a reading, recorded so it can be
disagreed with.** The design's row hugs its contents with a fixed 740-wide
folder box, so at 1038 it leaves 34 of slack after the menu. A fixed 740 holds
at one width only. The reading taken: **the folder starts 24 in; the words end
114 before the strip's right edge**, which is where the design puts them at its
own width (854 + 70 = 924 = 1038 − 114); the folder is cut 12 before the words.
Both go through the display's scale.

## Colour

**Not in the snapshot, and not yet taken off the file for the terminal.**
*Partly superseded on 2026-10-28: the body's panel and hairline were taken off
the file, in the section above.* What
is known is the palette the rest of the shell draws from —
`palette.toml` and
[ADR 0093](../decisions/0093-the-palette-is-twelve-semantic-roles-and-the-design-file-decides-their-values.md)
— and that the terminal's own frames come in **light and dark pairs**:
`500:29477` is named *the detailed states, light and dark*, and `485-28999.xml`
carries `· Light` and `· Dark` variants of each state.

The variable definitions read from `479:28965` give the surrounding page's
roles — `bg/surface #ffffff`, `bg/canvas #faf7f2`, `text/primary #102a43`,
`text/secondary #274c68`, `text/muted #596b78`, `border/default #e7ebef`, radii
8 / 12 / 18, spacing 12 / 16 — but **the transcript's own ground and ink are not
among them**, so they are owed before the transcript is drawn and are
deliberately absent here rather than guessed.

## How every figure above becomes code

**None of these numbers may reach a display as written.** `CLAUDE.md`: *nothing
is built to one screen size — the design's frames are 1440×960 and no machine is
obliged to be.* These frames are 1440×1040, which makes the point for itself.

So each figure is one of the three honest kinds:

- **a logical measure**, declared once in a measures file and converted for each
  display through that display's own scale, as `alo_dock::measures::ICON` is:
  the line height 22, the inset 24, the strip height 56, the border 1, the edge
  44;
- **a proportion** of the display, for anything the window's own extent is taken
  from;
- **a rule** that was never a number, such as *the transcript fills what the
  identity strip leaves*.

And the fourth kind, the one to watch: **another surface's current extent.** The
transcript's height is not 706 — it is *what the body leaves after the identity
strip*, and 706 is what that came to at 800. A figure like that can only be
asked for, never stored.

**The grid follows from the cell, not from a count.** How many columns and lines
a terminal has is `transcript width / cell advance` and `transcript height / 22`,
both measured from the face at the size the display is drawing — never 80×24,
which is a figure a test chooses and not one a window has.
