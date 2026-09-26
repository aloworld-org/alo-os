# One account in one order, and a sentence that waits on a rule

Tasks 13 and 14 of `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, both
2026-09-26 on the third PC. A follow-up to
[`every-sentence-at-a-desk-that-changed.md`](every-sentence-at-a-desk-that-changed.md),
which is **not edited**: it is published, it is still true, and the two findings it
recorded are what this takes up.

One of the two is done. The other cannot be until a rule exists, and the rule is
proposed rather than decided.

## Task 14 — the ordering, decided and written down

The finding: `Attached::resumed_to` does `notes.insert(0, Note::TheDeskChanged)`,
an ordering decision recorded in code, and across the three collections a resume
answers with there was none. A caller held `Attached::notes`, `Resumed::moved` and
`Resumed::came_back` with nothing to say which came first, so **the order in that
report's table was the walk's own choice** — two surfaces could have shown one
morning in two orders and both been correct, and neither would have known what
task 11 had already decided about the first sentence.

**Where it belongs: in `alo-displays`.** `Attached::the_account(&resumed, strings)`
returns everything a person reads about a resume, in the order they read it — the
notes, which already begin with what happened; then the screens that have gone and
where the work open on each is now; then the screens that are back and what
returned to them. What happened, then where their work went, then what came back.

The argument for putting it there rather than leaving it to whoever draws it is one
line: **which sentence follows which is not drawing.** Where each sits on a screen
and in what typeface is the shell's; what a person is told, and in what order, is
decided where the facts are. Half of this order was already in `alo-displays` on
purpose, and the half that was missing was missing rather than delegated.

Three things about the shape of it:

- The notes come from `self`, so a caller cannot hand it the notes of some other
  set of screens by mistake.
- `Attached::notes` stays, and now says what it is for — the sentences on their
  own, unordered against what a resume also answers with.
- It is **additive**. No sentence changed, no existing door moved, and nothing that
  reads `notes`, `moved` or `came_back` today reads differently.

`crates/alo-displays/src/attached.rs` holds a test of its own for it, rather than
leaving the guarantee to the walk that found it: a later change that appended the
notes after the screens would pass every other test in that crate.

**And the walk now reads the machine's order instead of composing one.** Its table
is unchanged — the sequence the machine gives is the sequence the walk had chosen —
which is the outcome worth having: the table was right, and it is now a measurement
of what a person meets rather than of what a test file decided.

## Task 13 — the reassurance, and why it is not fixed here

The finding: on the wake back at her own desk, `displays.the-desk-changed` says

> Your screens have changed since this machine went to sleep, so alo OS has set up
> the ones in front of you now — **the arrangement you made at the other desk is
> still here for when you are back at it**

to somebody who is back at it. It is a sentence making a promise about a situation
it is not in, and it will be wrong on **every return leg of every journey**.

**The answer is to split it**, and the reasoning is short. The clause is a
reassurance that what she is losing is kept. On the way out she is losing her own
desk's arrangement and the reassurance is exactly right — it is why task 11 wrote
it. On the way back she is losing nothing and *gaining* the arrangement she made,
which `displays.as-you-left-them` already announces in the same breath. So the
clause is apt precisely when **the set she has arrived at is not one whose
arrangement is kept**, which is a condition the machine can already answer:
`Changes::for_screens` is what decides it, and it is what chose between
`AsYouLeftThem` and a worked-out layout in the first place.

That makes `the-desk-changed` the plain fact — your screens changed, and these are
the ones set up now — and the reassurance a second sentence said under that
condition. One fact in one place, which also disposes of the duplicate: on that
morning *arranged the way you last left them* is the true one and the only one
needed.

**Why it is not done in this change.** Splitting a sentence means the remaining
half no longer means what the whole did, and a translation is keyed. Editing the
English under a published key leaves every existing translation of it **rendering
perfectly and saying something the product no longer says** — nothing fails and
nothing warns. Nothing in this repository said whether that was allowed: no
contract covers the vocabulary, `alo-strings` states no rule, and three earlier
commits have changed a `Word`'s text with nothing to consult.

That is a rule rather than a judgement, so it is
[ADR 0068](../decisions/0068-a-published-sentence-changes-by-getting-a-new-key.md),
**proposed**: a sentence whose meaning changes gets a new key and the old key is
retired; only a change that leaves the meaning alone may be made under the key it
already has. Task 13 is blocked on it, and blocked is the honest status — the
argument above is ready and waiting on one decision, not on more thinking.

The ADR is deliberately narrow. It says what a change to a published sentence looks
like; it does not say whose the sentence is, and it does not decide this one. A
crate reaching into another to coordinate phrasing would put a second author on a
sentence that already has one, and that is not what is being proposed.

## What this did not do

No sentence was added, reworded or moved. `displays.the-desk-changed` is exactly as
task 11 published it. Task 12's report and its table are untouched. Nothing in
`crates/alo-shell`, nothing on the machine, `logind` stays rented (ADR 0011).

## The fixture rule, moved somewhere a reviewer meets it

Task 12's report ended with a note about method: written first with
`Changes::untouched()` throughout, the walk had alo OS tell somebody three times
that the screen she had used all day had never been used with this machine before —
a machine remembering nothing has nothing to recognise. It reads exactly like a
serious finding, it would have been reported as one, and the real finding
underneath it only appeared once the walk kept what a session keeps.

That paragraph was in one report, where only somebody reading that report would
meet it. It is now a rule in `docs/autonomy/LOOP.md` under *What the loop may never
do*: **never accept a fault a fixture manufactured**, and before writing a fault
down, ask what in the fixture is unlike a machine. A fixture that would make any
code look broken is measuring itself.

## Evidence

    cargo fmt -p alo-displays -p alo-sleeping --check                     PASS
    cargo test -p alo-displays -p alo-sleeping                            PASS

    alo-displays   123 unit tests, the new ordered-account test among them
    alo-sleeping   the walk's 3 tests, its table unchanged

Full workspace gates in the change that lands this.

probe_should_be_7=7
