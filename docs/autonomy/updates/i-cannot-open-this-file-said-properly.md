# "I can't open this file", said properly

**Date:** 2026-09-17
**Workstream:** v0.5 — documents and paper (`docs/autonomy/v0-5-documents-and-paper-plan.md`, task 4)
**Contributor:** Claude Code worker in `C:\dev\alo-os-shell`, for the repository owner
**Status:** ready for integration

## What changed, for a person

When alo OS cannot open a file, it no longer stops at the reason. It says what
the file is, why this machine cannot open it, and **what would**:

| The file | What a person reads |
|---|---|
| Empty | *This file is empty: there is nothing in it to open.* *What would open is a complete copy, which whoever has the original can send again.* |
| Damaged (a download cut short) | *This file is damaged: it is a compressed file, but it has been cut short or does not hold together inside, so it cannot be opened as it is.* *What would open is a complete copy, which whoever has the original can send again.* |
| Not recognised | *This machine does not recognise what this file is, so it has not opened it.* *Whoever sent it can say which program made it, or send a copy saved in a different format.* |
| A program | *This file is a program. Opening a file never runs a program, so it has not been opened.* *If a document was expected, whoever sent it can send the document itself instead.* |
| Locked with a password | *This is an Office document protected with a password, and this machine cannot open a document protected that way.* *What would open is a copy saved without the password, which whoever sent it can make.* |
| A kind nothing here opens | *This is a PowerPoint presentation in the older format, and nothing on this machine opens it or converts it into something that does.* *What would open it is a machine with a program for this kind of file, or a copy saved in a different format by whoever sent it.* |

A damaged file sends a person back for a complete copy. A file the machine
has nothing for sends them on to another machine or another format. Those are
opposite directions, and they are now different sentences, each with its own
next step. No sentence names a program library, a file type code, an error
number or anything alo OS rents. None of them offers to upload the file or look
it up anywhere: a file this machine cannot open stays on this machine.

When an agent is asked to convert a document that cannot be opened, the
explanation is what the machine's record keeps, word for word. So *what
happened when I tried to open that?* can be answered afterwards from the record,
by the file's own path.

## What changed, in the code

**`crates/alo-opening`**
- `src/would.rs` (new): `Would`, a closed set of five answers: `ACompleteCopy`,
  `ACopyWithoutThePassword`, `TheDocumentItself`, `AnotherMachineOrFormat`,
  `WhoeverMadeIt`. Each has a `word()` and a `said()`, plus `Would::EVERY`.
- `src/outcome.rs`: `Cannot::would()` maps each reason to one `Would`.
  `Cannot::explained(strings)` returns `[said(), would().said()]`.
  `Outcome::said` for `CannotOpen` now returns the explanation, not only the
  reason. `Cannot::said` is unchanged and is still the first sentence.
- `src/words.rs`: five new words in a fourth group, `THE_REMEDIES`, each with a
  translator's note. `EVERY_WORD` now has 40 words. New tests check that no
  word offers to send the file anywhere (upload, online, cloud, service, search
  and so on). The machinery test now also refuses *library*, *LibreOffice*,
  *return*, *exit status* and *CUPS*. The group, gap and capital-letter tests
  cover the new group.
- `src/lib.rs`: `pub mod would`, `pub use would::Would`, and the crate docs say
  a file is explained and never sent anywhere to find out.
- `tests/a_file_this_machine_cannot_open_is_explained.rs` (new): one test per
  reason. Each writes a real file to disk and decides it through `decide`
  against the machine's collected vocabulary (`alo-saying`). Each asserts the
  exact sentences, the `Would`, and that no sentence is a bug, has an empty
  gap, names machinery, hedges or offers an upload. Damaged is checked in all
  three storages, next to the same whole PDF on a machine with nothing for it.
  One more test covers a lying name: the finding comes first, then the whole
  explanation.
- `src/decided.rs` and `tests/what_this_machine_can_do_with_a_file.rs`: the
  expected sentences for a program now include what would open it.

**`crates/alo-converting`**
- `src/converting.rs`: `NotConverted::NotWhatItsNameSays` used to cut what it
  said down to the finding. Now, when the decided outcome is `CannotOpen`, it
  keeps the whole explanation. When the file would open as something else,
  it still keeps only the finding, because *this machine opens it* is not true
  of a conversion that never happened. New unit test:
  `a_document_that_cannot_be_opened_is_explained_before_nothing_was_converted`.
- `tests/a_file_that_cannot_be_opened_is_recorded.rs` (new, Linux): the whole
  verb runs with no converting service: declared, proposed, approved once,
  redeemed, resolved, carried out. It covers an empty file, a damaged one, a
  program and an unrecognised one. Each entry is kept in an `alo_record::Record`
  and found again with `Asking` plus `What::touched(path)`. Its `told` lines
  must equal what the person read. Nothing may be created, and the original
  must be byte-for-byte the same. A second test covers a lying name.

`alo-printing` gets the explanation through `Outcome::said` with no change of
its own. Its tests pass unchanged.

## Decisions, and why

1. **The way out is a closed type, not prose.** `Would` is an enum, so which
   direction a reason sends a person is something a test can check and a shell
   can draw from. *Damaged and not recognised send a person in opposite
   directions* is now `Cannot::Damaged(_).would() != Cannot::Unrecognised.would()`
   rather than a hope about wording.
2. **Two sentences, not one longer one.** The reason sentences already name
   what the file is and why, and other crates already use them. Adding the way
   out as its own word gives translators two complete thoughts, each with a
   note, and changes no existing key or its meaning. `Cannot::said` keeps its
   signature, so `alo-applications` and `alo-portals` still compile and read
   the same.
3. **`Outcome::said` says the whole explanation.** Every caller that shows an
   outcome — converting, printing, `Decided` — now shows the way out without
   having to remember to. The alternative, a new method each caller has to opt
   into, is how a way out gets dropped.
4. **Not recognised goes to *whoever made it*, not to *another machine*.** This
   machine does not know what the file is, so it cannot honestly say another
   machine would open it. That would be a guess, and the plan forbids guessing.
   The sender knows what program made it.
5. **Recording goes through the verb that opens a file, not through
   `alo-opening`.** `alo-opening` depends on `alo-strings` and `thiserror` and
   nothing else. `tests/deciding_never_leaves_the_machine.rs` holds that,
   because deciding what a file is must never become an errand. The record
   crate depends on `alo-egress`. So `alo-opening` produces the explanation as
   values and sentences. The converting verb, which really opens a file under a
   grant, records it the way it records every conversion
   (`Entry::ran(..).telling(..)`, ADR 0039 §6). No new record type or field was
   needed, and `alo-record` is not edited.
6. **A lying name no longer hides the way out in converting.** A program named
   `invoice.docx` used to be recorded as *named as a Word document, but it is a
   program* and *nothing was converted*, with nothing about what to do next.

## Acceptance, and the test behind each clause

| Clause | Test |
|---|---|
| A sentence naming what it is, why, and what would, with a test per reason: empty | `alo-opening` `a_file_this_machine_cannot_open_is_explained::an_empty_file_is_explained` |
| … not recognised | `a_file_this_machine_cannot_open_is_explained::a_file_nothing_recognises_is_explained` |
| … a program | `a_file_this_machine_cannot_open_is_explained::a_program_is_explained` |
| … locked with a password | `a_file_this_machine_cannot_open_is_explained::a_document_locked_with_a_password_is_explained` |
| … nothing here opens it | `a_file_this_machine_cannot_open_is_explained::a_kind_nothing_here_opens_is_explained` |
| Damaged is said to be damaged rather than unsupported | `a_file_this_machine_cannot_open_is_explained::a_damaged_file_is_explained_as_damaged_not_unsupported`, `outcome::tests::damaged_and_unrecognised_are_answered_differently` |
| A lying name does not cut the explanation short | `a_file_this_machine_cannot_open_is_explained::a_file_named_as_what_it_is_not_is_still_explained`, `alo-converting` `converting::tests::a_document_that_cannot_be_opened_is_explained_before_nothing_was_converted` |
| Never names a library, a media type, a return code or a rented engine | `alo-opening` `words::tests::nothing_a_person_reads_names_the_machinery` |
| Nothing offers to send the file anywhere to find out | `alo-opening` `words::tests::nothing_offers_to_send_the_file_anywhere` |
| No guessing | `alo-opening` `words::tests::nothing_here_hedges` (now covers the new words) |
| The refusal is recorded like any other outcome, answerable afterwards | `alo-converting` `a_file_that_cannot_be_opened_is_recorded::each_reason_is_recorded_with_what_the_person_was_told`, `a_file_that_cannot_be_opened_is_recorded::a_name_that_lies_is_recorded_with_the_explanation_after_it` |
| Every sentence is in the collected vocabulary with a note | `alo-opening` `words::tests::every_word_carries_a_note_for_the_translator`, `what_this_machine_can_do_with_a_file::every_outcome_carries_a_sentence_from_the_machines_vocabulary` |

## Verification

All commands ran on the WSL Ubuntu gate distribution (kernel
6.18.33.2-microsoft-standard-WSL2, rustc 1.98.0), with `CARGO_TARGET_DIR` set to
the loop's `$HOME/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all`: clean.
- `cargo clippy -p alo-opening -p alo-converting --all-targets -- -D warnings`
  and the workspace-wide `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo test -p alo-opening`: 42 unit tests, 6 `a_file_that_does_not_hold_together`,
  7 `a_file_this_machine_cannot_open_is_explained`, 5
  `converting_waits_on_its_decision`, 2 `deciding_never_leaves_the_machine`,
  6 `deciding_reads_only_what_it_needs`, 4 `what_this_machine_can_do_with_a_file`,
  and 1 doctest. All pass.
- `cargo test -p alo-converting`: 58 unit tests, 2
  `a_file_that_cannot_be_opened_is_recorded`, 9 `converting_a_real_document`
  (through the real service and engine), 3 `converting_reaches_nothing`. All
  pass.
- `cargo test -p alo-printing`: all pass. This crate is not edited, but it
  reads `Outcome::said`.
- `cargo test -p alo-saying`: all pass. It collects the five new words.
- `cargo test -p alo-applications` and `cargo test -p alo-portals`: all pass.
  Neither is edited; both read `Cannot::said`, whose sentence is unchanged.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-opening -p alo-converting`:
  no warnings.
- Not run, as instructed: the full workspace suite.

### A second attempt, and what it changed

The first handoff of this task was refused by the workspace suite, and the
refusal excerpt named no failing test. The same run then failed to park with
*out of disk space* on `C:`. So the likely cause was the machine, not this
code. On 2026-09-17 every gate above was run again in the foreground on the
same tree, and all passed. One test was tightened while reviewing:
`each_reason_is_recorded_with_what_the_person_was_told` had built its expected
last line (*nothing was converted*) by copying the last line it received, which
could not fail. It now compares against
`alo_converting::words::NOTHING_WAS_CONVERTED` itself.

## Remaining limitations, and what is owed

- **The open-with portal still says only the reason.** `alo-applications`'
  `NothingOpens::TheFile(cannot).said` returns one `Said` (`cannot.said`), so an
  application asking what opens a program gets the reason and not the way out.
  In `alo-portals`, the answers file keeps `nothing-opens` / `the-file` without
  saying which reason. Both belong to the applications plan, not this one.
  **Proposed:** `NothingOpens::said` returns `Vec<Said>` built from
  `Cannot::explained`. `NothingOpensAs::TheFile` gains an additive, optional
  `reason` kept by identity, with the answers-file contract updated to match.
- **A person opening a file in the shell is not an agent's execution**, so
  nothing writes it to the agent record. The shell's window, and whatever
  record a person's own actions are kept in, belong to the shell plan.
- **No certified hardware.** WSL is development evidence only.

## Proposed updates to shared documents

- **CHANGELOG.md:** *When a file cannot be opened, alo OS now also says what
  would open it: a complete copy from whoever has the original, a copy without
  the password, the document itself instead of a program, another machine or a
  different format, or the sender saying what made it. A damaged file and one
  the machine does not recognise send you in different directions. Nothing
  offers to upload the file, and when an agent was asked to convert it, the
  record keeps exactly what you were told.*
- **ROADMAP.md (v0.5):** ★ *"I can't open this file" — converted, or plainly
  explained*: `- [x] The code.` Not ticked on the machine: the shell's window
  and the open-with portal still owe the way out (above).
- **QUEUE.md / STATE.md:** documents and paper task 4 is done, and this report
  is its evidence. Task 5 (every sentence, and the walk through them) is next.
  Its sentences for this step are `alo_opening::words::THE_REMEDIES`.
