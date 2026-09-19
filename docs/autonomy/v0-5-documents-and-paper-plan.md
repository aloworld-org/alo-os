# v0.5 — documents and paper: opening what a person was sent, and printing it

**Workstream:** four `ROADMAP.md` v0.5 lines that are one subject —
**Printing**, ★ *Printers, solved — found, set up, and fixed when they stop*,
`.docx`, `.xlsx`, `.pptx` *open*, and ★ *"I can't open this file" — converted,
or plainly explained.* They belong together because they are the same
predicament twice: **a person has been handed something made somewhere else,
and the machine either deals with it or admits it cannot.**
**Why it exists:** written 2026-09-14 so that a third machine has a plan of its
own. Nothing here overlaps a running lane.

**Why this is the honest half of the promise.** Every desktop before this one
claimed to open these formats and then produced a document that looked wrong in
ways the person only discovered after sending it on. ADR 0008 forbids the
silent fallback: a conversion that lost something says what it lost, and a file
this machine cannot open is refused in a sentence a person can act on rather
than opened badly. ★ *"I can't open this file" — **converted, or plainly
explained**.* The second half of that line is the deliverable, not the excuse.

**Crates this plan owns:** a new `crates/alo-printing` (what a printer is, how
one is found, what "it stopped" means and what to do about it) and a new
`crates/alo-opening` (what a file is, what this machine can do with it, and
what it cannot). **It reads and never edits** `alo-capability` (printing and
converting are verbs), `alo-egress` (law 1 — a converter or a driver that is
not on this machine is an errand that leaves, and the indicator fires for it),
`alo-granted` (a file is opened under a grant, not because it was named),
`alo-record`, `alo-saying` and `alo-strings` (every sentence a person reads),
and `alo-nearby` (a printer on the local network is found the way a machine is
— see its own rules). Nothing in `crates/alo-shell`: the window that shows a
print dialogue or a conversion notice is the shell plan's, and what this owes
it is the decisions it draws.

**What this plan may not do:** tick anything *on the machine* — a printer that
has never printed on certified hardware is `- [x] The code.` and nothing more;
name a rented engine anywhere a person reads (`docs/features.md`: *a person
never learns the name of anything we rented*); patch a rented engine (ADR 0011
— CUPS, and whatever converts a document, are configured and never patched, and
a source patch needs an ADR first); or convert anything by sending it off the
machine without that being an errand a person saw. Before writing the next
task, `git pull` and read the plan as published.

**Owner-authorized contribution, 2026-09-18:** the third PC may recover the
printer producer API required by broker task 2. This releases only the files
below to that task; this plan retains its ownership and unfinished work.

```owner-release
plan = docs/autonomy/v0-5-the-broker-and-the-disk-plan.md
task = 2
files =
  crates/alo-printing/src/changing.rs
  crates/alo-printing/src/found.rs
  crates/alo-printing/src/lib.rs
  crates/alo-printing/src/printer.rs
  crates/alo-printing/src/set_up_here.rs
  crates/alo-printing/src/setting_up.rs
  crates/alo-printing/src/verbs.rs
  crates/alo-printing/tests/changing_a_printer_set_up.rs
  crates/alo-printing/tests/serving/mod.rs
  crates/alo-printing/src/http.rs
  crates/alo-printing/src/service.rs
  crates/alo-printing/tests/cups_runtime/mod.rs
  crates/alo-printing/tests/the_real_printing_service.rs
```

## Tasks

### 1. What this machine can do with a file, and what it cannot

**Status:** ready. **Depends on:** nothing.

**Done, 2026-09-14.** `crates/alo-opening`: `decide` takes an open file, its
name and a `ThisMachine` and answers with a `Decided` — `AsItIs(Outcome)` or
`NotWhatItsNameSays { named, outcome }`, the finding wrapping the outcome.
`Outcome` is `OpensAsItIs`, `Converts` with its `Costs`, or `CannotOpen` with one
of six `Cannot` reasons (empty, unrecognised, damaged, a program, locked with a
password, nothing here opens it). What a file is comes from signatures, a zip's
list of contents, the older Office directory and whole-file text rules; the name
is compared afterwards. 35 strings collected by `alo-saying`. Report:
`docs/autonomy/updates/what-this-machine-can-do-with-a-file.md`. For tasks 2 and
4: `ThisMachine::converts` is where a conversion registers, and `Cannot` is the
set task 4's sentences extend.

Before anything converts, opens or prints, the machine has to be able to answer
one question honestly: *given this file, what are my options?* Today nothing can
answer it, which is why every system's answer is a spinner followed by a
disappointment.

- **Acceptance:** `alo-opening` holds what a file is for this purpose — what it
  appears to be **from its content rather than its name**, because an extension
  is a claim by whoever sent it and a machine that trusts it is a machine that
  can be handed anything; and what this machine can do with it, as a closed set
  — *open it as it is*, *convert it to something openable and say what that
  costs*, *cannot open it* — with **no variant meaning "probably"**, because a
  maybe is the spinner in type form; a file that is not what its name claims is
  reported as exactly that, and the mismatch is the finding rather than a thing
  to silently correct; and each outcome carries the sentence a person reads,
  collected in the vocabulary `alo-saying` gathers.
- **Constraint:** nothing here converts, opens, executes or reads more of a
  file than deciding requires. It decides; the doing is tasks 2 and 3. No
  outcome may be reached by asking anything off this machine — that would make
  *what is this file* an errand, and the answer to a question a person did not
  know they were asking.

### 2. `.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost

**Status:** **Done, 2026-09-16.** `crates/alo-converting`: `convert_document`
is a change under grants over both the document and where the copy goes, and
the copy is a PDF. The executor asks the grants again about the copy's own
path, opens the document read-only with `openat2` and no symlinks (a hard-linked
document is refused), decides its kind from its bytes with `alo-opening`,
creates the copy with `O_EXCL`, and passes both descriptors to `alo-convertd`
over `SCM_RIGHTS`. The service re-decides the kind, inventories the original,
runs the pinned engine with a fixed argument list and a cleared environment in
a private scratch folder, inventories the copy, and only then writes it.
`engine.rs` is the only file that names the engine. What the copy could not
carry is said by name — a substituted font, a field fixed at its value, macros,
linked content not fetched, comments, tracked changes — and *lost nothing* is
its own sentence, reachable only after both files were inventoried; an
inventory that cannot complete is a refusal and the copy is removed. Tested
through the real service against the owner's three documents, which lose
exactly what their `README.md` says they should. The image pins the engine's
archive by digest and ships the socket and service with no network, `AF_UNIX`
only and no capabilities. Published 506b6a5 by hand, because the supervisor
this lane runs under was killed by the machine running out of memory while the
gates were passing. Owed: the image has not been built or booted, so the
sandbox is `- [x] The code.` and nothing more. Report:
`docs/autonomy/updates/office-documents-converted-and-what-the-copy-lost.md`.

What it waited on, and how each was settled:
`docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md`
was accepted on 2026-09-15 (option A), and **the owner's three documents arrived
on 2026-09-16**: `crates/alo-converting/tests/documents/`, saved in Microsoft
Office 365, with their provenance and what each one proves in the `README.md`
beside them — Garamond in all three, a Word `DATE` field and an Excel `=NOW()`, a
picture linked to a path that exists on no other machine, and a comment in each
file. The third thing the decision lists, **the pinned engine on the machine that
gates this crate, is this lane's own step**: install it from the release and
digest the image pins, in the lane's own distribution, and paste the version into
the report. A conversion test that skips itself where the engine is missing is
refused (the decision's own rule). **Depends on:** 1.

**Done, 2026-09-16.** `crates/alo-converting`: `convert_document(file, into)`, a
change with grants over both, carried out by `convert` — the copy's own path
asked of the grants again, the document opened read-only following no link, its
kind decided from its bytes, the copy created with `O_EXCL` under the document's
name ending `.pdf`, and both handed as descriptors to `alo-convertd`, which
inventories the original, runs the pinned engine (LibreOffice 26.2.6, one file
names it) in a scratch folder of its own, inventories the PDF and only then
writes it. `Carried` is `Everything` or `NotEverything` with each `NotCarried` by
name — a font substituted, a field fixed, macros, linked content not fetched,
comments, tracked changes; an inventory that cannot complete is a refusal and
the copy is removed. Tested through the real service and engine against the
owner's three documents, with the refusals beside them. The image pins the
engine by digest and ships `alo-convertd.socket` and `.service` (no network,
Unix only, no home folders, a login of its own, 60992), held by `alo-image`.
`alo-record` gained an additive `told` stamp for what the copy lost. 37 strings
collected by `alo-saying`; the engine added to its rented list; the verb in the
contract and `docs/by-hand.md`. Report:
`docs/autonomy/updates/office-documents-converted-and-what-the-copy-lost.md`.
Owed: the image built and booted (the `/opt` link and the service's sandbox are
unmeasured on the base), the shell's window for the copy, and older Office and
OpenDocument files.

**Decided rather than built, 2026-09-14.** The first worker found that the code
could not be written without choosing things that are not a worker's to choose:
nothing in this product starts a program, and `alo-bounding` documents why; a
converter started from a turn is not known to stay inside the turn's kernel
boundary; a document's linked pictures would be fetched by the engine; the image
pins no converter; the gate machine has none, and installing one there is shared
maintenance; and no machine the loop runs on can make a real Office file. ADR
0039 sets out three shapes (a converting service of our own with no network and
no files, the verb starting the engine inside the turn, or the person's own
office application), what each costs, and recommends the first. It also decides
the copy (a PDF), the verb, and how *lost nothing* is told from *did not check*.
It lists the three things that must happen before this task is ready again:
the owner's answer, the pinned engine on the gate machine in an idle handoff,
and three real documents with their provenance.
`crates/alo-opening/tests/converting_waits_on_its_decision.rs` holds the ADR in
place and fails if a converter is built while it still says *proposed*. Report:
`docs/autonomy/updates/converting-office-documents-decided-before-it-is-built.md`.

`ROADMAP.md`: *`.docx`, `.xlsx`, `.pptx` open.* They are the formats a person
is actually sent, and the reason a sovereign desktop gets returned. The engine
that reads them is rented and unmodified; what is ours is that a person is
never misled about the result.

- **Acceptance:** a document in each of the three formats is opened through the
  rented converter running **on this machine**, with a test per format against
  a real file rather than a synthesised one; what the conversion **could not
  carry** is reported by name — a font that was substituted, a field that did
  not survive, a macro that was not run — and a conversion that lost nothing
  says so, because *lost nothing* and *we did not check* must not read the same;
  the converted document is written where the person chose and never over the
  original, so the thing they were sent still exists byte-for-byte; and
  converting is an `alo_capability` verb that runs under a grant on the folder
  the file is in, never because a path was typed.
- **Constraint:** the converter is rented (ADR 0011) and no source patch is
  written here. If it cannot carry something, that is a finding and a sentence,
  never a second converter of our own guessing at the difference. Nothing is
  uploaded: if the only way to convert a format were a service, the answer for
  v0.5 is *cannot open it* with the reason, not a quiet upload.

### 3. A printer is found, set up, and says what is wrong with it

**Status:** ready. **Depends on:** 1.

**Done, 2026-09-14.** `crates/alo-printing` talks to the rented printing service
(CUPS) in its own protocol, over its own socket or an address on this machine:
`ipp.rs` and `http.rs` are every byte it sends, and nothing starts a program.
`find` lists network (DNS-SD, IPP) and cable (USB, IPP-over-USB) printers and
adds nothing. `set_up` adds the one a person chose, driverless only, under a
derived queue, and makes it the printer this machine prints on. A printer that
needs its maker's program is refused in a sentence. `Stopped` is out of paper,
jammed, out of ink, refused the job, or `NotAnswering { tried }`, read from
`printer-state-reasons`; anything else is *not answering*, with no code.
`print_document` is a change under a grant over the document. `print` refuses a
printer across the network unless it holds the `alo_egress::Departing` for that
exact egress (`Why::Sending`, the printer's host, the authorised agent). The
plan named `Errand`, but that is egress with no agent behind it; the report
explains the choice. 28 strings collected by `alo-saying`; CUPS added to its
list of rented names; the verb is answered in `docs/by-hand.md` and the
contract. Report: `docs/autonomy/updates/printers-found-set-up-and-said-what-is-wrong.md`.
Owed: whether cupsd accepts `everywhere` itself, rather than only through
`lpadmin` (unmeasured, and the first thing to check on a machine); a real printer
on a certified machine; CUPS in the image; and the agent finding and setting up a
printer (no grant covers a device yet).

★ *Printers, solved — found, set up, and fixed when they stop.* The star is on
the last clause. Finding a printer is a solved problem everywhere; **saying
what is wrong in a sentence a person can act on** is solved nowhere, and it is
the entire reason printing is on a list of promises.

- **Acceptance:** `alo-printing` finds printers on the local network and over
  USB and sets one up without a person choosing a driver, a queue or a
  protocol — the machinery has names and a person meets none of them; printing
  is an `alo_capability` verb, and a printer that is **not on this machine**
  makes it an `alo_egress::Errand` so the indicator fires, because a document
  crossing the network to a printer is a document leaving; and when a printer
  stops, `alo-printing` says **which of a closed set of things is wrong** — out
  of paper, jammed, out of ink, refused the job, not answering — each with what
  a person does about it, and a state that is none of them is reported as
  *not answering* with what was tried, never as a code.
- **Constraint:** CUPS is rented, configured, unpatched. No driver is written
  here and no vendor's installer is run. Nothing here draws a print dialogue —
  that is the shell's, and this hands it the decisions. A printer is never
  added silently: a machine that acquired a printer by itself has made a place
  a document can go without anybody choosing it.

### 4. "I can't open this file", said properly

**Status:** ready. **Depends on:** 2, 3.

**Done, 2026-09-17.** `crates/alo-opening`: every `Cannot` now answers
`would()` with one of a closed set of five `Would`s — *a complete copy* (empty,
damaged), *a copy without the password*, *the document itself* (a program),
*another machine or a different format* (nothing here opens it), and *whoever
made it* (not recognised, where this machine cannot name a machine that would).
`Cannot::explained` is what a person reads — what the file is and why, then what
would — and `Outcome::said` says exactly that for `CannotOpen`, so every caller
(`alo-converting`, `alo-printing`, `Decided`) carries the way out without a
change of its own. Damaged and not recognised answer differently, and a test
holds it; no sentence names a library, a media type, a code or a rented engine,
and none offers to upload, look up or send the file anywhere. A converted
document whose name lies and that cannot be opened now keeps the whole
explanation. The refusal is recorded like any other outcome: a document the
converting verb cannot open is recorded with the explanation the person read,
and the record answers it by the file's own path. 5 strings added (40), collected
by `alo-saying`. Report:
`docs/autonomy/updates/i-cannot-open-this-file-said-properly.md`. Owed: the
open-with portal still says only the reason (`alo-applications`'
`NothingOpens::TheFile` returns one sentence) and its answers file keeps
`the-file` without which reason — both another plan's crates; the report
proposes the change. For task 5: the sentences are in `THE_REMEDIES`.

★ and the sharpest line in the group. Every system has this moment; every
system spends it badly — an error code, a dialogue with one button, or worst,
an application that opens and shows nothing. The promise is that the moment is
**useful**.

- **Acceptance:** for a file task 1 says cannot be opened, `alo-opening`
  produces a sentence naming **what it is, why this machine cannot open it, and
  what would** — another machine, a different format, a person who has the
  original — with a test per reason; the sentence never names a library, a MIME
  type, a return code or a rented engine; a file that is **damaged** is said to
  be damaged rather than unsupported, because those send a person in opposite
  directions; and the refusal is recorded like any other outcome, so *what
  happened when I opened that* is answerable afterwards.
- **Constraint:** no guessing. A file that might be openable is not opened
  hopefully to see — task 1's set has no *probably* and this task inherits
  that. Nothing here offers to send the file anywhere to find out; an offer to
  upload is the sentence this task exists to replace.

### 5. Every sentence this makes, and the walk through them

**Status:** ready. **Depends on:** 1, 2, 3, 4.

**Done, 2026-09-17.** `crates/alo-converting/tests/the_walk_through_documents_and_paper.rs`
walks a PDF arriving, the owner's `sample.docx` arriving and being converted
through the real service and engine, a printer on the network found, set up and
printed on with the indicator lit, the printer out of paper and ready again, and
a Word document cut short arriving. It compares every sentence, in order, with
the 19-row table it parses out of the report. So a sentence that changes without
the table fails, and so does a table that says what the machine does not.
`every_sentence_about_documents_and_paper_carries_a_note.rs` holds that the three
crates' words are exactly what `alo-saying` collects, each with a note that names
every gap (four `{printer}` notes were fixed).
`no_english_outside_the_vocabulary_in_documents_and_paper.rs` reads the shipped
source of all three crates. Its 20 exceptions are each argued: the service's
log, `Display`s mapped to a worded refusal, the wire protocol, and an HTTP head.
One untrue sentence changed: a print refused while the printer is out of paper,
ink or not answering promised *what was waiting prints*, and nothing was waiting.
Those sentences now say *documents it has already taken*, and a new
`printing.not-taken` tells the person to print it again. Report:
`docs/autonomy/updates/every-sentence-about-documents-and-paper-and-the-walk-through-them.md`.
A later change to a sentence along the walk publishes the table again in a
follow-up report and points the test's `THE_REPORT` at it.

The four tasks above end in sentences a person reads at the exact moment they
are already frustrated, which is the moment a system is judged.

- **Acceptance:** every sentence these crates can say is in the vocabulary with
  a translator's note, and a walk from *a file arrives* through *it is opened*,
  *it is converted and here is what that cost*, *it is printed*, *the printer
  stopped and here is why*, to *this cannot be opened and here is what would*
  produces the exact sequence a person meets, recorded verbatim in the report
  as a table and held by one test that fails if any sentence changes without
  the table changing; and a test reads the shipped source of both crates for
  English written outside `alo-strings` (`CLAUDE.md`: hardcoded English is a
  bug in a European product).
- **Constraint:** nothing here re-decides what the sentences describe. If a
  sentence is true and reads badly the sentence changes; if it reads well and
  is not true it changes the other way.

### 6. A `.pages`, a `.heic` and a `.dwg` — recognised, and converted or explained

**Status:** blocked — on
`docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md`
(proposed, 2026-09-19). **Depends on:** 1, 4, 5.

**One of the three is taken, 2026-09-19 — the photo.** The other two, a
`.pages` and a `.dwg`, are left for a reason that is not work, which is what
the decision above is about. The task stays blocked while it is proposed: the
third that is finished does not make the whole of it offerable, and a loop that
took it would find nothing it could honestly do.

**The photo.** `alo-opening` recognises a HEIF photograph from its `ftyp` brand
and reports it as `Kind::HeicPhoto`, *a photo in the format telephones save*. No
engine the image pins converts one, so it takes task 4's `NothingHereOpens` road
with *another machine or format*. Tested against a **real file with its
provenance** — `crates/alo-opening/tests/files/`, written by macOS's own ImageIO
from this repository's own artwork, its digest recorded and checked by the test
that reads it. Seven tests.

**It was worse than a shrug, and that is why this was taken first.** The plan was
written when all three read as *not recognised*. By 2026-09-19 a HEIC read as **a
film**: the media kinds added for ADR 0051 gave `ftyp` a meaning, and everything
that was not `M4A`/`M4B` fell through to `Mp4Video`. HEIF and MP4 are one
container and the brand is all that separates them. A machine confidently calling
somebody's photograph a film is a worse failure than one admitting it does not
know.

**What is left, and what it needs.** A `.pages` and a `.dwg` need **one real file
each**, saved by somebody who has the program — the way the owner saved the three
Office documents on 2026-09-16. Neither can be produced or obtained on the Mac
lane's machine with provenance anybody could check: Pages is not installed on it,
nothing there draws in AutoCAD's format, and a file pulled off the web to make a
test pass has exactly the provenance this acceptance exists to refuse. **Task 5's
walk gains the photo when the other two land** — that walk runs the office engine
and cannot run on an aarch64 gate, and a test nobody can run is not a test to
edit blind. Written up in
[A photo from a telephone](updates/a-photo-from-a-telephone.md).
**Depends on:** 1, 4, 5.

**The photograph stopped being blocked while this was being written.** The
worker measured, correctly, that no `.heic` with provenance existed on any
machine this team has — and while it worked, the Mac saved one from its own
ImageIO and took that third of the task. So `alo-opening` recognises a
photograph rather than merely declining to call it a film, and the tests written
against the older premise were dropped in favour of the ones written against a
real file (`tests/a_photo_from_a_telephone.rs`). What survives from this side is
the part that was better: `crate::iso_media`, which decides on the brands a file
carries rather than on its header alone, with the refusal that a file naming
none of the family it knows is not claimed to be anything.

**The other two are blocked, and two machines reached that separately.**
While the photo was being taken here, another lane measured the same task
from the other end and found what stops it: a `.pages` and a `.dwg` with
real provenance exist on no machine this team has. Both accounts are kept
below, because they were arrived at independently and agree — which is worth
more than either on its own.

**Status:** blocked — on `docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md`
(proposed, 2026-09-19). **Depends on:** 1, 4, 5.

**Decided rather than built, 2026-09-19.** The worker found that the one thing
this task cannot be finished without — *a real file with its provenance*, one of
each — exists on no machine this team has, and that was measured rather than
assumed: no `.heic`, `.pages` or `.dwg` anywhere on the development PC or in the
Linux tree the gates run in; no encoder for any of them installed there; no HEIF
encoder registered with Windows on the development PC; and the rented office
engine reads a Pages document and saves none of the three. The Mac that could
save one is stopped. The three ways out — wait for a real file, build one here
to the specification, or borrow somebody else's — decide between leaving the
star in `docs/features.md` unmet in v0.5 and shipping a rule whose only evidence
is that it agrees with its own author, and neither is a worker's to pick. ADR
0057 sets out the three options, what each costs, and recommends the first; it
also decides the parts that do not depend on a file, so that the work is
mechanical the day they arrive — what each of the three is recognised by, what
each one is called, that a Pages document **converts** through the engine
already pinned (a registration, in ADR 0039's own words, rather than a new
engine), and that a photograph is **explained** rather than converted because
the image ships no software decoder for a format somebody still licenses.
**That answer arrived on 2026-09-19 and does not change this one:**
[ADR 0058](../decisions/0058-which-software-decoders-the-image-ships.md) keeps
HEVC out of the image and leaves it to the silicon, so a HEIC photograph on a
machine without a hardware decoder is still explained rather than converted.
It lists the three things that must happen before this task is ready
again: the owner's answer, one real file of each with its provenance, and task
5's walk and table published again in a follow-up report.
`crates/alo-opening/tests/recognising_three_more_formats_waits_on_its_decision.rs`
holds the ADR in place and fails if any of the three is recognised while it
still says *proposed*.

**One wrong answer did not wait, 2026-09-19.** This task's premise says the
three are met today as *this machine does not recognise what this file is*. For
the photograph that was not true: `crates/alo-opening/src/looking.rs` took the
four bytes `ftyp` at offset four for the container MP4 names, so a photograph
from a telephone was reported as **an MP4 video** and a person reading that
would go looking for something to play it with. The header is a family, not a
format; `crates/alo-opening/src/iso_media.rs` now reads the brands after it, and
a file whose brands name none of the ones this machine knows is *not
recognised*. It does not recognise a photograph — it stops this machine saying
something untrue about one. Report:
`docs/autonomy/updates/a-photograph-is-not-a-film-and-three-formats-wait-on-a-real-file.md`.

`docs/features.md`: ★ *"I can't open this file." A `.pages`, a `.heic`, a
`.dwg`: the system converts it where it can, and where it cannot says plainly
what will open it, instead of shrugging.* Tasks 1 and 4 made the shrug
impossible, but the three files the promise names are still met as *this machine
does not recognise what this file is*. That is honest, but it is not the
sentence the promise describes, because this machine could know what each of
them is from its bytes. A photo from a phone is the commonest of the three.

- **Acceptance:** `alo-opening` recognises a Pages document, a HEIC photo and a
  DWG drawing **from their content**. A HEIC is the ISO media container with its
  own brand, a Pages document is a zip holding a Pages index, and a DWG has its
  version marker. Each is recognised by that and never by its extension. Each
  becomes a `Kind` with a name a person uses (*a Pages document from a Mac*, *a
  photo in the format iPhones save*, *an AutoCAD drawing*), and it lands in
  task 1's closed set. Where nothing on this machine opens or converts it, it is
  `NothingHereOpens` with task 4's *what would*. Where a rented engine the image
  already pins converts one on this machine, it is `Converts` with its costs,
  registered the way task 2 registers a conversion. Each kind is tested against
  **a real file with its provenance** beside it, as task 2's documents are, and
  never a synthesised header alone. A file that is none of these still says *not
  recognised*. Task 5's walk and its table gain the photo, published in a
  follow-up report.
- **Constraint:** no new engine is added to the image without an ADR. A format
  this machine cannot convert here is explained, never uploaded to be converted.
  Nothing names Apple's, Autodesk's or anybody's program as *what would open
  it*: *a Mac*, *a drawing program* and *a copy saved as a PDF* are places a
  person can go, and a product name is an advertisement.

### 7. An OpenDocument text, spreadsheet and presentation — converted, and what each copy lost

**Status:** ready. **Depends on:** 1, 2, 5.

Task 2's own *Owed* names this: **older Office and OpenDocument files**. The
two halves are not equally ready, and this task is deliberately the half that
is. ADR 0039 fixed the conversions at three *and* said each further kind is "a
registration and a test against a real file, in a later change" — this is that
change, for the three open-standard formats. The engine pinned in the image
reads and writes them, so the real file this task is measured against can be
saved **by the application people actually use for these files**, on the machine
that gates it, which is why it is not waiting on anybody. The three older
Microsoft formats are a later task and wait on real documents from the owner,
for the reason ADR 0057 sets out: a `.doc` saved by something that is not Word
is not evidence about the `.doc` files people are sent.

`docs/features.md`: ★ *"I can't open this file" — converted, or plainly
explained.* A person sent an `.odt` by a European public body meets this on
their first morning; today it is recognised, named, and then refused.

- **Acceptance:** an OpenDocument text document, spreadsheet and presentation
  are each converted to a PDF copy through the same service and the same pinned
  engine task 2 built — three more conversions in ADR 0039's closed set, each
  with its word on the socket, its name in the scratch folder, its line in the
  contract and `docs/by-hand.md`, and **no new engine** in the image; what each
  copy could not carry is said by name through task 2's `Carried`, with *lost
  nothing* reachable only after both files were inventoried, exactly as it is
  for a `.docx`; each is tested against a real file with its provenance beside
  it, saved by the pinned engine on the machine that gates this task and
  recorded in a `README.md` the way `crates/alo-converting/tests/documents/`
  records its three; and a document carrying a macro library says so, because
  an OpenDocument keeps macros where `alo-opening` can already see them.
- **Constraint:** the engine is rented and unpatched (ADR 0011), and a
  conversion that skips itself where the engine is missing is refused (ADR
  0039's own rule). Nothing is uploaded. No sentence changes without task 5's
  table changing with it, published again in a follow-up report.
