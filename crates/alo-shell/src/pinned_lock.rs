//! The lock screen asking for a password, with a name and one letter typed,
//! held to the pixels it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_appearance::{Appearance, DisplayId, Scheme};
use alo_egress::Indicator;

use super::{Drawn, LOGICAL, every_way, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::lock_testing::{typed, words};
use crate::{Contrast, LockBackground, LockLook, SignInLook, WindowControlLabels};

/// **The lock screen draws the pixels it drew before**, at every scale and text
/// size.
#[test]
fn the_lock_screen_draws_the_pixels_it_drew_before() {
    let surface = typed("ada", "x");
    let screen = surface
        .snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000),
            &Appearance::shipped(),
            &DisplayId::named("eDP-1").unwrap(),
            Some(alo_locking::Battery::reading(72, true).unwrap()),
            &Indicator::default(),
        )
        .unwrap();
    let region = alo_formats::Regionally::reading("en").unwrap();
    let timezone = alo_formats::Timezone::named("Europe/Berlin").unwrap();
    let strings = words();
    let now = every_way(|display, text| {
        let background =
            LockBackground::prepare(screen.image(), in_pixels(LOGICAL, display)).unwrap();
        let look = LockLook {
            appearance: SignInLook {
                contrast: Contrast::AsDesigned,
                scheme: Scheme::Dark,
                scale: text,
            },
            region: &region,
            timezone: &timezone,
            strings: &strings,
        };
        let drawn = crate::lock_raster::picture(
            &screen,
            &background,
            Some(surface.shows()),
            &mut WindowControlLabels::new().unwrap(),
            &look,
            display,
        )
        .unwrap();
        let mut hash = Hashed::begun();
        hash.raw(&drawn.pixels);
        hash.done()
    });
    held("The lock screen", &now, &BEFORE);
}

/// What the lock screen drew before. See `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 0] = [];
