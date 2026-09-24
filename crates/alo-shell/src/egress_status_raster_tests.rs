//! The laid-out indicator, read as pixels and rows: what is on it, where, and
//! what never is.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::egress_status_mark::rgb;
use crate::egress_status_testing::{
    a_provider, asking_a_provider, fetching, noon, the_agent, the_machine_down_the_corridor, words,
};
use crate::painted_text::sentence as shaped_sentence;
use crate::{EgressStatus, painted::Solid};
use alo_appearance::Token;
use alo_dock::Edge;
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

/// The indicator drawn with the machine's vocabulary on a 1920×1080 output,
/// with the dock where it ships.
fn drawn_as(indicator: &Indicator, look: EgressStatusLook) -> EgressStatusPicture {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        &handed(indicator),
        &words(),
        &Dock::shipped(),
        &mut labels,
        (1920, 1080),
        look,
        0,
    )
    .unwrap()
}

/// Terracotta, as the painter takes it.
fn terracotta() -> [u8; 3] {
    rgb(Token::Terracotta.colour())
}

/// **A question answered on this machine draws nothing; one answered anywhere
/// else draws its sentence.** Both halves, because the promise rests on the
/// difference: a local answer — the runtime alo OS ships, or a service the
/// person runs at this machine's own address — is not a departure and puts
/// not one pixel on the screen, while a provider and the machine down the
/// corridor each draw the line `alo-egress` wrote for them.
#[test]
fn a_question_answered_here_draws_nothing_and_one_answered_elsewhere_draws_its_sentence() {
    let strings = words();
    let nothing_at_all = drawn_as(&Indicator::default(), light());
    assert!(nothing_at_all.is_empty());

    for here in [
        InferenceSource::ThisMachine,
        InferenceSource::AServiceAtThisMachinesAddress,
    ] {
        let indicator = Indicator::default();
        // Asked here, there is no departure to begin: `alo-egress` refuses to
        // make one, so nothing reaches the indicator.
        assert!(Leaving::asking(&the_agent(), &here).is_err(), "{here:?}");
        let drawn = drawn_as(&indicator, light());
        assert!(drawn.is_empty(), "{here:?} drew {drawn:?}");
        assert_eq!(drawn, nothing_at_all);
        assert!(indicator.showing().is_empty());
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
        assert!(
            drawn
                .solids
                .iter()
                .any(|solid| solid.colour == terracotta())
        );
        assert!(!drawn.inked.is_empty());

        // And when the answer is back, the screen is empty again.
        assert!(indicator.ended(departing));
        assert!(drawn_as(&indicator, light()).is_empty());
    }
    let mut provider = Indicator::default();
    drop(provider.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    assert_eq!(
        drawn_as(&provider, light()).rows.first().unwrap().sentence,
        "@mail is asking a question of alo, in the EU"
    );
}

/// **An egress a policy refused draws nothing**: nothing left, and a surface
/// that drew what a rule stopped would teach a person to ignore it.
#[test]
fn an_egress_the_policy_refused_draws_nothing() {
    let mut indicator = Indicator::default();
    assert!(
        indicator
            .beginning(&EgressPolicy::NothingLeaves, asking_a_provider(), noon())
            .is_err()
    );
    assert!(drawn_as(&indicator, light()).is_empty());
}

/// **What alo OS does on its own errand is on the same indicator**, drawn the
/// same way as what an agent caused — a mark and its own sentence — because
/// *no telemetry* is only checkable by somebody who can see it.
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
    assert_eq!(drawn.rows.len(), 2);
    let said: Vec<String> = indicator
        .showing()
        .iter()
        .map(|line| line.said(&strings).into_text())
        .collect();
    let sentences: Vec<&str> = drawn.rows.iter().map(|row| row.sentence.as_str()).collect();
    assert_eq!(sentences, said);
    assert!(
        drawn.rows.first().unwrap().sentence.starts_with("alo OS"),
        "{sentences:?}"
    );
    for row in &drawn.rows {
        assert!(
            drawn.solids.iter().any(|solid| solid.colour == terracotta()
                && row.mark.intersection(solid.area) == Some(solid.area)),
            "{row:?} has no mark"
        );
    }

    assert!(indicator.ended(departing));
    assert!(indicator.ended_on_its_own(underway));
    assert!(drawn_as(&indicator, light()).is_empty());
}

/// **Every line is drawn exactly as `alo-egress` words it, and every one is in
/// the machine's vocabulary**: for every reason an agent causes egress, every
/// kind of destination, and every errand, the row holds the line's own
/// sentence and its pixels are that sentence shaped and nothing else.
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
        (3840, 2160),
        light(),
        0,
    )
    .unwrap();
    assert_eq!(drawn.rows.len(), lines);
    assert_eq!(drawn.inked.len(), lines);

    let palette = crate::egress_status_raster::palette(Scheme::Light, Contrast::AsDesigned);
    let metrics = Measure { percent: 100 }.metrics();
    for ((row, line), inked) in drawn.rows.iter().zip(indicator.showing()).zip(&drawn.inked) {
        let said = line.said(&strings);
        assert!(!said.is_a_bug(), "not collected: {said}");
        assert_eq!(row.sentence, said.text());
        let alone = shaped_sentence(
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

/// **A vocabulary that forgot the lines still lights the indicator.** The
/// sentence is then the key, marked as a bug by `alo-strings` — ugly, and far
/// better than a machine that went quiet because a word was missing.
#[test]
fn a_vocabulary_without_the_lines_still_draws_them() {
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let mut labels = WindowControlLabels::new().unwrap();
    let drawn = picture(
        &handed(&indicator),
        &Strings::of(Vocabulary::empty()),
        &Dock::shipped(),
        &mut labels,
        (1920, 1080),
        light(),
        0,
    )
    .unwrap();
    assert_eq!(drawn.rows.len(), 1);
    assert!(!drawn.rows.first().unwrap().sentence.is_empty());
    assert!(
        drawn
            .solids
            .iter()
            .any(|solid| solid.colour == terracotta())
    );
}

/// **Terracotta never arrives alone.** In both schemes and at every text size,
/// every terracotta pixel is inside a row's mark, that mark has an arrow drawn
/// in it in a colour that is not terracotta, and the same row has words inked
/// in the ink colour. No word is ever inked in terracotta.
#[test]
fn terracotta_never_arrives_alone() {
    let (_, largest) = TextScale::range();
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
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
            let palette = crate::egress_status_raster::palette(scheme, Contrast::AsDesigned);
            assert!(!drawn.rows.is_empty());
            let terracottas: Vec<&Solid> = drawn
                .solids
                .iter()
                .filter(|solid| solid.colour == terracotta())
                .collect();
            assert_eq!(terracottas.len(), drawn.rows.len(), "{look:?}");
            for fill in terracottas {
                let row = drawn
                    .rows
                    .iter()
                    .find(|row| row.mark.intersection(fill.area) == Some(fill.area))
                    .unwrap();
                let arrow = drawn.solids.iter().filter(|solid| {
                    solid.colour != terracotta()
                        && fill.area.intersection(solid.area) == Some(solid.area)
                        && solid.area != fill.area
                });
                assert!(arrow.count() >= 2, "{look:?}: a mark with no arrow");
                let words = drawn
                    .inked
                    .iter()
                    .find(|inked| row.area.intersection(inked.area) == Some(inked.area))
                    .unwrap();
                assert!(
                    words.pixels.contains(&palette.ink),
                    "{look:?}: words with no ink"
                );
            }
            assert!(
                drawn
                    .inked
                    .iter()
                    .all(|inked| !inked.pixels.contains(&terracotta())),
                "{look:?}: words inked in terracotta"
            );
        }
    }
}

/// **It sits at the far end of the dock wherever the dock is**: on all four
/// edges and in both reading directions, the first row is against the corner
/// `alo-dock` calls the status area and never over the dock.
#[test]
fn it_is_drawn_at_the_far_end_of_the_dock_wherever_the_dock_is() {
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let drawn_on = indicator;
    let (width, height, margin) = (1920, 1080, 8);
    for reading in [Direction::LeftToRight, Direction::RightToLeft] {
        for edge in Edge::ALL {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let thick = i32::try_from(
                dock.layout_on(
                    Screen::of(1920, 1080).unwrap(),
                    TextScale::ordinary(),
                    reading,
                )
                .thickness()
                .as_pixels(),
            )
            .unwrap();
            let mut labels = WindowControlLabels::new().unwrap();
            let look = EgressStatusLook { reading, ..light() };
            let drawn = picture(
                &handed(&drawn_on),
                &words(),
                &dock,
                &mut labels,
                (width, height),
                look,
                0,
            )
            .unwrap();
            let row = drawn.rows.first().unwrap().area;
            let (left, top) = (row.loc.x, row.loc.y);
            let (right, bottom) = (left + row.size.w, top + row.size.h);
            let at = (edge, reading);
            match edge {
                Edge::Bottom => {
                    assert_eq!(bottom, height - thick - margin, "{at:?}");
                    assert!(bottom <= height - thick, "{at:?} is over the dock");
                }
                Edge::Top => {
                    assert_eq!(top, thick + margin, "{at:?}");
                }
                Edge::Left => {
                    assert_eq!(left, thick + margin, "{at:?}");
                    assert_eq!(bottom, height - margin, "{at:?}");
                }
                Edge::Right => {
                    assert_eq!(right, width - thick - margin, "{at:?}");
                    assert_eq!(bottom, height - margin, "{at:?}");
                }
            }
            if matches!(edge, Edge::Bottom | Edge::Top) {
                match reading {
                    Direction::LeftToRight => assert_eq!(right, width - margin, "{at:?}"),
                    Direction::RightToLeft => assert_eq!(left, margin, "{at:?}"),
                }
            }
        }
    }
}

/// **The mark is on the side a person reads first**: before the words for a
/// left-to-right reader and after them for a right-to-left one.
#[test]
fn the_mark_is_on_the_side_a_person_reads_first() {
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let forwards = drawn_as(&indicator, light());
    let row = forwards.rows.first().unwrap();
    assert!(row.mark.loc.x + row.mark.size.w <= row.words.loc.x);

    let backwards = drawn_as(
        &indicator,
        EgressStatusLook {
            contrast: Contrast::AsDesigned,
            reading: Direction::RightToLeft,
            ..light()
        },
    );
    let row = backwards.rows.first().unwrap();
    assert!(row.words.loc.x + row.words.size.w <= row.mark.loc.x);
}

/// **No look turns it off**: in every scheme, at every text size, on every
/// edge, something leaving is drawn, and everything drawn is inside the
/// output and whole.
#[test]
fn no_look_and_no_dock_draws_a_lit_indicator_as_nothing() {
    let (smallest, largest) = TextScale::range();
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
    let drawn_on = handed(&indicator);
    let output = Rectangle::<i32, Physical>::from_size((1366, 768).into());
    let mut labels = WindowControlLabels::new().unwrap();
    for scheme in [Scheme::Light, Scheme::Dark] {
        for percent in [smallest, 100, 200, largest] {
            for edge in Edge::ALL {
                for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                    let mut dock = Dock::shipped();
                    dock.set_edge(edge);
                    let look = EgressStatusLook {
                        contrast: Contrast::AsDesigned,
                        scheme,
                        scale: TextScale::percent(percent).unwrap(),
                        reading,
                    };
                    let drawn = picture(
                        &drawn_on,
                        &words(),
                        &dock,
                        &mut labels,
                        (1366, 768),
                        look,
                        0,
                    )
                    .unwrap();
                    let at = (look, edge);
                    assert_eq!(drawn.rows.len(), 1, "{at:?}");
                    assert!(!drawn.inked.is_empty(), "{at:?}");
                    for area in drawn
                        .solids
                        .iter()
                        .map(|solid| solid.area)
                        .chain(drawn.inked.iter().map(|inked| inked.area))
                    {
                        assert_eq!(area.intersection(output), Some(area), "{at:?}: {area:?}");
                    }
                    for inked in &drawn.inked {
                        assert_eq!(
                            inked.pixels.len(),
                            (inked.area.size.w * inked.area.size.h) as usize
                        );
                    }
                }
            }
        }
    }
}

/// **Lines that do not all fit are never dropped silently**: as many as fit
/// are drawn in order, and the last row carries the indicator's own count of
/// everything that is leaving.
#[test]
fn lines_that_do_not_fit_end_with_the_indicators_own_count() {
    let strings = words();
    let (_, largest) = TextScale::range();
    let mut indicator = Indicator::default();
    for _ in 0..40 {
        drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    }
    let look = EgressStatusLook {
        contrast: Contrast::AsDesigned,
        scale: TextScale::percent(largest).unwrap(),
        ..light()
    };
    let mut labels = WindowControlLabels::new().unwrap();
    let handed_over = handed(&indicator);
    let drawn = picture(
        &handed_over,
        &strings,
        &Dock::shipped(),
        &mut labels,
        (1366, 768),
        look,
        0,
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
    let terracottas = drawn
        .solids
        .iter()
        .filter(|solid| solid.colour == terracotta())
        .count();
    assert_eq!(terracottas, drawn.rows.len());
}

/// **An output that cannot hold a dock refuses a lit frame** rather than
/// drawing it without the indicator — and a quiet indicator, which draws
/// nothing, is not refused anywhere.
#[test]
fn an_output_that_cannot_hold_a_dock_refuses_a_lit_frame_and_not_a_quiet_one() {
    let mut labels = WindowControlLabels::new().unwrap();
    let quiet = handed(&Indicator::default());
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let lit = handed(&indicator);
    for size in [(100, 100), (0, 0), (-5, 800), (1920, -1), (20_000, 1080)] {
        assert!(
            matches!(
                picture(
                    &lit,
                    &words(),
                    &Dock::shipped(),
                    &mut labels,
                    size,
                    light(),
                    0
                ),
                Err(RenderError::EgressStatusScene)
            ),
            "{size:?}"
        );
        assert!(
            picture(
                &quiet,
                &words(),
                &Dock::shipped(),
                &mut labels,
                size,
                light(),
                0,
            )
            .unwrap()
            .is_empty(),
            "{size:?}"
        );
    }
}

/// The row's words contrast with its ground as text must, in both schemes.
#[test]
fn the_words_are_readable_on_their_ground() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let palette = crate::egress_status_raster::palette(scheme, Contrast::AsDesigned);
        let colour = |[red, green, blue]: [u8; 3]| alo_appearance::Colour::of(red, green, blue);
        let contrast = colour(palette.ink).contrast_with(colour(palette.ground));
        assert!(
            contrast >= alo_appearance::ENOUGH_FOR_TEXT,
            "{scheme:?}: {contrast:.2}"
        );
    }
}

/// **High contrast draws the indicator in the other palette, mark and all.**
/// The rows are where they were, every flat colour is one
/// `alo_access::HighContrast` decided, and the mark is still the agent's — that
/// palette's own terracotta, which is the reserved colour taken deep enough to
/// read rather than another colour wearing the agent's meaning (ADR 0010).
#[test]
fn high_contrast_draws_the_indicator_and_its_mark_in_the_other_palette() {
    let mut indicator = Indicator::default();
    indicator
        .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
        .unwrap();
    for scheme in [Scheme::Light, Scheme::Dark] {
        let designed = drawn_as(&indicator, EgressStatusLook { scheme, ..light() });
        let high = drawn_as(
            &indicator,
            EgressStatusLook {
                scheme,
                contrast: Contrast::High,
                ..light()
            },
        );
        assert_eq!(
            designed.rows.len(),
            high.rows.len(),
            "{scheme:?}: high contrast changed how many rows there are"
        );
        let palette = crate::access_contrast::every_colour_of(
            Contrast::High,
            scheme,
            Token::Terracotta.colour(),
        );
        for solid in &high.solids {
            assert!(
                palette.contains(&solid.colour),
                "{scheme:?}: {:?} is drawn in {:?}, which is in no palette this crate may use",
                solid.area,
                solid.colour
            );
        }
        let agents = rgb(alo_access::HighContrast::of(scheme).accent);
        assert!(
            high.solids.iter().any(|solid| solid.colour == agents),
            "{scheme:?}: the agent's mark is not on the indicator in high contrast"
        );
        assert!(
            !high.solids.iter().any(|solid| solid.colour == terracotta()),
            "{scheme:?}: the design's terracotta survived into high contrast"
        );
    }
}
