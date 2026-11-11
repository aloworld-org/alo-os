# The alo UI kit: what alo's own applications are drawn with

**Status: decided by the owner on 2026-11-11; the plan itself is new and its
tasks are not started.** Asked which road alo's own applications take for their
interface, the owner chose *"Build a shared alo UI kit (Recommended)"*. The
decision is recorded in
[ADR 0002](../decisions/0002-the-shell-is-native.md)'s section of that date,
and the promise in `docs/features.md`, *The shell*. It is owned by the
coordinator lane on this PC (`alo-os-shell` checkout), as the charter's row
says.

**What it is for.** alo Mails (`alo-workplace`, the Flatpak
`com.alomails.Mails`, AGPL-3.0) is the first application alo makes for people
to use every day, and it waits on this kit. The kit is the shell's own drawing,
text, controls, keyboard road and accessibility, moved into crates an
application builds on, so alo Mails looks and behaves exactly like the desktop
around it. **The first usable version comes first**: everything here is ordered
so that alo Mails can open a window, list mail, read a message and fill in an
address and a subject as early as possible, and everything that is not needed
for that waits. Writing a body over many lines is task 14, straight after.

**What the person gets from the whole plan:** alo's own applications that use
the same typeface, colours, pointer, keyboard keys and screen-reader names as
the desktop, on a machine with no graphics card, at any display scale. **What
the person gets until alo Mails is built on it, after task 13: nothing they
can see.** The desktop draws
exactly what it drew before (law 3), and that is a test, not a hope.

## What the shell already has, surveyed 2026-11-11

Read off `main` at `ccb5c81a`. *Reusable* means it does not need a compositor
to work; *bound* means it does. This was read from the code once, by hand, and
nothing re-reads it; task 2's dependency test is what holds the answer from
then on.

**Reusable now, or after one type is swapped.** Almost all of alo's drawing
produces plain pixel lists and only borrows Smithay's rectangle type to say
where they go. Swapping `smithay::utils::Rectangle` for a rectangle of the
kit's own is the whole of the change for:

- `crates/alo-shell/src/painted.rs`'s `Solid`, `Inked` and `Washed` (flat
  shapes, inked text and premultiplied RGBA rasters) and
  `crates/alo-shell/src/painted_flat.rs` (laying one raster over another in
  memory). `painted.rs`'s `paint` writes into a Smithay `Frame` and is bound;
  it stays in the shell.
- `crates/alo-shell/src/settings_canvas.rs`: a CPU canvas with rounded shapes,
  circles, hairlines, artwork, words and a clip for a scrolled page, all
  anti-aliased by one rule. This is the rasteriser an application needs, and
  it is already software-only.
- `crates/alo-shell/src/painted_text.rs` and
  `crates/alo-shell/src/painted_text_wrapping.rs`: cosmic-text shaping and
  inking, Manrope at the design's weights, Geist Mono for monospace.
  `crates/alo-shell/src/window_control_label.rs` builds the font database from
  the bundled files in `crates/alo-shell/fonts/` and never reads the host's
  fonts, which is right for a Flatpak too. Its locale is fixed at `en-US`,
  which matters for which fallback face a script gets; the kit should take the
  person's language instead.
- `crates/alo-shell/src/display_scale.rs`: one display's whole-per-cent scale
  and the one road a logical measure takes to pixels, exact at 100.
- `crates/alo-shell/src/settings_type.rs` (the design's type styles, scaled
  with the person's text size), `crates/alo-shell/src/settings_palette.rs` and
  `crates/alo-shell/src/settings_measures.rs` (the Settings window's colours
  and figures). The type styles are general; the measures are Settings' own.

**Already outside the shell, and reusable as they are:** `alo-appearance`
(the palette as twelve roles, accent, scheme, text size, contrast
arithmetic), `alo-vector-shapes` (SVG paths to coverage, on the rasteriser
cosmic-text already uses), `alo-cursors`, `alo-strings` (every sentence, in the
person's language), and `alo-access`'s description of a control — its role,
name and state — in `crates/alo-access/src/tree.rs`.

**Built in the shell, and only part of what an application needs:**

- **Controls.** Settings' rows, switch, choice ring, slider, swatches, tags and
  segmented choice (`crates/alo-shell/src/settings_drawn_controls.rs`,
  `crates/alo-shell/src/settings_drawn_row.rs`) are drawn from the design's
  nodes, but each is laid out by Settings' own page types, so they move as
  drawing and are re-laid as controls. The Bar's field
  (`crates/alo-shell/src/the_composer_a_person_types_in.rs`) takes letters,
  erases the last one and adds a line; it has **no caret movement, no
  selection, no copy and paste and no undo**, which a mail field needs.
- **Scrolling** is per surface (`crates/alo-shell/src/a_task_window_scrolls.rs`
  reads the compositor's wheel events). An application gets its wheel from
  `wl_pointer` and needs its own.
- **Keyboard focus** between areas is the shell's
  (`crates/alo-shell/src/which_area_has_the_keyboard.rs`); the order a reader
  reads controls in is `alo-access`'s and is reusable.

**Bound to the compositor, and staying there:**

- `crates/alo-shell/src/scene_drawing.rs` and
  `crates/alo-shell/src/software_scanout.rs`: putting a frame on a screen.
- **The window band** (`window_edge*`): alo draws every window's band itself,
  and `crates/alo-shell/src/window_edge_who_draws.rs` records that the
  compositor answers `ServerSide` to every client that asks. So an application
  asks for server-side decoration and **draws no title bar of its own**; that
  is how its band is the desktop's by construction.
- **The pointer**: an application names a shape through `wp_cursor_shape_v1`,
  which the compositor offers (`delegate_cursor_shape`), and alo draws it.
- **The accessibility serving**: `crates/alo-shell/src/access_bus.rs`,
  `crates/alo-shell/src/access_nodes.rs` and
  `crates/alo-shell/src/access_serving.rs` answer the shell's own surfaces on
  the bus. The bus code is zbus and needs no compositor, but by its own words
  it answers `Accessible` and `Action` only — **no `Component`, no `Text`, and
  nothing emitted**. A screen reader follows focus by events and reads a field
  through `Text`, so a mail application needs all three. That is the measured
  gap task 7 decides how to close.

**What the compositor does not offer yet.** Its globals are compositor,
xdg-shell, xdg-decoration, shm, seat, output, data device, pointer gestures,
viewporter, text input, dmabuf and cursor shape. **`wp_fractional_scale_v1`
is not announced**, so an application at 150 % cannot be told to draw at
150 % (task 4).

**What the portals answer today** (`docs/contracts/portals.md`): Secret,
Settings (colour scheme and accent), NetworkMonitor and `OpenURI.OpenFile`.
**A web link is answered `2`** and **the file chooser and notifications are not
answered at all**, so a link in a message and an attachment cannot work in any
application yet. Those are alo OS's to build, not the kit's; task 11 lists
which lane holds each.

**AccessKit's Unix adapter, read off crates.io on 2026-11-11, once:**
`accesskit_unix` 0.24.1 (released 2026-10-25), MIT OR Apache-2.0, needs Rust
1.87 (this workspace is on 1.97), and depends on `zbus ^5.19` with `async-io`
— the version and feature `alo-shell` already locks — plus `atspi` 0.29,
`accesskit_atspi_common` 0.21 and `async-executor`. That is what it costs to
take. **What it answers is not measured**, and task 7 measures it.

## The crates, and the licence

Proposed, and put to the owner (see *Questions* below):

- **`alo-drawing`**: pixels and text. Shapes, rasters, the text stack and its
  bundled fonts, the display scale. **No Wayland, no Smithay, no D-Bus**, so
  the shell and every application can depend on it and a test can draw with
  it on any machine.
- **`alo-ui-kit`**: what an application is built from. The window, the
  controls, focus and the keyboard, the accessibility tree, the portal
  helpers. It depends on `alo-drawing`; the shell does not depend on it.

Both are **GPL-3.0-or-later**, the workspace's licence. alo Mails is
AGPL-3.0, and section 13 of each licence lets the two be combined. They live
in this repository, beside the shell they come from, and `alo-workplace`
depends on them by a pinned revision until the owner decides otherwise.

## How the desktop is kept the same

`crates/alo-shell/src/a_picture_hashed.rs` already hashes everything a
surface paints, and five surfaces are held to hashes taken before a change.
Task 1 extends that to **every native surface the extraction touches**, at
100, 150 and 200 % and two text sizes, with hashes taken on `main` before
anything moves. **Every later task that moves code out of `alo-shell` must
leave those hashes unchanged**, and runs the shell's whole suite, not the
moved files' tests. Those hashes run in CI on every change from then on. A
picture that changes on purpose is a design change and is not made in this
plan.

## Sizes

**S** is about a day, **M** two to four days, **L** one to two weeks, each for
one lane with its gate. Tasks 1 to 13 are the first usable version; 14 onwards
come after it.

## Tasks

### 1. The desktop's pictures are pinned before anything moves

**Status:** not started. **Size:** S. **Waits on:** nothing.

**For the person:** nothing visible. It makes every later task safe: if moving
code changes a pixel on the desktop, a test says so. **Reuses:**
`crates/alo-shell/src/a_picture_hashed.rs`. **Test:** the hashes of the
Settings window, the task window, the record, the sign-in and lock screens,
the egress status, the Bar and the window band, at three scales and two text
sizes, taken from `main` and committed.

### 2. `alo-drawing`: alo's pixels and text, out of the shell

**Status:** not started. **Size:** M. **Waits on:** task 1.

**For the person:** nothing visible yet; it is what makes an alo application
draw exactly as the desktop does. **Moves:** `painted.rs`'s three picture
types, `painted_flat.rs`, `settings_canvas.rs`'s canvas, `painted_text.rs`,
`painted_text_wrapping.rs`, the font database from
`window_control_label.rs` with `crates/alo-shell/fonts/`, `display_scale.rs`
and `settings_type.rs`, with a rectangle of the crate's own. The shell keeps
`paint` and converts at that one seam. **Test:** task 1's hashes unchanged;
the moved tests move with their files; and a test that reads
`alo-drawing`'s manifest and refuses any Wayland, Smithay or D-Bus
dependency. *This makes an existing promise true — ADR 0002's "the UI runtime
built for the shell" — and adds none.*

### 3. An alo application opens a window

**Status:** not started. **Size:** M. **Waits on:** task 2.

**For the person:** the first alo application window on the desktop, with
alo's band on it, alo's pointer over it, drawn in alo's colours with no
graphics card. **Builds:** in `alo-ui-kit`, an xdg-shell toplevel over
`smithay-client-toolkit` 0.19, already in `Cargo.lock`; asks for server-side
decoration; draws into shared-memory buffers with `alo-drawing`; draws at the
output's scale; names its pointer through `wp_cursor_shape_v1`; draws only
when the compositor's frame callback asks. No X11 code is linked. **Test:**
against alo's own compositor in the shell's test fixtures: the window maps,
its decoration mode is server-side, a shape it names is the one the
compositor records, and at scale 200 its buffer is twice its logical size.

### 4. The compositor tells an application its fractional scale

**Status:** not started. **Size:** S. **Waits on:** nothing; it is
`alo-shell`'s, and its owners are told first.

**For the person:** an alo application on a 150 % display is drawn at 150 %,
sharp, rather than drawn at 200 % and shrunk. **Builds:**
`wp_fractional_scale_v1` in the compositor, announcing each display's whole
per cent. *This makes an existing promise true — "Multi-monitor, display
scaling" — because a display at 150 % is a scale the shell already offers.*
**Test:** a client on a display at 150 % is told 150 and its viewport is the
logical size.

### 5. Typing reaches an alo application in the person's language

**Status:** not started. **Size:** M. **Waits on:** task 3.

**For the person:** keys typed into alo Mails are the letters of the keyboard
the person chose, dead keys and compose included, held keys repeat at the
person's rate, and an input method works where it is set up. **Builds:** the
keymap the compositor sends, read with `xkbcommon` (in `Cargo.lock`), repeat
from `wl_keyboard`, and `text-input-v3`, which the compositor offers.
**Test:** against the compositor with a German and a Latvian layout, a dead
key and a composed letter arrive as the right character.

### 6. Focus and the keyboard road

**Status:** not started. **Size:** S. **Waits on:** task 3.

**For the person:** every control can be reached with Tab and Shift+Tab, in
reading order, and the one with the keyboard is visibly marked; Escape leaves
what it can leave. **Reuses:** `alo-access`'s reading order. **Test:** a
window of every control in tasks 9 and 10 visited by Tab in reading order and
back by Shift+Tab, and **a control no key reaches fails the test**.

### 7. Measure the accessibility road, then choose

**Status:** not started. **Size:** M. **Waits on:** task 3.

**For the person:** nothing visible; it decides how a blind person and alo's
own reader will find everything in alo Mails. **Measures, on the booted qemu
machine with Orca running, inside a Flatpak:** a test window built once with
`accesskit_unix`, and once with alo's own serving extended. For each: does
Orca announce a focus change; read a field's text and follow its caret; read a
list row's position; find it through Flatpak's accessibility proxy; does
`alo-adapters`' reader walk it and press a button; and how much code each
costs, counted. **Test:** none for the measurement — **done once, by hand**,
and the report in `docs/autonomy/updates/` says so and names task 8 as where
each answer becomes a check that runs again. **Delivers:** the choice, with its
reasons, in that report.

### 8. An alo application's own accessibility tree

**Status:** not started. **Size:** M. **Waits on:** task 7.

**For the person:** Orca reads every control in an alo application by its
name in the person's language, and alo can read and press them through the
same tree. **Builds:** the road task 7 chose, with each control's role, name
and state from `alo-access`'s vocabulary. **Test:** on a real at-spi bus, as
`crates/alo-shell/tests/the_tree_a_reader_finds.rs` does for the shell: the
tree read back by `alo-adapters`' reader; focus moving emits the event; a
field's text and caret are readable; **a password field's text never is**.

### 9. Words, buttons and icons

**Status:** not started. **Size:** S. **Waits on:** tasks 2 and 6.

**For the person:** labels and buttons in alo Mails are the desktop's: the
Quiet button of `docs/design/the-settings-window.md` (48 tall, radius 12),
Manrope at the design's styles, icons from the design file's own SVG.
**Reuses:** `settings_drawn_controls.rs`'s drawing, `alo-vector-shapes`.
**Test:** a button drawn at 100 and 200 % is held to the design's figures;
pressing it by pointer, Enter and Space each does one thing once.

### 10. A text field

**Status:** not started. **Size:** M. **Waits on:** tasks 5 and 9.

**For the person:** a field for an address or a subject that behaves like a
field everywhere else: a caret that moves by letter and by word, selection by
Shift and by dragging, copy, cut and paste with other applications, undo,
never splitting a letter from its accent. Drawn as the Bar's field is in
`docs/design/the-alo-bar-when-it-is-open.md`. **Reuses:** the composer's
grapheme rules; the clipboard through `wl_data_device`. **Test:** editing in
Latvian, Greek and emoji keeps whole graphemes; text copied from the
terminal pastes; a password field is never copied out.

### 11. A list of rows, scrolling, a split and a toolbar

**Status:** not started. **Size:** M. **Waits on:** task 9, and the owner's
answer on where Mail's own design is (see *Questions*).

**For the person:** a folder list, a list of thousands of messages and the
message itself, wrapped and selectable to copy, that scroll smoothly by wheel, touchpad and keyboard, with the selected row
marked; a divider between the lists and the message that can be moved by
pointer and by keyboard; a row of tools across the top. **Reuses:** the Bar's
rows (56 tall, radius 12) and the Settings page's scroll under a clip.
**Test:** only the visible rows are drawn, measured with ten thousand; Page
Up, Page Down, Home and End; the divider moves by keyboard; the selected row
is the one a reader is told is selected.

### 12. Portal helpers, and what alo OS still owes

**Status:** not started. **Size:** M. **Waits on:** task 3.

**For the person:** alo Mails follows light and dark and the person's accent
the moment they change, and keeps an account's password in the person's
keyring rather than in a file. **Builds:** helpers for Settings (with
`SettingChanged`) and Secret, over zbus. **Owed by alo OS, and named here so
nobody waits silently:** a web link (`OpenURI` answers `2`) and the
file chooser are not answered on this machine, so links and attachments wait
on them; the file chooser is the desktop lane's (`docs/contracts/portals.md`)
and the web link is owed by ADR 0002's browser section. **Test:** against
`alo-portald` on a real bus: a colour change reaches the window; a refused
Secret request is said to the person in plain words, never a silent empty
password.

### 13. The first version, in alo Mails' hands

**Status:** not started. **Size:** S. **Waits on:** tasks 1 to 12.

**For the person:** alo Mails can be built on the kit. **Delivers:** the
kit's surface written as a contract in `docs/contracts/` (it is used from
another repository, so it changes additively), its words in all 24 EU
languages through `alo-strings`, and an example window in the crate with a
folder list, a message list, a field and a button. **Test:** that example,
built as a Flatpak, run on the qemu image with no graphics card: it opens,
takes the keyboard, scales, and Orca reads it — **a walk done by hand**, and
the report says it will not run again by itself.

### 14. A message is written in more than one line

**Status:** not started. **Size:** L. **Waits on:** task 13.

**For the person:** the body of a mail: wrapped plain text over many lines and
pages, scrolled with the caret, with everything task 10 does.
**Test:** a ten-thousand-line body stays responsive; caret by line, page and
paragraph; the reader follows the caret.

### 15. Menus and pop-ups

**Status:** not started. **Size:** M. **Waits on:** task 13.

**For the person:** right-click menus and pop-ups that are alo's, placed by
the compositor so they never fall off a small screen. **Reuses:** `alo-menus`;
`xdg_popup`. **Test:** a menu at a screen's corner is placed inside it; every
entry is reachable by keyboard and named to a reader.

### 16. The desktop draws its own controls with the kit

**Status:** not started. **Size:** L. **Waits on:** task 13.

**For the person:** nothing visible, which is the point: Settings' rows and
the Bar's field become the kit's controls, so the desktop and alo's
applications cannot drift apart. **Test:** task 1's hashes unchanged.

## Roughly how long

**The first usable version is tasks 1 to 13: five S and eight M — about four
and a half to seven and a half weeks for one lane**, less with two once task 3 has landed, because
tasks 4 to 12 mostly do not wait on each other. Tasks 14 to 16, one M and two L,
are two and a half to five weeks more and off that path. It is not days. Task 7's measurement can
move this either way, which is why it is early.

## Questions for the owner

1. **The crates' names**: `alo-drawing` and `alo-ui-kit`, or another pair?
2. **Where the kit lives**: in this repository, as proposed, or a repository of
   its own that both this one and `alo-workplace` depend on?
3. **The licence for others**: GPL-3.0-or-later means an application somebody
   else builds on the kit must be GPL-compatible. Is the kit for alo's own
   applications only, or should other people be able to build closed
   applications on it, which would need a different licence?
4. **Whose design governs alo Mails' lists, split and toolbar**: the alo OS
   design file has a button, a field and rows, measured in `docs/design/`, and
   no split pane or toolbar recorded there. Is there an alo Mails design, and
   where?
5. **The first version's scope**: is a short plain-text reply enough for the
   first usable version, as this plan assumes, with multi-line writing
   (task 14) straight after?
