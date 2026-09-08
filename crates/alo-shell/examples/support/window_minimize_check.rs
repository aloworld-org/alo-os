//! Full-frame GLES visibility and restoration through trusted shell operations.
use alo_shell::{Cursor, Server, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// The preceding maximize fixture leaves a white 32x24 root at (20,20).
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = server
        .mapped_surfaces()
        .next()
        .cloned()
        .ok_or("minimize root missing")?;
    for minimized in [true, false, true, false] {
        assert!(server.set_window_minimized(&root, minimized)?);
        assert!(!server.set_window_minimized(&root, minimized)?);
        assert_eq!(server.minimized_surfaces().count(), usize::from(minimized));
        let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
        let frame = render_scanout(
            renderer,
            (80, 80).into(),
            &roots,
            &server.popup_surfaces(),
            &Cursor::Hidden,
        )?;
        for (index, pixel) in frame
            .pixels()
            .pixels()
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
        {
            let x = index % 80;
            let y = index / 80;
            let expected = if !minimized && (20..52).contains(&x) && (20..44).contains(&y) {
                [255, 255, 255, 0]
            } else {
                [0; 4]
            };
            assert_eq!(*pixel, expected, "minimized={minimized} pixel {index}");
        }
        println!("Trusted minimized={minimized}: all 6400 GLES pixels passed");
    }
    Ok(())
}
