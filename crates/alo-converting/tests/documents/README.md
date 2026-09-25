# The documents the conversion is measured against

[ADR 0039](../../../../docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
asks for *documents saved by the office applications people actually send them
from*, because a file a script wrote has none of the awkward parts a real one
has, and a converter measured against synthesised files proves only that it
handles our own. These are those documents.

## Where they came from

Saved by the owner on 2026-09-16, on the development PC, in Microsoft Office 365
(the version each file records in its own `docProps/app.xml`):

| File | Application | Version it records | Saved |
|---|---|---|---|
| `sample.docx` | Microsoft Office Word | `16.0000` | 2026-09-16 07:17 UTC |
| `sample.xlsx` | Microsoft Excel | `16.0300` | 2026-09-16 07:22 UTC |
| `sample.pptx` | Microsoft Office PowerPoint | `16.0000` | 2026-09-16 07:29 UTC |

They hold nothing private and are published with this repository.

## What each one is for

Task 2 of `docs/autonomy/v0-5-documents-and-paper-plan.md` reports **what a
conversion could not carry, by name**. Each loss it must name has something in
these files that produces it, so a test asserts against a real cause rather than
a constructed one.

| What the conversion must report | What is in the files |
|---|---|
| **a font substituted** | all three set their text in **Garamond**, which the image does not ship |
| **a field shown as its value at conversion** | `sample.docx` has a Word `DATE \@ "M/d/yyyy"` field with *update automatically*; `sample.xlsx` has `=NOW()` in a cell |
| **linked content not fetched** | `sample.docx` links a picture by `file:///C:\Users\SBW\Downloads\…` rather than embedding it — the file is **absent on every machine but the one it was made on**, which is the case the converter must report rather than reach for |
| **comments not shown** | `sample.docx` (`word/comments.xml`), `sample.xlsx` (a threaded comment), `sample.pptx` (a modern comment) |
| **macros not run** | none of them carries a macro, and that is deliberate: a `.docm` would be a second finding for a later change |

The linked picture matters twice. It proves the report names what was not
carried, **and** it proves law 1: a converter that fetched it would be the
person's document reaching a path — and in the general case a server — that
nobody asked it to. ADR 0039's service has no network at all, so the honest
answer is that the picture was not fetched.

## The four OpenDocument files, and why they were made differently

Task 7 of `docs/autonomy/v0-5-documents-and-paper-plan.md` adds the three
open-standard formats. Its own words say why these did not have to wait for
anybody: **the application people actually use for these files is the one the
image already pins**, so the real file can be saved on the machine that gates
the work.

Saved on 2026-09-20 by **LibreOffice 24.2.7.2** (`420(Build:2)`, Ubuntu's
`libreoffice-writer`/`-calc`/`-impress` 4:24.2.7-0ubuntu0.24.04.6) running
headless in the Lima VM that gates this repository — Ubuntu 24.04.4 aarch64.

| File | Bytes | SHA-256 |
|---|---|---|
| `sample.odt` | 24 462 | `f1e4582e79f239276e968d1a010224d4b42acfe724b876993207c9a2d3f67c14` |
| `sample.ods` | 13 641 | `7946f351106364594bd002c94d15f2ff2a6b767ad2eab612ff8a6bd3ca52449f` |
| `sample.odp` | 17 645 | `6f1b0edf4fcb0a699713f750041b009b1959fd9a588b77733de3fa6467c698c0` |
| `sample-with-a-macro.odt` | 25 678 | `7e8246846d97fd87a08234bd30981548eaab11afeaa5df4f46e62952f952d60c` |

**The words in them are this repository's own** — the four laws, ADR 0039's own
sentences about what a conversion may and may not do. The content was written
here as flat ODF (`.fodt`, `.fods`, `.fodp`) and **LibreOffice wrote the
packages**: the zip, the manifest, the styles, the thumbnails and the Basic
library are all its work, not ours, which is the part that matters. It is the
same arrangement as `../files/document.pages` over in `alo-opening`, where the
words are ours and Pages wrote the file.

This is a weaker claim than the three Office files above, and it is worth being
exact about which. `sample.docx` was saved by Word from a person's own editing,
so it carries whatever Word does when nobody is watching. These four were
round-tripped through LibreOffice from content we wrote, so they carry what
**LibreOffice** does — which is the right evidence for these three formats,
because LibreOffice is what writes them in the world, and the wrong evidence for
a `.doc`, which is why the older Microsoft formats still wait for the owner.

### What each one is for

| What the conversion must report | What is in the files |
|---|---|
| **a font substituted** | all four set their text in **Garamond and in nothing else**, and neither the image nor the gate machine has it |
| **a field shown as its value at conversion** | `sample.odt` has a `<text:date>` that is not fixed; `sample.ods` has `of:=NOW()` in a cell |
| **linked content not fetched** | `sample.odt` links a picture that is on no machine — see the note below on what LibreOffice did to the path |
| **comments not shown** | all three formats carry one — and Impress keeps its comment under a **different element name**, which is the note below |
| **macros not run** | `sample-with-a-macro.odt` carries `Basic/Standard/Module1.xml`; the other three carry only the two empty library listings every OpenDocument has |

### One choice made deliberately

These four set their text in **one family**, Garamond, and in no other. That is
fixture design rather than convenience: the question these files are here to
answer is *what does the copy fail to carry*, and a document whose text is in
two families where only one is missing from the engine answers it with a maybe.
The first draft of `sample.odt` left LibreOffice's own default style in place
and so set some of its text in DejaVu Serif, which the engine's own package
probably ships — and *probably* is not a measurement anybody can write a test
against from a machine that cannot run the engine. The default style was set to
Garamond and the file saved again, before any of it was committed.

### Three things LibreOffice did that a synthesised file would not have shown

**It rewrote the linked picture's path.** It was written as
`file:///home/nobody/Pictures/a-picture-that-is-not-here.png`; LibreOffice saved
it relative to the document, as
`../../../../../../../home/nobody/Pictures/a-picture-that-is-not-here.png`,
because saving URLs relative to the file system is what it does by default. The
link is kept and the file is on no machine either way, so the case the converter
must report — **content that was linked and not fetched** — is intact, and it is
now the shape LibreOffice really produces rather than the shape we asked for.

**Calc put the family on the column, not on the cells.** The text of
`sample.ods` is set in Garamond and **no cell in that sheet names a style at
all**: LibreOffice wrote `table:default-cell-style-name` once on the column, and
a column is a sibling of the rows rather than an element around a cell. An
inventory that looked only at what was open around the text read that sheet as
setting text in no family, and would have reported a conversion that substituted
every font in it as having lost nothing. `crate::inventory::opendocument`
follows the column because this file said it had to.

**Impress keeps a slide comment as `officeooo:annotation`**, in OpenOffice's own
2009 namespace, and not as the `office:annotation` that Writer and Calc use. It costs nothing here, and only because ADR 0039 §5's XML reader hands on
**local names** and never a prefix, so both arrive as `annotation`. A reader
that kept prefixes would read this presentation as having no comments and miss a
loss that happened. The decision that made it free was made before this file
existed; the file is what shows it was the right one.

## What may not happen to them

They are **not edited to make a test pass**. A conversion that loses something
these files do not contain is a finding and a new document, added beside these
with its own provenance — never a change to one of these, which would quietly
move the bar the earlier measurements were made against.

## The three older files, and why they were made last

Task 10 of `docs/autonomy/v0-5-documents-and-paper-plan.md` adds `.doc`, `.xls`
and `.ppt` — the shapes people still send, and the ones ADR 0039's *What this
does not decide* left for a later change.

Saved by the repository's owner on 2026-09-25, **on the development PC**, by
**Microsoft Office 365 build `16.0.20326.20140`**, each from the application that
writes that shape: Word saved the `.doc` as *Word 97-2003*, Excel the `.xls` as
*Excel 97-2003*, PowerPoint the `.ppt` as *PowerPoint 97-2003*. They hold nothing
private and are published with this repository.

Each is a real **OLE2 compound file** and not an OOXML file under an older name,
which is checked by its own bytes rather than by its extension: all three begin
`d0 cf 11 e0 a1 b1 1a e1`, and each carries the stream its application writes —
`WordDocument` and `1Table`, `Workbook`, `PowerPoint Document` and
`Current User`. A `.docx` renamed would begin `50 4b`, and checking the name is
what this repository refuses everywhere else.

| File | Bytes | SHA-256 |
|---|---|---|
| `sample.doc` | 25 600 | `e739e9ea52e3e02378fc444cf234149db8d2e5cdec7334edfb9e0972df6ef78c` |
| `sample.xls` | 26 624 | `5d85b816888d8afbf353e5cc56440306b68e583ada76bace9153e0164471a7c7` |
| `sample.ppt` | 259 072 | `0127224de5102d88cb20afa21ed34040f1c494d9213f70a774b0ef8af4eb93c6` |

### What each one is for

| What the conversion must report | What is in the files |
|---|---|
| **a font substituted** | all three set their text in **Garamond**, which the image does not ship |
| **a field shown as its value at conversion** | `sample.doc` carries a Word `DATE` field with *update automatically*; `sample.xls` has `=NOW()` in `B4` and `=COUNTA()` in `B5` |
| **comments not shown** | `sample.doc` and `sample.xls` each carry one |

**What these three deliberately do not carry, and why.** No linked picture.
Word raises a **modal security notice** when it saves a document holding a
picture linked by absolute path, and a fixture is not worth answering a security
prompt for — the prompt was met on 2026-09-25 and the step was removed rather
than clicked through. `sample.docx` already carries that case, and each
fixture's row above lists only the losses that fixture really produces, so a
test asserts the list whole rather than searching it for one entry.

PowerPoint's own comments are not in the 97-2003 shape either, so `sample.ppt`
carries the substituted face alone. A loss a fixture cannot produce is not
written down as one.
