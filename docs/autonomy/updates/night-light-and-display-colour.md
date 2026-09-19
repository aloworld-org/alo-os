# Night light and display colour

**Date:** 2026-09-18
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 4
(*Night light and display colour*). `ROADMAP.md` v0.5, `docs/features.md`
*Devices and media*.
**Contributor:** development PC, one worker, one working tree.
**Crate:** `crates/alo-displays` — this plan owns it.
**Status:** ready for integration. Gates below were run on this change; the
whole-workspace suite is the supervisor's.

## What changed, in a sentence a person outside this repository can read

alo OS can now warm your screens in the evening. You either set the hours
yourself — *from ten at night until seven in the morning* — or you type where
you are, once, and the machine works out sunset and sunrise for that spot
itself. It does the arithmetic on your own computer; nothing is sent anywhere,
no service is asked where you live, and if you would rather not say, the
schedule is there and loses you about two minutes a year. How warm is yours to
choose, between two ends the machine will not go past, and the warmth is applied
to each of your screens as it is drawn rather than as a tint laid over the whole
desk. If you live far enough north or south that the sun does not set for weeks
at a time, alo OS says so instead of quietly doing nothing.

## What was built

Six new files in `crates/alo-displays/src`, each with one job.

| | |
|---|---|
| `time_of_day.rs` | `TimeOfDay` — a time on the person's own clock, written `22:00` |
| `between.rs` | `Between` — a stretch of that clock, which nearly always runs through midnight |
| `moment.rs` | `Moment` — what the machine's clock says, handed in rather than read |
| `sun.rs` | `Whereabouts`, `Sun` — when the sun sets and rises there, worked out here |
| `warmth.rs` | `Warmth`, `Warming` — how warm, and what that does to every colour |
| `nightly.rs` | `Nightly`, `Now` — when night light is on: never, a schedule, or the sun |
| `night_light.rs` | `NightLight`, `Tonight` — the person's setting, and what it is doing now |

And changes to five that were there: `changes.rs` and `keeping.rs` (the
`night-light` key in `displays.toml`), `wearing.rs` (where the warmth reaches a
screen), `notes.rs` and `words.rs` (eight new sentences).

Two new tests under `crates/alo-displays/tests`:
`the_sun_is_worked_out_on_this_machine.rs` and
`terracotta_still_means_the_agent_under_night_light.rs`.

## The acceptance criteria, clause by clause

### *on a schedule a person sets, or from sunset to sunrise computed on the machine*

`Nightly` has **three arms and no fourth**: `Never`, `Between(Between)`, and
`FromSunsetAt(Whereabouts)`. The missing fourth is the design: there is no
*follow the sun* that carries no place, so *sunset for your location* on a
machine that was never told a location is not a state this crate can be in, and
the constraint's *absent that, the setting offers a schedule instead* is true by
construction rather than by a check somebody could forget.

`Nightly::asked(Option<Between>, Option<Whereabouts>)` is the door a settings
panel and a settings file both come through, and asking for both at once is
refused in words (`displays.a-schedule-or-the-sun`) rather than resolved by
guessing which the person meant.

`sun.rs` is the published sunrise equation — a series approximation of where the
earth is in its year and how far it is leaning — accurate to about a minute in
the middle latitudes, which is a good deal finer than the question. It needs no
almanac file and no table.

### *never from a location service or a network lookup, held by a test that the calculation opens no socket*

`tests/the_sun_is_worked_out_on_this_machine.rs`. It reads every `.rs` file
under `crates/alo-displays/src`, strips comment lines (prose about the network
is exactly what those files should be full of), and refuses seventeen names:
`std::net`, `TcpStream`, `TcpListener`, `UdpSocket`, `ToSocketAddrs`, `IpAddr`,
`ureq`, `socket2`, `reqwest`, `hyper`, `curl`, `http://`, `https://`,
`geoclue`, `GeoClue`, `getaddrinfo`, `Command::new`. A second test reads
`Cargo.toml` and refuses a dependency on any client, socket library or bus. A
third shows the reader refusing a line that *would* go and ask, so that a reader
which found nothing could not pass on an empty directory. A fourth asks the
arithmetic the question a person asks, so that a test proving only an absence
would still fail if the calculation were deleted.

Deliberately **not** on the refused list: the bare word `socket`. A screen is
plugged into one, and this crate says so several hundred times.

The threat this guards against is not today's code — there is no network call in
it — but tomorrow's: somebody adding *look my town up for me* because typing a
latitude is awkward, which is a kindness that turns a sovereign machine into one
that phones out every evening.

### *its strength is a colour temperature within a closed range*

`Warmth` is kelvin, from `Warmth::WARMEST` (2000) to `Warmth::NEUTRAL` (6500),
refused outside in words that name both ends
(`displays.not-a-warmth`), and refused again on the way out of a settings file.

6500 K is not a preference: it is the white the colour space every screen in
this system draws in is defined against, so it is the one temperature at which
night light changes **nothing** — and *exactly* nothing, which is why
`Warming::at` divides the black-body curve through by its own value at that
point rather than using the raw approximation, which is only nearly one there. A
night light whose *off* left a faint tint would be a night light nobody could
turn off. `warmth::tests::the_neutral_warmth_leaves_every_colour_exactly_where_it_was`
holds it byte for byte across the whole palette.

### *applied per display*

Through `Wearing`, which was already this crate's answer to *what does this one
screen wear*: the background behind its windows, the edge its dock sits on, and
now how warm it is drawn. `Wearing::of` gained a `&Tonight` argument.

`wearing::tests::every_screen_wears_the_warmth_beside_its_own_background` holds
both halves: every screen on the desk carries the same warmth, and each still
carries its own background.

### *terracotta still means the agent under night light*

`tests/terracotta_still_means_the_agent_under_night_light.rs`, and this is the
clause that needed a decision rather than a check. See below.

## Decisions taken, and why

### 1. Hue distance is the wrong measure under night light, and this change says so rather than quietly using it

`alo_appearance::accent`'s own test asks how far each accent sits from
terracotta on the colour wheel, and thirty degrees is its floor. **That measure
collapses under night light for everybody.** Warming takes blue away first, so
magenta, pink and red all converge on orange, and the wheel distance between
terracotta and rose goes to nothing well before the warm end of any usable
range. A test that held the thirty-degree rule under night light would have
forced the warmest setting up to somewhere near neutral, which is a night light
not worth shipping.

What does not collapse is that they are still plainly *different colours* — a
bright warm orange and a dark dull one — and the measure for that is a distance
in a space built so that equal steps look equally different. The test uses
**CIE76 ΔE\*ab**, whose just-noticeable difference is about 2.3, with a floor of
**5.0**: twice that, and the distance at which two colours are routinely called
two colours rather than two printings of one. It walks every warmth in the range
at 50 K and every one of the ten accent values.

**Measured minimum: 9.479 — rose on a dark ground, at 2000 K**, the warmest
setting a person can choose. The margin over the floor is comfortable, and the
test prints the number rather than asserting it, so that a designer improving an
accent does not fail the gate on a margin nobody promised.

The ΔE arithmetic lives in the test file rather than in the crate, the way
`alo_appearance::accent`'s own hue arithmetic does: it is a measurement of a
promise, not a thing the product computes.

### 2. And this is one of two things bounding the warm end — but it is not the one that binds today

`Warmth::WARMEST` is 2000 K for a plainer reason: below about 1900 K the
black-body approximation has no blue left to take and stops being a description
of anything. The agent measurement above is checked across the whole range and
**would refuse a warmer floor**; it does not refuse this one. Both reasons are
written into `warmth.rs`'s header, with which one binds stated plainly, because
a comment claiming a bound that is not actually tight is how a later change
moves it in good faith.

### 3. *With its mark and word beside it as always*

There is a reading of the acceptance clause under which this change would have
had to add something. It does the opposite, and the test says so out loud.

ADR 0010's measured note records that **terracotta on cream is 2.87:1** — under
the 3.0 that WCAG 2.1 §1.4.11 asks of a shape carrying meaning, and under the
4.5 it asks of text. The agent's colour never carried its meaning by itself, on
a warmed screen or a cold one. So
`night_light_does_not_make_the_colour_sufficient_because_it_never_was` holds
that at every warmth in the range the contrast stays under
`ENOUGH_FOR_A_SHAPE` — which makes it impossible to read the ΔE test as *the
colour is enough now*, and makes any future change that would let night light
stand in for the mark and the word a change to ADR 0010 rather than a change to
this crate. The mark and the word themselves are the shell's and are drawn in
the shell plan's tasks; nothing here can draw one or suppress one.

### 4. A timezone is not a place, and is not used as one

The acceptance says *from a location the person typed or a timezone*. Knowing
somebody keeps central European time places them in a stripe fifteen degrees
wide and says nothing about how far north they are — and how far north is the
whole of when the sun sets. There is no honest sunset from an offset alone.

So the offset is used for **one** thing: putting a sunset worked out for the
whole earth onto the clock the person is looking at. `Moment::at` takes a
`SystemTime` and minutes east of universal time, exactly as
`alo_capturing::OnThisDay` does and for the same reason, and nothing in this
crate reads a clock, an environment variable or a timezone database. The
constraint settles the rest: absent a place the person typed, the setting offers
a schedule.

### 5. This crate spells a time of day itself

`alo-appearance` has a `TimeOfDay` and this crate reads that crate for other
things, so reusing it was the obvious move. Two reasons not to, and they are the
same reason. ADR 0038 gives a settings file to the crate that **declares its
shape**: a value in `displays.toml` refused in another crate's words would show
somebody editing their screens file a sentence filed under appearance. And the
sun answers in minutes since midnight, so a time built from this crate's own
remainders needs a door that cannot fail — `TimeOfDay::after_midnight`, which is
`pub(crate)` — and a type belonging to another crate cannot offer that without
opening the door to everybody.

It is also written differently, and better for the person: `"22:00"` as a string
rather than `{ hour = 22, minute = 0 }` as a table. One refusal
(`displays.not-a-time`) covers `25:00`, `half ten` and `22-00`, because they are
one thing to somebody mid-edit.

### 6. There is no *except this one screen*

The obvious extra feature here is turning night light off for one screen. It is
not built, and the reason is in `night_light.rs`: a person doing colour work has
a calibrated screen and turns night light off altogether; a setting that warmed
one screen and not the one beside it would be two screens that disagree about
what white is, which is the thing that person is trying to avoid. None of the
three systems people arrive from offers it either. *Applied per display* is met
by the warmth reaching each screen through `Wearing` as it is drawn — which is
what the shell plan's task 9 needs — rather than by a per-screen setting.

### 7. Inside a polar circle, the machine says what it is doing

`Sun` has three arms: `SetsAndRises`, `NeverSets`, `NeverRises`. Under a sun
that does not set, night light is off; under one that does not rise, it is on
all day. Both are said — `Note::TheSunDoesNotSet` and `Note::TheSunDoesNotRise`
— because a machine that silently did nothing for six weeks looks broken, and
Finland, Sweden and Norway are not edge cases for a European product.

The morning used is the sunrise computed for the same date rather than
tomorrow's; they differ by a minute or two, and a second day's arithmetic for a
dawn nobody is watching is not worth the code. Written down in `sun.rs`.

### 8. The file, and what it costs

`displays.toml` gains exactly one key, `night-light`, under the same `format =
1`: adding a key is additive, an older file without it still reads, and the
refusal for an unknown key still names the key. The shape is **flat** — a
warmth, and at most one of `between` and `sunset-at` — because it is read by
people in an editor, and a `when` with a `between` inside it would be a level of
nesting that exists only because the code has two types:

```toml
format = 1

[night-light]
warmth = 2700

[night-light.between]
from = "22:00"
to = "07:00"
```

Night light is one setting for the machine rather than one per set of screens:
the clock and the sun are the same at both desks, and somebody who warms their
screens at ten does not stop wanting that when they plug a monitor in.
`Changes::night_light` is `None` until the person has touched it at all — *the
person has not asked* and *the person asked for what the release ships* are
different things to a release that later ships something else — so a machine
nobody has asked still has no `displays.toml`.

## The refusal paths, tested beside the legitimate ones

| What is refused | Where | Test |
|---|---|---|
| A time that is not one: `25:00`, `22:60`, `half ten`, `" 9:05"`, `"٢٢:٠٠"` | `time_of_day.rs` | `what_is_not_a_time_is_refused` |
| …and again out of a settings file, as a key rather than an English sentence | `time_of_day.rs` | `a_time_out_of_a_settings_file_is_checked_again` |
| A schedule that begins and ends at the same moment | `between.rs` | `a_stretch_that_begins_and_ends_at_once_is_refused` |
| …and again out of a file, and a key that shape does not have | `between.rs` | `a_stretch_survives_a_file_and_an_empty_one_is_refused_again` |
| A warmth outside the closed range, naming both ends | `warmth.rs` | `a_warmth_no_screen_can_be_drawn_at_is_refused_and_names_the_range` |
| …and again out of a file | `warmth.rs` | `a_warmth_out_of_a_settings_file_is_checked_again` |
| Somewhere that is not on the earth, including *not a number at all* | `sun.rs` | `somewhere_that_is_not_on_the_earth_is_refused` |
| …and again out of a file, and a key that shape does not have | `sun.rs` | `somewhere_survives_a_file_and_is_checked_again_on_the_way_in` |
| A schedule **and** the sun at once, wherever it is asked for | `nightly.rs` | `a_schedule_and_the_sun_at_once_is_refused` |
| …and again out of a file, with no warmth at all also refused | `night_light.rs` | `a_hand_edited_file_that_asks_for_both_is_refused` |
| A hand-edited file asking for both — refused **whole**, the good arrangement in it included | `keeping.rs` | `a_hand_edited_night_light_that_asks_for_both_is_refused_whole` |
| A hand-edited warmth out of range — refused whole | `keeping.rs` | `a_hand_edited_warmth_outside_the_range_is_refused_whole` |
| Every road off this machine, in the source and in the manifest | `tests/the_sun_is_worked_out_on_this_machine.rs` | `nothing_here_can_reach_a_network`, `nothing_this_crate_depends_on_is_a_way_off_this_machine` |
| A warmth at which an accent would stop being plainly different from the agent's colour | `tests/terracotta_still_means_the_agent_under_night_light.rs` | `the_agents_colour_stays_apart_from_every_accent_at_every_warmth` |

`Whereabouts::typed` refusing *not a number at all* is also what makes
`impl Eq for Whereabouts` sound, and that is written where the impl is: two
numbers a person typed are equal or they are not, because neither can be a NaN.

## Verification

Platform: this Windows development PC, gates in WSL (Ubuntu) against a
synchronised copy at `/root/alo-trees/this-machine`, `CARGO_TARGET_DIR` at
`/root/alo-builds/this-machine`, as `tools/kernel-loop/src/gates.rs` runs them.

| Command | Result |
|---|---|
| `cargo fmt --all` (on the checkout, through WSL) | clean |
| `cargo clippy -p alo-displays --all-targets -- -D warnings` | clean, zero warnings |
| `cargo doc -p alo-displays --no-deps`, `RUSTDOCFLAGS="-D warnings"` | clean |
| `cargo test -p alo-displays` | 104 unit + 20 integration + 2 doc tests, all passing |
| `cargo test -p alo-saying` | 63 + 4 + 1 passing — it collects this crate's words |
| `cargo check --workspace --all-targets` | clean, 1m 20s |

Each of the six pieces of acceptance evidence was then run on its own with
`--exact`, and each reported exactly one passing test.

**Not run here, and not claimed:** `cargo test --workspace` and
`cargo build --workspace`, which the supervisor runs; the kernel gates, which
this change does not touch.

**No hardware.** Nothing here opens a device, sets a colour ramp or speaks a
protocol (ADR 0011). The shell applies the warmth per output in the shell plan's
task 9, and that is where a screen first actually goes orange. No certified
machine has drawn a warmed frame, and none is claimed.

## Limitations, and what is owed

**`docs/contracts/person-settings.md` still describes four kept files.** It
names neither `sleeping.toml` (this plan's task 2) nor `displays.toml` (task 3),
so this change adds a key to a file the contract does not yet describe. That gap
predates this task and was not closed here: writing the `displays.toml` section
properly means documenting an arrangement shape this task did not design, and
the contract's own intro promises that every listed file is held to its crate by
a `tests/the_contract_describes_this_file.rs` — a three-hundred-line test that
would also have to be written. Doing half of it would make the contract state
something untrue.

What is owed, concretely: a `displays.toml` section covering `arrangements`
(each screen as `make`/`model`/`serial` **or** `socket`, with `at`, `scale` and
`main`) and `night-light` (`warmth`, and at most one of `between` and
`sunset-at`), its `format = 1`, what a missing file means, and the
`displays.kept.*` refusals; the row in the table at the top; and
`crates/alo-displays/tests/the_contract_describes_this_file.rs` holding the two
together. The same is owed for `sleeping.toml`. **Proposed as a queue item for
the integration owner**, since it spans two finished tasks and a shared
contract.

**The dock's edge is still not per screen.** Unchanged from task 3's finding:
`alo_dock::Dock` holds one edge for the machine, this plan reads that crate and
never edits it, and `docs/features.md`'s *Per display, so the dock can sit along
the bottom of the laptop and down the side of the external screen* is still not
met. `Wearing::of` remains the one function that changes when `alo-dock`
decides otherwise.

**The sunrise equation is an approximation**, to about a minute in the middle
latitudes and less well very near the poles, where the sun crosses the horizon
at a shallow angle. The three polar answers (`NeverSets`, `NeverRises`, and a
crossing) are right; a crossing time within a day or two of the changeover at
69° north may be a few minutes out. Nobody at that latitude is checking their
screens against an almanac, and the alternative is an ephemeris file.

**Nothing watches the clock.** A schedule that begins at 22:00 takes effect when
something asks `NightLight::at`. Who asks, and how often, is the shell's — this
crate has no timer and will not grow one.

## Proposed changelog entry

> **Night light.** Your screens can warm in the evening, on hours you set or
> from sunset to sunrise worked out on your own machine from a place you typed —
> never from a location service and never over the network, which is a test
> rather than a promise. How warm is yours to choose within a range the machine
> will not go past, and it is applied to each screen as it is drawn. Where the
> sun does not set for weeks, alo OS says so. The agent's colour stays the
> agent's colour at every warmth, measured against all five accents.

## Proposed queue and roadmap updates

- v0.5 *Night light and display colour* (`docs/features.md`, *Devices and
  media*): the decision is complete in `alo-displays`; the drawing is the shell
  plan's task 9, which remains blocked on the shell's task 5.
- New queue item, for the integration owner:
  **the contract for `displays.toml` and `sleeping.toml`** — one section each in
  `docs/contracts/person-settings.md`, the table rows, and a
  `tests/the_contract_describes_this_file.rs` in each crate. Blocking nothing,
  but the contract is currently silent about two files a person can edit.
- `v0-5-the-session-and-the-displays-plan.md` task 4 is marked
  **Done, 2026-09-18** in this change. Tasks 5, 6 and 7 remain; 7 still depends
  on all of 1–6.
