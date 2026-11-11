//! The egress status with one question to a provider under way — its pill and
//! its sentence — held to the pixels it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::Scheme;
use alo_dock::Dock;
use alo_egress::{EgressPolicy, Indicator};
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;

use super::{Drawn, LOGICAL, every_way, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::egress_status_place::TheRoom;
use crate::egress_status_testing::{asking_a_provider, noon, words};
use crate::{Contrast, EgressStatus, EgressStatusLook, WindowControlLabels};

/// **The egress status draws the pixels it drew before**, at every scale and
/// text size.
#[test]
fn the_egress_status_draws_the_pixels_it_drew_before() {
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let mut status = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut status), &indicator),
        Drew::Shown
    );
    let handed = status.showing().unwrap().clone();
    let strings = words();
    let now = every_way(|display, text| {
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = crate::egress_status_raster::picture(
            &handed,
            &strings,
            &Dock::shipped(),
            &mut labels,
            TheRoom {
                size: in_pixels(LOGICAL, display),
                panel: None,
                scale: display,
            },
            EgressStatusLook {
                contrast: Contrast::AsDesigned,
                scheme: Scheme::Light,
                scale: text,
                reading: Direction::LeftToRight,
            },
        )
        .unwrap();
        let mut hash = Hashed::begun();
        for washed in &drawn.washed {
            hash.washed(washed);
        }
        hash.solids(&drawn.solids);
        hash.inked(&drawn.inked);
        hash.done()
    });
    held("The egress status", &now, &BEFORE);
}

/// What the egress status drew before. See
/// `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 6] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0xbae8_05dd_6a5b_a97a,
    },
    Drawn {
        scale: 100,
        text: 125,
        hash: 0xdd25_790b_aee5_948d,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0xcdf3_5d51_fddd_e36a,
    },
    Drawn {
        scale: 150,
        text: 125,
        hash: 0x7a7f_a132_c80c_3e03,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0x1200_9c7c_7d98_0849,
    },
    Drawn {
        scale: 200,
        text: 125,
        hash: 0xa8ba_8268_bf2a_887f,
    },
];
