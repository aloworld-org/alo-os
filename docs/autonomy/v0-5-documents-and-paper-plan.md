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

**Status:** **Done, 2026-09-21.** Recognition was already done — all three
formats from their own bytes, each measured against a real file held to its
digest. What was left was the Pages document's **conversion**, which waited on
an inventory nobody had taken; the measurement was taken on 2026-09-21 and the
conversion wired in the same day, by the third PC (`AGAI01`) at the owner's
instruction. It is not this plan's machine: the lane table gives this plan to
the development PC, and the owner assigned this one task across because the
engine that reads a Pages document is an **x86_64** build and this plan gates on
aarch64. Nothing else about the plan moves.

**The seventh conversion is in the set.** `Conversion::EVERY` holds it,
`Conversion::HELD_BACK` is empty, and a Pages document is a file this machine
opens by converting a PDF copy of it. **Its inventory is read from what the
engine reads of it** — `crates/alo-converting/src/inventory/read_from.rs`,
because a Pages document keeps its content in `Index/*.iwa` and nothing here
reads one — and the copy is still made from the original, never from the
rendering. Measured end to end against the real document and the real engine in
`converting_a_real_document.rs`: the two families it is set in, `Helvetica` and
`HelveticaNeue`, are named as substituted, and nothing else is, because the
engine's own substituted declarations are not families text is set in. **Task
5's walk gains the photograph and the Pages document**, and its table is
republished in the report below, which the walk's test now reads. Report:
[A Pages document is converted, and a photograph is explained](updates/a-pages-document-is-converted-and-a-photograph-is-explained.md).
**One thing this did not settle is task 8's**: no Keynote or Numbers document
with provenance exists on any machine this team has.

**The inventory of a Pages document, measured 2026-09-21.**
`crates/alo-opening/tests/files/document.pages`, 227,583 bytes, held to
`sha256:1b01189904934a7c5d6b59199c9719eab111759cfea5e2a7de17d3e45ee09c4d`, read
by **LibreOffice 26.2 on x86_64** (exit code 0, taken from a file the run wrote
rather than from `$?`, which does not survive the Windows-to-WSL boundary). All
six of `WHAT_AN_INVENTORY_ANSWERS`, from what the engine produced rather than
from what the format is assumed to hold:

| What an inventory answers | This document |
|---|---|
| every family its text is set in | **5** — `Helvetica` and `HelveticaNeue` are the document's own; `Liberation Sans`, `Liberation Serif` and `Noto Sans` are the engine's substitutes |
| every field whose value depends on when or where it is open | **0** |
| every kind of content taken from elsewhere | **0 taken from elsewhere**; one picture *embedded* (`Pictures/1000…7E.jpg`, the original's `Data/pasted-image-24.jpeg`) |
| whether it has comments | **no** — and the original carries `Index/AnnotationAuthorStorage-1732609.iwa` at 23 bytes, so the part exists and is empty |
| whether it has tracked changes | **no** |
| whether it carries macros | **no** |

The original holds 15 parts, two of them pasted images.

**Two things in that measurement are the point of taking it rather than assuming
it.** The fonts answer is a real conversion cost under ADR 0008: the document is
set in Helvetica and the engine silently substitutes Liberation and Noto, and a
person is owed that sentence. And *taken from elsewhere* is **0 while an embedded
picture is present** — the first reading of this measurement counted
`Pictures/…` as linked and reported 1, which is the opposite answer for a person
deciding whether a copy is complete offline. Embedded and linked are separated
here because conflating them is exactly the proxy this project has shipped
defects from.
[ADR 0057](../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
is **accepted, 2026-09-20**, as option A, and what answered it is that all three
files exist with their provenance. The fourth conversion is written whole and
held back rather than undecided: see *The fourth conversion*, 2026-09-20, below.

**The drawing, 2026-09-20 — the last of the three.** `Kind::AutocadDrawing`,
*an AutoCAD drawing*, recognised from the six characters every one of them
begins with. A drawing has no signature and no container: those six are the
version and there is nothing after them to check the guess against, so the rule
is a **closed list of the released versions** — `AC1012` through `AC1032`,
written out one by one — and never *anything beginning `AC`*. A rule that
matched more than the evidence supports is what ADR 0057 exists to refuse, and a
pattern is how it would have got in. Measured against a real file: a bench plan
drawn by us with `ezdxf` and **written as DWG by ODA File Converter 27.1**, the
Open Design Alliance's own converter, round-tripped back to DXF to check that
every layer, entity type and both text strings survived. It is **explained, not
converted** — nothing this image pins reads the format — so it takes task 4's
`NothingHereOpens` road with *another machine or format*, the same road as the
photograph, and that is proved twice in
`crates/alo-opening/tests/a_drawing_from_a_cad_program.rs`: against a machine
told it converts **every other kind there is**, and against the registry that
decides which kinds have a conversion at all. Written up in
[A drawing from a CAD program](updates/a-drawing-from-a-cad-program.md).

**The Pages document, 2026-09-19**, once the owner installed Pages.
`Kind::PagesDocument`, *a Pages document*, recognised from the parts its
container holds — never from `PK\x03\x04`, which would call a Word document a
Pages document, and never from the extension. Measured against a real file saved
by Pages 15.3.1 with this repository's own words and its own picture in it, held
to its digest so swapping the file fails a test.
`crates/alo-opening/tests/a_document_from_pages.rs` holds it in both directions:
the real document is recognised, and the real `sample.docx`, `sample.xlsx` and
`sample.pptx` are still themselves.

**Its conversion is measured, and deliberately not wired.** Three states, because
collapsing them is how a promise gets made that a machine cannot keep:

- **Proven possible.** The owner ran this document through the engine inside the
  **signed 0.0.3 image**, digest-verified — `soffice --convert-to odt` via filter
  `writer8`, **778 characters of its own text recovered**. So
  [ADR 0057](../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
  stands on this point: it converts through the engine already pinned, as a
  registration in ADR 0039's words. No iWork reader and no amendment to ADR 0039
  are needed. **The claim is one file wide** — it says *this document converts*,
  not that `libetonyek`'s IWA path works in general.
- **Not wired when this was written**, because the converter in the shipped
  image could not start: twelve runtime libraries were missing from 0.0.3 as
  published, eleven from 0.0.2. No document of any format converted on a real
  machine, `.docx` included, and ADR 0039's promise was unmet in the product
  from the day the converter landed. Registering a conversion into that would
  have made the machine say *this converts* and then fail everywhere, which
  ADR 0039 §1 forbids by name.

  **That condition is gone as of 2026-09-20.** Release **0.0.4** carries the
  twelve libraries and is signed and pinned, and its build converts a document
  as a build step and fails if nothing comes out — so a release that builds is
  a release whose engine ran. Measured on the pushed image: no missing
  libraries, and the Pages document above converted with 778 characters of text
  recovered.

  **So the wiring is unblocked, and what remains is the inventory.** ADR 0039 §4
  will not show a copy whose cost was not measured, and measuring what
  `libetonyek` gives for a Pages document means running the engine — x86_64
  only, which is the same wall that keeps task 5's walk off the Mac.
- **Why nobody caught it:** the recipe ends its converter step with
  `test -x …/soffice`, **which checks the executable bit rather than that it
  runs.** The fix is lane A's, in `image/`, and needs 0.0.4.

**The photo, 2026-09-19.** `Kind::HeicPhoto`, recognised from its `ftyp` brand,
*a photo in the format telephones save*. Nothing the image pins converts one, so
it takes task 4's `NothingHereOpens` road with *another machine or format*. It was
worse than the shrug this plan describes when it was found: a HEIC read as **a
film**, because the media kinds added for ADR 0051 gave `ftyp` a meaning and
everything that was not `M4A`/`M4B` fell through to `Mp4Video`.

**The Pages document, 2026-09-19**, once the owner installed Pages.
`Kind::PagesDocument`, *a Pages document*, recognised from the parts the
container holds and never from `PK\x03\x04` — which would call a Word document
a Pages document — nor from the extension.
`crates/alo-opening/tests/a_document_from_pages.rs` holds it in both directions:
the real document is recognised, and the real `sample.docx`, `sample.xlsx` and
`sample.pptx` in `alo-converting` are still themselves. Measured against a real
file saved by Pages 15.3.1 with our own words and our own picture in it, held to
its digest.

**The conversion is measured, and deliberately not wired.** Three sentences,
because they are three different states and collapsing them is how a promise gets
made that a machine cannot keep:

- **Recognition: done**, measured against a real file with its provenance.
- **Conversion: proven possible.** Measured 2026-09-19 by the owner, inside the
  **signed 0.0.3 image**, on this exact file digest-verified from the branch —
  `soffice --convert-to odt` through filter `writer8`, **778 characters of this
  document's own text recovered**. So
  [ADR 0057](../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
  stands on this point: it converts through the engine already pinned, as a
  registration in ADR 0039's words. No iWork reader and no amendment to ADR 0039
  are needed. **The claim is one file wide** — it says *this document converts*,
  not that `libetonyek`'s IWA path works in general.
- **Conversion: not wired when this was written**, because the converter in the
  shipped image could not start. Twelve runtime libraries were missing from
  0.0.3 as published, eleven from 0.0.2, so no document of any format converted
  on a real machine, `.docx` included — ADR 0039's promise was unmet in the
  product from the day the converter landed. Registering a conversion into that
  would have made the machine say *this converts* and then fail everywhere,
  which ADR 0039 §1 forbids by name.

  **Unblocked 2026-09-20 by release 0.0.4**, which carries the twelve libraries,
  is signed and pinned, and converts a document as a build step so that a
  release which builds is a release whose engine ran. What is left before the
  wiring is the inventory ADR 0039 §4 requires, and that needs the engine — so
  it needs an x86_64 machine.
  Nobody caught it because the recipe ends its converter step with
  `test -x …/soffice`, **which checks the executable bit rather than that it
  runs.** The fix is lane A's, in `image/`, and needs 0.0.4.

**The fourth conversion: written whole and held back, 2026-09-20.** Both things
the bullet above waits on have moved, and in opposite directions.

**The image blocker is cleared.** `image/Containerfile` pins **0.0.4**, and the
converter step now runs `ldd` for a missing library and **converts a real file**
before the build continues, in place of the `test -x` that checked the
executable bit. The recipe proves the converter starts.

**So the remaining blocker is the inventory, and it is a different one.**
ADR 0039 §4 makes the inventory of the original the step before any copy, and no
Pages document has been inventoried, because the engine that reads one is an
x86_64 build and this repository gates on aarch64. Registering the conversion
without one would make the machine say *this converts* and then refuse at the
inventory — ADR 0039 §1's named failure, reached one step earlier than before.

`Conversion::PagesDocument` is therefore written complete — its kind, its word
`pages-document` on the socket, Writer's export filter, `document.pages` in the
scratch folder — and put in `Conversion::HELD_BACK` rather than
`Conversion::EVERY`, whose being empty is the finished state. All three roads
into a conversion are shut in the code and tested shut: `Conversion::of` answers
`None` for the kind, `Conversion::asked` does not answer to the word, and
`with_what_converts` does not announce it, so a machine with the service
answering still says *nothing here opens* a real Pages document.
`crates/alo-converting/src/inventory/pages.rs` states the inventory's shape —
the fifteen parts read off the real document, the six things an `Original`
holds, and `Measured::NothingOnThisMachine`, one value the way ADR 0051's open
counsel question has one — **with no numbers in it**, and its test fails the day
somebody measures one. Written up in
[The fourth conversion, written whole and held back](updates/the-fourth-conversion-written-and-held-back.md).

**What is left, 2026-09-20.** Not the drawing, and not recognition — both are
done. The **inventory measurement** above, which needs the x86_64 engine; and —
to make one reasoned rule measured — one Keynote and one Numbers document,
because all three iWork applications write the same container and the exclusion
of the other two has never been checked against a real file of either. Task 5's
walk gains the Pages
document when it can be run: it runs the office engine and cannot run on an
aarch64 gate. Written up in
[A document from Pages](updates/a-document-from-pages.md).
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

**The account below was written by another lane on 2026-09-19, before the Pages
document existed.** It is kept in full because it was arrived at independently and
its reasoning still holds for the drawing; where it says all three wait on a real
file, two of them no longer do. The status above is the current one.

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

**Status:** **Done, 2026-09-20.** **Depends on:** 1, 2, 5.

Report:
[An OpenDocument converted, and a column that nearly told a lie](updates/an-opendocument-converted.md).
`Conversion::EVERY` holds **six**: the three current Office formats and the
three open-standard ones, each into a PDF, each with its word on the socket
(`opendocument-text`, `opendocument-spreadsheet`, `opendocument-presentation`),
its name in the scratch folder and its export filter — **no new engine**, and
the closed-set rule untouched, which is ADR 0039's own "a registration and a
test against a real file, in a later change".

**Four real files, saved on the machine that gates this**, by the application
people write these with, running headless: `sample.odt`, `sample.ods`,
`sample.odp` and `sample-with-a-macro.odt`, with their digests and provenance in
`crates/alo-converting/tests/documents/README.md`. The macro-carrying one is the
first document in this repository that can prove the sentence *macros were not
run* against a real file — the three Office documents deliberately carry none.

**The originals are measured here; the copies are not.** The new
`crate::inventory::opendocument` reads all three formats — one file, because
they are one format with three bodies — and its tests hold each document to what
it contains, on this gate. The conversion itself needs the pinned x86_64 engine,
so the four new conversion tests join the accepted failing set on an aarch64
gate and become measurements the first time the suite runs where the engine is.

**Two things only a real file said.** A spreadsheet's text finds its family
through the **column**, not the cell: `sample.ods` is set in Garamond and no cell
in it names a style at all. A reader that looked only at what was open around
the text found no family, and would have reported a conversion that substituted
every font in that sheet as having lost nothing. And a presentation writes its
comment in a namespace of its own, which costs nothing only because ADR 0039 §5's
reader hands on local names and never a prefix.

**A break on `main` was found and fixed on the way** — one this task did not
cause but did uncover, together with the gate practice that hid it. See the
report; the short version is that `cargo test` stops at the first failing test
binary, and with an accepted failing set in the tree that makes every binary
after it unproven, so `--no-fail-fast` is not optional here.

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

### 8. The iWork exclusion, measured against a real Keynote and a real Numbers document

**Status:** ready — **blocked on a real file of each.** **Depends on:** 6.

**One route that does not need the Mac's applications**, recorded on
2026-09-25 in [what closes this release, and in what
order](updates/what-closes-v0-0-5-and-in-what-order.md): Keynote and Numbers
on iCloud, in a browser on any machine, are the real applications writing the
real container. If that route is taken the provenance line says it was the web
version — which writer wrote a fixture is the whole value of recording
provenance. This stays inside v0.0.5.

Written 2026-09-21, by the lane that finished task 6, because it is the one
thing that task measured and could not close. A finished task's body is not
where a lane looking for free work looks, and leaving it there is how a
takeable piece of work becomes invisible (`docs/autonomy/SHARED_MAIN.md`, *A
stale blocker is invisible work*).

**What is unmeasured.** All three iWork applications write the same container:
`Index/Document.iwa` beside a stylesheet and a metadata plist, in one zip.
`crates/alo-opening/src/zip.rs` calls one of them a Pages document and excludes
the other two by parts only they have — a Keynote presentation's slides, masters
or theme, a Numbers spreadsheet's tables. **The inclusion is measured against a
real file and the exclusion is not**, because no `.key` and no `.numbers` with
provenance exists on any machine this team has. The rule as written says *an
iWork document is a Pages document unless it carries a part I listed*, and
nobody has ever handed it a document that should take the *unless*.

Task 6's own note says why that matters rather than being tidy: turning the rule
round — *unless* — is what would make it answer *this is a Pages document* about
a Keynote presentation written by a version whose parts nobody here listed. The
exclusion is the half that protects a person from being told their presentation
converts and then watching it fail, and it has never been shown working.

**Why it is blocked and not merely undone.** It needs one real Keynote
presentation and one real Numbers spreadsheet, each saved by the application
itself and each with its provenance recorded beside it, exactly as
`crates/alo-opening/tests/files/README.md` records the three files task 6
measured. [ADR 0057](../decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
is accepted as option A — **wait for a real file** — so synthesising a
container that carries the parts the rule looks for is not a way round this. It
would prove that the rule agrees with its own author, which is the thing that
decision exists to refuse.

**Who can unblock it:** the Mac lane, which saved `document.pages` with Pages
15.3.1 and has both other applications available on the same terms. It is
minutes of work on that machine and impossible on any other.

- **Acceptance:** `crates/alo-opening/tests/files/` holds one real `.key` and
  one real `.numbers`, each saved by Keynote and by Numbers on a Mac, each held
  to its digest with its provenance in that folder's `README.md` — what wrote
  it, when, what is in it, and what it is not. `alo-opening` is shown, against
  each, **not** calling it a Pages document: a Keynote presentation and a
  Numbers spreadsheet each come out as *not recognised* rather than as
  something this machine would offer to convert, and the real `document.pages`
  beside them is still a Pages document. Whether either becomes a `Kind` of its
  own is **not** part of this task: naming a format is a promise about what can
  be done with it, and that is a separate change with its own measurement.
- **Constraint:** no synthesised container, and no rule widened to make one
  pass. If the two real files show the exclusion is wrong — a version whose
  parts are named differently — the finding is the deliverable and the rule
  changes to match the evidence, never the other way round. Nothing names
  Apple's applications where a person reads.

### 9. A machine with no engine says so, instead of failing

**Status:** **Done, 2026-09-22**, by the third PC (`AGAI01`) at the owner's
instruction — not this plan's machine: the lane table gives this plan to the
development PC, and it was taken here because the condition can be *produced*
here. **Depends on:** 2. Report:
[A machine with no engine says so](updates/a-machine-with-no-engine-says-so.md).

**It is ten, and the nine below were a truncated measurement.**
`which-tests-need-the-engine.sh` runs `cargo test -p alo-converting`, which
**stops at the first test binary that fails** — so the run ended inside
`converting_a_real_document.rs` and never reached
`the_walk_through_documents_and_paper.rs`. Measured again on 2026-09-22 with
`--no-fail-fast`, the two machines agree: the tenth is
`the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table`,
which runs the engine at steps 4 and 5. Nothing about the Mac's architecture
was involved, and the difference this plan left open is closed.

**What was built.** `crates/alo-converting/tests/asking/mod.rs` asks whether
this machine can run the engine by **running it** — `engine.rs`'s own argument
list and cleared environment, with the question *what are you* — and answers a
run that could not be started, one that never answered, one that ended badly
and one that said nothing as four named reasons, each naming where the engine
was looked for. Each of the ten asks it first and, where it cannot, skips itself
and prints what was missing. `a_machine_that_cannot_run_the_engine_says_so.rs`
holds the four refusals, a wrapper over a binary that cannot run — the aarch64
shape, and the case `test -x` gets wrong — a program stopped for never
answering, and the list itself: exactly the tests that need the engine ask, read
off the sources both ways, so neither a conversion test without the ask nor an
ask on a test that does not need one can be added quietly.

**The decision it needed.** ADR 0039 forbade this in as many words, so the code
could not be written without amending it:
[ADR 0063](../decisions/0063-a-machine-that-cannot-run-the-engine-says-so-rather-than-failing.md),
recording the owner's decision in this task and amending ADR 0039's last two
sentences on skipping and nothing else. An engine that starts and then converts
badly still fails loudly, which is the part of ADR 0039 that was always right.

**Measured, on this machine, with the engine moved aside and put back on every
exit path:** the whole crate green — 120 passed, 0 failed across its ten test
targets — with exactly ten `skipped:` lines, each naming where the engine was
looked for. With the engine present: the same 120 passed, 0 failed, and nothing
skipped.

Every run on the Mac fails the same tests, because the engine `alo-converting`
drives is an **x86_64 build** and the Mac is aarch64. A suite that is red on one
machine for a reason nobody can fix there is a suite people learn to read past,
and the next real failure hides inside it.

**Which tests, measured rather than read.** Guessing from the sources which
tests need the engine is the kind of proxy this plan exists to refuse, so the
condition was reproduced on x86_64: the engine's directory was moved aside, the
suite run, and the failures recorded — then the engine was put back, on every
exit path including an interrupt.
`C:\dev\setup\which-tests-need-the-engine.sh` on the third PC is the run. **Nine
tests need it**, all in `tests/converting_a_real_document.rs`:
`a_document_that_loses_nothing_says_so`,
`a_document_with_a_macro_library_says_the_macros_were_not_run`,
`a_pages_document_is_converted_and_the_families_it_is_set_in_are_named`,
`a_word_document_is_converted_and_what_it_lost_is_named`,
`an_excel_workbook_is_converted_and_what_it_lost_is_named`,
`a_powerpoint_presentation_is_converted_and_what_it_lost_is_named`, and the
three `an_opendocument_*_is_converted_and_what_it_lost_is_named`.

**The Mac reports ten and this measured nine, and the difference is not
resolved.** The tenth may fail there for a reason that is not the engine's
absence — a different architecture, not merely a missing file. So the check is
written against **the engine being unavailable**, which covers both, and the
worker reconciles the list against a real run on the Mac rather than assuming
the sets are the same.

- **Acceptance:** each of those tests asks whether the engine can be run, and
  when it cannot, **skips itself and prints why**, in the shape
  `crates/alo-in-use/tests/a_stream_through_the_media_server_is_listed.rs`
  already uses — whose own words are the rule: *a skip nobody can see is the
  same colour as a pass*. The reason names what was missing, so a person reading
  a green suite on the Mac can tell it is green for a stated reason. **On
  x86_64 every one of them still runs and still passes**, held by a run on a
  machine that has the engine. The asking is **not `test -x`**: this plan has
  already shipped a defect from exactly that proxy, where the executable bit
  stood in for *the converter runs*. It runs the engine and reads what it says.
- **Constraint:** **no blanket `cfg(target_arch)`** that makes the tests vanish
  on aarch64 — a test that disappears is indistinguishable from a test that
  passed, and the next architecture would inherit the silence. Nothing about
  what the tests assert changes; only what they do when the engine is not there.
  The engine is still never named where a person reads.

### 10. Older `.doc`, `.xls` and `.ppt` — converted, and what each copy lost

**Status:** **Done, 2026-09-25.** Wired, worded and asserted; the three
assertions **run on an x86_64 machine with the pinned engine and skip on an
aarch64 one**, which is what task 9 built that answer for. See *What was
measured and where* below. **Depends on:** 2, 7.

#### What was measured and where

**The pinned engine has no aarch64 build.** `THE_ENGINE` is
`/opt/libreoffice26.2/program/soffice` and `image/Containerfile` fetches the
`x86_64` RPM tarball; The Document Foundation publishes no aarch64 Linux build
at that address. So on the Mac lane's gate these three skip themselves by name,
exactly as the other ten do — and that, rather than any fault in the tests, is
the standing reason the converter tests were an accepted failing set there
before #103. `docs/quirks.md` carries it.

It is easy to look installed when it is not: Ubuntu's own package puts
**24.2.7.2, aarch64** at `/usr/bin/soffice`, and that one runs. ADR 0011 does
not let one pinned engine stand in for another, so what was measured with it is
reported as that build and not as the pin.

**What that build did measure, and what it changed.** Each of the three renders,
and the rendering carries exactly what the fixtures' README records: the Word
document's, the substituted family, one comment and a live date field; the
workbook's, the family, one comment and a formula; the presentation's, the
family alone. Rendering the workbook through the **prose** writer instead
returned no formula at all — a spreadsheet read by the prose reader has had its
cells fixed to their values before anything could count them. So
`engine::the_rendering` is shaped like the document it is of, and the test that
held every rendering to being a text document has become the one that holds each
to being an OpenDocument of its own shape. Without that, a copy of somebody's
workbook would have reported a lost font and said nothing about `=NOW()`
becoming a number.

**Two machines, decided 2026-09-25** and corrected the same day, in
[what closes this release, and in what order](updates/what-closes-v0-0-5-and-in-what-order.md):
**the three files are made on the development PC and the conversion is measured
on the Mac.** That is the split the fixtures' own `README.md` already records,
and it was checked against it rather than assumed:

- **The files:** saved by the repository's owner, from Word, Excel and
  PowerPoint, in their 97-2003 formats — exactly as `sample.docx`, `sample.xlsx`
  and `sample.pptx` were saved on that machine on 2026-09-16, each recording its
  own version in its own parts. The legacy files already on that machine are a
  published standards list and company invoice data; **neither may be published
  in the fixtures**, which is what this task requires, so three of the owner's
  own are written instead. Nothing synthesised.
- **The conversion:** the pinned engine **is already standing on the Mac**,
  which saved task 7's four OpenDocument fixtures with LibreOffice 24.2.7.2
  headless in the Lima VM that gates this repository. #122 said the engine was
  not installed and that standing it up was a step of this task; it is
  installed, on the machine that has run every conversion this plan has
  measured, and no second copy is stood up to avoid a handoff.

Written 2026-09-22 by the lane that finished task 9, because nothing followed it
and a plan that names no next task sends the loop back at work already done
(`docs/autonomy/SHARED_MAIN.md`).

The last half of a sentence this plan has already honoured once. ADR 0039's
*What this does not decide* names **older `.doc`, `.xls`, `.ppt` and
OpenDocument files** together, and says each further kind is *a registration and
a test with a real file, in a later change*. Task 7 did the OpenDocument three
exactly that way — `Conversion::EVERY` went from three to six, each with its own
word on the socket, its scratch name and its export filter, and no new engine.
These three are what is left of it, and task 2's own *Owed* line names them.

`alo-opening` already recognises all three from their bytes; nothing there needs
to change. What is missing is the conversion, and the engine's readers for the
older formats are writers this repository has not measured — a `.doc` comes out
of the same writer a `.docx` does, and that is a claim rather than a measurement
until a real file goes through it.

**What it is blocked on, precisely.** One real `.doc`, one real `.xls` and one
real `.ppt`, saved by the office application people send them from and owned by
the repository's owner so they can be published, with their provenance in the
`README.md` beside the three already in `crates/alo-converting/tests/documents/`.
The same blocker task 2 carried and the owner cleared on 2026-09-16, and the
same one task 8 carries now. Nothing synthesised: a container this repository
assembled would measure the assembler.

- **Acceptance:** each of the three is converted through the real service and
  the pinned engine on a machine that has one, into a PDF beside the original,
  which is unchanged byte-for-byte; what each copy could not carry is asserted
  **whole**, as the list its `README.md` records, and not searched for one
  entry; `Conversion::EVERY` grows to nine with a word on the socket, a scratch
  name and an export filter each, and the closed set stays closed — no `exec`,
  no filter chosen from anything a request carries. Each of the three new tests
  asks whether the engine runs and skips itself saying why where it cannot
  (task 9, ADR 0063), and `THE_TEN` in
  `crates/alo-converting/tests/a_machine_that_cannot_run_the_engine_says_so.rs`
  grows with them — that test fails until it does, which is the point of it.
- **Constraint:** the engine is rented and unpatched (ADR 0011), and a format
  it converts badly is a **finding and a sentence**, never a reader of our own
  guessing at the difference. If one of the three cannot be converted honestly,
  the deliverable is *this machine cannot open it* with the reason (task 4), and
  the format does not join `Conversion::EVERY`. Nothing is uploaded, and nothing
  names the engine where a person reads.
