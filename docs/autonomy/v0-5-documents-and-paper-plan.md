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

**Status:** ready. **Depends on:** 1.

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
