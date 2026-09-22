# ADR 0063 — A machine that cannot run the engine says so, rather than failing

**Status:** **accepted, 2026-09-21, by the owner**, in task 9 of
`docs/autonomy/v0-5-documents-and-paper-plan.md` — written, argued and committed
by the owner in `b467e6a`, which states the decision, the shape it takes and the
two things it may not do. This file records it where a reader of
[ADR 0039](0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
will find it, because that decision says the opposite in as many words and two
documents disagreeing is how a rule stops being one.
**Date:** 2026-09-22
**Proposed by:** the documents and paper workstream, from the third PC
**Context:** [ADR 0039](0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
*What must happen before task 2 can be built* §2 — *the conversion tests then
fail loudly where it is absent, and are never `#[ignore]`d and never skipped. A
test that skips itself when the engine is missing reports green on exactly the
machines where nothing converts*; `CLAUDE.md`'s gate (*the refusal path tested
as carefully as the happy path*); `crates/alo-in-use/tests/a_stream_through_the_media_server_is_listed.rs`,
whose own words are the shape this takes — *a skip nobody can see is the same
colour as a pass*.

## The question in one line

**ADR 0039 forbids these tests to skip. One machine in the fleet cannot ever run
the engine. Which gives?**

## The new fact, which ADR 0039 did not have

ADR 0039 was written on 2026-09-14, when the fleet was two x86_64 PCs and the
sentence *the gate machine gets the pinned engine* named a thing somebody could
go and do. Absence of the engine meant **a machine nobody had finished setting
up**, and failing loudly is exactly right about one of those: the failure is a
work item, and it is addressed to somebody who can close it.

Since then the fleet includes a Mac, which is **aarch64**. The engine ADR 0039
pins (ADR 0039 §*The options*) is an x86_64 build, and the image pins it by
digest, so there is no release of it that machine could install. Absence there
is not an unfinished setup. It is a permanent property of the hardware, and the
failure it produces is addressed to nobody.

Ten tests failed on that machine on every run. A suite that is red for a reason
nobody can fix is one people learn to read past, and the next real failure hides
inside it — which costs more than the thing ADR 0039 was protecting.

## What ADR 0039 was actually protecting, stated fairly

Not *tests never skip*. The harm it names is precise: **green on exactly the
machines where nothing converts.** A gate machine that quietly reported success
while its converter was missing would be certifying a conversion nobody ran.
That harm is real and this decision does not dispute it; it is why option C
below is refused, and why the skip is narrow rather than general.

## The three that were weighed

### A — Leave it. The Mac's suite stays red

**What it costs:** ten permanently failing tests on one of five machines. Every
person and every loop reading that machine's run has to hold in their head which
ten failures are furniture, and the eleventh — a real one — reads the same as
the other ten. This is the position today, and it is the reason the owner wrote
the task.

### B — The test asks whether the engine runs, and where it cannot, skips and says why

**What it costs:** the run is green on a machine where nothing converts, which
is the sentence ADR 0039 refuses — with the reason printed beside the test's own
name, so *why* it is green is in the run rather than in somebody's memory. It
costs one engine start per test binary, and it costs a check that the list of
tests which ask stays exactly the list of tests which need to.

### C — `#[cfg(not(target_arch = "aarch64"))]` on the ten

**What it costs:** the tests do not exist on that machine. A test that is absent
and a test that passed print the same thing, so this is strictly worse than B on
ADR 0039's own argument, and the next architecture inherits the silence without
anybody deciding anything. The plan forbids it by name.

## The decision

**B**, with four terms, each of which is part of the decision:

1. **The ask runs the engine.** Not `test -x`, and not the presence of a file.
   The engine on a machine is a shell script that starts a binary beside it, so
   on a machine of another architecture the executable bit is set on a wrapper
   over a binary that cannot run — and this repository has already shipped one
   defect from the bit standing in for *the program runs*. The ask starts it
   with `engine.rs`'s own argument list and the question *what are you*, and
   reads the answer.
2. **The ask is only whether the engine runs at all.** An engine that starts
   and then converts badly — a missing export filter, a copy that is not a PDF —
   **still fails loudly**, in the test that converts, on the machine that has an
   engine. An ask that converted a document first would turn that defect into a
   skip, and that defect is precisely what ADR 0039 is right about.
3. **The skip says what was missing**, naming where the engine was looked for,
   in the shape `alo-in-use` already uses.
4. **Exactly the tests that need the engine ask**, held by a test that reads the
   sources both ways: a conversion test added without the ask fails, and an ask
   added to a test that does not need one fails. Without this the narrow skip
   becomes a general one by drift, which is how B turns into the thing ADR 0039
   feared.

## What this does not decide

- **Nothing about what those tests assert.** Every assertion is what it was, and
  on a machine with the engine all ten still run and still pass.
- **Nothing about `alo-image`.** The image's own check that the engine is in it
  is a different question asked of a different thing, and it is unchanged.
- **Whether the Mac ever converts a document.** It does not, and this decision
  does not pretend otherwise; it makes that legible rather than loud.
- **Any other crate's skips.** `alo-in-use`, `alo-sessiond` and `alo-files`
  already skip on their own grounds. Whether the fleet's suites are red anywhere
  else for reasons nobody can fix is unmeasured, and is a task rather than a
  guess.

## Consequences

- **ADR 0039 §*What must happen before task 2 can be built* §2 is amended**, in
  its last two sentences only. It keeps its status, its options, its
  recommendation and everything else it decided; a note there points here.
  `crates/alo-opening/tests/converting_waits_on_its_decision.rs`, which holds
  ADR 0039 in place, is unaffected and unchanged.
- `crates/alo-converting/tests/asking/mod.rs` is the ask, shared by the test
  files that need it, and names the engine through
  `alo_converting::engine::THE_ENGINE` — so `engine.rs` is still the one file
  that knows the name, and no sentence a person reads has changed.
- `crates/alo-converting/tests/a_machine_that_cannot_run_the_engine_says_so.rs`
  is the ask's own refusal paths and term 4's check.
- A machine with no engine runs `alo-converting` green, with ten stated skips.
  **A green run on such a machine is not evidence that this machine converts**,
  and nobody may read it as acceptance of a conversion. The conversion evidence
  is a run on a machine that has the engine, and the reports say which machine.
