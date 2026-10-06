# Two predicates in one file, and only one was armed

**Concluded:** *`matches!` is a non-exhaustive match, so a predicate written
with it cannot be broken by adding the variant it should have been told about.*
In `crates/alo-installing/src/program.rs`, `Program::path()` is an exhaustive
`match` and `Program::writes()` was a `matches!`. Adding a new program forced
the first to be updated and could not touch the second.

**What the second one is for.** `writes()` answers *did this run write to a
disk*, and it is what every refusal test counts:

```rust
fn wrote(&self) -> bool {
    self.ran.iter().any(Program::writes)
}
assert!(!machine.wrote(), "a refusal wrote to a disk: {:?}", machine.ran);
```

That assertion is the whole of the promise *nothing was changed* — the sentence
every refusal in this installer ends with, and the one thing a person most needs
to be true.

**What was added.** `Program::MakingTheRoot`, which runs
`mkfs.btrfs --force --label root` on one partition. It is the irreversible step
of the road that puts alo OS beside Windows, and on a disk that is being kept it
is the single most destructive thing the environment can do: pointed at the
wrong partition it is the end of somebody's Windows.

```rust
// as it stood
matches!(self, Self::Writing(_) | Self::RemovingTheArea { .. })
```

The compiler required `path()` to gain an arm — a program with no path does not
compile. It required nothing of `writes()`. **So for the length of that change,
every refusal test on the new road would have passed with a file system
already made.**

**Both are predicates about the same list and only one is a claim.** The
difference is not care or placement; they are eleven lines apart in the same
`impl`. It is the shape: `match` with no wildcard is re-read by the compiler on
every build, and `matches!` is a question asked of a list that was complete the
day it was written.

**Why `matches!` is the natural thing to reach for**, which is the part worth
remembering. It reads better. `matches!(self, A | B)` is one line and says
exactly what it means; the exhaustive form is fourteen lines of `=> false`. The
version that is easier to read is the version nothing checks, and at the moment
of writing there is no cost: the list *is* complete, the predicate *is* correct,
and the test *does* pass. The debt is created by a change that has not happened
yet, to a file somebody else will open.

**It is this directory's standing subject in a new place.** A check whose inputs
are less specific than its question — here, a check whose *body* silently
stopped matching its *name*. `writes()` kept answering the question it was given
in 2026-09, which was *is this one of the two programs that write*, long after
the question being asked of it had become *does a disk come out different*. Both
readings are defensible from the name. Only one of them is what the test means.

**The question that catches it** is not *is this predicate right* — it was right
— but:

> **When somebody adds a variant to this enum, which of the functions over it
> will the compiler make them visit?**

For `path()`: all of them. For `writes()`, `is_read()`, and anything else
written as `matches!`: none.

**What was done.** Both are exhaustive matches now, with a line per variant and
no wildcard, so the next program added cannot compile until somebody has decided
whether it writes to a disk and whether what it prints is read. The reason is in
the code beside them, because the next reader's instinct will be to collapse
them back into one line.

**Found while writing the walk** that asserts no partition of Windows' is ever
named to any program. The walk needed to know what *nothing was written* counts,
read `writes()`, and the answer did not include the program that makes file
systems.

**The general form, for the next enum.** A predicate over an enum that some test
trusts as a guarantee is not a convenience function; it is part of the
guarantee. Write it the way the guarantee deserves — exhaustively — and let it
be ugly. The one-line version is a sentence nobody has to read again.
