use super::*;

#[test]
fn touching_rectangles_share_no_pixel() {
    let left = Area::new((0, 0).into(), (10, 10).into());
    let right = Area::new((10, 0).into(), (10, 10).into());
    assert!(!left.overlaps(right));
    assert_eq!(left.intersection(right), None);
}

/// **A rectangle of no width inside another is its own intersection**, as it
/// was with Smithay's rectangle: `overlaps` compares edges strictly, and an
/// empty rectangle's left edge is inside. Kept, not corrected, because the
/// pixels a picture lands on depend on this arithmetic being the same.
#[test]
fn an_empty_rectangle_inside_is_its_own_intersection() {
    let whole = Area::from_size((10, 10).into());
    let empty = Area::new((2, 2).into(), (0, 5).into());
    assert_eq!(whole.intersection(empty), Some(empty));
}

#[test]
fn the_shared_pixels_are_the_overlap() {
    let a = Area::new((0, 0).into(), (10, 10).into());
    let b = Area::new((4, 6).into(), (10, 10).into());
    assert_eq!(
        a.intersection(b),
        Some(Area::new((4, 6).into(), (6, 4).into()))
    );
    assert!(a.contains_rect(Area::new((4, 6).into(), (6, 4).into())));
    assert!(!a.contains_rect(b));
}

#[test]
fn a_point_on_the_far_edge_is_outside() {
    let a = Area::new((2, 2).into(), (4, 4).into());
    assert!(a.contains((2, 2)));
    assert!(a.contains((5, 5)));
    assert!(!a.contains((6, 5)));
    assert!(!a.contains((5, 6)));
}
