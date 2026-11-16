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
///
/// **And on 2026-11-13**: *Accounts* after Default apps, approved by the
/// owner with the GitHub sign-in. Measured once by running this test on the
/// documents lane's machine; it runs again on every gate.
///
/// **And on 2026-11-16**: the side column's *Folders alo can use* became
/// *What alo can use* — the owner's "Fix wording, then merge". Taken from the
/// gate's own run of this test on the pull request that renamed it; it runs
/// again on every gate.
const BEFORE: [Drawn; 6] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0xcfeb_7743_6d34_b577,
    },
    Drawn {
        scale: 100,
        text: 125,
        hash: 0x776b_bbdc_888b_9b18,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0x1212_1175_6151_179e,
    },
    Drawn {
        scale: 150,
        text: 125,
        hash: 0x8ff8_335b_11cf_2902,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0xd81b_9f8c_4c39_ef54,
    },
    Drawn {
        scale: 200,
        text: 125,
        hash: 0x37ae_5159_1817_48d0,
    },
];
