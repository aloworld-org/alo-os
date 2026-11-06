//! **The three hands**: pointing at a link, open over something that can be
//! picked up, and closed while it is held.
//!
//! Each is drawn as the fingers, the palm and the thumb laid over one another
//! as separate pieces. They are one colour, so where they overlap cannot be
//! seen; and the white keyline round each piece shows as a fine line between
//! the fingers, which is what makes a hand read as a hand at 24 pixels.

use crate::artwork::{Artwork, Part};

/// A hand pointing up with its first finger. The tip of that finger clicks.
pub(crate) fn pointer() -> Artwork {
    Artwork {
        parts: vec![
            Part::filled(finger(12.0, 4.0, 16.0, 18.0)),
            Part::filled(finger(16.4, 11.0, 19.8, 19.0)),
            Part::filled(finger(20.2, 12.5, 23.6, 19.5)),
            Part::filled(palm(10.0, 15.0, 23.6, 27.5)),
            Part::filled("M10.5 19 L10.5 24 L5.8 18.5 Q4.8 17.4 5.3 16.3 Q6 14.9 7.4 16 Z"),
        ],
        hotspot: (14.0, 4.0),
    }
}

/// An open hand, all four fingers up and the thumb out.
pub(crate) fn grab() -> Artwork {
    Artwork {
        parts: vec![
            Part::filled(finger(9.0, 8.0, 12.0, 18.0)),
            Part::filled(finger(12.6, 5.5, 15.6, 17.0)),
            Part::filled(finger(16.2, 6.0, 19.2, 17.0)),
            Part::filled(finger(19.8, 8.5, 22.8, 18.0)),
            Part::filled(palm(9.0, 14.0, 22.8, 27.0)),
            Part::filled("M9.5 17 L9.5 22.5 L5 16.4 Q4 15.3 4.4 14.2 Q5 12.8 6.3 13.8 Z"),
        ],
        hotspot: (16.0, 16.0),
    }
}

/// A closed hand, the fingers folded down over the palm.
pub(crate) fn grabbing() -> Artwork {
    Artwork {
        parts: vec![
            Part::filled(finger(9.0, 11.0, 12.0, 18.0)),
            Part::filled(finger(12.6, 10.0, 15.6, 17.0)),
            Part::filled(finger(16.2, 10.0, 19.2, 17.0)),
            Part::filled(finger(19.8, 11.0, 22.8, 18.0)),
            Part::filled(palm(9.0, 14.0, 22.8, 26.0)),
            Part::filled("M9.5 18 L9.5 21 L6.5 19.8 Q5 19.6 5 18.3 Q5 16.8 6.5 17 Z"),
        ],
        hotspot: (16.0, 16.0),
    }
}

/// A finger: an upright capsule from `(left, top)` to `(right, bottom)`, round
/// at both ends.
fn finger(left: f32, top: f32, right: f32, bottom: f32) -> String {
    let r = (right - left) / 2.0;
    let (upper, lower) = (top + r, bottom - r);
    format!(
        "M{left} {upper} A{r} {r} 0 0 1 {right} {upper} L{right} {lower} \
         A{r} {r} 0 0 1 {left} {lower} Z"
    )
}

/// The palm: a rectangle with well-rounded corners.
fn palm(left: f32, top: f32, right: f32, bottom: f32) -> String {
    let r = 4.5_f32;
    let (inner_left, inner_right) = (left + r, right - r);
    let (inner_top, inner_bottom) = (top + r, bottom - r);
    format!(
        "M{inner_left} {top} L{inner_right} {top} A{r} {r} 0 0 1 {right} {inner_top} \
         L{right} {inner_bottom} A{r} {r} 0 0 1 {inner_right} {bottom} \
         L{inner_left} {bottom} A{r} {r} 0 0 1 {left} {inner_bottom} \
         L{left} {inner_top} A{r} {r} 0 0 1 {inner_left} {top} Z"
    )
}
