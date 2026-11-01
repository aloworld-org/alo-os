# The Ask alo task window, in numbers

**What this is.** The figures measured off the design's own frames for the window
a request to alo opens, so a layout built here can be checked against the design
rather than matched by eye. Read live on 2026-10-12 from file
`nDxyF5Ho9oC4RObjVzwBNJ`, section `417:27657` *Ask alo · Task conversation*.

**Two reads, because one of them is blind.** The committed tree is in
`figma-snapshot/417-27657.xml`, from `get_metadata`. That call returns geometry
and layer names and **nothing about appearance** — no fill, no stroke, no
typography, no component description. Everything in this file below *The tree's
figures* came from a second call, `get_design_context`, on the same section and
the same day. Neither call alone would have produced this page, and the snapshot
procedure in `figma-snapshot/MANIFEST.md` records only the first.

*This is the same gap that hid a stroke on a frame from an earlier capture: a
stroke is a property of a frame and not a child layer, so the tree cannot show
it. Appearance questions are answered by `get_design_context` or not at all.*

## The two target sizes are two surfaces, not a disagreement

The window edge's controls are **44 × 44** and the composer's are **48 × 48**,
in the same frame, and both are correct.

- **44** is the external window edge — `the-external-window-edge.md`, settled by
  the owner on 2026-10-09. This capture confirms it to the pixel: a 44-high
  interaction region, a 32-high visible strip at y 12–44, three non-overlapping
  44 × 44 targets at x 896, 944 and 992, and artwork at x 15 of 44.
- **48** is a control *inside* the window, on the composer. The four composer
  components each carry the description *48px minimum target*.

That same document records **48 as superseded** — but what it superseded was an
internal 48px band on the *window edge*, which is this first surface and not the
second. **A later reader who makes these two numbers agree will break one of
them.** The owner's own written requirement of *targets ≥ 48 × 48 logical*
reads onto the composer, which meets it; the edge is the settled exception with
its own document.

## The tree's figures

Every frame in the section is 1440 × 1040 and holds the same two things: an
`External window edge` 1040 × 44 at (200, 80), and an instance of
`alo / Task conversation` 1040 × 800 at (200, 124).

**1440 × 960 is the frame, not a machine.** Every figure below is a logical
measure to be declared once and converted through the display's scale, or a
proportion — never a constant that holds only at the size somebody drew.

### The conversation, 1040 × 800

The frame is 1040 wide with a 1px border on each side, which is why every row
inside it is **1038**. Five rows, top to bottom:

| Row | Height | Padding | Inner width |
|---|---|---|---|
| Task heading | 76 | px 32, gap 16 | 974 |
| Explicitly shared context | 52 | pl 32, gap 8 | — |
| Conversation / scroll | 484 | p 32, gap 20 | 974 |
| Reply composer | 150 | pt 8, px 32 | 974 |
| Task scope note | 28 | pl 32 | — |

- Surface `bg/surface`, border `border/default`, radius `radius/lg`, effect
  `Elevation/Floating`.
- **Heading**: `Identity` 680 × 54 (gap 4) and `Task status` 260 × 40, right
  aligned.
- **Shared context**: chips 184 × 36, radius `radius/sm`, `bg/cool`, pl 12,
  then a count label.
- **Conversation**: scrolls on y, clipped on x. A person's request is right
  aligned in a bubble **660** wide, p 16, radius `radius/md`, `bg/cool`. alo's
  reply is **920** wide, gap 8, no bubble. An activity block is **920** wide,
  p 16, radius `radius/md`, `bg/cool`.
- **Composer**: a `Message input` 974 × 128, radius 18, border
  `border/default`, `bg/surface`, pt 16 pb 12 px 12, gap 16, holding the
  placeholder and then the control row.

### The control row, 948 × 48

Left to right, gap 8 throughout:

| Control | Width | Radius | Fill | Artwork |
|---|---|---|---|---|
| Attach | 48 | 12 | `bg/cool` | 28 |
| Model | 176 | 12 | `bg/cool` | label + 24 chevron |
| Processing destination | 148 | — | none | a 16 cloud and 6, while a provider answers, then the words |
| Permissions | 48 | 12 | `bg/cool` | 28 |
| Flexible space | 440 | — | — | — |
| Send / Stop | 48 | 24 | `text/primary` | 28 |

**The cloud, since 2026-10-31.** The alo window the owner approved that day
(page `544:30910`; `549:112` in `549:26`) draws a 16 cloud (`549:113`), then 6,
then the words. It is drawn while a provider answers; the approved frames show
no other destination, and a cloud beside *On this computer* would say the
opposite of the words.

**Open: the approved window draws the destination 283 wide, and this row keeps
148.** Taking 283, with the flexible space giving up the difference, raises the
narrowest window the row fits in from 600 to 735, so a task window on a small
screen would be refused rather than drawn. How the row gives way on a narrow
window — the destination shrinking to its words, the flexible space going
first, or a narrower window being refused — is the owner's to decide.

**The artwork frame is 28 and the chevron is 24, with 2px strokes**, from the
component descriptions — and the artwork frame is not the target. The target is
48. *A glyph is not a target*; one constant for both is a fault this repository
has already made once.

### Three sums that had to agree, and did

A single figure cannot be caught being wrong. These can:

- The control row's eleven widths and gaps sum to **948**, which is the row's
  declared width. Agrees.
- The frame is 1040 and each row is 1038, which is the border on each side.
  Agrees.
- `Message input` is 974 with px 12, leaving 950 for a 948 row — see below.

### And two that did not

**The five row heights sum to 790, inside an inner height of 798.** Eight
logical pixels are unaccounted for, and a layout derived from these five
heights will be 8 short of the frame. The residue is not labelled in the
design. It is recorded here rather than silently distributed, because an
implementation that picks a row to absorb it has made a decision the design did
not make. The heading's row has a comparable 18 of horizontal slack after
680 + 16 + 260, which `justify-end` on the status accounts for; the vertical 8
has no such explanation in the export.

**And the control row is two narrower than the space it starts in.** The reply
field is 974 with 12 of padding each side, which leaves 950 — and the drawn row
is **948**, starting at x 44, which is 32 + 12 and so takes no account of the
field's own border. The row is therefore inset 12 on its left and 14 on its
right.

*Found by a test rather than by reading*, when a layout derived from
974 − 24 put the one button at 946 against the design's 944. Both residues are
reproduced rather than tidied in `alo_tasks::layout`, because the figure that
matters is where the button lands; taking the two off the left instead would
move every control on the row.

## Tokens

Named in the frame, with the fallback the export carried:

| Token | Value |
|---|---|
| `bg/surface` | white |
| `bg/cool` | `#eef2f4` |
| `border/default` | `#e7ebef` |
| `text/primary` | `#102a43` |
| `text/secondary` | `#274c68` |
| `text/muted` | `#596b78` |
| `accent/default` | `#0f6b72` |
| `radius/sm` | 8 |
| `radius/md` | 12 |
| `radius/lg` | 18 |

**`radius/lg` is new to this repository.** `figma-snapshot/variables.toml`
carries `radius/sm` and `radius/md` only, and both under `[unadopted.radius]`.
This capture is the first to need 18.

## Typography — Manrope throughout

| Style | Size / line | Weight | Tracking |
|---|---|---|---|
| Heading/Medium | 22 / 30 | SemiBold 600 | −0.2 |
| Label/Medium | 13 / 18 | SemiBold 600 | +0.1 |
| Label/Small | 11 / 16 | SemiBold 600 | +0.3 |
| Body/Large | 16 / 25 | Regular 400 | 0 |
| Body/Medium | 14 / 22 | Regular 400 | 0 |

The task title is Heading/Medium; speaker labels, the status and control labels
are Label/Medium; the `alo task · Work` line and the shared-file count are
Label/Small; message bodies are Body/Large; activity lines are Body/Medium. The
processing destination is **12 / 22 Regular**, which matches no named style.

**Bold, headings and code inside alo's answers** are drawn in styles from
this table — the owner's decision of 2026-10-28, in
[emphasis-in-alos-answers.md](emphasis-in-alos-answers.md).

## The seven states, from the component set

**`417:28422` is the component set, and one read of it returned every variant.**
This matters for how the rest of this page was obtained: a read of the *section*
(`417:27657`) expands only the instance it has selected, so seven of the eight
frames came back as bare 1040 × 800 boxes. Reading the component set instead
returned all seven variant masters with their interiors.

*An earlier draft of this file said each state "needs its own read before it is
built". That was wrong — it described the section read's limit as though it were
the design's. One call to the component set was enough.*

| Variant | Master | Status line | Status colour |
|---|---|---|---|
| Working | `417:27666` | alo is working | `accent/default` |
| Needs permission | `417:27766` | Waiting for your permission | `text/primary` |
| Model picker | `417:27874` | alo is working | `accent/default` |
| Permissions | `417:27996` | alo is working | `accent/default` |
| Stopped | `417:28116` | Stopped | `text/primary` |
| Complete | `417:28216` | Complete | `text/primary` |
| Dark | `417:28322` | alo is working | `accent/default` |

### Teal is conditional, and the condition is legible

**Teal appears only while alo is working.** The moment the task is waiting on
the person, stopped or finished, the status line is `text/primary` navy. The
`alo` speaker label on a reply stays `accent/default` in every variant, because
it marks who is speaking rather than what is happening.

This is ADR 0067's reservation of teal for alo, drawn: teal is *alo acting*,
not *alo's surface*. Together with *keyboard focus uses a visible navy outline*
on all five composer components, this section is consistent evidence that focus
is never teal.

### Which action the one button shows

| Variant | Button |
|---|---|
| Working, Needs permission, Model picker, Permissions, Dark | **Stop** `427:28086` |
| Stopped, Complete | **Send** `427:28090` |

**No variant draws both, and none draws neither.** Send and Stop are separate
components sharing one 48 × 48 slot at x 944, radius 24, fill `text/primary`,
artwork 28.

**Needs permission shows Stop**, which settles a question the written
requirement raised: a task awaiting approval is still a task alo is in the
middle of, so the control is Stop and not Send.

## Stopping is specified in words and not drawn

The section has a frame `418:28135` *Ask alo / 08 · Stopping*. **The component
set has no `Stopping` variant** — the seven above are all of them — and that
frame's instance `418:28150` therefore shows one of the seven.

So the state the contract describes most precisely is the one state with no
drawn appearance:

> Stop requests cancellation: show **Stopping** until acknowledged, then return
> to Send. … Disable … repeated cancellation.

What this leaves undefined: the status line's words and colour while stopping,
and how a disabled Stop is drawn. **Both have to be decided rather than
measured, and this file is where that is recorded.** They are not in the export
and must not be reported as if they were.

## Closing: the edge carries Close, and a running task does not close yet

`docs/features.md` promises that closing a running task asks. **The owner
decided on 2026-10-28**: the frames' `External window edge` is drawn on task
windows with **Close** only — minimise and maximise come with the right-hand
panel — and closing a running task will offer **Stop and close** and
**Cancel**, with *Keep running* added when the panel holds tasks.

**What is built:** the edge, laid out by the same code as a client window's,
always revealed, Close 44 × 44 at its trailing end, scaled with the display. A
task window is placed with the edge's 44 above it inside the view, so Close is
never off the screen. Close and Alt+F4 are one door: a finished or stopped
task closes; a running task asks *Stop and close* or *Cancel* (ADR 0139), and
closing never answers a waiting change for the person.

**The window in front keeps the keyboard** — the owner, 2026-10-30: *like every
desktop, the window you can see in front gets your typing.* Escape empties the
reply box and does nothing more. Closing the window that holds the keyboard
hands it to the task window now in front, or, with none left, lets it go.
*Withdrawn the same day by the owner's decision:* until 2026-10-30 Escape on an
empty reply box gave the keyboard up "so a person can always get back to their
applications". On a booted machine the window stayed drawn in front, every key
went to the terminal hidden behind it, and Alt+F4 found no window to close.

**The window you go to comes in front** — the owner's decision of 2026-10-31,
which settles the question this paragraph left open on 2026-10-30. Switching
to the terminal or an application — Super+Tab, Alt+Tab, Ctrl+Alt+T, a click, or
the Dock — brings it on top of any task window. The task window stays open
behind it, **one Super+Tab away**: the next-window shortcuts go round the
applications, then the terminal, then every open task window. A click goes to
whatever is drawn on top where it lands and brings that in front. So nothing
can type into a window hidden behind another, and no window is buried where
the person cannot find it (`docs/features.md`, *Every goal is a canvas*).

Until then every task window was drawn over the terminal and every
application, in a fixed order, so a shortcut from a task window sent typing to
a window the task window still covered. The order is between the three kinds
of window, not inside one: task windows keep theirs, the last opened in front.
Built in `crates/alo-shell/src/which_window_is_in_front.rs`, which the painter,
a press and the keyboard all read.

## The approval card, 920 × 152

`417:27797`, inside the conversation scroll, `bg/cool`, p 16, radius
`radius/md`, gap 8 — the same shell as an activity block, 32 taller.

| Line | Style | Content in the frame |
|---|---|---|
| Action | Label/Medium | Save a new file |
| Target | Body/Medium, 872 wide | Presentations / Launch presentation.odp |
| Effect | Body/Medium, same block | Creates one file. Existing files will not be changed. |
| Choices | row 872 × 48, gap 8 | **Allow once** (Primary, 144) · **Deny** (Quiet, 104) |

**Allow once and Deny sit side by side in one row**, which is the drawn form of
*equally reachable*. Deny is narrower because its label is shorter, not because
it is secondary — both are 48 high, both carry a border, and neither is hidden
behind a disclosure.

**Superseded for the approval card by the frame below**, which the owner
approved on 2026-10-31. The closing question (ADR 0139) still borrows this
card and its 104-wide Deny.

## The approval card, as approved — `549:193`, 920 wide

Card `549:193` in frame `549:135` *alo window · Waiting for your permission ·
1440×960 · Light*, page `544:30910`. **Measured with the Figma connector's
`get_design_context` on 2026-11-01, once, by hand**; what holds the build to
it on every build is `crates/alo-shell/src/the_approval_card_tests.rs` (the
figures) and `task_raster_tests.rs` (what is drawn), and neither asks Figma.

The card: `bg/surface`, a 1 `border/default` hairline, corner 18, shadow
*Elevation/Quiet* (0, 8, blur 24, `#07131F` at 8%), a column, clipped.

| Part | Node | Figures | Style | In the frame |
|---|---|---|---|---|
| Who is asking | `549:194` | row, gap 8, pt 16, px 20; mark 12 | Label/Small, `accent/default` | *alo is asking to* |
| What will change | `549:198` | column, gap 12, pt 6, pb 16, px 20 | | |
| · the action | `549:199` | full width | Heading/Medium, `text/primary` | *Save a new file in Presentations* |
| · the file | `549:200` | `bg/cool`, corner 12, p 12, gap 12; a 40 white tile, corner 10, holding a 20 *plusFile* | name SemiBold 14/22; path System/Mono 12/18 `text/muted` | *Launch presentation.odp* · *Presentations / Launch presentation.odp* |
| · what changes | `549:209` | row, gap 20; each a 16 *check*, 8, then words | Body/Medium, `text/primary` | *Creates one file.* · *Existing files will not be changed.* |
| Choices | `549:220` | `bg/canvas`, a 1 `border/default` rule above, px 20, py 14, gap 8 | | |
| · Allow once | | **144** × 48, Primary, corner 12 | Label/Medium on `text/primary` | |
| · Deny | | **144** × 48, Quiet, corner 12 | Label/Medium | |
| · beside them | `549:226` | after a flexible space, to 20 from the right | Body/Medium, `text/muted` | *Allows this one action. alo asks again next time.* |

**Allow once and Deny are now the same width**, 144 each: the approved frame
draws *equally reachable* as equally sized, which replaces the earlier card's
104-wide Deny.

### What is built, and what is not

Built in `crates/alo-shell/src/the_approval_card.rs` and `task_raster.rs`:
the card's white and hairline at corner 18, *alo is asking to* with alo's mark
(`crates/alo-shell/icons/the-approval-card/alo-mark.svg`, the design's own
export), the action in Heading/Medium — the very sentence the person approves —
and the row of choices in `bg/canvas` under its rule, with both answers 144 and
the line beside them. With a one-line action the card is **161** tall
(16 + 16 + 6 + 30 + 16, then 1 + 14 + 48 + 14).

**Not built, because nothing on the machine knows them**: the file row
(`549:200`) and the lines saying what will and will not change (`549:209`). A
change reaches the shell as one sentence (`alo_protocol::Standing`), which
carries neither the file it names nor what it leaves alone, and no verb
declares what it leaves alone (`docs/contracts/agent-verbs.md`, *The shape of a
verb*). Both are additions to a public contract, and the second is a sentence
per verb that promises a person something, so they wait for the owner's
decision. The card leaves no empty room for them.

**One departure, written down here**: the card has no shadow of its own. The
panel rasteriser draws one shadow, the window's, and a second under one card is
a change to that rasteriser rather than to this card.

### The action component both popovers and the card are built from

`AskAloAction`, two styles, 48 high, radius `radius/md`, border
`border/default`:

- **Quiet** — `bg/surface` ground, `text/primary` label.
- **Primary** — `text/primary` ground, `bg/surface` label.

Label is Label/Medium in both. Width is per use: 104 (Deny), 144 (Deny's
sibling default, View activity), 180 (Open presentation), 380 (every row inside
a popover).

## The two popovers, and the rule they share

| | Model picker | Permissions |
|---|---|---|
| Node | `417:27921` | `417:28043` |
| Size | 420 × 396 | 420 × 398 |
| Position | left 101, top 274 | left 441, top 272 |
| Title | Model and processing | Permissions for this task |

Both: `bg/surface`, border `border/default`, radius `radius/lg`, p 20, gap 12,
shadow `0 16px 40px -4px rgba(7,19,31,0.14)`, rows 380 wide.

### *Popovers open upward*, in numbers

Two independent instances give the same two relationships, which is what makes
this a rule rather than two positions:

- **Anchored to the button's left edge.** The Model button's left edge is at
  x 100 and its popover is at 101; the Permissions button is at 440 and its
  popover at 441. Both are +1, which is the border.
- **Bottom 18 above the control row.** Both popovers end at y 670. The control
  row begins at y 688, derived as 76 + 52 + 484 rows, + 8 composer padding,
  + 128 input height − 12 bottom padding − 48 row height.

Neither popover overlaps the composer, and neither extends below it. **The
derived figure is the 18, not the 274** — a layout that hard-codes top 274 will
be wrong on any window that is not 800 high.

### What each popover says

**Model and processing** — *Changes apply to the next request. The current
action keeps its model.* Then `SmolLM2 1.7B · Selected`, the label *On this
computer · Current model*, then *Connect a network machine…* and *Choose a
provider…*, and the footnote *External processing needs a visible choice before
conversation or files are sent.*

The three rows are the three inference sources a person can pick, and the
footnote is the no-silent-fallback rule stated where the choice is made.

**Permissions for this task** — *Only what you explicitly shared.* Then
`Read · 2 attached files` and *Manage shared files*; then three lines each
reading **Ask first** — *Create or change files*, *Operate applications*,
*Send or publish*; then *Revoke file access*; then the footnote *Revoking
blocks further access. It cannot undo a completed action or recall content
already sent.*

**Ask first is the drawn default for all three capabilities**, and the
revocation footnote is the honest limit of revocation written into the surface
rather than left to an implementer.

## The shadow — the export contradicts itself, and one side is provable

`Elevation/Floating` is defined as `DROP_SHADOW`, colour `#07131F24`, offset
(0, 16), **radius 40, spread −4**.

The conversation frame rendered it as `drop-shadow-[0px_16px_20px_rgba(7,19,31,0.14)]`
— radius 20, no spread. **But both popovers rendered the same effect as
`shadow-[0px_16px_40px_-4px_rgba(7,19,31,0.14)]`** — radius 40, spread −4,
matching the definition exactly.

So this is not a judgement about which to trust. The same effect came back two
ways in one response, and the form that agrees with the style definition is
also the form that can express spread at all. **Radius 40, spread −4, and
`#07131F24` is 14% alpha.** An implementation taken from the conversation
frame's figure would be visibly wrong and would look measured.

## What the export does not carry

Named explicitly, so none of it is mistaken for measured:

1. **Stopping has no drawn appearance.** See above. The status wording and the
   disabled Stop must be decided.
2. **Dark has a variant but no dark values.** `State=Dark` (`417:28322`)
   returned every colour as the same token with the same *light* fallback —
   `text/primary` as `#102a43`, `bg/surface` as white — and differs from
   `Working` only in node ids. The tokens are semantic and resolve per
   appearance, and this export flattened them to one mode. **No dark value for
   any token was returned, and none may be inferred from this capture.**

   **The owner's decision, 2026-10-12: build light first.** This takes nothing
   away, and the reason is worth being exact about, because law 5 turns on it.
   *Light and dark, following the time of day if a person wants* is promised in
   `docs/features.md`, and **no surface in this repository honours it yet** —
   `crates/alo-appearance/src/role.rs` says so in its own words: the design
   binds the twelve role names to different values in a dark mode and *none of
   that is here, because dark-theme parity needs its own measured* work. So a
   light-only task window is not this window withdrawing a choice; it is this
   window being consistent with every other surface, and dark remains a promise
   nobody has built. **Never offering is not taking away. Withdrawing is.**

   It is also the only thing this capture could have supported: there are no
   dark values in it to build from.
3. **Hover, pressed, disabled and focus are words, not geometry.** *Hover uses
   `bg/cool`* and *focus uses a visible navy outline* are component
   descriptions; no stroke width, offset or colour came back for either.
4. **The icon artwork.** The five composer icons came back as SVG asset URLs on
   Figma's own server, which expire after seven days. The 28/24 frame and the
   2px stroke are recorded, and **the URLs in the response must not be treated
   as a source.** *Since 2026-10-30 the paths are committed*: fetched again that
   day, one `download_assets` call per node, and kept byte for byte in
   `crates/alo-shell/icons/the-reply-box/` with the node each came from
   (ADR 0142). The icon frames are `427:28075` (Attach), `427:28079` (the
   model's chevron), `427:28083` (Permissions), `427:28088` (Stop) and the
   one inside `427:28090` (Send). *Checked on 2026-10-31* against the alo
   window the owner approved that day (page `544:30910`, frames `549:26` and
   `549:261`): the same components, and the same five files byte for byte.
5. **The example content is example content.** *Prepare the launch
   presentation*, `SmolLM2 1.7B`, `Launch brief.pdf`, `Launch presentation.odp`
   and `2 files shared with this task` are illustrative, and the component
   description says so twice. **Never simulate a completed action in
   production.** The model name in particular is a drawing, not this machine's
   catalogue.
6. **The 8px vertical residue** above has no label in the design.
