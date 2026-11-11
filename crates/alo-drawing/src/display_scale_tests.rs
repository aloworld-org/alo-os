use super::*;

#[test]
fn at_one_hundred_every_measure_is_itself() {
    for logical in [0, 1, 3, 8, 13, 48, 76, 344, 448, 16_384] {
        assert_eq!(
            DisplayScale::ONE.px(logical),
            i32::try_from(logical).unwrap_or(0)
        );
        assert_eq!(
            DisplayScale::ONE.logical(i32::try_from(logical).unwrap_or(0)),
            logical
        );
    }
}

#[test]
fn at_two_hundred_every_measure_is_exactly_twice_itself() {
    let dense = DisplayScale::of_per_cent(200);
    for logical in [0_u32, 1, 3, 8, 13, 48, 76, 344, 448] {
        assert_eq!(dense.px(logical), 2 * i32::try_from(logical).unwrap_or(0));
    }
}

#[test]
fn a_press_anywhere_in_a_doubled_slot_lands_in_the_slot_it_was_drawn_from() {
    let dense = DisplayScale::of_per_cent(200);
    // A slot 48 long starting 8 along is 16 to 112 on a two-times display.
    for pixel in 16..112 {
        let logical = dense.logical(pixel);
        assert!(
            (8..56).contains(&logical),
            "pixel {pixel} read as {logical}"
        );
    }
    assert_eq!(dense.logical(15), 7);
    assert_eq!(dense.logical(112), 56);
}

#[test]
fn a_fractional_scale_rounds_to_the_nearest_pixel() {
    let one_and_a_quarter = DisplayScale::of_per_cent(125);
    assert_eq!(one_and_a_quarter.px(48), 60);
    assert_eq!(one_and_a_quarter.px(13), 16);
    assert_eq!(one_and_a_quarter.px(3), 4);
}

#[test]
fn nought_is_read_as_one_and_a_point_before_the_edge_as_nought() {
    assert_eq!(DisplayScale::of_per_cent(0), DisplayScale::ONE);
    assert_eq!(DisplayScale::of_per_cent(200).logical(-5), 0);
}

#[test]
fn a_type_size_follows_the_scale() {
    assert!((DisplayScale::of_per_cent(200).type_size(15) - 30.0).abs() < f32::EPSILON);
    assert!((DisplayScale::ONE.type_size(15) - 15.0).abs() < f32::EPSILON);
}
