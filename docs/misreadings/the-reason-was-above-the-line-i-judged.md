# The reason was above the line I judged

**Concluded**, and said out loud twice before it was checked: *the installer's
walk asserts two rounds agree, `[false, false]` agrees, so it is green about a
machine that never booted — the clearest instance we have of a check that passes
in the failure case.*

**True.** The assertion is at line 1127. The test's own doc comment begins at
line 1036, under this heading:

> `## What this holds, and the one thing it cannot hold yet`

and says:

> **It cannot yet hold that the machine came up at all**, and that is not an
> oversight. The installed system's boot entry carries no console argument, so
> it says nothing on the serial line this reads and `came_up` is `None` **even
> when the machine has reached a login prompt — measured on 2026-09-27, when
> both rounds booted fine and reported nothing.** Until the installer plan's
> task 21 gives the installed system a console, *booted and idle* and *hung* are
> the same picture here... **Asserting `came_up.is_some()` today would fail on a
> working machine, which is why it says what it says instead.**

Every part of the criticism was already answered in writing:

```
the test's name     the_kept_computer_is_the_same_computer_twice
                    — named for the guarantee it gives, which is the cure the
                      neighbouring entry recommends. Already followed.
the same file       asserts came_up.is_some() at lines 1270 and 1753, where it
                    is valid. The author knew the assertion existed.
the gap's owner     the installer plan's task 21, named in the comment.
the evidence        a dated measurement. I had none against it.
```

**The mechanism.** A grep lands on a line, and a line has no context. The
judgement then gets made on the eight characters in front of the cursor, while
the paragraph that exists **specifically to answer that objection** sits twenty
lines above, outside the match. In a repository that writes its reasons into doc
comments, **grepping to an assertion is a way of skipping the argument.**

Two things made it worse rather than better. The criticism was **confident and
repeated** — once in a report, once in an entry heading toward `main` — and it
was about **another lane's crate**, so the first person to read it would have
been the author of the thing being misdescribed. And acting on it would have
been the `enqueue.py` error at a larger scale: asserting `is_some()` would
**make a working machine go red**, a fix applied to something that was not
broken, this time published as a criticism of someone else's work.

## This is a repeat, and that is the important part

[Counting arms does not read the paragraph underneath](counting-arms-does-not-read-the-paragraph-underneath.md)
is already in this directory, from earlier the same week. Its subject was
`permitting.rs`'s heading **Two arms, and what is deliberately not a third**,
which says *there never will be one here*. Its stated mechanism: **a gap and a
deliberate omission look identical to a count — the difference is in prose
immediately below, and counting does not reach it.** Its stated cure: *read the
file's header and the paragraph beneath the declaration before concluding
something is missing.*

That is this fault exactly, written down, by me, thirteen entries earlier. **It
did not fire.**

So the honest finding is not about installers. **A cure phrased as *read more
carefully* is advice to somebody who has already decided to look**, and the
whole failure is that no decision to look ever happened. There was no moment of
uncertainty to attach the rule to: a line was found, it looked wrong, and
looking wrong felt like knowing.

**The cure has to be mechanical, or it is the same entry again.** What this lane
now runs before judging any declaration or assertion it did not write:

```
bash /root/the-reason.sh <file> <line>
```

It prints the file header and the doc comment of the enclosing item, and nothing
else. Reading the reason stops being a decision and becomes the cost of having
an opinion. Where that is not available, the written form is **quote the prose
that would have to be wrong** — if nothing can be quoted, the file has not been
read.

**One more thing outranks reading the code: a dated measurement in a comment.**
*Measured on 2026-09-27, when both rounds booted fine and reported nothing* is
evidence. Nothing was offered against it, and nothing could have been from this
machine, which has never run that test.
