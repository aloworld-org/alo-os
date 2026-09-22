# ADR 0039 — A document is converted by a rented engine that can reach nothing

**Status:** accepted, 2026-09-15 — option **A**, with every decision under
*the recommendation* as written. Written 2026-09-14 by task 2 of
`docs/autonomy/v0-5-documents-and-paper-plan.md`. Of the three things below that
must happen before task 2 is built, the first is this answer; the pinned engine
on the gate machine and the three real documents remain, and task 2 stays
blocked on those two.
**Date:** 2026-09-14, accepted 2026-09-15
**Context:** `docs/autonomy/v0-5-documents-and-paper-plan.md` tasks 1, 2 and 4;
[ADR 0001](0001-the-capability-model.md) (no verb runs an arbitrary command);
[ADR 0006](0006-the-pinned-model-runtime.md) (a rented engine behind one file);
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (engines are
rented, configured, never patched);
[ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md) and
[ADR 0015](0015-the-kernel-learns-what-a-turn-is.md) (a turn's work is bounded
by the kernel); [ADR 0020](0020-a-question-is-carried-out-inside-the-turns-boundary.md)
(loopback is not checked); `crates/alo-bounding/src/lib.rs` (*what runs in the
cgroup, and why nothing is started*); `crates/alo-opening` (task 1's decision);
`docs/features.md` (*the documents people are actually sent open: .docx, .xlsx,
.pptx*; *a person never learns the name of anything we rented*).

## The question in one line

**Task 2 says a `.docx`, `.xlsx` or `.pptx` is opened "through the rented
converter running on this machine". Which converter, running as what, reached
how by a verb — and how does anybody know what the copy lost?**

## Why a worker could not simply choose

Every part of the answer is somebody else's settled rule, and the obvious code
breaks one of them.

1. **Nothing in this product starts a program today.** The one rented engine,
   the model runtime, is started by systemd and spoken to over a socket.
   `alo-bounding` says in its own documentation why a turn's work is one thread
   and nothing is spawned: *a program alo OS starts on an agent's behalf is one
   review away from a program an agent named*. A converter is a program. The
   first line of task 2 that calls `Command::new` from a verb is a change to how
   law 2 is kept, and that is not a worker's call.
2. **A program started from a turn is not known to stay inside the turn.** A
   turn's thread sits in a threaded control group. Where a process forked from
   that thread lands, and whether the kernel boundary follows it, has **not been
   measured** on the certified kernel. If it lands outside, the converter reads
   files with the person's whole authority and the kernel enforces nothing: a
   grant widened in fact while unchanged on paper, which ADR 0013 exists to make
   impossible.
3. **A document can ask to be fetched from elsewhere.** Office files carry
   linked pictures, linked data and remote templates by address, and an office
   engine resolves them when it converts. Converting would then cause egress
   nobody saw, breaking law 1 in a feature nobody would think to watch.
4. **The image pins no converter.** `image/Containerfile` has one download
   stage for a rented program, `alo-image` checks that one by name, and
   `alo-saying`'s list of everything we rent has no converter in it.
5. **The gate machine has no converter either**, and installing a package
   there is shared maintenance that `docs/autonomy/SHARED_MAIN.md` reserves for
   an explicit idle handoff between both workers. A test through the converter
   that fails on a machine without one is correct (`crates/alo-agentd/tests/`
   say why a test must not quietly skip), and it is also a gate no worker can
   pass today.
6. **"A real file rather than a synthesised one"** needs documents saved by the
   office applications people actually send them from. No office suite is on
   any machine this loop runs on, so no worker can make one.

## The options

All three rent the same engine. The only office engine that converts all three
formats on Linux, unmodified, under a licence the image can carry (MPL-2.0,
an aggregate under ADR 0012), is **LibreOffice** in its headless mode. The
options differ in **what runs it and how a document reaches it**. This ADR names
the engine because an ADR is where ADR 0006 named its own; no sentence a person
reads ever will.

### A — A converting service of our own, running the engine with no network and no files

A small system service of ours, `alo-convertd`, socket-activated, running as a
login of its own. It is started by systemd, never by `alo-agentd`. For each
request:

- **The verb's thread, inside the turn's boundary, opens the original read-only
  under the grant and creates the copy with `O_EXCL` in the folder the person
  chose.** It passes both **descriptors** over the socket. The service never
  learns a path, and cannot open one: its unit has no view of any home folder.
- The service copies the bytes into its own private scratch folder, runs the
  pinned engine with a **fixed argument list chosen from a closed set of three
  conversions**, reads the result, writes it to the copy's descriptor, empties
  its scratch folder, and closes both descriptors.
- **The unit has no network at all** (`PrivateNetwork=yes`, and every address
  family but Unix refused), so a linked picture or a remote template cannot be
  fetched. It is reported as not carried instead (below).
- **What was lost is found inside the service**, so a hostile document is parsed
  by an unprivileged login with nothing to reach, and never by the person's
  daemon.

**What it costs:** a second rented program on the image: a pinned, digest-checked
download stage, a unit, a login, and `alo-image` checks for all three, following
the model runtime's. About 300 MB on disk. A second hand-written protocol,
descriptors in and a report out, kept to one file the way ADR 0006 keeps the
runtime's. One engine start per conversion, measured in seconds on a laptop. It
also leaves one question for the gate: the test runs through the real service,
so the gate machine needs the engine present, and that is a shared-environment
install.

**What it keeps:** law 2 exactly as `alo-bounding` states it. Nothing is started
on an agent's behalf, and the program that runs is fixed by a unit file, not by
anything a turn sends. The grant stays enforced by the kernel, on the only reads
of the person's files that happen, which are the verb thread's. Law 1 is kept
structurally rather than by filtering.

### B — The verb's thread starts the engine itself, inside the turn

The verb spawns the engine with fixed arguments from inside the turn's control
group, and the kernel boundary bounds what it reads.

**What it costs:** it reverses `alo-bounding`'s stated reading of law 2 and the
test that holds it (`a_turn_is_this_thread.rs` reads that crate for spawning; a
new crate would escape that check, which is worse). It depends on the
**unmeasured** question in reason 2: if the child leaves the turn's group, the
converter runs unbounded with the person's authority. The engine inherits the
person's environment, writes a profile into their home folder, and is kept off
the network only by the turn's `socket_connect` hook. That hook does not check
loopback (ADR 0020), so a document could reach any service listening on this
machine. The gate question is the same as A's.

**What it keeps:** no new service and no new unit.

### C — The person's own office application converts, through an adapter

An office suite installed as a sandboxed application (ADR 0005) converts through
its automation interface, driven by an adapter (`docs/contracts/app-adapters.md`).

**What it costs:** a fresh machine opens none of the three formats, which quietly
narrows *the documents people are actually sent open* into *open if you install
something*. That is a promise narrowed, which this ADR may not do. The adapter
SDK it would stand on is not built. And what the copy lost would depend on
whichever version the person installed.

**What it keeps:** nothing is added to the image.

### Rejected outright

- **A converter of our own**, even for one format. Task 2's constraint, and
  ADR 0035's first decision applied to documents: the engine has twenty years of
  compatibility work and a rewrite is a race lost permanently.
- **Converting anywhere but this machine.** The plan's constraint: if a format
  could only be converted by a service, the v0.5 answer is *cannot open it* with
  the reason. **Nothing is uploaded**, not as a fallback and not with a setting.

## The recommendation

**A.** It is the only option that keeps law 1, law 2, the kernel-enforced grant
and the promise in `docs/features.md` all true at once, and its costs are the
ones this repository already pays once for the model runtime. B trades a
settled reading of law 2 for a saved unit file, on a kernel behaviour nobody has
measured. C is a narrowed promise.

### What is decided with it, so the code is wiring rather than choosing

1. **The copy is a PDF.** "Open" in *the documents people are sent open* means
   read what was sent, faithfully laid out. Editing belongs to an application.
   PDF is the one target whose rendering does not depend on another engine
   interpreting the copy again. `alo-opening` registers the three conversions as
   `ThisMachine::converts(kind, Kind::Pdf)` **only when the service answers**,
   so a machine without it says *nothing here opens it*, which is true. Which
   window shows a PDF is the shell's.
2. **The verb is `convert_document(file, into)`**, `Effect::Change`, with grants
   required over both arguments. It is a change because it writes a new file.
   Its sentence names both arguments. It gets a `docs/by-hand.md` entry, and the
   person's own road is opening the file from the shell, which uses the same
   service.
3. **The copy is never the original.** It is created with `O_EXCL` in the
   folder the person chose. An existing name is a refusal with a sentence, not
   an overwrite and not a silent rename. A copy half-written when the engine
   fails is removed by the verb that created it, and nothing else is.
4. **What the copy could not carry is found from the documents, never from the
   engine's log.** The original is inventoried before conversion and the copy
   after, and the difference is reported by name, as a closed set:
   - **a font substituted**: a family the original sets text in, of which the
     copy contains no font. A family declared in a font table but set on no text
     is not counted, because counting it would report losses that are not there;
   - **a field shown as its value at conversion**: a field whose value depends
     on when or where the document is open (a date, a time, a file name, an
     author, a merge field, a cell formula calling for the current moment or a
     random number) is fixed in the copy at what it showed on this machine, and
     that is said by the field's kind;
   - **macros not run and not carried**: already found by task 1 (`Macros::Inside`);
   - **linked content not fetched**: every relationship whose target is
     external, by what it is (a picture, data, a template), because fetching it
     would have been egress;
   - **comments and tracked changes not shown**: present in the original, absent
     from a PDF's page, and not a loss a person would otherwise notice.

   *Lost nothing* is `Carried::Everything`, said in its own sentence. It is only
   reachable after both inventories completed. **An inventory that cannot
   complete is a refusal to show the copy**, with its reason, and never a copy
   with an unstated cost. There is no third variant meaning *not checked*, so
   *lost nothing* and *did not check* cannot read the same.
5. **Reading inside a document needs two things this workspace has never had**:
   decompressing a zip entry and reading XML. It rents a pure-Rust inflater that
   forbids `unsafe` (`miniz_oxide`) and uses the XML reader already in
   `Cargo.lock` (`quick-xml`). Both are used **only in the service's crate**, and
   each entry is capped (64 MiB decompressed, and a ratio) so a compression bomb
   is a refusal. `alo-opening`'s own dependency list stays exactly as task 1's
   test holds it, because deciding stays separate from doing.
6. **Every conversion and every refusal is recorded** through `alo-record`,
   with the grant it ran under and what the copy could not carry.
7. **The code lives in a new crate, `crates/alo-converting`**: the verb, the
   protocol file that alone names the engine, and the inventories. The service
   binary is a thin `main`. The plan's list of crates grows by one, additively.

## What this does not decide

- **Older `.doc`, `.xls`, `.ppt` and OpenDocument files.** The same service
  could convert them, and task 1 already recognises them. Task 2's line is the
  three current formats, and each further kind is a registration and a test with
  a real file, in a later change.
- **Printing.** Task 3's CUPS is a different engine with its own unit.
- **Where a forked process lands in a threaded control group.** A does not
  need the answer, and B needs it measured before it could be chosen. It is left
  as a question, not a guess, and belongs in `docs/quirks.md` the day anybody
  measures it.

## What must happen before task 2 can be built, whichever option is chosen

1. **The owner accepts, amends or rejects this ADR.**
2. **The gate machine gets the pinned engine**, installed from the same
   upstream release and digest the image pins, in an explicit idle handoff under
   `docs/autonomy/SHARED_MAIN.md`. The conversion tests then **fail loudly**
   where it is absent, and are never `#[ignore]`d and never skipped. A test that
   skips itself when the engine is missing reports green on exactly the machines
   where nothing converts.

   **Amended 2026-09-22 by
   [ADR 0063](0063-a-machine-that-cannot-run-the-engine-says-so-rather-than-failing.md),
   in these last two sentences only.** They were written when absence of the
   engine meant a machine nobody had finished setting up, and failing loudly is
   right about one of those. The fleet now includes an aarch64 machine, where
   the pinned engine is an x86_64 build and no release of it can ever be
   installed; ten tests failed there on every run for a reason nobody on that
   machine could fix. Under ADR 0063 those ten ask whether the engine **runs** —
   by running it, never by `test -x` — and where it cannot, skip and say what
   was missing. Everything else here stands, including the part this repository
   still holds to: an engine that starts and then converts badly fails loudly,
   on the machine that has one.
3. **Three real documents** reach `crates/alo-converting/tests/documents/`:
   one each of `.docx`, `.xlsx` and `.pptx`, saved by the office applications
   people send them from, owned by the repository's owner so they can be
   published. Between them they carry a font not shipped on the image, a date
   field, a formula calling for the current moment, a linked picture and a
   comment, so each loss above is proven against a real file. Their provenance
   (which application, which version, saved when) goes in a `README.md` beside
   them.

## Consequences if accepted

- Task 2 of the plan becomes buildable, and its status moves from *blocked* to
  *ready* in the change that accepts this.
- `image/Containerfile` gains a pinned, digest-checked converter stage, a unit
  and a login. `alo-image` gains the checks, with a break-it test for each.
  `alo-saying`'s list of everything we rent gains the engine.
- `docs/contracts/agent-verbs.md` gains `convert_document`, and
  `docs/by-hand.md` its entry.
- Task 4 inherits one reason it can say plainly: *this machine has nothing that
  converts it* is true on a machine where the service does not answer.
