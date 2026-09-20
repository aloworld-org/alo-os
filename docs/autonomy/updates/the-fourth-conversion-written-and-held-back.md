# The fourth conversion: written whole, and held back at the one step nobody here can take

**Date:** 2026-09-20
**Workstream:** v0.5 — documents and paper, task 6
**Task:** 6, *A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or
explained.* This is the conversion half of the Pages document. **It does not
finish task 6**, which is still blocked on the `.dwg` and is the owner's.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory; built, linted and tested in the
Lima VM on that Mac — Ubuntu 24.04.4 aarch64, 6 CPUs, 4 GB. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.
Nothing else was fetched; no crate, no document and no image was pulled.

## What was blocked on 2026-09-19, and what cleared it

The report beside this one measured that a Pages document converts — the owner
ran this exact file through the engine inside the **signed 0.0.3 image** and
recovered **778 characters of its own text** — and then said the conversion was
*not wired*, for a reason that had nothing to do with Pages: **the converter in
the shipped image could not start**, twelve runtime libraries short, so no
document of any format converted on a real machine, `.docx` included.

That is cleared, and it is cleared in the repository rather than in a message.
`image/Containerfile` now pins **0.0.4**, and the step that used to end with
`test -x …/soffice` — *which checks the executable bit rather than that it
runs* — now runs `ldd` for a missing library and then **converts a real file**
before the build is allowed to continue. The recipe proves the converter starts;
it no longer proves a file exists.

So the road is open, and this change walks as far down it as this machine
honestly can.

## What is here

`Conversion::PagesDocument` exists and is complete in every part that can be
decided without running the engine:

| | |
|---|---|
| What it is from | `Kind::PagesDocument`, recognised from the container's parts by task 6's first half |
| What it is into | a PDF, as all three others are |
| Its word on the socket | `pages-document` |
| Its export filter | `pdf:writer_pdf_Export` — Writer's, the same one a Word document leaves by |
| Its name in the scratch folder | `document.pages`, whose ending is what tells the engine which reader to open it with |

And it is **not in `Conversion::EVERY`**. It is in a second list,
`Conversion::HELD_BACK`, whose being empty is the finished state.

## Why a fourth conversion is written and not offered

Because of the one step this machine cannot take. ADR 0039 §4 makes the
**inventory of the original the step before any copy** — `serving.rs` refuses
with `OriginalNotChecked` before it writes a single byte into the scratch folder
— and nobody has inventoried a Pages document, because the engine that reads one
is an x86_64 build and this repository gates on aarch64.

A conversion registered without one would make the machine say *this converts*
and then refuse at the inventory, which is **ADR 0039 §1's named failure**. It
is the same shape as the thing this plan spent 2026-09-19 refusing to do, at one
remove: last time the engine could not start, this time the engine could start
and the step before it cannot.

So the holding-back is not a note in a file. It is three doors, and all three
are shut in the code:

- **The verb's road.** `Conversion::of(Kind::PagesDocument)` is `None` — written
  as its own arm rather than left to fall through, so a reader sees a decision
  and not an omission.
- **The socket's road.** `Conversion::asked` reads `EVERY`, so the word
  `pages-document` is not a request. The word can still be *written* — the line
  is decided, and the day the conversion is offered nothing on the wire changes
  — and `wire.rs` has a test holding exactly that asymmetry, because an
  asymmetry nobody wrote down is a bug waiting to be tidied away.
- **What the machine says it can do.** `with_what_converts` announces `EVERY`,
  so a machine with the service answering and every conversion it makes
  announced still says *nothing here opens* a Pages document.
  `tests/a_pages_document_is_not_offered_as_a_conversion.rs` puts the real
  227,583-byte document in front of that machine and holds it to that sentence.

## The inventory, written as a shape with the numbers absent

`crates/alo-converting/src/inventory/pages.rs` is the new file, and it is the
part of this change worth arguing about.

The other three formats' files each read a document and say what it holds. This
one **reads nothing and refuses**, with `NotInventoried::NotMeasured`. What it
carries instead is the shape:

- `THE_PARTS` — the fifteen parts of the one real Pages document this repository
  holds, in the order its zip lists them, **read off that file**. A test opens
  the document and asserts the list is still exactly what is in it, so the list
  stays a reading rather than becoming a description.
- `WHAT_AN_INVENTORY_ANSWERS` — the six things an `Original` holds, so whoever
  measures one knows when they are finished. An inventory that answered five and
  left the sixth would be a partial `Original`, which this crate does not have.
- `Measured::NothingOnThisMachine` — one value, the way
  `alo_playing::right::SoftwareDecoders` has one value while counsel has not
  answered. **A question held open is not a `bool` with a default**, because a
  default is an answer somebody eventually reads as measured.

**What is deliberately not in it**: which part the fonts are in. It is tempting
to write that `Index/DocumentStylesheet.iwa` holds the families — the name says
so, and it is probably true. It has not been opened. Two of the fifteen names
end in a number that belongs to *this* document and not to the format, which is
a standing reminder of the difference between reading a file and reading a file
listing. **A part's name is not a reading of it**, and the file says so in as
many words.

## The test that fails when the work is done

`a_pages_document_is_not_inventoried_yet` asserts that the real document comes
back as `Err(NotMeasured)`. **It fails the day somebody measures one**, which is
what it is for: whoever runs the engine over this document and writes down what
it holds deletes that test, fills in `inventory`, and moves `PagesDocument` out
of `HELD_BACK` — three halves of one change, with the test as the thing that
will not let one ship without the others.

This is the second time this repository has held a question open this way rather
than guessing at it. The first was ADR 0051's software decoders, where
`alo-playing` has a `Right::IN_ORDER` with **no fourth step** and an enum with
one value saying counsel has not answered. That one is waiting on a lawyer; this
one is waiting on an x86_64 machine. The shape is the same because the mistake
it prevents is the same.

## What is not mine, and is not done

- **The inventory itself.** It needs the engine. The owner is running it and
  filling the numbers in. Nothing here estimates them.
- **ADR 0057's acceptance**, and the `.dwg`. Both the owner's.
- **Task 5's walk gaining the Pages document.** It runs the office engine and
  cannot run on an aarch64 gate.

## No changelog line, deliberately

`CHANGELOG.md` says a line there *describes what somebody can now do, or what
stopped being wrong*. **Nothing about this change is visible to a person.** A
Pages document said *nothing here opens it* before this and says exactly that
after it, and the test that holds it to that sentence is the point of the
change. A line announcing a conversion that is not offered would be the promise
this whole change exists to avoid making, written in the one file people outside
this repository read. The line belongs in the release where the inventory lands.

## Gates

Nine gates in the Lima VM, `gates-touched.sh` over the crates this change
touched and everything depending on them. The accepted failing set is unchanged:
the ADR 0039 §5 converter tests, which need the x86_64 engine on an aarch64
gate.

## Crates touched

`crates/alo-converting` only: `conversion.rs`, `engine.rs`, `inventory/mod.rs`,
`inventory/original.rs`, the new `inventory/pages.rs`, `wire.rs`, and the new
integration test. No other lane's crate was edited.

## One finding, for whoever owns `alo-converting` next

`docs/autonomy/v0-5-documents-and-paper-plan.md` task 6 carries **the Pages
account twice**, in near-identical paragraphs — the result of a rebase on
2026-09-19 resolved by keeping both sides, which was the right call at the time
and is now two copies of one thing. It is not corrected here because a published
account is not rewritten and because this change is not the place; it is
recorded so the next person editing that task knows the duplication is known
about rather than unnoticed.
