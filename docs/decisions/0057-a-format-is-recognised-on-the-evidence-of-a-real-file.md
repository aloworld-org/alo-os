# ADR 0057 — A format is recognised on the evidence of a real file, and three of them must come from outside this team

**Status:** proposed, 2026-09-19. Written by task 6 of
`docs/autonomy/v0-5-documents-and-paper-plan.md`, which cannot be built until
this is answered.
**Date:** 2026-09-19
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

The machine that could save a Pages document is a Mac, and
`docs/autonomy/SHARED_MAIN.md` records that the Mac is stopped. A drawing in
`.dwg` comes from a drawing program nobody here runs. A photograph in `.heic`
comes from a telephone.

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

1. **The owner accepts, amends or rejects** this decision.
2. **Three real files with their provenance** arrive under
   `crates/alo-opening/tests/`, with a `README.md` beside them in the shape of
   `crates/alo-converting/tests/documents/README.md`.
3. **Task 5's walk and its table** gain the photograph, published in a follow-up
   report, with `THE_REPORT` in
   `crates/alo-converting/tests/the_walk_through_documents_and_paper.rs` pointed
   at it — because a published report is never rewritten.

Until then the task stays blocked, and
`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`
holds this file in place: while it says *proposed*, no kind for any of the three
exists and no word names one.

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
