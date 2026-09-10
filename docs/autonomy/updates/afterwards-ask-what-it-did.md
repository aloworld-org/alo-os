# Afterwards, ask what it did

**Date:** 2026-09-10
**Workstream:** v0.01 delivery plan, task 8 (`docs/autonomy/v0-01-delivery-plan.md`)
**Contributor:** Claude, in `C:\dev\alo-os-claude`
**Status:** ready for integration

## What this is

`ROADMAP.md`'s v0.01 exit gate ends *ask what it did and get an answer from the
record*. `alo-record` has held every execution and every refusal since the
beginning — ADR 0001 §7 as working code — and `alo-keeping` has written it to a
disk and read it back. **Nothing read it back to a person.**

`crates/alo-recounting` is that surface's model. It is one crate, six files, and
it draws nothing:

| | |
|---|---|
| `recounting.rs` | `Recounting` — the record on this machine, asked, and read off the disk every time |
| `account.rs` | `Account` — what the record answers, and what the record says about itself |
| `told.rs` | `Told` and `Outcome` — one entry as a person reads it |
| `surface.rs` | `Compositor` — the port whatever owns the screen implements |
| `refusing.rs` | `NotRecounted` — every way the question goes unanswered |
| `words.rs` | Thirteen strings, under a new `recounting` area |

## The acceptance, and how each criterion is met by a shape rather than a promise

**A person asks and is answered from the record on the disk, not from memory of
the session.** `Recounting` holds a `PathBuf` and nothing else. There is no
record in it, no entries, no account from last time, and no constructor that
takes any of those — the only one is `Recounting::kept_at(&Path)`. Every
question goes through `alo_keeping::Reading::at`, which opens the file. That is
not a lazy performance decision; it is the guarantee, because the two things
that would differ between a remembered answer and the file are exactly the two
that matter: an entry the daemon failed to write, and a record somebody has
since shortened or deleted. A surface that answered from memory would show the
version nothing can be checked against.

The acceptance test proves it the only way it can be proved: the turn is run,
the `Writing` is dropped, the session ends, and only then is a `Recounting` made
— from a path. Every sentence it answers with is then found in the bytes of the
file it claims to have read. And a second entry appended after the first answer
appears in the second answer, which is only true of something that reads.

**A turn that was refused reads back as refused.** `Outcome` is ten values
derived from `alo_record::Happened` in one exhaustive match, and the three ways
of being stopped stay three: *nobody was asked*, *the person said no*, *the
grants said no at the moment it would have happened*. Each has its own clause;
none of them reads as something that ran. A kind of entry added to the record
and not given a clause here is a build that fails, not a blank line somebody
reads.

**Nothing in the answer is a sentence a model wrote.** `Told::of` takes an
`alo_record::Entry` and there is no other door — no constructor from text, no
public field, no `From`, no deserialiser, and two compile-fail examples (E0599
and E0616) that turn adding one into a failing build. `Told::of` also takes no
`Strings`, which is the second half of the argument: everything on the type is
*already* text, worded by whoever decided it when it happened, so this crate
cannot re-word a person's own record even by accident.

## Decisions I made, and why

**The crate is named `alo-recounting`.** To recount is to tell what happened,
and the gerund matches the surfaces beside it (`alo-approving`, `alo-picking`).
`alo-record` and `alo-keeping` were taken by the two crates it reads.

**What a model wrote is quoted, but it is never a sentence.** A verb name that
never became a call is text the model was persuaded to send, and `alo-record`
keeps it because *what did it try* is the question a security review actually
asks. It comes back from `Told::asked_for` alone; `Told::sentence` answers
`None`, because nothing was validated to generate a sentence from. The
acceptance test sends `"Archived every invoice for you\u{1b}[2K"` as a verb name
— a string shaped like a plausible account of an afternoon, with a control
character that would clear a terminal line — and asserts it never becomes the
sentence, that no control character survives anywhere, and that this crate's own
strings never quote it.

It is quoted a second time, inside `alo-capability`'s *there is no verb called
`…`*, and that is correct rather than a leak: the sentence around it is the
machine's, written before anything was asked of it, and what is quoted has been
through `alo_record::Line`. The test asserts exactly two quotations and names
both. This is written into the rustdoc on `Told::because` so the next reader
meets the argument rather than the exception.

**This crate has no opinion about what counts as a refusal.** `Told` has no
`was_refused`. *What was it stopped from doing* is
`alo_record::Only::Refusals`, asked of the record, and a second definition here
would be a machine that answers one question two ways. That decision has a
consequence the acceptance test records: the record counts *a person said no* as
a refusal, so a question about refusals answers with all three. Which of them it
was is the clause on the line.

**An account always says what the record is.** `Account::said` answers with the
record's own sentence about whether it goes all the way back — `alo-keeping`'s
`Head::said`, in both its cases — beside whatever else it has to say, and with
`alo-keeping`'s damage sentences after it. *Nothing in this machine's record
answers that question* alone would be read as *the agent did nothing*, and on a
shortened record that is false. Showing the *whole* case too is deliberate: the
positive claim is what makes the negative one worth anything.

**Three sentences, never one with the others stuck on the end.** `said` answers
`Vec<Said>`, following `alo-shortcuts` (item 9c) and `alo-keeping`: the join
between two sentences is not punctuation a program can pick for a language it
does not know.

**Only three strings here are new sentences about a record.** The ten clauses
say what became of one entry, one remark says nothing answered the question, and
two refusals say there is nowhere to show it. Everything else — the sentence
describing a change, why something was refused, where it went, whether the
record is whole — arrives already worded by the crate that decided it. The unit
test `the_only_sentences_this_crate_says_of_its_own_are_its_own_two` holds that
line by loading only this crate's vocabulary and counting what still reads.

**Nowhere to show it is answered before the disk is read.** `Recounting::show`
asks the compositor first, as `alo-approving` does, so a machine with no screen
does not read a year of evidence to find that out.

**There is no verb for this and there cannot be one.** An agent able to read the
record is an agent able to learn what it has already been refused and shape the
next attempt around it. `alo-protocol` reaches nothing here, and ADR 0001 §4's
*context is offered, never watched* is the reason it is not reachable from a
turn either.

## A registration that would have failed later

`alo-saying` did not collect this crate, so the crate-level doc example failed
on `Said::is_a_bug` — a shell would have shown `«recounting.outcome.ran»` where
a clause belongs. `crates/alo-saying/Cargo.toml` and
`crates/alo-saying/src/collecting.rs` now carry it (twenty-one lists, one string
named from each, and the sum test that says nothing was lost or shared).

## Verification

Windows 11, `C:\dev\alo-os-claude`, run before the handoff was written:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings. It caught
  one unfulfilled lint expectation in `told.rs` (a `#[expect(unwrap_used)]` over
  a test module that never unwraps), which was removed rather than allowed.
- `cargo test --workspace` — passes.
- `cargo test -p alo-recounting` — 42 unit tests, 5 acceptance tests, 3 doc
  tests and 2 compile-fail examples.

The acceptance tests reach the machine the way a shell does: `alo_turn::Turning`
against the closed list of verbs `alo-files` declares, real files that really
move on the filesystem the tests run on, the record written by `alo-keeping` to
a file that is really there, and the strings from
`alo_saying::everything_this_machine_can_say`.

**This is not the hardware verification `CLAUDE.md` asks for.** It is whatever
the tests were run on. No box in *On the machine* moves, and no pixel is claimed
or tested — drawing an account is the compositor's, and the desktop worker's.

## Limitations, honestly

- **Nothing draws this yet.** The crate is the model and the port; `alo-shell`
  has not implemented `Compositor`. That is the same state task 7's approval
  surface and task 9's indicator are in, and it is deliberate: the compositor
  lane is the desktop worker's.
- **A moment is not worded.** `Told::at` answers a `SystemTime` and this crate
  writes no date format, for the reason `alo_approving::Asked::lapses_in` gives:
  how a date is written belongs to the reader's region, which is not the same
  thing as their language. When there is one way to word a moment it will be
  `alo-strings`', and every surface will use it.
- **How many is a number beside the account, never inside a sentence.** There is
  no plural in this crate's vocabulary, and a test keeps it that way.
- **The question language is `alo_record::Asking`.** There is no natural-language
  question here and this crate invents none. What turns *what did it do this
  afternoon* into an `Asking` is a shell's, or a later task's.

## Proposed changelog entry

> **Ask what the machine did, and be answered from its record.** The record alo
> OS keeps of every execution and every refusal can now be read back to the
> person whose machine it is. The answer is read off the disk each time it is
> asked, so it is what was written down rather than what this session
> remembers; a machine whose record is missing says so instead of showing an
> empty list, because a machine that has done nothing and a machine whose record
> was deleted are not the same thing. What was refused reads back as refused,
> and which of the three refusals it was — the machine's rules, the person's
> answer, or a permission taken back at the last moment — is on the line. Every
> sentence in the answer was generated by alo OS from arguments it had already
> checked; where an agent asked for something that does not exist, what it typed
> is quoted as what it asked for and never as a description of what happened.

## Proposed roadmap and queue updates

- v0.01 delivery plan task 8 is marked **Done, 2026-09-10** in the same change,
  with this report named. Task 9 was already written, so no new task was added.
- No `ROADMAP.md` exit-gate box moves. *Ask what it did and get an answer from
  the record* now has a model and a port behind it; the gate is about a
  certified machine, and this is not that.
- `docs/features.md` line *[v0.01] Every execution recorded with its origin,
  approval and grant* is unchanged and unnarrowed. Nothing here weakens it; this
  crate is the reading half of it.
- No contract moved. `docs/contracts/record-file.md` describes the file, which
  is untouched; no verb was added to `docs/contracts/agent-verbs.md`, and there
  could not be one.

## Files

- `crates/alo-recounting/Cargo.toml` — new
- `crates/alo-recounting/src/lib.rs` — new
- `crates/alo-recounting/src/recounting.rs` — new
- `crates/alo-recounting/src/account.rs` — new
- `crates/alo-recounting/src/told.rs` — new
- `crates/alo-recounting/src/surface.rs` — new
- `crates/alo-recounting/src/refusing.rs` — new
- `crates/alo-recounting/src/words.rs` — new
- `crates/alo-recounting/src/testing.rs` — new
- `crates/alo-recounting/tests/afterwards_ask_what_it_did.rs` — new
- `Cargo.toml` — the crate is a member
- `Cargo.lock` — the member's entry
- `crates/alo-saying/Cargo.toml` — collects this crate's words
- `crates/alo-saying/src/collecting.rs` — the same, and the three tests that
  count them
- `docs/autonomy/v0-01-delivery-plan.md` — task 8 marked done
- `docs/autonomy/updates/afterwards-ask-what-it-did.md` — this report
