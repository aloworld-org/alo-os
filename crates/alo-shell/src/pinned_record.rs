//! The record, opened by hand on an afternoon of entries, held to the pixels it
//! drew before the UI kit moved anything (`crate::the_desktops_pictures_pinned`).
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{Drawn, LOGICAL, every_way, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::record_testing::{a_clock, an_afternoon_kept, light, words};
use crate::{RecordLook, RecordOpened, RecordWindow, WindowControlLabels};

/// **The record draws the pixels it drew before**, at every scale and text size.
#[test]
fn the_record_draws_the_pixels_it_drew_before() {
    let kept = an_afternoon_kept();
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&kept.recounting()),
        RecordOpened::Shown
    );
    let strings = words();
    let now = every_way(|display, text| {
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = crate::record_raster::picture(
            window.shows(),
            &strings,
            &mut labels,
            in_pixels(LOGICAL, display),
            RecordLook {
                scale: text,
                ..light()
            },
            a_clock(),
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
    held("The record", &now, &BEFORE);
}

/// What the record drew before. See `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 0] = [];
