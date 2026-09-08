//! Real scoped descriptor and calloop pause through the compositor loop.

use super::*;
use crate::{DirectFrame, DirectLoopError, FrameTarget, RenderError};
use smithay::{
    reexports::{
        calloop::{EventLoop, channel},
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Physical, Size},
};
use std::os::unix::fs::PermissionsExt;

/// Retirement writes through the live borrowed fd; drop must precede manager close.
struct Target<'a> {
    /// Session-owned descriptor, never duplicated.
    fd: std::os::fd::BorrowedFd<'a>,
    /// Manager observations.
    state: Rc<RefCell<State>>,
    /// Number of submitted frames.
    frames: Rc<std::cell::Cell<usize>>,
}
impl FrameTarget for Target<'_> {
    fn size(&self) -> Size<i32, Physical> {
        (20, 20).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.frames.set(self.frames.get() + 1);
        Ok(vec![])
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        assert_eq!(self.state.borrow().closes, 0);
        assert_eq!(rustix::io::write(self.fd, b"retired").unwrap(), 7);
        Ok(())
    }
}
impl crate::direct_loop::LoopTarget for Target<'_> {
    fn check(&self) -> Result<(), RenderError> {
        Ok(())
    }
}
impl Drop for Target<'_> {
    fn drop(&mut self) {
        assert_eq!(self.state.borrow().closes, 0);
        assert_eq!(rustix::io::write(self.fd, b"dropped").unwrap(), 7);
    }
}

#[test]
fn direct_loop_real_pause_retires_and_drops_before_descriptor_close() {
    for idle in [false, true] {
        let (mut device, state) = fixture();
        let mut events = EventLoop::try_new().unwrap();
        let (tx, rx) = channel::channel();
        events
            .handle()
            .insert_source(rx, |event, _, device| {
                if let channel::Event::Msg(event) = event {
                    crate::direct_session::session_event(event, device);
                }
            })
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = crate::Server::bind(dir.path(), "session-loop").unwrap();
        let frames = Rc::new(std::cell::Cell::new(0));
        let result = crate::active_session::run(&mut device, &mut events, |fd, poll| {
            let target = Target {
                fd,
                state: state.clone(),
                frames: frames.clone(),
            };
            let mut steps = 0;
            crate::direct_loop::run(&mut server, target, poll, &mut || {
                steps += 1;
                assert!(steps <= 2);
                if steps == 2 {
                    tx.send(Event::PauseSession).unwrap();
                    tx.send(Event::ActivateSession).unwrap();
                }
                if idle {
                    DirectFrame::Idle
                } else {
                    DirectFrame::Render(1)
                }
            })
        })
        .unwrap();
        assert!(matches!(
            result.outcome.outcome,
            Err(DirectLoopError::Session(SessionError::Inactive))
        ));
        result.outcome.retirement.unwrap().unwrap();
        result.outcome.flush.unwrap().unwrap();
        result.cleanup.unwrap();
        assert_eq!(frames.get(), usize::from(!idle));
        assert_eq!(state.borrow().closes, 1);
        assert!(server.presentation.output.is_none());
        let mut bytes = [0; 14];
        state
            .borrow_mut()
            .peers
            .first_mut()
            .unwrap()
            .read_exact(&mut bytes)
            .unwrap();
        assert_eq!(&bytes, b"retireddropped");
        assert_closed(&state);
    }
}
