//! GLES evidence that resize planning leaves the committed scene unchanged.
use alo_shell::{Cursor, ResizeEdge, ResizeGeometryError, Server, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Exercise every edge against a real mapped root and compare every pixel.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots
        .first()
        .ok_or("resize geometry requires a mapped root")?;
    let popups = server.popup_surfaces();
    let before = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
    use ResizeEdge::*;
    for edge in [
        Top,
        Bottom,
        Left,
        Right,
        TopLeft,
        TopRight,
        BottomLeft,
        BottomRight,
    ] {
        let geometry = server.window_resize_geometry(root, edge)?;
        let initial = geometry.requested_size((0.0, 0.0))?;
        let origin = geometry.committed_origin(initial)?;
        let requested = geometry.requested_size((8.0, -4.0))?;
        assert!(requested.0 > 0 && requested.1 > 0);
        assert_eq!(geometry.committed_origin(initial)?, origin);
        assert_eq!(
            geometry.requested_size((f64::NAN, 0.0)),
            Err(ResizeGeometryError::Delta)
        );
        let after = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
        assert_eq!(before.pixels().pixels(), after.pixels().pixels());
    }
    assert_eq!(server.mapped_surfaces().cloned().collect::<Vec<_>>(), roots);
    println!("Resize geometry: all eight edges and invalid deltas preserve all 6,400 GLES pixels");
    Ok(())
}
