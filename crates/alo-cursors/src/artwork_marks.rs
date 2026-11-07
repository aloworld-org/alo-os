//! **The cursors that are a mark rather than an arrow or a hand**: the text
//! cursor, the cross, the struck circle, the turning ring, the magnifiers, and
//! the two small badges the arrow carries.

use crate::artwork::{Artwork, Part};

/// The middle of the box.
const MIDDLE: (f32, f32) = (16.0, 16.0);

/// The crosshair's line width: eight thirds of a unit, which is two pixels in a
/// 24-pixel box.
const CROSS_WIDTH: f32 = 8.0 / 3.0;

/// The I-beam: a stem with a short bar at each end. Its middle clicks.
pub(crate) fn text() -> Artwork {
    Artwork {
        parts: vec![Part::filled(
            "M12 5 L20 5 L20 7 L17 7 L17 25 L20 25 L20 27 L12 27 L12 25 L15 25 L15 7 L12 7 Z",
        )],
        hotspot: MIDDLE,
    }
}

/// The I-beam turned on its side.
pub(crate) fn vertical_text() -> Artwork {
    Artwork {
        parts: vec![Part::filled(
            "M5 12 L7 12 L7 15 L25 15 L25 12 L27 12 L27 20 L25 20 L25 17 L7 17 L7 20 L5 20 Z",
        )],
        hotspot: MIDDLE,
    }
}

/// A fine cross with an open middle, so what is aimed at stays visible.
pub(crate) fn crosshair() -> Artwork {
    Artwork {
        parts: vec![Part::stroked(
            "M16 4 L16 13 M16 19 L16 28 M4 16 L13 16 M19 16 L28 16",
            // **Two whole pixels at 24 and at 48**, centred on the middle, so the
            // cross is crisp where it is used most. Two units was a pixel and a
            // half, which no pixel covered solidly: a soft cross nobody can aim.
            CROSS_WIDTH,
            false,
        )],
        hotspot: MIDDLE,
    }
}

/// A ring struck through from top-left to bottom-right.
pub(crate) fn not_allowed() -> Artwork {
    Artwork {
        parts: vec![
            Part::stroked("M25 16 A9 9 0 1 1 7 16 A9 9 0 1 1 25 16 Z", 3.0, false),
            Part::stroked("M9.64 9.64 L22.36 22.36", 3.0, false),
        ],
        hotspot: MIDDLE,
    }
}

/// Three quarters of a ring, which is what *turning* looks like standing still.
pub(crate) fn wait() -> Artwork {
    Artwork {
        parts: vec![Part::stroked("M16 7 A9 9 0 1 1 7 16", 3.0, true)],
        hotspot: MIDDLE,
    }
}

/// A magnifier: a lens, a handle, and a plus or a minus in the lens. The middle
/// of the lens clicks.
pub(crate) fn magnifier(plus: bool) -> Artwork {
    let sign = if plus {
        "M10 13.5 L17 13.5 M13.5 10 L13.5 17"
    } else {
        "M10 13.5 L17 13.5"
    };
    Artwork {
        parts: vec![
            Part::stroked(
                "M21 13.5 A7.5 7.5 0 1 1 6 13.5 A7.5 7.5 0 1 1 21 13.5 Z",
                2.5,
                false,
            ),
            Part::stroked("M19 19 L26.5 26.5", 3.5, true),
            Part::stroked(sign, 2.0, false),
        ],
        hotspot: (13.5, 13.5),
    }
}

/// The small turning ring the arrow carries while something is working.
pub(crate) fn a_small_ring() -> Part {
    Part::stroked("M24 20.2 A3.8 3.8 0 1 1 20.2 24", 2.2, true)
}

/// The small plus the arrow carries when what is dragged will be copied.
pub(crate) fn a_small_plus() -> Part {
    Part::stroked("M20.5 24 L27.5 24 M24 20.5 L24 27.5", 2.4, false)
}
