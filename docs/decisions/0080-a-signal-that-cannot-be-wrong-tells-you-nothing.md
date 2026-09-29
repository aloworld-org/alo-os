# ADR 0080 — A signal that cannot be wrong tells you nothing

**Status:** proposed, 2026-09-29. Written from five instances found across three lanes
in one evening. **Nothing here weakens a gate or adds one**; it names a way a gate,
a document or a message can already be dead while reading as though it is working, and
what to write instead.
**Date:** 2026-09-29
**Context:** `CLAUDE.md`'s gate — *the refusal path tested as carefully as the happy
path*, *code tested only when it works has not been tested*, *the capability
guarantees are tests, not prose*; `.github/workflows/gate.yml`;
`.github/the-tests-a-hosted-runner-cannot-run.txt`;
`crates/alo-reconciling/src/the_gate.rs`;
`crates/alo-reconciling/tests/the_gate_is_held_to_itself.rs`;
[ADR 0077](0077-a-machine-signs-its-own-account-of-what-left-it.md), which supplies
both the last instance and the place where the third sentence was applied before a
fault could happen.

## The question in one line

**`CLAUDE.md` already says the refusal path is tested as carefully as the happy one.
Why did five things in this repository still report a state they could not tell from its
opposite — and what does a writer do differently?**

## The answer, in three sentences

    A check that cannot distinguish its two outcomes has one outcome,
    and the colour it shows is decoration.

    A refusal path that says nothing is indistinguishable from a success.

    A signal that fires when nothing is wrong teaches people to dismiss it,
    and then the one case that matters is dismissed with the rest.

They are deliberately **not attributed**. None of the three was produced by one lane
alone: each is the second lane naming what the first had already walked into, and a
name on a rule only gives a later reader a way to weigh it by its author. The rule is
worth what its instances are worth, and those are attributed below.

## It is not about CI, and the last instance is how we know

The first four are instruments — a workflow, a status, a repository gate, a probe. The
last is a **document**: nothing running, nothing to be green or red, no check
involved. If the three sentences hold there too — and they do, word for word — then
this is not a rule about checks. **It is a rule about anything that carries a state it
cannot verify**, and CI is merely where we happened to meet it five times in one day.

The list below is one instance at each distance from a person:

| | what it was | how far from a person |
|---|---|---|
| 1 | a workflow's exemption | a machine talking to itself |
| 2 | a status nothing could post | a machine talking to a lane |
| 3 | a repository gate | a crate talking about the repository |
| 4 | a probe that nearly measured nothing | a script talking to whoever reads the log |
| 5 | a document's own header | a record talking about itself |

A reader who thinks *this is a CI problem* is answered by the last row without an
argument: nothing was running, nothing could be green or red, and all three sentences
hold there word for word.

**The table shows distances, not frequencies.** Five instances in one day across three
lanes is a strong sample for a Tuesday and a weak one for a principle, and a table
makes anything look tidier than it was. It earns its place because of the objection it
answers, not because five points make a shape.

**And the first draft of this table had the fault this record is about.** It listed six
rows against six instances and the correspondence was not real: one row was an instance
numbered differently, one row named something that was not an instance at all, and the
probe was missing. Every row read as though it were backed by an entry below it, and a
reader had no way to tell the rows that were from the row that was not. Left in, it
would have been the seventh instance rather than an illustration of the other five.

**And the direction of travel is the reason to keep the record.** A rule found in CI
that turns out to govern an interface has been tested somewhere cheap before it is
applied somewhere expensive, which is the opposite of how interface rules usually
arrive. This one arrived as a workflow that had been green-blind for 102 runs, a status
nobody could post, and a probe that nearly measured nothing — and only then turned out
to say something about what a person should be shown when they switch a machine on.

## The five

**1. A gate's name standing in for its reason.** `gate.yml` asked *which gate failed*
and exempted `the workspace's tests`, because five tests cannot execute inside a
network namespace on a hosted runner. That is also the answer when a change under test
breaks a test, so the run was silent about a real regression **by construction** — in
the file whose own header argues against this shape twice. *Found 2026-09-29; fixed the
same day.*

**2. A status nothing could ever post.** In the same file, `post alo/gates-on-a-runner`
ran `if: success()`, and the step that decided success was the one that always failed.
**102 runs, zero greens**, and the status the workflow exists to produce had never once
existed. Four changes were landed building on it. A red check on a pull request a lane
was going to attest by hand anyway is indistinguishable from a red already accounted
for — the colour was decoration in the strictest sense, because nothing read it.
*Found and fixed 2026-09-29.*

**3. A finder held only to its own fixture.**
`alo_reconciling::the_gate::tiers_that_disagree` exists to catch a promise sitting at
one tier in `docs/features.md` and another in `ROADMAP.md`. Its test asserts only that
the finder still finds *Camera and microphone* — a liveness check on the instrument
standing in for a gate on the repository. `docs/features.md` said `[v0.5] Printing` for
three days after `ROADMAP.md` moved it to v1, in front of a check that was looking
straight at it. *Found 2026-09-29; open, and claimed by the lane that owns the crate.*

**4. The one that did not become a bug, and is the most useful.** `gate.yml`'s
namespace probe read `target/debug/deps`, which never exists — the supervisor builds
each checkout under its own directory. So it measured nothing on the very run that
proved the job could be green.

It cost **one run** rather than a wrong conclusion, because its absent-subject path
*spoke*: *no ipv6 test binary under target/debug/deps… nothing is measured here and
nothing is claimed.* Written as a bare `exit 0`, or left to abort under `-e` — which it
would have, since a glob matching nothing exits non-zero — the run would have been
green, the probe silent, and the question still open while somebody believed it was
being measured.

**A near-miss belongs in this record more than a corpse does**, because it shows the
remedy working rather than the fault landing. *2026-09-29.*

**5. A record still asking a question it had answered.**
[ADR 0077](0077-a-machine-signs-its-own-account-of-what-left-it.md) carried
*one thing is still asked of the owner* in its status line. The answer arrived and was
written into the bottom of the file. **Had the status line not been corrected in the
same change, the top of the document would have gone on asking a question settled three
branches earlier — and nothing would ever have said so.** A record with an answered
question and a record with an open one read identically from the top. *Found and fixed
2026-09-29, in the change that answered it.*

## A sixth was withdrawn, and why is part of the argument

A sixth instance was drafted: a lane's gate script that printed `CLIPPY CLEAN`
underneath the clippy error it had just printed, because the success echo was chained
off a `grep` whose zero exit meant *I found the failure*.

**It was cut because its evidence no longer exists.** The lane went to read the script
rather than confirm from memory, and found that it had been rewritten at the time to
use clippy's own exit code. The scripts live outside this repository, so there is no
history of it: the text cannot be read by anybody now. What remains is one lane's
recollection, which that lane reports with confidence and no artifact behind it.

**A record whose whole argument is that a claim must be distinguishable from its
absence cannot carry an instance that is not.** Keeping it would have bought a rounder
number at the price of the thing being argued for, which is the trade this record warns
against. So the set is five verifiable instances rather than six with an asterisk, and
the sixth is described here so that its removal is visible rather than silent — a
withdrawn instance deleted without trace would be the same fault again.

The pattern it showed is not lost. It is the general form in remedy 1 below: a `grep`
that found something is *output exists*, not *the output means what I am about to say
it means*.

## What to write instead

1. **Make the success condition depend on the claim, not on the presence of output.**
   This is the single mechanism behind all five. A `grep` that found something, a gate
   that has a name, a finder that still finds its fixture, a probe that ran, a status
   line that exists — every one of those is *output exists*, and none is *the output
   means what I am about to say it means*. If the sentence is *these tests failed and
   all of them are ones this runner cannot run*, the check compares the failing set
   against a declared set.

2. **Give every refusal path words.** A path that exits 0 in silence cannot be told
   from success. The probe's refusal names what it searched, so the next person adds a
   place rather than re-deriving the question.

3. **A declared exception expires.** A list of tests a runner cannot run is an excuse,
   and an excuse nothing is ever removed from becomes a list of things nobody may
   break. `.github/the-tests-a-hosted-runner-cannot-run.txt` is therefore checked in
   both directions: every failure must be on it, **and every line on it must still
   fail**. Two tests were on the informal version of that list and had quietly started
   passing.

4. **Hold the instrument and the repository separately.** A test asserting that a
   finder still finds its fixture is worth having — it is how you learn the finder
   broke — but it is not the gate. Instance 3 needs both, and only one exists.

5. **A harness reads its subject out of the thing under test.** A pasted copy passes
   for ever while the original drifts. `gate.yml`'s deciding step is extracted from the
   file with `awk` and refuses outright if the step was renamed. This is instance 3's
   fault applied to harnesses.

6. **Distinguish *asked and there is none* from *the read failed*.** A status reader
   returned `no` for a `Not Found` body because `Array.isArray(a) && a.some(…)`
   collapses a non-array to false. Both answers refused to act, so nothing *behaved*
   differently — the fault was entirely in what got **said**, since `no` asserts
   something a failed read never learned.

7. **Do not fire when nothing is wrong.** A machine with no security chip still
   produces a true statement about what left it, signed as well as that machine can
   sign; what differs is what the statement is *worth* to somebody who did not watch it
   being made. That is a statement of worth, not an alarm, and dressing it as an alarm
   would teach people to dismiss it. Sentence three, from the interface side, and the
   same reasoning as [ADR 0009](0009-a-good-computer-without-the-agent.md)'s refusal to
   nag.

## What a working one looks like while it is working

Worth stating because it answers the obvious objection. On 2026-09-29 a branch was
gated four times and refused three, each on *main is unmoved: no*, and three ten-minute
runs were thrown away. That is not the check failing three times; it is the check
working three times, and refusing to let an attestation stand for a tree that was no
longer the one merging.

**The near-miss and the repeated refusal are the two shapes of a working check, and
neither looks like success while it is happening.** So cost is not evidence that a
check is wrong — and a check that has never been inconvenient is one to go and read.

## What this record does not say

**It does not add a gate.** What a gate *is* lives in one place, and a lane growing the
gate out of an ADR would be doing the thing this repository has an integration owner to
prevent.

**It does not say a liveness check is wrong.** Instance 3's test is a good test badly
scoped. The error is not writing it; it is stopping there and letting its green stand
for the repository.

**And it does not claim the five are all fixed.** Three are. One is the near-miss and
needed none. **Instance 3 is open** and belongs to another lane.

**One thing in it is deliberately not tidied.** The status reader from remedy 6 has had
its *yes* and *no* answers exercised on real pull requests; its `unknown` answer has
only ever run in a harness, never against a genuine failed read. Two lanes hold the
same instrument in the same state, and both of us nearly let *it has all three answers*
stand for *all three have been seen* — which is this record's own subject, one level in.

## What is asked of the owner

Nothing. This changes no promise, tier or law. The three sentences at the top are the
whole of it, and the five instances are why they are worth believing.
