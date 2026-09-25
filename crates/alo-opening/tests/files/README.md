# The files this crate's recognition is measured against

The documents-and-paper plan's task 6 asks that each kind be tested against **a
real file with its provenance beside it, and never a synthesised header alone**
— the same rule `crates/alo-converting/tests/documents/README.md` follows, and
for the same reason: a file a script wrote has none of the awkward parts a real
one has, so a rule measured against one proves only that it handles our own.

## `photo.heic`

| | |
|---|---|
| Written by | **macOS 26.5.2's own ImageIO**, through `/usr/bin/sips`, on an Apple M3 |
| When | 2026-09-19 |
| From | `docs/artwork/wallpapers/alo-quiet-horizon.png`, this repository's own artwork, resized to 800×450 |
| Command | `sips -s format heic -Z 800 docs/artwork/wallpapers/alo-quiet-horizon.png --out photo.heic` |
| Size | 37 008 bytes |
| SHA-256 | `2bb53cff09dad7a4a31963e609cf2081081ceb225c82f18b39dcae3850b98368` |
| Brands it carries | major `heic`, compatible `heic` and `mif1` |

**What this file is and is not.** It is a genuine HEIF still picture, written by
the same Apple library that writes them on a telephone — the `ftyp`, `meta`,
`hdlr` and `iinf` boxes are all really there, and it is not a header this
repository assembled. It is **not** a photograph taken by a camera: it holds no
Exif from a lens, no make or model, no location. Nothing in the rule it tests
reads any of that, so the difference does not weaken the test — but it is
recorded here rather than left for somebody to assume, because the provenance is
the point of the file.

The picture is this repository's own artwork, so it carries no licence anybody
has to honour and is published here with the rest.

## `document.pages`

| | |
|---|---|
| Written by | **Pages 15.3.1** (`com.apple.Pages`, from the Mac App Store), on macOS 26.5.2, on an Apple M3 with 8 GB |
| When | 2026-09-19 |
| How | a blank Pages document, its text set by AppleScript and the picture pasted in, then saved by Pages itself |
| What is in it | our own words — a short passage on reading a file from its bytes, and the four laws of `CLAUDE.md` — and a picture made from `docs/artwork/wallpapers/alo-quiet-horizon.png` |
| Styled | the heading is Helvetica-Bold 22pt, the body HelveticaNeue 11pt — **two families on purpose**, so that what a conversion substitutes can be measured later |
| Size | 227 583 bytes |
| SHA-256 | `1b01189904934a7c5d6b59199c9719eab111759cfea5e2a7de17d3e45ee09c4d` |

**What is actually inside it**, because that is what the recognition rule reads.
It is the **flattened single-file form** — one zip, not a package — stored
uncompressed, holding 15 entries:

| Entry | What it is |
|---|---|
| `Index/Document.iwa` | the document itself, and **the part the rule looks for** |
| `Index/DocumentStylesheet.iwa`, `Index/DocumentMetadata.iwa`, `Index/Metadata.iwa`, `Index/ViewState.iwa` | the rest of the document's index |
| `Index/CalculationEngine-*.iwa`, `Index/AnnotationAuthorStorage-*.iwa` | numbered per document, so their names are never matched on |
| `Metadata/Properties.plist`, `Metadata/DocumentIdentifier`, `Metadata/BuildVersionHistory.plist` | readable plists; the last records `Template: Blank (dev/15.3)` and `M15.3.1-7050.1.1-2` |
| `Data/pasted-image-24.jpeg`, `Data/pasted-image-small-25.jpeg` | the picture, and the thumbnail Pages keeps beside it |
| `preview.jpg`, `preview-micro.jpg`, `preview-web.jpg` | what a file browser shows without opening it |

The `.iwa` parts are compressed protobuf, and **nothing in this repository reads
them** — which is why this document is recognised but not yet converted. The
report for this task explains that.

**What this file is and is not.** It is a genuine Pages document, written by
Pages, holding real text and a real picture. It is **not** a document a person
typed: its text was set through AppleScript, because nobody sat at the keyboard.
Nothing the rule reads depends on that — the container is the same either way —
but it is recorded rather than left to be assumed.

Its words and its picture are this repository's own, so it carries no licence
anybody has to honour and is published here with the rest. It holds nothing
private.


## `presentation.key`

| | |
|---|---|
| Written by | **Keynote 15.3.1** (`com.apple.Keynote`, Mac App Store id 361285480), on macOS 26.5.2, on an Apple M3 with 8 GB |
| When | 2026-09-25 |
| How | a new Keynote presentation, its slides' text set by AppleScript, then **saved by Keynote itself** |
| What is in it | two slides of our own words — *A short presentation* over *Made to be opened, not to be read*, and *Three things* over three lines about reading a file from its bytes |
| Size | 491 152 bytes |
| SHA-256 | `47e0b3ef473a5f854a07a98f452cc8a5bc1791f733a870ea713a2565e2f41f4e` |

## `spreadsheet.numbers`

| | |
|---|---|
| Written by | **Numbers 15.3.1** (`com.apple.Numbers`, Mac App Store id 361304891), on macOS 26.5.2, on an Apple M3 with 8 GB |
| When | 2026-09-25 |
| How | a new Numbers spreadsheet, its table named and filled by AppleScript, then **saved by Numbers itself** |
| What is in it | one table, *A short spreadsheet*, with a header row — Thing, Count, Note — and three rows under it |
| Size | 112 072 bytes |
| SHA-256 | `7c999a1693d32b0ade722ca8f3bf01016aaa9286c2ebd351b64ad2e6403ecb45` |

### What these two are for, and what they are not

They are here for the **exclusion**, which is the half of the Pages rule that had
never been shown working. `document.pages` above shows that a Pages document is
recognised; these two show that the other two iWork applications' documents are
**not** — and they are the only things that can show it, because the rule is
about parts a real Keynote and a real Numbers file carry.

Both are genuine iWork containers, saved by the applications themselves through
their own AppleScript dictionaries. Nothing here was assembled: ADR 0057 accepts
option A — wait for a real file — and a container this repository built to carry
the parts the rule looks for would prove only that it can build one.

**Both carry `Index/Document.iwa`**, which is the part that says *an iWork
document*. That is the whole point of them: a rule that said *an iWork document
is a Pages document* would call both of these Pages documents, and a person
would be told their presentation converts and then watch it fail.

| | `presentation.key` | `spreadsheet.numbers` |
|---|---|---|
| entries | 56, stored uncompressed | 43, stored uncompressed |
| `Index/Document.iwa` | yes | yes |
| what excludes it | `Index/Slide.iwa` and `Index/Slide-2652176.iwa` | 30 entries under `Index/Tables/` |
| recognised as | nothing — not a Pages document | nothing — not a Pages document |

**One arm of the rule is measured and two are not.** The Keynote exclusion looks
for `Index/Slide*`, `Index/MasterSlide*` **or** `Index/Theme*`; this file carries
slides and neither of the other two, so it exercises the first alone. A
presentation made from one of Keynote's themes rather than from a new blank
document would carry them, and until one is saved those two arms rest on the
same reading of the format that the first arm did before today. Recorded here
rather than left for somebody to assume it was all measured.

## `drawing.dwg`

| | |
|---|---|
| Drawn by | **us**, with `ezdxf` 1.4.4 on the development PC — a bench plan for the certified laptop: desk, machine, external display, cable run |
| Written as DWG by | **ODA File Converter 27.1**, the Open Design Alliance's own converter |
| When | 2026-09-20 |
| Size | 16 352 bytes |
| SHA-256 | `d63d11d6b592e9a09b6cbec8843723bd5a087c3f708e1708d34670a49609dddc` |
| Format | `AC1032` — AutoCAD 2018/2019/2020, as `file` reports it |
| Holds | five layers (`DESK`, `MACHINE`, `DISPLAY`, `CABLE`, `NOTES`), nine entities across four types, two dimensions, two text strings |

**Why this counts as a real file.** The bytes were written by the Open Design
Alliance's converter, which is the DWG implementation the CAD industry licenses
— not by this repository assembling a header. It was checked by converting it
**back** to DXF with the same tool and reading what came out: every layer, every
entity type and both text strings survived the round trip. That is what makes it
evidence rather than a claim.

**What it is and is not.** It is a genuine DWG. It is **not** a drawing made in
AutoCAD by somebody who draws for a living, so it carries none of the awkward
parts a real engineering drawing would have — no blocks, no external references,
no paper-space layouts, no hatch patterns. Nothing in the rule it tests reads any
of those, so the difference does not weaken the test; it is recorded here rather
than left for somebody to assume, because the provenance is the point of the
file.

The drawing is our own, so it carries no licence anybody has to honour.

**Why it had to be made rather than found.** Nothing on any machine this team
has could write one: no CAD application, no `dwgread`, `dwgwrite`, `teigha`,
`librecad`, `freecad` or `qcad`, and `sips` has no CAD format at all — measured
on the Mac on 2026-09-19. GNU LibreDWG's writer exists and is experimental, and
a file out of it would have been nearer a synthesised header than something real
software wrote. A file pulled off the web has exactly the provenance the rule
above exists to refuse. So the drawing is ours and the writer is the format's own
consortium, which is as close to *saved by somebody who has the program* as this
team can get without owning AutoCAD.

## What is missing, and why

**And two documents this team does not have that would make one rule measured
rather than reasoned.** A Pages document, a Keynote presentation and a Numbers
spreadsheet are all written into the same container, so `Index/Document.iwa`
says *one of these three* and not which. The rule therefore also requires the
absence of the parts the other two carry — Keynote's slides, masters and theme,
Numbers' tables — and **that half has never been checked against a real file of
either**, because neither application is installed here. `a_document_from_pages.rs`
tests that the exclusion does what it says, using assembled containers; what it
cannot test is whether those are the right parts to exclude. One real Keynote
document and one real Numbers document would settle it, and both are a free
install away on the same machine that now has Pages.
