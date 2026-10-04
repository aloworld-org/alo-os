# A breakdown has a check inside it, and I shipped one unchecked

**Concluded**, in a committed decision record: **52 files name that unit — 18
`.rs`, 26 `.md`, 4 `.service`, 1 `.toml`.**

**True:** `18 + 26 + 4 + 1 = 49`. And the real split, measured on main at
`0d2ac4db`:

```
  27 md
  18 rs
   4 service
   1 toml
   1 conf                    ← omitted entirely
   1 image/Containerfile     ← omitted entirely
  --
  52
```

Three errors in one sentence: a wrong count, two whole file types missing, and
**a total that the parts beside it contradict**. The 52 was right; everything
explaining it was wrong.

**The mechanism.** The total and the breakdown were measured at different times
and never compared, because a breakdown *looks* like evidence for the total
rather than a claim of its own. **It is both**, and the arithmetic between them
is free. A reader who trusts the sentence inherits 49 as the size of the job.

This also hid behind a true neighbour. The same paragraph said **six
directives**, a re-measure printed **9**, and the 9 was right about a different
question — `grep -c` counts comments, and three comments name the number beside
the six directives. **Checking the number that was wrong is what surfaced the
one that was fine**, and the opposite would have been as likely.

**The cure.** Any split quoted next to a total gets added up before it ships —
and when the claim is load-bearing enough to sit in a decision record, **put the
addition in the script that writes the file**, so the commit cannot be made while
the two disagree. That check now runs on this ADR's own text:

```
  claimed total: 52
  parts:         [27, 18, 4, 1, 1] plus 1 named file(s) = 52
  the breakdown sums to its own total
```

**And name the scope in the sentence.** The count is 52 on main and 53 at the
branch head, because **the record naming the unit is itself one of the files
naming the unit**. A bare "52 files" would have gone stale the moment it merged.
