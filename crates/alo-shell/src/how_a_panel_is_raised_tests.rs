//! A look's figures at a display's scale: doubled at 200%, unchanged at 100%,
//! and an impossible scale refused rather than drawn.

use super::HowItIsRaised;

#[test]
fn at_two_hundred_every_figure_doubles() {
    let one = HowItIsRaised::FLOATING.at(1.0);
    let two = HowItIsRaised::FLOATING.at(2.0);
    assert_eq!(one.corner(), 18.0);
    assert_eq!(two.corner(), 36.0, "the corner did not double at 200%");
    assert_eq!(two.falls(), one.falls() * 2.0, "the fall did not double");
    assert_eq!(two.blurs(), one.blurs() * 2.0, "the blur did not double");
    assert_eq!(two.spreads(), -8.0, "the spread did not double");
    assert_eq!(two.a_measure(12), 24.0, "a well's corner did not double");
    assert_eq!(
        two.darkness(),
        one.darkness(),
        "a shadow is not darker on a denser screen"
    );
}

/// **The Dock's figures are the ones it drew before it had a look**, which is
/// the arithmetic half of *the Dock looks identical*; the pixel half is in
/// `crate::dock_surface`'s tests.
#[test]
fn the_docks_look_unscaled_is_the_docks_measures() {
    let dock = HowItIsRaised::THE_DOCKS.unscaled();
    assert_eq!(dock.corner(), 18.0);
    assert_eq!(dock.falls(), 8.0);
    assert_eq!(dock.blurs(), 24.0);
    assert_eq!(dock.spreads(), 0.0);
    assert_eq!(dock.darkness(), 0.08);
    assert_eq!(
        dock.reach(),
        32,
        "the raster's reach moved, which moves every Dock panel"
    );
    assert_eq!(dock, HowItIsRaised::THE_DOCKS.at(1.0));
}

/// The floating shadow reaches its fall and its blur past the panel, and a
/// spread that tucks it in does not make the raster smaller than the blur
/// needs.
#[test]
fn the_reach_covers_the_fall_and_the_blur_at_every_scale() {
    assert_eq!(HowItIsRaised::FLOATING.at(1.0).reach(), 56);
    assert_eq!(HowItIsRaised::FLOATING.at(2.0).reach(), 112);
    assert_eq!(HowItIsRaised::FLOATING.at(1.25).reach(), 70);
}

/// **A display described with no scale draws an ordinary panel**, not a
/// panel with no corner or a raster of no size.
#[test]
fn a_scale_that_is_not_a_positive_number_is_taken_as_one() {
    for nonsense in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            HowItIsRaised::FLOATING.at(nonsense),
            HowItIsRaised::FLOATING.at(1.0),
            "a scale of {nonsense} was drawn at"
        );
    }
}
