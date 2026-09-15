# v0.5 — hands on the desktop: dividing the screen, and everything a hand does

**Workstream:** two `ROADMAP.md` v0.5 lines that are one subject — ★ *Divide the
screen: halves and quarters by drag or keyboard, splits that hold while you work
and are remembered, per display*; and *Input: drag and drop, context menus,
gestures, virtual desktops, keyboard layouts with dead keys and a compose key,
input methods*. They belong together because they are **what a person's hands do
to the desktop**, and every one of them is a decision about intent before it is a
pixel.
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**Crates this plan owns, all new:** `crates/alo-dividing` (splits: which window
has which share of which display, how a split holds under resizing, and how it is
remembered), `crates/alo-desktops` (virtual desktops and what moves between
them), and `crates/alo-keyboards` (layouts, dead keys, the compose key, input
methods, and the layout offered with a language). **It reads and never edits**
`alo-shortcuts` (a keyboard split is a shortcut; the binding is that crate's
value), `alo-displays` (the session plan's — a split is per display), `alo-dock`,
`alo-clipboard` (drag and drop carries the same payloads copy and paste does),
`alo-strings` (the language a person chose), `alo-capability` and `alo-context`
(a dropped file is not a grant, and a context menu is not context an agent is
offered), and `alo-saying`. **Nothing in `crates/alo-shell`**: the shell plan's
later tasks draw and route what this plan decides. `window_tiling` in `alo-shell`
is v0.01's tile; this plan decides the v0.5 split the shell will then honour.

**What this plan may not do:** tick anything *on the machine*; write a keyboard
driver, an input-method engine or a layout database of our own — `xkeyboard-config`
and `libxkbcommon`'s compose tables, and a rented input-method framework, are
configured and never patched (ADR 0011); or make a drop, a menu or a gesture a road
by which an agent is granted anything. Before writing the next task, `git pull`
and read the plan as published.

## Tasks

### 1. A split: halves and quarters that hold

**Status:** ready. **Depends on:** nothing.

★ The star is on *hold*. Every system can snap a window to half the screen. Almost
none keeps the two halves a pair: resize one and the other stays where it was,
overlapping or leaving a gap.

- **Acceptance:** `alo-dividing` holds a division of one display as a tree of
  shares — halves, quarters, and a split of an existing half — each share naming
  the window in it; **resizing the boundary between two shares resizes both**, and
  a test holds that no share ever overlaps another or leaves a gap on the display;
  a window dragged to an edge proposes a half and to a corner a quarter, and the
  proposal is shown before it is committed so a drop can be abandoned; a keyboard
  split of what is already open divides the focused share with the next window,
  bound through `alo-shortcuts`; a window closed inside a division gives its share
  to its neighbour rather than leaving a hole; and a window with a minimum size
  larger than its share is **not** squeezed below it — the division refuses and
  says why.
- **Constraint:** nothing draws. Geometry is in logical units the display's scale
  turns into pixels, so a division means the same thing on a scaled screen.

### 2. A split remembered, per display

**Status:** blocked — on `v0-5-the-session-and-the-displays-plan.md` task 3, whose
`alo-displays` gives a display the stable identity a remembered split is kept under.
**Depends on:** 1.

- **Acceptance:** returning to a pair of windows restores the division they were in
  rather than each window's last position, held by a test that closes and reopens
  both; a division is per display, so the laptop's own screen and an external one
  divide independently (`docs/features.md` v0.5), and unplugging a display keeps its
  divisions to restore when it returns; what is remembered is **applications and
  shares, never a window's title or a document's name**, kept by this crate in the
  person's folder at a path it is handed (ADR 0038).
- **Constraint:** the agent's *arrange* verb (v0.01) proposes a division through
  the same type and is approved like any change; it does not get a second road to
  place windows.

### 3. Virtual desktops

**Status:** ready. **Depends on:** nothing.

- **Acceptance:** `alo-desktops` holds a person's desktops per display — add,
  remove, name, reorder — and which windows are on each; **removing a desktop moves
  its windows to a neighbour and never closes one**; a window can be on every
  desktop; switching is a gesture (task 5) or a shortcut; each desktop keeps its own
  divisions (task 2); and the egress indicator, the approval surface and the agent
  overlay are **on every desktop at once**, held by a test, because a promise that
  lived on desktop 1 would be a promise nobody on desktop 3 could see.
- **Constraint:** nothing draws.

### 4. Drag and drop, and context menus

**Status:** ready. **Depends on:** nothing.

- **Acceptance:** a drop carries what copy and paste carries — text, images, files
  — through `alo-clipboard`'s payload types rather than a second set, and the
  target application receives it through the portal a sandboxed application expects;
  **dropping a file onto an agent's surface offers it as context for that turn only
  and is not a grant** — a test holds that no grant exists afterwards, because a
  grant is made by `alo-picking` and nothing else (ADR 0001 §3); a context menu is a
  closed list of actions the thing under the pointer offers, each an action a person
  could reach another way (ADR 0009), and **no menu entry sends anything to the
  agent without the person choosing the entry that says so**.
- **Constraint:** nothing draws. No *drop to grant*, however convenient it looks.

### 5. Gestures

**Status:** ready. **Depends on:** 3.

- **Acceptance:** touchpad scroll, pinch to zoom and three- or four-finger swipes
  between desktops are decided from the input library's gesture events into a
  closed set of intents with a test per intent; a person can turn each off; natural
  and traditional scrolling is a setting kept by this plan's crate; and a gesture
  never triggers the agent — the overlay has one key (v0.01) and no gesture.
- **Constraint:** `libinput` is rented and unmodified.

### 6. Keyboards: layouts, dead keys, compose, input methods

**Status:** ready. **Depends on:** nothing.

*"Müller" and "Liège" are test cases in a European product, not edge cases.*

- **Acceptance:** `alo-keyboards` offers, for each of the 24 languages
  `alo-strings` carries, **the layout people of that language actually type on** —
  a test per language names the layout and fails if a language has none — so
  choosing Greek never means hunting for a Greek keyboard; switching layouts is one
  shortcut and is shown in the status area; dead keys and the compose key produce
  `ü`, `è`, `ß`, `ł`, `ő`, `č`, `ġ`, `ħ` and the Irish and Maltese letters, **each
  typed in a test through the rented compose tables**, not through a table of our
  own; input methods for non-Latin scripts are a rented framework configured to
  start with the session, and a person adds one without knowing its name; and
  layouts are kept by this crate in the person's folder (ADR 0038).
- **Constraint:** no layout or compose data is written here. What is ours is which
  layout is offered with which language, and that the offer is complete.

### 7. Every sentence, and the walk through a working morning

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — split two windows, resize the boundary, move one to
  a second desktop, drop a file onto an application, switch layout and type
  *Müller* — produces the exact sequence a person meets, recorded as a table and
  held by one test; no sentence names `libinput`, XKB, a keysym or an input-method
  framework.
- **Constraint:** nothing here re-decides what the sentences describe.
