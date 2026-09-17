# One crate reaches the media server, and it knows four facts apart

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media, which now owns `crates/alo-media-server`
**Task:** none: the owner's instruction, from the finding this lane reported
twice — *three crates with their own copy of the same twenty lines and the same
stream-reading rule, fixed three times in two days, is the same fault as the
hand-maintained lists that broke lanes yesterday: one truth written down in
several places.*
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**, against the live PipeWire 1.0.5 session.
**Egress:** none.
**Status:** done. Four crates read it; none of them keeps a copy of any of it.

## What was wrong

**Four** crates reached the machine's media server, each its own way:

| Crate | What it was doing | Plan |
|---|---|---|
| `alo-in-use` | reading the graph to say what is watching or listening | capture |
| `alo-capturing` | opening a stream to take a picture of the screen | capture |
| `alo-sound` | reading and setting the devices a person hears through | devices |
| `alo-cameras` | listing what can see | devices |

Each started the server's own tool, cleared an environment, and (three of them)
parsed a record. **The copies drifted**, and both times the drift cost a day:

- one of them did not pass `XDG_RUNTIME_DIR`, so it told a machine with a working
  server that its server would not answer — and that was found by a gate refusing
  an unrelated publish;
- one of them turned a record that arrived as two lists into *this machine
  answered something unreadable*, and the fix had to be written three times, once
  per copy.

## What it is now

`crates/alo-media-server`, owned by the devices and media plan and **read by two
crates that plan does not own**. That is stated in the plan's crate list and in
the lane table, so nobody adopts it later as unowned.

| | |
|---|---|
| `reaching.rs` | starting one of the server's tools: no shell, a cleared environment, the machine's own programs, the C locale, and **where the server is listening** — handed in as an argument, as `alo-choosing` takes the variables it reads |
| `asking.rs` | asking for the record, and turning what happens into one of four facts |
| `record.rs` | what it said, read as a **stream** of lists, an object listed twice taken as it was listed last |
| `refusing.rs` | the four facts |

## The four facts, which are why this is a crate rather than a function

The owner named this as the crate's whole job, and it is:

| What is true of the machine | What it used to say | What it says now |
|---|---|---|
| there is no such tool | nothing here handles sound and video | `NothingHandlesIt` |
| the tool is there, **no session is running** | *the server would not answer* | `NoServerIsRunning` |
| the server is there and failed | the server would not answer | `ItWouldNotAnswer` |
| the server answered something unreadable | it answered something unreadable | `ItAnsweredSomethingUnreadable` |

**An ordinary build host is not a broken machine.** The second row is a machine
with the package installed and nobody signed in; reading it as a broken server
sent somebody looking for a service that was never started, and made an
on-a-machine test fail where it should have skipped.

**A server that answered nonsense must not take the indicator off a screen.** The
fourth row is its own fact for the opposite reason: `NotAsked::is_a_machine_without_one`
answers true for the first two and **false** for this one, because an indicator
showing nothing because a record would not parse looks exactly like an indicator
on a machine where no camera is on — and that is the wrong answer given
confidently about the one thing an indicator exists for.

Each of the four reads differently in a log, and a test holds that they do.

## What the crate deliberately does not do

- **It says nothing to a person.** No vocabulary, and there will not be one:
  *what is watching*, *which speaker is this* and *what can see* are different
  sentences belonging to the crates that know which is being asked. Each reader
  maps the four facts onto its own words — and each maps the first two onto one
  sentence, because to a person they are one thing and the difference belongs in
  the diagnosis beside it.
- **It decides nothing about what an object means.** Two crates reading the same
  node mean different things by it. The record is handed over; the meaning is
  theirs.

## What moved, crate by crate

- `alo-in-use`: `media_server.rs` is a page shorter and its own `heard::in_use_in`
  now takes objects rather than text. The text-level cases of its *a record that
  cannot be read is refused* test moved to `alo-media-server`, where they belong;
  what stayed is the half this crate judges — an object in a good record that does
  not say what an indicator needs.
- `alo-sound`, `alo-cameras`: the same, each keeping its own refusals and its own
  sentences.
- `alo-capturing`: uses the tool-starting half, and keeps the one thing that is
  its own — the name the server shows a person for whoever is asking.

## And the rule this came with

The owner adopted it fleet-wide and it is now written in `nofail.sh` itself, where
the next person meets it:

> The run that decides which failures are known must execute in the same
> environment as the run that produces them. A judge that cannot see what the gate
> saw refuses honest work and accepts nothing in its place.

It belongs in `docs/autonomy/LOOP.md` as well as in the scripts. I have not put it
there: the loop's documents have one writer, and that is not this lane.
