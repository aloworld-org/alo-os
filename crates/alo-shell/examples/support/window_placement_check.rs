//! Independent multi-window pixel expectations through the GLES scene painter.
use alo_shell::{Cursor, Server, render_scanout, window_buffer_origin};
use smithay::backend::renderer::gles::GlesRenderer;

/// Move one root and its descendants while the second root remains stationary.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let [root, stationary] = roots.as_slice() else {
        return Err("placement fixture requires two mapped roots".into());
    };
    let popups = server.popup_surfaces();
    let before = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
    // Fixture root has committed geometry (2,3), initially buffer-origin zero.
    assert_eq!(window_buffer_origin(root), (0.0, 0.0).into());
    assert_eq!(window_buffer_origin(stationary), (0.0, 0.0).into());
    verify(before.pixels().pixels(), (0, 0));
    for (position, delta) in [((7, 9), (5, 6)), ((-3, -3), (-5, -6)), ((77, 77), (75, 74))] {
        server.place_window(root, position)?;
        let after = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
        verify(after.pixels().pixels(), delta);
        assert_eq!(window_buffer_origin(stationary), (0.0, 0.0).into());
        assert_eq!(server.mapped_surfaces().cloned().collect::<Vec<_>>(), roots);
    }
    server.place_window(root, (2, 3))?;
    let restored = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
    assert_eq!(before.pixels().pixels(), restored.pixels().pixels());
    assert_eq!(server.mapped_surfaces().cloned().collect::<Vec<_>>(), roots);
    println!(
        "Native placement GLES pixels: root/child/popup movement, stationary second window, all-edge clipping and restoration passed"
    );
    Ok(())
}

/// Analytic opaque fixture layers, independent of production placement helpers.
fn verify(pixels: &[u8], delta: (i32, i32)) {
    assert_eq!(pixels.len(), 80 * 80 * 4);
    for (index, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
        let (screen_x, screen_y) = (index as i32 % 80, index as i32 / 80);
        let (x, y) = (screen_x - delta.0, screen_y - delta.1);
        let expected = if (8..24).contains(&x) && (12..28).contains(&y) {
            [255, 0, 0, 0] // blue popup above the moving root
        } else if (24..40).contains(&x) && (4..20).contains(&y) {
            [0, 255, 255, 0] // yellow subsurface moves with its root
        } else if (0..16).contains(&x) && (0..16).contains(&y) {
            if y < 8 {
                [0, 0, 255, 0] // red upper half of the moving root
            } else {
                [0, 255, 0, 0] // green lower half
            }
        } else if screen_x < 16 && screen_y < 16 {
            [255, 0, 255, 0] // magenta second root does not move
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "pixel {index}, delta {delta:?}");
    }
}
