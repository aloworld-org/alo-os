//! The teal edge and the band with Stop, as `11:146` measures them.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::the_tasks_a_person_opened::WhichTask;
use smithay::utils::{Point, Size};

/// The Notes window of `11:146`: 1056 wide, 466 under its 54-tall band.
fn notes() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((56, 124)), Size::from((1056, 466)))
}

/// alo, as it is in a window.
fn alo(how: HowAloIsHere) -> AloHere {
    AloHere {
        which: WhichTask::numbered(0),
        how,
        status: match how {
            HowAloIsHere::Working => alo_tasks::words::ALO_IS_WORKING,
            HowAloIsHere::WaitingForThePerson => alo_tasks::words::WAITING_FOR_YOUR_PERMISSION,
            HowAloIsHere::Stopping => alo_tasks::words::STOPPING,
        },
        what: "summarise this document".to_owned(),
    }
}

/// The fonts every drawing shapes with.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::MANROPE.to_vec());
    fonts
}

/// **The band and Stop where `11:173` and `11:175` put them**: 20 in, 20 up,
/// 48 tall; Stop 64 × 30, 12 from the band's end and 9 down.
#[test]
fn the_band_and_stop_are_where_the_frame_puts_them() {
    let band = the_band_of(notes()).expect("room for the band");
    assert_eq!(
        band.band,
        Rectangle::new(
            Point::from((76, 124 + 466 - 20 - 48)),
            Size::from((1016, 48))
        )
    );
    assert_eq!(
        band.stop,
        Rectangle::new(
            Point::from((76 + 1016 - 12 - 64, 124 + 466 - 20 - 48 + 9)),
            Size::from((64, 30))
        )
    );
}

/// **A window too small for the band draws none**, rather than a band whose
/// Stop falls off it; Stop is then in the task window, as always.
#[test]
fn a_window_too_small_for_the_band_draws_none() {
    let small = Rectangle::new(Point::from((0, 0)), Size::from((200, 466)));
    assert!(the_band_of(small).is_none());
    let short = Rectangle::new(Point::from((0, 0)), Size::from((1056, 80)));
    assert!(the_band_of(short).is_none());
}

/// **What the band says is the task's status and the person's own words.**
#[test]
fn the_band_says_the_status_and_what_was_asked() {
    assert_eq!(
        what_the_band_says(&alo(HowAloIsHere::Working)),
        "alo is working \u{b7} summarise this document"
    );
    assert_eq!(
        what_the_band_says(&alo(HowAloIsHere::Stopping)),
        "Stopping\u{2026} \u{b7} summarise this document"
    );
}

/// The colours a set of rasters uses where they are fully covered.
fn solid_colours(rasters: &[Washed]) -> Vec<[u8; 3]> {
    rasters
        .iter()
        .flat_map(|it| it.pixels.as_chunks::<4>().0)
        .filter_map(|pixel| match pixel {
            [red, green, blue, 255] => Some([*red, *green, *blue]),
            _ => None,
        })
        .collect()
}

/// **Deep teal only while alo is acting** — the edge and the band's ink —
/// and never while it waits on the person, which is navy.
#[test]
fn teal_only_while_alo_is_acting() {
    let teal = rgb(Role::AccentDefault);
    let card = Rectangle::new(Point::from((56, 70)), Size::from((1056, 520)));
    for (how, teal_expected) in [
        (HowAloIsHere::Working, true),
        (HowAloIsHere::Stopping, true),
        (HowAloIsHere::WaitingForThePerson, false),
    ] {
        let here = alo(how);
        let (rasters, words) = drawn(
            AlosBand {
                here: &here,
                card,
                window: notes(),
                stop_pointed_at: false,
            },
            &mut fonts(),
            100,
            (4000, 3000),
        );
        assert_eq!(
            solid_colours(&rasters).contains(&teal),
            teal_expected,
            "{how:?}"
        );
        assert!(
            words.len() >= 2,
            "{how:?}: the words and Stop were not both written"
        );
    }
}

/// **The edge goes round the whole card**, band included, and nothing is
/// drawn outside it.
#[test]
fn the_edge_goes_round_the_whole_card() {
    let card = Rectangle::new(Point::from((56, 70)), Size::from((1056, 520)));
    let here = alo(HowAloIsHere::Working);
    let (rasters, _) = drawn(
        AlosBand {
            here: &here,
            card,
            window: notes(),
            stop_pointed_at: false,
        },
        &mut fonts(),
        100,
        (4000, 3000),
    );
    let edge = rasters.first().expect("the edge").area;
    assert_eq!(edge, crate::painted::drawn_at(in_pixels(card, 100)));
    for raster in &rasters {
        assert!(
            in_pixels(card, 100).contains_rect(crate::painted::on_the_frame(raster.area)),
            "{:?} is drawn outside the window",
            raster.area
        );
    }
}
