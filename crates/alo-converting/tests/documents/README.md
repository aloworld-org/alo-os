# The three documents the conversion is measured against

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

## What may not happen to them

They are **not edited to make a test pass**. A conversion that loses something
these files do not contain is a finding and a new document, added beside these
with its own provenance — never a change to one of these, which would quietly
move the bar the earlier measurements were made against.
