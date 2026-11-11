//! The Settings window, opened on its first page, held to the pixels it drew
//! before the UI kit moved anything (`crate::the_desktops_pictures_pinned`).
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{Drawn, LOGICAL, every_way, held, in_pixels};
use crate::WindowControlLabels;
use crate::a_picture_hashed::Hashed;
use crate::settings_testing::{a_persons_machine, light, noon, settings_words};

/// **Settings draws the pixels it drew before**, at every scale and text size.
#[test]
fn settings_draws_the_pixels_it_drew_before() {
    let machine = a_persons_machine("pinned");
    let window = machine.opened();
    let words = settings_words();
    let now = every_way(|display, text| {
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = crate::settings_raster::picture(
            &window,
            &words,
            &mut labels,
            in_pixels(LOGICAL, display),
            crate::SettingsLook {
                scale: text,
                ..light()
            },
            noon(),
            None,
            display,
        )
        .unwrap();
        let mut hash = Hashed::begun();
        if let Some(panel) = drawn.panel {
            hash.area(panel);
        }
        if let Some(washed) = &drawn.washed {
            hash.washed(washed);
        }
        hash.done()
    });
    held("Settings", &now, &BEFORE);
}

/// What Settings drew before. See `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 6] = [
    Drawn { scale: 100, text: 100, hash: 0xe3c8_8e17_bd8c_da48 },
    Drawn { scale: 100, text: 125, hash: 0x5c77_5c56_72f2_2bc6 },
    Drawn { scale: 150, text: 100, hash: 0xec6b_55c0_e4e8_a19f },
    Drawn { scale: 150, text: 125, hash: 0xf2ae_8b01_1e82_2fb6 },
    Drawn { scale: 200, text: 100, hash: 0x94f8_4cc9_3c9d_29ad },
    Drawn { scale: 200, text: 125, hash: 0x7d7d_2df0_d85f_4a51 },
];
