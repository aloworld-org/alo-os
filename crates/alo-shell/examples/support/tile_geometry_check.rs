//! GLES evidence that half-output planning never mutates the committed scene.
use alo_shell::{Cursor, Server, TileGeometryError, TileSide, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Compare every pixel after accepted plans and invalid actual-size calculations.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots
        .first()
        .ok_or("tile planning requires a mapped root")?;
    let popups = server.popup_surfaces();
    let before = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
    // Stage 3 already submitted the fixture's 33x32 output through Server::render.
    for (side, size, origin) in [
        (TileSide::Left, (16, 32), (0, 0)),
        (TileSide::Right, (17, 32), (16, 0)),
    ] {
        let plan = server.window_tile_geometry(root, side)?;
        assert_eq!(plan.requested_size(), size);
        assert_eq!(plan.committed_origin(size)?, origin);
        assert_eq!(plan.committed_origin((0, 32)), Err(TileGeometryError::Size));
        let after = render_scanout(renderer, (80, 80).into(), &roots, &popups, &Cursor::Hidden)?;
        assert_eq!(before.pixels().pixels(), after.pixels().pixels());
    }
    assert_eq!(server.mapped_surfaces().cloned().collect::<Vec<_>>(), roots);
    println!("Tile geometry: both halves and invalid sizes preserve all 6,400 GLES pixels");
    Ok(())
}
