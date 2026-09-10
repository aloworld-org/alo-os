# One approval, and the sentence a person approves

**Date:** 2026-09-10. **Workstream:** v0.01 delivery plan, task 7.
**Contributor:** Claude Code (`C:\dev\alo-os-claude`), while the desktop worker
owns the compositor lane.
**Status:** ready for integration.

## What changed, in words a person outside this repository can read

alo OS could already decide that a change needs approval, word it in one
sentence, refuse it if the grants said no, carry it out once and write down what
happened. What it could not do is **ask anybody**. `ROADMAP.md`'s v0.01 exit
gate has *approve the sentence, see it happen* in the middle of it, and until
this change the sentence had nowhere to appear.

`crates/alo-approving` is that surface. A change an agent proposes is put in
front of the person as the sentence the machine itself generated from the
arguments it validated — never a description a model wrote — with the agent it
belongs to and how long is left to answer. Approving it carries it out exactly
once; saying *no* is a whole answer and nothing asks why; a question that stood
too long is neither shown nor carried out, and says so in words that quote the
change so it can be asked for again. When there is nowhere to put the question —
no desktop, no screen — the machine says that too, rather than leaving an agent
looking as though it ignored an instruction.

## The plan's acceptance, and where each half is held

> *A proposed change is shown as the sentence `alo-turn` renders, approved once,
> carried out, and refused after its proposal has expired — with the record
> carrying what was approved and by whom.*

| Criterion | Where it is held | Test |
|---|---|---|
| Shown as the sentence the turn renders | `Asked::of` takes an `alo_capability::Waiting` and nothing else; no public field, no `From`, no deserialiser, no constructor from text | `a_proposed_change_is_shown_as_the_sentence_the_turn_renders`, plus two compile-fail examples on `Asked` |
| Approved once | `Approving` takes the question off the surface before the turn is touched; `Approvals::approve` takes the proposal off its list in the act of answering | `one_approval_carries_the_change_out_exactly_once` |
| Carried out | every answer goes through `alo_turn::Turning::approving` — the only road, and the one that writes the record | `an_approved_change_really_happens` |
| Refused after it expired | `Asked::of` answers `None` once the question stops standing, and the turn refuses the answer in `alo-capability`'s own words | `a_proposal_that_expired_is_refused_rather_than_carried_out` |
| The record carries what was approved and by whom | nothing new: the entry `alo-turn` already writes, read back off a disk by `alo-keeping` | `the_record_says_what_was_approved_and_by_whom` |

The refusal paths are tested beside the legitimate ones throughout: saying no,
answering with nothing in front of you, a number that names nothing, a question
that lapsed at both moments it can be met, no compositor, and a compositor with
no screen.

## Source

- `crates/alo-approving/src/asked.rs` — `Asked`: the question as a person reads
  it. One constructor, from a change that is really waiting.
- `crates/alo-approving/src/surface.rs` — `Compositor` and `SurfaceRefused`: the
  port whatever owns the screen implements, with no rendering in it.
- `crates/alo-approving/src/approving.rs` — `Approving`, `Asks`, `Answered`: the
  change in front of somebody and the one answer it gets.
- `crates/alo-approving/src/refusing.rs` — `NotAsked` and `NotAnswered`.
- `crates/alo-approving/src/words.rs` — five strings under a new `approving`
  area.
- `crates/alo-approving/src/testing.rs` — fixtures, `cfg(test)` only.
- `crates/alo-approving/tests/one_approval_and_the_sentence_a_person_approves.rs`
  — the acceptance test, on a real disk with the record read back.
- `crates/alo-turn/src/turning.rs` — two additive accessors, below.
- `crates/alo-saying/src/collecting.rs`, `crates/alo-saying/Cargo.toml`,
  `Cargo.toml` — the new crate's words are collected, and the crate is a member.

## Decisions taken where the task left something open

**A new crate rather than a module of `alo-shell` or `alo-overlay`.** The three
surfaces published before this one — the summoning seam, the egress indicator,
the folder picker — are each their own crate with a compositor port and no
pixels in it, and the desktop worker owns `alo-shell`. Following that shape
keeps two workers out of one file and keeps this reviewable on its own.

**The two answers are strings; the sentence is not, and never can be.** What a
person approves is `alo_capability::Proposal::sentence`, filled from the
validated arguments. This crate declares *Approve* and *No* and nothing that
describes a change — there is a test (`nothing_here_describes_a_change`) that
fails if a string on this list ever starts naming a verb or a place. There is
deliberately **no heading** over the sentence either: a title would be a second
sentence about the same moment, one more thing to translate and one more thing
that can drift from what the machine will actually do.

**How long is left is a `Duration`, not a sentence.** `Asked::lapses_in` hands
back the time and words nothing. alo OS has no way to say *four minutes* in
twenty-four languages yet, and inventing one here would make this crate the
owner of a time format for the whole system. When there is one it will be
`alo-strings`', and every surface will use it. What a shell draws from the
duration — a countdown, a bar, nothing — is the shell's.

**Answering forgets the question, whichever way the turn answers.** A refusal is
a fact about *that* answer rather than a reason to leave the same button under
somebody's cursor; the list underneath agrees, since `Approvals::approve`
removes a lapsed proposal as it refuses it. A shell shows the refusal in the
words it arrives with; putting the question back up is the person's act.

**One question at a time.** `Approving::ask` replaces whatever was on the
surface, and answers nothing in doing so — the change it replaced is untouched
and still waiting. A person answering two changes at once is a person approving
the second without reading it.

**Two additive accessors on `alo_turn::Turning`**, both `#[must_use]` reads with
no new behaviour:

- `proposed(id)` — one change this turn put to somebody, *whether or not the
  question still stands*. `waiting_at` filters lapsed ones out, so a surface
  built on it alone would have one sentence for *that was answered already* and
  *that stood too long* — two facts a person does two different things about,
  and only one of which means asking the agent again. It delegates to
  `Approvals::of`, which was already public.
- `strings()` — the machine's own vocabulary, so the surface cannot word a
  proposal in a vocabulary of its own. It delegates to `Machine::strings`, which
  was already public.

**"By whom" is the agent, and the approval number beside it.** The record names
the agent whose authority the change ran under and the number of the approval
that let it happen; it does not repeat the person on every line. A v0.01 machine
is personal and has one signed-in account (`alo-accounts`), and an entry that
named them each time would be a record that tracked the person rather than the
agent — which ADR 0001 §4 is against. This is stated rather than assumed,
because a managed machine with several people is a later question and will need
one answered deliberately.

## Verification

Windows 11, `C:\dev\alo-os-claude`, all three gates run from the checkout before
this was handed over:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets --workspace -- -D warnings` — clean, zero
  warnings.
- `cargo test --workspace` — passing, including the seven new acceptance tests
  and thirty-three new unit tests and four doctests in `alo-approving`.

Executed, not pending. What is **not** claimed: nothing here was run on a
certified machine, nothing draws a pixel, and no *On the machine* box moves.
`ROADMAP.md`'s exit gate is about a cold boot on real hardware, and a green
suite on a developer's laptop is not that.

## Limitations

- **It does not draw.** Honouring the request is the compositor's, which is
  `alo-shell` and another worker's chain. This crate fixes what a shell must not
  be free to decide differently and stops there.
- **It shows one question at a time.** A list of several waiting changes is
  `Turning::waiting_at`'s and a different surface; nothing here prevents one
  being built on top.
- **Nothing reaches this over `alo-protocol` yet.** The daemon wiring — a shell
  asking the daemon for the change and sending the answer back — is not part of
  this task and is not claimed.

## Proposed shared-document updates

For the integration owner; not edited here, per `SHARED_MAIN.md`.

**`CHANGELOG.md`**, under unreleased:

> **The change an agent proposes is now put to a person.** alo OS could decide
> that a change needs approval and word it in one sentence; nothing had ever
> shown one. A proposed change now reaches a screen as the sentence the machine
> generated from the arguments it checked — never a description a model wrote —
> with the agent it belongs to. Approving it carries it out exactly once, saying
> no is a whole answer and nothing asks why, a question that stood too long is
> refused with the change quoted so it can be asked for again, and a machine
> with nowhere to put the question says so instead of staying silent.

**`ROADMAP.md`:** v0.01's exit gate step *approve the sentence, see it happen*
now has executable evidence off hardware
(`crates/alo-approving/tests/one_approval_and_the_sentence_a_person_approves.rs`).
No box moves: the gate is a cold boot on a certified machine.

**`docs/autonomy/QUEUE.md` / `STATE.md`:** task 7 of
`v0-01-delivery-plan.md` is done and marked in the plan in this change. Task 8,
*afterwards, ask what it did*, was already written and is the next one.
