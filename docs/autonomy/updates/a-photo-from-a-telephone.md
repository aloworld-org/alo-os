# A photo from a telephone, and a machine that was confidently wrong about it

**Date:** 2026-09-19
**Workstream:** v0.5 — documents and paper, task 6
**Task:** 6, *A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or
explained.* **One of the three is taken.** The other two are blocked on a file,
not on work, and the ask is at the end.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 4 GB of memory**. Nothing here is ticked *on the machine*.
**Egress:** **none.** No sample file was downloaded, and that is the substance of
half this report rather than a footnote: a file fetched off the web to make a
test pass has exactly the provenance this task's acceptance exists to refuse.

## Why this plan was reopened at all

`docs/autonomy/v0-5-documents-and-paper-plan.md` reads as finished, and has been
treated as finished — the owner said as much when sending this lane into
`alo_opening::Kind` for the media kinds. It is **5 of 6**. Task 6 was ready, with
every dependency done, and nobody was holding it.

It is worth saying how it stayed invisible. Counting done tasks by looking for
`**Status:** **Done,` reads this plan as **1 of 6**, because its lane marks tasks
with a `**Done, <date>.**` paragraph *under* a `**Status:** ready.` line — which
the supervisor's own rule accepts (`marked_done` in
`tools/kernel-loop/src/plan.rs` takes the mark after the label **or** at the start
of a line). Counted properly, v0.5 is **228 of 271**, and four plans are one task
short. A trailing task on a plan that reads as finished is invisible twice.

## What was wrong, which was worse than what the plan described

The plan says the three formats are met as *this machine does not recognise what
this file is* — honest, if not the sentence the promise describes.

**That was no longer true for a photograph. It read as a film.**

`alo-opening`'s rule at `ftyp` sends every ISO media container that is not `M4A`
or `M4B` to `Kind::Mp4Video`. HEIF and MP4 **are the same container**; the brand
after `ftyp` is the only thing that separates them, and a photograph's is `heic`.
So a person double-clicking a picture from their telephone was told they had a
film.

**This lane put that there.** The media kinds went in on the owner's instruction
for ADR 0051, on 2026-09-17, and gave `ftyp` a meaning it had not had. Before
that a HEIC was unrecognised. The change made nine kinds right and one thing
worse, and nothing caught it because every media kind was tested and photographs
were not a kind yet.

**Being confidently wrong is a worse failure than admitting ignorance.** *Not
recognised* sends a person to another machine; *a film* sends them to look for a
video player, and then to conclude the machine is broken. The whole point of this
crate is that the second kind of sentence does not happen here.

## What it does now

| | |
|---|---|
| Recognised as | `Kind::HeicPhoto` — *a photo in the format telephones save* |
| From | the `ftyp` brand, at offset eight, never the extension |
| Brands taken as a photograph | `heic`, `heix`, `heim`, `heis`, `mif1`, `msf1`, `avif`, `avis` |
| Brands deliberately **not** | `hevc`, `hevx` — those are an HEVC *sequence*, which is a film |
| When nothing opens it | `Cannot::NothingHereOpens` with *another machine or format* |
| When something does | `Outcome::OpensAsItIs` |

**The name does not say iPhone.** The acceptance suggested *a photo in the format
iPhones save*; the word shipped is *a photo in the format telephones save*, and
the translator's note says why — it is not one company's format, and a person
reading the sentence may not own that make of telephone. Naming the commonest
manufacturer in a sentence about a file somebody was sent is an advertisement in
the place where an explanation belongs, which is the constraint this task already
applies to remedies.

**Nothing registers a conversion for one.** `alo_converting::Conversion::EVERY`
holds three — a Word document, a spreadsheet, a presentation — and
`with_what_converts` tells a machine about those and nothing else. So a
photograph takes the `NothingHereOpens` road because **no conversion for it
exists to offer**, which is a fact about this repository and checkable in one
file.

**Corrected 2026-09-19.** This paragraph first said *no engine converts one — the
office engine the image pins converts documents*, and claimed that was checked
rather than assumed. **It was assumed.** Whether the rented engine could read a
HEIF was never measured here, and the same reasoning — from what our crates do to
what a rented engine does — was wrong about a Pages document the same day, where
the engine turned out to carry an iWork reader all along. What the engine can do
is not the reason a photograph is explained; what this machine offers is, and
that is the sentence above.

## The file is real, and the provenance is tested

Task 6: *each kind is tested against a real file with its provenance beside it,
and never a synthesised header alone.*

`crates/alo-opening/tests/files/photo.heic` was written by **macOS 26.5.2's own
ImageIO**, through `sips`, from this repository's own wallpaper — 37 008 bytes,
major brand `heic`, compatible `heic` and `mif1`, SHA-256 recorded in the README
beside it. The same library writes them on a telephone, so the `ftyp`, `meta`,
`hdlr` and `iinf` boxes are really there in the order a real one has them.

**What it is not** is a photograph taken through a lens: no Exif, no make, no
model, no location. Nothing in the rule reads any of that, so it does not weaken
the test — but it is written in the README rather than left for somebody to
assume, because the provenance *is* the point of the file.

And the provenance is **evidence, so it is checked**: a test reads the file's
length and its `ftyp` brands, then reads the README and fails if it does not name
the file or record that digest. A README describing a file somebody later
replaced would be a test passing against a story.

The picture is our own artwork, so it carries no licence anybody has to honour
and is published with the repository.

## What is left, and the one thing it needs

**A `.pages` and a `.dwg`, one real file each.**

Not work — a file. I can write both rules today: a Pages document is a zip
holding a Pages index, a DWG carries its version marker at offset zero. What I
cannot do is test either against a real file, and the acceptance forbids a
synthesised header as the only evidence — rightly, because a rule measured
against bytes we assembled proves only that it matches what we *think* the format
is.

On this machine: **Pages is not installed**, nothing draws in AutoCAD's format,
and no file of either kind exists anywhere on it or in the repository — checked,
not assumed. Downloading one would be egress for a file whose provenance is *a
stranger's website*, which is the thing the rule refuses.

**The ask:** one `.pages` and one `.dwg`, saved by somebody who has the program,
the way the owner saved `sample.docx`, `sample.xlsx` and `sample.pptx` on
2026-09-16. They need hold nothing private; a blank document and a blank drawing
carry the same markers as full ones. With those two files the rest of task 6 is
an afternoon.

**Task 5's walk gains the photo when they land**, not now. That walk runs the
office engine through `alo-convertd` and therefore cannot run on an aarch64 gate
— it is one of the five known failures here (ADR 0039 §5). Editing a test nobody
on this machine can run, to add a row nobody on this machine can verify, is how a
green branch breaks somebody else's gate. It goes in with the other two kinds, on
a machine that can run it.

## One more finding

**`crates/alo-applications/src/spelled.rs` was the only thing that caught the new
kind**, and it caught it properly: its `match` names every kind so that a kind
`alo-opening` adds is a compile error rather than a kind a person cannot choose
an application for. That file's own comment says so, and it earned it today.

Nothing else in the workspace had to change — which is the other half of the
finding. **No test anywhere asserted that a photograph is not a film**, and none
would have. The kinds are tested one at a time, each against its own bytes, and a
container shared by two kinds has no test that belongs to either of them. The new
test file now holds both directions: a real photograph is a photograph, and the
brands a real recording carries are still films.
