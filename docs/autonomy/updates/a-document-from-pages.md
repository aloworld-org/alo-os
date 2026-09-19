# A document from Pages: recognised, its conversion measured, and deliberately not wired

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

## The conversion: measured, and deliberately not wired

**It converts.** Measured by the owner on 2026-09-19, inside the **signed 0.0.3
image**, on this exact file fetched from the branch and digest-verified
(`1b011899…`, 227 583 bytes):

```
convert /tmp/document.pages as a Writer document -> /tmp/document.odt
  using filter : writer8
-rw-r--r-- 71359 /tmp/document.odt
characters of text: 778
```

And it is this document's own text, not a stub — *"alo OS — a document this
machine was sent. This document exists to be recognised…"* comes back out.

**So ADR 0057 stands on this point.** A Pages document converts through the
engine already pinned, as a registration in ADR 0039's words rather than a new
engine. No iWork reader is needed and **no amendment to ADR 0039 is needed**.

**The scope of that claim, held deliberately narrow: this is one measured file.**
It says *this document converts*. It is **not** a claim about `libetonyek`'s IWA
path in general — that support is partial and varies by Pages version, and a
second document could behave differently. One file answers one question, which is
the whole argument of ADR 0057.

### What I got wrong on the way here

I first reported that this was a conflict between ADR 0057 and ADR 0039: that no
engine here reads IWA, so an inventory was impossible, so either an iWork reader
had to be rented under ADR 0011 or **ADR 0039 had to be amended**.

That was wrong, and the way it was wrong matters. I checked what *this
repository's crates* read — zip and XML — and generalised it to the rented engine
without opening the engine. `libetonyek-0.1-lo.so.1` in the 0.0.3 image carries
`IWAParser::parseText`, `IWASnappyStream` and the literal strings
`Index/Document.iwa` and `Index/Metadata.iwa`, and `ApplePages` is a registered
filter in `share/registry/writer.xcd`. **The reader I said we would have to rent
was already rented**, and I had proposed weakening the rule the product exists to
argue for, on a premise I never checked.

Refusing to wire it blind was still right. The premise was the wrong part.

### And it is still not wired, for a different and larger reason

**The converter in the shipped image cannot start at all.** Measured the same
day, against 0.0.3 as published:

```
ldd /opt/libreoffice26.2/program/soffice.bin
  libX11.so.6, libX11-xcb.so.1, libXext.so.6, libXinerama.so.1,
  libXrender.so.1, libxcb.so.1, libICE.so.6, libSM.so.6,
  libcairo.so.2, libfontconfig.so.1, libfreetype.so.6, libcups.so.2
    => not found

oosplash: error while loading shared libraries: libXinerama.so.1
```

Twelve runtime libraries are missing; 0.0.2 has the same defect with eleven. The
conversion above only ran because those libraries were installed by hand first.

**So no document of any format converts on a real alo OS machine today — `.docx`
included.** ADR 0039's promise has been unmet in the shipped product since the
converter was added.

Wiring a Pages conversion into this would make the machine say *this converts a
copy into a PDF* and then fail on every machine that shipped, which is the shape
ADR 0039 §1 forbids by name. So the capability is **recorded as measured** and
the promise is **not made until the image can keep it**. The test for that road
is written and passing against a machine *told* it converts — it proves the
wiring, not the claim — so the day the image is fixed, registering it is
mechanical.

### Why nobody caught it, which is the part worth keeping

`image/Containerfile` ends its converter step with:

```
test -x /opt/libreoffice26.2/program/soffice
```

**That checks the executable bit.** A file can be executable and never run.

It is the third variant of one error in a single day, and this is the one that
shipped:

| Where | The proxy that was read | The thing it stood for |
|---|---|---|
| this lane, searching for Pages | the filename `Pages.app` | an application with bundle id `com.apple.Pages` — it had been renamed |
| lane A, watching a gate | `cat /tmp/gate.result \|\| echo "still running"` | work in progress — a missing file reads the same as one not yet written |
| the image, since the converter landed | `test -x soffice` | a converter that starts |

The fix is lane A's, in `image/`: the twelve libraries added, and `test -x`
replaced by **a real conversion at build time**, so the build fails if no
document comes out. That needs 0.0.4 and a signature.

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
- **The Pages conversion** — measured and proven possible, and waiting on the
  image rather than on knowledge: the shipped converter cannot start until the
  twelve missing libraries are added and `test -x` is replaced by a real
  conversion at build time. That is lane A's, and it needs 0.0.4.
- **Nothing for Keynote or Numbers.** `docs/features.md`'s star names three
  formats, and measuring two more is work outside the scope gate; the default
  answer to that is no. The iWork exclusion stays marked reasoned-not-measured,
  to be measured if a task ever asks for those formats.
- **Task 5's walk**, which gains the Pages document when it can be run — it runs
  the office engine and cannot run on an aarch64 gate.
