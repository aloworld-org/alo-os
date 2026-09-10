# The egress indicator, on a screen

**Date:** 2026-09-10
**Workstream:** v0.01 delivery plan, task 9 — *The egress indicator, on a screen*
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What changed, in words a person outside this repository can read

alo OS is sold on one promise before any other: **nothing leaves your machine
silently.** Until now that promise was kept where nobody could see it. The part
of the system that decides what may leave has always put every departure on a
list at the moment it permits one — but no screen had ever shown that list, so
*the indicator stayed dark all day* was a claim about something a person could
not look at.

This change is the indicator a person looks at. It is a light with a readable
list under it. While a question is answered on the machine itself, the light is
dark and there is nothing under it. The moment a question, a fetch or anything
else goes to the network, the light comes on and a line appears saying who is
doing it, where to, and why — in the person's own language. When the connection
ends, the light goes out again.

Three things about it are deliberate and are the point:

- **It shows what the machine's own list says, and there is no other way to put
  anything on it.** Nothing can light it by handing it a number.
- **A connection the machine refused never appears.** Nothing left, so there is
  nothing to show — an indicator that lit up for the connections a rule stopped
  is one people learn to ignore.
- **If it cannot be shown at all, the machine says so in a sentence.** A machine
  that quietly failed to draw the indicator would look exactly like a machine on
  which nothing was leaving, which is the one failure this whole promise does not
  survive.

## What was built

A new crate, `crates/alo-indicator`, six files:

| File | What it is |
|---|---|
| `src/lamp.rs` | `Lamp` — the light: dark, or lit for this many things |
| `src/drawn.rs` | `Drawn` — the light and the readable lines under it, from one moment |
| `src/surface.rs` | `Compositor`, `SurfaceRefused` — the compositor's half, with no rendering in it |
| `src/indicating.rs` | `Indicating`, `Drew` — what is on the screen, kept in step with what is leaving |
| `src/refusing.rs` | `NotShown` — the refusal in words when there is nowhere to show it |
| `src/words.rs` | Every string this crate can say, and the English beside each |

And the wiring one new crate that says something needs: `Cargo.toml`
(workspace member), and `alo-saying` — the dependency, `EVERY_LIST`, the
`declare_into` call and the three tests in `collecting.rs` that count and name
what the machine can say. A crate whose strings `alo-saying` does not collect is
a crate whose sentences reach a real shell as keys in guillemets; task 3 found
that the hard way for `alo-overlay`, and doing it in the same change is the
lesson.

## Decisions this task left open, and what was chosen

**A crate of its own, named `alo-indicator`.** The alternative was another
module inside `alo-overlay`. It is not the overlay: the overlay is summoned by a
key and dismissed, and this is up for as long as the session is. `alo-egress`
already owns the type called `Indicator` — the decision and the list — and this
crate deliberately exports nothing by that name: `Lamp` is the light, `Drawn` is
the picture. A stranger asking *where is the egress indicator drawn* should find
it by the crate's name, which is what settled it.

**A struct with a private field, not an enum with a lit variant.** The plan's
fourth criterion is *it cannot be drawn from a value that was not a departure*.
A public `enum Lamp { Dark, Lit(NonZeroUsize) }` would let any caller construct
a lit lamp out of a number, so `Lamp` is a struct whose one field is filled by
`Lamp::of(&Indicator)` and by nothing else. The same holds for `Drawn`. Two
compile-fail examples on each say so as tests rather than as prose: a second
door added later stops being a design discussion and becomes a failing build.

**Deliberately no `of_how_many`, which `alo-overlay`'s `Quiet` has.** `Quiet`
needs one because a shell can be *told* what a daemon's indicator holds over
`alo-protocol`. Giving the light on the screen the same door would mean a
machine with two answers to *has anything left this machine*, which is exactly
what law 1 does not survive. Drawing an indicator from another machine's report
is a different feature; it will be a different type with the report in its name,
and it is not v0.01's.

**The request handed to the compositor *is* the picture.** `alo-overlay`'s
summoning passes an empty `SurfaceRequest`, because *summon the agent* has
nothing to carry. Here the compositor is handed the `Drawn` itself, so it can
never be asked to put up an indicator and then left to decide what it says.
That is what makes *nothing can be drawn that was not a departure* a fact about
the seam rather than about the compositor's good behaviour.

**One change, one redraw.** `Indicating` keeps the last picture and does not
send an identical one, so a shell may call it on every frame. A refusal
**forgets** that picture. The alternative — remembering it — means that when a
display comes back showing the same count as before, nothing is sent, and a
person is left looking at an indicator that has not been asked a question since
their screen was unplugged. Forgetting costs one redraw; remembering costs the
guarantee.

**The indicator goes up dark rather than appearing only when something
leaves.** A person cannot tell a dark light from a light that is not there, so a
light that only existed while something was leaving would have nothing to say on
the day it mattered.

**Four new strings, under a new `indicator` area** — a reading for the dark
light, a counted one for the lit light, and the two refusals. The two readings
say nearly what `overlay.egress.nothing-is-leaving` and
`overlay.egress.how-many-are-leaving` say, and that near-duplication was
considered rather than overlooked: the overlay's are lower-case clauses read in
a list under two other clauses, and these are **the whole of what one control
says**, announced on their own by a screen reader, which is why they are
capitalised sentences. Each note tells the translator which of the two places it
is for, and says the two are shown at the same time on one machine and must not
disagree. A light with no name is a light that does not exist for somebody using
a screen reader, and law 1 is not a promise alo OS keeps for people who can see
the light and breaks for people who cannot.

**The refusals are read somewhere other than the screen they are about** — a
service log, or a text console — because the screen is the thing that is
missing. They are still declared and still translated: a person reading a log on
their own machine reads their own language.

## Acceptance, and the test behind each criterion

Every one of these was run on its own, on Windows, against this change.

| Criterion | Test |
|---|---|
| The indicator is drawn from `alo_egress::Indicator` and nothing else | `the_indicator_is_drawn_from_the_machines_own_indicator_and_nothing_else` |
| A local answer leaves it dark | `a_local_answer_leaves_the_indicator_dark` |
| A provider answer lights it while the question is in flight | `a_provider_answer_lights_it_while_the_question_is_in_flight` |
| It cannot be drawn from a value that was not a departure | `it_cannot_be_drawn_from_a_value_that_was_not_a_departure` |

All four are in
`crates/alo-indicator/tests/the_egress_indicator_on_a_screen.rs`, and all four
reach the machine the way a shell does: a real `alo_egress::Indicator` that
really permitted something, a picture that reaches a compositor through the port
`alo-shell` will implement, and strings from
`alo_saying::everything_this_machine_can_say` — the vocabulary a shell really
holds, so a string this crate declares and nobody collects fails there rather
than on somebody's screen.

### The refusal paths, tested beside the legitimate ones

- `nowhere_to_show_it_is_refused_in_words_rather_than_in_silence` — both ways of
  having nowhere (no compositor at all; a compositor with no display) refuse
  with a sentence a person can read, neither leaves anything believing an
  indicator is up, and a screen arriving redraws without waiting for the machine
  to change.
- `it_cannot_be_drawn_from_a_value_that_was_not_a_departure` — an egress refused
  by `NothingLeaves`, and a second refused by `InTheBuilding`, so it is a rule
  about refusals rather than about one policy.
- `an_egress_the_policy_refused_leaves_the_lamp_dark`,
  `an_egress_the_policy_refused_is_drawn_nowhere` — the same at the unit level,
  in the two files where a refusal could plausibly have leaked into the light.
- `a_compositor_with_no_screen_is_refused_and_asked_again`,
  `no_compositor_refuses_in_words_and_believes_nothing`,
  `a_screen_that_came_back_is_redrawn_rather_than_assumed` — a refusal is not a
  picture that is still up.
- Four `compile_fail` examples: `Lamp::of_how_many`, `Lamp { lit: … }`,
  `Drawn::of_lines`, `Drawn { … }`. The refusal path that is held by the
  compiler rather than by a test.
- `a_refusal_is_read_in_the_language_the_person_reads`,
  `the_light_reads_in_the_language_the_person_reads` — and the untranslated case
  is still English and says it is.

## Verification

Windows 11, `C:\dev\alo-os-claude`, this working tree.

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets` | zero warnings, zero errors |
| `cargo test -p alo-indicator` | 31 unit + 5 integration + 8 doc tests, all pass |
| `cargo test -p alo-saying` | passes with the new crate collected |
| `cargo test --workspace` | passes |
| Each acceptance test run individually | passes |

**Not measured, and deliberately.** Nothing here is run on hardware, because
nothing here touches the machine: no device, no socket, no filesystem, no
kernel state. It is a portable model with one dependency, and the integration
test says in its own header that a green suite on a developer's laptop is not
`ROADMAP.md`'s exit gate.

## Limitations, honestly

- **No pixels.** This crate claims none and tests none. What the light looks
  like, where it sits and how it is drawn is the compositor's, and `alo-shell`
  wiring the `Compositor` port up is a separate task in the desktop worker's
  chain. `docs/features.md` puts the indicator's home in the dock's status area
  at **v0.5**; this crate does not know about furniture, which is why that later
  move costs it nothing.
- **`docs/features.md` line 323 is not yet fully shown by this alone.** The
  promise is *every network egress an agent causes, visible at the moment it
  happens*; `alo-egress` makes it decidable and this makes it drawable, and a
  compositor that actually draws it closes it. Task 11 is where that is
  reconciled against evidence, and this report is what that task should read.
- **A shell in a different process from the daemon cannot draw this yet**, by
  design: the only door is a real `alo_egress::Indicator` in the same process.
  Carrying the list over `alo-protocol` is a real piece of work and a real
  decision — the type that does it must be named for the report it is, so that
  nobody mistakes a machine's own list for another machine's account of one.
- **The `alo-saying` crate documentation still says "fifteen crates"** in two
  places; it was already stale at eighteen before this change and is nineteen
  now. Left alone rather than corrected in passing, since it is prose in another
  contributor's file and a one-line fix somebody should make deliberately.

## Proposed changelog, roadmap and queue updates

For the integration owner; not edited here, per `docs/autonomy/SHARED_MAIN.md`.

**CHANGELOG.md** — under the current unreleased section:

> **The egress indicator, on a screen.** What is leaving this machine is now
> something a person can look at: a light that is dark while a question is
> answered on the machine and lit while one is on its way somewhere else, with a
> readable line for each connection saying who, where to and why. It is drawn
> from the machine's own record of what it permitted and from nothing else, a
> connection the machine refused never appears on it, and a machine with nowhere
> to show it says so in a sentence rather than failing quietly.

**QUEUE.md / STATE.md** — v0.01 delivery plan task 9 is done, marked in
`docs/autonomy/v0-01-delivery-plan.md` in this change. Task 10 (*the image
carries the shell, the session and the daemon*) depends on 5 and 9; 5 is lane
B's and is still open, so 10 is not yet startable.

**ROADMAP.md** — no *On the machine* box moves. Nothing here ran on hardware and
nothing here draws.
