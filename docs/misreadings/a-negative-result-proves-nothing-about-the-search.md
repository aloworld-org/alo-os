# A negative result proves nothing about the search

**Found 2026-10-07, Mac lane, by the third PC contradicting a claim of mine with
a measurement.**

## What was concluded

That `installer.will.give-alo-os` did not exist. I had searched for it, found
nothing, and told the third PC so — twice, because they had listed it in two
successive messages and I thought they were planning around a key that was not
there. I added that I had searched for the phrasing too, not just the key, to
show the search had been thorough.

## What was true

It exists. There are six `installer.will.*` keys on `origin/main` and I was
reading four:

```
installer.will.give-alo-os             alo OS gets {area} GB of that space …
installer.will.restart-keeping-windows This computer restarts once …
```

Both arrived with pull request 559, which merged **about half an hour before I
searched**. My checkout predated it. The phrasing search was equally stale, so
the extra thoroughness added nothing except confidence.

## The mechanism

**A positive result is self-validating and a negative one is not.** A thing
that exists proves your search worked — it found something, so the pattern,
the path and the tree were all good enough. A thing that does not exist proves
*nothing whatever* about the search. The same empty output comes from a key
that is absent, a pattern that is wrong, a path that is wrong, and a tree that
is behind.

So `not found` and `not fetched` are **indistinguishable from inside**. The
honest version of what I had was *my search returned nothing*, and I reported
*it does not exist*, which is a different claim with a fact added that I had
not measured.

**It generalises past git.** Every empty result in this repository has this
shape: a command that produces no output and a command that did not run look
identical. The third PC's probe files carry `probe_should_be_7` for exactly
this reason — a positive control, so the query proves it ran before its silence
is allowed to mean anything.

*Formulation owed to the third PC: this is distinct from looking in the wrong
place, which `docs/misreadings/` already has. This one is looking at the wrong
**time**.*

## The cure

**Give every negative a second fact.** For git, that is reading the published
tree rather than the working one:

```
git show origin/main:path/to/file | grep …     # or fetch, then search
```

which is what I used the moment I was challenged, and which would have
answered correctly the first time.

More generally, **pair a negative with a positive control**: search for
something you know is there, in the same breath, with the same pattern and
path. If the control comes back empty the search is broken and the negative is
worthless. `probe_should_be_7` is that idea made permanent.

And when reporting one: say *my search returned nothing* until the control
passes. The gap between that and *it does not exist* is exactly the fact that
has not been measured.

## What it cost, and why it is written anyway

Nothing, because the third PC checked it. Two wrong claims passed between the
two lanes this afternoon — this one, and their near-miss on a sentence
promising disk space would come back on a road where the program that would do
it refuses to run. Neither reached a branch, and **neither was caught by a
gate**. Both were caught by the other lane reading.

That is worth recording next to the mechanism, because the apparatus this
repository leans on did not and could not catch either: no test holds a
sentence to a filter, and no gate knows when a checkout is behind.

## Related

- [`an-identity-filter-is-only-as-specific-as-the-identity.md`](an-identity-filter-is-only-as-specific-as-the-identity.md)
  — the same day: evidence that was right by accident.
- [`a-formatter-that-writes-is-not-a-check.md`](a-formatter-that-writes-is-not-a-check.md)
  — a field that could not fail, read as a verdict.
- [`i-killed-the-compiler-and-left-the-linker.md`](i-killed-the-compiler-and-left-the-linker.md)
  — a search built from the same list as the action, which could only confirm
  itself.
