# The recovery and rollback screen

**Task 13 of `v0-5-the-shell-plan.md`. 2026-09-20. Built and gated on its own,
and *not published*: a screen reader has nothing to say about it yet, and the
decision that would fix that is not this plan's to make. No certified machine
has seen it either.**

## Why this is not merged

`alo-access` holds what a screen reader is told about every surface the shell
draws, and its own
`tests/every_surface_the_shell_draws_is_read_aloud.rs` reads `alo-shell`'s
export list and fails the workspace when a frame appears there with no
`alo_access::Surface` naming it. Adding `RecoveryFrame` and `RecoveryScreen`
fails it:

> the shell draws `["RecoveryFrame", "RecoveryScreen"]` and a screen reader
> would be told nothing about it — add it to `alo_access::Surface` with its
> role, name and state

**That is the right failure.** A screen a person cannot be told about is a
screen they cannot use, and this is the screen somebody reaches on the day
their machine will not start — the worst possible one to be unreachable. It is
also the reason the whole branch stays in draft rather than being published
with a red workspace behind it.

What it asks for is the role, the name and the state of each of this screen's
controls, and the words for them. That is an **accessibility** decision.
`docs/autonomy/v0-5-the-shell-plan.md` says this plan owns `crates/alo-shell`
and nothing else, and that *if a surface needs a decision that is not there,
that is a finding in the report and the task stays open; it is never a decision
made in a drawing crate.* So it was not made here. What is needed, concretely:

- `alo_access::Surface::Recovery`, with `drawn_by` naming
  `["RecoveryFrame", "RecoveryScreen"]`;
- its `read_aloud` controls — at least the window itself, the sentence that is
  read and not acted on, and the two moments as buttons with nothing chosen;
- the words for those in `alo_access::words`, with
  `EVERY_NAME_A_READER_SAYS` moved;
- its place in `alo_access::reaching`'s focus order and what leaving it means —
  which for this screen is *nothing*, since there is nowhere to be sent.

Once that exists the branch merges unchanged. Nothing in `alo-shell` has to
move.

It is worth saying that this is the same question task 12 of this plan asks —
*the shell exposes every surface's role, name and state to AT-SPI as
`alo-access` decides them* — so the two want the same decision, and whichever
lane makes it unblocks both.

`ROADMAP.md` v0.5: *recovery and rollback screen — reachable when the workspace
is not.*

## What was built

| | |
|---|---|
| `src/recovery_screen.rs` | What the screen stands at, and the one thing a person can choose |
| `src/recovery_keys.rs` | What one key press means: move, choose, or nothing |
| `src/recovery_seat.rs` | Those keys through the seat this compositor already has |
| `src/recovery_raster.rs` | The panel laid out and rasterised for one output |
| `src/recovery_paint.rs` | Validating and painting it as the whole output |
| `src/recovery_reached.rs` | Reaching it **because the desktop would not start** |
| `src/nested_recovery.rs` | Pumping it, drawing it, and the frame that draws one or the other |

## Reachable when the workspace is not

The only way that promise is true is if the thing that notices the workspace is
not there is the thing that draws the screen. So
`Nested::submit_desktop_or_recovery` composes the ordinary desktop first and,
when that refuses **for any reason**, draws the recovery screen in its place and
hands the refusal back beside it — for a maintainer's log, never onto the
screen, where it would be a sentence no vocabulary collected.

`recovery_reached_tests.rs` starts the shell with the desktop made to fail — an
egress indicator that has never been told what is leaving, which is a refusal
the desktop already makes — and finds the recovery screen, with going back
offered on it and the desktop's refusal carried beside rather than drawn.

There is no third fallback. A machine that can draw neither its desktop nor one
panel hands the refusal back, because a panel holding half a sentence about
replacing the operating system would be the most misleading screen on the
machine.

## Reachable before sign-in, and it touches nothing a person owns

`tests/recovery_source.rs` reads the seven shipped files and fails the build if
any of them names an account, a session, a greeting or the sign-in screen; opens
a file, names a path or reaches a person's folder; stages, returns, runs a
command or starts a process; or words a sentence or names a colour of its own.
The screen is handed what the base reported and the person's vocabulary, both of
which exist before any account does.

## It decides nothing and carries nothing out

Whether going back can be offered at all is `alo_keeping_up::GoingBack::offered`
— decided from what the base reports and the record's last change, **before
anything is offered**, so a return that cannot be done says so instead of being
offered and failing halfway. Every sentence is that crate's: the offer is
`GoingBack::said`, each refusal is `CannotGoBack::said`, and the two moments are
`GoingBack::when_word` through the vocabulary. Choosing hands back the
`GoingBack` that was decided and the moment the person chose, and the screen is
gone; making it happen is `Returning` through the broker.

## Keyboard alone

`RecoveryKey` is Tab, Shift+Tab, Enter and Space, and nothing else — routed
through the seat by `Server::recovery_key`, which intercepts every key and
forwards none. **There is no Escape.** The approval surface has none because
dismissing would be a third answer; this screen has none for a different reason:
a key that took it away would leave a person looking at a machine that does
nothing, with no way back to the one road out. Leaving this screen is restarting
the machine.

**Nothing is preselected**, and an Enter held down from whatever failed a moment
ago chooses nothing — the seat returns `None` for a repeated press of a key
already down, and the screen returns itself unchanged when Enter arrives with
nothing selected. Both halves are measured.

**A selected moment is told apart by two shapes, never a colour**: a thicker
edge *and* a bar under its words. `a_selected_moment_is_told_apart_by_two_shapes_and_not_by_a_colour`
holds that the set of colours the panel paints is identical whether something is
selected or not (EN 301 549; `docs/features.md`, ★).

## How it was measured

Under the nested-compositor measurement this crate uses for every other surface:
the panel is laid out and rasterised and the raster is asserted. 25 tests across
the screen, the keys, the seat, the raster and the road to it.

## The half that is not drawn either, and why

**The acceptance asks for *what is running* and *what it replaced*, and no crate
words either.** `alo-keeping-up` has a sentence for the offer, a sentence for
every reason it cannot be offered, and words for the two moments — but nothing
that says *this machine is running the build it started on Tuesday* or *it
replaced the one before*. `Deployments` has no `said`; `Since` has no words at
all; and that crate's own header says a person is told that an update is ready,
**never which build it is**, so drawing a digest would break its rule as well as
this plan's.

Writing those sentences here would be the drawing crate deciding, which is the
thing this plan exists to refuse. So they are **not drawn**, exactly as task 5
did not draw a clock it had no crate to ask: that half needs a word in
`alo-keeping-up` and is recorded as a finding rather than quietly dropped.

What a person does learn about the build they are running is carried by the
refusals: a machine whose base names no booted build reads *which version of its
system this machine runs could not be read*, which is
`CannotGoBack::NotRunningABuild` and is `alo-keeping-up`'s own sentence.

## Other findings

**The recovery panel and the approval panel lay their answers out twice.** Both
draw a sentence with framed answer boxes under it, a thicker edge and a bar for
the selected one. They are not the same layout — the approval surface puts two
short answers side by side when they fit and mirrors them for a language read
right to left, while the recovery screen always stacks two whole sentences one
above the other and has no row to mirror — so the two were not merged. A shared
*two answers on a panel* module is the tidy version and is a change of its own.

**A certified machine has seen none of this**, and neither has a machine whose
desktop really failed to start: the desktop in the test was made to refuse by
withholding what the egress indicator needs, which exercises the same road a
real failure would take but is not one.

## What a screen reader is told about it

`alo_access::Surface::Recovery` was added with this screen, because a surface the
shell draws that `alo-access` cannot name fails that crate's guard —
`every_surface_the_shell_draws_is_read_aloud` reads `alo-shell`'s own exports and
holds the tree to them. It names the controls and **none of the sentences**: a
Window for the screen, a Label for what is running, a Label for what it replaced,
a List of the choices, and a Button for each of the two moments. Every sentence
stays `alo-keeping-up`'s — `GoingBack::said`, `CannotGoBack::said`,
`GoingBack::when_word` — read after the name, the way an application's own name
is read after *an application*.

Three decisions, and the reasons rather than the values:

- **Nothing is preselected.** Both moments are `CanBeUsed` and neither carries a
  state that reads as chosen. One of the two restarts the machine, and an Enter
  held down from whatever just failed would otherwise be that.
- **Two labels are named and say nothing yet.** `Deployments` has no `said` and
  `Since` has no words, so *what is running* and *what it replaced* have a name
  and no sentence. Naming them anyway is what lets a reader announce the line;
  the gap is written down as a finding in
  `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` and is that crate's to
  fill.
- **Recovery comes before sign-in** in `Surface::ALL`, which is the reading order
  and therefore the focus order. Everywhere else sign-in is first; this surface
  exists precisely when the desktop will not start, so somebody who cannot see
  the screen must reach it without an account. Escape on it is
  `Leaving::AlreadyTheFloor` — there is nowhere further back, and leaving it for
  real is restarting the machine, which is why `RecoveryKey` takes no Escape at
  all.

The crate's own tests agreed with all three: 32 names in the tree, 41 words in
the crate, and the focus order is the reading order with the labels taken out.
