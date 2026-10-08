# Inserting above an item steals its doc comment

**Found six times between 2026-09 and 2026-10-07** — in `alo-arranging`, in
`alo-access`, in `session_device.rs`, and most recently in `surfaces.rs` while
building `more-than-one-display-plan.md` task 7. Written up after the
fourth, which was itself the finding; the fifth and sixth happened in one
morning on the lane that had just read it, which is a second finding and is at
the bottom of this file.

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

## It also lands between an attribute and the doc comment

**2026-10-07, twice in one morning**, in `alo-installer` while wiring the
Windows half of alo OS beside Windows. Both inserts were scripted, both
anchored on a declaration, and the cure above was in this file at the time.

The first landed where this entry describes, between a doc comment and its
function, and took three paragraphs with it. The second landed one line
higher - **between the item's `#[must_use]` and its doc comment**:

```rust
/// Its name after the restart, whatever it is to this installer now.
/// ...
#[must_use]                       // this belongs to `after_the_restart`
/// The number of the start-up area on this disk, where it has one.
/// ...
#[must_use]
pub fn the_start_up_area(&self) -> Option<PartitionNumber> { .. }

pub fn after_the_restart(&self) -> Option<DiskName> { .. }   // no doc, no attribute
```

So an item is not *doc comment, then declaration* - it is a run of doc
comments and attributes in whatever order they were written, and **every line
of that run is somewhere an insert can land.** Anchoring on the declaration
puts the new text at a point chosen by how many attributes the previous item
happens to carry.

**What the compiler said was `unused attribute ... attribute also specified
here`.** That is `rustc`, not clippy, and it fires because two `#[must_use]`
ended up on one item. It names the right file and describes the wrong problem:
the message is about an attribute, and the damage is a paragraph. Had the
robbed function carried no attribute, nothing would have been reported at that
step at all.

**And the lint named above is a narrower backstop than it reads.** It asks
whether an item has documentation, never whether the documentation is about
that item - so it fires only when the robbed item ends up with *none*. An
insert that appends its own first line to the previous item's comment leaves
both items documented, and both the lint and the rustdoc gate pass. That is the
first of these two, and it was caught by reading the diff rather than by any
check.

**What the lost paragraph said**, which is why this is worth another section
rather than a larger number:

> Partition numbers on a disk are Windows' to reassign, and formatting or
> removing by number alone would be one renumbering away from destroying
> somebody's partition. The place a partition begins does not move.

That is the reason a guard exists in a program that repartitions somebody's
only computer. A reader finding that function undocumented a year from now
would not know it had ever had a reason.

### The cure, made specific enough to follow

The cure above is right and was not followed, because *anchor on the end of the
preceding item* is a sentence while `fn still_the_area(` is a string that is
easy to search for. The specific form:

> **Match the last line of the previous item's body - its closing `}` or its
> trailing `;` - together with enough of what is above it to be unique, and
> append after it. Assert the match occurs exactly once before writing
> anything.**

Both scripts here asserted their anchor was unique, and both anchors *were*
unique. Uniqueness is not the property that was missing; **being at the end**
was. An assertion that an anchor is unique says nothing about whether it is in
the right place.

## Related

- [`a-formatter-that-writes-is-not-a-check.md`](a-formatter-that-writes-is-not-a-check.md)
  — same day, also about a tool-shaped habit rather than a misread fact.
