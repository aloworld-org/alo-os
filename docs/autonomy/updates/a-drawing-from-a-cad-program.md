# A drawing from a CAD program: recognised on six characters, and explained rather than converted

**Date:** 2026-09-20
**Workstream:** v0.5 — documents and paper, task 6
**Task:** 6, *A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or
explained.* **The drawing was the last of the three**, and with it all three are
recognised from their own bytes against a real file. The task is **not** closed:
the Pages *conversion* is written and held back, on one measurement that only an
x86_64 machine can take.
**Decision:** [ADR 0057](../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md),
**accepted 2026-09-20** as option A.
**Contributor:** lane A on the development PC, checkout `/root/alo-os` inside WSL
**Machine:** Ubuntu in WSL on the development PC, x86_64. Nothing is ticked *on
the machine*.
**Egress:** none. No file was fetched, and no file was sent anywhere to find out
what it was.

## What was blocked

Task 6 asks that each format be recognised from its content and measured against
**a real file with its provenance, and never a synthesised header alone**. The
photograph arrived on 2026-09-19 from the Mac's own ImageIO, and the Pages
document the same evening once the owner installed Pages. The drawing had
nothing behind it: no machine this team has runs a CAD program, and the two ways
to make one without the program — writing it to the specification, or fetching
somebody else's — are exactly the options ADR 0057 was written to refuse.

That is what ADR 0057 held open, and it is why this crate's guard,
`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`,
refused to let a kind for a drawing exist while the decision said *proposed*.

## What unblocked it

A real drawing landed in PR #51: **drawn by us** with `ezdxf` — a bench plan for
the certified laptop, with the desk, the machine, the external display and the
cable run on five layers — and then **written as DWG by ODA File Converter
27.1**, the Open Design Alliance's own converter, which is the DWG
implementation the trade licenses. It was checked by converting it **back** to
DXF with the same tool and reading what came out: five layers, nine entities
across four types, two dimensions and both text strings survived the round trip.

`AC1032`, 16 352 bytes, SHA-256
`d63d11d6b592e9a09b6cbec8843723bd5a087c3f708e1708d34670a49609dddc`, with its
provenance in `crates/alo-opening/tests/files/README.md`.

**What that evidence is and is not.** The bytes were not assembled here, which
is the half that matters: the rule is measured against what the format's own
consortium writes. It is **not** a drawing made by somebody who draws for a
living — no blocks, no external references, no paper-space layouts, no hatch
patterns — and nothing the rule reads touches any of those. The limit is written
beside the file rather than left for somebody to discover.

## The rule, and why it is a list rather than a pattern

A photograph is recognised from the brands its container names; a Pages document
from the parts its zip holds. A drawing has neither. **It begins with six
characters that are its version, and then goes straight into its data** — there
is nothing after them to check the guess against.

So the rule in `crates/alo-opening/src/looking.rs` is a **closed list** of the
released version markers, written out one by one:

| Marker | Written by |
|---|---|
| `AC1012` | Release 13 |
| `AC1014` | Release 14 |
| `AC1015` | 2000, 2000i and 2002 |
| `AC1018` | 2004, 2005 and 2006 |
| `AC1021` | 2007, 2008 and 2009 |
| `AC1024` | 2010, 2011 and 2012 |
| `AC1027` | 2013 to 2017 |
| `AC1032` | 2018 onward |

`AC1032` is the one there is a file of. The other seven are the released
versions the format's published history names, and they are admitted because a
drawing saved ten years ago is the same file to the person who was sent it
today.

**What is refused, and why that is the point.** *Anything beginning `AC`* would
have been one line shorter and would have claimed files nobody here has ever
seen — an unreleased marker, a version that does not exist, a file of text that
happens to start with those two letters. *This is a drawing*, said of something
that is not one, is a **confident wrong answer**, which ADR 0008's instinct puts
below saying nothing, and which is the exact failure a photograph read as a film
was in July. `only_the_released_versions_are_admitted` and
`text_that_begins_with_the_same_letters_is_still_text` are the tests that hold
the list closed.

## The road it takes, proved rather than assumed

A drawing is **explained, not converted**. Nothing this image pins reads the
format — the rented office engine does not — so it takes task 4's road:
`Cannot::NothingHereOpens` with `Would::AnotherMachineOrFormat`, the same road
as the photograph.

That was not assumed. It is proved twice in
`crates/alo-opening/tests/a_drawing_from_a_cad_program.rs`:

- against a machine told it converts **every other kind there is** — every kind
  `alo-opening` knows except a drawing, into the PDF it opens — which still
  explains a drawing. Stronger than naming the conversions this repository
  registers today, and it cannot go stale: that closed set grew from three to
  six while this was being written, and the first draft of this test failed on
  the number rather than on anything about a drawing.
- against **the registry itself**, read as a file: the place that decides which
  kinds have a conversion still names no drawing, so the day somebody writes
  one, the tests above are told they are describing the wrong road.

And the sentence a person reads was checked against ADR 0057's own constraint:
the **kind** carries a product name, because the format is that one program's
and no other word identifies it, and **what would open it names no program at
all**. A machine telling somebody to go and buy something is an advertisement,
not an answer.

## What the person gets

A file that used to answer *this machine does not recognise what this file is*
now answers *this is an AutoCAD drawing*, followed by *nothing on this machine
opens it or converts it into something that does*, followed by what would. Less
than opening it, and it is the sentence the star in `docs/features.md` describes:
where it cannot convert, it says plainly what will, instead of shrugging.

## What is still open

- **The Pages conversion is written and held back**, and not by this task. The
  image blocker cleared while this was being written — 0.0.4's recipe converts a
  real file before the build continues — and what remains is **one measurement**:
  ADR 0039 §4 makes the inventory of the original the step before any copy, and
  no Pages document has been inventoried, because the engine that reads one is an
  x86_64 build and this repository gates on aarch64. So
  `Conversion::PagesDocument` sits in `Conversion::HELD_BACK` rather than
  `Conversion::EVERY`. Registering it without the inventory would make the
  machine say *this converts* and then refuse, which ADR 0039 §1 forbids by name.
- **Task 5's walk and its table** still have to gain the photograph and the
  drawing in a published follow-up. That walk runs the office engine and cannot
  run on an aarch64 gate, so it waits on an x86_64 machine.
- **One Keynote and one Numbers document** would make the Pages rule's other
  half measured rather than reasoned. Both are a free install away on the Mac
  that now has Pages.
- **No drawing made by somebody who draws for a living** has ever been shown to
  this rule. Nothing it reads would change, but the sentence is here rather than
  in somebody's head.

## Gates

`crates/alo-opening/**` reaches that crate and every crate that depends on it,
`crates/alo-applications/**` likewise, and the `.rs` files in the diff make
`cargo fmt --all --check` unconditional. Run: formatting, clippy across the
workspace with warnings denied, the workspace's tests, and rustdoc with warnings
denied. Exit codes are in the pull request.
