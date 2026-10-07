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

### A control proves the search ran, not that it could have answered

**The formulation above is not the whole of it**, and the sharper version came
from the third PC within the hour — by following this entry's rule correctly
and still reaching a wrong conclusion.

They looked for `crates/alo-shell/tests/the_camera_has_one_home.rs`, found
nothing, ran a control in the same breath that matched `camera` in three other
files, and concluded the file was absent. It was not: it is on `origin/main`,
added by #474 and last touched by #556, and I had edited and run it that
afternoon.

**Their tree was current and their control was sound.** What they ran was
`grep -rn "camera_has_one_home" --include=*.rs .` — a search of file
**contents** for a **filename**. A test file does not generally contain its own
name, so it could not match whatever the tree. Verified: that file contains its
own name zero times, and a content search for it finds only the two prose
references in `the_session_stays_with_its_owner.rs`.

So:

> **A positive control proves the search ran. It says nothing about whether the
> search could have answered the question.**

Theirs ran perfectly. It confirmed the instrument while the instrument was
pointed somewhere else — their words: *I checked my instrument and never
checked my aim.* `ls` and `git ls-tree` were the question's own instruments and
neither was used.

### Two failures, two axes, one habit

| | tree | instrument |
|---|---|---|
| mine | **stale** | right — grep for a key in the vocabulary |
| theirs | current | **wrong** — content search for a filename |

A same-breath control catches neither. It cannot see the tree, because it is
subject to the same tree. It cannot see the aim, because it is aimed the same
way. **Both of us validated what we had rather than what we needed.**

So the cure has two halves, and the second is the one neither of us applied:

- **read the published tree** — `git show origin/main:<path>`,
  `git ls-tree origin/main <dir>` — which answers *does this exist* without a
  control at all; and
- **ask whether the instrument can answer the question.** Existence is
  `ls` or `git ls-tree`. Content is `grep`. A content search can only ever tell
  you about content, and no control will mention that it is the wrong tool.

*The third PC's case is described as what it was at their insistence: I had
first recorded it as a stale tree, which is my failure mode and not theirs, and
they corrected it before the entry could carry a wrong attribution. The rule
being followed correctly and the conclusion still being wrong is the stronger
argument, and it only works if the account is accurate.*

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
