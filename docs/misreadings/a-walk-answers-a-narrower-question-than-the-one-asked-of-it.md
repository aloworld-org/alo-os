# A walk answers a narrower question than the one asked of it

**Concluded** of a green walk: *it passes, so the thing works.*

**True**, of the surface walk that holds a report's table to what the
accessibility tree reads aloud:

```
it caught a stale table the moment the code moved          — the good case
a surface short by two rows matches a report short by two   — the blind case
```

The same artefact **catches drift and is structurally blind to omission**, and
those are **different guarantees wearing one green tick**. A walk is also the
thing a lane points at when asked whether something really works, so the narrow
answer gets read as the broad one.

**The mechanism.** A walk compares two things it can see, and **absence is not
one of them**. Nothing in a pairwise comparison asks whether the pairs are all
there.

**The cure.** Ask of every walk: **what would this do if the thing were absent
entirely?** If the answer is *pass*, it checks agreement rather than existence,
and existence wants its own assertion — *every drawn control has a row*, not
*every row matches*.

Then **write which guarantee it gives into the walk's name and its report**,
rather than letting a green tick imply the larger one. The surface walk's report
now says so instead of having its table quietly corrected.

**Found by** the applications lane while landing
[ADR 0089](../decisions/0089-what-a-control-is-called.md).

## The second example this entry used to carry was wrong

It cited `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs`,
whose `assert_eq!(came_up_twice.first(), came_up_twice.get(1))` passes on
`[false, false]`. **That assertion is deliberate, documented and dated, and this
entry had it backwards.** The mistake is its own entry —
[the reason was above the line I judged](the-reason-was-above-the-line-i-judged.md)
— and is left here rather than deleted, because an entry quietly losing its
strongest example teaches less than one that says why it lost it.
