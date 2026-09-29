//! Minimizing cancels only the owning interactive transaction.
use super::{Fixture, mapped, minimize};
use smithay::backend::input::ButtonState;

#[test]
fn window_minimize_cancels_move_and_resize_without_late_placement()
-> Result<(), Box<dyn std::error::Error>> {
    for resize in [false, true] {
        let f = Fixture::keyboard();
        f.backend(|s| s.enable_pointer())?;
        let mut app = mapped(&f);
        let root = f.root();
        let _other = mapped(&f);
        let other = f
            .backend(|s| s.mapped_surfaces().nth(1).cloned())
            .ok_or("second missing")?;
        if resize {
            // A resize begins on the shell's own edge band, which is the road task
            // 4 built and the only one left: ADR 0071 refuses the client's request.
            // The press is still delivered to the application, because it lands
            // inside the frame — the band begin is what starts the transaction.
            f.backend(|s| s.pointer_motion(4.0, 5.0, 1))?;
            assert!(f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
            app.sync();
            let frame = root.clone();
            assert!(f.backend(move |s| s.begin_a_resize(&frame, alo_shell::FrameEdge::TopLeft)));
        } else {
            // A move begins on the band above the frame, and reaches no client.
            f.backend(|s| s.pointer_motion(4.0, -16.0, 1))?;
            assert!(!f.backend(|s| s.pointer_button(0x110, ButtonState::Pressed, 2))?);
        }
        app.sync();
        minimize(&f, &other, true)?;
        f.backend(|s| s.pointer_motion(14.0, 15.0, 3))?;
        app.sync();
        if resize {
            assert_eq!(app.events.resizing.last(), Some(&true));
        }
        let target = root.clone();
        let before = f.backend(move |_| alo_shell::window_buffer_origin(&target));
        if !resize {
            // Anchored at the band press, (4, -16), rather than at (4, 5).
            assert_eq!(before, (10.0, 31.0).into());
        }
        minimize(&f, &root, true)?;
        app.sync();
        if resize {
            assert_eq!(app.events.resizing.last(), Some(&false));
        }
        if resize {
            let serial = app.events.serial.ok_or("configure missing")?;
            app.xdg.ack_configure(serial);
        }
        app.attach_resized();
        app.sync();
        f.backend(|s| s.pointer_motion(24.0, 25.0, 4))?;
        assert!(!f.backend(|s| s.pointer_button(0x110, ButtonState::Released, 5))?);
        minimize(&f, &root, false)?;
        let target = root.clone();
        assert_eq!(
            f.backend(move |_| alo_shell::window_buffer_origin(&target)),
            before
        );
        // Neither hiding nor a stale press serial creates fresh drag authority.
        let seat = app.events.keyboard.seat.as_ref().ok_or("seat missing")?;
        app.toplevel._move(seat, app.events.pointer.button_serial);
        app.sync();
        f.backend(|s| s.pointer_motion(34.0, 35.0, 6))?;
        assert_eq!(
            f.backend(move |_| alo_shell::window_buffer_origin(&root)),
            before
        );
    }
    Ok(())
}
