//! Real GLES evidence for stable trusted forward/backward cycling.
use alo_shell::{Cursor, Server, WindowSwitchDirection, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Select each fixture client and verify its opaque pixel through GLES readback.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let original: Vec<_> = server.mapped_surfaces().cloned().collect();
    let [first, second] = original.as_slice() else {
        return Err("switching fixture requires two mapped roots".into());
    };
    server.activate_window(first)?;
    for (direction, root, expected) in [
        (WindowSwitchDirection::Forward, second, [255, 0, 255, 0]),
        (WindowSwitchDirection::Backward, first, [0, 0, 255, 0]),
    ] {
        assert_eq!(server.switch_window(direction)?, *root);
        let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
        let scene = render_scanout(
            renderer,
            (33, 32).into(),
            &roots,
            &server.popup_surfaces(),
            &Cursor::Hidden,
        )?;
        assert_eq!(
            scene.pixels().pixels().get(136..140),
            Some(expected.as_slice())
        );
    }
    println!("Window cycling GLES pixels passed in both directions; no DRM evidence");
    Ok(())
}
