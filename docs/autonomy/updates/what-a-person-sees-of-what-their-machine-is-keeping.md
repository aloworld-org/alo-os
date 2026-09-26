# What a person sees of what their machine is keeping

Task 15 of `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, done
2026-09-25 on the third PC.

ADR 0045 point 5 has two halves — *what an undo may keep is **visible** and
forgettable* — and task 14 built the forgetting. Nothing showed a person any of
it. Every piece existed: `alo_keeping_up::HowFarBack` is the window,
`alo_letting_go::keeping` reads and writes the file they change it in,
`alo_letting_go::ask` is the one act, `alo_keeping_up::WhatWasKept::forgetting`
is the sentence they approve, and `alo-measuring` can count a folder. A person
could not read any of it and could not ask for the space back short of editing a
settings file by hand.

`crates/alo-changing-undo` is the pane's model: four files, three sentences,
28 tests. It draws nothing — that is the shell's — and it is the shape
`alo-changing-updates` already has.

## No second copy of anything

| what a person reads | whose it is |
|---|---|
| how far back this machine keeps | **this crate's**, with both numbers filled in |
| how much room that is holding | **this crate's**, with the size already worded |
| nothing has needed keeping yet | **this crate's** |
| the sentence they approve | `alo_keeping_up::WhatWasKept::forgetting` |
| a pair of numbers that is not a window | `alo_keeping_up::HowFarBack`'s refusal |
| this machine keeps nothing of what your files were | `alo_keeping_up::words::NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE` |
| why a size is not the whole truth | `alo_measuring::Counted::said` |

The third row down is the same fact an undo refuses with, so a person meets one
wording of it however they arrived. `words.rs` holds a test that no sentence of
this crate's is word for word a sentence of either crate it says the others out
of — checked by the sentence rather than by the key, because a copy somebody made
would arrive with a key of its own and that is what would catch it.

## The finding that shaped the crate

**A pane cannot ship a dependency on `alo-letting-go`.**

`alo-saying` is the machine's one vocabulary, so it ships every crate that
declares a word — which puts every such crate inside the process an agent's turn
runs in. `alo-letting-go` holds the one privileged remover on this machine, and
`crates/alo-letting-go/tests/a_turn_cannot_arrive_at_this_road.rs` holds that the
only road into it from where a turn runs is its vocabulary.

So a pane that read the person's file or performed the act would carry that road
into a turn's process, and that test would fail — **correctly**. The obvious way
to write this crate was the wrong one, and the test that says so was written by
task 14 against exactly this mistake.

What the pane does instead:

- `WhatIsKept` is what a person reads. It is handed the window and the
  measurement; it opens nothing.
- `Wanted` is a window a person asked for, already through the only check there
  is. The **caller** keeps it.
- `Offered` → `Approved` is the one act. `Approved` carries the moment of
  approval and is neither `Clone` nor `Copy`, so one approval cannot become two
  acts; neither type is constructible from outside the crate. The **caller**
  carries it out.

The caller is the session the person is sitting at, which is not a turn.
`alo-letting-go` is a **dev-dependency** — a test's, and in nobody's process —
which is how `tests/the_machine_really_obeys_the_changed_window.rs` takes the
road the crate may not.

That the file and the act stay out of the crate is also why it can hold no second
copy of them: **it has no copy at all.**

## Refusals, held as carefully as the happy path

- A machine that keeps nothing is a **variant**, not a flag, so
  `WhatIsKept::may_forget` has nothing to return — the promise of ADR 0045's
  sixth term is kept by the shape of the value rather than by a check somebody
  has to remember. It says *this machine keeps nothing of what your files were*
  instead.
- An act declined returns **nothing at all**. Not an empty value, not a status to
  inspect: no value reaches a caller, so there is nothing any road would accept
  and every snapshot is where it was.
- A window of zero days or zero changes is refused before anything could write
  it, and a test holds that no file was left behind on the way to the refusal.
- A size that **could not be read** is not a none. Zero bytes seen with
  `Counted::NotRead` still offers the act, because that is exactly when a person
  wants the space back, and it says why the number is not the whole truth in
  `alo-measuring`'s own words.
- A **known** none says *nothing has needed keeping yet* rather than an amount of
  no bytes.

## The test task 15 asked for by name

`tests/a_turn_cannot_arrive_at_this_pane.rs` — the same walk
`a_turn_cannot_arrive_at_this_road.rs` makes, four cases: this pane ships no road
to the act at any depth; the only road into it from where a turn runs is its
vocabulary; this pane's own sources name no road a person alone may take; and
nothing a turn runs inside spells anything of it but `declare_into` and
`changing_undo_words`.

The third of those **failed first** and was right to: the crate's own header
spelled `alo_letting_go::keeping::keep` and named `undo.toml` in prose. Either
would also have failed `alo-letting-go`'s road test, which reads text and does not
care that a doc comment is not a code path. The prose was reworded; that suite
was re-run green beside this one.

## Two findings this task did not fix

**1. ADR 0045's fourth accepted term is not built.** It asks that *what is
filling the disk counts snapshots, by name* — `alo-measuring`'s answer including
what undo is holding, as its own line. `alo-measuring` has **no mention of a
snapshot or an undo at all**: zero matches over the whole crate. Task 15's own
text says that line already exists, and it does not.

That crate is **lane B's**, so this task did not build it, and it did not count
anything itself either — a pane that counted would be the second answer the
constraint forbids. `Holding::measured` takes an `alo_measuring::Node` and
carries its `size` and its `counted` through unchanged, so the number the pane
shows *is* that crate's answer, held by a test; the caller measures the folder.

**What is still owed:** the named line in *what is filling the disk*. A person
who goes looking there for what their undo is holding will not find it, which is
the exact failure the owner's fourth term was written to prevent — *an answer that
hid it would send a person hunting for space the machine itself was keeping.* It
belongs to whoever owns `alo-measuring`.

**2. This crate does not belong in `alo-declared`.** Task 15 says to collect it
into `alo-saying` and `alo-declared` *as every crate with words is*. But
`alo-declared` is **every verb alo OS ships**, and this crate is forbidden a verb
by the same ADR whose term the task is keeping. `alo-changing-updates`, the
words-only crate this is modelled on, is not in it either. `alo-saying` alone,
which is the precedent; registering it in `alo-declared` would have contradicted
the task's own constraint.

## Smaller things worth knowing

**Registering one crate in `alo-saying` touched four hand-kept lists** in
`collecting.rs`: the name array, the `declare` run, `ONE_STRING_EACH`, and the
per-crate word count. Two were found only by a failing test. That is the class
the software plan's task 10 exists for, and it cost this task a gate run.

**The room sentence takes the size already worded.** How a number of bytes reads
in a person's language is a drawing decision, and a second opinion in this
repository about what `5183545344` should say would be one more answer nobody
could reconcile. The pane exposes the `u64` it was given, and a test holds it
equal to `alo-measuring`'s.

## A note on the name

`alo-changing-undo` matches `alo-changing-updates`, `-drives`, `-printers` and
`-network`. Every one of those is the surface of a **broker verb family**, and
two of them are the broker plan's. This one is not a broker verb: ADR 0045's
seventh term keeps forgetting off the broker's list deliberately, because an
agent that can forget an undo can erase the evidence of what it did.

The name is kept — the plan named it and the owner confirmed it, and a defensible
reading holds: *the Settings surface for a privileged change.* The distinction is
written into the crate's own header so the family's meaning is documented rather
than quietly broken. A later task that wants the family to mean only *broker verb
surface* has one crate to rename and nothing else.

## What this does not do

Draw anything, touch the person's settings file, perform the act, declare a verb,
open a socket, or read a file. Nothing on the machine, and no ADR re-decided.

## Proposed for `CHANGELOG.md`

Not written here: `CHANGELOG.md` has one writer
(`docs/autonomy/SHARED_MAIN.md`). Proposed entry:

> **What your machine is keeping, where you can see it.** A machine that can put
> the agent's changes back now says how far back it reaches and what that is
> costing in disk space, lets you widen or narrow it, and offers one act that
> forgets all of it — one sentence, one approval. A machine that cannot do it
> says so plainly instead of offering something that would fail.

## Evidence

Run in WSL on the third PC, sources on the Linux disk, linked with mold, in the
one build directory this machine has.

    cargo fmt --all --check                                        PASS
    cargo clippy --workspace --all-targets -- -D warnings          PASS
    cargo test --workspace --no-fail-fast                          FAIL — see below
    cargo doc --workspace --no-deps  (RUSTDOCFLAGS=-D warnings)    PASS
    the supervisor's formatting                                    PASS
    the supervisor's clippy                                        PASS
    the supervisor's own tests                                     PASS

    661 test suites ok.  alo-changing-undo 28 tests, 0 failed.
    alo-letting-go re-run green, its road test included.
    alo-saying re-run green after the four hand-kept lists.

**The one failure is not this change's, and it is already written down.**
`alo-converting`'s `an_older_word_document_is_converted_and_what_it_lost_is_named`
— 16 passed, 1 failed — expects one substituted font and gets two:

    left:  {FontSubstituted("Garamond"), FontSubstituted("Liberation Serif"), FieldFixed(Date), Comments}
    right: {FontSubstituted("Garamond"), FieldFixed(Date), Comments}

`docs/quirks.md`, *A conversion test names one substituted font on its own
machine and two on this one*, records it as **red on `main` itself**, checked in a
worktree of `origin/main` at `cf641d5` with no branch merged, three times, and not
cured by installing `fonts-liberation`. `alo-converting` is not this lane's crate
and this change touches no file of it — the six files it touches are
`Cargo.toml`, `alo-saying`'s manifest and `collecting.rs`, this plan, the new
crate, and this report.

**New with this run: it reproduces on the third PC.** The quirks entry was
measured on the development PC. The same assertion, the same extra *Liberation
Serif*, on a different machine with a different font set — so the question for
that crate's owner is not one machine's fonts. A line was added to that entry
saying so.

## Two machine findings from gating this

**A second build directory filled the build filesystem.** This machine's rule is
one build directory and one source copy (supervisor 5c51b4d); a run that made a
second pair put 25 GB and 9.4 GB on a 35 GB disk and died with *No space left on
device* — `mold: failed to write to an output file. Disk full?` and a rustc SIGBUS
behind it. **D: was never touched**, which is the whole point of the 75 GB cap the
rebuild put on: an unrecoverable failure became a `rm -rf` and a resize.

**35 GB was not enough for the full gates, and 40 GB is only just.** The
filesystem was grown to 40 GB — in a 45 GB file; online `ext4` resize stops short
of the file, bounded by reserved descriptor blocks — and the run that passed
finished at **95% used, 2.2 GB free**, with the workspace tests alone taking it
from 26 GB to 35 GB. The gate script now prints free space after every gate, so a
run that fills the disk says which gate filled it. A later task that adds test
targets should expect to grow this again.

probe_should_be_7=7
