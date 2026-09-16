# What is watching or listening, right now

**Date:** 2026-09-15
**Workstream:** `docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 1 — the
first task of the v0.5 capture plan, and the one that comes before anything that
captures.
**Contributor:** Claude Code worker, `C:\dev\alo-os`.
**Status:** ready for integration. The code is whole and gated. One measurement
is owed on hardware and is named below rather than claimed.

## What this is

`docs/features.md` promises, starred: *a visible indicator whenever the screen,
camera or microphone is in use — by any application, including ours. Law 1 is
about egress; this is the same instinct applied to the room you are sitting in.*

This change is that promise's model: a new crate `crates/alo-in-use` holding, as
a live value, every current use of the screen, the camera and the microphone,
each naming who is using it, read from the machine's own media server — and the
line a person reads while something is in use.

It deliberately does nothing else. It cannot capture anything, it cannot decide
whether anything may, and there is no verb, grant or approval anywhere in it.

## What changed

### A new crate, `crates/alo-in-use`, twelve files

| File | What it is |
|---|---|
| `src/used.rs` | `Used` — the screen, the camera, the microphone. Three, closed, with a name, a mark and a place each |
| `src/by.rs` | `By` — who is using one: an application, an agent, alo OS itself, or something this machine can see and cannot name |
| `src/uses.rs` | `Use` and `UseId` — one current use, under the number the media server records it by |
| `src/in_use.rs` | `InUse` — every current use. `read_from` is the only constructor |
| `src/line.rs` | `Line` — a mark, a sentence, a position, and only then a colour |
| `src/mark.rs` | `Mark` — one silhouette per thing in use |
| `src/position.rs` | `Position` — where a line sits, fixed by what is in use and never by who |
| `src/streams.rs` | `Streams` — the one question a media server is asked |
| `src/media_server.rs` | `TheMediaServer` — the one thing on a machine that answers it |
| `src/heard.rs` | The rented server's own record, read into `Use` values |
| `src/refusing.rs` | `NotHeard` — why the machine could not answer, in words |
| `src/words.rs` | Ten strings: four lines, three clauses that say who, three refusals |

and three integration tests: `tests/every_sentence_here_is_collected.rs`,
`tests/two_lines_and_neither_is_the_other.rs`, and
`tests/a_stream_through_the_media_server_is_listed.rs`.

### Registration, which nothing else would have caught

- `Cargo.toml` — the new workspace member.
- `crates/alo-saying/Cargo.toml`, `src/collecting.rs` — the crate's ten strings
  are collected into the machine's one vocabulary: the dependency, `EVERY_LIST`
  (39 → 40), the `declare` call, and both counting tests. A crate that declares
  words and is not collected compiles, ships and says nothing to anybody in any
  language; `crates/alo-collected` turns that into arithmetic over the workspace
  and passes.
- `crates/alo-saying/src/rented.rs` — **PipeWire is now on
  `EVERYTHING_WE_RENT`** (16 → 17). This crate reads the rented media server's
  record, so the rule that file states — *adding a rented thing to this product
  means adding its name here in the same change* — applies. Its own check then
  holds every sentence alo OS says, in every language, to never naming it; the
  crate's integration test asks the same question of this crate's ten.

### The plan

`docs/autonomy/v0-5-capture-and-the-room-plan.md` — task 1 marked
**Done, 2026-09-15** with what was built. Tasks 2 to 7 already stand after it, so
no next task needed writing.

## A user-readable change description

alo OS now knows, at any moment, everything on the machine that is reading the
screen, the camera or the microphone, and who is doing it — and it knows it from
the part of the machine that actually carries the sound and the picture, not from
what applications say about themselves. A program that stops announcing itself
does not disappear from the list.

What a person will read, once the shell draws it, is one line per use: *The
camera is in use by Video Call (com.example.VideoCall)*, *The microphone is in use
by alo OS itself*, *The screen is in use by @alo, the agent on this machine*. Each
line carries a shape of its own and sits in its own place, so which of the three
is in use can be read without being able to tell one colour from another. Only
the agent's line is terracotta, and it says *the agent* in words as well.

There is no way to hide a line, no list of applications that are trusted not to
show one, and no setting that turns it off. When the machine cannot find out what
is watching, it says so — because an indicator that quietly showed nothing would
look exactly like a machine in a quiet room.

## Decisions taken, and why

The task left several things open. Each was chosen the way the plan's constraints
point, and each is written into the code as well as here.

**1. The uses are read from the *source* side of the media server's graph, not
the stream side.** Every capture has two sides in the record: the source the
sound or picture comes from, and the stream a client reads it through. On the
stream side, a client reading a camera and a client receiving a shared screen are
written down identically — telling them apart would mean guessing, and guessing
about *is my camera on* is exactly what this crate may not do. On the source side
the server has already distinguished them. So a use is a source the server says
is **running**: an audio source is the microphone, a video source is the camera,
and a client producing video into the graph is the screen. Who is using it is
whoever is linked to it on the reading side, one line each, with several links
between one pair (a stereo capture is two) counted once.

**2. There is a fourth answer to *who*, and it is not in the plan's three.**
`By::something_on_this_machine` — a use the machine can see and cannot name. The
plan names three (an application, an agent's turn, alo OS itself), but a running
source with nothing recorded reading it, or a client whose name is not one a verb
could ever hold, has to go somewhere. The only two candidates were a fourth
honest answer and *leave it off the list*, and leaving it off is the one failure
this indicator exists to prevent. A use nobody can name is the one a person most
needs to be told about.

**3. What the server vouches for, and what it merely repeats.** The plan's
*rather than from what applications say they are doing* has a security half, and
the rule is one sentence: **nothing an application can write about itself reaches
alo OS's answer or the agent's.** An application on alo OS is sandboxed (ADR
0005) and the machine stamps its identity onto its connection; where that stamp
is present it is the answer and it can only ever say *this application*. Without
a stamp the connection is from something not in a sandbox — one of alo OS's own
components, or a program the person started in their own terminal (ADR 0043) —
and only there is alo OS's reserved identifier read as alo OS or as an agent.
There is a test for an application that writes alo OS's identifier and the
agent's name about itself and is drawn as neither.

The residual is stated rather than papered over: a program **the person
themselves started outside a sandbox** could write alo OS's reserved identifier
and be listed as alo OS. It cannot make itself invisible, which is the property
that matters. Tightening it means comparing the connection's process against alo
OS's own, which is a measurement on a machine rather than an assumption in a
file; it belongs with the capture tasks that will set those properties.

**4. The record is asked for through the server's own tool, not a library
binding.** The tool is part of the thing we rent, it prints exactly the record
the server holds, and it keeps a C library out of every process that wants to
draw a status area. ADR 0011 says the media server is configured and never
written; this reads it and patches nothing.

**5. An unreadable record is a refusal, not a skip.** Anything in the record that
should be readable and is not fails the whole read with `NotHeard::NotUnderstood`
and a sentence telling the person the indicator cannot be trusted until it is
fixed. The alternative — passing quietly over what could not be read — leaves an
indicator that reads exactly like a quiet room. Objects that are simply not uses
(sinks, ports, clients, a node the record says has gone) are passed over, which
is a different thing and is tested as such.

**6. Position means *fixed place*, not *ordinal*.** ADR 0010 asks for mark, word
and position beside a colour. `Position::of` takes what is in use and takes
nothing else, so the camera is in the same row whether an application or an agent
has it: the glance a person learns keeps working at the moment it matters most.

**7. This crate's three clauses are not `alo-capability`'s twelve.** A grant
distinguishes *one picture of the screen* from *the screen, continuously*, which
is right for an approval and wrong here: to somebody sitting in the room they are
the same fact. Three here, twelve there, neither a fallback for the other, and
`words.rs` carries the argument for a translator who meets both.

**8. `alo-appearance` is a dependency, which the plan's read-list does not
name.** Terracotta must be *the* terracotta (ADR 0010). A second definition of it
in this crate would be a second answer to *is the machine acting on my behalf*.
`Line::colour` answers `Token::Terracotta` or `Token::Navy` and nothing else.

## Acceptance, clause by clause

| The plan says | Where it is |
|---|---|
| holds, as a live value, every current use of the screen, the camera and the microphone | `InUse`, `in_use.rs`; `Used::EVERY` is the three |
| each naming who is using it — an application, an agent's turn, or alo OS itself | `By`, `by.rs`; and a fourth honest answer, above |
| read from the media server's own record of open streams rather than from what applications say | `Streams`/`TheMediaServer`/`heard.rs`; `InUse::read_from` takes a `Streams` and there is no other constructor |
| held by a test that opens a stream through the rented server with no portal involved and finds it listed | `tests/a_stream_through_the_media_server_is_listed.rs` — **written, and skipped on every host this repository has; see *What is owed on hardware*** |
| the line is a sentence in the vocabulary naming what and who | `Line::said`, `words.rs`; *The camera is in use by Video Call (com.example.VideoCall)* |
| alo OS's own captures appear on it like anybody else's, with a test | `in_use.rs::what_alo_os_does_itself_is_on_the_list_like_anybody_elses`, `line.rs::what_alo_os_does_itself_reads_like_anybody_elses_line`, `heard.rs::alo_oss_own_capture_is_read_as_alo_os` |
| mark, word and position, never colour alone (ADR 0010) | `Mark`, `Line::said`, `Position`; `line.rs::mark_word_and_position_tell_the_lines_apart_with_no_colour_at_all` |
| not terracotta unless an agent is the one using it | `line.rs::no_line_is_terracotta_unless_the_agent_is_the_one_using_it`; and `By::the_agent` refuses an application grantee |
| no variant that hides a use | `in_use.rs::every_use_the_server_answered_with_is_on_the_list`, `heard.rs::a_running_source_nothing_is_recorded_reading_is_still_shown`, `heard.rs::a_client_that_names_itself_unusably_is_still_shown`, and the compile-fail examples on `InUse` |
| no allow-list of trusted applications | nothing in the crate takes a list of names; the compile-fail example `InUse::read_from_except` |
| no setting that turns the indicator off | the compile-fail examples `InUse::shown(false)` and `Line::hidden()` |
| the two indicators are two lines, and neither is drawn as the other | `tests/two_lines_and_neither_is_the_other.rs`, six tests |
| it shows; it never decides | no verb, no grant, no approval, no `alo-portals` dependency |

### The refusal paths, tested beside the legitimate ones

- a media server that is not on the machine — `media_server.rs`, and every run of
  the on-a-machine test on a host without one;
- a media server that ran and failed, with what it said kept for whoever is
  fixing it;
- a media server that answered something unreadable — six shapes of it;
- an application claiming alo OS's identifier and an agent's name;
- a client whose name is not one a verb could hold;
- a grantee that is an application, offered as the agent;
- a source running with nothing reading it;
- every one of the three refusals read in a person's own language, and none of
  them able to be mistaken for a quiet room.

## Verification

All commands from `C:\dev\alo-os`. The Linux half runs through WSL against the
same working tree, with this checkout's own build directory
(`docs/autonomy/SHARED_MAIN.md`); the desktop lane's target was not touched.

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | Windows | clean |
| `cargo clippy -p alo-in-use --lib -- -D warnings` | Windows | clean |
| `cargo clippy -p alo-in-use -p alo-saying -p alo-collected --all-targets -- -D warnings` | WSL Ubuntu | clean, zero warnings |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-in-use --no-deps` | WSL Ubuntu | clean |
| `cargo test -p alo-in-use` | WSL Ubuntu | 66 unit + 2 + 3 + 6 integration + 8 doc tests, all pass |
| `cargo test -p alo-saying` | WSL Ubuntu | 63 unit + 4 integration + 1 doc test, all pass |
| `cargo test -p alo-collected` | WSL Ubuntu | 8 unit + 11 integration tests, all pass |

`cargo test -p alo-in-use` cannot run on Windows and that is not this change:
the `alo-saying` dev-dependency reaches `ring`, which needs a C compiler this
host does not have. It is the same reason `alo-sessiond`'s own collected-sentence
test is a Linux run.

The full workspace suite was **not** run here, per the instruction for this task;
the supervisor runs it.

## What is owed on hardware, and is not claimed

`tests/a_stream_through_the_media_server_is_listed.rs` is the plan's acceptance
test. It is written, it is `#[cfg(target_os = "linux")]`, and on every host this
repository can reach it **skips with a sentence saying why**:

```
skipped: this machine has no media server to ask (pw-dump is not on this
machine: No such file or directory (os error 2))
```

No media server is installed in this checkout's WSL, and installing one is shared
maintenance that `docs/autonomy/SHARED_MAIN.md` says needs an idle handoff with
both workers rather than a worker's decision. So:

- **the plan's *held by a test that opens a stream through the rented server* is
  met in code and has not been met as a measurement.** The test opens a real
  recording stream, polls the machine's own media server, and fails loudly if a
  held-open stream is not listed. It has never had a media server to prove it
  against.
- **The property names in `heard.rs` are written from the rented server's
  documented interface, and the fixtures in its tests are transcribed from that
  interface rather than captured from a running server.** That is the one thing
  in this change that a machine could contradict, and it is deliberately
  confined to one file so that a correction is a correction to one file.

The honest reading, in `ROADMAP.md`'s own terms, is that the code half of this is
finished and the *on the machine* half is not taken. What to take, in order, on a
machine with a media server:

1. run this crate's integration test and see it not skip;
2. capture a real record with the server's own tool while a camera, a microphone
   and a screen share are each open, and check `heard.rs`'s three source classes
   and its property names against it;
3. confirm that a sandboxed application's identity really is stamped by the
   machine on that machine's configuration, which is what decision 3 above rests
   on.

Nothing in this change should be read as a claim that any of those three has been
done.

## Proposed updates to the shared documents

The integration owner owns `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md`
and `docs/autonomy/STATE.md`; none of them was edited here.

**`CHANGELOG.md`** — under Unreleased:

> **What is watching or listening.** alo OS now holds every current use of the
> screen, the camera and the microphone, each naming who is using it, read from
> the machine's own media server rather than from what applications say about
> themselves — and the line a person reads while something is in use: a shape, a
> sentence and a fixed place before it is a colour, terracotta only for the
> agent. Nothing can hide a use, there is no list of trusted applications, and
> there is no setting that turns it off; a machine that cannot find out says so
> rather than showing an empty indicator. `crates/alo-in-use`.

**`ROADMAP.md`** — the v0.5 line *Capture: screenshots, annotation, screen
recording with audio, screen sharing — and an indicator whenever screen, camera
or microphone is in use*. The indicator's model exists and is gated;
`- [x] The code.` may be ticked **for this task's part of that line only** if the
owner reads the line as divisible, naming `crates/alo-in-use`. **`On the
machine` may not be ticked**: see *What is owed on hardware*. If the line is read
as one thing, nothing should be ticked yet — the capture half has not been built.

**`docs/autonomy/QUEUE.md`** — no queue item names this work; it comes from the
published plan.

**`docs/autonomy/STATE.md`** — reference this report and the plan's task 1, and
carry forward the hardware measurement above as the thing outstanding.

## Limitations

- The on-a-machine measurement, above. It is the main one.
- A client producing video into the graph is read as the screen. On an alo OS
  machine that is what such a node is; a virtual camera would be read the same
  way, and that is a thing to check when the devices plan's camera work lands.
- `Use` carries no time. When a use started belongs to a record rather than to an
  indicator, and nothing in task 1 needs it. Task 4's *stop, from the indicator
  itself* will need identity, which `UseId` already carries.
- The lines are values; nothing here draws. The shell plan's later tasks draw
  them, and `Mark`, `Position` and `Token` are what they will be handed.
