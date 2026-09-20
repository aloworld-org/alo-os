# Several displays, each with its own background and its own dock

**Task 9 of `v0-5-the-shell-plan.md`. 2026-09-20. The code only — no certified
machine, and no second panel, has seen this.**

## What was built

Three files in `crates/alo-shell`, one responsibility each:

| | |
|---|---|
| `src/screens.rs` | Which outputs there are, where each is drawn, how large, and what each wears |
| `src/screen_background.rs` | What is behind the windows on one screen: the chosen colour or picture, fitted to that screen's room and warmed by its night light |
| `src/screens_raster.rs` | The desk: one picture per screen — its background, then its own dock over it |

`alo_shell::desk` is the public door: a session hands it the screens, the
person's dock and their appearance, and gets one `ScreenPicture` per output to
paint.

## Every decision in it is somebody else's

`alo-displays` answers which screen is which, where each sits, how large each
draws, which is the main one, whether this set has been arranged before, and
where the windows of a screen that was unplugged belong. `alo-appearance`
answers what is behind the windows on each. `alo-dock` answers which edge the
dock is on. Night light's warmth is `alo-displays`' too, and is applied **per
screen** because that is where that crate puts it.

`tests/screens_source.rs` reads the three shipped files and fails the build if
any of them builds an `Arrangement`, places a screen, invents a `Scale`, writes
to a setting it only draws, or names a colour. A test that drew two screens
would only show the desk it was handed; this one reads the files instead.

## How it was measured

Under the nested-compositor measurement this crate uses for every other
surface: the pictures are laid out and rasterised on the CPU and the raster is
read back and asserted, which is exactly what `desktop_raster_tests.rs` and
`nested_desktop_tests.rs` do for the ordinary desktop. **Two outputs** in these
tests means two `alo_displays::Reported` screens on one desk — a 13.3-inch
laptop panel at 1920×1080 and a 27-inch monitor at 3840×2160, both reporting
the numbers those panels really report — laid out by `alo-displays` and drawn
as two pictures.

Every clause of the acceptance, and where it is held:

| Clause | Test |
|---|---|
| Outputs laid out as `alo-displays` arranges them, at the scale it names | `every_screen_is_where_alo_displays_put_it_at_the_size_it_named` |
| An arrangement restored when a known set is plugged in | `an_arrangement_is_restored_when_that_set_is_plugged_in_again` |
| Windows moved off an unplugged display to where that crate says | `what_was_on_a_screen_that_went_belongs_where_alo_displays_says` |
| Each display its own background and its own dock on its own edge | `each_screen_wears_its_own_background_on_the_edge_alo_dock_names`, `two_screens_are_two_pictures_each_with_its_own_background_and_dock` |
| Night light applied per display | `night_light_reaches_every_screen_beside_its_own_background`, `night_light_warms_the_background_and_the_dock_on_every_screen` |

One measured number worth keeping: the two screens' docks are **not** the same
size. The laptop's band is laid out for the laptop's room and the monitor's for
the monitor's, because `alo_dock::Dock::layout_on` is asked once per screen.
A dock laid out once for the whole desk and cut into pieces is the bug that
gives a 4K monitor the dock a 1366-wide laptop needed, and
`two_screens_are_two_pictures_each_with_its_own_background_and_dock` fails if
that ever becomes true.

The room a surface lays itself out in is the screen's pixels at the size it is
**drawn** at — `OnScreen::drawn_at`, the rounded one — and not the size it was
asked for. Laying a dock out for a size the machine cannot draw is how a dock
ends up half off the panel.

## What this does not prove

**Two displays have never been plugged into this machine.** Everything above is
the layout logic and the pixels it produces, measured on two outputs a test
describes. Hotplug on real hardware — a cable going in or coming out, a monitor
that reports itself differently on the second plug, a mode the kernel refuses —
is not measured here and the plan is explicit that a headless test and a
certified machine are different things. The `- [x]` this task earns is *the
code*, and nothing more.

There is also no direct-display submission of a desk: `desk` produces the
pictures and `ScreenPicture::paint` paints one into a frame, but the DRM path
still drives one output at a time.

## Findings

**`alo-dock` still holds one edge for the whole machine.** Each screen's dock is
drawn from that screen's own `alo_displays::Wearing`, which is the one place a
per-screen edge will appear — but `Wearing::edge` is `Dock::edge` today, so
every screen's dock is on the same edge. `alo-displays`' own `wearing.rs` says
so in its header and this plan reads that crate rather than editing it. When
`alo-dock` decides an edge per screen, nothing in `alo-shell` has to change.

**The image reader words every failure as the lock screen's.** The two doors a
background is read through — `lock_background_path::selected` and
`lock_image_decode::decode` — are shared with the lock screen, and a missing
file comes back as `RenderError::Submission("lock background: …")`. Neither
door refuses for any other reason, so every refusal they make is *the desktop
could not draw what a person chose*, and it is said that way here. A desktop
frame refusing in the lock screen's words would send somebody looking at the
wrong surface. Renaming those two files to something neither surface owns is the
tidy version of this and was not done in a task about displays.

**A rotating background rotates on one clock for the whole desk.**
`alo-appearance` decides which picture of a folder is showing from how long the
session has been running, and that is one answer for the machine — so two
screens rotating through the same folder show the same picture at the same
time. Nothing decides otherwise, so nothing here invented a second clock.
