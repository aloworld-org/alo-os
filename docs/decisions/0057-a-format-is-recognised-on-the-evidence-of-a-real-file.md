# ADR 0057 — A format is recognised on the evidence of a real file, and three of them must come from outside this team

**Status:** accepted, 2026-09-20 — option **A**, with every decision under
*the recommendation* as written. Written 2026-09-19 by task 6 of
`docs/autonomy/v0-5-documents-and-paper-plan.md`, which could not be built until
it was answered. **What answered it is that all three files now exist**, each
with its provenance beside it; how each was obtained is below, because the
answer to this decision is evidence rather than an opinion.
**Date:** 2026-09-19, accepted 2026-09-20
**Context:** `docs/autonomy/v0-5-documents-and-paper-plan.md` task 6;
`docs/features.md`'s ★ *"I can't open this file." A `.pages`, a `.heic`, a
`.dwg`: the system converts it where it can, and where it cannot says plainly
what will open it, instead of shrugging*;
[ADR 0039](0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md)
(a document is converted by a rented engine that can reach nothing, measured
against real documents, with a closed set of conversions);
[ADR 0051](0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
(what this machine plays, and which software decoders may ship in the image —
open, and for counsel);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (engines are
rented, configured, never patched);
[ADR 0008](0008-where-inference-happens.md) (never a silent fallback);
`crates/alo-opening` (what a file is, from its own bytes) and its
`tests/making/mod.rs` (*built, not borrowed*).

## The question in one line

**Task 6 says a `.pages`, a `.heic` and a `.dwg` are recognised from their
content, each measured against "a real file with its provenance … never a
synthesised header alone". No machine this team has can make any of the three.
So what ships, on whose evidence — and what does the star in `docs/features.md`
promise in the meantime?**

## What answered it, 2026-09-20

Option A asked for one real file of each of the three, each with its
provenance, each publishable with this repository. **All three are here.** They
did not arrive the way the decision expected — the photograph and the document
came from a machine this team already owned, and the drawing had to be written
by the format's own consortium — and that is recorded rather than smoothed over,
because what each file proves depends on where it came from.

| The file | How it was obtained | Where its provenance is |
|---|---|---|
| `photo.heic`, 37 008 bytes | **macOS 26.5.2's own ImageIO**, through `sips`, on an Apple M3, 2026-09-19, from this repository's own artwork. Written by the same library that writes them on a telephone; **not** taken through a lens, so it carries no Exif, no make and no location — and nothing the rule reads does either | `crates/alo-opening/tests/files/README.md` |
| `document.pages`, 227 583 bytes | **Pages 15.3.1** on that same Mac, 2026-09-19, once the owner installed it. This repository's own words and its own picture, a heading and a body in two faces on purpose. Saved by Pages itself — not a container assembled here | the same README |
| `drawing.dwg`, 16 352 bytes | **drawn by us** with `ezdxf` — a bench plan for the certified laptop — and **written as DWG by ODA File Converter 27.1**, the Open Design Alliance's own converter, 2026-09-20. Checked by converting it back to DXF with the same tool: five layers, nine entities and both text strings survived the round trip | the same README |

**The drawing is the one this decision was really about, so its half-measure is
named rather than implied.** Nothing on any machine this team has could save
one: no CAD application, no `dwgread`, `dwgwrite`, `teigha`, `librecad`,
`freecad` or `qcad`, and `sips` has no CAD format at all. What was done instead
is neither option B nor option C. It is not B, because the bytes were **not**
written here to a specification — they were written by the implementation the
trade licenses, which is the author whose output the rule has to survive. It is
not C, because nothing was fetched: no third party's file is redistributed here
and no licence is carried. What it is **not** is a drawing made by somebody who
draws for a living: it has no blocks, no external references, no paper-space
layouts and no hatch patterns, and the rule reads none of those. That limit is
in the README beside the file and it is the honest shape of what this evidence
covers.

**So the recognition rules stand on files rather than on agreement with
ourselves**, which was the whole of the question. Each of the three is
recognised in this change or was in an earlier one, each is measured against its
own file held to its digest, and a file that is none of them still says *not
recognised*.

## Why a worker could not simply choose

### 1. The files do not exist here, and that was measured rather than assumed

Checked on 2026-09-19, on both machines this workstream runs on:

| Looked for | Where | Found |
|---|---|---|
| a `.heic`, `.heif`, `.pages` or `.dwg` anywhere a person keeps files | the development PC, under the owner's home | none |
| the same, under `/opt`, `/usr/share`, `/usr/lib` | the Linux tree the gates run in | none |
| an encoder that could make one: `heif-enc`, `heif-convert`, ImageMagick, `dwgwrite`, `exiftool`, `ffmpeg`, Python imaging | the Linux tree the gates run in | none installed |
| an image encoder registered with Windows for the photograph format | the development PC's own encoder list | only a JPEG XL encoder; **no HEIF encoder** |
| an application that saves any of the three | both machines | the rented office engine opens a Pages document and **saves none of the three** |

The machine that could save a Pages document is a Mac, and this said it was
stopped — quoting `docs/autonomy/SHARED_MAIN.md`, which was out of date. The
Mac was running, and measured the real answer on 2026-09-19: **Pages is not
installed on it and cannot be**, because the App Store and iCloud both need a
screen somebody is sitting at and the owner's own account. So the file waits on
a person rather than on a machine being restarted, which is a different kind of
blocked and a different thing to ask for. A drawing in
`.dwg` comes from a drawing program nobody here runs. A photograph in `.heic`
comes from a telephone.

#### What the Mac itself measured, 2026-09-19

The Mac was **not** stopped, and two of the three rows above are now answered
from it rather than inferred about it. Measured on an **Apple M3 with 8 GB,
macOS 26.5.2**, and recorded here so that this decision's open question reads
*we tried, and here is what it would take* rather than *nobody has looked*.

| Looked for | How | Found |
|---|---|---|
| an encoder for the photograph | `sips --formats` | **`public.heic`, writable** — macOS's own ImageIO writes the format |
| **Pages** | `/Applications`, `/System/Applications`, `~/Applications` | **not installed** when this was written — **installed on 2026-09-19 and no longer a blocker**, see below |
| anything that writes `.dwg` or `.dxf` | `/Applications`; `dwgread`, `dwgwrite`, `ODAFileConverter`, `teigha`, `librecad`, `freecad`, `qcad` on the path; `ezdxf` for Python; `sips --formats` | **nothing.** No CAD application, no converter, no library, and `sips` has no CAD format at all |

**So the photograph was taken, and it is no longer one of the three.** The Mac
saved a real `.heic` out of ImageIO — 37 008 bytes, major brand `heic`, its
`sips` command, digest and what it is *not* recorded in
`crates/alo-opening/tests/files/README.md` — and it is measured by
`crates/alo-opening/tests/a_photo_from_a_telephone.rs`, which also reads the
provenance and fails if the file stops matching what the README says it is. The
row above saying an encoder was looked for and none found was true of the
Windows PC and the Linux tree; it was not true of the Mac.

That file is **not** a photograph taken through a lens — no Exif, no make, no
model — and this decision should weigh that rather than have it discovered
later. Nothing in the rule it measures reads any of those, so it does not
weaken the evidence; a photograph off a telephone would additionally prove that
the transfer did not silently convert it, which is the thing option A's first
bullet asks for and this file cannot show.

**Pages was answered on 2026-09-19.** The owner installed it on the Mac, and the
row above — *not installed* — is corrected rather than left standing, because it
was this lane that measured it and said so.

A real document now sits at `crates/alo-opening/tests/files/document.pages`:
saved by **Pages 15.3.1** on macOS 26.5.2 on an Apple M3, carrying this
repository's own words and a picture made from its own artwork, with a heading
and a body in two different faces so that what a conversion substitutes can be
measured. Its size, digest and full list of parts are in the README beside it,
and `crates/alo-opening/tests/a_document_from_pages.rs` holds the file to that
digest, so swapping it fails a test rather than quietly changing what the other
tests are about.

**What that settles is recognition, and only recognition.** A Pages document is
read as one, from the parts its container holds and never from its name. Whether
the engine this image already pins converts **this** document well enough is
**not measured**, and the reason is a machine rather than a decision: the engine
is x86_64 only, and the lane that made the file gates on aarch64. It is neither
claimed to convert nor claimed not to.

**One correction worth keeping**, because it is the same mistake this decision is
about: Pages was installed for some time before this lane found it, because the
application had been renamed on disk to `Pages Creator Studio.app` and every
search looked for `Pages.app`. It was identified in the end by what it *is* —
bundle identifier `com.apple.Pages`, Apple-signed, with a Mac App Store receipt —
rather than by its name. A file recognised by its extension and an application
recognised by its filename are the same error.

**The drawing is the one nothing here shortens.** It is not that no worker has
tried: nothing on any of the three machines can write that format, and the two
ways to produce one without the program — assembling it to the specification, or
fetching somebody else's — are exactly options B and C, whose costs are set out
below. **This measurement does not change the recommendation; it removes the
possibility that the recommendation was made without looking.**

### 2. Borrowing one is a licence somebody has to hold

`crates/alo-opening/tests/making/mod.rs` already argues the general case in its
own words: *a document downloaded from somewhere would be a file whose licence
and whose contents nobody here checked, and a test that depended on one would
pass or fail on what some other program happened to write that year.* Task 2's
documents are published **with this repository** because the owner made them and
they hold nothing private. A file fetched from somewhere is a third party's
work redistributed under terms somebody must read, in a repository that is
published — and that is not a worker's call to make on a Saturday.

### 3. Building one to the specification is the very thing this crate refuses

`alo-opening` exists because **an extension is a claim by whoever named the
file**. A fixture this team writes to the specification is a claim by whoever
wrote the fixture: the rule and the file it is measured against would then have
one author, and the test would say only that we are consistent with ourselves.
The whole class of bug that matters here — a real photograph whose brands are
not the ones the specification's example shows — is exactly the one a
self-written fixture cannot catch. Task 6 says so in one clause, and it is
right.

### 4. And doing none of it narrows a promise

`docs/features.md` carries the three formats **by name**, with a star. Deciding
that v0.5 recognises none of them is deciding that the star line is not met in
v0.5. `CLAUDE.md` puts that decision above a worker: *never quietly narrow a
promise in `docs/features.md`*. It is not quiet if it is written down here,
which is why this file exists rather than a paragraph in a report.

### 5. One of the three was being answered wrongly, and that did not wait

Until this change, `crates/alo-opening/src/looking.rs` took the four bytes
`ftyp` at offset four for the container MP4 names. A photograph from a telephone
begins with those four bytes, so it was reported as **an MP4 video** — a person
told that goes looking for something to play it with. Task 6's own premise says
the three files are met today as *this machine does not recognise what this file
is*; for the photograph that was not true.

That is a wrong answer rather than a missing one, so it is corrected in the same
change that proposes this decision, and it needs no file to prove: the brands
after the header decide, a file whose brands name none of the ones this machine
reads is *not recognised*, and the rule is a refusal rather than a claim
(`crates/alo-opening/src/iso_media.rs`,
`crates/alo-opening/tests/a_photograph_is_not_a_film.rs`). **It does not
recognise a photograph.** It stops this machine saying something untrue about
one.

## The options

### A — The owner supplies one real file of each, and nothing is recognised until they arrive

The shape ADR 0039 already used for the three Office documents: the rule is
written when the file that proves it is on the disk beside it, with its
provenance in a `README.md` — where it came from, what made it, when, and what
it is for.

What is needed, exactly:

- **a photograph**, taken on a telephone that saves in that format and copied
  off it **without being converted on the way** — which most transfers do
  silently, so the file must be checked to still begin with its own brand;
- **a Pages document**, saved by Pages on a Mac, with a picture and some styled
  text in it so that what a conversion costs can be measured later;
- **a drawing**, saved by a drawing program that writes `.dwg`, in a version
  that program records in the file.

Each holding nothing private, and publishable with this repository.

**What it costs:** the star line in `docs/features.md` is **not met** until the
files arrive, and this machine answers all three with *this machine does not
recognise what this file is* — honest, and the sentence task 6 exists to
replace. The delay is not ours to bound: it is a person finding three files.

### B — Recognise all three now, measured against files built here to each specification

Write the rules from the published formats, build a file of each in
`tests/making/` the way every other fixture in this crate is built, and mark
in the report that no real file has ever been shown to the rule.

**What it costs:** the rule and its evidence share an author, so the test proves
consistency rather than recognition — and this crate's whole premise is that a
claim about a file is not evidence about a file. A rule that is wrong in the way
a real file would have shown produces a **confident wrong answer** — *this is a
Pages document*, of something that is not — which ADR 0008's instinct puts below
saying nothing. It also spends the strongest argument this repository has: that
what it says about a file, it has seen.

### C — Obtain the files from outside: a sample published by somebody else

Fetch a file of each from a public source — a specification's own examples, a
library's test suite, an encyclopaedia's media — and publish it here under its
licence with attribution.

**What it costs:** a licence obligation this repository carries for ever, in a
published tree, for three test fixtures; a rule measured against whatever that
publisher's tool happened to write, which is the precise failure
`tests/making/mod.rs` refuses; and, for the photograph, no assurance it came
from a telephone rather than from the same kind of converter option B would have
used. It also requires reaching the network for a test fixture at the moment the
product's own first law is that nothing leaves silently — which is not a
contradiction, but it is a sentence somebody would have to explain.

## The recommendation

**Option A**, and the parts of task 6 that do not depend on a file are decided
here so that the work is mechanical the day they arrive.

### What each of the three becomes

| The file | What it is recognised by, in its own bytes | What this machine then does |
|---|---|---|
| a Pages document | a zip whose list of contents holds a Pages index — the names a Pages document keeps, never the extension | **converts a copy into a PDF** through the engine already pinned, which reads this format |
| a photograph | the ISO base media header with a brand of its own family among the brands it names | **explained**: recognised, and nothing here opens it |
| a drawing | the version marker every `.dwg` begins with | **explained**: recognised, and nothing here opens it |

- **The photograph is not converted, and that is already decided.** A
  photograph in that format holds a still picture encoded the way HEVC encodes
  one, and ADR 0051 leaves *which software decoders may ship in the image* open
  and for counsel — with the conservative reading in force meanwhile: the image
  ships **no software decoder for an encumbered format**. So converting one here
  would need a decoder this repository has already decided not to ship yet.
  Task 6's own constraint says the same thing from the other side: *no new
  engine is added to the image without an ADR*. When counsel answers ADR 0051,
  this is one of the things that changes, and the ADR to write then is that one
  rather than a new one.
- **The Pages document is converted, and that is a registration rather than a
  new engine.** ADR 0039 fixed the conversions at three *and* said each further
  kind is "a registration and a test against a real file, in a later change".
  The engine pinned in the image reads this format already; the fourth
  conversion is its word on the service's socket, its name in the scratch
  folder, its line in the contract, and a real document to measure what the copy
  could not carry. Nothing new is rented.
- **The drawing is explained.** No engine here reads it, and adding one is a
  decision with its own cost that nothing in v0.5 asks for. The sentence a
  person reads is task 4's: *nothing on this machine opens it or converts it
  into something that does*, and then what would.
- **Whichever option is chosen, nothing is uploaded** to find out what a file is
  or to convert it, and no sentence names Apple's, Autodesk's or anybody's
  program as what would open it. *A Mac*, *a drawing program* and *a copy saved
  as a PDF* are places a person can go; a product name is an advertisement.

### What each one is called

A name a person already uses, in the shape every other kind in
`crates/alo-opening/src/words.rs` is written in — lowercase, with its article,
read inside *This is {what}*:

- *a Pages document*, with the translator's note saying Pages is Apple's word
  processor and that the product name is normally not translated;
- *a photograph in the format telephones save*, rather than the format's
  initials: the initials are what the machinery calls it, and a person who was
  sent one by their sister knows only that it came from a telephone;
- *an AutoCAD drawing* — the one place a product name is unavoidable, because
  the format **is** that product's and no other word identifies it; this is a
  name of a thing, not an instruction to go and buy it, and no sentence about
  what would open it may name it.

## What must happen before task 6 is ready again

1. **The owner accepts, amends or rejects** this decision. **Done, 2026-09-20:
   accepted as option A.**
2. **Three real files with their provenance** arrive under
   `crates/alo-opening/tests/`, with a `README.md` beside them in the shape of
   `crates/alo-converting/tests/documents/README.md`. **Done** — the table under
   *What answered it* says which, when and from what.
3. **Task 5's walk and its table** gain the photograph, published in a follow-up
   report, with `THE_REPORT` in
   `crates/alo-converting/tests/the_walk_through_documents_and_paper.rs` pointed
   at it — because a published report is never rewritten. **Still open**: that
   walk runs the office engine and cannot run on an aarch64 gate, so it waits on
   an x86_64 machine rather than on this decision.

`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`
held this file in place while it said *proposed*: no kind for any of the three
could exist and no word could name one. That half of it stops asserting on
acceptance, by design — what it goes on checking is that this decision still
exists under its number, that the plan points at it from task 6, and that it
still sets out the options and their costs, so the argument a reader lands on is
the one that was actually answered.

## What it costs

**The star line in `docs/features.md` is not met in v0.5 unless three files
arrive.** That is the price of option A and it is written here rather than
discovered in a release note. What is bought for it is the thing this product
sells: when this machine says what a file is, it has seen one.

**A person is answered honestly in the meantime.** All three read as *this
machine does not recognise what this file is*, followed by *whoever sent it can
say which program made it, or send a copy saved in a different format*. That is
a shrug with directions — less than the promise, and not the thing the promise
was written to replace.

**One wrong answer is already gone.** A photograph is no longer called a film,
and that much shipped with this decision rather than behind it.
