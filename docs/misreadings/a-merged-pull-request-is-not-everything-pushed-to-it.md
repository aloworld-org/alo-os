# A merged pull request is not everything pushed to it

## What was concluded

> #949 is merged, so the two commits on its branch are on `main`.

The pull request said `merged: true`. The branch had been pushed. Both commits
were on the remote. Nothing anywhere said otherwise.

## What was true

**Only the first commit landed.** The merge queue merged #949 **between the two
pushes**: the first commit went up, the queue took the pull request, and the
second commit arrived at a branch whose pull request was already closed.

Measured by asking `main` for the content rather than the history:

```
$ git show origin/main:crates/alo-shell/src/bar_raster.rs | grep -c 'room: (bar.display.w'
0
$ git show origin/main:crates/alo-shell/src/a_key_reaches_the_bar.rs >/dev/null && echo present
present
```

One fix of the two. The other was a fix for a panel drawn at **less than half
its width**, which a person would see the moment they opened the Bar.

## The mechanism

Two things have to go wrong together, and each is ordinary on its own.

**A pull request is a pointer to a branch, and a merge is a snapshot of it.**
Pushing again after the queue has taken it is not an error and nothing refuses
it: the branch moves, the merged commit does not, and no field anywhere reads
*there is more here than was merged*. `merged: true` is true. It is just not an
answer to the question being asked of it.

**And the obvious way to check tells you the wrong thing.** This repository
squash-merges, so a landed branch's commits are **never** ancestors of `main`.
So:

```
$ git diff --stat origin/main...origin/a-key-a-person-types-reaches-the-bar
 8 files changed, 663 insertions(+)
```

Read straight, that says *nothing landed* — and it is wrong in the opposite
direction to the fault. Reading it would have meant re-opening work that was
already on `main`. The same squash is why `git branch --merged` reports nothing
and why `docs/misreadings/` already carries
[delete merged branches by asking GitHub].

So the history is unreadable in both directions at once: the pull request
overstates what landed, and the diff overstates what did not.

## The cure

**Ask `main` for the content, by name, and never the history.** For each thing
a change was supposed to deliver, one question:

```
git show origin/main:<path> | grep -c '<the line that is the fix>'
```

Present or absent. It cannot be confused by a squash, a rebase or a second
push, because it is not a question about commits.

**Do it as a build's first step, not as a review.** This was caught because the
image build verifies every fix by name before it starts — six checks, five
returned `1` and one returned `0`. That list costs six lines and it is the only
reason the panel fix was not silently absent from a machine built to
demonstrate it.

**And push before the queue can take it.** The window is real but small: it is
the time between a pull request going green and the queue merging it. Work that
arrives in that window needs its own pull request, and the honest signal that
it is needed is the content check above rather than a memory of what was
pushed.

## Related

- `delete-merged-branches-ask-github.md` — the same squash, making `git`'s own
  merged test wrong.
- `two-dot-diff-before-a-push.md` — the other way a diff against `main`
  answers a different question than the one asked.
