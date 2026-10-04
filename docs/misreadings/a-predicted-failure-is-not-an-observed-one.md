# A predicted failure is not an observed one

**Concluded, twice in one day:** *this will conflict, so regenerate it* — and
*this cannot report a refusal, so fix it.*

**True:**

```
the vocabulary snapshot   predicted to collide after two lanes added a word on
                          the same day. Rebased, the test PASSED: the two keys
                          sort 1,230 lines apart and git merged them.
enqueue.py                "0 sys.exit calls, so it cannot report a refusal."
                          It uses `raise SystemExit`, and line 42 already
                          exited 1. It was never broken.
```

The first was caught, because the remedy was written but **gated behind *does
the test still pass***. The second was not: a working file got two unreachable
duplicate lines, and the proof offered for the fix — *exit 1 on a merged pull
request* — **would have passed before the change too**.

**The mechanism.** A prediction makes the remedy feel owed. The work then goes
into writing the fix rather than into checking whether the fault is there, and a
fix applied to something that was not broken **leaves a record claiming it
was**: a commit saying *resolved a conflict* about a conflict that never
existed, or a diff implying a file could not report failures when it always
could.

**The cure.** Write the remedy if you like, but **put the check in front of
it**: run the thing and look at the failure before curing it. And when a fix is
done, ask whether its proof would have passed on the unchanged file — **if it
would, the proof is about something else.**

**Named by** the applications lane, from watching the first case go right.
