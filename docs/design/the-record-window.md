# The record window — *What alo did*

The window a person opens with **Super+R** or the row in Settings to read
everything alo did on this computer. **The owner approved its redesign on
2026-11-01** — page `554:2` of `nDxyF5Ho9oC4RObjVzwBNJ`: `560:142` at
1440 × 960, `560:316` at 1280 × 800, `560:490` dark, and the Put back
confirmation `560:664`. Every figure below was read off the live file that day
with the plugin API; `crates/alo-shell/src/record_measures.rs` holds them as
logical measures.

## Where it goes

| | figure |
|---|---|
| width | **880**, or the display less 16 each side where that is narrower |
| foot | **108 above the display's foot** — the Dock's 72, its 12 off the edge, and 24 between — in both frames |
| height | **796**, or what the room above that allows with **40** left at the top |
| so | 880 × 796 at (280, 56) on 1440 × 960, and 880 × 652 at (200, 40) on 1280 × 800 — both frames exactly |

**A rule, not a size.** Both frames come out of the one rule and nothing in it
is a figure that holds only at 1440. On a display taller than 960 the window
stays 796 tall; the list scrolls.

## The window

| | figure |
|---|---|
| title strip | the band 44, the strip drawn 32 from 12, top corners **10**, `bg/surface` in a 1 `border/strong` line — the *focused* look of the owner's first answer, because the keyboard is in this window while it is open |
| its icon | 16 at (14, 20) on a `bg/cool` tile at corner 4: the history clock `I560:177;554:26` |
| its title | *What alo did*, SemiBold 13 / 18 at (38, 19) |
| Close | a 44 target ending 4 from the window's edge, its 14 mark at (15, 21) |
| body | from 44, corner **18**, `bg/surface` in 1 `border/strong`, *Elevation/Floating* — 16 down, 40 blur, −4, 14 % |
| heading | *What alo did*, SemiBold 28 / 36 at (40, 32) |
| under it | Regular 14 / 22 `text/secondary`, at most 640 wide, 8 below |
| today's counts | 10 below: *Today* SemiBold 13 / 18 `text/muted`, then pills 28 tall, fully round, `bg/cool`: a 14 icon 10 in, the count SemiBold 11 / 16 30 in, 12 after; 8 between |
| the rule | 20 below the counts, the body's whole width, `border/default` |
| the list | 20 in from the rule, 40 in from the sides, 24 above the footer |
| a day | *Today* SemiBold 13 / 18 `text/primary` 4 in, its date 8 after it in `text/muted`; 10 to its card |
| a card | one per day, 800 wide, corner 12, a 1 `border/default` line; its entries touch, divided by a line |
| between days | 26: the list's 10, the design's 6 spacer, 10 again |
| an entry | 16 above and below, 20 either side, 16 between columns |
| its time | SemiBold 13 / 18 `text/secondary`, a column 44 wide at the least |
| its tile | 32 at corner 8, `bg/cool`, its 16 icon in the middle: a file for a change, an arrow out for what left, a circle struck through for a refusal — and for anything else the record keeps, a question answered here or an update, the window's own clock (a reading: the design draws only the three) |
| its head | 22: the agent's 12 mark at 5 down, 6, its name SemiBold 13 in `accent/default`, 6, the clause SemiBold 14 / 22 `text/primary` |
| its lines | Regular 14 / 22 `text/secondary`, 4 apart |
| its marks | 4 below the lines, 16 tall: a 14 icon, 6, SemiBold 11 / 16 — *that you said yes* in `status/positive`, *Refused* in `status/danger`; 12 between |
| *Already sent* | SemiBold 11 / 16 `text/muted`, at the head row's end |
| footer | 52, a 1 `border/default` line over it, a 14 padlock at 40 and the words SemiBold 11 / 16 `text/muted` at 62, each in the middle of its height |

Dark (`560:523`): surface `#102a43`, cool `#183b56`, lines `#274c68`, the
window's own edge `#738294`, ink `#fffcf8`, secondary `#cbd5e1`, muted
`#a4b4c2`, the agent `#77c8c6`, positive `#7ed3a6`, danger `#ff8d86`.

## Readings taken, for the owner to overrule

- **Every entry is drawn whole or not at all.** The frame shows the last card
  running under the footer; the window has moved by whole entries since
  2026-10-17 and keeps that, so no view hides half of one. The rail that says
  where the view is stays, 4 wide and 14 in from the trailing edge, where the
  frame draws none.
- **The clause is the record's own words**, `alo_recounting::Outcome::said` —
  *the agent did this*, *something left this machine* — where the frame writes
  *renamed a file*, *sent your request to openai*. The window has never worded
  what happened, because a surface able to word it is a machine with two
  accounts of one moment; the frame's head is drawn in the frame's shape with
  the record's words in it. Likewise the frame's line *In the task "Rename
  march-invoice.pdf"* is not drawn: the record does not say which task an
  entry was in.
- **The agent's name is what the record names** — `@files`, `@mail` — where
  the frame draws *alo*. An entry that names no agent, alo OS's own errand, is
  drawn with no name and no mark rather than an invented one.
- **Deep teal is drawn, in exactly one place**: the agent's name. Until this
  redesign the window drew no teal, *it means the agent acting, and this window
  is a person reading*; the agent's name is the agent acting, and a test holds
  every teal pixel to an entry's head that names one.
- **Today's counts are numbers of entries**, not a summary in words: the
  changes the person said yes to, what left this computer, and what was
  refused, today in the person's own day. They narrow nothing — every entry is
  still in the list — and a test checks them against it.
- **The machine's sentence is drawn in the body's type**, Regular 14, where the
  frame sets a file change's line in Medium 12; one style for every line keeps
  a refusal's reason and a change's path equally plain.
- **Minimise and maximise are not drawn**, where the strip in `560:177` has
  them: the record window cannot be minimised or made larger yet, and a control
  that does nothing is worse than none. **Close is drawn and works**, by
  pointer and by Escape. A press anywhere else on the window is the window's
  and reaches nothing behind it.
- **The window does not move.** It is drawn where the rule above puts it, over
  the applications, as it always has been; it is not yet one of the windows a
  person arranges.
- **Put back** (`560:241`, `560:664`) is drawn since the change after this
  one — see below.

## Put back

The owner's answer of 2026-11-01: *a button in the record window, confirmed on
a sheet (`560:664`), doing exactly what `alo-back` does* — with the three
readings the coordinator settled that day, recorded in `docs/features.md`.

| | figure |
|---|---|
| *Put back…* (`560:242`) | 40 tall, `bg/surface` in a 1 `border/default`, corner 12; the 16 icon `560:243` 18 in, its words SemiBold 13 8 after it, 20 to spare; at the head row's end, 20 in from the card's edge |
| the confirmation (`560:763`) | under the change's lines, the content column's width; `bg/canvas` in a 1 `border/default`, corner **10**; 14 above, 16 either side and below, 8 between |
| its question | SemiBold 14 / 22 `text/primary` — *Put {folder} back as it was on {date} at {time}?*, or *these folders* for more than one |
| its lines | Regular 14 / 22 `text/secondary`: one per folder, *What is in {folder} now is kept beside it, as {aside}.*, then *Putting back is written in this list too.* |
| its buttons (`560:767`, `560:769`) | 6 more above; 48 tall, 26 either side of the words, corner 12, 8 apart; *Put back* `text/primary` filled with `bg/surface` words, *Cancel* white in a hairline |
| where the keyboard is | a ring 2 thick, 2 off the button |

**Readings taken, for the owner to overrule:**

- **The question names the moment, not what changed.** The frame's second
  sentence — *invoice-march.pdf becomes march-invoice.pdf again* — needs the
  two sides of the change compared, which nothing on this machine does yet
  (ADR 0045); the confirmation names each folder and where what is in it now
  will be, which is what `alo-back` shows.
- **Every folder the change's turn touched goes back**, worked out from the
  paths its changes were given: a folder it moved something into is itself, a
  file is its folder, one inside another is the outer one.
- **A change that cannot be put back says why in a plain line** under it —
  *older than Put back*, or *no longer on this computer* — and draws no
  button.
- **The keyboard**: Tab and Shift+Tab move between the changes that offer
  *Put back…*, bringing each to the top of the view; Enter or Space presses;
  on the confirmation Tab moves between its two buttons and Escape is
  *Cancel*.
- **While it is done**, the buttons give way to *Putting back…*; when it is
  done the record is read again, so the putting back is its newest line; when
  nothing went back, the confirmation stays and says why.
