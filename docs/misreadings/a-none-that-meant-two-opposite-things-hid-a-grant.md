# A `None` that meant two opposite things hid the grant that most needed showing

**Date:** 2026-10-25. **Found by:** the lane that had just built the thing,
while wiring the surface — not by a test, and not by review.

## What was concluded

That *a grant may last until the person removes it* was built. Two crates had
landed with their own tests green: `alo-capability` could make the grant
(`Grant::checked_until_removed`, 5 new tests), and `alo-remembering` could keep
it on the disk (`expires = "removed"`, format 3, 5 new tests). What was left
looked like wording — the picker's fourth row and a line in Settings.

## What was true

**The grant was invisible and therefore permanent.** On the list a person
revokes from, it had no row at all. Measured before the fix, on the machine's
own `Grants`:

```
assertion `left == right` failed: an endless grant has no row
  left: 0
 right: 1
```

So the daemon honoured it, the file kept it across restarts, and there was
nothing in Settings to take it back by. Of the four properties ADR 0001 §3
gives a grant — enumerated, visible, revocable, expiring — this one had given
up *expiring* on the owner's instruction, and had then quietly lost *visible*
and *revocable* too, which were the whole of what was supposed to replace it.

## The mechanism

`Grant::expires_in(now)` answers `Option<Duration>`, and it answers `None` for
**two opposite facts**: a grant that has run out, and a grant that never will.
The list was built before the second existed, so it read `None` the only way it
could then mean:

```rust
grant.expires_in(now).map(|left| Seen { .., left })   // no row when None
```

Every test around it was correct. `alo-capability`'s said `expires_in` answers
`None` for a grant with no end — deliberately, with a comment explaining that
the two are told apart by `lasts_until_removed` instead. `alo-granted`'s said
an expired grant is never shown as live. Both were right about what they
tested, and the composition of two correct beliefs was the bug.

**That is the shape to watch for: not a function that is wrong, but a function
whose return type cannot express a distinction its caller depends on.** The
`Option` was a fine summary of *how long is left* while every grant ended. The
day one did not, it became a summary that threw away the only thing a caller
needed to branch on — and nothing anywhere had to change for that to happen,
which is why no test went red.

It is the same family as the four already in `CLAUDE.md`'s gate — *a check
that stands in for the thing* — seen from the other side. The question there is
*what would have to be true for this to pass while the product is broken*. Here
the answer was: **a second meaning arrives for a value that already had one.**

## The cure

Three parts, and the first is the only general one.

1. **Where two facts share a value, the type carries the fact, not the value.**
   `alo_granted::Lasts` is now `For(Duration)` or `UntilRemoved`. A surface
   cannot read a row's remaining time without having decided which it is,
   because there is no `Duration` to read until it has matched. `Seen::of` asks
   `lasts_until_removed()` **before** anything reads a duration.
2. **When a protection is retired, test what replaced it, not what went.**
   The owner retired *expiring* for one length. The protection that replaced it
   was *findable and revocable*, and nothing tested that until this was found.
   `crates/alo-granted/tests/a_grant_until_removed_is_on_the_list.rs` does, with
   a control at the other end: a grant that ends is still dropped once it has,
   so a list that simply kept everything fails too.
3. **The walk, not the unit.** `a_folder_given_until_removed_is_on_the_list_and_says_so`
   in `crates/alo-shell/src/settings_window_tests.rs` presses the keys a person
   presses — open the picker, into the folder, give it, choose the length — and
   then reads the row, its wording and the bytes on the disk. Two crates with
   green suites and no walk between them is what this entry is about.

**Did anything catch it?** No. It was found by building the next layer, which
is the third time in two days the same sentence has had to be written in this
repository: *both ends built, the join missing*. Each one was found by using
the thing, never by a test.
