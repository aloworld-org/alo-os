# The remover says which of two things it found

**Branch:** `task/mac/the-remover-says-which-of-two-things-it-found`, off `main`
**Date:** 2026-10-17
**Decisions:** none new.
[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md) term 6's own words
are *"not yet on this machine"*, and this is the journal catching up with them.

## The judgement, in one sentence

**This makes an existing promise true**: ADR 0045 term 6 promises a machine that
cannot keep an undo *answers honestly*, and the sentence it answered with said
something else — so nothing is added for anybody, and a sentence is made to mean
what the term says it means.

## What was wrong

There are two ways for a run to remove nothing, and they are **not the same
fact**:

| what happened | what somebody should do about it |
|---|---|
| the filesystem cannot keep an undo at all | reinstall, or accept undo will never work here |
| it looked and the folder held nothing | nothing — this is a machine working |

Both were written to the journal as:

```
alo-letting-go: this machine keeps nothing an undo could put back, so there was
                nothing to let go of
```

**That reads as the second while it means the first.** So the machine on which
undo can *never* work was reported in the words of a machine on which it is
simply idle — and the reading it invites is the one that needs no action, which
is how a machine that will never keep an undo becomes the one nobody looks into.

Both binaries said it, in their own words for their own job.

## What it says now

```
alo-letting-go: this machine's filesystem cannot keep an undo at all, so there is
  nothing to let go of. **It is not that nothing is kept — it is that nothing can
  be**, until this machine is reinstalled on a filesystem with snapshots (ADR 0045
  term 6). A machine that keeps nothing *yet* says so differently, and that
  difference is the point of this sentence.
```

and the forgetter's, which ends with what it means for the person who asked:
*"Whoever asked to be forgotten has had their asking answered truthfully: there
was never anything to forget."*

Measured on a machine, both units, read from the journal with a cursor rather
than `tail` so no previous run could be read.

## What this is not

**It is not translated, and that is not an oversight.** These go to
`StandardError=journal`, read by whoever administers the machine, and both
units' own comments say so. What a *person* reads is the record the run wrote,
in their own language, through `alo-changing-undo`. A journal line in the
language of the service log is what every other unit here does.

## Where it came from, and a claim of mine that was wrong

The desktop lane raised it, from a different angle: that *a stale binary* and *a
non-keeping filesystem* produce the same sentence, so on a `btrfs` machine the
remover's honest answer would be ambiguous exactly where it matters.

**I measured that and it is not true**, which is worth saying plainly because it
changes what their Sunday script needs. The two roads print different things:

```
folder absent     → /var/lib/alo/undo was not asked about: No such file or
                    directory (os error 2)
                    0 outside the window, 0 for room
folder present,   → this machine's filesystem cannot keep an undo at all ...
not keeping
```

The first **names the path**, so a remover built before the folder moved says
`/var/lib/alo/undo` and identifies itself. Their `strings`-the-binary
precondition is therefore belt-and-braces rather than the thing standing between
them and a false pass — and they should know that, because a precondition
believed to be load-bearing gets trusted in places it is not.

**The ambiguity underneath was real, though**, and it is the one fixed here: not
between two binaries, but between two facts in one sentence.

## A fourth fault in my own measuring, for the day's tally

The first time I measured the folder-absent road on this branch, it printed the
**filesystem** sentence rather than naming the path, and I nearly wrote that up
as the two roads having collapsed into one. They had not. This branch is off
`main`, where `THE_FOLDER` is still `/var/lib/alo/undo` — the move is in #744 and
unmerged — and I had created that very directory two commands earlier while
setting up a different case. So the folder existed, the filesystem does not keep,
and road two was correct.

**Two paths, two branches, one machine**, and the fixture from one measurement
silently satisfied the next. The fix was to read the constant on the branch I was
standing on rather than the one in my head.

## What runs forever

`crates/alo-letting-go/tests/the_two_answers_a_person_reads_are_not_the_same_sentence.rs`
— five tests over both programs' source:

- the phrase that blurred the two facts is in neither;
- each says the filesystem **cannot** rather than does not, *and* says that
  nothing *can* be kept — the positive half, without which deleting the sentence
  entirely would satisfy the negative one;
- each names what would change it (*reinstalled*) and cites the term that makes
  it an answer rather than a failure;
- neither claims the other's job, so a journal holding both units' lines needs no
  unit name to tell which ran;
- **the control**: each file is a program at all and over 500 bytes, and the
  forbidden-phrase filter is run over a line that does contain it — because a
  `said_by` returning an empty string would satisfy every `!contains` above.

**Mutation-proved.** Reverting `alo-letting-go.rs` to the old sentence fails
three of the five, and the two controls stay green, which is what they are for.

**And the test normalises whitespace before matching**, because Rust string
literals here are wrapped with a trailing `\` that swallows the next line's
indentation — so a sentence a reader sees as one phrase is two fragments in the
file. The first version of this test did a plain `contains` and **failed on a
sentence it was asserting about**. That is written into the function's own
documentation, since the next person will reach for `contains` too.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings` clean.
`doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings` clean.
`cargo test --no-fail-fast -p alo-letting-go -p alo-changing-undo -p alo-citing
-p alo-image`: **0 failures summed across every result line.**

**What this did not run**: the rest of the workspace, because the change is two
`eprintln!` strings, two doc comments and one new test file. CI runs the whole
suite and is the required check. This is a head status: these gates passed on
this tree, not on the tree that becomes `main`.
