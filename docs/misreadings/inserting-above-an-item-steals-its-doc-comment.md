# Inserting above an item steals its doc comment

**Found four times between 2026-09 and 2026-10-06** — in `alo-arranging`, in
`alo-access`, in `session_device.rs`, and most recently in `surfaces.rs` while
building `more-than-one-display-plan.md` task 7. Written up only after the
fourth, which is itself the finding.

## What was concluded

That adding a method above an existing one is a local change touching only
the new method.

## What was true

A doc comment belongs to whatever item follows it. Insert a new item between
a doc comment and the item it describes, and the comment silently becomes the
**new** item's — and the old item has none.

```rust
/// What this display is looking at, or the origin at life size.
pub(crate) fn camera_of(..) { .. }        // before

/// What this display is looking at, or the origin at life size.
pub(crate) fn camera_at(..) { .. }        // after: it took the comment
pub(crate) fn camera_of(..) { .. }        // and this one has none
```

Both items compile. The new one carries a description of something else, and
the old one carries nothing.

## The mechanism

**The anchor is at the wrong end.** These edits are made by script — find a
signature, insert before it — and a signature is the *start* of an item whose
documentation sits above it. Anchoring on the start of the following item
therefore lands **inside** the preceding item's text, between its comment and
its body.

Nothing about the edit looks wrong afterwards. The diff shows an addition and
no deletion, because nothing was deleted: a comment changed owner.

**Why four times rather than once.** The first three were each recorded where
they happened — a note on the function that lost its comment. That is the
right place to explain *this* function's history and the wrong place to stop
the next one: a note in `session_device.rs` cannot warn somebody editing
`surfaces.rs`. The practice was being recorded and the lesson was not
generalised, which is exactly what `docs/misreadings/` is for and why this
entry exists.

## The cure

**Anchor on the end of the preceding item, not the start of the following
one.** Insert after a closing `}`, never before a `pub fn`. The script form:
match the item you mean to follow and append, rather than matching the item
you mean to precede and prepending.

**When prepending is unavoidable, carry the comment deliberately.** Include
the existing doc comment in the matched text and re-emit it below the new
item, so the move is visible in the diff instead of invisible.

**And read the two items after any scripted insertion.** Not the whole file —
the item inserted and the one after it. That is the pair the fault always
lands between, and it takes seconds.

## What catches it, and what does not

`clippy::missing_docs_in_private_items` is denied in this workspace, so the
compiler **does** catch it — as *missing documentation for a method*, pointing
at the item that was robbed rather than at the edit that robbed it. That is a
real backstop and it is why none of the four reached `main`.

It is a slow backstop: it costs a full clippy cycle, the message names the
wrong site, and it would not fire at all in a crate without that lint. A
warning that arrives minutes later and points elsewhere is a poor substitute
for an insertion made at the right end.

## Related

- [`a-formatter-that-writes-is-not-a-check.md`](a-formatter-that-writes-is-not-a-check.md)
  — same day, also about a tool-shaped habit rather than a misread fact.
