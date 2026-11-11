//! The band alo draws above a window — a Notes window named *The next
//! chapter*, revealed, its controls shown, nothing pointed at — held to the
//! pixels it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
//!
//! **At three scales and one text size**: the band has no text size among its
//! inputs, so a second would draw the same picture.

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{Drawn, LOGICAL, every_scale, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::window_edge::{Decorations, edge_of};
use crate::window_edge_look::WhichLook;
use crate::window_edge_paint::{HowTheControlsLook, Pointing};
use crate::window_edge_picture::{EdgePicture, OverTheEdge, TheWindow};

/// A window with room above it for its band, in logical pixels.
fn window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((40, 100)), Size::from((1056, 466)))
}

/// Everything a band paints, in painting order.
fn hashed(drawn: &EdgePicture) -> u64 {
    let mut hash = Hashed::begun();
    for washed in drawn.behind().iter().chain(drawn.over()) {
        hash.washed(washed);
    }
    let (solids, inked) = drawn.flat();
    hash.solids(solids);
    hash.inked(inked);
    for washed in drawn.on_top() {
        hash.washed(washed);
    }
    hash.inked(drawn.on_top_words());
    hash.done()
}

/// **The window band draws the pixels it drew before**, at every scale.
#[test]
fn the_window_band_draws_the_pixels_it_drew_before() {
    let band = edge_of(
        window(),
        Decorations::TheShellDraws,
        true,
        Size::from(LOGICAL),
    );
    let now = every_scale(|display| {
        let mut fonts = crate::WindowControlLabels::new()
            .map(|labels| labels.fonts)
            .unwrap_or_else(|_| unreachable!("the shell's own fonts load"));
        let drawn = EdgePicture::of(
            &band,
            OverTheEdge::default(),
            TheWindow {
                title: Some("The next chapter"),
                application: Some("Notes"),
                look: WhichLook::of(true),
                body: Some(window()),
                laid_over: false,
                controls: HowTheControlsLook {
                    pointing: Pointing::default(),
                    open: false,
                    how_shown: crate::the_controls_fade::ALL_THE_WAY,
                    look: WhichLook::of(true),
                },
            },
            &mut fonts,
            in_pixels(LOGICAL, display),
            u32::from(display.per_cent()),
        )
        .unwrap_or_else(|| unreachable!("the band is wholly on the display"));
        hashed(&drawn)
    });
    held("The window band", &now, &BEFORE);
}

/// What the band drew before. See `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 3] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0x57e3_6fb5_798c_2521,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0x7371_bcb3_8115_b755,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0xcc81_b324_b016_7af1,
    },
];
