# A formatter that writes is not a check

**Found 2026-10-06, Mac lane, when a pull request whose local gate read
`FMT=0` failed CI on formatting.**

## What was concluded

That the tree was formatted. My gate script printed three fields and I read
all three as verdicts:

```
FMT=0        cargo fmt -p alo-shell
CLIPPY=0     cargo clippy --workspace --all-targets -- -D warnings
SUITE-EXIT=0 cargo test -p alo-shell
```

Two of those are checks. The first is not.

## What was true

`cargo fmt` **writes**. It reformats the files and exits 0 because
reformatting succeeded. `FMT=0` therefore means *formatting was applied*, and
it would print `FMT=0` on a tree that was wildly misformatted a moment
earlier — that is the whole point of running it.

So the field could not fail, and what it actually told me was nothing. Pull
request 547's gate read `FMT=0`, I then added two test files, committed, and
pushed a tree whose newest files had never been through the formatter. CI,
which runs `--check`, failed it.

Two smaller faults rode along: the script formatted `-p alo-shell` while CI
checks the **whole workspace**, and the step ran at the *start* of the gate,
so anything edited afterwards was never covered even in the writing sense.

## The mechanism

**A command that repairs the thing it is asked about always succeeds**, and in
a column of exit codes it is indistinguishable from one that inspected it.
`CLIPPY=0` and `SUITE-EXIT=0` are claims about the tree. `FMT=0` is a claim
about `cargo fmt`.

This is `CLAUDE.md`'s *a check that stands in for the thing is not the thing*
with a twist worth naming separately: the stand-in here is not a narrower
measurement, it is **not a measurement at all**. The question that catches it
is the one that file already asks — *what would have to be true for this to
pass while the product is broken?* — and the answer was *anything at all*.

It survived because the other two fields in the row are real, and a row of
three numbers that agree reads as corroboration. One of them was never voting.

## The cure

**Run the formatter in the mode that can fail**, over the same scope CI uses:

```
cargo fmt --all -- --check      # fails on an unformatted tree
```

and if the gate should also *fix*, format first and then check, so the check
is what reports:

```
cargo fmt --all ; cargo fmt --all -- --check ; echo "FMT=$?"
```

**And format before committing, not before gating.** The ordering matters as
much as the flag: a step that runs at the start of a long gate says nothing
about edits made while it ran.

The general rule, which is the part to keep: **when a field in a report comes
from a command that mutates, it is a log line and not a verdict.** Put
verdicts and actions in different columns, or the actions will be read as
verdicts by whoever reads the row quickly — which, a week later, is everyone.

## Related

- [`i-watched-the-client-and-called-it-the-job.md`](i-watched-the-client-and-called-it-the-job.md)
  — same day, same shape: a signal that could not observe what its name
  claimed.
- [`a-fixture-i-wrote-agreed-with-the-parser-i-wrote.md`](a-fixture-i-wrote-agreed-with-the-parser-i-wrote.md)
  — same day: four tests that passed about a shape the server never sends.
