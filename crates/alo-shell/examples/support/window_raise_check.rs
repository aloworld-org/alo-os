//! Pixel evidence for production stacking changes, including popup subtrees.
use alo_shell::{Cursor, Server, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// Raise the opaque magenta second client, then restore the first popup tree.
pub fn run(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let original: Vec<_> = server.mapped_surfaces().cloned().collect();
    let [first, second] = original.as_slice() else {
        return Err("raising fixture requires two mapped roots".into());
    };
    for (root, expected) in [
        (second, [[255, 0, 255, 0], [255, 0, 255, 0]]),
        (first, [[0, 0, 255, 0], [255, 0, 0, 0]]),
    ] {
        server.raise_window(root)?;
        let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
        let scene = render_scanout(
            renderer,
            (33, 32).into(),
            &roots,
            &server.popup_surfaces(),
            &Cursor::Hidden,
        )?;
        for ((x, y), expected) in [(1, 1), (9, 13)].into_iter().zip(expected) {
            let offset = (y * 33 + x) * 4;
            assert_eq!(
                scene.pixels().pixels().get(offset..offset + 4),
                Some(expected.as_slice())
            );
        }
    }
    println!(
        "Window raising GLES pixels passed: second root occludes first root/popup, restoring parent restores popup; no DRM evidence"
    );
    Ok(())
}
