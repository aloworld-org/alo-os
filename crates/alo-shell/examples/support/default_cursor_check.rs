//! GLES golden pixels for alo's own arrow, layered over real client content.
//!
//! **What the golden is now.** Until 2026-11-06 it was an independent row-span
//! specification of an 18-row black and white mask. The arrow is now drawn by
//! `alo_cursors` from a vector source, anti-aliased, with a white keyline and a
//! soft shadow, so the golden is that picture laid over the frame behind it,
//! premultiplied, the way both renderers blend. What this checks is therefore
//! the road from picture to screen — the hotspot on the position, the clipping
//! at all four edges, the blending — and `alo_cursors`' own tests check the
//! picture.
use alo_shell::{Cursor, Popup, RenderError, render_scanout};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// How far a blended channel may differ from the arithmetic, in levels: a GPU
/// blends in floating point and rounds once, which this crate's integers do
/// not reproduce exactly.
pub const BLENDING_ROUNDS: u8 = 2;

/// Compare every output pixel, including transparent gaps and all four edges.
pub fn run(
    renderer: &mut GlesRenderer,
    roots: &[WlSurface],
    popups: &[Popup],
) -> Result<(), RenderError> {
    let background = render_scanout(renderer, (33, 32).into(), roots, popups, &Cursor::Hidden)?;
    let arrow = the_arrow()?;
    for (x, y) in [
        (10.0, 14.0),
        (31.0, 31.0),
        (-4.0, -5.0),
        (3.9, 4.8),
        (32.0, -3.0),
        (-1.5, 25.0),
        (f64::MAX, 0.0),
    ] {
        let prepared = render_scanout(
            renderer,
            (33, 32).into(),
            roots,
            popups,
            &Cursor::Arrow {
                location: (x, y).into(),
            },
        )?;
        assert_eq!(prepared.surfaces(), background.surfaces());
        for (index, (pixel, base)) in prepared
            .pixels()
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(background.pixels().pixels().as_chunks::<4>().0)
            .enumerate()
        {
            let dx = (index % 33) as f64 - x.floor();
            let dy = (index / 33) as f64 - y.floor();
            let expected = golden(&arrow, dx, dy, *base);
            assert!(
                close(*pixel, expected),
                "arrow at ({x},{y}), output pixel {index}: {pixel:?} against {expected:?}"
            );
        }
    }
    for location in [(f64::NAN, 0.0), (0.0, f64::INFINITY)] {
        assert!(
            render_scanout(
                renderer,
                (33, 32).into(),
                roots,
                popups,
                &Cursor::Arrow {
                    location: location.into()
                }
            )
            .is_err()
        );
    }
    // Switching away (also after refusal) cannot retain arrow pixels or identities.
    for cursor in [Cursor::Hidden, Cursor::Default] {
        let clean = render_scanout(renderer, (33, 32).into(), roots, popups, &cursor)?;
        assert_eq!(clean.pixels().pixels(), background.pixels().pixels());
        assert_eq!(clean.surfaces(), background.surfaces());
    }
    Ok(())
}

/// alo's arrow as a display at one-to-one draws it.
pub fn the_arrow() -> Result<alo_cursors::Picture, RenderError> {
    alo_cursors::draw(
        alo_cursors::CursorIcon::Default,
        alo_cursors::Silhouette::default(),
        alo_cursors::measures::pixels_at(100),
    )
    .map_err(|why| RenderError::Submission(why.to_string()))
}

/// The output pixel `(dx, dy)` from the pointer's position should be: the
/// arrow's own pixel there laid over `base`, or `base` where the arrow is not.
///
/// Pixels are DRM's B, G, R, X; the fourth byte is not compared.
pub fn golden(arrow: &alo_cursors::Picture, dx: f64, dy: f64, base: [u8; 4]) -> [u8; 4] {
    let x = dx + f64::from(arrow.hotspot.0);
    let y = dy + f64::from(arrow.hotspot.1);
    let size = f64::from(arrow.size);
    if !(0.0..size).contains(&x) || !(0.0..size).contains(&y) {
        return base;
    }
    let index = ((y as usize) * arrow.size as usize + x as usize) * 4;
    let Some(&[red, green, blue, alpha]) = arrow.pixels.get(index..index + 4) else {
        return base;
    };
    let over = |ink: u8, under: u8| {
        let kept = u16::from(under) * (255 - u16::from(alpha)) / 255;
        u8::try_from(u16::from(ink) + kept).unwrap_or(u8::MAX)
    };
    [
        over(blue, base[0]),
        over(green, base[1]),
        over(red, base[2]),
        base[3],
    ]
}

/// Whether two output pixels agree in colour, within rounding.
pub fn close(pixel: [u8; 4], expected: [u8; 4]) -> bool {
    pixel
        .iter()
        .zip(expected)
        .take(3)
        .all(|(one, other)| one.abs_diff(other) <= BLENDING_ROUNDS)
}
