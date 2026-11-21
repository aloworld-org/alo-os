//! **Every kind of alo's own window: its band, drawn by the one function,
//! shows no controls at rest and every control it has on approach.**
//!
//! A test of the rule over every kind, through `every_kind_of_band_testing`'s
//! exhaustive list, and through [`crate::Server::the_band_drawn`], which is
//! what a booted machine draws every band with
//! (`tests/every_band_is_drawn_through_the_band.rs` holds that no other file
//! draws one). The count drawn on approach is held to
//! `window_edge_reading::the_controls_of`, the list a reader is told and a
//! press is checked against, so what is drawn, heard and pressed cannot part.
//!
//! **What would have to be true for this to pass while the product is
//! broken:** the frame would have to stop asking this function, or paint the
//! picture somewhere off the screen. The first is the source test's; the
//! second is the walk's band section (#1440), which reads the pixels.

use smithay::utils::{Physical, Point};

use crate::Server;
use crate::every_kind_of_band_testing::{
    A_DISPLAY, every_kind, on_the_band, opened, the_pointer_at,
};
use crate::which_band::ABand;

/// The far corner of the display, in no band.
const FAR_AWAY: (i32, i32) = (A_DISPLAY.0 - 2, A_DISPLAY.1 - 2);

/// How many controls `band`'s band draws now, by the one drawing function.
fn drawn(server: &Server, band: &ABand) -> usize {
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    server
        .the_band_drawn(
            band,
            &crate::approval_testing::words(),
            &mut fonts,
            (A_DISPLAY, 100),
        )
        .expect("the band is on the display")
        .how_many_controls_are_drawn()
}

/// **None at rest, all on approach, none again once the pointer leaves**, on
/// every kind of alo's window.
#[test]
fn every_kind_of_alo_window_draws_no_controls_at_rest_and_all_on_approach() {
    for kind in every_kind() {
        let mut it = opened(kind, &format!("band-drawn-{kind:?}"));
        let band = ABand::Alos(it.band);
        let has = crate::window_edge_reading::the_controls_of(it.band.decorations()).len();
        assert!(has > 0, "the control: {kind:?} has controls");

        the_pointer_at(&mut it.server, Point::<i32, Physical>::from(FAR_AWAY));
        assert_eq!(
            drawn(&it.server, &band),
            0,
            "{kind:?}'s band drew controls at rest"
        );
        the_pointer_at(&mut it.server, on_the_band(it.window));
        assert_eq!(
            drawn(&it.server, &band),
            has,
            "{kind:?}'s band did not draw every control it has on approach"
        );
        the_pointer_at(&mut it.server, Point::<i32, Physical>::from(FAR_AWAY));
        std::thread::sleep(
            crate::the_controls_fade::THEY_GO_IN + std::time::Duration::from_millis(40),
        );
        it.server.the_edges_were_told_where_the_pointer_is();
        assert_eq!(
            drawn(&it.server, &band),
            0,
            "{kind:?}'s controls stayed after the pointer left"
        );
    }
}

/// **Always show window controls is obeyed by every kind.**
#[test]
fn every_kind_of_alo_window_obeys_always_show_window_controls() {
    for kind in every_kind() {
        let mut it = opened(kind, &format!("band-always-{kind:?}"));
        let band = ABand::Alos(it.band);
        let has = crate::window_edge_reading::the_controls_of(it.band.decorations()).len();
        it.server
            .the_window_controls_are(alo_appearance::WindowControls::Always);
        the_pointer_at(&mut it.server, Point::<i32, Physical>::from(FAR_AWAY));
        assert_eq!(
            drawn(&it.server, &band),
            has,
            "{kind:?}'s band ignored Always show window controls"
        );
    }
}
