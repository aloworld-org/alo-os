//! Full-frame GLES boundaries for trusted tile/restore transactions.
use alo_shell::{Cursor, Server, TileSide, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Verify every pixel before proceeding to the next protocol boundary.
pub fn stage(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    stage: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots.first().ok_or("tile root missing")?;
    if stage == 23 {
        assert!(
            server
                .set_window_tiled(root, Some(TileSide::Right))?
                .is_some()
        );
        assert!(
            server
                .set_window_tiled(root, Some(TileSide::Right))?
                .is_none()
        );
    }
    let (origin, size) = match stage {
        23 | 24 => ((20, 20), (32, 24)), // Request and ack leave the old pixels.
        25 => ((16, 0), (17, 32)),       // Exact odd-width right half.
        26 => ((0, 0), (16, 32)),        // Exact left half, with no stale right pixels.
        27 => ((0, 0), (16, 32)),        // Restore ack alone cannot move the buffer.
        28 => ((20, 20), (32, 24)),
        _ => return Err("unknown tile stage".into()),
    };
    assert_eq!(
        alo_shell::window_buffer_origin(root),
        (f64::from(origin.0), f64::from(origin.1)).into()
    );
    let prepared = render_scanout(
        renderer,
        (80, 80).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    for (index, pixel) in prepared
        .pixels()
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let x = (index % 80) as i32;
        let y = (index / 80) as i32;
        let expected = if (origin.0..origin.0 + size.0).contains(&x)
            && (origin.1..origin.1 + size.1).contains(&y)
        {
            [255, 255, 255, 0]
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "tile stage {stage} pixel {index}");
    }
    if stage == 25 {
        server.set_window_tiled(root, Some(TileSide::Left))?;
    }
    if stage == 26 {
        server.set_window_tiled(root, None)?;
    }
    println!("Tile/restore stage {stage}: all 6400 GLES pixels passed");
    Ok(())
}
