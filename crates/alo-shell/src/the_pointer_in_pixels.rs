//! Where the pointer is, in this display's physical pixels.
//!
//! The seat keeps the pointer in logical coordinates as `f64`; everything the panel was
//! laid out in is physical pixels. One conversion, asked in one place.
//!
//! # Why this is a file rather than a line repeated
//!
//! Three roads need it — the peek, the reveal, and a click bringing a window back — and
//! the first two each carried their own copy of the cast with their own
//! `#[expect(clippy::cast_possible_truncation)]` and their own paragraph explaining why
//! truncation is correct. Two copies is a coincidence; a third is a rule nobody wrote
//! down, and the next person to need it would have had to decide which of two
//! justifications to trust.
//!
//! The justification belongs here, once: **a pointer inside a pixel belongs to that
//! pixel.** Truncation is the half-open rule the rectangles already use — a pointer at
//! `4.9` is in the pixel that starts at `4` — so it is the answer wanted rather than an
//! accident of arithmetic.

use smithay::utils::{Physical, Point};

impl crate::Server {
    /// The pointer in physical pixels, or `None` when the seat has no pointer.
    ///
    /// **No pointer is not the origin.** A machine whose seat has never seen a pointing
    /// device has no position, and answering `(0, 0)` would put it in the top-left
    /// corner of the screen — which on a right-to-left layout is inside the panel's own
    /// column, so a question about the panel would answer *yes* on a machine with no
    /// mouse attached.
    pub(crate) fn where_the_pointer_is_in_pixels(&self) -> Option<Point<i32, Physical>> {
        let at = self.surfaces.pointer.as_ref()?.location;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer inside a pixel belongs to that pixel: the half-open rule the \
                      rectangles use, and the reason is in this file's header"
        )]
        Some(Point::from((at.x as i32, at.y as i32)))
    }
}
