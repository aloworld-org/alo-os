# ADR 0080 — A signal that cannot be wrong tells you nothing

**Status:** proposed, 2026-09-29. Written from ten instances found across three lanes
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
both instance 5 and the place where the third sentence was applied before a
fault could happen.

## The question in one line

**`CLAUDE.md` already says the refusal path is tested as carefully as the happy one.
Why did ten things in and around this repository report a state they could not tell
from its opposite — and what does a writer do differently?**

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

## It is not about CI, and four of the instances are how we know

Most are instruments — a workflow, a status, a repository gate, a probe, a count. **Four
are not.** One is a **document**. One is **this document**. One has **no code in it at
all**: a gate's duration estimated from the part of it anybody had ever watched finish.
And one is **a lane's description of the shape of its own work in flight**, which every
tool it touched would have gone on agreeing with, because a stacked branch gates, tests
and lands exactly like an independent one.

Nothing running, nothing to be green or red, no check involved, and the three sentences
hold in all four word for word.

So this is not a rule about checks. **It is a rule about anything that carries a state
it cannot verify**, and CI is merely where we happened to meet it most often in one day.

They sit at different distances from a person, which is the list below:

| | what it was | how far from a person |
|---|---|---|
| 1 | a workflow's exemption | a machine talking to itself |
| 2 | a status nothing could post | a machine talking to a lane |
| 3 | a test asserting one of six findings | a crate talking about the repository |
| 4 | a probe that nearly measured nothing | a script talking to whoever reads the log |
| 5 | a document's own header | a record talking about itself |
| 6 | this record's own instance 3 | a record talking about another lane's work |
| 7 | a count taken from a key prefix | a crate counting what it can say |
| 8 | a resize band where a client had drawn | **under somebody's finger** |
| 9 | a gate's duration, estimated from the part ever watched | **no code at all** |
| 10 | three branches described as independent | a lane describing its own work |

A reader who thinks *this is a CI problem* is answered by the last three rows without an
argument. Row 8 is a region that moves under a person's hand. **Row 9 has no code in it
whatever**: the instrument was a person's memory of a measurement, and the fault survived
the complete removal of tooling. **Row 10 is the only one nobody went looking for** — it
surfaced from a line printed beside an unrelated answer, purely so the output would read
better. And row 6 is this document.

**The table shows distances, not frequencies.** Ten instances in one day across three
lanes is a strong sample for a Tuesday and a weak one for a principle, and a table
makes anything look tidier than it was. It earns its place because of the objection it
answers, not because ten points make a shape.

**And the first draft of this table had the fault this record is about.** It listed six
rows against six instances and the correspondence was not real: one row was an instance
numbered differently, one row named something that was not an instance at all, and the
probe was missing. Every row read as though it were backed by an entry below it, and a
reader had no way to tell the rows that were from the row that was not. Left in, it
would have been a further instance rather than an illustration of the others.

**And the direction of travel is the reason to keep the record.** A rule found in CI
that turns out to govern an interface has been tested somewhere cheap before it is
applied somewhere expensive, which is the opposite of how interface rules usually
arrive. This one arrived as a workflow that had been green-blind for 102 runs, a status
nobody could post, and a probe that nearly measured nothing — and only then turned out
to say something about what a person should be shown when they switch a machine on.

## The first five

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

**3. A test that asserted one of its function's six findings.**
`alo_reconciling::the_gate::tiers_that_disagree` catches a promise sitting at one tier
in `docs/features.md` and another in `ROADMAP.md`. Run against the real documents it
reports **six findings across five promises**. Its test asserts **one** — that the
finder still finds *Camera and microphone*.

So four promises stood unmentioned in a repository that runs this on every gate, and a
reader of that pass would conclude the check had one finding and it was known about.
**The pass could not distinguish *one known finding* from *six, one of which is
known*.** Before the printing tier was corrected there was a seventh, and it was in the
function's output the whole time with nothing to read it.

*Found 2026-09-29; being corrected by the lane that owns the crate, which measured the
six rather than reasoning about them and deleted the throwaway that printed them.*

**How that lane came to read it is the most useful thing in this record.** They had
grepped for the function, read the test's assertion and the loop above it — lines 190 to
215, twenty-five lines — created the branch, written the throwaway, and measured the
six. Then they attempted the edit, and **a tool refused it**: *File has not been read
yet. Read it first before writing to it.* Satisfying that refusal meant reading from
line 150, and the docstring was there.

So the correction was not produced by care, a checklist, a reviewer or a second thought.
**It was produced by a precondition that would not let a file be written over unread**,
enforcing something it was not built to enforce. Every other instance in this record
ends with somebody deciding to go and look; this one ends with the looking being
compulsory. Had that refusal not fired, the wrong description would have stayed and this
document would have had no reason to be revisited. **The correction was one refusal away
from not happening.**

*And the same refusal fired on the author of this paragraph, on this file, while writing
it.* The edit adding these words was rejected — *File has been modified since read* —
and reading it again was the condition of continuing. That is not a coincidence worth
smiling at: it is the mechanism working twice in ten minutes on two different people, in
the one case by construction rather than by anybody being careful.

**This entry is a correction, and the first version of it was wrong in this record's
own way.** It said the check *could not fail* and that nobody had noticed. Both are
false. The test can fail — if the finder stops finding, the assertion fires — and the
person who wrote it had noticed at length: the docstring above it says in its own words
that **neither check can gate yet**, gives two matcher settings measured against the
real documents with their numbers, refuses to tune one until the answer looks right
because *that would be fitting the check to the documents*, names the actual remedy —
for each box in the gate to name the promises it answers, making the comparison exact
with no heuristic — and points at the report written for whoever owns `ROADMAP.md`.

**Three of the six are probably not faults at all**, which is its own finding: two
`[v0.5]` promises and one `[v1]` promise are answered inside **v0.01's** gate, which is
work delivered *earlier* than promised. The check compares where a promise is answered
against where it is promised, so **a promise kept ahead of its tier reports identically
to one at the wrong tier.** That limit was not known before this correction.

## How this record got instance 3 wrong, which is instance 6

The paragraph above replaced one written from **a summary of that test rather than the
test**. This record's author opened the file at the assertion — lines 195 to 215,
twenty-one lines — and described the whole from them. The docstring explaining why it
deliberately does not gate begins forty-five lines above and was never read. The lane
that owns the crate had described the check from memory twice, and this record published
that description.

**That is the same act with less excuse.** The other lane never opened the file until a
tool made them; this author had it open and read upward from the assertion, stopping
exactly where the answer began.

**A record whose subject is claims that cannot be checked, published a claim its author
had not checked, about a file open in front of him.** It is the sharpest instance in
the set and it is this document's own.

### Crossing a lane boundary added authority without adding verification

What left one lane was *a description of a check by somebody who had not read it*. What
arrived at the other was *a report from the lane that owns the crate* — which is about
as good as testimony gets, and is why publishing it was reasonable. **Nobody added a
single piece of evidence in between.** The boundary did that by itself.

That is the transport for most of what is in this record: a claim has to travel from
wherever it was guessed to somewhere it is written down before it does any harm. *The
check cannot fail* travelled from a lane into this document. So did *the contract has
thirteen rules*, which went the other way between the same two lanes and was caught only
because somebody eventually counted — **two instances of one mechanism, in opposite
directions, in one evening.**

**And a boundary is not only between people.** `crates/alo-dock/src/windows.rs` pointed
at a design note that has never existed, and that design note points back at
`windows.rs` **by path, and resolves.** Two documents referring to each other with one
direction dead — and the dead direction reads exactly like the live one. **The half that
resolves is evidence to a reader that the pair was checked**, so a dead pointer beside a
working one is worse than a dead pointer alone: the working one launders it. Nothing was
lost in that case, and the substance is in the design note under *What is owed
elsewhere*; only the path is wrong.

It also completes a pattern the three lanes found in themselves within one evening, each
about their own work and each readable at the moment it was spoken about:

- a lane read a gate's **total** and its nine `PASS` lines about twenty times and never
  the breakdown, so nobody knew that `rustdoc, warnings denied` is **1637 s** — longer
  than the whole test suite — until somebody looked;
- a lane estimated a gate at 25 to 30 minutes from the part of it they had ever watched
  finish, and measured **49**;
- this record described a test from its last fifteen lines.

**In every case the thing itself was available and a summary of it was used instead.**
That is the general form the instances share, stated more precisely than *the presence
of output*:

> **A thing that answers from where something is declared to be, rather than from where
> it actually is.**

A key prefix standing in for the number of refusals a crate can make. A hit region
measured from window geometry while the client drew somewhere else. A probe reading
`target/debug/deps` while the build was under `alo-builds`. An estimate taken from the
observed part of a run. A description taken from the part of a file that was read.

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

## Four more, found the same night

Added 2026-09-29 after the first five had landed. They are kept separate rather than
folded in, so a reader can see that the set grew rather than arriving whole.

**7. A count taken from how keys are spelled.** A lane wrote a test that every sentence
its crate can say is declared, and every sentence declared can be said. It counted
refusals by key prefix — anything under `elsewhere.not-` — and answered **seven for
eight**, because one refusal reads to a person as part of the work's own story and its
sentence lives under `elsewhere.work.`. *A property of how keys happen to be spelled was
standing in for the number of things that can be refused*, and it would have gone on
answering wrong as variants were added under the prefix.

**And the general form of it is sharper than the instance.** That test's other half
compared twenty-two declared against twenty-two published and both were true while one
of the twenty-two could not be reached by anything. **No count can find an unreachable
declaration, because the counts agree.**

*That fault was inside a test written to catch this family — the second time in one
evening a new instrument contained the thing it was built to find. The first was a
status reader's `Array.isArray` collapse, inside the fix for another instance. The shape
is common enough that writing a check for it is itself an occasion for it.*

**8. A shell region measured from where a window is declared to be.** A lane's resize
band was computed from window geometry, and a client may draw outside that geometry;
where the application drew, the press is the application's.
`Server::resize_from_the_edge_under` now asks `pointer_target(at).is_some()` and returns
false *before* it consults a band. Two further findings came with it: a top resize band
lay entirely inside a frame's name band, so **whichever hit test ran first decided
silently**; and corners must be searched before edges, because a 24px corner overlaps
the last 24 of two 6px edges by construction and the corner is the more specific
gesture.

**This is the instance with a person's hand on the end**, and it is the one where being
wrong moves a target under somebody's finger. Its two tests are the part worth copying:
one **sweeps the whole band** rather than testing a point, because an overlap at one end
is how that bug returns; the other goes through `pointer_button` rather than calling the
compositor directly — which is the test that would have caught the original gap, because
**the bands had no production caller at all.** A drawn region nothing could reach looked
like part of the product and was not.

**9. A gate's duration, estimated from the part of it ever watched.** A lane reported
its gate at 25 to 30 minutes and measured **49**. Every previous failure of theirs had
stopped at the workspace's tests, so they had never watched past them and reported the
part they had observed as though it were the whole.

**There is no code in this instance at all.** The instrument was a person's memory of a
measurement; the fault survived the complete removal of tooling. It is the answer to the
objection that this record is about CI, and it is stronger for having happened to that
lane, about their own tooling, within an hour of their agreeing with the principle.

What the measurement found is worth keeping beside it: `rustdoc, warnings denied` is
**1637 s** of that run, longer than the entire test suite at 1189 s. `cargo doc
--workspace --no-deps` over 104 crates is the most expensive of the nine gates, and
three lanes had read *9 of 9* and a total perhaps fifty times between them without one
of them reading the parts. **A total is a number that cannot be wrong, and it told
nobody anything for a day.**

**10. Three branches described as independent, which were stacked.** A lane had said
*three branches, independent, waiting* — to two other lanes, twice. Each had been made
with `git checkout -B` from whatever branch happened to be current, so the second
carried the first's work and the third carried both. Three pull requests would each have
contained the previous one's changes, every diff would have misrepresented itself, and a
refusal of the first would have taken the other two with it.

**Nothing would ever have contradicted it.** A stacked branch gates clean, tests clean,
and lands correctly *if the order holds*. It is wrong only in what it **claims about
itself**, which is the one thing no gate looks at.

**And it is the only finding in this set that nobody went looking for.** It surfaced
from a script run to answer *will these still apply after a rebase* — the answer to which
was yes for all three — in a line printed beside the result only so the output would be
readable. Every other instance here was found by somebody deciding to check a specific
thing. This one was found because a tool said what it was touching when nobody had
asked.

That is a remedy in itself, and a cheap one: **make work state what it touches, in
passing, even when nothing is asking.** A line nobody requested is where the claim
nobody was auditing gets contradicted.

## One was withdrawn, and why is part of the argument

A further instance was drafted: a lane's gate script that printed `CLIPPY CLEAN`
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
against. So every instance kept is one that can be read by somebody else, and
the withdrawn one is described here so its removal is visible rather than silent — a
withdrawn instance deleted without trace would be the same fault again.

The pattern it showed is not lost. It is the general form in remedy 1 below: a `grep`
that found something is *output exists*, not *the output means what I am about to say
it means*.

## One more, whose remedy is unlike every other one here

Added 2026-09-30. It is kept in a section of its own rather than numbered with the
rest, because every other instance in this record is answered by **going and looking**,
and this one is not — looking is what produced it.

**A correction that feels like arrival.** One machine's gate was measured at 2919
seconds, of which rustdoc was 1637, and the lane published *rustdoc is the expensive
gate*. Corrected later the same night to *these are this machine's times*, when another
machine measured 32 seconds for the same gate. Corrected again to *these are this run's
times*, when the same machine measured 1637 and then 59 three hours apart. Corrected a
fourth time when a third lane found its own dominant cost was neither compilation nor
cache but **the suite's own execution** — 590 of 813 seconds spent running 8428 tests,
a floor no cache state can lower.

Three lanes, three different dominant costs, and each lane generalised its own.

**Each correction was more right than the one it replaced, and that is what made them
dangerous.** An error that irritates gets questioned again; an improvement that
satisfies does not. The lane that made the first three had published the general form
of the mistake — *these are this run's times* — and then, within the hour, gave two
other lanes a single number for a run it had not finished. **Knowing the general form
of a mistake does not stop you making it**, which is the finding, and it rules out the
obvious remedy: *understand the pattern* was already true.

**The remedy is to make the check refutable in advance.** The fourth correction exists
only because that lane wrote down *short* **before** running, and got 813 seconds. Had
they not, 813 would have read as *about twelve minutes, as expected*, and a mechanism
that had not predicted it would have survived by agreeing with it. A measurement taken
with no prior expectation cannot disagree with anything; it can only be interpreted.

> **A prediction made in advance is the only version of a measurement that can be
> wrong.**

Which is this record's own thesis applied to the act of correcting: a correction that
nothing can refute is not a finding, it is a restatement. So the question to ask of any
correction, including the ones in this record, is **what would still be true if this
correction were also wrong** — and the way to make that answerable is to say the number
you expect before you have it.

**Which was tested on this paragraph's author, immediately, and failed.** Gating the
change that adds this section, the lane predicted in advance — in writing, to the other
two — *about 400 seconds, dominated by about 270 of tests*, reasoning that no Rust had
changed and the cache was one commit back. **It took 167 seconds, of which 146 were
tests.** Wrong by a factor of 2.4, and wrong in the mechanism and not only the number:
its own previous documents-only run, on the same machine the same night, had measured
272 seconds of tests for the same suite. Same machine, same day, same gate, same kind
of change, and the test time alone moved 1.9×.

So there is now a **fifth** reading in the sequence, and it is not another theory of
which gate is expensive. It is that on one machine the dominant cost is not stable
across an hour, which means **no lane can predict its own gate, including the lane that
has just finished writing down why nobody can.** That is worth more than any of the
four mechanisms it replaces, and it is only available because the number was written
down before the run. Had it not been, 167 seconds would have been read as *fast, as
expected* and filed as agreement.

## And one whose cost of checking is unbounded

Added 2026-09-30. It is the only instance here that **nobody can disprove without
auditing the whole repository**, and that property is the whole of why it is worse than
the rest.

`docs/autonomy/v0-01-evidence.md` said a screen to see it on was owed **"and nothing
else in this repository"**, while a caller that computes `TheRoom` from real windows was
missing. Not a false claim about one thing. **A claim about everything.**

**Every other instance in this record has a something.** A `grep` that found something,
a gate with a name, a status line that exists, a path that does not land — a reader can
go and look at the thing named and win. *And nothing else is owed* names nothing, so
there is nothing to go and look at. Disproving it means enumerating the repository, and
nobody ever will.

> **The cost of checking is unbounded and the cost of asserting is one clause.**

That sentence is the Mac lane's, and it is the property rather than the complaint. It is
also the exact inverse of the rule this repository arrived at for attestations — *an
attestation should carry what a stranger needs to disprove it.* A completeness claim is
engineered, accidentally, to carry nothing. **And it reads as more rigorous than the
list it replaces**, which is why nobody challenges it: a list of three owed things
invites a reader to check three things, and *nothing else is owed* invites them to check
none.

**There is no remedy that makes it checkable, and that is the finding.** Every other
entry here ends in a mechanism. This one ends in an instruction:

> **Do not write it.** Say what is owed, and let the absence of a fourth line be the
> claim — a reader can count lines and cannot audit a repository.

*The same shape, one level down, is a refusal that leaves nothing behind. A refusal's
message is not the evidence that it refused; the state is. `bring_across` asserts the
branch that was here is identical afterwards and still has its own file, because a
refusal that had already fetched would be the fault wearing a refusal's words.*

## What to write instead

1. **Make the success condition depend on the claim, not on the presence of output.**
   This is the single mechanism behind all ten. A `grep` that found something, a gate
   that has a name, a finder that still finds its fixture, a probe that ran, a status
   line that exists — every one of those is *output exists*, and none is *the output
   means what I am about to say it means*. If the sentence is *these tests failed and
   all of them are ones this runner cannot run*, the check compares the failing set
   against a declared set.

2. **Where the fault is describing instead of reading, make the act require the
   reading.** Not a rule about reading — a precondition that cannot proceed unread.
   This is the only remedy here that does not depend on anybody's attention, and it is
   the one this record learned last: instance 3's correction happened because a tool
   refused to write over a file nobody had read, which is a guard nobody designed for
   this and which fired anyway. Every other remedy below asks somebody to be careful at
   the right moment. **This one removes the moment.**

   **And it is not an anecdote, because it has a count: three times in one evening,
   across two lanes, on three unrelated files** — the test whose docstring nobody had
   read, this record while the paragraph about the guard was being written, and a doc
   comment in `alo-dock` during the fix for the dead pointer below. Each time the guard
   was enforcing something it was not built to enforce, and each time the person it
   stopped was not being careless. **A precondition that fires on the careful is the
   only kind worth having**, because carelessness is not the failure mode — describing
   from what you already believe is.

3. **A peer's report is a reason to look, not a substitute for looking.** Stated as
   observed rather than proposed: this record's second error was stopped because its
   author ran a throwaway test against the real documents instead of taking the owning
   lane's count. That is not scepticism about a colleague — the report was honest and
   the lane was the right one to give it. It is a refusal to let a report *be* the
   evidence, and it is remedy 2 applied at the boundary between people rather than
   between a tool and a file. **A claim that crosses a boundary gains authority without
   gaining evidence**, so the reading has to happen on the far side.

4. **Give every refusal path words.** A path that exits 0 in silence cannot be told
   from success. The probe's refusal names what it searched, so the next person adds a
   place rather than re-deriving the question.

5. **A declared exception expires.** A list of tests a runner cannot run is an excuse,
   and an excuse nothing is ever removed from becomes a list of things nobody may
   break. `.github/the-tests-a-hosted-runner-cannot-run.txt` is therefore checked in
   both directions: every failure must be on it, **and every line on it must still
   fail**. Two tests were on the informal version of that list and had quietly started
   passing.

6. **Hold the instrument and the repository separately.** A test asserting that a
   finder still finds its fixture is worth having — it is how you learn the finder
   broke — but it is not the gate. Instance 3 needs both, and only one exists.

7. **A harness reads its subject out of the thing under test.** A pasted copy passes
   for ever while the original drifts. `gate.yml`'s deciding step is extracted from the
   file with `awk` and refuses outright if the step was renamed. This is instance 3's
   fault applied to harnesses.

8. **Distinguish *asked and there is none* from *the read failed*.** A status reader
   returned `no` for a `Not Found` body because `Array.isArray(a) && a.some(…)`
   collapses a non-array to false. Both answers refused to act, so nothing *behaved*
   differently — the fault was entirely in what got **said**, since `no` asserts
   something a failed read never learned.

9. **Do not fire when nothing is wrong.** A machine with no security chip still
   produces a true statement about what left it, signed as well as that machine can
   sign; what differs is what the statement is *worth* to somebody who did not watch it
   being made. That is a statement of worth, not an alarm, and dressing it as an alarm
   would teach people to dismiss it. Sentence three, from the interface side, and the
   same reasoning as [ADR 0009](0009-a-good-computer-without-the-agent.md)'s refusal to
   nag.

## Five instruments were written to catch this, and all five contained it

The only observation here about **producing** a check rather than about a check that
already exists, and it has a number: **five instruments written to catch this family,
six first-run faults, none surviving to a second run.** Four were written in one
evening; the fifth the following night, by a lane that had read this section first and
contained the family twice anyway.

- A status reader written to tell *asked, and there is none* from *the read failed*
  collapsed a non-array body to `no`, because `Array.isArray(a) && a.some(…)` is false
  for both. It asserted something a failed read had never learned.
- A test written to check that every sentence a crate can say is declared **counted
  refusals by key prefix** and answered seven for eight, because one refusal's sentence
  lives under a different prefix by design.
- A harness written to confirm the one-bit fix for the namespace exec **printed its
  report and its return value on the same channel**, so the caller captured the
  commentary as the reading and announced *NOT CONFIRMED* about a result it had just
  produced correctly.
- A check written to ask whether every `docs/design/…` pointer lands — the question
  `alo-citing` asks for decisions, one directory away — found one that does not, and
  **reported that the thing it recorded was therefore *recorded nowhere*.** It had no
  evidence for that: it measures whether a path exists, not whether the content exists
  elsewhere. On the single pointer it found, the claim was false — the substance is in
  `docs/design/the-alo-dock.md` and only the path is wrong. **A checker reporting more
  than it measured**, written to find checkers reporting more than they measured.
- A check written to hold every path this repository's prose names to landing — the
  question `alo-citing` asks for decisions and for reports, one family over — reported
  **correct writing as a fault, twice, on its first two runs.** It read
  `src/{carrying_out,refusing,words}.rs` as one filename, and then
  `src/sign_in_*.rs` as another: two forms this repository uses on purpose to name a
  family of files, neither of which is a claim that one file exists. The same check
  also carried a third version of the fault that never ran, because **its own unit test
  caught it**: it asked whether a source contained the literal `Type::member`, and a
  method is written `impl Hiding { pub const fn showing`, so the two halves never
  appear together and it would have refused nearly every true sentence in the
  repository. That half was then removed entirely — see below.

**That is not three coincidences.** Writing a check means deciding what would count as
the answer, and *that decision is the exact place this fault lives*. So the moment of
writing a check is the moment of maximum exposure to it, and an instrument is at its
least trustworthy on the run where it is newest.

**And there is a second decision, later and less obvious: what the check refuses to
look at.** The fifth instrument was first built to check Rust names as well as paths.
Run once against the repository, most of what it found was correct writing — `std`'s
unstable `ErrorKind::FilesystemLoop` quoted in the quirk log, another project's type,
and names of work a plan has not done yet. **A name absent from this repository is
evidence of nothing**: it may be the language's, another project's, or next week's. The
half was deleted rather than tuned, because a check that cannot tell those apart
produces confident findings about `std`, and confident findings about `std` are how an
instrument loses the right to be believed. A false positive in a gate is not a smaller
fault than a false negative — it is the one that teaches three lanes to route around
the gate, which is this record's own thesis arriving from the other side.

**The practice that caught all five is the same one, arrived at independently by three
lanes: run a new instrument against a known answer before trusting it against an
unknown one.** `0750` gives 126 and `0751` gives 0, both measured, before a line of the
fix was written. A throwaway printed six findings against the real documents before a
word was written about what the check knows. The harness was pointed at trees whose
ownership was set on purpose before it was pointed at a runner. The fifth was run
against the whole repository and **every finding it produced was read one at a time
against the file it named** before the check was allowed to gate anything — which is
how the two false positives were found, and they were found because reading twenty-two
findings is cheap and believing twenty-two findings is not.

Everything else in this record is a remedy for a check that is already wrong. This is
the one that applies while it is being written.

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

**And it does not claim they are all fixed.** Six are. One is the near-miss and
needed none. **Instance 3 is open** and belongs to another lane.

**One thing in it is deliberately not tidied.** The status reader from remedy 8 has had
its *yes* and *no* answers exercised on real pull requests; its `unknown` answer has
only ever run in a harness, never against a genuine failed read. Two lanes hold the
same instrument in the same state, and both of us nearly let *it has all three answers*
stand for *all three have been seen* — which is this record's own subject, one level in.

## What is asked of the owner

Nothing. This changes no promise, tier or law. The three sentences at the top are the
whole of it, and the ten instances are why they are worth believing.
