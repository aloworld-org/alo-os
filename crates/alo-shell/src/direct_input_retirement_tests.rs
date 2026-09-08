//! Real seat state must be cleared before even a failing output retirement.
use super::*;
use crate::surfaces::Surfaces;
use smithay::{
    backend::input::KeyState,
    input::keyboard::{FilterResult, KeyboardHandle, XkbConfig},
    reexports::wayland_server::{
        Client, DataInit, Dispatch, Display, DisplayHandle, Resource,
        protocol::wl_surface::{self, WlSurface},
    },
    utils::{Physical, SERIAL_COUNTER, Size},
};
use std::{cell::Cell, os::unix::fs::PermissionsExt, rc::Rc, sync::Arc};

// Identity-only resource: no compositor dispatch or mapped window is simulated.
// Client delivery is separately verified by tests/direct_keyboard.
struct Identity;
impl Dispatch<WlSurface, ()> for Identity {
    fn request(
        _: &mut Self,
        _: &Client,
        surface: &WlSurface,
        _: wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        surface.post_error(0u32, "identity fixture does not accept requests");
    }
}

struct Retirement {
    keyboard: KeyboardHandle<Surfaces>,
    calls: Rc<Cell<usize>>,
    submissions: Rc<Cell<usize>>,
    refuse: bool,
}
impl FrameTarget for Retirement {
    fn size(&self) -> Size<i32, Physical> {
        (32, 32).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.submissions.set(self.submissions.get() + 1);
        Err(RenderError::Submission("unexpected submission".into()))
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        assert!(self.keyboard.pressed_keys().is_empty());
        assert!(self.keyboard.current_focus().is_none());
        self.calls.set(self.calls.get() + 1);
        if self.refuse {
            Err(RenderError::Submission(
                "injected retirement refusal".into(),
            ))
        } else {
            Ok(())
        }
    }
}
impl LoopTarget for Retirement {
    fn check(&self) -> Result<(), RenderError> {
        Ok(())
    }
}

#[test]
fn direct_input_retirement_precedes_disable_on_stop_and_pause_even_when_disable_refuses()
-> Result<(), Box<dyn std::error::Error>> {
    for pause in [false, true] {
        for refuse in [false, true] {
            let dir = tempfile::tempdir()?;
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
            let mut server = Server::bind_keyboard(dir.path(), "cleanup", XkbConfig::default())?;
            let display = Display::<Identity>::new()?;
            let mut handle = display.handle();
            let (socket, _peer) = std::os::unix::net::UnixStream::pair()?;
            let client = handle.insert_client(socket, Arc::new(()))?;
            let surface = client.create_resource::<WlSurface, (), Identity>(&handle, 1, ())?;
            let keyboard = server
                .surfaces
                .keyboard
                .as_ref()
                .ok_or("missing seat")?
                .seat
                .get_keyboard()
                .ok_or("missing keyboard")?;
            // Seed XKB through its real input path. The loop must not dispatch
            // (which would prune this identity-only focus) before cleaning up.
            keyboard.set_focus(
                &mut server.surfaces,
                Some(surface),
                SERIAL_COUNTER.next_serial(),
            );
            keyboard.input::<(), _>(
                &mut server.surfaces,
                50u32.into(),
                KeyState::Pressed,
                SERIAL_COUNTER.next_serial(),
                1,
                |_, _, _| FilterResult::Forward,
            );
            assert_eq!(keyboard.pressed_keys().len(), 1);
            let calls = Rc::new(Cell::new(0));
            let submissions = Rc::new(Cell::new(0));
            let result = run(
                &mut server,
                Retirement {
                    keyboard,
                    calls: calls.clone(),
                    submissions: submissions.clone(),
                    refuse,
                },
                &mut || {
                    if pause {
                        Err(SessionError::Inactive)
                    } else {
                        Ok(())
                    }
                },
                &mut || {
                    assert!(!pause);
                    DirectFrame::Stop
                },
            );
            assert_eq!(result.outcome.is_err(), pause);
            assert_eq!(
                result.retirement.ok_or("missing retirement")?.is_err(),
                refuse
            );
            result.flush.ok_or("missing flush")??;
            assert_eq!(calls.get(), 1);
            assert_eq!(submissions.get(), 0);
        }
    }
    Ok(())
}
