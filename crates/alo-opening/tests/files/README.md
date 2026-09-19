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

## What is missing, and why

Task 6 names three formats, and **a `.dwg` drawing has no file here.** Nothing on
any machine this team has can write one: no CAD application, no `dwgread`,
`dwgwrite`, `ODAFileConverter`, `teigha`, `librecad`, `freecad` or `qcad`, no
`ezdxf` for Python, and `sips` has no CAD format at all — measured on the Mac on
2026-09-19. A file pulled off the web to make a test pass has exactly the
provenance the rule above exists to refuse. It wants one real drawing, saved by
somebody who has the program, the way the three Office documents in
`alo-converting` were saved by the owner on 2026-09-16.

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
