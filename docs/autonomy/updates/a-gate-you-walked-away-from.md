# A gate you walked away from, and an hour spent waiting for nothing

- Date: 2026-09-13
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **three finished tasks went unhanded-over the same way in one day.**

Three workers on 2026-09-12 did their task completely and correctly, started
their crates' test suites **in the background**, wrote some version of

> alo-choosing and alo-agentd are still running, and the handoff must wait on
> them, so I'll wait for the background task's completion notice

and ended before the notice came. Finished code, finished tests, finished
report, the plan marked — and no handoff, which is the one artefact the
supervisor acts on. Every one was recovered by hand.

Two things were wrong, one on each side.

## A gate you started and walked away from is a gate you did not run

The prompt tells a worker not to run the whole workspace suite, and that is
right: it takes the better part of an hour here and killed two tasks at the
ninety-minute deadline. What it never said is that starting your own crates'
suites and then waiting on a notification is not running them. The worker obeys
the letter — it did not run the workspace — and loses the task anyway.

So the prompt now says **stay with your gates**: run them in the foreground,
wait for the exit code yourself, and if they are slow run fewer crates rather
than running them asynchronously. A test pins the phrase, beside the one that
pins *do not write the handoff until the gates pass*.

## An hour is a person's wait, not an exited worker's

`WAITING_AT_MOST` is an hour, and the reason is good: a person writing the
handoff between iterations needs an afternoon's grace. But the loop applies it
even when **it launched the worker itself** and that worker has already exited.
At that point the handoff is either on disk or it is never coming, and every
further second is spent on an answer that cannot change. The journal shows it
exactly: `1789242171` worker finished, `1789245771` gave up — 3600 seconds. Then
again for the next task. Two hours, twice, waiting for a program that had
already gone.

There are two bounds now. A person still gets the hour. A worker that has
exited gets **ten seconds** — one look, a pause, another look, because a
worker's last act may be the write itself and a file being created is not
instantaneous.

## What this does not change

The supervisor still reads the tree and the handoff rather than a worker's
account of itself, still runs every gate, and still publishes only what passes.
Nothing here makes a missing handoff acceptable; it makes an already-missing
one cheap to discover.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a worker stays with its gates; the loop stops waiting on
one that has exited.
