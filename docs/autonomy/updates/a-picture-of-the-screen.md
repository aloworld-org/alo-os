# A picture of the screen: the screen, a window, a region — to a file or the clipboard

**Date:** 2026-09-16
**Workstream:** `docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 2 — the
first thing that captures, built on top of the indicator that shows it.
**Contributor:** Claude Code worker, `C:\dev\alo-os`.
**Status:** ready for integration. The code is whole and gated. Two measurements
are owed on hardware and are named below rather than claimed.

## What this is

`docs/features.md` promises at v0.5 *capture: screenshots, annotation, screen
recording with audio, screen sharing*. This is the screenshot: a new crate
`crates/alo-capturing` that takes the whole screen, one window or a selected
region through the rented screen-capture mechanism, and writes it to a file in a
folder the person chose or to the clipboard.

It is deliberately built **after** the indicator and **on top of** it. The plan
put task 1 first so that nothing could be built that captures without the
indicator already existing to show it, and this change honours that in the only
way that means anything: alo OS's own picture of the screen appears on that
indicator because the capture is a stream on the machine's own media server,
read back by `alo-in-use` out of the server's record — not because this crate
tells the indicator anything.

## What changed

### A new crate, `crates/alo-capturing`, twenty-two files

| File | What it is |
|---|---|
| `src/lib.rs` | What the crate is, and what it deliberately is not |
| `src/screen.rs` | `Screen` — the screen a picture is taken from, as a size |
| `src/region.rs` | `Region` — a rectangle of it, and whether it is on it |
| `src/window.rs` | `Window`, `WindowId` — one window, and the two things about it that refuse a picture |
| `src/what.rs` | `What` — the whole screen, one window, a part of it; and the rectangle each is |
| `src/session.rs` | `Session`, `WhoseSession` — whose session, and whether the lock screen is up |
| `src/folder.rs` | `Folder` — the folder a person chose to keep pictures in |
| `src/where_it_goes.rs` | `WhereItGoes` — a file, the clipboard, or both because somebody asked |
| `src/on_this_day.rs` | `OnThisDay` — the moment, as a calendar date |
| `src/naming.rs` | What a saved picture is called: the date, and nothing else |
| `src/picture.rs` | `Picture` — the bytes, and what form they are in |
| `src/grabs.rs` | `Grabs` and `NotGrabbed` — the rented mechanism, as one question |
| `src/the_screen_cast.rs` | `TheScreenCast` — the one thing on a machine that answers it |
| `src/announcing.rs` | How a capture of ours announces itself, so it is on the indicator |
| `src/taking.rs` | `Screenshot` — the act, and every refusal before it |
| `src/writing.rs` | Writing the file, without touching anything already there |
| `src/to_the_clipboard.rs` | Handing the picture to the clipboard, as its owner |
| `src/taken.rs` | `Taken` — what happened to it, and what a person is told |
| `src/asked.rs` | `Asked`, `ForAnApplication` — an application asking through the portal |
| `src/refusing.rs` | `NotTaken` — nine ways a picture does not happen |
| `src/words.rs` | Thirteen strings: three messages, ten refusals |
| `src/testing.rs` | The fixtures its own tests are written against |

and six integration tests:

- `tests/a_person_takes_a_picture_of_their_screen.rs`
- `tests/a_picture_of_the_screen_is_on_the_indicator.rs`
- `tests/what_cannot_be_captured.rs`
- `tests/an_application_asking_is_judged_first.rs`
- `tests/every_sentence_here_is_collected.rs`
- `tests/nothing_leaves_when_a_picture_is_taken.rs`

### Registration, which nothing else would have caught

- `Cargo.toml` — the new workspace member.
- `crates/alo-saying/Cargo.toml`, `src/collecting.rs` — the crate's thirteen
  strings are collected into the machine's one vocabulary: the dependency,
  `EVERY_LIST` (40 → 41), the `declare` call, `ONE_STRING_EACH` (40 → 41), and
  the arithmetic in `the_machine_says_what_the_crates_say_between_them`. A crate
  that declares words and is not collected compiles, ships and says nothing to
  anybody in any language; `crates/alo-collected` turns that into arithmetic
  over the workspace and passes.
- `crates/alo-saying/src/rented.rs` — **GStreamer is now on
  `EVERYTHING_WE_RENT`** (17 → 18), beside PipeWire. The picture comes out of a
  pinned encoder, so the rule that file states — *adding a rented thing to this
  product means adding its name here in the same change* — applies, and every
  sentence alo OS says is then held to never naming it.

### One change to `crates/alo-in-use`, which this plan also owns

`src/heard.rs`: six constants that were private are now `pub` — `A_NODE`,
`A_LINK`, `RUNNING`, `VIDEO_INTO_THE_GRAPH`, `WHAT_IT_CALLS_ITSELF` and
`THE_NODE_NAME` — with a section of the module's documentation saying why.

The reason is the one that makes the indicator true rather than decorative:
those names are **read** there and **announced** here. A capture of alo OS's own
has to appear on the indicator like anybody else's, and the only honest way for
that to be so is for the capture to open a stream on the machine's own graph
announcing itself under the same names `heard.rs` reads back. Two spellings of
`application.id`, one in each crate, would be a drift nobody notices until the
day alo OS's own screenshot is the one use the indicator does not show.

Nothing about what `heard.rs` decides moved, and it is still the only file in
that crate that knows how the server writes things down: what is public is the
vocabulary, not a second reading of it.

### The plan

`docs/autonomy/v0-5-capture-and-the-room-plan.md` — task 2 marked **Done,
2026-09-16** with what was built and what is owed. Tasks 3 to 7 already stand
after it, so no next task needed writing.

## A user-readable change description

alo OS can take a picture of the screen: all of it, one window, or a part
somebody selected. It goes into a folder the person chose, or onto the
clipboard ready to paste — and never both unless they asked for both, so a
picture taken to paste into a message does not quietly leave a copy in a folder
they were not thinking about.

The file is called after the moment it was taken — `2026-09-16-120000.png` — and
that is the whole of its name. It has no word in it in any language, and nothing
in it about what was on the screen: a folder of pictures is something people
scroll past, show over their shoulder and hand to a repair shop, and a window's
title in a file name is a sentence about somebody's appointment readable long
after the picture was deleted. A picture is never written over something already
in the folder, and it is readable by the person and by nobody else.

While the picture is being taken, **the indicator says the screen is in use by
alo OS itself** — the same line a person would read about a video call using
their camera, from the same place, because alo OS's own capture is a stream on
the machine like anybody else's.

Two things cannot be photographed at all: the lock screen, and a window
belonging to somebody else signed in on the same machine. Being refused the
second does not tell anybody who that person is. And an application asking for a
picture through the portal is judged against what the person granted it, in
`alo-portals`, before anything is taken — a camera grant is not a grant to
photograph the screen, and a grant that has ended is not a grant.

Nothing is uploaded, shared or sent anywhere by taking a picture. There is no
cloud folder, no *share* step, and nothing in this crate or anything it depends
on that could open a connection.

## Decisions taken, and why

The task left several things open. Each was chosen the way the plan's
constraints point, and each is written into the code as well as here.

**1. The picture comes through the media server, and that is what makes the
indicator true.** A compositor has the pixels and could have handed them over
directly; that road would have been faster and completely invisible to
`alo-in-use`, which reads the media server's record and nothing else. So the
frame is read off the screen cast's stream on the machine's own graph, and
`announcing.rs` sets the two properties that make the reader alo OS's own. The
acceptance *appears on task 1's indicator* is then a property of how the capture
works rather than a message this crate sends, and the test goes through
`InUse::read_from` — the only door that crate has.

**2. The rented mechanism is the screen cast plus a pinned encoder, reached
through the encoder's own tool.** ADR 0011 and the plan both forbid writing a
screen grabber or an encoder here. `TheScreenCast` runs `gst-launch-1.0` with a
cleared environment and no shell — the shape `alo_in_use::TheMediaServer` uses —
reading one frame from the screen-cast stream, cropping to the rectangle this
crate decided, and writing a lossless image on its output. PNG rather than
anything else: it is lossless, unencumbered, and every machine a person owns
opens it. **Which encoder and format a *recording* uses is still task 4's
decision** and is not pre-empted here.

**3. The environment is cleared down to four things, and the fourth is a
difference from task 1.** `XDG_RUNTIME_DIR` is passed through where the session
set it, because a client that cannot find the media server's socket is a
screenshot that never happens. It is the session's to say and this crate does
not invent one. Task 1's `TheMediaServer` clears it, and whether that is right
is one of the things the hardware run below will answer for both.

**4. *Which pixels* is decided here and never inside a rented tool.**
`What::across` turns each of the three into one rectangle, and `Region` answers
what to cut from each edge. The rented tool is told four numbers. That keeps the
whole of the geometry in values a test can read with no machine in the room,
which is the half a rented tool cannot be asked to get right on our behalf.

**5. The refusals are refusals of the constructor, not checks inside `take`.**
`Screenshot::of` answers the lock screen, another person's window and a
rectangle off the screen; `Screenshot::take` is the only thing that can reach
`Grabs`. A picture that may not be taken is therefore not a value that exists
and then declines to be taken — it is a value that cannot be built, and there is
nothing to call `take` on. The same shape carries the application's grant:
`ForAnApplication` is the only value with a `take` on it for an application's
picture and `Asked::judged` is its only constructor, so *before anything is
taken* is a fact about the types. The tests still measure it with a mechanism
that counts how often it was asked, and find it at zero.

**6. The lock screen is refused for every kind of capture, and twice.** Once as
the session's state (`Session::the_lock_screen_is_up`) and once as the window
itself (`Window::the_lock_screen`). They are different facts: a compositor can
hold the lock screen's surface while the session behind it is already unlocked,
during the moment it is coming down, and a capture asked for in that instant
must not get it.

**7. The file's name has no word in it at all — not even *Screenshot*.** A name
with an English word in it is one language's word on every machine in the world,
which `CLAUDE.md` calls a bug; a translated file name is worse, because the same
folder would then hold files named in whatever language the machine was set to
that month. A date is read everywhere, sorts correctly everywhere, and says
nothing. Several pictures in one second are told apart by a number after the
time — `-2`, `-3` — bounded at a hundred, after which the machine says so rather
than looping while it looks busy.

**8. The date is made from a moment and an offset, both handed in.** Nothing
here reads a clock and nothing reads a timezone setting, which alo OS does not
yet have a crate for. `OnThisDay::at` takes a `SystemTime` and minutes east of
universal time — minutes, because several places people live are not a whole
number of hours from it — and there is **no other constructor**: a date somebody
could write by hand is a file name somebody could write by hand, and the promise
is that the name says nothing about what was on the screen.

**9. `Folder` is this crate's own type, and it makes a smaller claim than it
could.** `alo_picking::Picked` is sealed and would have been the perfect fit,
but it cannot be obtained outside `alo-picking` — `Picker::pick` answers a
`Chosen`, which has no accessor for the `Picked` inside it — and reaching into
another workstream's crate to add one was not this task's to do. `Folder` is
what is written down instead, and its documentation says exactly what it
guarantees: **there is no folder in this crate that anybody did not choose** — no
default, nothing read from an environment, nowhere a path is assembled — and it
refuses the two paths that cannot be a deliberate choice, a relative path and
the root of a filesystem. That is a smaller claim than *this was picked in a
picker*, and it is deliberately the one stated.

**10. *Both* is a destination somebody asks for.** The plan says *never both
unasked*, which is not the same as *never both*. `WhereItGoes` has three answers
and the third is reachable only through `a_file_and_the_clipboard`; there is no
`Default`, no `From`, and nothing that widens one destination into two after the
fact, which the compile-fail examples on the type hold. `Screenshot::take` is
handed the machine's one clipboard whatever the destination, so that *the
clipboard was not touched* is something a test can watch rather than something a
reader has to take on trust because the code never had the opportunity.

**11. Two refusals are carried whole rather than reworded.** The rented
mechanism's (`NotGrabbed`) and `alo-portals`' (`Refused`, which is
`alo-capability`'s under it). An application refused a picture of the screen
reads the same words as an application refused a camera, because the thing that
refused it is the same thing — and a second wording here would be this crate
having an opinion about grants that it is not allowed to have.

**12. There is a refusal for a clipboard hand-over that cannot fail today.**
`NotTaken::NotCopied` covers `alo_clipboard::Offer::copied` refusing the
one-form offer this crate makes. It cannot: `the_offer_this_file_makes_is_one`
says so. It is a refusal rather than an unwrap for the reason every list in this
workspace is — a library that panicked over its own table would take the shell
down with it — and its sentence is tested beside the others.

**13. There is no verb, and no agent can reach any of this.** A person
photographing their own screen is not an agent or an application doing
something, which is the line `alo-picking` and `alo-granted` already draw. The
agent's picture of the screen is the plan's task 6, through a verb whose
proposal says *a picture of your screen* and which is approved like any other
change; nothing here anticipates it.

## Acceptance, clause by clause

| The plan says | Where it is |
|---|---|
| takes the whole screen, one window or a selected region | `What`, `what.rs`; `tests/a_person_takes_a_picture_of_their_screen.rs::the_whole_screen_one_window_and_a_region_all_reach_the_mechanism` |
| through the rented screen-capture mechanism | `Grabs` is the only road to pixels and `TheScreenCast` is the only implementation; `the_screen_cast.rs`, and nothing in this crate reads a display |
| writes it to a file in a folder the person chose | `Folder`, `writing.rs`; `a_picture_to_a_file_is_written_where_she_chose_and_the_clipboard_is_untouched` |
| or to the clipboard | `to_the_clipboard.rs`; `a_picture_to_the_clipboard_is_ready_to_paste_and_no_file_was_written` |
| never both unasked | `WhereItGoes`, three answers with *both* reachable only by asking; `both_happens_only_when_somebody_asked_for_both`, the two compile-fail examples, and the clipboard handed in and coming back untouched |
| the file name carries the date | `naming.rs`; `the_files_name_is_the_moment_and_nothing_else` |
| and nothing about what was on screen | `name_for` takes a moment and a number and has no other argument; `what_was_captured_makes_no_difference_to_the_name` |
| a screenshot is a use of the screen and appears on task 1's indicator for its moment, held by a test | `announcing.rs`; `tests/a_picture_of_the_screen_is_on_the_indicator.rs`, five tests, through `InUse::read_from` |
| a window from another person's session cannot be captured | `Screenshot::of`; `tests/what_cannot_be_captured.rs::another_persons_window_cannot_be_captured` |
| or the lock screen | `Session` and `Window::the_lock_screen`; `nothing_can_be_captured_while_the_lock_screen_is_up`, `the_lock_screens_own_window_cannot_be_captured_even_when_the_session_is_not_locked` |
| an application asking through the portal is judged by `alo-portals` against its grant before anything is taken | `asked.rs`; `tests/an_application_asking_is_judged_first.rs`, six tests |
| nothing is uploaded, shared or sent anywhere | `tests/nothing_leaves_when_a_picture_is_taken.rs`, three tests over the manifests and the source |
| no automatic cloud folder, no *share* step on by default | `WhereItGoes` has two destinations and both are on this machine |

### The refusal paths, tested beside the legitimate ones

- the lock screen, for all three kinds of capture, and the lock screen's own
  window in a session that is not locked;
- a window in another person's session — and her own window in her own session
  captured, because a refusal is only worth testing beside the thing it allows;
- a rectangle with no width, no height, or hanging over an edge of the screen;
- a screen with no width or no height;
- a folder that is not a full path, and the root of a filesystem;
- a folder that is not there — refused with what the disk said, and **not made**;
- a folder already holding every name the moment can make;
- a file already in the folder under the name a picture would take, left
  untouched;
- a rented mechanism that is not on the machine, one that ran and failed, and
  one that succeeded and printed nothing — and, for each, nothing written and
  nothing copied;
- an application nobody granted anything; one granted the camera; one granted
  *the screen, continuously*; one whose grant has ended; one whose grant was
  revoked — and the mechanism at zero in every one;
- an identifier that is not one;
- a clipboard asked for a form that was not offered;
- every refusal read in a person's own language, and no two reading the same.

## Verification

All commands from `C:\dev\alo-os`. The Linux half runs through WSL against the
same working tree, with this checkout's own build directory
(`$HOME/alo-builds/alo-os-88e6ebddb0cab76e`, per
`docs/autonomy/SHARED_MAIN.md`); the desktop lane's target was not touched.

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | WSL Ubuntu | clean |
| `cargo clippy -p alo-capturing -p alo-in-use -p alo-saying -p alo-collected --all-targets -- -D warnings` | WSL Ubuntu | clean, zero warnings |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-capturing -p alo-in-use --no-deps` | WSL Ubuntu | clean |
| `cargo test -p alo-capturing` | WSL Ubuntu | 111 unit + 30 integration + 3 doc tests, all pass |
| `cargo test -p alo-in-use` | WSL Ubuntu | 66 unit + 11 integration + 8 doc tests, all pass |
| `cargo test -p alo-saying` | WSL Ubuntu | 63 unit + 4 integration + 1 doc test, all pass |
| `cargo test -p alo-collected` | WSL Ubuntu | 8 unit + 11 integration tests, all pass |
| `cargo test -p alo-by-hand` | WSL Ubuntu | 27 unit + 13 integration tests, all pass — the other check that walks the workspace's members |

`cargo build -p alo-capturing` cannot run on Windows and that is not this
change: the tree reaches `ring`, which needs a C compiler this host does not
have. It is the same reason task 1's gates were a Linux run.

The full workspace suite was **not** run here, per the instruction for this
task; the supervisor runs it.

## What is owed on hardware, and is not claimed

Two things, and the first is inherited.

**1. Task 1's measurement, unchanged.** `alo-in-use`'s property names are
written from the rented media server's documented interface and have never been
checked against a running one. This change leans on exactly those names in
`announcing.rs`, so it inherits that debt whole rather than adding to it: if the
record's spelling is wrong there, it is wrong here, and the correction is still
a correction to one file.

**2. The pipeline in `the_screen_cast.rs` has never run.** No machine this
repository can reach has the rented screen cast or the pinned encoder installed,
and installing them is shared maintenance that `docs/autonomy/SHARED_MAIN.md`
says needs an idle handoff with both workers rather than a worker's decision.
So:

- **the geometry is measured and the plumbing is not.** What the tool is told —
  which stream, one frame, how much to cut from each edge — is built by
  `asking_for`, which is a pure function with four tests over it. Whether the
  tool takes those arguments in that spelling, and whether `XDG_RUNTIME_DIR`
  plus the announced properties are enough for it to reach the media server, is
  the part a machine has to answer.
- **the announcement is measured against `alo-in-use` and not against a
  server.** The indicator test proves that a node carrying those properties is
  read as *the screen, by alo OS itself*; it does not prove that a client
  started this way carries them.

What to take, in order, on a machine with the media server and the encoder:

1. open a screen cast, note the stream's number, and run this crate's pipeline
   by hand — `gst-launch-1.0 -q "pipewiresrc path=N num-buffers=1" ! videoconvert
   ! "videocrop …" ! pngenc ! "fdsink fd=1"` — and check that one image comes
   out and that the crop is the rectangle asked for;
2. while it runs, capture the media server's record and confirm that the
   reading node carries `application.id = os.alo.AloOs` and that
   `alo-in-use` lists it as *the screen, in use by alo OS itself*;
3. confirm whether the cleared environment is enough, and in particular whether
   `XDG_RUNTIME_DIR` alone gets a client to the server — the answer applies to
   `alo_in_use::TheMediaServer` as well, which clears it.

Nothing in this change should be read as a claim that any of those three has
been done.

## Limitations

- The two measurements above.
- **One screen.** `Region` is a rectangle on one `Screen`, with that screen's
  own origin at its top left. A capture spanning two displays is not something
  this task promises, and modelling it with a signed corner before `alo-displays`
  and `alo-dividing` exist would be modelling a guess. When they arrive, a
  `Screen` is what one of them hands over and nothing here changes.
- **A window is a rectangle.** The screen cast can target a window node
  directly, and on a machine where it does, a window's picture would not include
  whatever overlaps it. What this crate does today is crop the screen to the
  window's rectangle, which includes anything drawn over it. That is a fact
  about the shell's compositing rather than about these values, and it belongs
  with the shell plan's task that opens the cast.
- **Nothing here draws.** The selection rectangle a person drags, the shutter
  and the notification are the shell plan's later tasks; what is settled here is
  every value and every refusal that lane would otherwise decide while wiring
  it.
- **No annotation.** Marks, blur and the rest are the plan's task 3, which
  depends on this one and is now unblocked.

## Proposed updates to the shared documents

The integration owner owns `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md`
and `docs/autonomy/STATE.md`; none of them was edited here.

**`CHANGELOG.md`** — under Unreleased:

> **A picture of the screen.** alo OS can take one of the whole screen, one
> window, or a part somebody selected, and put it in a folder they chose or on
> the clipboard — never both unless they asked. The file is named after the
> moment it was taken and after nothing else: no word in any language, and
> nothing about what was on screen. Nothing already in the folder is written
> over, and the picture is the person's own to read. While it is being taken,
> the indicator says the screen is in use by alo OS itself, from the machine's
> own media server and like anybody else's use. The lock screen and a window
> belonging to somebody else signed in on the machine cannot be photographed at
> all, and an application asking through the portal is judged against what the
> person granted it before anything is taken. Nothing is uploaded, shared or
> sent anywhere. `crates/alo-capturing`.

**`ROADMAP.md`** — the v0.5 line *Capture: screenshots, annotation, screen
recording with audio, screen sharing — and an indicator whenever screen, camera
or microphone is in use*. Two of that line's four things now have code
(`crates/alo-in-use`, `crates/alo-capturing`); annotation, recording and sharing
do not. **`On the machine` may not be ticked**: see *What is owed on hardware*.
Whether `- [x] The code.` may be ticked for part of a line is the owner's
reading of it, as task 1's report already said.

**`docs/autonomy/QUEUE.md`** — no queue item names this work; it comes from the
published plan.

**`docs/autonomy/STATE.md`** — reference this report and the plan's task 2, and
carry forward the two hardware measurements above as what is outstanding for
this workstream.
