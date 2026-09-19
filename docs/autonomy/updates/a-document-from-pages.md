# A document from Pages, and a conversion nobody has measured yet

**Date:** 2026-09-19
**Workstream:** v0.5 — documents and paper, task 6
**Task:** 6, *A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or
explained.* **Two of the three are recognised.** The `.dwg` is still blocked on a
file nobody here can make, and whether a Pages document *converts* is unmeasured
— on a machine that cannot take that measurement, which is the substance of the
second half of this report.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3 with 8 GB unified memory**, macOS 26.5.2, Pages 15.3.1;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 4 GB**. Nothing is ticked *on the machine*.
**Egress:** Homebrew's own bottle for `mas` 7.0.0, and Pages itself from the Mac
App Store — both to get a program that could save the file this task needs.
Nothing was downloaded to put in the repository: the document's words and its
picture are this repository's own.

## What was blocked, and what unblocked it

Task 6 asks for three formats recognised from their bytes, each measured against
**a real file with its provenance, and never a synthesised header alone**. The
photograph was taken on 2026-09-19. The Pages document needed Pages, which was
not installed, and which a worker does not install on somebody's personal machine
uninvited — so it was written down as a request, and the owner installed it.

## One thing I got wrong first, and it is the same mistake as the task

After the install I reported that Pages still was not there. It was. The
application had been **renamed on disk to `Pages Creator Studio.app`**, and every
search I ran looked for the filename `Pages.app`.

It was identified in the end by what it **is** — bundle identifier
`com.apple.Pages`, version 15.3.1, Apple-signed, carrying a Mac App Store receipt
— rather than by what it was called.

**An application recognised by its filename and a file recognised by its
extension are the same error**, and I made it inside the task whose entire
subject is not making it. It is recorded here and in ADR 0057 rather than quietly
fixed.

## The document

`crates/alo-opening/tests/files/document.pages`, saved by Pages itself.

| | |
|---|---|
| Written by | Pages 15.3.1, macOS 26.5.2, Apple M3 |
| Content | our own words — a passage on reading a file from its bytes, and the four laws of `CLAUDE.md` — and a picture made from `docs/artwork/wallpapers/alo-quiet-horizon.png` |
| Styled | heading Helvetica-Bold 22pt, body HelveticaNeue 11pt |
| Size | 227 583 bytes |
| SHA-256 | `1b01189904934a7c5d6b59199c9719eab111759cfea5e2a7de17d3e45ee09c4d` |

**Two font families on purpose.** ADR 0039 §4 measures what a copy could not
carry, and a substituted font is the first item on that list. A document set
entirely in one face could never show it.

**The picture was pasted, not scripted.** Pages' AppleScript has no image
constructor — `make new image` answers *Don't know how to create
TMAScriptImageInfoProxy* — so the picture went in through the clipboard and a
paste into the real application. The document is Pages' own output either way.
It is **not** a document a person typed, and the README says so rather than
leaving it to be assumed.

## What is inside it, which is what the rule reads

The **flattened single-file form** — one zip, stored uncompressed, 15 entries:
`Index/Document.iwa` and the rest of the index, `Metadata/` plists,
`Data/pasted-image-*.jpeg`, and three preview JPEGs. The full table is in the
README beside the file.

## The rule, and why `PK\x03\x04` is not it

A Pages document is a zip. **So is a Word document, a spreadsheet, a
presentation, every OpenDocument and every plain archive anybody was ever sent.**
A rule keyed on the signature would call all of them Pages documents.

So the rule reads the **list of parts**, which is where `crates/alo-opening/src/zip.rs`
already decides between Office and OpenDocument, and it is written in
`crate::iso_media`'s shape: **an allowance, not a refusal.** A container carrying
`Index/Document.iwa` and **neither** of the parts the other two iWork
applications have is a Pages document; anything else is claimed to be nothing.

Turned round — *an iWork document is a Pages document unless it looks like the
others* — it would read the same until somebody was sent a form nobody listed,
and then say *this is a Pages document* about a presentation. That is exactly
what this crate did to a photograph until this morning, and it is worse than
saying nothing because a person acts on it.

**One half of that rule is reasoned and not measured, and it is marked so.** All
three iWork applications write the same container, so the exclusion depends on
Keynote's slides, masters and theme and Numbers' tables being the right parts to
exclude — and this team has no document from either to check it against. The test
proves the exclusion *does what it says*, using assembled containers; it cannot
prove those are the right parts. **One real Keynote and one real Numbers document
would settle it**, and both are a free install away on the machine that now has
Pages.

## The conversion is unmeasured, and I first said something stronger

I wired the conversion — a fourth `Conversion`, its word on the socket, its name
in the scratch folder, its export filter — then took it out, and reported that
ADR 0057 and ADR 0039 were in conflict: that no engine here reads IWA, so an
inventory was impossible, so either an iWork reader had to be rented under ADR
0011 or ADR 0039 had to be amended.

**That was wrong, and the way it was wrong is worth recording.** I checked what
*this repository's crates* read — zip and XML — and generalised it to the rented
engine without checking the engine. The owner measured it against the pinned
image the same day:

> `/opt/libreoffice26.2/program/libetonyek-0.1-lo.so.1` in
> `ghcr.io/aloworld-org/alo-os:0.0.3` contains `IWAParser::parseText`,
> `IWASnappyStream`, and the literal strings `Index/Document.iwa` and
> `Index/Metadata.iwa`. `ApplePages` is a registered import filter in
> `share/registry/writer.xcd`.

So **the reader I said we would have to rent is already rented**, ADR 0039 needs
no amendment, and its rule — an inventory before any copy is shown, and never
*this converts* followed by a failure — is exactly the rule that makes this
product's argument. I had proposed weakening the thing the product is for, on a
premise I had not checked.

**Not wiring it was still right, for a different reason.** `libetonyek`'s IWA
support is partial and varies by Pages version, so *the engine reads the format*
is not *the engine reads this document well enough to say what the copy lost*.
That is a measurement, and **this lane cannot take it**: the engine is x86_64
only, which is why the image does not run on an aarch64 gate.

So the conversion is **neither claimed nor ruled out**. It is not registered,
because a machine that says *this converts* and then refuses is the shape ADR
0039 §1 forbids; and it is not written off, because nobody has run the file
through the engine. The owner is doing that on the PC that can:

```
soffice --headless --convert-to odt document.pages
```

If the text comes through, ADR 0057's claim stands and the wiring is mechanical —
and the test for that road is already written and passing against a machine
*told* it converts, so the sentence a person meets is held in advance. If it
fails or comes out empty, ADR 0057 asserted something nobody had measured, and a
Pages document takes the explain road the photograph takes.

**Either way the answer is a file rather than an argument**, which is the whole
of what ADR 0057 is about — and I had started making it an argument.

## Evidence — tests this task publishes

`crates/alo-opening/tests/a_document_from_pages.rs`, **10 tests, all passing**:

| | |
|---|---|
| `the_document_is_the_one_its_provenance_records` | size **and SHA-256**, and that the README names both — swapping the file fails here rather than quietly changing what the rest of the file is about |
| `a_real_pages_document_is_recognised` | the real document is `Kind::PagesDocument` |
| `it_is_recognised_under_a_name_that_lies` | the same bytes named `minutes.docx` are still a Pages document, **and reported as misnamed** |
| `the_other_real_zips_are_not_claimed_to_be_pages_documents` | the real `sample.docx`, `sample.xlsx`, `sample.pptx` are still themselves |
| `a_zip_that_holds_nothing_of_the_sort_is_an_archive` | a plain archive stays an archive |
| `an_iwork_container_with_another_applications_parts_is_not_called_pages` | the exclusion does what it says, for slides, tables and themes |
| `the_metadata_alone_does_not_make_a_pages_document` | the index is what names it; its absence is not guessed past |
| `a_machine_with_nothing_explains_it_rather_than_shrugging` | the road it takes **today**, with *another machine or format* |
| `a_machine_that_converts_one_offers_a_pdf_copy` | the road ADR 0057 intended, held for when it lands |
| `a_pages_document_is_not_a_kind_that_is_played` | not a kind `alo-playing` is for |

The three guards that caught the photograph caught this kind too, and each is a
different omission: `alo-applications`' exhaustive `spelled` match, its media-type
table (`application/vnd.apple.pages`), and the published contract in
`docs/contracts/person-settings.md`, which lists every kind a person's settings
file may be keyed by.

## What is left in task 6

- **The `.dwg`** — one real drawing. Measured again on 2026-09-19: no CAD
  application, no `dwgread`, `dwgwrite`, `ODAFileConverter`, `teigha`, `librecad`,
  `freecad` or `qcad`, no `ezdxf`, and `sips` has no CAD format. This is the whole
  of what keeps ADR 0057 proposed.
- **The Pages conversion**, on a measurement this lane cannot take: the engine is
  x86_64 and this machine is aarch64. One `soffice --headless --convert-to odt
  document.pages` on a PC that can run the image settles it either way.
- **One Keynote and one Numbers document**, to make the exclusion measured.
- **Task 5's walk**, which gains the Pages document when it can be run — it runs
  the office engine and cannot run on an aarch64 gate.
