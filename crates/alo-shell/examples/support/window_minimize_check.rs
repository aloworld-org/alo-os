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
        if minimized {
            use alo_shortcuts::{Action, Shortcuts};
            server.keyboard_focus(Some(&root))?;
            let settings = Shortcuts::shipped();
            let chord = settings
                .chord_for(Action::MinimiseWindow)
                .ok_or("missing minimize binding")?;
            assert_eq!(
                server.dispatch_window_command(&settings, chord)?,
                Some(Action::MinimiseWindow)
            );
        } else {
            assert!(server.set_window_minimized(&root, false)?);
        }
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
