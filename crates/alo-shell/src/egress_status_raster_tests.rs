//! The laid-out indicator, read as pixels and rows: what is on it, where, and
//! what never is.
//!
//! Since 2026-10-31 it is a quiet circle or a column of pills in the top-left
//! corner (the owner's decisions of that day), so these read its rasters as
//! well as its rows.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::EgressStatus;
use crate::egress_status_testing::{
    a_provider, asking_a_provider, fetching, noon, the_agent, the_machine_down_the_corridor, words,
};
use alo_egress::{Destination, EgressPolicy, Errand, Indicator, Leaving, OnItsOwn, Why};
use alo_indicator::{Drew, Indicating};
use alo_models::InferenceSource;
use alo_strings::Vocabulary;
use smithay::utils::{Physical, Rectangle};

/// The ordinary light look, read left to right.
fn light() -> EgressStatusLook {
    EgressStatusLook {
        contrast: Contrast::AsDesigned,
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
    }
}

/// What the machine's indicator hands a compositor right now, handed over the
/// way a session hands it: through `Indicating` to an `EgressStatus`.
fn handed(indicator: &Indicator) -> Drawn {
    let mut status = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut status), indicator),
        Drew::Shown
    );
    status.showing().unwrap().clone()
}

/// The indicator drawn on an output of `room`, with the dock where it ships.
fn drawn_in(indicator: &Indicator, look: EgressStatusLook, room: TheRoom) -> EgressStatusPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        &handed(indicator),
        &words(),
        &Dock::shipped(),
        &mut labels,
        room,
        look,
    )
    .unwrap()
}

/// The indicator drawn with the machine's vocabulary on a 1920×1080 output.
fn drawn_as(indicator: &Indicator, look: EgressStatusLook) -> EgressStatusPicture {
    drawn_in(
        indicator,
        look,
        TheRoom::with_nothing_put_aside((1920, 1080)),
    )
}

/// An indicator with one question to a provider under way.
fn asking() -> Indicator {
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    indicator
}

/// The colour drawn at `(x, y)` on the screen in the picture's rasters, the
/// last raster holding the point winning, as the painter paints them.
fn raster_at(drawn: &EgressStatusPicture, x: i32, y: i32) -> Option<[u8; 3]> {
    drawn.washed.iter().rev().find_map(|washed| {
        let (left, top) = (x - washed.area.loc.x, y - washed.area.loc.y);
        if left < 0 || top < 0 || left >= washed.area.size.w || top >= washed.area.size.h {
            return None;
        }
        let at = usize::try_from((top * washed.area.size.w + left) * 4).ok()?;
        <[u8; 3]>::try_from(washed.pixels.get(at..at + 3)?).ok()
    })
}

/// The middle of a rectangle.
const fn middle_of(area: Rectangle<i32, Physical>) -> (i32, i32) {
    (area.loc.x + area.size.w / 2, area.loc.y + area.size.h / 2)
}

/// **Nothing leaving draws the quiet circle; something answered elsewhere
/// draws its sentence in a pill.** A local answer — the runtime alo OS ships,
/// or a service at this machine's own address — is not a departure and leaves
/// the circle quiet, while a provider and the machine down the corridor each
/// draw the line `alo-egress` wrote for them.
#[test]
fn a_question_answered_here_leaves_it_quiet_and_one_answered_elsewhere_draws_its_sentence() {
    let strings = words();
    let quiet = drawn_as(&Indicator::default(), light());
    assert!(quiet.rows.is_empty(), "a quiet indicator drew a line");
    assert_eq!(quiet.washed.len(), 1, "the quiet circle is not drawn");

    for here in [
        InferenceSource::ThisMachine,
        InferenceSource::AServiceAtThisMachinesAddress,
    ] {
        let indicator = Indicator::default();
        assert!(Leaving::asking(&the_agent(), &here).is_err(), "{here:?}");
        assert_eq!(drawn_as(&indicator, light()), quiet, "{here:?}");
    }

    for elsewhere in [a_provider(), the_machine_down_the_corridor()] {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(
                &EgressPolicy::Anywhere,
                Leaving::asking(&the_agent(), &elsewhere).unwrap(),
                noon(),
            )
            .unwrap();
        let drawn = drawn_as(&indicator, light());
        assert_eq!(drawn.rows.len(), 1, "{elsewhere:?}");
        let expected = indicator.showing().first().unwrap().said(&strings);
        assert!(!expected.is_a_bug(), "{expected}");
        assert_eq!(drawn.rows.first().unwrap().sentence, expected.text());
        assert!(!drawn.inked.is_empty());

        // And when the answer is back, it is quiet again.
        assert!(indicator.ended(departing));
        assert_eq!(drawn_as(&indicator, light()), quiet);
    }
    assert_eq!(
        drawn_as(&asking(), light()).rows.first().unwrap().sentence,
        "@mail is asking a question of alo, in the EU"
    );
}

/// **The quiet circle is the design's**: 36 across, 20 from the left and 14
/// from the top, the surface's ground, with the grey arrow in its middle.
#[test]
fn the_quiet_circle_is_where_and_what_the_design_draws() {
    let quiet = drawn_as(&Indicator::default(), light());
    assert_eq!(
        quiet.band,
        Some(Rectangle::new((20, 14).into(), (36, 36).into()))
    );
    let palette = palette(Scheme::Light, Contrast::AsDesigned);
    let circle = quiet.band.unwrap();
    // Just inside the left of the circle, on its ground.
    assert_eq!(
        raster_at(&quiet, circle.loc.x + 6, circle.loc.y + 18),
        Some(palette.ground)
    );
    // Its middle carries the arrow's ink.
    let (x, y) = middle_of(circle);
    assert_ne!(raster_at(&quiet, x, y), Some(palette.ground), "no arrow");
}

/// **The band is what was drawn, not the room it could have grown into**:
/// every pill is inside it, and it starts where the first one does.
#[test]
fn the_band_holds_every_pill_and_starts_where_they_do() {
    let mut indicator = asking();
    drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
    let drawn = drawn_as(&indicator, light());
    let band = drawn.band.unwrap();
    for row in &drawn.rows {
        assert_eq!(band.intersection(row.area), Some(row.area), "{row:?}");
    }
    let first = drawn.rows.first().unwrap().area;
    assert_eq!((band.loc.x, band.loc.y), (first.loc.x, first.loc.y));
}

/// **An egress a policy refused leaves it quiet**: nothing left, and a surface
/// that drew what a rule stopped would teach a person to ignore it.
#[test]
fn an_egress_the_policy_refused_leaves_it_quiet() {
    let mut indicator = Indicator::default();
    assert!(
        indicator
            .beginning(&EgressPolicy::NothingLeaves, asking_a_provider(), noon())
            .is_err()
    );
    let drawn = drawn_as(&indicator, light());
    assert!(drawn.rows.is_empty());
    assert_eq!(drawn, drawn_as(&Indicator::default(), light()));
}

/// **What alo OS does on its own errand is on the same indicator**, in a pill
/// of its own beside what an agent caused.
#[test]
fn what_alo_os_does_on_its_own_is_drawn_beside_what_an_agent_caused() {
    let strings = words();
    let mut indicator = Indicator::default();
    let underway = indicator.beginning_on_its_own(
        OnItsOwn::for_(
            Errand::CheckingForAnUpdate,
            Destination::at("updates.alo.example").unwrap(),
        ),
        noon(),
    );
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, fetching(), noon())
        .unwrap();

    let drawn = drawn_as(&indicator, light());
    let said: Vec<String> = indicator
        .showing()
        .iter()
        .map(|line| line.said(&strings).into_text())
        .collect();
    let sentences: Vec<&str> = drawn.rows.iter().map(|row| row.sentence.as_str()).collect();
    assert_eq!(sentences, said);
    assert!(
        sentences.first().unwrap().starts_with("alo OS"),
        "{sentences:?}"
    );
    assert_eq!(drawn.washed.len(), 2, "each line is a pill of its own");

    assert!(indicator.ended(departing));
    assert!(indicator.ended_on_its_own(underway));
    assert!(drawn_as(&indicator, light()).rows.is_empty());
}

/// **Every line is drawn exactly as `alo-egress` words it**: for every reason
/// an agent causes egress, every kind of destination and every errand, the
/// pill holds the line's own sentence and its words are that sentence shaped in
/// `Label/Medium` and nothing else.
#[test]
fn every_line_is_drawn_exactly_as_alo_egress_words_it_and_is_collected() {
    let strings = words();
    let mut indicator = Indicator::default();
    let destinations = [
        Destination::at("alo.example").unwrap(),
        Destination::paired("workstation-2").unwrap(),
        Destination::of(&a_provider()).unwrap(),
    ];
    for why in [Why::Asking, Why::Fetching, Why::Sending] {
        for destination in &destinations {
            drop(indicator.beginning(
                &EgressPolicy::Anywhere,
                Leaving::because(&the_agent(), why, destination.clone()),
                noon(),
            ));
        }
    }
    for errand in Errand::EVERY {
        drop(indicator.beginning_on_its_own(
            OnItsOwn::for_(errand, Destination::at("alo.example").unwrap()),
            noon(),
        ));
    }
    let lines = indicator.showing().len();
    assert_eq!(lines, 3 * destinations.len() + Errand::EVERY.len());

    // A screen tall enough for all of them, so none gives way to the count.
    let mut labels = WindowControlLabels::new().unwrap();
    let drawn = picture(
        &handed(&indicator),
        &strings,
        &Dock::shipped(),
        &mut labels,
        TheRoom::with_nothing_put_aside((3840, 2160)),
        light(),
    )
    .unwrap();
    assert_eq!(drawn.rows.len(), lines);
    assert_eq!(drawn.inked.len(), lines);

    let palette = palette(Scheme::Light, Contrast::AsDesigned);
    let metrics = cosmic_text::Metrics::new(13.0, 18.0);
    for ((row, line), inked) in drawn.rows.iter().zip(indicator.showing()).zip(&drawn.inked) {
        let said = line.said(&strings);
        assert!(!said.is_a_bug(), "not collected: {said}");
        assert_eq!(row.sentence, said.text());
        let alone = crate::painted_text::sentence(
            &mut labels.fonts,
            said.text(),
            row.words.size.w,
            metrics,
            palette.ground,
            palette.ink,
        );
        assert_eq!(
            inked.pixels, alone.pixels,
            "{:?} is not drawn as its own sentence",
            row.sentence
        );
    }
}

/// **A vocabulary that forgot the lines still lights the indicator.**
#[test]
fn a_vocabulary_without_the_lines_still_draws_them() {
    let mut labels = WindowControlLabels::new().unwrap();
    let drawn = picture(
        &handed(&asking()),
        &Strings::of(Vocabulary::empty()),
        &Dock::shipped(),
        &mut labels,
        TheRoom::with_nothing_put_aside((1920, 1080)),
        light(),
    )
    .unwrap();
    assert_eq!(drawn.rows.len(), 1);
    assert!(!drawn.rows.first().unwrap().sentence.is_empty());
}

/// **Deep teal never arrives alone.** In both schemes and at every text size,
/// each pill's lamp is deep teal at its edge with a light arrow in its middle,
/// the pill carries its words inked in the ink colour, and no word is inked in
/// deep teal.
#[test]
fn deep_teal_never_arrives_alone() {
    let (_, largest) = TextScale::range();
    let mut indicator = asking();
    drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [100, 200, largest] {
            let look = EgressStatusLook {
                contrast: Contrast::AsDesigned,
                scheme,
                scale: TextScale::percent(percent).unwrap(),
                reading: Direction::LeftToRight,
            };
            let drawn = drawn_as(&indicator, look);
            let palette = palette(scheme, Contrast::AsDesigned);
            assert!(!drawn.rows.is_empty(), "{look:?}");
            for row in &drawn.rows {
                let (x, y) = middle_of(row.mark);
                // A third of the way in from the lamp's left, on its fill and
                // off the arrow.
                let on_the_fill = raster_at(&drawn, row.mark.loc.x + row.mark.size.w / 6, y);
                assert_eq!(
                    on_the_fill,
                    Some(deep_teal()),
                    "{look:?}: the lamp is not deep teal"
                );
                assert_ne!(
                    raster_at(&drawn, x, y),
                    Some(deep_teal()),
                    "{look:?}: a lamp with no arrow"
                );
                let words = drawn
                    .inked
                    .iter()
                    .find(|inked| row.area.intersection(inked.area) == Some(inked.area))
                    .unwrap();
                assert!(words.pixels.contains(&palette.ink), "{look:?}: no ink");
            }
            assert!(
                drawn
                    .inked
                    .iter()
                    .all(|inked| !inked.pixels.contains(&deep_teal())),
                "{look:?}: words inked in deep teal"
            );
        }
    }
}

/// **It sits in the top-left corner**, 20 from the left and a pill 10 from the
/// top — in both reading directions, because it is a corner of the screen and
/// not the end of a row.
#[test]
fn it_is_drawn_in_the_top_left_corner_whichever_way_a_person_reads() {
    for reading in [Direction::LeftToRight, Direction::RightToLeft] {
        let drawn = drawn_as(&asking(), EgressStatusLook { reading, ..light() });
        let pill = drawn.rows.first().unwrap().area;
        assert_eq!((pill.loc.x, pill.loc.y), (20, 10), "{reading:?}");
        assert_eq!(pill.size.h, 40, "{reading:?}");
    }
}

/// **A Dock along the top is cleared, and one down the left is stepped over.**
#[test]
fn a_dock_on_the_top_or_the_left_is_cleared() {
    use alo_dock::{Edge, Layout, Screen};
    let room = TheRoom::with_nothing_put_aside((1920, 1080));
    let on = |edge| Layout::along(edge, Screen::of(1920, 1080).unwrap());
    let floating = i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap();

    let bottom = Place::in_the_top_left_corner(on(Edge::Bottom), room, 20);
    assert_eq!(bottom.across, Across::FromLeft(20));
    assert_eq!(bottom.stacked, Stacked::Downwards(0));

    let top = on(Edge::Top);
    let thick = i32::try_from(top.thickness().as_pixels()).unwrap();
    assert_eq!(
        Place::in_the_top_left_corner(top, room, 20).stacked,
        Stacked::Downwards(thick + floating)
    );

    let left = on(Edge::Left);
    let lane = i32::try_from(left.thickness().as_pixels()).unwrap() + floating;
    assert_eq!(
        Place::in_the_top_left_corner(left, room, 20).across,
        Across::FromLeft(lane + 20)
    );
}

/// **The lamp is on the side a person reads first**: before the words for a
/// left-to-right reader and after them for a right-to-left one.
#[test]
fn the_lamp_is_on_the_side_a_person_reads_first() {
    let forwards = drawn_as(&asking(), light());
    let row = forwards.rows.first().unwrap();
    assert!(row.mark.loc.x + row.mark.size.w <= row.words.loc.x);

    let backwards = drawn_as(
        &asking(),
        EgressStatusLook {
            reading: Direction::RightToLeft,
            ..light()
        },
    );
    let row = backwards.rows.first().unwrap();
    assert!(row.words.loc.x + row.words.size.w <= row.mark.loc.x);
}

/// **No look turns it off**: in every scheme, at every text size and in both
/// reading directions, something leaving is drawn, and everything drawn is
/// inside the output.
#[test]
fn no_look_draws_a_lit_indicator_as_nothing() {
    let (smallest, largest) = TextScale::range();
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
    let output = Rectangle::<i32, Physical>::from_size((1366, 768).into());
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [smallest, 100, 200, largest] {
            for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                let look = EgressStatusLook {
                    contrast: Contrast::AsDesigned,
                    scheme,
                    scale: TextScale::percent(percent).unwrap(),
                    reading,
                };
                let drawn = drawn_in(
                    &indicator,
                    look,
                    TheRoom::with_nothing_put_aside((1366, 768)),
                );
                assert_eq!(drawn.rows.len(), 1, "{look:?}");
                assert!(!drawn.inked.is_empty(), "{look:?}");
                for area in drawn
                    .washed
                    .iter()
                    .map(|washed| washed.area)
                    .chain(drawn.inked.iter().map(|inked| inked.area))
                {
                    assert_eq!(area.intersection(output), Some(area), "{look:?}: {area:?}");
                }
            }
        }
    }
}

/// **Lines that do not all fit are never dropped silently**: as many as fit
/// are drawn in order, and the last pill carries the indicator's own count.
#[test]
fn lines_that_do_not_fit_end_with_the_indicators_own_count() {
    let strings = words();
    let (_, largest) = TextScale::range();
    let mut indicator = Indicator::default();
    for _ in 0..40 {
        drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    }
    let look = EgressStatusLook {
        scale: TextScale::percent(largest).unwrap(),
        ..light()
    };
    let handed_over = handed(&indicator);
    let mut labels = WindowControlLabels::new().unwrap();
    let drawn = picture(
        &handed_over,
        &strings,
        &Dock::shipped(),
        &mut labels,
        TheRoom::with_nothing_put_aside((1366, 768)),
        look,
    )
    .unwrap();
    assert!(drawn.rows.len() < 40, "{} rows fit", drawn.rows.len());
    let last = drawn.rows.last().unwrap();
    let count = handed_over.lamp_said(&strings);
    assert!(!count.is_a_bug(), "{count}");
    assert_eq!(last.sentence, count.text());
    assert!(last.sentence.contains("40"), "{}", last.sentence);
    for row in drawn.rows.iter().take(drawn.rows.len() - 1) {
        assert_eq!(row.sentence, "@mail is asking a question of alo, in the EU");
    }
}

/// **An output that cannot hold a dock refuses a lit frame** rather than
/// drawing it without the indicator — and a quiet one is not refused anywhere.
#[test]
fn an_output_that_cannot_hold_a_dock_refuses_a_lit_frame_and_not_a_quiet_one() {
    let mut labels = WindowControlLabels::new().unwrap();
    let quiet = handed(&Indicator::default());
    let lit = handed(&asking());
    for size in [(100, 100), (0, 0), (-5, 800), (1920, -1), (20_000, 1080)] {
        let room = TheRoom::with_nothing_put_aside(size);
        let refused = picture(&lit, &words(), &Dock::shipped(), &mut labels, room, light());
        assert!(
            matches!(refused, Err(RenderError::EgressStatusScene)),
            "{size:?}"
        );
        assert!(
            picture(
                &quiet,
                &words(),
                &Dock::shipped(),
                &mut labels,
                room,
                light()
            )
            .is_ok(),
            "{size:?}"
        );
    }
}

/// The words contrast with the pill's ground as text must, in both schemes.
#[test]
fn the_words_are_readable_on_their_ground() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let palette = palette(scheme, Contrast::AsDesigned);
        let colour = |[red, green, blue]: [u8; 3]| alo_appearance::Colour::of(red, green, blue);
        let contrast = colour(palette.ink).contrast_with(colour(palette.ground));
        assert!(
            contrast >= alo_appearance::ENOUGH_FOR_TEXT,
            "{scheme:?}: {contrast:.2}"
        );
    }
}

/// **High contrast draws the lamp in the other palette's own deep teal**, the
/// reserved colour taken deep enough to read (ADR 0010), and never the
/// design's.
#[test]
fn high_contrast_draws_the_lamp_in_the_other_palette() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let high = drawn_as(
            &asking(),
            EgressStatusLook {
                scheme,
                contrast: Contrast::High,
                ..light()
            },
        );
        let row = high.rows.first().unwrap();
        let (_, y) = middle_of(row.mark);
        let on_the_fill = raster_at(&high, row.mark.loc.x + row.mark.size.w / 6, y);
        let agents = crate::egress_status_mark::rgb(alo_access::HighContrast::of(scheme).accent);
        assert_eq!(on_the_fill, Some(agents), "{scheme:?}");
        assert_ne!(on_the_fill, Some(deep_teal()), "{scheme:?}");
    }
}

/// **On a two-times output the pills are twice the size and twice as far in.**
#[test]
fn at_two_hundred_the_pills_double() {
    let at = |size: (i32, i32), per_cent: u16| {
        drawn_in(
            &asking(),
            light(),
            TheRoom {
                size,
                panel: None,
                scale: crate::display_scale::DisplayScale::of_per_cent(per_cent),
            },
        )
    };
    let (one, two) = (at((1920, 1080), 100), at((3840, 2160), 200));
    let (a, b) = (
        one.rows.first().unwrap().area,
        two.rows.first().unwrap().area,
    );
    assert_eq!(b.size.h, 2 * a.size.h, "the pill's height");
    assert_eq!(
        (b.loc.x, b.loc.y),
        (2 * a.loc.x, 2 * a.loc.y),
        "where it sits"
    );
}

/// **The indicator never overlaps the put-aside panel's column or the top
/// controls' band** — at every display size and scale, quiet, lit with one
/// line and lit with many, and with the panel showing or not. The owner's
/// decisions of 2026-10-31: the indicator owns the top-left corner, the top
/// controls start just to its right, and the panel keeps the top-right.
#[test]
fn it_never_overlaps_the_panels_column_or_the_top_controls() {
    let mut many = Indicator::default();
    for _ in 0..12 {
        drop(many.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
    }
    for (width, height) in [
        (1024, 600),
        (1280, 800),
        (1366, 768),
        (1920, 1080),
        (3840, 2160),
    ] {
        for per_cent in [100, 150, 200] {
            for panel_is in [0, 112, 300] {
                let column = Rectangle::<i32, Physical>::new(
                    (width - panel_is, 0).into(),
                    (panel_is, height).into(),
                );
                let room = TheRoom {
                    size: (width, height),
                    panel: Some(column),
                    scale: crate::display_scale::DisplayScale::of_per_cent(per_cent),
                };
                for indicator in [&Indicator::default(), &asking(), &many] {
                    let drawn = drawn_in(indicator, light(), room);
                    let band = drawn.band.unwrap();
                    let at = (width, height, per_cent, panel_is);
                    assert!(
                        column.size.w == 0 || !band.overlaps(column),
                        "{at:?}: the indicator {band:?} is on the panel's column {column:?}"
                    );
                    let top_controls = crate::top_controls_region::after_the_indicator(
                        crate::top_controls_region::reserved((width, height), column, false),
                        drawn.band,
                    );
                    if let Some(top_controls) = top_controls {
                        assert!(
                            !band.overlaps(top_controls),
                            "{at:?}: the indicator {band:?} and the top controls \
                             {top_controls:?} claim one point"
                        );
                        assert!(
                            !top_controls.overlaps(column) || column.size.w == 0,
                            "{at:?}: the top controls reach the panel's column"
                        );
                    }
                }
            }
        }
    }
}
