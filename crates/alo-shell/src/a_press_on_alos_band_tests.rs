//! Where a press lands on alo's band: Stop, the band, or nothing — and nothing
//! anywhere on a window alo operates, which wears the border alone.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{Lands, where_a_press_lands};
use crate::alo_in_a_window::{AloHere, HowAloIsHere, Wears};
use crate::the_tasks_a_person_opened::WhichTask;

/// The Notes window of `11:146`.
fn notes() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((56, 124)), Size::from((1056, 466)))
}

/// alo, at work in a window, wearing it this way.
fn alo(wears: Wears) -> AloHere {
    AloHere {
        which: WhichTask::numbered(0),
        how: HowAloIsHere::Working,
        status: alo_tasks::words::ALO_IS_WORKING,
        what: "summarise this document".to_owned(),
        wears,
        sending: None,
    }
}

/// **On an offered window, Stop is Stop and the rest of the band is the
/// band's**; off the band, the press is the window's.
#[test]
fn on_an_offered_window_stop_and_the_band_take_their_presses() {
    let band = crate::alos_band_in_a_window::the_band_of(notes()).expect("room for the band");
    let here = alo(Wears::TheBand);
    let on_stop = Point::from((band.stop.loc.x + 2, band.stop.loc.y + 2));
    let on_the_band = Point::from((band.band.loc.x + 2, band.band.loc.y + 2));
    assert_eq!(
        where_a_press_lands(&here, notes(), on_stop),
        Some(Lands::OnStop)
    );
    assert_eq!(
        where_a_press_lands(&here, notes(), on_the_band),
        Some(Lands::OnTheBand)
    );
    assert_eq!(
        where_a_press_lands(&here, notes(), Point::from((60, 130))),
        None
    );
}

/// **On a window alo operates, no press is the band's** — not where Stop would
/// be, not where the band would be — so the application keeps every press on
/// its own content.
#[test]
fn on_a_window_alo_operates_no_press_is_the_bands() {
    let band = crate::alos_band_in_a_window::the_band_of(notes()).expect("room for the band");
    let here = alo(Wears::TheBorder);
    for at in [
        Point::from((band.stop.loc.x + 2, band.stop.loc.y + 2)),
        Point::from((band.band.loc.x + 2, band.band.loc.y + 2)),
    ] {
        assert_eq!(
            where_a_press_lands(&here, notes(), at),
            None,
            "a press at {at:?} on a window alo operates was taken from it"
        );
    }
}
