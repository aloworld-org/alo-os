# A Pages document is converted, and a photograph is explained

**Date:** 2026-09-21
**Workstream:** v0.5 — documents and paper, task 6, *A `.pages`, a `.heic` and a
`.dwg` — recognised, and converted or explained.*
**Contributor:** the third PC's lane — Claude Code, checkout `C:\dev\alo-os-3`
**Machine:** Windows Server 2022 Standard, 31 GB; gates and tests in WSL2 —
**Ubuntu 24.04.5 LTS, x86_64, 4 CPUs**, Linux 6.18.33.2-microsoft-standard-WSL2,
rustc 1.98.1. **The pinned office engine is on this machine**: LibreOffice
26.2.6.3 at `/opt/libreoffice26.2/program/soffice`, the build ADR 0039 names,
which is why the owner assigned this one task to a lane that is not the plan's.
**Egress:** none. Nothing was downloaded; every file this task measures was
already in the repository.
**Status:** ready for integration. Nothing is ticked *on the machine*: no image
was built and no certified hardware ran any of this.

## What was left, and what this does

Task 6 recognised all three formats from their own bytes, each against a real
file with its provenance, and `Conversion::PagesDocument` had been written whole
and put in `Conversion::HELD_BACK` rather than `Conversion::EVERY`. One thing
stood between it and the set: **ADR 0039 §4 makes the inventory of the original
the step before any copy, and no Pages document had ever been inventoried** —
because the engine that reads one is an x86_64 build and the plan's own machine
gates on aarch64. The measurement was taken on 2026-09-21 and published in the
plan. This change is what it unblocks.

Two things are delivered:

1. **The seventh conversion is wired.** A Pages document is a file this machine
   opens, by converting a PDF copy of it and saying what the copy could not
   carry — measured end to end against the real document and the real engine.
2. **Task 5's walk gains the photograph**, which the task's acceptance asks for,
   and the Pages document with it. The table below is the walk's, republished
   whole, and `the_walk_through_documents_and_paper.rs` now reads it here.

## What changed

### Where an inventory's bytes come from — `crates/alo-converting/src/inventory/read_from.rs` (new)

The question this task turned on is not *can the engine convert a Pages
document* — it was measured converting one on 2026-09-19 — but **what is the
inventory of the original an inventory of**.

Six of the seven conversions read the original's own zip: `word`, `excel`,
`powerpoint` and `opendocument` each do, and each is measured against real
files. A Pages document keeps its text, its styles and its annotations in
`Index/*.iwa` — compressed protobuf, to a schema nobody publishes — and
**nothing in this repository reads one**. So `read_from` says, per conversion,
which bytes its inventory reads:

- `ReadFrom::ItsOwnBytes` — the six.
- `ReadFrom::WhatTheEngineReadsOfIt(Conversion::OpenDocumentText)` — the Pages
  document. The service asks the engine for one more export, an OpenDocument
  text rendering of the original, and the measured OpenDocument reader
  inventories **that**.

**Why that is a document and not a log.** ADR 0039 §4 says what a copy could not
carry is found from the documents and *never from the engine's log*. A log is
what the engine chose to mention. What this reads is a document the engine wrote
out of the original's own content — the families its text is set in, its fields,
what it links, its comments, its tracked changes — and nothing is taken on the
engine's word about what it did.

**And what it cannot see, written down rather than discovered.** Whatever the
engine's reader does not carry out of the original reaches neither the rendering
nor the copy, so the difference between them cannot name it. That is a real
limit. It is the reason this road is taken **only** for a format nothing here
reads, and never as a shortcut past a reader somebody could write; the module
says so in its own docs, and `read_from`'s tests hold that every rendering is of
a kind this crate does read for itself, so an inventory is one step and never a
chain.

**The copy is made from the original, never from the rendering**, so nobody's
PDF is a conversion of a conversion. That costs a second run of the engine, and
`engine::LONGEST_ALTOGETHER` is what the service now waits for — a conversion
that started the engine twice inside a limit written for one run would have been
cut off by the client while the engine was still working, and a person told the
service did not answer.

### The set — `conversion.rs`

`Conversion::EVERY` is seven; `Conversion::HELD_BACK` is empty, which its own
documentation already called the finished state. `Conversion::of` answers
`Some(PagesDocument)` for `Kind::PagesDocument`, `Conversion::asked` answers to
`pages-document` on the socket, and `with_what_converts` announces it because it
reads `EVERY` and always did.

`HELD_BACK` and the three tests that hold a held-back conversion unreachable are
**kept, walking an empty list**. Deleting them with the conversion that stopped
needing them would mean the next worker writing the holding rule again from a
report, and a rule rewritten from a report comes back weaker.

### The refusal that replaced the old one — `inventory/original.rs`

`NotInventoried::NotMeasured` is gone and `NotInventoried::NotFromItsOwnBytes`
is in its place. `Original::of` is the inventory *of a document's own bytes*, so
handed a Pages document it refuses: a caller that reached it has skipped the
rendering, and an empty `Original` would then tell a person their copy lost
nothing — the one sentence ADR 0039 §4 says is unreachable without both
inventories. That refusal is tested.

### The measurement, written where it was taken — `inventory/pages.rs`

The file no longer describes a shape nobody has filled in. It records what the
engine read of `crates/alo-opening/tests/files/document.pages` on 2026-09-21,
and the two findings that were worth measuring rather than reasoning about.

| What an inventory answers | This document |
|---|---|
| every family its text is set in | `Helvetica` and `HelveticaNeue` — the document's own |
| every field whose value depends on when or where it is open | none |
| every kind of content taken from elsewhere | none; its one picture is *embedded* |
| comments | none |
| tracked changes | none |
| macros | none |

**The fonts, and why the answer is two and not five.** The rendering *declares*
five families: the document's two, and `Liberation Sans`, `Liberation Serif` and
`Noto Sans`, which the engine put there because it does not have the two it was
asked for. An inventory that counted declarations would report five, three of
which the copy carries — and the loss a person is owed, *your document is set in
Helvetica and this machine has no Helvetica*, would arrive buried in three
findings that are not losses. `inventory`'s standing rule is that **a family
counts when text is set in it**, and under that rule the answer is the
document's own two. That is ADR 0008's sentence and it is the one a person can
act on. Measured: in the rendering, `style:font-name` appears on text as exactly
`Helvetica` and `HelveticaNeue`; the other three appear only in
`office:font-face-decls`.

**Taken from elsewhere is none, beside an embedded picture.** The document
carries a pasted image and the rendering writes it into its own package as
`Pictures/10000000000001F40000011919E7127E.jpg`. A reader that counted that as
linked would tell a person a copy is incomplete offline when it is complete —
the opposite of the answer they asked for. The OpenDocument reader already draws
that line (a link counts when the package does not hold what it names), so the
answer is none. The plan records that the first reading of this measurement got
it the other way round; this is the reading that survives being run.

### The contract — `docs/contracts/agent-verbs.md`

`convert_document`'s closed set gains a Pages document, additively, which is the
shape that file already describes for itself: *this set is additive and will
grow; a format appears here on the day somebody measures one and not before*.
Nothing else about the verb moves — same arguments, same sentence, same effect.
`docs/by-hand.md` needs nothing: it describes the person's own road, which is
the same service, and names no format. `docs/contracts/person-settings.md`
already lists `pages-document` among the kinds a default application can be
chosen for.

### The walk — `tests/the_walk_through_documents_and_paper.rs`

Two moments added, and the table republished below:

- **A Pages document arrives from somebody on a Mac**, is proposed, approved and
  converted through the real service and the real engine, and the copy names
  both families it could not carry (rows 10–15).
- **A photograph from a telephone arrives** and is explained: recognised for
  exactly what it is, with nothing here that opens one, and told where one would
  (rows 26–27). That is the ★ promise's second half, met as a sentence rather
  than as a shrug.

Both are real files rather than bytes assembled by the test —
`crates/alo-opening/tests/files/document.pages`, saved by Pages 15.3.1, and
`photo.heic`, written by macOS's own ImageIO — with their provenance in that
folder's `README.md`.

## The walk, sentence by sentence

Every sentence a person meets, in order, from the walk run on this machine with
the pinned engine. The folder the walk runs in is written as `/home/anna`; that
is the one substitution, and nothing else in any sentence is touched.

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
| 10 | A Pages document arrives | This is a Pages document, and this machine opens it by converting a copy into a PDF document. The original stays exactly as it was, and what the copy could not carry over is said once it has been converted |
| 11 | The agent asks to convert the Pages document | convert /home/anna/Downloads/notes.pages into a PDF copy in /home/anna/Documents, leaving the original as it is |
| 12 | The Pages document is converted | notes.pdf is a PDF copy of notes.pages, made on this machine. notes.pages is exactly as it was |
| 13 | The Pages document is converted | The copy does not carry everything in the original. This is what is different |
| 14 | The Pages document is converted | The font Helvetica is not on this machine, so the text set in it is shown in a similar font instead, and lines and pages can break in different places |
| 15 | The Pages document is converted | The font HelveticaNeue is not on this machine, so the text set in it is shown in a similar font instead, and lines and pages can break in different places |
| 16 | The list of printers is opened | Brother HL-L2350DW series, on your network |
| 17 | The person chooses it | Set up Brother HL-L2350DW series, so this machine can print on it |
| 18 | It is set up | Brother HL-L2350DW series is set up, and this machine prints on it |
| 19 | The agent asks to print the copy | print /home/anna/Documents/report.pdf on this machine's printer |
| 20 | The indicator shows it leaving | @documents is sending something to 192.168.1.20 |
| 21 | It is printed | The document has been sent to the printer |
| 22 | The printer stops | The printer is out of paper. Put paper in its tray, and documents it has already taken print once it has some |
| 23 | Paper goes in, and the printer is asked again | The printer is ready |
| 24 | A damaged Word document arrives | This file is damaged: it is a compressed file, but it has been cut short or does not hold together inside, so it cannot be opened as it is |
| 25 | A damaged Word document arrives | What would open is a complete copy, which whoever has the original can send again |
| 26 | A photo from a telephone arrives | This is a photo in the format telephones save, and nothing on this machine opens it or converts it into something that does |
| 27 | A photo from a telephone arrives | What would open it is a machine with a program for this kind of file, or a copy saved in a different format by whoever sent it |

Rows 26 and 27 are the whole point of the star in `docs/features.md`. The
machine does not say *I do not know what this is*; it says what it is, that it
cannot open one, and where a person can go — and it names no company while doing
it (task 6's constraint: *a Mac*, *a drawing program*, *a copy saved as a PDF*,
never a product).

## Decisions taken here, and why

**A Pages document's inventory is read from what the engine reads of it, not
from its own bytes.** The alternative was a reader for `Index/*.iwa` — snappy
framing, then protobuf to a schema Apple publishes nowhere. Writing one means
guessing which field number is a font name and validating the guess against a
single file, which is exactly the rule-without-evidence ADR 0057 exists to
refuse; it would also be a new dependency and several hundred lines in the most
security-sensitive crate this plan owns, to read a format four other readers
already cover. What is written instead is the honest shape: one road for a
format nothing here reads, one file that says why, and the limit named out loud.
No ADR is needed, because ADR 0039 §4's rule — from the documents, never from
the engine's log — is kept rather than bent: a rendering is a document.

**The rendering is a separate run of the engine, not the copy re-read.** Cheaper
would have been to render once and export the PDF from the rendering. That would
make every person's copy a conversion of a conversion, with two engines'
interpretations between their document and their page. The second run costs time
this machine has; the fidelity does not come back.

**`Export` on the engine has two values, not one per kind.** One conversion
needs a rendering and it is a text document. A rendering of another shape would
come out of another of the engine's writers, and writing `ods:calc8` and
`odp:impress8` now would be two argument lists nobody has ever run — the same
mistake as `pdf:pages_pdf_Export`, which looks right and does not exist.
`read_from`'s `every_rendering_the_engine_is_asked_for_is_a_text_document` is
what fails the day a second one arrives without its argument list.

**`HELD_BACK` stays in the code, empty.** Named above; it is a promise the next
format will need, and promises re-derived from reports lose their edges.

## Acceptance criteria, and what holds each

| Criterion (task 6) | Held by |
|---|---|
| A Pages document, a HEIC photo and a DWG drawing are recognised from their content | `alo-opening`: `a_document_from_pages.rs`, `a_photo_from_a_telephone.rs`, `a_drawing_from_a_cad_program.rs` — already done, unchanged here |
| Where a rented engine the image already pins converts one, it is `Converts` with its costs, registered the way task 2 registers a conversion | `a_pages_document_is_recognised_and_converted`, and the measurement `a_pages_document_is_converted_and_the_families_it_is_set_in_are_named` |
| …and a machine where nothing converts still says *nothing here opens it* | `a_machine_that_converts_nothing_still_says_nothing_here_opens_it` |
| Where nothing on this machine opens or converts it, it is `NothingHereOpens` with task 4's *what would* | rows 26–27 of the walk, held by `the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table` |
| Each kind tested against a real file with its provenance | the walk and the conversion both read the real files in `alo-opening/tests/files/`; `the_parts_are_the_parts_of_the_one_measured_document` holds the Pages document to its own contents |
| Task 5's walk and its table gain the photo | the table above, and the same test |
| No new engine is added to the image | `converting_reaches_nothing.rs`'s `the_engine_is_named_in_one_file`; `image/` is untouched by this change |

### Refusal paths tested beside the legitimate ones

| Refusal | Held by |
|---|---|
| A Pages document handed to the inventory of a document's own bytes | `a_pages_document_is_not_inventoried_from_its_own_bytes` |
| Every conversion missing the part its format cannot be without | `a_document_missing_its_main_part_is_not_inventoried`, now asserting the right refusal for each road |
| A rendering the engine could not make | `an_engine_that_is_not_there_converts_nothing`, both exports |
| A rendering asked for as a PDF, which would compare a copy with itself | `a_rendering_is_the_engines_own_text_document_and_never_a_pdf` |
| A held-back conversion reachable from a kind, from a word, or announced | `a_held_back_conversion_is_written_and_reached_by_nothing`, `a_held_back_conversion_can_be_written_and_is_not_read_back`, `no_held_back_conversion_is_announced` |
| A document that cannot be inventoried shown as a copy anyway | `a_document_that_cannot_be_checked_is_not_converted` |

## Verification

Run on this machine, in `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine` — the serialized Linux copy and
the one build cache `SHARED_MAIN.md` requires — with
`RUSTFLAGS=-C link-arg=-fuse-ld=mold` and `RUSTDOCFLAGS=-D warnings`.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean; the two files it reformatted were carried back into the checkout |
| `cargo clippy -p alo-converting --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-converting` | 79 unit + 14 + 3 + 3 + 6 + 2 integration tests, all passing |
| `cargo test -p alo-saying`, `-p alo-declared`, `-p alo-image` | the three crates that depend on `alo-converting`, all passing |
| `cargo doc -p alo-converting --no-deps`, then `cargo doc --workspace --no-deps`, both with `RUSTDOCFLAGS=-D warnings` | clean; **this is the gate that refused the first hand-over** — see below |

**The engine ran.** `converting_a_real_document.rs` and the walk do not skip
themselves; both started `/opt/libreoffice26.2/program/soffice` through
`alo-convertd` and converted real documents, the Pages one included.

**Not run here, deliberately:** the whole workspace suite, which the supervisor
runs after this. **Not done at all:** anything on real hardware. No image was
built, no machine booted, nothing prints on a real printer.

### The gate that refused this once, and the two lines that fixed it

**The first hand-over of this work was refused, and the reason was documentation
rather than code.** The supervisor ran the nine gates twice on the tree
described above. Both times gates one to six passed — formatting, workspace
clippy, **the whole workspace suite**, and the supervisor's own three — and both
times the seventh, `rustdoc, warnings denied`, failed with three errors in this
crate:

- `conversion.rs:27`, a link to `crate::inventory::pages` — *this item is private*
  (`rustdoc::private_intra_doc_links`);
- `conversion.rs:32` and `serving.rs:15`, links to `crate::inventory::read_from` —
  *all items matching … are private or doc(hidden)*
  (`rustdoc::broken_intra_doc_links`).

`inventory` is a **private** module of `alo-converting` and was one before this
task; only its own siblings may link into it. `lib.rs` already had the right
habit for exactly this reason — its table writes `` `inventory` `` in backticks
and never as a link — and the two new paragraphs in `conversion.rs` and
`serving.rs` did not follow it. Links written *inside* `inventory/` are not
affected and were never the problem: rustdoc documents no private item, so it
lints no link in one, which is why `inventory/excel.rs` has carried the same
shape since before this change.

**The fix is the smallest one that makes it true:** the three links became plain
backticked paths, matching `lib.rs`. Three alternatives were considered and
rejected. Making `inventory` public would publish a whole reading surface — four
format readers, the difference engine, the copy reader — as a contract third
parties may build against (`CLAUDE.md`, *contracts outlive code*) to fix a
cross-reference in a comment. `#[allow(rustdoc::broken_intra_doc_links)]` is an
exemption, which a worker may never take. And deleting the paragraphs would
remove the sentence that says where an inventory's bytes come from, which is the
one thing a reader of `conversion.rs` most needs pointing at. Nothing in the
crate's behaviour changed: the diff of the correction is three comment lines.

**What this cost, and the cheap way to not pay it again.** `cargo test` never
reads a doc comment, so a crate can be entirely green and still fail the gate
that comes after the workspace suite — which is forty minutes into a run. The
first hand-over's verification table has no rustdoc row, and that omission is
exactly the gap the refusal fell through. It has a row now. **A change that
writes a rustdoc link runs `RUSTDOCFLAGS=-D warnings cargo doc -p <crate>
--no-deps` before it hands over**; it takes seconds against one crate and names
every broken link in it.

**Re-run after the correction**, in the same place and with the same
environment: `cargo fmt --all` (nothing to reformat, `--check` clean),
`cargo clippy -p alo-converting --all-targets -- -D warnings` clean,
`cargo test -p alo-converting` — 79 unit tests and 33 across seven integration
binaries, all passing, with the engine started rather than skipped — and both
rustdoc runs above clean, the workspace one documenting 104 crates. Each of the
thirteen tests in the hand-off's evidence was also run **on its own**, with
`--exact`, and each passed as one test. The crates that depend on
`alo-converting` were not re-run: the correction changes three comment lines in
`alo-converting` and nothing a dependent compiles against, and the workspace
suite had already passed on this tree at the supervisor's own gate.

## Limitations, and what is still owed

- **What the engine's Pages reader drops is invisible to the inventory.** Named
  in `read_from`'s own documentation and above. It is the accepted cost of the
  one format nothing here reads; it would not be accepted for a format somebody
  could write a reader for.
- **The claim is measured one document wide.** *This Pages document converts,
  and these two families are substituted* — not that `libetonyek`'s IWA path
  works in general. That was true of the recognition rule too, and it is the
  standard ADR 0057 sets rather than a shortfall against it.
- **The iWork exclusion is still unmeasured.** All three iWork applications
  write the same container, and `alo-opening` excludes a Keynote presentation
  and a Numbers spreadsheet by parts only one of them has. No real file of
  either exists on any machine this team has, and ADR 0057 (accepted, option A)
  says wait for one rather than synthesise it. **It is written into the plan as
  task 8** rather than left inside a finished task's body, so the next lane with
  a Mac can see it.
- **The drawing is explained, not converted**, and stays so: ADR 0058 keeps the
  decoder out of the image, and nothing this image pins reads DWG.
- **The printer in the walk is a loopback service** that speaks the protocol. A
  real printer on a certified machine is still owed (task 3).
- **The approval sentences (rows 3, 11 and 19) begin in lowercase**, which is
  how `alo-capability` writes every verb's sentence for the approval surface to
  put inside its own. That belongs to that crate and is left as it is.

## Proposed shared-document updates

Not made here — `SHARED_MAIN.md` gives these four documents one writer.

- **CHANGELOG.md:** *A document written in Pages opens.* A `.pages` file sent
  from a Mac is no longer a file this machine can only name: it opens by
  converting a PDF copy, with the original untouched and what the copy could
  not carry said by name — for the one that was measured, the two families it is
  set in, which this machine does not have. And a photo in the format telephones
  save is explained rather than shrugged at: the machine says what it is and
  where a person can open one.
- **ROADMAP.md:** nothing ticks *on the machine*. ★ *"I can't open this file" —
  converted, or plainly explained* remains `- [x] The code.` until it runs on
  certified hardware.
- **QUEUE.md / STATE.md:** documents and paper, task 6 **done, 2026-09-21**. The
  plan now names task 8, *The iWork exclusion, measured against a real Keynote
  and a real Numbers document*, blocked on a Mac.
