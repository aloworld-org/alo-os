# The answer to *why is it waiting* reaches a person

**Branch:** `task/mac/the-answer-reaches-a-person`, off `main`
**Plan:** task 1 of `docs/autonomy/the-terminal-in-full-plan.md`, which is in
flight as #754 and deliberately not linked here until it lands — a link to a file
this branch does not carry is the fault that failed #751's gate earlier today
**Date:** 2026-10-17

## The judgement, in one sentence

**This makes an existing promise true rather than adding one**: the five answers
and all four mechanisms were built, measured against real processes on a real
kernel, and `docs/features.md`'s own word is *ask* — what was missing was a road
between a person and an answer that already worked.

## What a person can now do

Open a terminal, run something that waits, press `Ctrl`+`Alt`+`W`, and read
**waiting for the keyboard** — or for the network, for another process, still
computing, or that nobody can tell — as a line in the transcript with `alo`
beside it in the gutter.

The owner said what this is, on 2026-10-17: *"Analysis the command and see why
it's not working and then reply the user on why it is waiting."* So the **answer**
is the deliverable. The four actions stay built and unoffered; `docs/features.md`
records the pairing of answers to actions as the owner's to settle, and it is not
inferred here.

## Where it appears was derived, not invented

The design file is silent on this feature, but its gutter annotation is not:

> SYSTEM means an attributed observation, with scope and time. **ALO is
> interpretation.**

The waiting answer **is** interpretation of observations — three unprivileged
reads of `/proc`, read into one of five answers. So it is a line in the
transcript carrying the `alo` label. **No new region, no new chrome**, and the
place had existed since ADR 0113 and never been used, because nothing had ever
written into a transcript as alo.

## What was built

- **`alo_terminal::WhatAloSaid`** — where alo's remark lives. A **vocabulary key,
  never a sentence**, so it is read in the person's own language and so there is
  no road from a program's bytes to a line labelled `alo`. **One slot, not a
  list**: asking twice replaces, because a transcript that grew a line per ask
  would push the program's own output off the screen. Five tests.
- **`regions_of` takes two counts** — the program's lines and alo's. Still only
  counts, so `nothing_reads_the_bytes_to_decide_who_spoke` still holds: it cannot
  see what anybody wrote. The invariant sweeps were **widened to both
  dimensions** rather than merely fixed for the new arity.
- **`Action::WhyIsItWaiting`**, its word, its translator's note, a German test
  fixture, and `Ctrl`+`Alt`+`W` — beside `Ctrl`+`Alt`+`T`, which opens the
  terminal it asks about. **Which key is an implementation, not a promise**: the
  promise says *ask*, the design names no chord, and a person can rebind or clear
  it in Settings.
- **The dispatch**, beside the chord that opens a terminal and for the same
  reason: it borrows nothing.
- **The drawing** — alo's line in the transcript's own type at its own inset, and
  the gutter's `alo` label beneath the program's region.

## The closed enum made four crates refuse to compile, which is what it is for

| crate | the decision it forced |
|---|---|
| `alo-dividing` | it divides nothing — one line into a transcript, in a window already where the person left it. *That file's comment says it had earned itself three times; this is the fourth.* |
| `alo-access` | reached at the window's own controls, because what it acts on is that window's command — **and nothing there does it yet**, named plainly as `Action::Terminal`'s arm does rather than smoothed over |

## Sub-work this needed, which nobody had listed

**`alo_shell::the_road_every_action_takes` was cited and did not exist.**
`alo_shortcuts::Action::Launcher`'s own documentation says that module *"names it
as the single exception rather than letting it hide among the others"*. There was
no module, no test and no file of that name — so nothing named `Launcher` as
unreached, and **nothing held any action to having a road at all.** My new
action's dispatch had nothing holding it either, which is how I found it.

`CLAUDE.md`: *work a task needs is part of that task — a caller that does not
exist, a module with no consumer, **a test that never ran** — build it, in the
same change, and say in the record that you did.* So it is built, and this is the
record.

It reads the source, because the fault cannot be provoked: a missing arm does not
fail or log, the chord simply falls through, and no input separates *not bound*
from *bound to nothing*.

## Three attempts at that guard, each defeated by the same trap

Worth reading in order, because the trap is this repository's own shape:

1. **It matched the action's name anywhere in `src/`.** Mutation: delete the
   dispatch arm. **Passed** — the handler still names its own action in its own
   first line. A handler is not a road.
2. **It then required the chain to name the handler.** Same mutation. **Passed
   again** — the comment I had written *above* the arm names the handler the arm
   calls. In this repository every arm is explained directly above itself, so the
   explanation and the thing are always adjacent.
3. **Comments stripped from the chain's text.** Same mutation: **fails**, naming
   the uncalled handler.

And a fourth, found by the guard against itself: it first reported `Launcher` as
routed because three **test** modules name it — two listing it as a Settings row
and one asserting it is *not* dispatched. **The test that mentions it most is the
one saying it has no road.** Test modules are excluded now.

`a_comment_naming_a_handler_is_not_a_call_to_it` is the control that cost two of
those attempts, asserted in both directions so a filter that stripped everything
would fail too.

## What runs forever

| | |
|---|---|
| `what_alo_said_tests.rs` | 5 — nothing said draws nothing, a remark is one line, asking twice replaces, it can be emptied, and every one of the five answers is sayable |
| `who_is_speaking_tests.rs` | 8, sweeps widened to both counts; the whole-vector assertion catches a nought-line region appearing in front of alo's |
| `a_chord_asks_why_a_command_is_waiting_tests.rs` | 5 — carried with a terminal, carried without one, **another chord is not this one** (the control), a prompt gets nothing said, and **a real `cat` waiting for the keyboard answered through the chord** on a real pseudoterminal |
| `the_road_every_action_takes.rs` | 7 — every action routed or named, nothing named is routed after all, each excuse says enough, the handler-is-called check, and two controls |

## A flake found and deliberately not fixed

`alo-terminal`'s `a_program_whose_name_has_a_space_is_still_read_correctly`
failed once and then passed **11 consecutive runs** — alone three times, in
parallel eight. Not a short timeout: its poll is already five seconds. I could
not reproduce it and have no failure message, so **it is not fixed**: sending a
remedy without the measurement that produced it is the fault `CLAUDE.md` names.
Recorded here so there is a trail if CI meets it.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings` clean.
`doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings` clean.
`cargo test --no-fail-fast` over `alo-terminal`, `alo-shortcuts`, `alo-shell`,
`alo-access`, `alo-dividing`, `alo-saying` and `alo-citing`: **0 failures summed
across every result line**.

**The vocabulary snapshot was regenerated**, because a new `Word` moves it —
`UPDATE_VOCABULARY_SNAPSHOT=1`. It refuses to write for any key whose text moved
without a declaration; nothing moved here, a key was added.

**What this did not run**: the rest of the workspace. CI runs the whole suite and
is the required check. This is a head status: these gates passed on this tree,
not on the tree that becomes `main`.
