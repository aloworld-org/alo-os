//! That a control's name appears beside it, toward the canvas and outside the
//! bar, in the look the owner chose, at the display's scale — and that a Dock
//! with no name to show is the same bytes it was.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_dock::a_dock_entry::ANativeAction;
use alo_dock::places::WhatIsHere;
use alo_dock::{AppId, Dock, Edge};
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use super::{
    A_LINE_IS, ABOVE_AND_BELOW_THE_WORDS, placed, show_the_name, the_name_shown, what_it_says,
};
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;
use crate::the_docks_words::TheDocksWords;

/// The faces the shell draws with, and only those, so a hash or a width here
/// does not depend on the machine's own fonts.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts
}

/// The Dock's words, in English.
fn words() -> TheDocksWords {
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    TheDocksWords::said(&strings, &crate::WhatTheDockHasOpen::Nothing)
}

/// These applications on the Dock, one window each.
fn on_the_dock(named: &[&str]) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("a patch");
    let mut windows = alo_dock::Windows::none();
    for (number, name) in named.iter().enumerate() {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(AppId::named(name).expect("a named application")),
            "",
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::Holding::nothing().showing(&windows)
}

/// The Dock on `edge` of a display of `size` at `scale`, holding `named`.
fn drawn(edge: Edge, size: (i32, i32), scale: DisplayScale, named: &[&str]) -> DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let apps = on_the_dock(named);
    crate::dock_raster::picture(
        &dock,
        noon_look(&an_appearance(), Direction::LeftToRight),
        size,
        &apps,
        &apps,
        (&crate::WhatTheDockHasOpen::Nothing, &words()),
        None,
        &alo_dock::artwork::NOTHING,
        &mut fonts(),
        scale,
    )
    .unwrap_or_else(|why| panic!("{edge:?} would not draw: {why:?}"))
}

/// Which slot `what` is, counted as the keyboard and the pointer count.
fn slot_of(picture: &DockPicture, what: &WhatIsHere) -> usize {
    let place = picture
        .standing
        .places
        .each()
        .iter()
        .find(|place| place.what() == what)
        .unwrap_or_else(|| panic!("{what:?} is not on this Dock"));
    crate::the_bars_pressable_slots::which_one_is(&picture.standing.places, place)
        .expect("a control a press reaches")
}

/// Whether two rectangles share any pixel.
fn overlap(a: Rectangle<i32, Physical>, b: Rectangle<i32, Physical>) -> bool {
    a.loc.x < b.loc.x + b.size.w
        && b.loc.x < a.loc.x + a.size.w
        && a.loc.y < b.loc.y + b.size.h
        && b.loc.y < a.loc.y + a.size.h
}

/// **Icons and marks have a tooltip; a control that draws its own words has
/// none.**
#[test]
fn a_tooltip_says_the_name_of_what_draws_no_words() {
    let words = words();
    let app = WhatIsHere::Application(AppId::named("Files").expect("a named application"));
    assert_eq!(what_it_says(&app, Edge::Bottom, &words), Some("Files"));
    assert_eq!(
        what_it_says(
            &WhatIsHere::ANativeAction(ANativeAction::Settings),
            Edge::Bottom,
            &words
        ),
        Some("Settings")
    );
    assert_eq!(
        what_it_says(
            &WhatIsHere::ANativeAction(ANativeAction::Applications),
            Edge::Bottom,
            &words
        ),
        Some("Applications")
    );
    assert_eq!(
        what_it_says(&WhatIsHere::TheOverflow, Edge::Bottom, &words),
        Some("Show more apps")
    );
    for drawn_in_words in [
        WhatIsHere::TheComposer,
        WhatIsHere::ShowAll,
        WhatIsHere::History,
        WhatIsHere::TheDivider,
    ] {
        assert_eq!(
            what_it_says(&drawn_in_words, Edge::Bottom, &words),
            None,
            "{drawn_in_words:?}"
        );
    }
}

/// **Toward the canvas on every edge, a gap clear of the bar, centred on the
/// control.**
#[test]
fn the_panel_opens_toward_the_canvas_on_every_edge() {
    let display = (1920, 1080);
    let gap = 8;
    let size = (100, 26);
    for (edge, band) in [
        (
            Edge::Bottom,
            Rectangle::new((600, 996).into(), (700, 76).into()),
        ),
        (Edge::Top, Rectangle::new((600, 8).into(), (700, 76).into())),
        (
            Edge::Left,
            Rectangle::new((8, 200).into(), (70, 600).into()),
        ),
        (
            Edge::Right,
            Rectangle::new((1842, 200).into(), (70, 600).into()),
        ),
    ] {
        let slot = if edge.runs_across() {
            Rectangle::new((900, band.loc.y + 14).into(), (48, 48).into())
        } else {
            Rectangle::new((band.loc.x + 11, 500).into(), (48, 48).into())
        };
        let at = placed(band, slot, size, edge, gap, display);
        assert!(
            !overlap(at, band),
            "{edge:?}: {at:?} is on the bar {band:?}"
        );
        match edge {
            Edge::Bottom => assert_eq!(at.loc.y + at.size.h, band.loc.y - gap),
            Edge::Top => assert_eq!(at.loc.y, band.loc.y + band.size.h + gap),
            Edge::Left => assert_eq!(at.loc.x, band.loc.x + band.size.w + gap),
            Edge::Right => assert_eq!(at.loc.x + at.size.w, band.loc.x - gap),
        }
        let (centre, slots) = if edge.runs_across() {
            (at.loc.x + at.size.w / 2, slot.loc.x + slot.size.w / 2)
        } else {
            (at.loc.y + at.size.h / 2, slot.loc.y + slot.size.h / 2)
        };
        assert_eq!(centre, slots, "{edge:?}: not centred on its control");
    }
}

/// **A control at the very end of the display gets a panel moved onto the
/// display**, never one hanging off it.
#[test]
fn a_panel_at_the_end_of_the_display_is_moved_onto_it() {
    let band = Rectangle::new((0, 996).into(), (700, 76).into());
    let slot = Rectangle::new((0, 1010).into(), (48, 48).into());
    let at = placed(band, slot, (200, 26), Edge::Bottom, 8, (1920, 1080));
    assert_eq!(at.loc.x, 0);
    let slot = Rectangle::new((1900, 1010).into(), (48, 48).into());
    let at = placed(band, slot, (200, 26), Edge::Bottom, 8, (1920, 1080));
    assert_eq!(at.loc.x + at.size.w, 1920);
}

/// **No name to show leaves the Dock the bytes it was**, which is what keeps
/// the Dock's own pixel hashes true: nothing pointed at, and the pointer on
/// the composer, which draws its own words.
#[test]
fn no_name_to_show_changes_nothing() {
    let before = drawn(
        Edge::Bottom,
        (1920, 1080),
        DisplayScale::ONE,
        &["Files", "Mail"],
    );
    let mut after = before.clone();
    show_the_name(
        &mut after,
        None,
        &words(),
        TextScale::default(),
        Direction::LeftToRight,
        &mut fonts(),
    );
    assert_eq!(after, before, "nothing pointed at, and the Dock changed");
    let composer = slot_of(&before, &WhatIsHere::TheComposer);
    show_the_name(
        &mut after,
        Some(composer),
        &words(),
        TextScale::default(),
        Direction::LeftToRight,
        &mut fonts(),
    );
    assert_eq!(
        after, before,
        "the composer draws its own words and got a tooltip"
    );
    show_the_name(
        &mut after,
        Some(999),
        &words(),
        TextScale::default(),
        Direction::LeftToRight,
        &mut fonts(),
    );
    assert_eq!(after, before, "a slot that is not there drew something");
}

/// **On every edge, an application's name is drawn outside the bar, on the
/// display, in the design's navy with rounded corners, one line tall.**
#[test]
fn an_applications_name_is_drawn_outside_the_bar_on_every_edge() {
    let navy = alo_appearance::Role::TextPrimary.colour();
    for edge in Edge::EVERY {
        let picture = drawn(edge, (1920, 1080), DisplayScale::ONE, &["Files", "Mail"]);
        let files = WhatIsHere::Application(AppId::named("Files").expect("named"));
        let shown = the_name_shown(
            &picture,
            slot_of(&picture, &files),
            &words(),
            TextScale::default(),
            Direction::LeftToRight,
            &mut fonts(),
        )
        .unwrap_or_else(|| panic!("{edge:?}: no name shown"));
        assert!(!overlap(shown.area, picture.band), "{edge:?}: on the bar");
        assert!(
            shown.area.loc.x >= 0
                && shown.area.loc.y >= 0
                && shown.area.loc.x + shown.area.size.w <= 1920
                && shown.area.loc.y + shown.area.size.h <= 1080,
            "{edge:?}: off the display: {:?}",
            shown.area
        );
        assert_eq!(
            shown.panel.area, shown.area,
            "{edge:?}: a tooltip has no shadow"
        );
        let one_line = i32::try_from(A_LINE_IS + 2 * ABOVE_AND_BELOW_THE_WORDS).expect("small");
        assert_eq!(shown.area.size.h, one_line, "{edge:?}: Files is one line");
        assert!(
            shown.area.size.w <= 216,
            "{edge:?}: wider than the owner's 200 for the words and 8 either side"
        );
        assert_eq!(shown.words.len(), 1, "{edge:?}");

        let at = |x: i32, y: i32| {
            let w = usize::try_from(shown.panel.area.size.w).expect("a width");
            let i = 4
                * (usize::try_from(y).expect("a row") * w + usize::try_from(x).expect("a column"));
            &shown.panel.pixels[i..i + 4]
        };
        assert_eq!(
            at(2, shown.area.size.h / 2),
            [navy.red(), navy.green(), navy.blue(), 255],
            "{edge:?}: the panel is not the design's navy"
        );
        assert!(at(0, 0)[3] < 255, "{edge:?}: the corner is not rounded");
    }
}

/// **A long name wraps to two lines and ends in an ellipsis**, and the panel
/// grows to hold both rather than overrunning.
#[test]
fn a_long_name_takes_two_lines_and_then_an_ellipsis() {
    let long =
        "An-application-whose-name-goes-on-and-on-far-past-two-hundred-pixels-of-words-and-more";
    let picture = drawn(Edge::Bottom, (1920, 1080), DisplayScale::ONE, &[long]);
    let app = WhatIsHere::Application(AppId::named(long).expect("named"));
    let shown = the_name_shown(
        &picture,
        slot_of(&picture, &app),
        &words(),
        TextScale::default(),
        Direction::LeftToRight,
        &mut fonts(),
    )
    .expect("a name shown");
    assert_eq!(shown.words.len(), 2, "not two lines");
    let two_lines = i32::try_from(2 * A_LINE_IS + 2 * ABOVE_AND_BELOW_THE_WORDS).expect("small");
    assert_eq!(shown.area.size.h, two_lines);
    assert!(shown.area.size.w <= 216);
    let (lines, cut) = crate::painted_text_wrapping::wrapped(
        long,
        200,
        &mut fonts(),
        cosmic_text::Metrics::new(11.0, 18.0),
        2,
    );
    assert!(cut, "the control: this name does not fit in two lines");
    assert_eq!(lines.len(), 2);
}

/// **At twice the display's scale, everything about the panel is twice as
/// large**: the gap from the bar, the height, and the padding — so a dense
/// screen draws the same tooltip sharper, not smaller.
#[test]
fn a_dense_display_draws_the_same_tooltip_twice_as_large() {
    let files = WhatIsHere::Application(AppId::named("Files").expect("named"));
    let shown = |size: (i32, i32), scale: DisplayScale| {
        let picture = drawn(Edge::Bottom, size, scale, &["Files"]);
        let shown = the_name_shown(
            &picture,
            slot_of(&picture, &files),
            &words(),
            TextScale::default(),
            Direction::LeftToRight,
            &mut fonts(),
        )
        .expect("a name shown");
        (picture.band, shown.area)
    };
    let (band, one) = shown((1920, 1080), DisplayScale::ONE);
    let (dense_band, two) = shown((3840, 2160), DisplayScale::of_per_cent(200));
    assert_eq!(two.size.h, 2 * one.size.h, "the height did not double");
    assert_eq!(
        dense_band.loc.y - (two.loc.y + two.size.h),
        2 * (band.loc.y - (one.loc.y + one.size.h)),
        "the gap from the bar did not double"
    );
    assert!(
        (two.size.w - 2 * one.size.w).abs() <= 2,
        "the width did not double: {} against {}",
        two.size.w,
        one.size.w
    );
}

/// **A larger text size makes a taller name**, because a name is a sentence.
#[test]
fn the_persons_text_size_reaches_the_name() {
    let files = WhatIsHere::Application(AppId::named("Files").expect("named"));
    let picture = drawn(Edge::Bottom, (1920, 1080), DisplayScale::ONE, &["Files"]);
    let at = |text: TextScale| {
        the_name_shown(
            &picture,
            slot_of(&picture, &files),
            &words(),
            text,
            Direction::LeftToRight,
            &mut fonts(),
        )
        .expect("a name shown")
        .area
        .size
        .h
    };
    let ordinary = at(TextScale::default());
    let double = at(TextScale::percent(200).expect("200 per cent is a text size"));
    assert_eq!(
        double - 2 * i32::try_from(ABOVE_AND_BELOW_THE_WORDS).expect("small"),
        2 * (ordinary - 2 * i32::try_from(ABOVE_AND_BELOW_THE_WORDS).expect("small"))
    );
}

/// **The pointer names the control a click would press**, and a pointer off
/// the Dock names nothing.
#[test]
fn the_pointer_names_the_control_a_click_would_press() {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind_keyboard(
        runtime.path(),
        "the-pointer-names-a-control",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..smithay::input::keyboard::XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    let picture = drawn(
        Edge::Bottom,
        (1920, 1080),
        DisplayScale::ONE,
        &["Files", "Mail"],
    );
    let mail = WhatIsHere::Application(AppId::named("Mail").expect("named"));
    let which = slot_of(&picture, &mail);
    let place = crate::the_bars_pressable_slots::the_one_at(&picture.standing.places, which)
        .expect("the slot");
    let x = picture.band.loc.x
        + i32::try_from(place.from_the_start() + place.across() / 2).expect("small");
    let y = picture.band.loc.y + picture.band.size.h / 2;
    server.the_docks_slots_were_drawn("a-display", picture.standing.clone());

    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    assert_eq!(server.the_slot_the_pointer_is_on("a-display"), Some(which));
    assert_eq!(
        crate::a_press_on_the_dock::what_a_press_on_the_dock_does(&picture.standing, (x, y).into()),
        crate::a_press_on_the_dock::WhatAPressOnTheDockDoes::GoTo(
            AppId::named("Mail").expect("named")
        ),
        "the control: a click here would open Mail"
    );
    assert_eq!(
        server.the_slot_the_pointer_is_on("another-display"),
        None,
        "a Dock on another display was named"
    );
    server
        .pointer_motion(10.0, 10.0, 2)
        .expect("the pointer moved");
    assert_eq!(server.the_slot_the_pointer_is_on("a-display"), None);
}

/// **History's word is its tooltip on a side Dock, where it is drawn as an
/// icon, and nowhere else**, where it is drawn as the word.
#[test]
fn historys_word_is_its_tooltip_only_where_it_is_an_icon() {
    let words = words();
    for edge in Edge::EVERY {
        let side = matches!(edge, Edge::Left | Edge::Right);
        assert_eq!(
            what_it_says(&WhatIsHere::History, edge, &words),
            side.then_some("History"),
            "{edge:?}"
        );
    }
    let picture = drawn(Edge::Left, (1920, 1080), DisplayScale::ONE, &["Files"]);
    let shown = the_name_shown(
        &picture,
        slot_of(&picture, &WhatIsHere::History),
        &words,
        TextScale::default(),
        Direction::LeftToRight,
        &mut fonts(),
    )
    .expect("History's name is shown on a side Dock");
    assert!(!overlap(shown.area, picture.band), "on the bar");
}

/// **In a right-to-left language each line starts from the panel's right**,
/// eight in from its edge as it is eight in from the left otherwise, and a
/// shorter line is flush right under a longer one. Where the panel sits does
/// not change: it is centred on the control either way.
#[test]
fn in_a_right_to_left_language_the_lines_start_from_the_right() {
    // Two lines, so the shorter second one shows which edge lines align to.
    // Latin words, because the bundled faces carry no Hebrew or Arabic: what
    // is measured is where a line is placed, which the script does not change.
    let name = "Correspondence for the whole household Mail";
    let picture = drawn(Edge::Bottom, (1920, 1080), DisplayScale::ONE, &[name]);
    let app = WhatIsHere::Application(AppId::named(name).expect("named"));
    let shown_in = |reading: Direction| {
        the_name_shown(
            &picture,
            slot_of(&picture, &app),
            &words(),
            TextScale::default(),
            reading,
            &mut fonts(),
        )
        .expect("a name shown")
    };
    let ltr = shown_in(Direction::LeftToRight);
    let rtl = shown_in(Direction::RightToLeft);
    assert_eq!(
        ltr.area, rtl.area,
        "the panel moved with the reading direction"
    );
    assert_eq!(rtl.words.len(), 2, "the premise: two lines");
    let beside = i32::try_from(super::BESIDE_THE_WORDS).expect("small");
    let right = rtl.area.loc.x + rtl.area.size.w - beside;
    for line in &rtl.words {
        assert_eq!(
            line.area.loc.x + line.area.size.w,
            right,
            "a line does not end eight in from the panel's right"
        );
    }
    for line in &ltr.words {
        assert_eq!(
            line.area.loc.x,
            ltr.area.loc.x + beside,
            "a line does not start eight in from the panel's left"
        );
    }
    let widths: Vec<i32> = rtl.words.iter().map(|line| line.area.size.w).collect();
    assert!(
        widths[1] < widths[0],
        "the premise: the second line is the shorter: {widths:?}"
    );
    assert!(
        rtl.words[1].area.loc.x > rtl.area.loc.x + beside,
        "the shorter line starts from the left in a right-to-left language"
    );
}
