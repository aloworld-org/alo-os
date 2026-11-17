# The file chooser

**Decided by the owner on 2026-11-17**, quoted as chosen: *"Yes, build it"* —
alo OS has its own file chooser, the same for every application. The person
uses it; alo may answer it only inside the folders the person granted (one card
at *ask each time*, none at full access). This note is the sheet the person
sees. alo's half is a later change and is not drawn here.

## The design is silent, and what was followed instead

**No frame in the design snapshot draws a file chooser, an *Open* or a *Save*
sheet.** Checked on 2026-11-17 by searching every layer name in
`docs/design/figma-snapshot/*.xml` for *open*, *save*, *file*, *chooser*,
*browse* and *picker*. The only hits are other things: *Model picker*, *Country
picker*, *Action / Choose files* (a Dock action), and a caption *System folder
picker · explicit selection.* in the terminal settings frames (`500:29477`),
which names a picker and draws none. **The live Figma file was not checked**,
so a frame there would not be known to this note.

So the sheet follows **the nearest drawn sheet**, *Open with* (frame `25:52`,
[`default-apps-and-open-with.md`](default-apps-and-open-with.md)), part for
part, and draws its rows with Settings' own parts, as that sheet does. Each
reading below is one somebody may disagree with. Each is a reading and none is
a measurement.

| | Reading | From |
|---|---|---|
| The sheet | 480 wide, 25 inside, the window's corner and a floating shadow, centred **in the room above the alo Bar**, never wider than that room less 16 each side | *Open with*, `25:57`; the room is the owner's word of 2026-11-17 on seeing the first pictures: *"Yes, fix the overlap"*. The room is the display less the Bar's band as drawn, the rule Settings fits by (`crate::where_a_sheet_sits`) |
| Title | *Open a file*, or *Save a file* | *Open with*'s title, `25:58` |
| The line under it | *&lt;App&gt; asked to open a file*, or *to save a file* | *Open with*'s *&lt;App&gt; asked to open &lt;address&gt;*, `25:59` |
| Where the person is | a line *In &lt;the whole path of the folder&gt;*, muted, under that | none drawn. The whole path, because a folder's own name alone (*Documents*) does not say whose or where |
| The rows | one card of Settings rows: the folder above first (*The folder above*), then folders (*Folder*, with a chevron, because a folder opens something), then files (*File*). Each has the Settings *Folder* or *File* icon in its well | the *Open with* card, `25:60`; Settings' rows with a lead icon |
| How many rows | as many as the display holds, from 3 to 8, a page at a time, so the row the keyboard is on is always shown | none drawn. **Derived from the display**, never a fixed count |
| Where the keyboard is | ringed in `text/primary`, as a Settings row is | *Open with* |
| A sentence about what just happened | under the card, muted: a folder that would not open, a name that is not one, a name a folder has, a name that is there already | none drawn |
| The name, when saving | *Name*, then a field holding the name, outlined in `text/primary` while the keyboard is in it | none drawn. The field is the card's outline around one line of words |
| Buttons | *Cancel*, quiet, then *Open* or *Save*, navy, at the right, 48 tall | *Open with*, `25:74`–`25:77` |

**Hidden names are not shown.** A name beginning with a dot is hidden, as every
Linux desktop hides it. A person can still save under one by typing it. A
switch to show them is not offered, and nothing was withdrawn: no choice about
hidden files has been promised anywhere in this repository.

## What each key does

The behaviour is `alo_file_chooser::chooser`'s, and the module's header is the
full list. In short:

- **Opening:** Up and Down move, Home and End go to the ends, Enter on a folder
  goes in and on a file chooses it, Backspace goes up, a letter jumps to the
  next name that begins with it, Escape cancels.
- **Saving:** the keyboard starts in the name, holding the name the application
  suggested. Tab moves between the name and the rows. Enter in the name saves.
  Enter on a file takes its name. **A name that is there already is not
  replaced on the first Enter**; the sheet says so and a second Enter replaces
  it.
- **A click** on a row moves the keyboard there, and a click on the row it is
  already on uses it, as Enter does. **A screen reader's press** on a row uses
  it at once.

## What a screen reader hears

A dialogue named by the title; the line; *In &lt;folder&gt;*; a list whose items
are named by their names, with *The folder above*, *Folder* or *File* as their
description, the one the keyboard is on selected. Then, when saving, a text
field named *Name* holding the name. Then any sentence about what just
happened, *Cancel*, and *Open* or *Save*. Every word heard is a word drawn.

## What the person is never stopped from

The sheet starts where the application asked, or in the person's home. **The
person may go anywhere their own account can read, up to the root**, and save in
any folder they reach. alo's grants never narrow this. They decide only what alo
may answer for the person.
