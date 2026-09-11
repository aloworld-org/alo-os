# Parking survives the road that most often leads to it

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **the last known recovery gap, closed by one line and held by a test.**

Task 21's publish rebased the combined tree, hit a real conflict (two writers
had reconciled the same ledger entry in different words), and the rebase stopped
where git leaves it. Parking then tried `git switch --create` — which refuses to
move while a rebase stands — so the work could not be put anywhere safe and the
run stopped. Parking existed precisely so a failed task could not stop the run,
and the road that leads to parking most often — a conflicted publish — was the
one it could not survive.

The fix is one line at the top of `parked()`: `git rebase --quit`, dropped
rather than checked, before the branch is made. `--quit` keeps the working tree
exactly as it stands — conflict markers included, which are part of what a
person picking the branch up needs to see — and `main`'s own ref is never moved
by a rebase, so abandoning one loses nothing. When no rebase is in progress the
command refuses, and that refusal is the no-op it sounds like.

Held by `parking_abandons_a_stopped_rebase_before_it_switches`, on the source
the way the push rule is: the defect was an ordering in one file, and a fixture
rebase would have tested git rather than the order.

Found the way every one of these has been found: by the loop running into it.
The recovery paths never execute on a good day, and each real failure has been
the first execution of one.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a conflicted publish parks instead of stopping the run.
