# A cancelled check is not a failing one

**Concluded:** *the gate failed, so the change is wrong* — read off `gh pr
checks`, which is how all three lanes find out whether their work passed.

**What was true.** The same check, asked two ways, in the same minute:

```
$ gh pr checks 532
gate    fail    15m4s    https://github.com/…/runs/37366232907/job/…

$ gh pr view 532 --json statusCheckRollup
gate: status=COMPLETED  conclusion=CANCELLED
```

**`fail` and `CANCELLED` are different states and only one of them is about
the change.** A failing run has a step that failed and a log that names it. A
cancelled run has **no failing step at all** — it was stopped, by a queue
superseding it, a timeout, a runner going away. Nothing is wrong with the
commit.

And the two behave differently afterwards, which is the part that costs time:

- A **failure** is fixed by changing the code and pushing.
- A **cancellation** is fixed by pushing *anything*, because a cancelled run
  never becomes success on its own. It is not waiting for a fix; it is waiting
  for a run.

So a reader who sees `fail` goes looking for a defect, finds the log, finds no
failing step in it, and concludes the log is truncated or the failure is
flaky — none of which is true.

## The mechanism

**One word standing for two states, in the layer a person actually reads.**
The distinction exists and is reachable; `statusCheckRollup` carries
`conclusion` and names it exactly. The summary view collapses *everything that
is not success* into `fail`, which is right for a dashboard and wrong for a
diagnosis, and nothing in its output says a collapse happened.

That is this directory's standing subject in a tool rather than in our own
code: **a check whose answer is narrower than the question it is asked.** We
ask *did my change pass*; it answers *is this green*.

**It is worse than a plain ambiguity because it is asymmetric.** `fail` on a
real failure sends you to the log and the log helps. `fail` on a cancellation
sends you to the log and the log is **silent in exactly the way that looks like
your own mistake** — no failing step, no error, nothing to fix. The wrong
answer is the one that makes you doubt your reading rather than the tool.

## Measured on one day

Both readings happened within hours, on one pull request, and only the first
was real:

| what `gh pr checks` said | what it was | what it needed |
|---|---|---|
| `fail` | clippy: missing doc on a function | a code change |
| `fail` | `CANCELLED`, no failing step | a push |
| `fail` | a citation naming a file that does not exist | a code change |

**Two of three were genuine**, which is why the label is believable. An
instrument that lied every time would have been caught in a morning.

And the cancellation was first diagnosed by **another lane**, who measured
`conclusion` rather than reading the summary, and whose note said the thing
plainly: *a cancelled check never becomes success on its own, so it is waiting
on a run that will never finish.*

## The cure

**Ask for the conclusion, not the summary**, whenever the answer will decide
what you do next:

```sh
gh pr view <n> --json statusCheckRollup \
  --jq '.statusCheckRollup[]? | "\(.name): \(.status) \(.conclusion)"'
```

`gh pr checks` is fine for *is it green yet*. It is not fine for *why is it
not*, and those are different questions asked of the same command.

**And before fixing anything, confirm there is a failing step.** A gate log
with no `error`, no failing step and no undeclared test is not a log you are
reading wrong. It is a run that did not finish, and no amount of reading will
find the defect, because there is not one.
