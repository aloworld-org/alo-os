# Every sentence about documents and paper, and the walk through them

**Date:** 2026-09-17
**Workstream:** v0.5 documents and paper
(`docs/autonomy/v0-5-documents-and-paper-plan.md`, task 5)
**Contributor:** Claude (worker in `C:\dev\alo-os-shell`)
**Status:** ready for integration

## What changed

The first four tasks of the plan each end in sentences a person reads at the
moment they are already frustrated. Each crate's tests held its own sentences one
at a time. This task holds them **together**: the order a person meets them in,
the notes a translator is handed with them, and the source of all three crates
for English written anywhere else.

- **One walk, through the real code.**
  `crates/alo-converting/tests/the_walk_through_documents_and_paper.rs` follows
  one person's afternoon. A PDF arrives. One of the owner's real Word documents
  arrives. The agent asks to convert it, and the pinned engine converts it
  through the real `alo-convertd`. A printer on the network is found and set up.
  The agent asks to print the copy, and the indicator shows it leaving. The
  printer runs out of paper, and is ready again. A Word document that was cut
  short arrives. Every sentence the person meets is collected in order and
  compared with the table below, **read out of this report**. So no sentence
  along the walk can change without this table changing.
- **Every sentence is the machine's, with a note that says something.**
  `crates/alo-converting/tests/every_sentence_about_documents_and_paper_carries_a_note.rs`
  holds three things. What `alo-opening`, `alo-converting` and `alo-printing`
  declare is exactly what `alo-saying` collects under their areas. Every note
  has at least eight words. And every gap in a sentence (`{printer}`, `{what}`,
  `{file}` …) is named in its note, so a translator knows what will stand there.
  This check found **four notes that did not say what `{printer}` holds**. All
  four are fixed.
- **No English outside the vocabulary.**
  `crates/alo-converting/tests/no_english_outside_the_vocabulary_in_documents_and_paper.rs`
  reads the shipped `src/` of all three crates for English outside their
  `words.rs`. It uses the same rules as `alo-choosing`'s search for the settings
  crates (`tests/reading_source/mod.rs`). It found 20 literals, and none of them
  reaches a person. Each is on an exception list with its argument, and an
  exception the source no longer needs fails the test:
  - the converting service's standard error, kept in its systemd log;
  - the `Display` of the service's internal reasons, which `serving.rs` discards
    with `map_err(|_| …)` for a `Refusal` that is said in words;
  - a line of the wire protocol between the verb and the service;
  - the HTTP request head sent to the printing service.
- **One sentence that was not true, corrected.** When a print was refused
  because the printer was out of paper (or ink, or not answering), a person read
  *"Put paper in its tray, and what was waiting prints once it has some"*. But
  the printer had not taken the document, so nothing was waiting, and the person
  would have waited for a page that never came.
  - The out-of-paper and out-of-ink sentences now say only what is true on both
    paths: *documents it has already taken print once it has some*.
  - `NotPrinted::said` adds one new sentence after a stop:
    `printing.not-taken`, *"The printer did not take this document, so it will
    not print by itself. Once that is put right, print it again"*.
  - A refused document gets no extra sentence, because it already says that
    sending it again changes nothing. A new test,
    `a_refused_document_is_not_told_to_be_printed_again`, holds that.

  This is the plan's constraint working as intended: *if it reads well and is
  not true it changes*. It changes no decision: which stop is which, and when
  a document is refused, are exactly as they were.

User-readable change description: *When a printer stops and does not take the
document you were printing, alo OS now tells you to print it again once the
problem is fixed. Before, it said the document would come out by itself, and it
would not have. Every sentence about opening, converting and printing documents
is now checked for a translator's note that explains every blank in it.*

## The walk, sentence by sentence

The walk runs under a temporary folder. In the approval sentences that folder
is written as `/home/anna`, which is the only substitution the test makes.

| Step | Moment | What a person reads |
|---|---|---|
| 1 | A PDF arrives | This is a PDF document, and this machine opens it as it is |
| 2 | A Word document arrives | This is a Word document, and this machine opens it by converting a copy into a PDF document. The original stays exactly as it was, and what the copy could not carry over is said once it has been converted |
| 3 | The agent asks to convert it | convert /home/anna/Downloads/report.docx into a PDF copy in /home/anna/Documents, leaving the original as it is |
| 4 | It is converted | report.pdf is a PDF copy of report.docx, made on this machine. report.docx is exactly as it was |
| 5 | It is converted | The copy does not carry everything in the original. This is what is different |
| 6 | It is converted | The font Garamond is not on this machine, so the text set in it is shown in a similar font instead, and lines and pages can break in different places |
| 7 | It is converted | The original has a date that changes by itself when the document is opened. The copy shows it as it was at the moment it was converted, and it will not change |
| 8 | It is converted | The original shows a picture kept somewhere else rather than inside the document. It was not fetched, so the copy does not have it. Whoever sent the document can send the picture itself |
| 9 | It is converted | The original has comments. The copy does not show them; open the original on a machine with an office application to read them |
| 10 | The list of printers is opened | Brother HL-L2350DW series, on your network |
| 11 | The person chooses it | Set up Brother HL-L2350DW series, so this machine can print on it |
| 12 | It is set up | Brother HL-L2350DW series is set up, and this machine prints on it |
| 13 | The agent asks to print the copy | print /home/anna/Documents/report.pdf on this machine's printer |
| 14 | The indicator shows it leaving | @documents is sending something to 192.168.1.20 |
| 15 | It is printed | The document has been sent to the printer |
| 16 | The printer stops | The printer is out of paper. Put paper in its tray, and documents it has already taken print once it has some |
| 17 | Paper goes in, and the printer is asked again | The printer is ready |
| 18 | A damaged Word document arrives | This file is damaged: it is a compressed file, but it has been cut short or does not hold together inside, so it cannot be opened as it is |
| 19 | A damaged Word document arrives | What would open is a complete copy, which whoever has the original can send again |

Rows 3, 13 and 14 are sentences from `alo-capability` and `alo-egress`. They are
here because a person meets them in this sequence, and they are held here in the
same way. None of them was changed.

## Decisions

- **The test reads this report's table, not a copy of it.** The plan asks for
  the table to be *recorded verbatim in the report* and held by a test that fails
  when a sentence changes without the table. Parsing the report is the only way
  both are literally true. A published report is never rewritten. So a later
  change that moves a sentence along the walk publishes the table again in a
  follow-up report and points `THE_REPORT` in the test at it. The test's own doc
  comment says so.
- **The walk lives in `alo-converting`'s tests.** It needs all three crates.
  `alo-converting` and `alo-printing` each depend on `alo-opening` and not on
  each other, so one of them has to take the other as a **dev-dependency**. The
  walk's longest stretch is converting, so it went there. `alo-printing` and
  `alo-egress` are dev-dependencies only, and converting still never prints.
  The printing fixture (`alo-printing/tests/serving/mod.rs`) is included by
  `#[path]` rather than copied, so the walk's printer speaks exactly the
  protocol the printing tests watch.
- **Converting is real; the printer is a loopback service.** Step 4 runs the
  pinned engine through `alo-convertd` against the owner's `sample.docx` and does
  not skip itself where the engine is missing (ADR 0039). The printer is the same
  loopback printing service that `alo-printing`'s tests use. A real printer is
  still owed (task 3).
- **The walk follows what the printing service does.** A printer that runs out
  of paper still takes documents into its queue. So in the walk the document is
  sent, and then the printer stops. The other path is a print refused while the
  printer is stopped, which is where the untrue sentence was. That path is held
  by `alo-printing`'s own test, with both of its sentences written out.
- **The scan duplicates the reader from `alo-choosing` instead of depending on
  it.** A test file cannot be a dependency. The rules must mean the same thing in
  both searches, so the reader was carried over unchanged, with one repair:
  `\r` and `\0` in a literal are now read as themselves rather than as `r` and
  `0`. It is a module of its own because *what a literal is* and *which literals
  are allowed* change for different reasons.
- **The note rule is mechanical and small:** at least eight words, and every gap
  named as `{gap}`. It found four real omissions and nothing else, so no note
  was padded to pass it.
- **Row 18 says *a compressed file*.** That is what a Word document cut in half
  can be shown to be from its bytes. Saying *a Word document* would be the name
  deciding, which task 1 forbids. It reads acceptably, and it is true.

## Acceptance criteria

| Criterion | Held by |
|---|---|
| Every sentence these crates can say is in the vocabulary | `every_sentence_the_three_crates_declare_is_the_machines` |
| …with a translator's note | `every_sentence_the_three_crates_say_carries_a_note_naming_its_gaps`, `a_note_that_says_nothing_or_leaves_a_gap_unexplained_is_found` |
| The walk produces the exact sequence, recorded here as a table, held by one test | `the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table`, `only_the_walks_own_table_is_read_and_a_changed_sentence_is_a_difference` |
| No English outside `alo-strings` in the shipped source | `the_three_crates_write_no_english_outside_the_vocabulary`, `every_exception_says_why_nobody_reads_it`, `the_search_finds_every_crates_declared_vocabulary`, `the_search_reads_every_shipped_file_and_no_test_module`, `a_sentence_outside_the_vocabulary_is_found_however_it_is_spelt`, `what_is_not_a_sentence_on_a_screen_is_not_found` |
| A sentence that is not true changes | `a_printer_that_does_not_take_the_document_says_which_thing_is_wrong`, `a_refused_document_is_not_told_to_be_printed_again` |

No approval was needed: no grant, gate, ADR or promise in `docs/features.md`
moved.

## Verification

All commands ran on the WSL Ubuntu gate distribution with `CARGO_TARGET_DIR` set
to `$HOME/alo-builds/alo-os-shell-cd217193b5311c25`. The pinned engine
(`/opt/libreoffice26.2`) was installed there by task 2.

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy -p alo-converting -p alo-printing --all-targets -- -D warnings`:
  exit 0.
- `cargo test -p alo-converting -p alo-printing -p alo-saying --no-fail-fast`:
  exit 0.
  - `alo-converting`: 58 unit tests, then 2
    `a_file_that_cannot_be_opened_is_recorded`, 9 `converting_a_real_document`,
    3 `converting_reaches_nothing`, 3
    `every_sentence_about_documents_and_paper_carries_a_note`, 6
    `no_english_outside_the_vocabulary_in_documents_and_paper`, and 2
    `the_walk_through_documents_and_paper` (through the real service and
    engine).
  - `alo-printing`: 33 unit tests, then 5 `a_printer_that_stopped`, 7
    `finding_printers`, 9 `printing_a_document`, and 2
    `printing_reaches_only_this_machines_printing_service`.
  - `alo-saying`: 63 unit tests, 4 `what_this_machine_can_say`, and its
    doctests. It collects the new word.
- Each evidence test was also run on its own with `--exact`, and each passed.
- Before the fixes, the new tests failed for real. The note test named the four
  `{printer}` notes, the source scan named the 20 literals now argued as
  exceptions, and the walk reported its sequence against an empty table.
- Not run, as instructed: the full workspace suite. `alo-opening` is not edited.

## Limitations

- The printer in the walk is a loopback service that speaks the protocol. A real
  printer on a certified machine is still owed (task 3).
- The image has not been built or booted (task 2).
- The approval sentences (rows 3 and 13) begin in lowercase, which is how
  `alo-capability` writes every verb's sentence for the approval surface to put
  inside its own. That belongs to that crate and was left as it is.
- The open-with portal still says only the reason a file cannot be opened
  (task 4's report proposes the change to `alo-applications`).

## Proposed shared-document updates

- **CHANGELOG.md:** the user-readable description above.
- **ROADMAP.md:** nothing ticks. `.docx`, `.xlsx`, `.pptx` *open*, ★ *Printers,
  solved* and ★ *"I can't open this file"* each remain `- [x] The code.` until
  they run on the machine.
- **QUEUE.md / STATE.md:** documents and paper, task 5 done. The plan now names
  task 6: `.pages`, `.heic` and `.dwg` files recognised, and converted or
  explained, as `docs/features.md` promises.

## The second attempt: a decision pointer that did not land

The supervisor refused the first handover on `the workspace's tests`, twice:
`alo-citing`'s `every_decision_this_repository_points_at_exists` found that
ADR 0047 — written by the adapting work in `525bac3`, not by this task — linked
ADR 0014 under a file name that decision no longer has, calling it *our own
hosted model* where the file on disk says *alo's own model*. The link now names
the file that exists, which is the whole change; ADR 0047's decision is
untouched.

The dead spelling is deliberately not written out here. The check reads every
decision this repository points at, and it cannot tell a link from a quotation
of a broken one — so a report that spelled the old name would fail the very
gate it is describing. That is the check being right rather than blunt: a name
that appears nowhere cannot be followed by accident.

**For the integration owner:** the adapting lane made the same correction on
`main` in `370f0d7` while this task was gating, covering ADR 0047 and ADR 0048
together. The duplicate was dropped from this tree on rebase rather than
resolved by hand, so nothing here re-applies it.

The gates were run as the supervisor runs them, in WSL Ubuntu (clippy on the
Windows host reports `alo-converting`'s Linux-only modules as dead code, which
is the platform, not this change): `cargo fmt --all --check`, `cargo clippy
--workspace --all-targets -- -D warnings` (exit 0), and `cargo test -p
alo-citing -p alo-converting -p alo-printing`, all passing.
