# What this machine plays, and where the right to decode came from

**Date:** 2026-09-19
**Workstream:** v0.5 — devices and media, task 1
**Task:** 1, *Which codecs this machine carries, decided before anything plays* —
everything around the one question that needs a lawyer.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 4 GB of memory**. **Nothing here is ticked on the machine.** No file was
played. This crate decides and plays nothing, which is stated in its own first
paragraph so that nobody reads the tests as playback.
**Egress:** none. Nothing was fetched; the codec facts come from ADR 0051, which
was written on 2026-09-17 with its own sources.

## What was left, and what this closes

ADR 0051 was accepted in both halves on 2026-09-17, with **one question open and
marked for counsel**: *for a machine sold and distributed in the EU, which of
AAC-LC, baseline H.264, main-profile H.264 and HEVC may alo OS include as a
software decoder in the image it ships, and under what notice?*

The instruction was to leave that and close everything around it. Everything
around it turns out to be most of the task: the decision settles what this
machine produces, the order in which a right to decode exists, what a refusal
looks like, and the split between a container and the codecs inside it. None of
those need the lawyer.

## The crate

`crates/alo-playing`, owned by this plan and already named in the lane table.

| | |
|---|---|
| `codec.rs` | the codecs the decision names, closed, and which carry no royalty |
| `producing.rs` | what this machine encodes — free on every machine, for every purpose |
| `right.rs` | **where a right to decode comes from**, and the question counsel has not answered |
| `machine.rs` | what a machine has, handed in rather than read here |
| `inside.rs` | the tracks in a file, because **a kind is the wrapping, not the codec** |
| `deciding.rs` | the order, walked once per track |
| `refusing.rs` | the one sentence, which is `alo-opening`'s |

## The three things worth arguing about

**1. The order is a type, not a boolean.** *Can this machine decode H.264?* has
four answers, not two, and three of them are yes for different reasons:

- **royalty-free** — nothing is licensed, so there is nothing to have a right to;
- **from the silicon** — the licence was paid for with the chip, and the file
  never touches a decoder we distributed;
- **from a licensed decoder** — Cisco's `openh264` and its like, whose publisher
  paid the royalty and permits redistribution;
- **none** — and this machine says so.

They are separate because only one of them **could be withdrawn**: a publisher
can stop paying, and a chip cannot be taken out of a machine somebody owns. That
difference is recorded as `Right::could_be_withdrawn`, which nothing in the crate
reads today — it is there because a type that records a difference nobody can
read has not recorded it.

**2. Every track, not any track.** A film whose picture decodes and whose sound
does not is the failure that looks like success: somebody watches it in silence
and reports that alo OS broke their video. `Plays::all_of_it` is an *and* across
every track, and a test holds the specific case — hardware that decodes H.264,
no right to the AAC beside it, refused rather than played silently.

**3. The refusal is not ours.** ADR 0051 named the road before this crate
existed: `alo_opening::Cannot::NothingHereOpens(Kind)`, the same sentence a
person meets when a document cannot be opened, with the same *and here is what
would*. `refusing.rs` is four lines of code and a page of tests, and one of those
tests asserts that a film and a Word document produce the **same remedy** — a
test that only checked the variant would still pass the day somebody gave media a
remedy of its own.

There is no vocabulary in this crate and there will not be one.

## How the open question is held open

The wrong way to be ready for a lawyer's answer is to assume it. The right way
had to survive somebody tidying, so:

- `right::SoftwareDecoders` is an enum with **one value**,
  `NotAnsweredByCounsel`. A comment saying *counsel has not answered* is a
  comment somebody deletes; a type every test asserts against is a hole with a
  shape.
- The crate decides with **three** steps, not four. There is no *ship a software
  decoder* branch to enable — it does not exist yet, so nobody can reach it by
  accident. This is the decision's own **none may ship** row, taken as the safe
  reading rather than as the answer.
- **AAC-LC is not marked royalty-free**, and neither is baseline H.264. MP3 is,
  because its last patents expired in 2017 — a fact with a date rather than a
  reading of filings. Marking AAC free would be assuming the answer in the
  permissive direction, which is the more expensive of the two mistakes.
- `tests/what_this_machine_plays_is_what_the_decision_says.rs` **reads ADR 0051**
  and asserts the heading *What is open, and for counsel* is still there. **The
  day somebody answers, that test fails**, and the crate has to change with it.
  An answer that arrived and changed no code is an answer nobody acted on.

The same test file also holds the deadline — *before the certified laptop goes to
anybody outside this team* — because that is the sentence most likely to be
softened into *before v1* by somebody tidying, and *before v1* is not a date
anybody can act on.

## What the tests actually claim

32, all pure logic, all on a build host:

- **Everything this machine produces plays on a machine that has nothing** —
  exhaustively, across both hardware answers and both destinations. That is the
  encoding half paying off: a recording made on one alo OS machine opens on every
  other one, with no hardware decoder and no licence in the room.
- **Nothing decodes without silicon or a licence** — every encumbered codec, on a
  bare machine, refuses. There is no path through this crate that reaches *yes*
  any other way.
- **A free codec is free whatever the machine has** — including on a machine that
  has every hardware decoder and every licence. AV1 answering *from the silicon*
  would say a right came from a chip, and a machine without that chip would then
  look like it had lost a right it never needed.
- **Every codec on every machine lands on one of the four answers** — three
  machine shapes across all eleven codecs.
- **The common case refuses**, and that is the decision working: a plain laptop,
  an H.264 film from somebody's phone, two tracks neither of which this machine
  may decode.

## What is not claimed

- **Nothing plays.** No bytes move. The acceptance *a test per format that plays
  a real sample file through the rented stack* is the whole of what is left in
  task 1, and it needs both the counsel answer and a machine.
- **No file is read.** What is inside a container arrives as `Inside`, from
  whoever parsed it. This crate knows nothing about container formats.
- **No codec was added to the image**, which the task's own constraint forbids
  before the decision is accepted — and the open half is not.

## One finding

**`alo_opening::Kind::is_played` names nine kinds, and `alo-playing` now names
eleven codecs; nothing holds those two lists to each other.** It cannot, honestly
— a Matroska file can hold any of them, which is the whole *wrapping is not the
codec* point — but it means a tenth media kind could be added to `alo-opening`
with no codec behind it and nothing would notice. `alo-opening` belongs to the
documents-and-paper plan, which has finished and has no lane, so this is a note
rather than an edit: if anybody adds a media kind, the question to ask is which
codecs people actually put in it.
