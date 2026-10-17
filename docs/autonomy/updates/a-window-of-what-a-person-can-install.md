# A window of what a person can install

**Date:** 2026-10-17
**Workstream:** third-PC lane — software a person works in
**Contributor:** Claude Code worker in `C:\dev\alo-os-3`
**Status:** ready for integration. The window's model, its keys, its words and
the whole install conversation are complete and tested. **Nothing draws it
yet** — the raster is the next change, and *What was not built* says so rather
than leaving it implied.

## What changed, for a person

Still nothing a person can see. What changed is that the list added in #748 now
has something that knows how to be read: a window that shows every application
a person can install, narrows to one kind of work at a time, and closes.

## Where this sits

#748 added `catalogue.toml` and the reader that refuses it when it is wrong.
This adds the window's **model**: what it shows, what a key press does, and the
sentences it is drawn through. The pixels come next.

The three files follow `running_*` exactly, which is this crate's shape for a
window a person reads: `market_keys.rs` is what a key press means,
`market_window.rs` is the model, and the words live in the crate that owns the
subject rather than in the shell.

## Three things the architecture decided, not me

**The shell may not word a sentence.** `tests/desktop_source.rs` holds every
`desktop_*`, `dock_*`, `running_*` and `filling_*` file to naming no `Word::`
and no `.say(`, because a raster that looked a sentence up would be a second
place deciding what a person reads. So the eleven new strings are in
`alo-software`'s vocabulary — the window's name, the promise under the list,
and one for each of the eight kinds of work — and `Category::word()` is how the
enum reaches them.

That closes an obligation `category.rs` wrote for itself in #748: *a market
screen may not draw `Category::named`*, which is the spelling the data file
uses and which nothing translates. It now cannot, because there is a word to
draw instead.

**No key here installs anything.** `MarketKey` has Up, Down, Home, End, Left,
Right, Enter and Escape, and Enter only records that the person asked. A window
cannot reach this machine, and *Asking to install* below says what carries the
ask out. `no_key_changes_what_is_on_the_machine` is the test.

**There is no *nothing here* state**, and that is not an omission.
`Catalogue::read` refuses a list that leaves a kind of work empty, so narrowing
to one always leaves at least one application. The impossible state is made
impossible upstream instead of handled downstream, and
`every_kind_of_work_has_something_under_it` holds the two files to each other.

## Asking to install, without being able to install

The window can now be asked for an application, and it still cannot install
one. `MarketKey::Install` (Enter) produces `MarketPressed::AskedToInstall`;
the host reads `what_they_asked_for`, does the installing, and answers with
`it_arrived` or `it_was_refused`. `HowItIsGoing` is the four states a row can
be in — not asked, asked, here, refused — and a refusal is carried as
`alo_software::Stopped` rather than a sentence, because this crate may not word
one.

**The rule that decided this shape.** ADR 0001 §5 is about an agent's turn: *a
read executes inside the turn… a change comes back as a proposal.* A person
pressing a key in their own window is not an agent proposing a change, and
CLAUDE.md's second law says the same — it binds the agent, never the person. So
there is no second approval between a person and the thing they just chose, and
the approval surface stays what it is for.

**What is not negotiable is the indicator.** `alo_software::install` takes an
`Underway` and refuses to run unless the errand is already on it, so law 1 is
not a convention the host is trusted to follow here — it is a precondition of
the function. The window could not install silently even if it tried to.

Four decisions, each a test:

- **one ask at a time**, because two rows doing something at once make the
  first answer ambiguous;
- **what is already here cannot be asked for**, because the row would wait for
  an answer that never comes;
- **an answer about another application is ignored**, because it answers a
  question this window did not put;
- **a refusal does not lock the row** — asking again clears the old reason
  rather than showing it beside a new attempt.

## Two decisions worth recording

**Moving stops at the ends; narrowing wraps.** A list a person is reading that
jumped from the last row to the first would move the thing they were about to
choose out from under them. The kinds of work are a ring somebody steps around
to see what there is, and there is no last one to fall off.

**Narrowing returns to the first application**, because the row somebody was on
belongs to the list they have just left.

## The design these are drawn from

Six states on page `03 — alo OS · Living canvas`, built this session and
approved by the owner:

| state | node |
|---|---|
| where it opens from | `519:30826` |
| browse | `514:30765` |
| one application | `515:30776` |
| installing | `515:30971` |
| installed | `515:31172` |
| nothing found | `516:30809` |

The window is **632 wide**, which is the widest window this design has, and it
uses the file's own `Canvas / Window title` component (`71:2`) — minimise,
focus and close, with the resize grip in the corner every other window keeps it
in. The eighteenth Dock icon was drawn for *Apps*, which was the only pinned
application with no artwork and which the specification was therefore drawing
as a letter.

**Two faults in the first drawings were caught by reading the repository rather
than by looking at them.** The market was first built as a full-screen page,
and nothing in this design is one — the largest surface anywhere in the file is
632 wide, and the product's own words are *your canvas, everything where you
left it*. And every Install button was deep teal, which ADR 0067 reserves for
the agent; a primary action here is `text/primary`, which is what
`Action / Erase and install` is. Both are corrected in the file.

## What was checked

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` and
  `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` on the final
  tree, after the last hand edit.
- `cargo test -p alo-software --lib`: **125 passed, 0 failed**.
- `cargo test -p alo-shell --lib market`: **22 passed, 0 failed**, listed by
  name rather than inferred from a total.
- `cargo test --workspace`.

Clippy refused `Category::EVERY[Category::EVERY.len() - 1]` in a test as
indexing that may panic; it is `Category::EVERY.last().copied()`, which is
better than what it replaced.

## What was not built

**No raster. Nothing on a machine draws this window, and no key reaches it
yet** — it is a model with tests and no surface. That is scope cut rather than
depth cut: everything here is finished, and what is missing is the next change
rather than the other half of this one.

**No host carries the ask out, no detail view, no search.** The window asks and
waits; what reads `what_they_asked_for`, puts the errand on the indicator and
calls `alo_software::install` is the next change after the raster. The detail
view and the search field are drawn in the design and are not modelled here.

**Nothing has run on hardware.** Whether the eight applications install, start
and draw on an alo OS machine is unmeasured, and stays unmeasured until there
is a screen to install them from.
