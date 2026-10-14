# ADR 0114 — The terminal's engine is pinned, and its face is bundled

**Status:** **proposed, 2026-10-14**, by the Mac lane, on the owner's
instruction to begin building the terminal. **One part needs the owner's word
before any code**: bundling a monospace face, for the reason
`crates/alo-shell/fonts/README.md` records about the last one.

## What is actually missing, which is less than it looks

The first framing was *which terminal emulator do we adopt*. That is the wrong
question and the desktop lane corrected it: **`alo-shell` already draws text**,
through cosmic-text, and already owns a compositor, windows and input.

So the thing missing is not an application. It is **a PTY and a VT state
machine** — the part that turns a byte stream full of escape sequences into a
grid of cells, keeps a scrollback, and tracks what the program did to the
cursor.

A second correction worth recording because the names collide: **in Rust, `vte`
is Alacritty's own VT parser crate, not GNOME's `libvte`.** The first is a
parser and brings nothing with it; the second is a GTK widget and would be an
engine of a different kind entirely. An ADR that named the wrong one would be
read as settling a question it had not understood.

## The decision

**`alacritty_terminal`, as a library, pinned, behind an interface of ours.**

It carries `Term`, `Grid`, `tty` and `event_loop`, and re-exports the `vte`
parser — the state machine, the grid, the scrollback and the pty, none of which
we then write. It is the backend Alacritty itself is built on, used as a library
by other editors, and has no GPU or windowing dependency on that path.

**The alternative, rejected: take `vte` alone and write the rest.** A parser is
only the first half; the grid, the scrollback, the selection model, the reflow
on resize and the alternate screen are the half with the bugs in it. Writing
them would be **writing an engine**, which `CLAUDE.md` forbids — *engines are
configured, never patched* — and the clause exists precisely so a small team
does not end up maintaining a terminal emulator as a side effect of shipping a
terminal.

**Pinned and unpatched.** A source patch to it requires its own ADR, like any
other engine. What is ours is the interface in front of it and everything above:
what is drawn, what is recorded, what is permitted.

## The face, and this is a blocker rather than a detail

**There is no monospace face in this product.** Measured 2026-10-14 with a
control: `crates/alo-shell/fonts/` holds `Manrope.ttf` and `Inter.ttf`, both
proportional; **zero** files anywhere ask for a monospace family; the image
installs no font package at all.

And the tree already records what a missing face does, from a real boot, in
`window_control_label.rs`'s own words:

> cosmic-text found no face, laid out no glyphs, and gave back a box of the
> right size with nothing in it. A booted machine showed the wallpaper, the
> monogram's square and the entry field, and **not one word, including the
> person's own name**.

**So a terminal asking for a monospace family today draws an empty rectangle of
exactly the right size.** Not an error. Correct geometry, no glyphs — and a test
asserting cell dimensions would pass. This is the failure mode that is hardest
to notice and this product has already shipped it once.

**Geist Mono is what the design specifies** — `docs/decisions/0093` records the
brief as Manrope for interface labels with `System/Mono` in Geist Mono — so the
face is chosen and what is owed is bundling it, under its licence, with its
revision recorded beside Manrope's.

**That needs the owner's word.** `fonts/README.md` says the last one plainly:
*the owner authorised bundling it that day.* Bundling a typeface is a licensing
decision as well as a rendering one, and this record does not assume an
instruction to build the terminal covers it.

## One font system, because this fault has already happened once

Measured: **exactly one production site loads faces** —
`window_control_label.rs`, two of them. Every other `load_font_data` in the
crate is inside a test module.

The terminal **uses that one**, and does not create its own. The reason is in
the same comment as the empty-box failure: *the Dock was unaffected because
`dock_raster` loads Manrope into a font system of its own, **which is how one
capability written twice hides a fault in the other***. A terminal with its own
font system would be the third copy and would hide the next one.

## What this decision is not

- **Not a terminal application.** No window of its own, no GPU path, no winit.
  The compositor draws it, as it draws everything else.
- **Not a shell.** The shell is the person's own, started on the pty, with
  their prompt, their history and their aliases untouched.
- **Not AI.** Nothing in this record reaches a model.
  `docs/features.md` is explicit that the terminal stays fully useful with AI
  unavailable, and the first thing built has none in it.

## What is owed before the first line of code

1. **The owner's word on bundling Geist Mono**, with its licence file and the
   upstream revision recorded as Manrope's is.
2. The crate added to the workspace at a pinned version, and a note in
   `docs/quirks.md` if its behaviour and the specification disagree anywhere.

## What is owed after it, and in this order

A crate that owns the pty and the engine and hands the compositor a grid to
draw; the compositor drawing it; sessions; and the window edge. **No AI in any
of it.** `docs/features.md`'s first terminal promise is that the thing works
with the agent switched off, and building it that way is how that promise is
kept rather than asserted.

**Nothing here is built**, and the second measurement in this record exists so
that a lane which skips the face discovers it in review rather than on a booted
machine showing a terminal with no letters in it.
