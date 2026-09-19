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

## What is missing, and why

Task 6 names three formats. A **`.pages`** document and a **`.dwg`** drawing have
no file here, because neither can be produced or obtained on this machine with
provenance anybody could check: Pages is not installed on it, nothing here draws
in AutoCAD's format, and a file pulled off the web to make a test pass has
exactly the provenance the rule above exists to refuse. They want one real file
each, saved by somebody who has the program — the way the three Office documents
in `alo-converting` were saved by the owner on 2026-09-16 — and the plan records
that as what is left of the task.
