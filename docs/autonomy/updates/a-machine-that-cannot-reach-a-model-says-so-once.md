# A machine that cannot reach a model says so once

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 15
**Contributor:** Claude (checkout `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this closes

`docs/features.md` promises at v0.01:

> **★ And it never nags.** A machine that cannot reach a model does not follow
> somebody around asking them to buy credit — that is the greyed-out panel
> ADR 0009 already refused, in a different disguise.

ADR 0009 says what that means in full, under *no nagging*: **a machine that
cannot reach a model says so once, where it happened, and continues.**

Until this change it was one of six v0.01 promises with **no crate, no test and
no line anywhere** — and the worst kind of the six, because a promise about what
a machine does *not* do repeatedly has nothing that would notice when something
starts doing it. Everything the promise needed existed except one thing:
`alo-answering` knows the eight ways a place can fail to answer and words every
one of them, `alo-asking` and `alo-turn` both hand the failure back whole, and
**nothing remembered that a person had already been told.** A machine whose
provider account had emptied would therefore produce the same four lines at the
end of every turn, all afternoon: the greyed-out panel ADR 0009 rejected, rebuilt
one honest sentence at a time.

`crates/alo-telling` is that memory, and deliberately nothing more.

## What changed

### `crates/alo-telling` — new

| File | What it is |
|---|---|
| `src/unavailable.rs` | `Unavailable`: what makes one unavailability the same as another |
| `src/who_asked.rs` | `WhoAsked`: whether a person is waiting, which is what makes saying it again not a nag |
| `src/told_once.rs` | `ToldOnce`: the four lines a person reads, in the order they read them |
| `src/telling.rs` | `Telling` and `Tell`: what this machine has already said, for as long as a session lasts |
| `src/words.rs` | Two strings, and the English beside each |
| `src/testing.rs` | The places, failures and vocabulary the crate's own tests are written against |
| `tests/a_machine_that_cannot_reach_a_model_says_so_once.rs` | One test per acceptance criterion, against the vocabulary a real machine holds |

### Registration, which is where this kind of change actually fails

- `Cargo.toml` — the workspace's member list.
- `crates/alo-saying/Cargo.toml` and `crates/alo-saying/src/collecting.rs` — the
  crate, its two strings, its entry in `EVERY_LIST`, its line in
  `ONE_STRING_EACH` and its count in the arithmetic that proves nothing was lost
  and nothing was shared. This is the failure that really happened one floor
  down: `alo-overlay` declared nine strings that nothing collected, and every
  test inside that crate passed.
- `docs/autonomy/v0-01-evidence.md` — the ledger entry for *and it never nags*,
  and a second closure note. `crates/alo-reconciling` refuses a ledger that names
  a report which is not there, and it refused this one until this file existed.

## The design, and the decisions inside it

The task left several things open. Each was decided the way a senior engineer
would and is written down here rather than left to be inferred.

### The failure is taken by value, so a repeat is consumed rather than ignored

`Telling::about` takes an `alo_answering::Failed` **by value**. On a repeat it
returns `Tell::SaidAlready`, which carries nothing, and the failure goes out of
scope inside the call.

This is the whole design. The obvious alternative — hand the failure back and let
the caller decide — makes suppression a recommendation: a surface could word the
failure itself and the promise would hold only for callers who read the
documentation. Consuming it makes the guarantee structural. **On a suppressed
telling there is no value left anywhere in the program that a screen could be
drawn from.**

Nothing is lost by it. A suppressed telling shows nothing, so nobody can answer
an offer, so there is nothing for the offers to be for.

### What is remembered is an identity with no sentence in it

`Telling` holds `Unavailable` — a source and a reason — and nothing else. No
failure, no sentence, no moment. `Unavailable` has no `said` and will not get
one.

That is ADR 0009's *where it happened* as a shape. A telling exists at the place
the turn failed because there is nowhere else it could be built from: nothing
this crate holds can be rendered, so nothing it holds can reappear on a screen at
some later moment, in some other part of the machine, as a reminder.

### Two statuses from one service are two reasons

`WentWrong::HavingTrouble` carries the number a service answered with, and that
number is **inside the sentence a person reads**. So `503` and `500` are two
different tellings. Collapsing them would show somebody a line about a failure
that is not the one that happened, and the acceptance is strict in exactly this
direction: *a machine that swallowed the second failure would hide the one that
mattered.*

### `WhoAsked` has no `Default`

A caller that had not thought about the question would inherit whichever answer
somebody wrote first, and **both possible defaults are wrong in the direction
that matters**: `ThePerson` turns every background retry into a telling;
`TheMachine` silences somebody who genuinely asked again — the key that silently
does nothing, which the agent overlay's seam already ruled out as the worst
outcome available. It costs one word at each call site and buys a decision that
was actually made. A compile-fail example holds it.

### The memory is bounded, and it forgets the oldest first

Sixty-four unavailabilities, oldest evicted. It has to be bounded: a service
flapping between statuses would otherwise grow it without limit, and an operating
system with an unbounded list in it has a way of ending badly that has nothing to
do with nagging.

The **direction** is the argument. Forgetting an old telling costs at most one
repeat of something that happened long ago. Refusing to remember a new one would
suppress a failure nobody has been told about. **A bound may cost a repetition.
It may never cost a telling.**

### One per session, not one per turn and not one per surface

`alo_turn::Turning` is a turn and ends with it, so a memory kept inside one would
forget between turns — the same sentence in front of somebody every time they ask
anything, which is the panel ADR 0009 refused at a different cadence. One per
surface is worse: two surfaces each saying a thing once is a thing said twice,
and the person with two of them open is the likeliest to meet it.

So `Telling` is held where the session is held. That is also the honest limit of
what this change proves, and it is written into the ledger: nothing in this
repository runs a session with an agent in it yet, so the promise is kept by the
thing that could break it rather than by a machine anybody has watched all
afternoon.

### Two strings, and where the other two lines come from

A telling is four lines, in reading order:

1. `telling.the-agent-cannot-answer` — **this crate's.** Every sentence
   `alo-answering` says is about a *question* (*nothing answered on this
   machine*); none of them names the agent, so somebody who pressed the key
   expecting an assistant is never told which part of their computer stopped.
2. `alo_answering::Failed::said` — what went wrong, and where.
3. `alo_answering::Failed::nothing_was_sent` — that nothing left, whether or not
   there was anywhere to send it.
4. `telling.carry-on` — **this crate's**, and the *and continues* half:
   *You can carry on. Nothing else on this machine depends on the agent, and
   nothing here is waiting for you to do anything about this.*

The two in the middle are not re-worded here. A telling that reworded a failure
would be a machine with two accounts of one moment, and a test with only this
crate's vocabulary loaded fails if it ever starts.

The order is `ToldOnce::lines`' rather than the surface's, because the order is
part of the sentence ADR 0009 describes: a surface that put the reassurance first
would be telling somebody not to worry before saying what about.

### The constraint, and the test that holds it

*It never asks anybody to buy anything.* `words.rs` has a test that searches both
sentences **and their translator notes** for thirteen ways of selling something
and fails on any of them; the integration test repeats it against the machine's
real vocabulary. The line about an account being empty is `alo-answering`'s and
says what is so rather than what to do about it, which is ADR 0009's *why the
money case matters most*: the person who cannot pay must not become a
second-class user of a computer they own, and the first step towards that is a
machine that keeps mentioning it.

*It never chooses another source.* The offers a failure carried travel with the
telling, unranked and unchosen, and `ToldOnce::take` is
`alo_answering::Failed::take` unchanged. Running out opens no door that a runtime
which was simply not running would not have opened — that is a test, and it is
the same test `alo-answering` runs one layer down, because ADR 0008's *never a
silent fallback* runs in both directions.

### What this crate deliberately does not cover, said plainly

ADR 0009 lists six ways an agent becomes unavailable. This crate is about the
ones that arrive as an `alo_answering::Failed`: the money running out, no model
there, the machine offline, the provider down, the key expired.

- **Declining the agent at setup** is not here on purpose. No turn happens on
  such a machine, so there is nothing to suppress and nothing to say; ADR 0009
  answers it in the stronger way, by the agent's surfaces being absent rather
  than present-but-disabled.
- **A policy refusing a source** is not a telling either. The machine can reach a
  model; a rule says it may not, `alo_egress::NotPermitted` names that rule in
  its own words, and suppressing it would be this crate quietly hiding a decision
  an administrator made (ADR 0016).

Nothing is written down. `alo-answering` already settled that an entry per
failure would build a log of somebody's questions failing, one honest entry at a
time, and a crate whose whole subject is *how often has this happened* is the
last one that should start keeping that count on a disk.

## Verification

Run from `C:\dev\alo-os-claude` through `wsl -d Ubuntu -u root`, with
`CARGO_TARGET_DIR=$HOME/target-claude` — the arrangement
`tools/kernel-loop/src/gates.rs` uses.

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace` | pass |
| `cargo doc --workspace --no-deps` | clean |
| `cargo fmt --all --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test` in `tools/kernel-loop` | clean |

The acceptance, one test each, in
`crates/alo-telling/tests/a_machine_that_cannot_reach_a_model_says_so_once.rs`:

| The acceptance | The test |
|---|---|
| the same unavailability told once is told once | `the_same_unavailability_told_once_is_told_once` |
| telling it again takes the source, the reason or the person asking again | `telling_it_again_takes_something_changing` |
| a different reason is a different telling and is not suppressed | `a_different_reason_is_a_different_telling` |
| nothing is told until a turn actually failed | `nothing_is_told_until_a_turn_actually_failed` |
| every string is in the vocabulary `alo-saying` collects | `every_string_is_in_the_vocabulary_this_machine_collects` |
| it never asks anybody to buy anything and never chooses another source | `nothing_asks_anybody_to_buy_anything_and_nothing_chooses_another_source` |

Refusal paths are tested beside the legitimate ones throughout: a suppressed
failure leaving nothing to show, a full memory forgetting the oldest rather than
refusing the newest, a word that starts selling something, an offer from another
failure being refused with the failure handed back, and — as compile-fail
examples, so that adding a second door becomes a failing build — an
`Unavailable` assembled from a place and a reason, a `ToldOnce` made from a
sentence somebody wrote, a `ToldOnce` made from a remembered identity, and a
`WhoAsked` obtained from `Default`.

**Not run, and not claimed.** No hardware. No machine has been watched for an
afternoon, because nothing in this repository runs a session with an agent in it;
the ledger says so in the entry rather than in this report alone. No pixels are
drawn and none are tested — the surface is task 3's overlay.

## Limitations

- **Nothing adopts it yet.** `alo_turn::NoAnswer::DidNotAnswer` still hands the
  failure to its caller whole, which is correct: the caller holds the session and
  the session holds the `Telling`. Adoption belongs with whatever runs a session,
  which does not exist (task 13, blocked on ADR 0024).
- **The memory ends with the session.** A person who signs out and back in is
  told again. That is deliberate — a memory that outlived a session would be
  state on a disk about somebody's failures — but it is a behaviour worth knowing
  before somebody reports it as a bug.
- **Two surfaces sharing one `Telling` is the caller's arrangement**, not
  something this crate can enforce. It is documented; a shell that built one per
  window would say things twice and nothing here would notice.

## Proposed shared-document updates

For the integration owner; not edited here, per `docs/autonomy/SHARED_MAIN.md`.

**`CHANGELOG.md`** — under Unreleased:

> **A machine that cannot reach a model says so once.** When a question cannot be
> answered — the model is not running, the provider is down, a key expired, an
> account has run out — alo OS says so once, where it happened, and carries on.
> It does not repeat itself, and it never asks anybody to buy anything. Asking
> again yourself always gets an answer, and a *different* failure is always
> reported: the machine goes quiet about what you have already been told, never
> about something new.

**`docs/autonomy/QUEUE.md`** — task 15 done; task 16, *A default nobody chose, or
a promise that says so*, added to the v0.01 delivery plan and ready.

**`docs/autonomy/STATE.md`** — reference this report. One of the audit's six
promises with no evidence at all is closed, leaving four; none of the four can be
closed without a screen, a decision or a machine.

**`ROADMAP.md`** — nothing moves. *On the machine* does not move, and no exit
gate is ticked by this.
