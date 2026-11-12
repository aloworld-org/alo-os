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
///
/// **Pinned again on 2026-11-12**: the side column gained *Default apps*
/// between Appearance and Region and time, so every picture of it moved.
const BEFORE: [Drawn; 6] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0x7eb0_efaa_4acc_763c,
    },
    Drawn {
        scale: 100,
        text: 125,
        hash: 0x2589_973a_1c7e_ab97,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0x3c6a_1c6e_2a81_acc4,
    },
    Drawn {
        scale: 150,
        text: 125,
        hash: 0x42e4_afbe_7081_c895,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0xdd3c_b1c0_1b7c_691a,
    },
    Drawn {
        scale: 200,
        text: 125,
        hash: 0xe955_25cd_d89b_aac3,
    },
];
