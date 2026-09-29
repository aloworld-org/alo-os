//! Output-clipped, compositor-owned arrow geometry; no client resources or theme I/O.

use crate::{Cursor, RenderError};
use smithay::utils::{Physical, Rectangle};

/// One opaque output pixel and its contrast color.
type Pixel = crate::cursor_mask::Pixel;

/// Original scale-one mask: black outline, white interior, transparent elsewhere.
/// Tip at (0, 0). Neutral contrast is independent of the pending shell palette.
const ROWS: [&str; 18] = [
    "B",
    "BB",
    "BWB",
    "BWWB",
    "BWWWB",
    "BWWWWB",
    "BWWWWWB",
    "BWWWWWWB",
    "BWWWWWWWB",
    "BWWWWWWWWB",
    "BWWWWWWWWWB",
    "BWWWWWWBBBBB",
    "BWWBWWB",
    "BWB BWWB",
    "BB  BWWB",
    "B    BWWB",
    "     BWWB",
    "      BB",
];

/// Opaque pixels clipped in floating point before any integer conversion.
/// Malformed positions refuse even if no pixel would intersect the output.
pub(crate) fn pixels(
    cursor: &Cursor,
    bounds: Rectangle<i32, Physical>,
) -> Result<Vec<Pixel>, RenderError> {
    let Cursor::Arrow { location } = cursor else {
        return Ok(Vec::new());
    };
    // The tip **is** the hotspot for this one, so the mask goes down where the
    // pointer is with no adjustment — which is exactly the half
    // `crate::resize_cursor` does differently.
    crate::cursor_mask::painted(&ROWS, *location, bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_fractional_position_and_shape_extent() -> Result<(), RenderError> {
        let pixels = pixels(
            &Cursor::Arrow {
                location: (3.9, 4.8).into(),
            },
            Rectangle::from_size((32, 32).into()),
        )?;
        assert_eq!(pixels.first().map(|(r, _)| r.loc), Some((3, 4).into()));
        assert_eq!(pixels.iter().map(|(r, _)| r.loc.x).max(), Some(14));
        assert_eq!(pixels.iter().map(|(r, _)| r.loc.y).max(), Some(21));
        assert!(pixels.iter().all(|(r, _)| r.size == (1, 1).into()));
        Ok(())
    }

    #[test]
    fn edge_clipping_and_extreme_positions_are_bounded() -> Result<(), RenderError> {
        let bounds = Rectangle::from_size((32, 32).into());
        for location in [(31.0, 31.0), (-11.0, -17.0), (-1.5, 0.5), (0.0, -1.5)] {
            let pixels = pixels(
                &Cursor::Arrow {
                    location: location.into(),
                },
                bounds,
            )?;
            assert!(pixels.iter().all(|(rect, _)| bounds.contains_rect(*rect)));
        }
        assert_eq!(
            pixels(
                &Cursor::Arrow {
                    location: (31.0, 31.0).into()
                },
                bounds
            )?
            .len(),
            1
        );
        for value in [f64::MAX, f64::MIN, f64::from(i32::MAX), f64::from(i32::MIN)] {
            assert!(
                pixels(
                    &Cursor::Arrow {
                        location: (value, value).into()
                    },
                    bounds
                )?
                .is_empty()
            );
        }
        Ok(())
    }

    #[test]
    fn malformed_coordinates_refuse_and_non_arrow_states_add_nothing() {
        let bounds = Rectangle::from_size((32, 32).into());
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for location in [(value, 0.0), (0.0, value)] {
                assert!(
                    pixels(
                        &Cursor::Arrow {
                            location: location.into()
                        },
                        bounds
                    )
                    .is_err()
                );
            }
        }
        for cursor in [Cursor::Hidden, Cursor::Default] {
            assert!(pixels(&cursor, bounds).is_ok_and(|pixels| pixels.is_empty()));
        }
    }
}
