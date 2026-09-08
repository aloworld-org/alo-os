//! GLES evidence that an unanswered size suggestion preserves committed pixels.
use alo_shell::{Cursor, Server, WindowSizeError, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Compare the complete framebuffer before and after a pending size configure.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    let root = roots
        .first()
        .ok_or("sizing fixture requires a mapped root")?;
    let before = render_scanout(
        renderer,
        (33, 32).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    assert!(server.request_window_size(root, (80, 60))?.is_some());
    assert!(server.request_window_size(root, (80, 60))?.is_none());
    assert_eq!(
        server.request_window_size(root, (0, 60)),
        Err(WindowSizeError::InvalidSize)
    );
    let after = render_scanout(
        renderer,
        (33, 32).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    assert_eq!(before.pixels().pixels(), after.pixels().pixels());
    assert_eq!(server.mapped_surfaces().cloned().collect::<Vec<_>>(), roots);
    println!(
        "Pending window size preserves complete GLES framebuffer; duplicate and invalid requests checked"
    );
    Ok(())
}
