//! Real GLES evidence for stable trusted forward/backward cycling.
use alo_shell::{Cursor, Server, WindowSwitchDirection, render_scanout};
use alo_shortcuts::{Action, Changes, Chord, Key, Modifier, Modifiers, Shortcuts};
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
    // Exercise person-owned bindings through the production command bridge as
    // well as the underlying cycling operation above.
    let chord = Chord::checked(
        Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
        Key::Space,
    )
    .map_err(|_| "invalid configured fixture chord")?;
    for (action, expected) in [
        (Action::NextWindow, [255, 0, 255, 0]),
        (Action::PreviousWindow, [0, 0, 255, 0]),
    ] {
        let mut changes = Changes::none();
        changes.set(action, Some(chord));
        let settings = Shortcuts::shipped().with(changes);
        assert_eq!(
            server.dispatch_window_shortcut(&settings, chord)?,
            Some(action)
        );
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
    println!("Configured window command GLES pixels passed; raw key dispatch remains separate");
    println!("Window cycling GLES pixels passed in both directions; no DRM evidence");
    Ok(())
}
