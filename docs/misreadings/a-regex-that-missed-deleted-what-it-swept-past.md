# A regex that missed deleted what it swept past

## What was concluded

That a multi-line `perl -0pi -e 's|...|...|'` over a source file had removed the
one function it named, and that the two `field is never read` complaints
clippy raised afterwards were pre-existing noise in a crate the change had
barely touched.

## What was true

The pattern did not match the function. It matched from that function's doc
comment to a closing brace **two functions further down**, and deleted all of
it: 80 lines, including `their_displays` and `the_shortcuts` — the code that
reads a person's own keyboard shortcuts and works out their display
arrangement and night light. Neither had anything to do with the change.

`shortcuts_at` and `minutes_east_of_universal_time` were not dead fields. They
were fields whose only readers had just been deleted.

**Every test still passed.** The deleted methods are trait implementations
driven by a compositor on a real machine; nothing in the suite calls them.

## The mechanism

**A regex that fails to match is silent, and a regex that over-matches is
silent in the same way.** `perl -0pi -e` reports neither: it exits 0 having
changed nothing, or exits 0 having changed far more than was meant, and the
two are indistinguishable from the command's own output. The usual guard —
reading the result back — was done, and it showed the *intended* edit present
at the place it was expected, because the damage was forty lines below where
anybody was looking.

What made it worse is that the fault reported itself **somewhere else and in
another vocabulary**. The signal was not *you deleted two methods*; it was two
unrelated-looking lint complaints about struct fields, in a file the change
had never opened. That is the easiest kind of signal to explain away, and it
was about to be explained away as pre-existing.

The same shape is already in this directory twice over, in
`a-search-with-no-output-is-not-a-search-that-found-nothing.md`: a tool that
answers nothing is not a tool that found nothing. This is its other half — a
tool that answers nothing is also not a tool that did only what you asked.

## The cure

**Count the diff, not the match.** After any scripted multi-line edit, read
`git diff --stat` for the file and check the deletion count against what was
intended. An edit meant to add a function is `N insertions(+)` and nothing
else; a non-zero deletion count on an additive change is the whole signal, and
it costs one command. `git diff -- <file> | grep '^-'` then names exactly what
went.

**And do not use a multi-line regex to move code.** Use an exact-string edit,
or restore the file and reapply by hand. Moving a function out of an `impl`
block is a structural edit, and a pattern language that cannot see the
structure will happily consume it.

**When a lint complains about something the change did not touch, that is a
finding and not noise.** The question to ask is not *was this already here* but
*what would make this true*, because a field whose readers were deleted and a
field that was always unused look identical to clippy and are not the same
event. Here, `git diff origin/main --stat` answered it in one line.

*Written 2026-10-24, the same hour it happened, by the lane that did it. The
80 lines never reached the remote: clippy's two complaints arrived before the
push.*
