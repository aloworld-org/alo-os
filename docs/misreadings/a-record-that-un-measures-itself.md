# A record that un-measures itself

**Found 2026-10-22, Mac lane, while measuring how often the gate's 45-minute
limit actually bites — and found only because the answer came back cleaner than
the evening had been.**

Every other entry in this folder is about a **check** that measured nothing.
This one is about a **record** that stops having measured. The fault is not in a
test or a flag; it is that the act of repairing an instance deletes the evidence
that the instance happened, so the problem gets quieter the more diligently it
is fixed.

## What was concluded

The question was narrow and ought to have been cheap: *does
`gate.yml`'s `timeout-minutes: 45` actually cancel runs, or is it comfortable
headroom?*

The obvious measurement is the run history. It says:

```
$ gh run list --workflow=gate.yml
success   task/mac/law-five-says-what-a-choice-is
success   task/mac/the-roadmap-says-what-is-true
...
```

Read straight, that is *the limit is not a problem*. I had **watched both of
those runs get cancelled at the limit earlier the same evening** and reran them
myself. Had I not, I would have had no reason to doubt the listing, and the
honest conclusion from it would have been that the limit does not bite.

## What was true

The conclusion a listing shows is the conclusion of the **latest attempt**. A
rerun does not add a row; it replaces the verdict on the existing one.

```
run 39808195998   conclusion=success   run_attempt=2
  attempts/1:     conclusion=cancelled

run 39808660869   conclusion=success   run_attempt=2
  attempts/1:     conclusion=cancelled
```

The timeout survives, at `/actions/runs/<id>/attempts/1`, and **nothing asks for
it**. Not `gh run list`, not `gh run view`, not the checks API, not the pull
request's own page. A reader has to already suspect the fault to find the
evidence of it.

Measured on the same day, the limit had cancelled at least three jobs — and one
of them matters much more than the other two:

| | |
|---|---|
| `asking-alo-says-what-it-needs` | 45 min, branch gate |
| `asking-alo-says-what-it-needs` | 45 min, branch gate |
| **`gh-readonly-queue/main/pr-940-0b4144cf`** | **45 min, the queue's own build** |

## Two separations the measurement needed first

Both were wrong on the first attempt, and the numbers would have been useless
without them.

**`cancelled` is mostly not a timeout.** On one branch, five cancellations: one
was the limit, and the other four ran 5, 14, 16 and 19 minutes — superseded
pushes. Counting `conclusion == cancelled` overstates the fault **fourfold**,
which is worse than useless: a watcher that cries *rerun* four times out of five
is one a lane learns to ignore, and then the fifth goes unread too.

**The run's clock is not the job's clock.** `createdAt → updatedAt` on the run
includes queue wait, and made one cancellation read as **51 minutes** against a
45-minute limit — a figure that cannot be true and would have discredited the
whole measurement. The discriminator is the **job's** own `started_at →
completed_at`.

## The mechanism

**A timeout on a queue build is not slowness, it is silent disappearance.** The
merge queue builds its own commit — the one that becomes `main`. When that build
is cancelled, the pull request falls out of the queue. It keeps its green head
status, keeps its approval, and simply stops being in line. Nothing says so.

That is what happened to #940: a watcher logged `re-enqueuing #940` and the lane
that wrote the watcher did not know why for forty minutes. Without something
re-enqueuing blindly, the change would have sat indefinitely — approved, built,
green, and never landing, with no artefact anywhere saying *this fell out*.

**And then the repair erases the record of it.** Rerunning is the correct
response to a timeout; it is what both watchers do and what any lane would do by
hand. Each rerun flips one row from `cancelled` to `success`. So:

- the more conscientiously lanes rerun timeouts, the cleaner the history looks;
- the cleaner the history looks, the stronger the case that the limit is fine;
- the stronger that case, the less likely anybody raises the limit;
- and the limit keeps cancelling queue builds.

**The usual cure in this folder does not reach it.** Most entries are fixed by
making a label carry its scope, or by giving a check a control. Here the check
has no scope problem and a control would not help: `gh run list` is answering
exactly the question it is asked — *what is this run's conclusion* — and
answering it correctly. The question that needed asking was *what was this
run's conclusion **the first time***, and there is no reason to ask it unless
you already know.

This is the inverse of
[`a-negative-result-proves-nothing-about-the-search.md`](a-negative-result-proves-nothing-about-the-search.md).
There, an empty answer hid a query that could never have run. Here a **full,
correct, specific** answer hides a fact that was overwritten. Nothing is empty,
nothing is broken, and nothing is lying.

## What this adds to the cancelled-check entry, and what it corrects in it

[`a-cancelled-check-is-not-a-failing-one.md`](a-cancelled-check-is-not-a-failing-one.md)
already holds the near half of this: `gh pr checks` collapses everything that is
not success into `fail`, a cancelled run has no failing step, and the cure is to
ask for `conclusion` rather than the summary. All of that stands and this entry
assumes it.

Three things are new here, and the third **corrects that entry's cure**:

1. **A rerun overwrites the conclusion.** That entry's cure — ask for
   `conclusion` — is sound at the moment of the cancellation and gives the wrong
   answer *afterwards*, because by then the conclusion is the latest attempt's.
2. **A timeout on a queue build drops the pull request**, which is a different
   consequence from any other cancellation: the change stops being in line while
   staying green and approved.
3. That entry says a cancellation **is fixed by pushing anything**, *because a
   cancelled run never becomes success on its own.* **That is true for a branch
   gate and false for a queue build.** Pushing to the branch does not put the
   pull request back in the queue — it makes a new head that must be gated and
   enqueued again. The queue build's cancellation is fixed by **re-enqueuing**,
   and a lane that pushes instead has spent another full gate to arrive back
   where it was. It also mixes the two cases the section above separates:
   *superseded by a push* wants nothing at all, where *timed out* wants a rerun,
   and only the job's duration tells them apart.

## The cure

**Count timeouts from the job, and from attempt 1.** Any claim about how often
the limit bites has to walk `/actions/runs/<id>/attempts/<n>` rather than read
conclusions, and compute duration from the **job's** `started_at →
completed_at`. A claim built on a listing is a claim about the last attempt of
each run, which is not what anybody means by it.

**Treat a cancelled queue build as a dropped pull request, not a failed one.**
The response is to re-enqueue, and something must notice: a green head beside an
absent queue entry is the signature, and it is exactly the state `CLAUDE.md`
warns reads as reassuring — *a head status must not read as cleared to land*.

**Distinguish `cancelled` by duration before acting on it.** At the limit, it is
a timeout and wants a rerun. Well short of it, it is a superseded push and wants
nothing at all.

**And write the timeout down where a rerun cannot reach it.** A rerun is not
reversible and the listing is not a record. Whoever reruns a timed-out gate
should say so in the place the next person will look — the pull request, or the
report for that change — because the run history will shortly claim it never
happened. *This entry is that place for the three instances above, which is the
only reason they are still countable.*

## Related

- [`two-true-sentences-whose-conjunction-was-false.md`](two-true-sentences-whose-conjunction-was-false.md)
  — a scope that lived in the script and in neither sentence. Same family: the
  record was accurate and the reading it invited was not.
- [`a-negative-result-proves-nothing-about-the-search.md`](a-negative-result-proves-nothing-about-the-search.md)
  — its inverse, as above.
- [`a-cancelled-check-is-not-a-failing-one.md`](a-cancelled-check-is-not-a-failing-one.md)
  — the near half of this, and the entry whose cure the section above corrects
  for queue builds.
- [`a-rule-i-remember-is-not-a-rule-the-repository-has.md`](a-rule-i-remember-is-not-a-rule-the-repository-has.md)
  — walked into while writing this one. The first draft cited two entries by the
  names they have in my own notes, and **neither existed under those names
  here.** The link check caught it; nothing else would have.
- *A red verdict needs a baseline too* — the desktop lane's entry from the same
  evening: a failing run is as capable of measuring nothing as a passing one,
  and this is the third case, where a *repaired* run stops having measured at
  all. **Not linked, because as of 2026-10-22 it is on an unlanded branch**
  (#932) and a link from `main` would dangle. Whoever lands it should add the
  link both ways.
