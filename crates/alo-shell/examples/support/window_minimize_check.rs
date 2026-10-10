//! A client's own minimize request, and every pixel it hides.
//!
//! **This was mostly the control strip** - pressing the minimise tile, pointer
//! release ownership, cancelling an out-and-back drag, and the tile's hover
//! feedback. The strip was retired on 2026-10-10 and all of that went with it.
//!
//! What is left is the half that was never the strip's: a client asking over
//! the wire to be hidden, 6400 pixels confirming nothing of it remains, and the
//! restore afterwards preserving its buffer.
//!
//! **Worth saying why this was not deleted with the tiles.** `window_minimize`
//! and `window_maximize` have **no unit tests at all**, so this and
//! `window_maximize_check` are the only checks of client-requested minimise,
//! maximise and restore anywhere in the repository. Deleting them with the
//! surface they also drew would have taken a guarantee out with it.

use alo_shell::{Cursor, Server, render_scanout};
use smithay::backend::renderer::gles::GlesRenderer;

/// A real wire request has hidden the preceding root; inspect every hidden pixel,
/// then restore through trusted controls and verify its preserved client buffer.
pub fn client_request(
    server: &mut Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(server.mapped_surfaces().count(), 0);
    let root = server
        .minimized_surfaces()
        .next()
        .cloned()
        .ok_or("hidden root missing")?;
    assert_eq!(server.minimized_surfaces().count(), 1);
    let frame = render_scanout(
        renderer,
        (80, 80).into(),
        &[],
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    assert_eq!(frame.pixels().pixels().len(), 6400 * 4);
    assert!(frame.pixels().pixels().iter().all(|byte| *byte == 0));
    assert!(server.set_window_minimized(&root, false)?);
    crate::window_maximize_check::stage(server, renderer, 21)?;
    println!("Client minimize and trusted restore: two full 6400-pixel frames passed");
    Ok(())
}
