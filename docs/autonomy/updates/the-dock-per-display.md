# The dock sits per display, and it was one line where it said it would be

**2026-09-27.** *Per display, so the dock can sit along the bottom of the laptop
and down the side of the external screen* is a `[v0.5]` promise in
`docs/features.md`. It is paid. Two screens, one dock, two edges.

The interesting part is not the feature. It is that three files had each written
down, in advance, what would have to change — and all three were right, so the
change was small and nothing had to be redesigned to take it.

## What each file said, and what it cost

`crates/alo-dock/src/dock.rs` opened with **One dock, one edge**, quoted this
promise, and said what the answer would look like: *the v0.5 setting is additive: a
display singled out becomes an exception to the edge, exactly as `alo-appearance`
made a display an exception to a background.*

`crates/alo-dock/src/changes.rs` said why its file had one key: *a file with one
key in it now is a file that gains keys additively later.*

`crates/alo-displays/src/wearing.rs` named the function: *when `alo-dock` gains an
edge per screen, `Wearing::of` is the one function that changes, and every caller
of it keeps working.*

What it cost:

| file | what changed |
|---|---|
| `alo-dock/src/changes.rs` | a `displays: Vec<(DisplayId, Edge)>`, four methods around it, and the key in `Written` |
| `alo-dock/src/dock.rs` | `edge_on`, `set_edge_on`, `put_display_back` |
| `alo-displays/src/wearing.rs` | **one line**: `dock.edge()` became `dock.edge_on(screen.named_for_the_shell())` |
| `alo-shell` | **nothing** |

## The shell needed nothing, which was the surprise

`crates/alo-shell/src/screens_raster.rs` already does this, per screen:

```rust
let mut on_this_screen = dock.clone();
on_this_screen.set_edge(place.edge());
```

It draws each screen's dock on the edge that screen's `Wearing` gave it. So the
drawing has been per display all along and the promise was never blocked on it —
only on `Wearing::of` having one edge to give. A reading of the gate that said *a
dock on each display is drawn and they all take the same edge* was exactly right
about the symptom and would have sent somebody into `alo-shell` to fix it, which
was the wrong crate.

## The shape, and why it is `alo-appearance`'s

`Dock::edge_on` answers **the exception the person made for this display, or the
edge they chose for everywhere, or the edge the release ships** — in that order,
which is `Appearance::background_on`'s order. Two things a screen wears, decided
the same way, because two answers in this repository about what *per display* means
would be one too many.

Three consequences worth stating:

- **Two identical screens are two names.** `Reported::named_for_the_shell` is the
  socket and the screen's own description together, so an edge chosen for one of a
  matched pair does not appear on the other. That is not new — it is why the
  background already worked — and `Wearing::of` asks for the edge with the same
  name it asks for the background, two lines apart.
- **`dock.toml` gains the key additively.** The list is absent rather than empty
  when nobody singled a screen out, so a machine that has not used this writes
  exactly the file it wrote before the key existed, byte for byte.
- **`is_untouched` counts a screen singled out.** It did not, and would have been
  wrong the moment this landed: a machine whose only change is one screen's edge
  has been changed, and a reader that said otherwise would write no file and lose
  the change at the next sign-in. It is no longer `const`, which is the whole cost.

## Putting things back is two calls, on purpose

`put_back(Setting::Edge)` puts the edge for everywhere back and **leaves** a screen
singled out; `put_display_back(&display)` stops singling that screen out and leaves
the edge for everywhere alone. They are two things a person means, and
`Setting::Edge` says so in its own words now, as `alo-appearance`'s does. A test
holds both, because a panel offering *put it back* has to offer the right one.

## What is owed

**A cable.** Nothing in this repository has ever had a second display attached, so
*the dock along the bottom of the laptop while it runs down the side of the external
screen* is arithmetic and two tests rather than something anybody has looked at. The
machine half of the box says so and is not ticked.

And the other half of the same line in `docs/features.md` — *the dock's size, and
whether it hides when a window needs the room* — is still half owed. The size is
there; the hiding has no code and `layout.rs` says so at the place that work would
go. That is why the gate's single box for the two promises had to become **two
boxes** before either could be true, and why the gate now reads 27 of 96 rather
than 26 of 93. A box grouping two promises cannot say that one of them is paid,
which is the argument
`docs/autonomy/updates/the-gate-is-a-check-now-and-what-it-cannot-check-yet.md` makes for each
box naming the promises it answers — made here by a box rather than in prose.

## The gate

`alo-dock` and `alo-displays` each `cargo fmt --check`, `clippy -D warnings` and
their tests — 64 and 124 unit tests, `GATES=0` on both — and then all seven on the
workspace.

`crates/alo-dock` belongs to
`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, whose seven tasks
are done, and `crates/alo-displays` to
`docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, 14 of 14. Neither had a
task for this and no lane was working in either crate; the last change to touch
them was #200 on the palette. Both plans' own words said what to do, which is the
best case for a promise with no task: the crates had already had the argument.
