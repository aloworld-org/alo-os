# An identity filter is only as specific as the identity

**Found 2026-10-07, Mac lane, while answering *do I have anything open*.**

## What was concluded

That this lane had no open pull requests. The check:

```
gh pr list --author @me --state open
```

It answered **none**, and I reported none to the owner.

## What was true

The claim was right and the evidence was not. `--author @me` is the
**authenticated account**, and in this repository *every lane commits as the
repository owner* — `CLAUDE.md` requires it in as many words, and forbids an
agent setting an author of its own. So all four lanes share one account, and
`--author @me` has never meant *this lane's pull requests*. It means **every
lane's**.

It read correctly only because no other lane happened to have one open at that
moment. Minutes later the third PC opened theirs and the same command answered
**one** — a pull request in a crate I do not own, which I would have had to
look at to discover was not mine.

## The mechanism

**An identity filter is only as specific as the identity it filters on**, and
here the identity is deliberately shared. The authorship rule exists so that
the work stands as the owner's; a consequence nobody wrote down is that
authorship stopped being able to tell lanes apart. The filter did not become
wrong — it was never measuring what I read it as.

This is the family this repository keeps meeting — *a check whose inputs are
less specific than its question* — with one feature that makes it worse than
most: **the evidence was right by accident.** A check that is wrong announces
itself the first time it matters. A check that is right for the wrong reason
agrees with the truth until the day it does not, and nothing distinguishes the
two from the inside. I had already reported the conclusion.

## The cure

**Filter on whatever the convention actually varies.** Here that is the branch
prefix, which is what the naming convention exists for:

```
gh pr list --state open --json number,headRefName \
  | filter headRefName starting "task/mac/"
```

`task/mac/`, `task/third-pc/`, `task/panel/`, `task/dev-pc/` — the prefix is
the lane, because the prefix is the thing lanes do not share.

The general rule, which is the part to keep: **before trusting a filter, ask
what it would answer if every row belonged to somebody else.** If the answer
is *the same thing*, it is not a filter.

And where a check happens to agree with the truth, that is not confirmation.
**Ask why it agrees.** The reason here was *nobody else has one open just now*,
which is a fact about the minute rather than about the repository.

## What this does not say

Nothing against the authorship rule, which is settled and right: the work is
the owner's and no agent invents a contributor. The fault is entirely in
reading an account as a lane.

## Related

- [`i-watched-the-client-and-called-it-the-job.md`](i-watched-the-client-and-called-it-the-job.md)
  — a `pgrep` on the host standing in for a job in the guest.
- [`a-formatter-that-writes-is-not-a-check.md`](a-formatter-that-writes-is-not-a-check.md)
  — a field in a report that came from a command that mutates.
- [`i-killed-the-compiler-and-left-the-linker.md`](i-killed-the-compiler-and-left-the-linker.md)
  — a check built from the same list as the action it verified.
