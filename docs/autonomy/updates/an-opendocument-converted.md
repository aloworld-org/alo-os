# An OpenDocument converted, and a column that nearly told a lie

**Date:** 2026-09-20
**Workstream:** v0.5 — documents and paper, task 7
**Task:** 7, *An OpenDocument text, spreadsheet and presentation — converted, and
what each copy lost.* **Done.**
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory; built, linted and tested in the
Lima VM on that Mac — Ubuntu 24.04.4 aarch64, 6 CPUs, 4 GB. Nothing is ticked
*on the machine*.
**Egress:** `git fetch`/`git push` against `github.com/aloworld-org/alo-os`, and
Ubuntu's own archive for `libreoffice-writer`, `-calc`, `-impress` and
`fonts-crosextra-carlito` (4:24.2.7-0ubuntu0.24.04.6, aarch64) installed **in the
Lima VM only**, to save the four real files this task is measured against.
Nothing was downloaded into the repository: the words in those files are this
repository's own.

## Why this task was not waiting for anybody

The three Office formats are measured against documents the owner saved in Word,
Excel and PowerPoint, because ADR 0057's rule is that a `.doc` saved by something
that is not Word is not evidence about the `.doc` files people are sent.

The open-standard formats invert that. **The application people write these with
is the one the image already pins**, so the real file can be saved on the machine
that gates the work — which is what the task's own text said, and it was right.

## What is here

`Conversion::EVERY` holds **six**, each into a PDF:

| Conversion | Its word on the socket | Its scratch name | Its export filter |
|---|---|---|---|
| Word document | `word-document` | `document.docx` | the prose writer |
| Excel workbook | `excel-workbook` | `document.xlsx` | the sheet writer |
| PowerPoint presentation | `powerpoint-presentation` | `document.pptx` | the slide writer |
| **OpenDocument text** | `opendocument-text` | `document.odt` | the prose writer |
| **OpenDocument spreadsheet** | `opendocument-spreadsheet` | `document.ods` | the sheet writer |
| **OpenDocument presentation** | `opendocument-presentation` | `document.odp` | the slide writer |

**No new engine**, and the closed set is not relaxed — it grew the way ADR 0039
said it would: *a registration and a test against a real file, in a later
change*. The engine has exactly three writers, so six conversions share three
filters, and the test that used to demand one filter per conversion now demands
that there be exactly those three and no fourth — because a fourth would mean
somebody had written down a writer this engine does not have.

## The four documents

Saved on 2026-09-20, headless, in the Lima VM that gates this repository. The
words in them are ours — the four laws, ADR 0039's own sentences — written here
as flat ODF; the **packages** are the application's work: the zip, the manifest,
the styles, the thumbnails, the Basic library. It is the same arrangement as
`document.pages` in `alo-opening`.

`sample-with-a-macro.odt` is the first document in this repository that can prove
the sentence **macros were not run** against a real file. The three Office
documents deliberately carry no macro, and a `.docm` was left to a later change;
an OpenDocument keeps its macros in the open, as a module under `Basic/`, where
`alo-opening` already sees them.

Digests and full provenance: `crates/alo-converting/tests/documents/README.md`.

## The column that nearly told a lie

This is the part worth the whole task.

`sample.ods` sets its text in Garamond. **No cell in that sheet names a style at
all.** The application wrote `table:default-cell-style-name` once, on the
*column* — and a column is a sibling of the rows, not an element around a cell,
so nothing about what is open around a piece of text can find it.

The first version of the reader resolved a family the way a text document does:
from the innermost style outwards. Run against the real spreadsheet it found
**no families whatsoever**, which would have meant a conversion that substituted
every font in that sheet reporting that it **lost nothing**.

That is ADR 0039 §4's failure in its quietest form. It does not crash, it does
not refuse, it does not say *this converts* and then fail. It produces a copy and
a sentence saying the copy is faithful, and the sentence is false. A synthesised
fixture would never have shown it, because whoever wrote the fixture would have
put the style where the reader looks.

`crate::inventory::opendocument` now tracks a table's columns — including
`table:number-columns-repeated`, capped, because a sheet's trailing empty columns
are written as one element repeated sixteen thousand times and the document came
from a stranger.

## The other thing only a real file said

A presentation keeps its slide comment as `officeooo:annotation`, in a namespace
of its own, where a text document and a spreadsheet write `office:annotation`.

It cost nothing — and only because ADR 0039 §5 decided that the XML reader hands
on **local names and never a prefix**, so both arrive as `annotation`. A reader
that kept prefixes would have read that presentation as having no comments and
missed a loss that happened. The decision was made long before this file existed;
the file is what shows it was the right one.

## One fixture choice, made deliberately and worth declaring

The first `sample.odt` left the application's own default style in place, so some
of its text was set in DejaVu Serif — which the engine's own package **probably**
ships, and *probably* is not something a machine that cannot run the engine may
write a test against.

So the document's default style was set to Garamond and the file saved again,
before any of it was committed, and now all four set their text in **one family
and no other**. That is fixture design rather than convenience — these files
exist to answer *what did the copy fail to carry*, and a document in two families
where only one is missing answers it with a maybe — but it is a change made to
remove an inconvenient measurement, so it is declared here and in the README
rather than left for somebody to find.

## What is measured here, and what is not

**Measured on this gate:** every original. `inventory::original`'s tests hold
each of the four documents to exactly what it contains — the families, the
fields, the linked picture, the comments, the macro library.

**Not measured on this gate:** the copies. The four new conversion tests need the
pinned x86_64 engine, so on an aarch64 gate they join the accepted failing set.
Their loss lists are held by rules this repository has already measured three
times over on the Office documents — a family the engine lacks is substituted, a
field that is not fixed is fixed, a link is not fetched, a comment is not shown,
a macro is not run — and by nothing about these files in particular. **They
become measurements the first time this suite runs where the engine is**, and the
tests say so in their own words.

## A break on `main`, and the gate practice that hid it

**This is the most important thing in this report, and it is mine.**

While running this task's tests I found that
`no_english_outside_the_vocabulary_in_documents_and_paper` **was already failing
on `main`** — broken by my own PR #52 the same day, because
`inventory/pages.rs` introduced English literals nobody had declared.

I gated #52. I gated `main` after merging it. Both came back as *eight of nine,
the accepted ADR 0039 §5 converter set only*, and I reported that. Both were
true and both were useless, for one reason:

> **`cargo test` stops at the first failing test binary.** The accepted failing
> set lives in `converting_a_real_document`, which runs before
> `no_english_outside_the_vocabulary…` — so on a tree with an accepted failure,
> every test binary after it is **unproven, not passing**, and the gate's summary
> line cannot tell the two apart.

The judge, `nofail.sh`, runs `--no-fail-fast` and would have caught it. But
`publish.sh` only reaches for the judge *when the gates report a failure it does
not recognise*, and I ran `gates-touched.sh` by hand and read its summary as an
answer. A summary that says *the accepted set only* is a summary that cannot say
that, because it never ran the rest.

**So: on this repository, while an accepted failing set exists, the judge is not
optional.** I measured `main`'s real state with it — six failing, not five: the
five accepted converter tests plus my regression. This change fixes the
regression and the branch's own failing set is the five accepted plus the four
new engine-dependent ones.

I would rather record this than the clean version of it. The guard worked
perfectly; the way I ran it is what failed, and it failed silently, which is the
kind of failure this repository exists to be intolerant of.

## Crates touched

`crates/alo-converting` only — `conversion.rs`, `engine.rs`, `inventory/` (the
new `opendocument.rs` and `formula.rs`, and `mod.rs`, `original.rs`, `excel.rs`,
`pages.rs`), its tests and its test documents — plus `docs/contracts/agent-verbs.md`,
which gained the list of what converts and the rule by which it grows. No other
lane's crate was edited.

## Findings for somebody else

- **`publish.sh` and `gates-touched.sh` should run the judge whenever the tree
  has a known failing binary**, not only when a gate reports something
  unrecognised. That is a lane-script change, outside this repository.
- `v0-5-documents-and-paper-plan.md` task 6 still carries the Pages account
  twice, from the 2026-09-19 rebase resolved by keeping both sides. Recorded in
  the previous report and still true.
