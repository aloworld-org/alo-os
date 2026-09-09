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
    for (step, minimized) in [true, false, true, false, true, false]
        .into_iter()
        .enumerate()
    {
        if step == 4 {
            use alo_shell::{
                PaintedWindowControls, WindowControlPointerEvent as Event, WindowControlRelease,
                WindowControlRoute as Route,
            };
            use smithay::backend::input::ButtonState;
            let route = |server: &mut Server, position, event| {
                server.route_window_control_pointer(
                    Some(PaintedWindowControls {
                        surface: &root,
                        viewport: (120, 48),
                        origin: (3, 4),
                    }),
                    position,
                    event,
                    1,
                )
            };
            let down = Event::Button(0x110, ButtonState::Pressed);
            let up = Event::Button(0x110, ButtonState::Released);
            assert_eq!(route(server, (4.0, 5.0), down)?, Route::Consumed);
            server.cancel_window_control();
            assert_eq!(
                route(server, (4.0, 5.0), up)?,
                Route::Released(WindowControlRelease::Cancelled)
            );
            assert_eq!(server.mapped_surfaces().count(), 1);
            assert_eq!(route(server, (4.0, 5.0), down)?, Route::Consumed);
            assert_eq!(route(server, (35.0, 5.0), Event::Motion)?, Route::Consumed);
            assert_eq!(route(server, (4.0, 5.0), Event::Motion)?, Route::Consumed);
            assert_eq!(
                route(server, (4.0, 5.0), up)?,
                Route::Released(WindowControlRelease::Cancelled)
            );
            visible_after_cancellation(server, renderer)?;
            assert_eq!(route(server, (4.0, 5.0), down)?, Route::Consumed);
            assert_eq!(route(server, (5.0, 6.0), Event::Motion)?, Route::Consumed);
            assert_eq!(
                route(server, (4.0, 5.0), up)?,
                Route::Released(WindowControlRelease::Executed(
                    alo_shortcuts::Action::MinimiseWindow
                ))
            );
            assert_eq!(
                server.release_window_control((120, 48), (3, 4), (4.0, 5.0))?,
                WindowControlRelease::Unowned
            );
        } else if minimized {
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

/// Out-and-back motion must preserve the complete visible scene before release retry.
fn visible_after_cancellation(
    server: &Server,
    renderer: &mut GlesRenderer,
) -> Result<(), Box<dyn std::error::Error>> {
    let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
    assert_eq!(roots.len(), 1);
    let frame = render_scanout(
        renderer,
        (80, 80).into(),
        &roots,
        &server.popup_surfaces(),
        &Cursor::Hidden,
    )?;
    assert_eq!(frame.pixels().pixels().len(), 6400 * 4);
    for (index, pixel) in frame
        .pixels()
        .pixels()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let expected = if (20..52).contains(&(index % 80)) && (20..44).contains(&(index / 80)) {
            [255, 255, 255, 0]
        } else {
            [0; 4]
        };
        assert_eq!(*pixel, expected, "cancelled motion pixel {index}");
    }
    println!("Native out-and-back cancellation: all 6400 visible GLES pixels passed");
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
