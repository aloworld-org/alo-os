//! **The composer's field on the Dock is filled `#F5F6F7`**, the design's
//! value for it (`349:25790` on the edge Dock, `I321:17430;74:53` on the bottom
//! one; `docs/design/the-docks-visual-specification.md`, *The composer*), read
//! back out of the Dock's own picture.
//!
//! **Asked of the Dock and not of the surface.** `dock_surface_tests` handed
//! the surface `#F5F6F7` itself, in a fixture with no wells, so it could only
//! ever show that the surface paints the colour it is given. What the Dock
//! gives it was asked nowhere, and the Dock gave it the badge's `#EEF2F4`.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the case it was on"
)]

use alo_appearance::{Following, Scheme};
use alo_dock::places::WhatIsHere;
use alo_dock::{Dock, Edge};
use alo_strings::Direction;

use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;

/// The design's fill for the composer's field in the light appearance, opaque.
const THE_DESIGNS_FIELD: [u8; 4] = [0xF5, 0xF6, 0xF7, 0xFF];

/// **On every Dock that has a composer, its field is the design's colour**:
/// every edge, at one to one and at twice the pixels, in the light appearance
/// the design measures.
#[test]
fn the_composers_field_is_the_designs_colour_on_every_dock() {
    let mut light = an_appearance();
    light.follow(Following::from(Scheme::Light));
    let look = noon_look(&light, Direction::LeftToRight);
    assert_eq!(look.scheme(), Scheme::Light, "the premise: a light look");

    for edge in Edge::EVERY {
        for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let per_cent = i32::from(scale.per_cent());
            let drawn = crate::dock_raster::picture(
                &dock,
                look,
                (1440 * per_cent / 100, 960 * per_cent / 100),
                &[],
                &[],
                (
                    &crate::what_the_dock_has_open::NOTHING,
                    crate::desktop_testing::no_words(),
                ),
                None,
                &alo_dock::artwork::NOTHING,
                &mut crate::WindowControlLabels::new()
                    .expect("the bundled faces")
                    .fonts,
                scale,
            )
            .unwrap_or_else(|why| panic!("{edge:?} at {scale:?} would not draw: {why:?}"));

            let field =
                crate::dock_raster::where_this_slot_is(&drawn.standing, &WhatIsHere::TheComposer)
                    .unwrap_or_else(|| panic!("the premise: the {edge:?} Dock has a composer"));
            let surface = drawn
                .surface
                .as_ref()
                .unwrap_or_else(|| panic!("{edge:?} at {scale:?} drew no surface"));

            // The middle of the field, where no corner reaches. The field's
            // words are ink, painted after the surface, so they are not in
            // this raster at all.
            let (x, y) = (
                field.loc.x + field.size.w / 2 - surface.area.loc.x,
                field.loc.y + field.size.h / 2 - surface.area.loc.y,
            );
            let at = usize::try_from(y * surface.area.size.w + x).expect("inside the raster") * 4;
            let pixel = surface
                .pixels
                .get(at..at + 4)
                .unwrap_or_else(|| panic!("no pixel at ({x}, {y})"));
            assert_eq!(
                pixel, THE_DESIGNS_FIELD,
                "{edge:?} at {scale:?}: the composer's field is not the design's #F5F6F7"
            );
        }
    }
}
