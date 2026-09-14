# What this machine can do with a file, and what it cannot

**Date:** 2026-09-14
**Workstream:** v0.5 — documents and paper (`docs/autonomy/v0-5-documents-and-paper-plan.md`, task 1)
**Contributor:** Claude Code worker in `C:\dev\alo-os`, for the repository owner
**Status:** ready for integration

## What changed, for a person

Before alo OS opens, converts or prints a file somebody was sent, it can now say
honestly what it is able to do with it: open it as it is; open a converted copy
— and say that the original stays untouched and that anything the copy could not
carry over will be named; or not open it, with the reason. There is no
"maybe": the machine decides first and says what it decided.

What a file *is* is read from the file itself, never from the end of its name.
A file called `invoice.pdf` that is really a program is reported as exactly
that, before anything else is said, and it is never opened. A photograph saved
as `report.docx` is shown as the photograph it is — after the person has been
told the name was wrong. A download that stopped half way is called **damaged**
(ask for it again), which is a different sentence from **this machine does not
recognise what this file is** (try another machine or another format), because
the two send a person in opposite directions.

Deciding never sends anything anywhere and never reads more of a file than the
decision needs: a Word document's contents, a photograph's pixels and a
spreadsheet's cells are not read to decide what they are.

Nothing opens or converts yet — that is the next task. This is the decision
those tasks are built on, and every sentence it makes is in the translatable
vocabulary.

## What changed, in the code

**`crates/alo-opening`** (new)
- `src/deciding.rs` — `decide(file, name, machine) -> Result<Decided, Unreadable>`.
  Takes an **open** `Read + Seek` (whoever holds the grant opens it; this crate
  takes no path). `Unreadable` carries the `io::ErrorKind` for a log and its own
  sentence for a person.
- `src/decided.rs` — `Decided::AsItIs(Outcome)` or
  `Decided::NotWhatItsNameSays { named, outcome }`. The finding **wraps** the
  outcome, so no caller reaches an open window from a mismatched file without
  matching the mismatch first. `said()` gives the finding, then the outcome.
- `src/outcome.rs` — `Outcome::{OpensAsItIs { kind, macros }, Converts { from,
  into, costs }, CannotOpen(Cannot)}`; `Cannot::{Empty, Unrecognised,
  Damaged(Container), AProgram, PasswordProtected, NothingHereOpens(Kind)}`;
  `Costs` (macros left behind; the two costs true of every conversion are always
  said). Each carries its sentence.
- `src/kind.rs` — `Kind`, 18 kinds: PDF; Word, Excel, PowerPoint current and
  older; OpenDocument text, spreadsheet, presentation; rich text; Unicode text;
  text in an older character set; PNG, JPEG, GIF, WebP; zip archive.
- `src/appears.rs` — `Appears` (a kind, a program, a password-protected
  document, nothing, unrecognised, damaged), `Container` (what a damaged file is
  known to be: PDF, compressed, older Office storage), `Macros::{Inside, NoneSeen}`.
- `src/naming.rs` — `Named` and the claim table (29 extensions). Read only
  after the bytes decided; `agrees_with` accepts what a name honestly covers.
- `src/machine.rs` — `ThisMachine` built by whoever knows what is installed;
  `NotAnAbility` refuses a conversion into a kind not opened, from a kind already
  opened, a second conversion for a kind, opening a kind already converted, and
  a conversion into itself.
- `src/looking.rs` — the rules in order: empty → 512-byte head → signatures at
  offset zero → the one further read each signature needs → text.
- `src/zip.rs` — end record (with comment and zip64 forms), then the central
  directory a piece at a time; OOXML by `[Content_Types].xml` plus exactly one of
  `word/`, `xl/`, `ppt/`; OpenDocument by its stored `mimetype` in the head;
  macros by `vbaProject.bin` or a module under `Basic/`/`Scripts/`.
- `src/compound.rs` — older Office storage: header, allocation table (header
  DIFAT and DIFAT chain), directory chain, root's children only; `WordDocument`,
  `Workbook`/`Book`, `PowerPoint Document`, `EncryptedPackage`; macros by
  `Macros`/`_VBA_PROJECT_CUR`. Every loop is bounded by the file's length.
- `src/text.rs` — whole-file streaming text rules (UTF-16 with a mark, UTF-8,
  older 8-bit character set), stopping at the first byte that settles it.
- `src/reading.rs` — the only reader: a bounded piece at an offset.
- `src/words.rs` — 35 strings (22 names read inside sentences, 13 sentences),
  each with a translator's note.
- `tests/making/mod.rs` — builders for zips (stored entries, CRC-32, comment,
  zip64) and older Office storage files (v3, chained directory, loops, dangling
  tree, cut after the header), and a byte-counting file.
- `tests/what_this_machine_can_do_with_a_file.rs`,
  `tests/deciding_reads_only_what_it_needs.rs`,
  `tests/a_file_that_does_not_hold_together.rs`,
  `tests/deciding_never_leaves_the_machine.rs`.

**`crates/alo-saying`** — collects `alo-opening` (`Cargo.toml`,
`src/collecting.rs`: `EVERY_LIST` 32 → 33, `ONE_STRING_EACH`, the count test).

**Workspace** — `Cargo.toml` member, `Cargo.lock`.

**`docs/quirks.md`** — two entries under *Filesystems and paths*: a PDF's
`%%EOF` in the last kilobyte, and an OpenDocument's macro listings without a
macro.

**`docs/autonomy/v0-5-documents-and-paper-plan.md`** — task 1 marked done.

**`crates/alo-bounding/tests`** (second worker, see *Why the first handoff was
refused*) — `the_boundary_decides_and_forgets.rs` and
`the_kernel_refuses_an_attribute_change.rs` add `nodump` to the flags a file
has instead of replacing them with it. **`docs/quirks.md`** — a third entry,
*Setting a file's flags to exactly `nodump` asks ext4 to take its extents
away*, and the two earlier open entries about the same failure marked settled.

## Why the first handoff was refused

The supervisor's `cargo test --workspace` failed in
`alo-bounding::the_boundary_decides_and_forgets::ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`:
`EOPNOTSUPP` setting `nodump` on a file outside any turn. Nothing in this task
touches that crate; the test fails the same way on the untouched tree, and two
entries in `docs/quirks.md` had already recorded it as unexplained. It stopped
this task because it is the gate this task had to pass, so it was this task's
to settle.

**The cause, measured with no boundary loaded:** `FS_IOC_SETFLAGS` replaces a
file's flags. Every ext4 file laid out in extents carries `EXTENTS_FL`, so
`NODUMP` alone is a request to convert the file back to indirect blocks, which
ext4 refuses with `EOPNOTSUPP` for a file written and then cut short without an
`fsync` — exactly what the test does to it just before. `/tmp` on the gate
machine is ext4; the test was written when it was `tmpfs`, which has no extents.

**The fix, and why it weakens nothing:** both tests read the flags first and
set `had | NODUMP`, as `chattr +d` does; the ordinary-day test restores exactly
`had`. The assertions are unchanged — the flag must land outside a turn, and
must be refused on the private file inside one. The refusal is still the
`SETFLAGS` request's, because the `file_ioctl` hook answers `FS_IOC_GETFLAGS`
before it walks anything (the existing
`a_read_of_a_files_flags_is_not_walked` test). Deleting or ignoring the test
was not considered: the ordinary program *should* be able to set that flag,
and now it is asked to in the way one would.

## Decisions

1. **The mismatch wraps the outcome rather than replacing it.** The plan says a
   file that is not what its name claims is "reported as exactly that, and the
   mismatch is the finding rather than a thing to silently correct". Two readings
   were possible: refuse every mismatched file, or report the mismatch and still
   say what the file really is. Refusing would turn a photograph saved under the
   wrong name into "cannot open", which is false; opening it with a side note
   would be the silent correction. So `Decided::NotWhatItsNameSays` holds the
   outcome inside it — the type forces the finding to be taken apart first, and
   `said()` puts it first. A program under a document's name still ends in
   `Cannot::AProgram`, because what it really is decides that.
2. **What the machine has is handed in, not shipped.** No list in this crate
   claims a machine opens anything, because that is a fact about what is
   installed. Task 2 registers its conversions with `ThisMachine::converts`;
   the shell registers its viewers with `opens`. Contradictions are refused as
   values (`NotAnAbility`), with no English `Display`, so nothing a person could
   read is written outside the vocabulary.
3. **Programs, locked documents, empty files and unrecognised bytes are not
   `Kind`s.** A machine therefore cannot be told it opens one. A program is
   `Cannot::AProgram` — "opening a file never runs a program" — which is law 2
   applied to what a person is sent.
4. **Text reads to the end.** It is the one kind with no signature; "the first
   8 KiB were text" (the rule `alo-finding` uses for indexing) is a guess about
   the rest, and task 1 forbids a *probably*. The read streams in 64 KiB pieces
   and stops at the first byte that rules text out. Text in an older 8-bit
   character set is its own kind rather than *unrecognised*, because a CSV
   exported in Windows-1252 is the commonest such file a European person is sent;
   which character set it is, is not guessed.
5. **Macros are `Inside` or `NoneSeen`, never "none".** An older PowerPoint keeps
   its macros inside its one stream, which this crate does not read.
6. **A damaged file names only what is known** (`Container`): a zip cut short
   cannot be seen to be a Word document, so the sentence says "a compressed
   file". A name that claims any zip-stored kind agrees with a damaged zip, so a
   truncated `.docx` is not given a second, false finding.
7. **Signatures at offset zero only.** A PDF with bytes before `%PDF-` is
   *unrecognised* rather than risk calling a letter about PDFs a damaged
   document. Recorded in `docs/quirks.md`.
8. **A password-protected document cannot be opened in v0.5.** There is no
   password flow in the plan; saying so is true today, and task 4 can add what
   would.
9. **Not a public contract yet.** Nothing here is an agent verb, D-Bus
   interface or config key, so `docs/contracts/` is unchanged. Converting
   becomes an `alo_capability` verb in task 2, and the contract moves then.

## Acceptance criteria and where each is shown

| Criterion (plan, task 1) | Test |
|---|---|
| What a file is comes from its content, not its name | `what_this_machine_can_do_with_a_file::what_a_file_is_comes_from_its_bytes_and_never_its_name` |
| A closed set: opens as it is / converts and says what it costs / cannot — no *probably* | `what_this_machine_can_do_with_a_file::every_file_lands_in_exactly_one_of_three_outcomes` |
| A file not what its name claims is reported as exactly that, not silently corrected | `what_this_machine_can_do_with_a_file::a_file_that_is_not_what_its_name_says_is_that_finding_first` |
| Each outcome carries a sentence in the vocabulary `alo-saying` gathers | `what_this_machine_can_do_with_a_file::every_outcome_carries_a_sentence_from_the_machines_vocabulary` |
| Constraint: reads no more of a file than deciding requires | `deciding_reads_only_what_it_needs::a_document_is_decided_without_reading_its_contents` |
| Constraint: no outcome reached by asking anything off this machine | `deciding_never_leaves_the_machine::nothing_under_this_crate_can_reach_the_network` |
| Refusal path: damaged is said as damaged, not unsupported | `a_file_that_does_not_hold_together::a_damaged_file_is_said_to_be_damaged_not_unrecognised` |

Refusal paths beside the legitimate ones: every damage rule is tested next to a
whole file of the same kind (`a_file_that_does_not_hold_together.rs`: a zip
whose count, offset or entry signature is wrong, cut to its first header; an
older Office file whose chain loops, whose tree points nowhere, whose version is
wrong, cut after its header), unknown-but-well-formed containers are
unrecognised rather than guessed at (a saved mail message, an EPUB, a zip with
Word and PowerPoint parts, a storage file naming two documents), and
`machine.rs` tests each `NotAnAbility`.

## Verification

Platform: Windows Server 2022 host, gates run in WSL Ubuntu with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e` (the loop's own).
The Windows host has no C compiler for `ring`, so the host cannot build
`alo-saying`'s dependency tree; this is the supervisor's configuration too.

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --all` | clean; only this change's files touched |
| Lint | `cargo clippy -p alo-opening -p alo-saying --all-targets -- -D warnings` | 0 warnings |
| Tests | `cargo test -p alo-opening` | 38 unit + 6 + 2 + 6 + 4 integration + 1 doctest, 0 failed |
| Collection | `cargo test -p alo-saying -p alo-collected` | all passed (includes the rented-name check over the new words and the every-crate-collected check) |
| Rustdoc | `cargo doc -p alo-opening --no-deps` with `RUSTDOCFLAGS=-D warnings` | 0 warnings |

**The fixtures were checked by readers this repository did not write**: the
builder's output was written to disk once and Python's `zipfile` listed and
CRC-checked the plain, commented and zip64 forms (no bad entries), and
`file(1)` identified the Word fixture as "Microsoft Word 2007+" and the older
Office fixture — whose `WordDocument` stream is in the second directory sector —
as "CDFV2 Microsoft Word". That check is not a test in the suite (it would need
Python and libmagic at test time); the scratch test that wrote the files was
deleted.

**After the refusal, second worker, in WSL as root on the gate machine:**
`cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D
warnings` clean; `cargo test -p alo-bounding --no-fail-fast` every test binary
and `--doc` passed (the failing test reproduced first, then passed after the
change; `the_kernel_refuses_an_attribute_change` 12 passed, including
`a_files_flags_are_inside_the_grant`); `cargo test -p alo-opening -p
alo-saying` all passed.

**Not run:** the whole workspace suite (the supervisor runs it). **Not
measured:** real documents saved by Office or an OpenDocument suite on a
certified machine — task 2 opens real files and is where that belongs; the
OpenDocument macro rule in `docs/quirks.md` is marked unmeasured for that
reason. There is no hardware in this task beyond reading a file handle, and the
integration tests do read files written to the real disk.

## Remaining limitations

- Kinds outside the 18 are *unrecognised*, including formats people do meet:
  HEIC photographs, TIFF, SVG, EPUB, OpenDocument drawings and templates, Apple
  iWork files, `.msg` mail. Adding one is a variant, a word and a rule.
- A PDF with bytes before its header is not recognised (decision 7).
- Legacy Office documents encrypted with the pre-2007 scheme appear as the
  document kind; the lock is found by whatever opens it, not here.
- Which older character set a text file uses is not decided.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "Before opening a file, alo OS now decides what it really
  is from its contents — not its name — and says whether it can open it, open a
  converted copy, or not at all, and why. A file whose name lies about what it is
  (a program called `invoice.pdf`) is reported as exactly that, and a damaged
  download is told apart from a format this machine does not know."
- **ROADMAP.md:** no box changes. This is the groundwork under v0.5
  `.docx`, `.xlsx`, `.pptx` *open* and ★ *"I can't open this file"*; neither
  line's code is done until tasks 2 and 4.
- **CHANGELOG.md, second line:** "Two boundary tests that failed on ext4 now set
  a file flag the way an ordinary program does, and the cause — ext4 refusing to
  drop a file's extents — is written down."
- **QUEUE.md / STATE.md:** documents-and-paper plan task 1 done; tasks 2
  (conversion) and 3 (printing) are now unblocked.
