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
something you know is there, and do it **in the same breath** — same pattern
shape, same path, same tree, same command. If the control comes back empty the
search is broken and the negative is worthless.

**In the same breath is the load-bearing part, not a convenience.** A control
run separately, or from memory, or earlier, is a *second claim* that needs its
own validation — and it validates whatever tree and pattern *it* ran against,
which is exactly the thing in doubt. My phrasing search was a second look in
the same stale tree: more thoroughness, no more evidence.

That is why the third PC's `probe_should_be_7` lives **inside the probe's own
output file** rather than being something asserted afterwards about having run
it. A control you report is a claim; a control in the artefact is a fact about
that artefact.

### A control cannot validate the tree it ran in

**And here is where the rule above is not enough**, found within the hour by
the third PC following it and still reaching a wrong conclusion. They searched
for a test file, found nothing, and ran a control in the same breath — the same
pattern matched three other files — so by the paragraph above the negative was
sound. It was not: the file is on `origin/main`, added by #474 and last touched
by #556.

A control verifies the **pattern**, the **path** and the **command**. It cannot
verify the **tree**, because the control is subject to the same staleness as
the search: a control in an old checkout passes exactly as well as one in a
current checkout. That is the single failure mode a same-breath control cannot
see, and it is the one that caused both of this entry's wrong claims.

So listing *same pattern, same path, same tree, same command* as though the
four were equally checked was wrong. Three of them a control establishes. For
the fourth, either:

- the control must be something that **would only exist if the tree were
  current** — a file or key you know landed recently; or
- skip controls and read the published tree: `git show origin/main:<path>`,
  `git ls-tree origin/main <dir>`.

The second is shorter and does not depend on remembering what landed when.

*Credited to the attempt rather than to the suggestion: the rule was followed
and the conclusion was still wrong, which is better evidence than anybody's
opinion about the rule.*

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
