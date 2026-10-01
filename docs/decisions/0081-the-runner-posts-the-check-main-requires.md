# ADR 0081 — The runner posts the check `main` requires

**Status:** proposed, 2026-10-01. It reopens the 2026-09-28 decision recorded in
`.github/workflows/gate.yml` on the one new fact that decision lacked: **the five
tests a hosted runner could not run now pass there.** Nothing here weakens a gate,
removes one, or changes what the nine gates are.
**Date:** 2026-10-01
**Context:** `.github/workflows/gate.yml`;
`.github/the-tests-a-hosted-runner-cannot-run.txt`, which is now empty;
`docs/autonomy/SHARED_MAIN.md`, whose gate-inheritance and turn-taking rules exist
only because of what this changes;
[ADR 0080](0080-a-signal-that-cannot-be-wrong-tells-you-nothing.md), which supplies
the standard the runner's deciding step is held to here.

## The question in one line

**Three machines share one queue and two of them are idle by construction. Why — and
is the reason still true?**

## The answer

The reason is one status name.

`main`'s protection requires `alo/nine-gates`. Nothing produces it but a lane: a
person runs the gates on their own machine and types the status with `curl`, twice
per change — once for the pull request's head and once for the commit the merge
queue builds, which is a different tree. `gate.yml` already runs the same nine gates
on a hosted runner, on every pull request **and every queue commit**, and posts
`alo/gates-on-a-runner` — a status nothing requires.

So the queue absorbs one change per hand-run gate, and a lane that does not hold the
turn cannot usefully gate at all: under tree-identity inheritance the condition is a
still `main`, and every landing invalidates every other lane's gated tree. The
machines are not merely unhelpful while waiting. They are **unusable**.

That was the right decision on 2026-09-28, for a reason that has since stopped being
true. Five tests of the suite failed on a hosted runner — four in
`alo-asking::an_office_that_cannot_connect_still_has_working_ai` and one in
`alo-nearby::asked_and_answered_over_ipv6`. All five re-run their own test binary
inside `unshare --map-root-user --net`, and all five exited **126**: found, but could
not be executed. A runner that cannot run the suite cannot honestly claim the nine
gates, so a person had to.

**The cause was one bit.** `/home/runner` is `drwxr-x---` owned by uid 1001.
`--map-root-user` maps uid 0 and nothing else, so a process in that namespace holds
no capability over that directory, `CAP_DAC_OVERRIDE` stops covering for a mode that
would otherwise deny, and `0750` grants others nothing — **traversal was denied to
root itself.** The job grants `o+x` before the gates run and the five execute.
`.github/the-tests-a-hosted-runner-cannot-run.txt` reached zero by being satisfied,
not by being waived: nothing is skipped, `#[ignore]`d or weakened, and the run checks
the list's emptiness on every pass rather than trusting it.

## The decision

**Branch protection requires the check the runner posts, and no required check is
produced by hand.**

The lane's own gate run is kept and is not weakened — it is how a lane learns its
change is sound before asking anything of a queue. What stops is a *person* being
the only thing that can satisfy protection.

## What was measured, and what is inferred

Measured, on the queue commit of the change that landed this morning:

    run 36830997454   head 7c236e0e (queue commit for #352)
    verdict           9 of 9 gates passed in 775.055s, exited 0
    FAIL lines        none
    runner            GitHub-hosted ubuntu-24.04

Measured, over the twelve most recent runs of `gate.yml`: twelve successes, seven of
them on `merge_group` — the queue's own branch, which is the half of the contract a
person has been typing.

Measured, by reading the deciding step rather than trusting it: a failed gate that is
**not** the tests gate fails the job; a tests gate that failed with no readable `FAIL`
line fails the job, because an unparseable failure is this step unable to do its work
rather than a suite that is fine; a failure not on the declared list fails the job;
and a declared test that did **not** fail is reported loudly, because an excuse
nothing removes becomes a list of things nobody may break. That last branch had a gap
of its own — when every gate passed, the step returned before comparing, so the
liveness check was bypassed by the very success that makes a line stale — and it is
fixed. This is ADR 0080's standard met in the one place that would otherwise decide
whether `main` is protected.

Inferred, and flagged as inference: that the runner stays green. It has been green on
every run since the permission fix, and twelve runs is not a guarantee. The mechanism
for being wrong about this is named below.

## What it costs

**A gate on the runner takes 775s against 267s on this machine.** Slower per change,
and that is the wrong unit: three lanes gating at once beats one lane gating fast
while two wait. The throughput that matters is changes landed per hour, and today it
is bounded by turns rather than by compute.

**It is a flag day on protected `main`.** Changing a required check while three lanes
are landing is how a queue stops for a reason nobody can see. It is done with the
queue empty and all three lanes told, not between two landings.

**The runner becomes the thing that can be wrong.** Today a wrong green requires a
person to mistype a status; afterwards it requires the workflow to conclude success
over a red suite. That is a real transfer of risk and the honest reason to accept it
is that the workflow's deciding step is now tested in all eight of its decisions and
a person's `curl` is tested in none. The off-switch is the same switch: restore
`alo/nine-gates` as required and lanes resume typing it.

## What it retires, and this is the point

Every one of these exists to make a hand-typed status trustworthy:

- gate-script digest recording and comparison, across three machines' untracked tools
- tree-identity inheritance, and the serialisation it forces
- the four-sha guard before enqueueing
- gating the queue commit separately from the head
- the turn, the hold, and the message that releases it

**On 2026-09-30 and 2026-10-01 the three lanes found eight faults in that tooling, and
not one was in the product.** Each lane independently discovered its own gater did not
enforce the rule it printed: one printed a digest and never compared it; one compared
a tree against a sha the operator typed; one attested a commit while having gated the
working tree. At least two of the eight were caught by luck, so the count of what was
found is not evidence about what remains. The tooling is also untracked on all three
machines, which makes drift detectable but not recoverable.

A mechanism whose faults outnumber its subject's is the thing to remove, not to
harden further.

## The alternative rejected

**Inheritance on a gate's observed inputs** — recording which files a run actually
opened, so a verdict survives a change that touched none of them, and lanes could
gate in parallel without a still `main`. It is the better mechanism in principle and
it was the direction two lanes were converging on.

Rejected because it answers a question that stops being asked. It also has an
unmeasured remedy's problem: `alo-reconciling`'s tests read files at run time by a
path the ledger's **own content** supplies, so the input set is not statically
computable, and a manifest-based version would inherit a green over a docs change
that genuinely breaks tests. Only the set a run actually opened would do, and nobody
has built that. Choosing between a measured cost and an unmeasured remedy is how the
cost gets paid twice.

## How we notice if this is wrong

The failure to watch for is a queue commit merging on a green that was not earned.
The signal is the lane's own gate run, which is kept: a lane that gates a tree
already in `main` and finds it red has found exactly this. That is the one check
nobody else can perform, and it is the reason the lane's gate is not retired along
with the status it used to produce.

Roadmap: v0.01
