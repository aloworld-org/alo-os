//! GLES golden pixels for the owned arrow, layered over real client content.
use alo_shell::{Cursor, Popup, RenderError, render_scanout};
use smithay::{
    backend::renderer::gles::GlesRenderer,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

/// Compare every output pixel, including transparent gaps and all four edges.
pub fn run(
    renderer: &mut GlesRenderer,
    roots: &[WlSurface],
    popups: &[Popup],
) -> Result<(), RenderError> {
    let background = render_scanout(renderer, (33, 32).into(), roots, popups, &Cursor::Hidden)?;
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
            let expected = golden(dx, dy).unwrap_or(*base);
            assert_eq!(*pixel, expected, "arrow at ({x},{y}), output pixel {index}");
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

/// Independent row-span specification of the 12 by 18 pixel golden arrow.
fn golden(x: f64, y: f64) -> Option<[u8; 4]> {
    if !(0.0..12.0).contains(&x) || !(0.0..18.0).contains(&y) {
        return None;
    }
    let (x, y) = (x as i32, y as i32);
    let (opaque, white) = match y {
        0..=10 => (x <= y, x > 0 && x < y),
        11 => (true, (1..=6).contains(&x)),
        12 => (x <= 6, matches!(x, 1 | 2 | 4 | 5)),
        13 => (x <= 2 || (4..=7).contains(&x), matches!(x, 1 | 5 | 6)),
        14 => (x <= 1 || (4..=7).contains(&x), matches!(x, 5 | 6)),
        15 => (x == 0 || (5..=8).contains(&x), matches!(x, 6 | 7)),
        16 => ((5..=8).contains(&x), matches!(x, 6 | 7)),
        17 => (matches!(x, 6 | 7), false),
        _ => (false, false),
    };
    opaque.then_some(if white { [255, 255, 255, 0] } else { [0; 4] })
}
