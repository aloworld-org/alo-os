//! **The magnifier: what is under the pointer, drawn larger, over the whole
//! screen.**
//!
//! Task 12 of `docs/autonomy/v0-5-the-shell-plan.md`: *the magnifier and high
//! contrast apply as decided*. What a magnifier is and how much larger it draws
//! are `alo-access`' — [`alo_access::Setting::Magnifier`] and
//! [`Magnification`], which is kept in tenths so that one and a half times is a
//! whole number in a file — and nothing about either is decided here. This is
//! the drawing: one frame in, one frame out, the same extent.
//!
//! # It magnifies the frame, not the surfaces in it
//!
//! A magnifier that asked every window to draw itself larger would be a second
//! way of laying the screen out, and a window whose client declined would be a
//! hole in it. This takes the frame that was going to be shown and magnifies
//! **that**, so everything on the screen is magnified — a client's window, this
//! crate's own panels, the cursor, the egress indicator that law 1 promises is
//! always visible — and there is nothing that can be left out of it.
//!
//! # What is under the pointer stays under the pointer
//!
//! The view is the piece of the screen the magnification leaves room for,
//! centred on the pointer and **clamped inside the screen**, so a pointer at a
//! corner shows that corner rather than a band of whatever is beyond the edge.
//! Nothing outside the frame is ever sampled, and no pixel is invented: every
//! pixel drawn is a pixel of the frame that was going to be shown.
//!
//! # Nearest, not smoothed
//!
//! A magnified pixel is the pixel it came from, repeated. Smoothing would
//! soften exactly the edges — a letter's stem, the caret, the focus ring — that
//! somebody using a magnifier is trying to make out, and it would be this crate
//! deciding how a person's screen should look at four times the size.

use alo_access::{Magnification, Setting, TurnedOn};

use crate::ScanoutPixels;

/// **How much larger the magnifier draws, where the person turned it on.**
///
/// [`None`] where it is off, so a frame cannot be magnified by a machine nobody
/// asked.
#[must_use]
pub fn magnifying(turned_on: &TurnedOn) -> Option<Magnification> {
    turned_on
        .has(Setting::Magnifier)
        .then(|| turned_on.magnification())
}

/// A frame that could not be magnified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotMagnified {
    /// The frame's extent and its bytes disagree, or the extent is one no
    /// screen has. A frame that cannot be read is never half-drawn.
    #[error("the frame's extent and its pixels do not agree")]
    Frame,
    /// There was no room to keep the frame's pixels.
    #[error("there was no room for a magnified frame")]
    NoRoom,
}

/// **The frame, magnified `by` around the pointer at `at`.**
///
/// The same extent and the same stride: what is handed back is a frame a
/// display takes exactly as it would have taken the one handed in.
///
/// # Errors
/// [`NotMagnified::Frame`] for a frame whose extent and pixels disagree, and
/// [`NotMagnified::NoRoom`] when the pixels cannot be kept.
pub fn magnified(
    frame: &ScanoutPixels,
    at: (i32, i32),
    by: Magnification,
) -> Result<ScanoutPixels, NotMagnified> {
    let (width, height) = frame.size();
    let stride = usize::try_from(width).map_err(|_| NotMagnified::Frame)?;
    let stride = stride.checked_mul(4).ok_or(NotMagnified::Frame)?;
    if width == 0 || height == 0 {
        return Err(NotMagnified::Frame);
    }
    let source = frame.pixels();
    let length = stride
        .checked_mul(usize::try_from(height).map_err(|_| NotMagnified::Frame)?)
        .ok_or(NotMagnified::Frame)?;
    if source.len() != length {
        return Err(NotMagnified::Frame);
    }

    let view = View::of((width, height), at, by);
    let mut magnified = Vec::new();
    magnified
        .try_reserve_exact(length)
        .map_err(|_| NotMagnified::NoRoom)?;
    magnified.resize(length, 0);
    for row in 0..height {
        let from = view.row(row, height);
        let Some(taken) = source.get(from * stride..(from + 1) * stride) else {
            return Err(NotMagnified::Frame);
        };
        let Some(into) = magnified.get_mut(
            usize::try_from(row).map_err(|_| NotMagnified::Frame)? * stride
                ..(usize::try_from(row).map_err(|_| NotMagnified::Frame)? + 1) * stride,
        ) else {
            return Err(NotMagnified::Frame);
        };
        for (column, pixel) in into.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let from = view.column(column, width);
            let Some(source) = taken.as_chunks::<4>().0.get(from) else {
                return Err(NotMagnified::Frame);
            };
            *pixel = *source;
        }
    }
    frame.redrawn(magnified).ok_or(NotMagnified::Frame)
}

/// **The piece of the screen the magnified frame is made of**: where it starts
/// and how big it is, in the frame's own pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct View {
    /// Its left edge in the frame.
    left: u32,
    /// Its top edge in the frame.
    top: u32,
    /// How wide it is.
    width: u32,
    /// How tall it is.
    height: u32,
}

impl View {
    /// The view for a screen of `size`, centred on `at` and kept inside it.
    fn of(size: (u32, u32), at: (i32, i32), by: Magnification) -> Self {
        let tenths = u32::from(by.tenths()).max(1);
        let width = (size.0 * 10 / tenths).clamp(1, size.0);
        let height = (size.1 * 10 / tenths).clamp(1, size.1);
        Self {
            left: start(size.0, width, at.0),
            top: start(size.1, height, at.1),
            width,
            height,
        }
    }

    /// The frame row a magnified row is taken from.
    fn row(self, row: u32, height: u32) -> usize {
        let within = u64::from(row) * u64::from(self.height) / u64::from(height.max(1));
        (u64::from(self.top) + within) as usize
    }

    /// The frame column a magnified column is taken from.
    fn column(self, column: usize, width: u32) -> usize {
        let within = column as u64 * u64::from(self.width) / u64::from(width.max(1));
        (u64::from(self.left) + within) as usize
    }
}

/// Where a view of `view` pixels starts on a screen of `whole`, centred on
/// `at` and never hanging over either edge.
fn start(whole: u32, view: u32, at: i32) -> u32 {
    let furthest = whole.saturating_sub(view);
    let centred = i64::from(at) - i64::from(view / 2);
    centred.clamp(0, i64::from(furthest)) as u32
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::RowOrder;

    /// A frame of `width` by `height` whose every pixel says where it is.
    fn a_frame(width: u32, height: u32) -> ScanoutPixels {
        let mut rgba = Vec::new();
        for y in 0..height {
            for x in 0..width {
                rgba.extend_from_slice(&[x as u8 + 1, y as u8 + 1, (x + y) as u8 + 1, 255]);
            }
        }
        crate::readback::convert((width, height), RowOrder::TopToBottom, &rgba).unwrap()
    }

    /// The pixel at (`x`, `y`) of a frame, as the frame keeps it.
    fn pixel(frame: &ScanoutPixels, x: u32, y: u32) -> [u8; 4] {
        let stride = frame.size().0 as usize * 4;
        let at = y as usize * stride + x as usize * 4;
        frame.pixels()[at..at + 4].try_into().unwrap()
    }

    /// Twice, which is what a magnifier draws until somebody changes it.
    fn twice() -> Magnification {
        Magnification::twice()
    }

    /// **A machine nobody has asked magnifies nothing.**
    #[test]
    fn nothing_is_magnified_until_somebody_turns_it_on() {
        let mut turned_on = TurnedOn::nothing();
        assert_eq!(magnifying(&turned_on), None);
        turned_on.turn_on(Setting::Magnifier);
        assert_eq!(magnifying(&turned_on), Some(twice()));
        turned_on.turn_off(Setting::Magnifier);
        assert_eq!(magnifying(&turned_on), None);
    }

    /// **How much larger is the person's answer**, not a number in this file:
    /// a magnification of one and a half draws a different frame from twice.
    #[test]
    fn how_much_larger_is_the_persons_own_answer() {
        let frame = a_frame(16, 16);
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(Setting::Magnifier);
        turned_on.magnify_by(Magnification::of_tenths(15).unwrap());
        let theirs = magnifying(&turned_on).unwrap();
        assert_eq!(theirs, Magnification::of_tenths(15).unwrap());
        let at_theirs = magnified(&frame, (8, 8), theirs).unwrap();
        let at_twice = magnified(&frame, (8, 8), twice()).unwrap();
        assert_ne!(at_theirs.pixels(), at_twice.pixels());
    }

    /// **The frame handed back is a frame a display takes**: the same extent
    /// and the same stride as the one handed in.
    #[test]
    fn the_magnified_frame_is_the_same_frame_to_a_display() {
        let frame = a_frame(24, 18);
        let magnified = magnified(&frame, (12, 9), twice()).unwrap();
        assert_eq!(magnified.size(), frame.size());
        assert_eq!(magnified.pixels().len(), frame.pixels().len());
        magnified.frame().unwrap();
    }

    /// **Twice is twice**: each pixel of the half-sized view under the pointer
    /// becomes a two-by-two block, and the block is that pixel repeated rather
    /// than anything smoothed or invented.
    #[test]
    fn at_twice_every_pixel_is_its_own_two_by_two_block() {
        let frame = a_frame(8, 8);
        // The pointer in the middle: the view is the four-by-four square from
        // (2, 2), which is the half of the screen the pointer is in the middle
        // of.
        let magnified = magnified(&frame, (4, 4), twice()).unwrap();
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(
                    pixel(&magnified, x, y),
                    pixel(&frame, 2 + x / 2, 2 + y / 2),
                    "({x}, {y}) is not the pixel it magnifies"
                );
            }
        }
    }

    /// **What is under the pointer is what is magnified**: away from the
    /// edges, the pixel the pointer is on is drawn at the middle of the screen
    /// it is magnified onto.
    ///
    /// Near an edge it is not in the middle and must not be: the view is kept
    /// inside the screen, so the pointer moves within it rather than the screen
    /// showing a band of nothing. It is still drawn, which is the half of the
    /// promise that holds everywhere, and the next test holds the other half.
    #[test]
    fn what_is_under_the_pointer_is_in_the_middle_of_what_is_drawn() {
        let frame = a_frame(40, 40);
        for at in [(20, 20), (12, 28), (28, 12)] {
            let magnified = magnified(&frame, at, twice()).unwrap();
            assert_eq!(
                pixel(&magnified, 20, 20),
                pixel(&frame, at.0 as u32, at.1 as u32),
                "the pointer at {at:?} is not what the middle shows"
            );
        }
        // Against an edge, where the view is clamped: still drawn, somewhere.
        for at in [(31, 9), (0, 0), (39, 39)] {
            let magnified = magnified(&frame, at, twice()).unwrap();
            let under = pixel(&frame, at.0 as u32, at.1 as u32);
            assert!(
                (0..40).any(|y| (0..40).any(|x| pixel(&magnified, x, y) == under)),
                "the pointer at {at:?} is not on the magnified screen at all"
            );
        }
    }

    /// **The view is kept inside the screen**: a pointer in a corner shows
    /// that corner, and every pixel drawn is a pixel of the frame — nothing
    /// beyond an edge is invented or repeated to fill the screen out.
    #[test]
    fn a_pointer_at_a_corner_shows_the_corner_and_never_beyond_it() {
        let frame = a_frame(20, 20);
        for at in [(0, 0), (19, 0), (0, 19), (19, 19), (-40, 60)] {
            let magnified = magnified(&frame, at, twice()).unwrap();
            let inside: Vec<[u8; 4]> = (0..20)
                .flat_map(|y| (0..20).map(move |x| (x, y)))
                .map(|(x, y)| pixel(&frame, x, y))
                .collect();
            for y in 0..20 {
                for x in 0..20 {
                    assert!(
                        inside.contains(&pixel(&magnified, x, y)),
                        "({x}, {y}) at {at:?} is a pixel the frame never had"
                    );
                }
            }
        }
        // The top-left corner's own pixel is drawn for a pointer at the corner.
        let magnified = magnified(&frame, (0, 0), twice()).unwrap();
        assert_eq!(pixel(&magnified, 0, 0), pixel(&frame, 0, 0));
    }

    /// **The smallest magnification is still one**: the whole screen is drawn
    /// at every magnification `alo-access` allows, with no row or column left
    /// black.
    #[test]
    fn every_magnification_that_crate_allows_draws_the_whole_screen() {
        let frame = a_frame(32, 24);
        for tenths in [Magnification::SMALLEST, 20, 45, Magnification::LARGEST] {
            let by = Magnification::of_tenths(tenths).unwrap();
            let magnified = magnified(&frame, (16, 12), by).unwrap();
            for y in 0..24 {
                for x in 0..32 {
                    assert_ne!(
                        pixel(&magnified, x, y),
                        [0, 0, 0, 0],
                        "{tenths} tenths left ({x}, {y}) undrawn"
                    );
                }
            }
        }
    }
}
