//! Full-frame GLES checks through trusted maximize/restore transactions.
use alo_shell::{Cursor, Server, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Check the entire actual client buffer and unchanged background at each boundary.
pub fn stage(
    server: &mut Server,
    renderer: &mut GlesRenderer,
    stage: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots.first().ok_or("maximize root missing")?;
    let (origin, size) = match stage {
        17 => {
            assert!(server.set_window_maximized(root, true)?.is_some());
            assert!(server.set_window_maximized(root, true)?.is_none());
            ((20, 20), (32, 24))
        }
        18 => ((20, 20), (32, 24)), // Acknowledgement has no pixel effect.
        19 => {
            assert!(server.set_window_maximized(root, false)?.is_some());
            ((0, 0), (33, 32)) // The client's real output-sized buffer, no scaling.
        }
        20 => ((0, 0), (33, 32)), // Restore acknowledgement also waits for commit.
        21 => ((20, 20), (32, 24)),
        _ => return Err("unknown maximize stage".into()),
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
        assert_eq!(*pixel, expected, "maximize stage {stage} pixel {index}");
    }
    println!("Maximize/restore stage {stage}: all 6400 GLES pixels passed");
    Ok(())
}
