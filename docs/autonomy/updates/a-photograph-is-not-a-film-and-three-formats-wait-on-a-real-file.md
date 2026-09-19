# A photograph is not a film, and three formats wait on a real file

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-documents-and-paper-plan.md`, task 6 —
*A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or explained*
**Contributor:** this development PC, one worker, one working tree
**Status:** ready for integration. The task itself is **decided rather than
built**: it is marked blocked in the plan, on the decision this change proposes.

## What this task turned out to be

Task 6 asks for three formats to be recognised from their content, each
**"tested against a real file with its provenance … and never a synthesised
header alone"**. That clause is the task. It is also the reason the task cannot
be finished this week, and finding that out was most of the work.

**No machine this team has can make one of any of the three.** Measured on
2026-09-19 rather than assumed:

| Looked for | Where | Found |
|---|---|---|
| a `.heic`, `.heif`, `.pages` or `.dwg` | the development PC, under the owner's home, to six levels | none |
| the same | `/opt`, `/usr/share`, `/usr/lib` in the Linux tree the gates run in | none |
| `heif-enc`, `heif-convert`, ImageMagick, `dwgwrite`, `dwgread`, `exiftool`, `ffmpeg`, Python imaging | the same tree | none installed |
| an encoder registered with Windows for the photograph format | the development PC's own WIC encoder list | only a JPEG XL encoder; **no HEIF encoder** |
| an application that saves any of the three | both machines | the rented office engine reads a Pages document and saves none of the three |

The Mac that could save a Pages document is stopped (`SHARED_MAIN.md`). A
drawing comes from a drawing program nobody here runs. A photograph comes from a
telephone.

That leaves three ways forward, and each of them is somebody else's decision to
make rather than a worker's:

- **wait for a real file** — and the star line in `docs/features.md` naming
  these three formats is then not met in v0.5, which is narrowing a promise;
- **build one here to each specification** — and the rule and its only evidence
  share an author, in the one crate whose whole premise is that a claim about a
  file is not evidence about a file;
- **borrow somebody else's** — a licence obligation carried for ever in a
  published tree, for three test fixtures, and a rule measured against whatever
  that publisher's tool happened to write.

So the task became the decision:
`docs/decisions/0056-a-format-is-recognised-on-the-evidence-of-a-real-file.md`,
proposed, with the three options, what each costs, a recommendation, and — so
that the work is mechanical the day the files arrive — everything about task 6
that does **not** depend on having one.

## What changed

### The decision — `docs/decisions/0056-…-on-the-evidence-of-a-real-file.md`

Recommends waiting for a real file of each, says exactly what each file must be
and what it must not have been converted by on the way, and decides the rest of
task 6 now:

- **what each of the three is recognised by** — a Pages document by the index
  its list of contents holds, a photograph by its own brand among the brands its
  ISO base media header names, a drawing by the version marker it begins with;
  never by an extension;
- **that a Pages document converts** into a PDF through the engine already
  pinned, which reads that format. This is a *registration* in ADR 0039's own
  words — "each further kind … a registration and a test against a real file, in
  a later change" — and not a new engine;
- **that a photograph is explained rather than converted**, and this one was
  already decided elsewhere: a still picture in that format is encoded the way
  HEVC encodes one, and ADR 0051 leaves which software decoders may ship in the
  image open and for counsel, with the conservative reading — **no software
  decoder for an encumbered format** — in force meanwhile. Task 6's own
  constraint says the same from the other side: no new engine without an ADR.
  When counsel answers ADR 0051, that is the ADR that changes, not a new one;
- **that a drawing is explained**, through task 4's sentences;
- **what each one is called** to a person, in the shape `alo-opening`'s other
  names are written in — including the one place a product name is unavoidable,
  because the drawing format *is* that product's, and the rule that no sentence
  about *what would open it* may name a product.

### The wrong answer that did not wait — `crates/alo-opening/src/iso_media.rs`

Task 6's premise says the three files are met today as *this machine does not
recognise what this file is*. For the photograph that was **not true**.
`looking.rs` took the four bytes `ftyp` at offset four for the container MP4
names, so a photograph from a telephone was reported as *This is an MP4 video,
and nothing on this machine opens it* — and a person told that goes looking for
something to play it with.

`ftyp` is a family, not a format: the same header begins the container MP4
names, QuickTime's, 3GP's, a photograph's and a raw picture from a camera. What
tells them apart is the **brands** the header goes on to name — a major brand,
then every brand the file claims to be compatible with. The new file reads them,
and it is written as an **allowance**: it lists the brands that mean the
container MP4 names, and a file whose brands are none of them is not claimed to
be anything. The opposite shape — a film unless the brand is a known exception —
reads identically until somebody is sent a format nobody here has heard of, and
then it lies again.

It does **not** recognise a photograph. It stops this machine saying something
untrue about one, and needs no real file to prove that, because it is a refusal
rather than a claim.

Behaviour that changed, for a person:

| A file whose header says | Before | Now |
|---|---|---|
| `ftyp` with a brand in the MP4 family (`isom`, `mp42`, `qt  `, `3gp4`, `M4A `, …), as major or compatible brand | an MP4 video, or a sound recording | unchanged |
| `ftyp` with a brand nobody here lists — a photograph, a picture in the newer web format, a brand this machine has not heard of | **an MP4 video** | this machine does not recognise what this file is, and whoever sent it can say what made it |
| `ftyp` cut off inside its own header | an MP4 video | not recognised |

### Where it was held

`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`
holds the decision in place, the way `converting_waits_on_its_decision.rs` holds
ADR 0039: the decision exists once under its number and still stands; the plan
points at it from task 6 and marks that task blocked while it is proposed; it
sets out three options with their costs, a recommendation, what no option may do
and what must happen before the task is ready again; and **while it says
*proposed*, none of the three is recognised** — no kind is named for one, no
sentence says one, no extension claims one, and a file of each still reads as
what this machine honestly does not recognise.

`crates/alo-opening/tests/a_photograph_is_not_a_film.rs` holds the correction,
against files written to a real disk: a photograph is not called a film; a
photograph named `.mp4` is the *finding* rather than a film nothing opens; every
film and recording people are actually sent is still named, from the major brand
or from the compatible brands beside it; and a header cut short claims nothing
more than it named.

## Decisions taken here, and why

- **The task became an ADR rather than code.** The alternative was shipping
  recognition with fixtures this repository wrote and calling task 6 done, which
  is claiming unfinished work is finished. ADR 0039's precedent is exactly this
  shape, and the plan records it approvingly.
- **The brand fix shipped anyway, ahead of the decision it sits beside.** A
  missing answer can wait; a wrong one should not, and this one needs no file to
  prove. The holding test therefore holds the three formats and not this rule.
- **The allowance lists QuickTime and 3GP brands as the container MP4 names.**
  They are the same container under other titles, `alo-applications` already
  maps `video/quicktime` to that kind, and dropping them would have turned a
  correct answer into *not recognised* — under-claiming in the name of fixing
  over-claiming.
- **Sound with no picture is still told apart by the major brand alone.** A film
  may list `M4A ` among the formats it is compatible with; only its major brand
  says it holds no picture.
- **A new file rather than more of `looking.rs`.** `looking.rs` is the rules in
  the order they run; the brand table is a second reason to change it (it moves
  when formats do), which `CLAUDE.md`'s fourth law says is a split.
- **Task 7 was written**, because the plan named none after 6 and a plan that
  ends is a loop that stops. It is *an OpenDocument text, spreadsheet and
  presentation — converted, and what each copy lost*: task 2's own *Owed*, and
  the half of it that is **not** waiting on anybody, because the engine already
  pinned both reads and writes those formats, so the real file it is measured
  against can be saved by the application people actually use for them, on the
  machine that gates it. The three older Microsoft formats stay a later task and
  wait on documents from the owner, for ADR 0056's reason: a `.doc` saved by
  something that is not Word is not evidence about the `.doc` files people are
  sent.

## Verification

Run from this checkout's source, copied to the Linux tree the gates run in
(`/root/alo-trees/this-machine`), building into the one build directory this
machine shares (`/root/alo-builds/this-machine`), on Ubuntu under WSL 2, rustc
1.98.0:

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, zero warnings |
| `cargo test -p alo-opening` | passed |
| `cargo test -p alo-citing` | passed — the citation check, which is what a new decision reaches |

Acceptance, each test run on its own:

| What it shows | Test |
|---|---|
| the decision exists once under its number and still stands | `alo-opening` · `recognising_three_more_formats_waits_on_its_decision` · `the_decision_exists_once_under_its_number` |
| the plan points at it and steps over task 6 while it waits | `…` · `the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits` |
| three options, their costs, a recommendation, what no option may do, what must happen first | `…` · `the_decision_sets_out_the_options_their_costs_and_a_recommendation` |
| **refusal** — none of the three is recognised while it says *proposed* | `…` · `none_of_the_three_is_recognised_while_the_decision_is_proposed` |
| the checks would notice what they read going missing | `…` · `the_checks_would_notice_what_they_read_going_missing` |
| **refusal** — a photograph from a telephone is not called a film | `alo-opening` · `a_photograph_is_not_a_film` · `a_photograph_from_a_telephone_is_not_called_a_film` |
| **refusal** — a photograph named as a film is the finding | `…` · `a_photograph_named_as_a_film_is_the_finding` |
| the films people are sent are still named | `…` · `the_films_people_are_sent_are_still_named` |
| **refusal** — a header cut short claims nothing more than it named | `…` · `a_header_cut_short_claims_nothing_more_than_it_named` |

**Not run, deliberately:** the whole workspace suite. The change reaches
`alo-opening` and the documents; `SHARED_MAIN.md`'s *gate what the change can
reach* covers exactly this, and the integration owner runs the nine gates over
the combined tree.

**Not measured:** nothing on real hardware, and nothing is claimed about any. No
image was built or booted by this change, and it pins nothing.

## What is still owed

- **Task 6 itself.** Blocked on ADR 0056 and, if it is accepted as recommended,
  on one real file of each with its provenance and on task 5's table published
  again in a follow-up report. The plan says so where a worker will read it
  before starting.
- **A brand this machine has not heard of.** A film in one now reads as *not
  recognised* rather than as a film. That is honest and it is one line in
  `THE_MP4_FAMILY` away from being named; a real file is what would add it.
- **Nothing in `alo-playing`.** ADR 0051's split is unchanged: this crate says
  what the wrapping is, and what is inside it is still a different question
  asked of a different crate.

## Proposed changelog entry

> A photograph from a telephone is no longer reported as a film. The four bytes
> at the start of an MP4 file also start several other formats, and this machine
> now reads the brands after them rather than the header alone: a file whose
> brands it does not know is said to be unrecognised, with what would open it,
> instead of being named as something it is not.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: task 6 of the documents-and-paper plan moves from
  ready to **blocked on ADR 0056**; a new task 7 (*an OpenDocument text,
  spreadsheet and presentation — converted, and what each copy lost*) is ready.
- `ROADMAP.md`: no line moves. The ★ *"I can't open this file"* line stays
  unmet for the three formats it names, and ADR 0056 is where that is now
  written down rather than discovered.
- `docs/decisions/`: ADR 0056 is **proposed** and needs the owner's answer.
