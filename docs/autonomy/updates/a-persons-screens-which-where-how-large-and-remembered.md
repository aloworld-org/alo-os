# A person's screens: which, where, how large, and remembered

**Date:** 2026-09-18
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 3
(*A person's screens: which, where, how large, and remembered*). Serves
`ROADMAP.md` v0.5 *Multi-monitor, scaling, hotplug* and the per-display half of
*Making it yours*.
**Contributor:** Claude Code worker in `C:\dev\alo-os`.
**Status:** ready for integration. Gated on the crates touched; the supervisor
runs the whole workspace.

## What changed, in words a person outside this repository can read

alo OS now remembers a person's screens. Which screen is which is read from what
the screen says about itself — its make, its model and its serial number where it
has one — rather than from which socket the cable happens to be in, so the
external monitor is the same monitor tomorrow morning and in a different port.
Each set of screens gets its own arrangement: docking at the office restores the
office layout, docking at home restores home's, and the laptop on its own is a
third arrangement of its own.

A screen the machine has never seen is not set to 100%. Its size is worked out
from its own pixels and the size of its glass, so a 27-inch 4K panel arrives at
175% rather than covered in text four millimetres high; where the machine can
only draw whole multiples, the nearest one is used and the person is told which
two sizes are involved. Unplugging a screen moves what was open on it to the main
screen rather than losing it, and plugging it back brings it home — including
when the screen it was moved onto is unplugged in its turn.

## What changed, with paths

A new crate, `crates/alo-displays`, owned by this plan. Fourteen modules, one
responsibility each.

| File | What it holds |
|---|---|
| `src/identity.rs` | `Identity`, `Panel`, `Socket`, `Stability`, `whoever_is_attached` |
| `src/reported.rs` | `Reported`, `Resolution`, `Millimetres`, `NotAScreen`, `which_screens_these_are` |
| `src/scale.rs` | `Scale`, `ScaleError`, `Support`, `Rounded`, and the derivation for a screen nobody has sized |
| `src/placed.rs` | `Position`, `Placed`, and whether two screens would be drawn over each other |
| `src/arrangement.rs` | `Arrangement`, `Screens`, `NotArranged`, and the shape of the file's rows |
| `src/changes.rs` | `Changes` — one arrangement per set of screens, newest last |
| `src/attached.rs` | `Attached`, `OnScreen` — the screens plugged in now, laid out |
| `src/coming_and_going.rs` | `Moved`, `CameBack`, `NotAttached` |
| `src/wearing.rs` | `Wearing` — the background and the dock edge each screen wears |
| `src/notes.rs` | `Note` — what a person is told about how their screens were set up |
| `src/keeping.rs` | `displays.toml` in the person's own folder (ADR 0038) |
| `src/unkept.rs` | `FileNotRead`, `FileNotWritten` — what that file's refusals say |
| `src/unreadable.rs` | `NotRead` — the key a deserialiser writes where there is nobody to ask for words |
| `src/words.rs` | Every sentence this crate can say, 27 of them, each with a translator's note |

Three integration tests: `tests/three_sets_of_screens_remembered_apart.rs`,
`tests/a_screen_that_goes_and_comes_back.rs`,
`tests/every_sentence_here_is_collected.rs`.

Two registrations outside the crate, both required for it to exist at all:

- `Cargo.toml` — `crates/alo-displays` added to the workspace members.
- `crates/alo-saying/Cargo.toml` and `crates/alo-saying/src/collecting.rs` — the
  crate's words collected into the machine's one vocabulary. `alo-collected`
  reads the workspace and would have named this crate as one nothing collects.

`Cargo.lock` records the new member.

## The decisions this task left open, and what was chosen

The plan named the acceptance and left the shapes to the worker. Each of these
was decided the way a senior engineer would and is argued in the file that holds
it; they are collected here because a reader of the plan will want them in one
place.

**1. A screen that says nothing is remembered by its socket — and so are two
screens that say the same thing.** The plan required the first. The second is the
case it does not mention and that a desk with two identical monitors meets on day
one: two screens reporting the same make and model and no serial cannot be told
apart by what they say, so `whoever_is_attached` hands **both** of them their
socket. It is decided over the whole attached set rather than one screen at a
time, because whether a description is unique is not a fact about one screen. The
person is told (`displays.told-apart-by-their-sockets`), because swapping the two
cables swaps the two screens and that is a surprise worth pre-empting.

**2. A size follows the screen; a place follows the set.** An arrangement is
remembered under the exact set of screens, which is what the plan asks for — but
plugging a fourth screen in makes a set nobody has arranged while three of the
four are screens somebody has already sized. Resolving the whole thing from
scratch would throw away three deliberate choices. So `Attached` takes the
*position* and the main screen from the arrangement for exactly this set, and
each screen's *size* from the most recent arrangement that screen appears in,
falling back to the derivation only for a screen the machine has genuinely never
seen. How large a screen should draw is a fact about that screen; where it sits
is a fact about the set it is in.

**3. What was on a screen that is unplugged goes to the main screen.** Not the
nearest, which changes when somebody moves a monitor on their desk; not the
first, which is whichever socket the machine happened to enumerate first. The
main screen is the one thing about an arrangement the person has already decided,
and it is where a window with nowhere else to go belongs everywhere else in this
system. Unplugging the **last** screen is refused rather than obeyed, because
what is open on it would have nowhere to go.

**4. The crate holds no window identifier, and the decision is carried by
display.** The plan says the moving is the shell's. Rather than tracking which
windows moved, `Attached` records *which screen's windows are sitting on which
other screen* — so `plugged_in` answers *what goes back to this screen* without
this crate ever knowing a window exists. The chain is kept: unplug the screen the
windows were moved onto and every displaced screen's entry is repointed, so each
still finds its way home.

**5. A remembered arrangement that no longer fits is set aside rather than
forced on.** A screen whose resolution changed — a projector renegotiating, a
monitor switched to another input and back — can turn a saved arrangement into
two screens drawn over each other, which is a machine with a window nobody can
reach. The saved places are checked against the pixels reported *now*; one that
would overlap, or that has no place for a screen that is plugged in, is set aside
for one worked out, with `displays.did-not-fit` said. **What the person chose is
not deleted**: the next time those screens report themselves as they did, it fits
again.

**6. A scale is a whole per cent between 100 and 300.** 125%, 150% and 175% are
the sizes a modern panel wants and each of them is fractional in the sense the
plan means: not a whole multiple of the screen's own pixels. `Support` is asked
of the compositor once and handed in; where it says whole multiples only, the
nearest is used and it is never the smaller one — text that is too large can be
read and text that is too small cannot.

**7. The derivation is arithmetic, not a table of models.** Pixels per inch
across the screen, against 96 of them, rounded to the nearest quarter, held
between 100% and 300%. A 27-inch 4K panel comes out at 175%, a 24-inch 1080p one
at 100%, a 13.3-inch 1080p laptop panel at 175%. A screen that reports no
physical size at all is the one case that is 100%: there is nothing to divide by,
and inventing a number would be worse than saying so.

**8. `Position` is signed and measured from the main screen's corner.** A screen
to the left of the main one is at a negative first number, which is exactly the
case the laptop that forgets gets wrong.

## A finding: `alo-dock` holds one edge for the machine, not one per screen

The acceptance says *a per-display background and a per-display dock edge are read
from `alo-appearance` and `alo-dock` and applied to the display they name*. Half
of that exists today and half does not:

- **The background does.** `alo_appearance::Appearance::background_on` takes the
  name the shell knows a screen by and answers with the exception the person made
  for that screen, or their choice for everywhere, or the wallpaper the image
  shipped. `Wearing::of` asks it under
  `Reported::named_for_the_shell` — the socket and the screen's own description
  together, which is exactly what `alo_appearance::DisplayId`'s own documentation
  describes (*a connector and the monitor's own description*). Two identical
  screens are therefore two names, because their sockets differ.
- **The dock's edge does not.** `alo_dock::Dock` holds **one** `Edge` for the
  machine. There is no per-screen edge in that crate and no file key for one.

So `docs/features.md`'s v0.5 line *Per display, so the dock can sit along the
bottom of the laptop and down the side of the external screen* is **not met by
this task**, and this report says so rather than letting a per-display background
stand in for both halves. The promise is not narrowed — it is named, with the one
crate that has to change and the one function here that changes with it.

The plan says *`alo-dock` (per-display placement is its decision)* and that this
plan **reads and never edits** it. So what `Wearing` does today is what
`alo-dock` has decided today: the edge the person chose, applied to every screen.
This is written down here rather than quietly narrowed. `Wearing::of` is the one
function that changes when `alo-dock` gains an edge per screen, and every caller
of it keeps working; the change belongs to whoever owns `alo-dock` next, in
`crates/alo-dock/src/changes.rs` beside the edge it already keeps.

## Known limits, stated rather than hidden

- **A screen remembered by its socket is weaker, and cannot be made stronger.**
  Another screen plugged into that socket is set up as though it were this one.
  The person is told once, in `displays.remembered-by-its-socket`. There is no
  third option: refusing to remember such a screen would mean the laptop's own
  panel — most machines' only screen — was the one thing never remembered.
- **Identities can change while a screen is away.** Plugging in a twin of a
  screen that some windows were moved onto turns that screen's identity from a
  description into a socket. `CameBack::were_on` answers `None` in that case
  rather than naming a screen that is no longer attached; the windows still go
  back to the screen that returned, which is the part that matters.
- **Nothing here has run on certified hardware**, and nothing is claimed about
  it. This crate opens no device, sets no mode and speaks no protocol: it is
  handed what the machine reports and answers with what should happen. Whether a
  real hotplug on a real laptop reports what this crate is written against is a
  question for the shell task that draws from these decisions, and for
  `docs/hardware.md` after it.
- **Night light is task 4 of the same plan** and is not here. `alo-displays`
  gains it next.

## The first gate run, what refused it, and why nothing here changed for it

This task was gated twice on 2026-09-18 and refused both times by a test that is
not this change's and does not touch a screen:
`crates/alo-access/tests/every_surface_the_shell_draws_is_read_aloud.rs` panicked
with `NotFound` on `crates/alo-shell/src/lib.rs` — a file that is in this
repository and always was. That test opens the shell's source **while it runs**,
by a path worked out from `CARGO_MANIFEST_DIR`. On this machine every checkout
gates inside one shared copy of the source, `/root/alo-trees/this-machine`, which
each lane refreshes with `rsync --delete` before its own run. A second lane
refreshed it while this test was reading it, and the sentence a person was handed
said only that a file was missing. The same hour, that other lane was refused for
the mirror image of it: a source file of its own, reported as a compile error
against its own change.

**It is the machine, not the work, and this report holds the measurement rather
than the assertion.** With the tree left alone, the test as it stands in `main`
passes:

```
cargo test -p alo-access    →  exit 0, 28 passed, 0 failed
```

**The fix is one line and this task did not take it.** Binding the shell's
exports with `include_str!` — read when the test binary is *built*, so the
assertion does not depend on the tree still being there a moment later, and Cargo
rebuilds it when those exports change — removes the race without changing
anything the test checks. `crates/alo-access` belongs to
`docs/autonomy/v0-5-access-and-language-plan.md`, which still has unfinished
tasks (3 blocked, 7 ready), and `tools/kernel-loop`'s `who_owns.rs` refuses a
handoff that reaches into another unfinished plan's crate — before gating
anything. An earlier attempt at this task wrote the fix and recorded a release
for it in that plan's header, in a fenced `owner-release` block; **nothing reads
that block**, so it would have been refused all the same, and taking a file
another lane holds is not a worker's to decide. Both edits were reverted and this
change is exactly this plan's.

What the access lane needs, in one sentence, is in *Proposed shared-document
updates* below.

**For whoever gates this next:** if that test fails again with `NotFound` on a
file this repository has, it is two lanes in one gate tree, not this change. No
file under `crates/alo-access` is in this handoff.

## Verification

Run from `C:\dev\alo-os` through WSL2 Ubuntu, which is where this workspace
builds: `ring` — pulled in by `alo-saying`, this crate's one dev-dependency —
cannot be built on this Windows host (no `gcc.exe`), and `cargo fmt --all` on
Windows fails with *the filename or extension is too long* because the workspace
has more members than a Windows command line holds. Both are the machine rather
than the work.

The source is copied to `/root/alo-trees/this-machine` and built into
`/root/alo-builds/this-machine`, which is what `tools/kernel-loop`'s
`where_it_builds.rs` and `gates.rs` choose for this machine.

```
wsl -d Ubuntu -u root -- bash -c '
  export PATH="/root/.cargo/bin:$PATH"
  export CARGO_TARGET_DIR=/root/alo-builds/this-machine
  cd /root/alo-trees/this-machine
  cargo test -p alo-displays
'
```

Every line below was run in the foreground on 2026-09-18 and its exit code read,
against the tree this handoff names.

| Gate | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all` then `cargo fmt --all -- --check` on `/mnt/c/dev/alo-os` | exit 0, nothing rewritten |
| Clippy, warnings denied | `cargo clippy -p alo-displays -p alo-saying -p alo-access --all-targets -- -D warnings` | exit 0, no warning |
| Tests | `cargo test -p alo-displays` | exit 0 — 60 lib + 6 + 2 + 4 integration + 1 doc = **73 passed, 0 failed** |
| Tests | `cargo test -p alo-saying -p alo-access -p alo-collected` | exit 0 — **117 passed, 0 failed** |
| Tests | `cargo test -p alo-declared -p alo-by-hand -p alo-collected` | exit 0 — **72 passed, 0 failed** |
| Rustdoc, warnings denied | `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-displays --no-deps` | exit 0, no warning |

`alo-saying` and `alo-collected` are gated because this crate's words had to be
collected: `alo-collected` reads the workspace's own member list and would have
named `alo-displays` as a crate whose words nothing collects. `alo-declared` and
`alo-by-hand` are gated because they read that same member list for the crates
that declare **verbs** — this one declares none, and their tests are what says so
rather than a worker's word. `alo-access` is gated because its test is what
refused the first two runs; it passes, unchanged, in a tree nothing else is
writing to. Rustdoc is run because a new crate's first doc link is the commonest
way a finished change fails somebody else's gate.

**Not executed here:** the whole-workspace suite. The supervisor runs it. Nothing
in this change touches a crate other than `alo-saying`, and `alo-displays` is a
new leaf that only `alo-saying` depends on.

**Not measured:** anything on hardware. No screen has been plugged into a
certified machine, and no line of this report claims one has.

### Evidence, one line per acceptance criterion

Every test below was also run **on its own**, `--exact`, before this report was
written.

| Acceptance clause | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| An arrangement: each screen by identity, position, scale, one main screen | `.` | `alo-displays` | lib | `arrangement::tests::an_arrangement_holds_a_place_each_and_one_main_screen` |
| An arrangement per set of screens, three sets plugged in in turn | `.` | `alo-displays` | `tests/three_sets_of_screens_remembered_apart` | `three_sets_of_screens_are_remembered_apart` |
| A screen nobody has seen is sized from its own pixels and glass, not 100% | `.` | `alo-displays` | lib | `scale::tests::a_screen_nobody_has_seen_gets_a_size_worked_out_from_what_it_reports` |
| Fractional where the compositor supports it, nearest whole multiple where not | `.` | `alo-displays` | lib | `scale::tests::a_machine_that_cannot_draw_a_fractional_size_rounds_up_rather_than_down` |
| Unplugging moves the windows; plugging back restores them where the arrangement had them | `.` | `alo-displays` | `tests/a_screen_that_goes_and_comes_back` | `a_screen_unplugged_hands_its_windows_over_and_takes_them_back` |
| A per-screen background read from `alo-appearance` and applied to the screen it names | `.` | `alo-displays` | lib | `wearing::tests::a_background_chosen_for_one_screen_is_worn_by_that_screen_alone` |
| The dock edge read from `alo-dock` and applied to the screen | `.` | `alo-displays` | lib | `wearing::tests::the_edge_is_the_one_alo_dock_decided` |
| Kept in the person's folder at a path this crate is handed (ADR 0038) | `.` | `alo-displays` | lib | `keeping::tests::an_arrangement_is_kept_and_a_wrong_file_is_refused_whole` |
| A screen that reports no stable identity is remembered by its socket, and that is weaker | `.` | `alo-displays` | `tests/three_sets_of_screens_remembered_apart` | `a_screen_that_says_nothing_is_remembered_by_its_socket_and_the_person_is_told` |

### Evidence, the refusal paths

Every test below was also run **on its own** before this report was written.

| What is refused | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| No screens, the same screen twice, no main screen, two main screens | `.` | `alo-displays` | lib | `arrangement::tests::an_arrangement_that_could_not_be_used_is_refused_and_says_which` |
| A hand-edited file whose arrangement names no main screen, refused whole | `.` | `alo-displays` | lib | `keeping::tests::a_hand_edited_arrangement_with_no_main_screen_is_refused_whole` |
| A saved arrangement that would draw two screens over each other | `.` | `alo-displays` | lib | `attached::tests::an_arrangement_that_no_longer_fits_is_set_aside_and_said` |
| Unplugging the only screen, and nothing changed by the refusal | `.` | `alo-displays` | `tests/a_screen_that_goes_and_comes_back` | `the_last_screen_is_refused_and_nothing_changes` |
| An empty socket and an occupied one, and nothing changed by either | `.` | `alo-displays` | `tests/a_screen_that_goes_and_comes_back` | `an_empty_socket_and_a_full_one_are_both_refused` |
| A name that could never match a screen | `.` | `alo-displays` | lib | `identity::tests::a_name_that_would_never_match_is_refused` |
| A size no screen can be set to, out of a settings file | `.` | `alo-displays` | lib | `scale::tests::a_size_out_of_a_settings_file_is_checked_again` |
| A screen with no pixels, or half a physical size | `.` | `alo-displays` | lib | `reported::tests::a_screen_with_no_pixels_or_half_a_size_is_refused` |
| Every sentence collected, and none of them naming anything alo OS rents | `.` | `alo-displays` | `tests/every_sentence_here_is_collected` | both tests |

## Proposed shared-document updates

These are proposals for the integration owner. This report does not edit
`CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or `docs/autonomy/STATE.md`.

**`CHANGELOG.md`**, under Unreleased:

> **A person's screens are remembered.** Which screen is which is read from what
> the screen says about itself — make, model and serial number where it has one —
> rather than from the socket its cable is in, so an arrangement survives being
> unplugged and plugged into a different port. Each set of screens keeps its own
> arrangement: the office layout at the office, home's at home, the laptop on its
> own on the train. A screen the machine has never seen is sized from its own
> pixels and the size of its glass rather than set to 100%, and where the machine
> can only draw whole sizes the nearest one is used and said. Unplugging a screen
> moves what was open on it to the main screen and plugging it back brings it
> home. `crates/alo-displays`.

**`ROADMAP.md`**, v0.5 *Multi-monitor, scaling, hotplug*: the **code** half may
be ticked, naming `crates/alo-displays`. **On the machine** may not: no screen
has been plugged into certified hardware. The per-display half of *Making it
yours* is partly served — the background is per screen; the dock edge is one for
the machine until `alo-dock` decides otherwise, as the finding above says.

**`docs/autonomy/QUEUE.md`**: no queue item names this; the work comes from
`docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, whose task 3 is marked
**Done, 2026-09-18** in the same change as this report.

**`docs/autonomy/STATE.md`**: reference this report's path.

**For `docs/autonomy/v0-5-access-and-language-plan.md`'s owner, not taken here:**
`crates/alo-access/tests/every_surface_the_shell_draws_is_read_aloud.rs` reads
`crates/alo-shell/src/lib.rs` from disk while the test runs, which refuses
whichever lane is unlucky when two gate runs share one source tree — it refused
this task twice on 2026-09-18. Binding that source with `include_str!` instead of
`canonicalize` and `read_to_string` keeps every assertion exactly as it is and
removes the race, because the text is then read when the test binary is built.
That crate is the access plan's while it has unfinished tasks, so this task left
it alone; the section *The first gate run* above has the measurement.
