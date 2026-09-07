//! Injected session failures with real kernel descriptor lifetimes.

#![allow(clippy::unwrap_used)]

use super::*;
use std::{cell::RefCell, io::Read, os::unix::net::UnixStream, rc::Rc};

/// Shared observations survive destruction of the device owner.
#[derive(Default)]
struct State {
    /// Backend activation state, independently changeable by a test.
    active: bool,
    /// Acquisition attempts.
    opens: usize,
    /// Manager close attempts.
    closes: usize,
    /// Inject permission failure.
    open_error: bool,
    /// Inject close failure after consuming the descriptor.
    close_error: bool,
    /// Socket peers prove actual descriptor closure via EOF.
    peers: Vec<UnixStream>,
}

/// Error retaining a real errno through Smithay's trait.
#[derive(Debug)]
struct Denied;
impl AsErrno for Denied {
    fn as_errno(&self) -> Option<i32> {
        Some(13)
    }
}

/// Fake manager only; the owned descriptors are real Unix sockets.
struct Fake(Rc<RefCell<State>>);
impl Session for Fake {
    type Error = Denied;
    fn open(
        &mut self,
        path: &std::path::Path,
        flags: rustix::fs::OFlags,
    ) -> Result<OwnedFd, Denied> {
        assert_eq!(path, std::path::Path::new("/dev/dri/card0"));
        assert!(flags.contains(rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOCTTY));
        let mut state = self.0.borrow_mut();
        state.opens += 1;
        if state.open_error {
            return Err(Denied);
        }
        let (fd, peer) = UnixStream::pair().unwrap();
        peer.set_nonblocking(true).unwrap();
        state.peers.push(peer);
        Ok(fd.into())
    }
    fn close(&mut self, fd: OwnedFd) -> Result<(), Denied> {
        drop(fd);
        let mut state = self.0.borrow_mut();
        state.closes += 1;
        if state.close_error {
            Err(Denied)
        } else {
            Ok(())
        }
    }
    fn change_vt(&mut self, _: i32) -> Result<(), Denied> {
        Err(Denied)
    }
    fn is_active(&self) -> bool {
        self.0.borrow().active
    }
    fn seat(&self) -> String {
        "seat-test".into()
    }
}

/// Active manager and lazy device owner.
fn fixture() -> (SessionDevice<Fake>, Rc<RefCell<State>>) {
    let state = Rc::new(RefCell::new(State {
        active: true,
        ..State::default()
    }));
    (
        SessionDevice::new(Fake(state.clone()), "/dev/dri/card0".into()),
        state,
    )
}

/// Every acquired descriptor has really closed, not just lost its tracking entry.
fn assert_closed(state: &Rc<RefCell<State>>) {
    for peer in &mut state.borrow_mut().peers {
        assert_eq!(peer.read(&mut [0_u8]).unwrap(), 0);
    }
}

#[test]
fn active_access_reuses_device_and_drop_closes_once() {
    let (mut device, state) = fixture();
    assert_eq!(state.borrow().opens, 0);
    device.device().unwrap();
    device.device().unwrap();
    assert_eq!(state.borrow().opens, 1);
    assert_eq!(
        state
            .borrow_mut()
            .peers
            .first_mut()
            .unwrap()
            .read(&mut [0])
            .unwrap_err()
            .kind(),
        io::ErrorKind::WouldBlock
    );
    drop(device);
    assert_eq!(state.borrow().closes, 1);
    assert_closed(&state);
}

#[test]
fn pause_activate_batch_retires_old_device_before_reacquisition() {
    let (mut device, state) = fixture();
    device.device().unwrap();
    device.event(Event::PauseSession).unwrap();
    device.event(Event::PauseSession).unwrap();
    assert!(matches!(device.device(), Err(SessionError::Inactive)));
    assert_closed(&state);
    device.event(Event::ActivateSession).unwrap();
    device.device().unwrap();
    assert_eq!(state.borrow().opens, 2);
    device.shutdown().unwrap();
    drop(device);
    assert_eq!(state.borrow().closes, 2);
    assert_closed(&state);
}

#[test]
fn initially_inactive_never_opens_until_both_states_activate() {
    let state = Rc::new(RefCell::new(State::default()));
    let mut device = SessionDevice::new(Fake(state.clone()), "/dev/dri/card0".into());
    assert!(matches!(device.device(), Err(SessionError::Inactive)));
    device.event(Event::ActivateSession).unwrap();
    assert!(matches!(device.device(), Err(SessionError::Inactive)));
    assert_eq!(state.borrow().opens, 0);
    state.borrow_mut().active = true;
    device.device().unwrap();
}

#[test]
fn backend_inactivation_closes_even_without_pause_notification() {
    let (mut device, state) = fixture();
    device.device().unwrap();
    state.borrow_mut().active = false;
    assert!(matches!(device.device(), Err(SessionError::Inactive)));
    assert_closed(&state);
}

#[test]
fn acquisition_failure_retains_errno_and_never_retries_implicitly() {
    let (mut device, state) = fixture();
    state.borrow_mut().open_error = true;
    let error = device.device().unwrap_err();
    assert!(
        matches!(error, SessionError::Backend { stage: "open device", source } if source.raw_os_error() == Some(13))
    );
    device.event(Event::ActivateSession).unwrap();
    assert!(matches!(device.device(), Err(SessionError::Failed)));
    drop(device);
    assert_eq!(state.borrow().opens, 1);
    assert_eq!(state.borrow().closes, 0);
}

#[test]
fn close_failure_is_terminal_and_still_consumes_descriptor() {
    let (mut device, state) = fixture();
    device.device().unwrap();
    state.borrow_mut().close_error = true;
    assert!(
        matches!(device.event(Event::PauseSession), Err(SessionError::Backend { stage: "close device", source }) if source.raw_os_error() == Some(13))
    );
    assert_closed(&state);
    device.event(Event::ActivateSession).unwrap();
    assert!(matches!(device.device(), Err(SessionError::Failed)));
    drop(device);
    assert_eq!(state.borrow().closes, 1);
}

#[test]
fn notifier_loss_retires_device_and_cannot_be_reactivated() {
    let (mut device, state) = fixture();
    device.device().unwrap();
    device.fail();
    assert_closed(&state);
    device.event(Event::ActivateSession).unwrap();
    assert!(matches!(device.device(), Err(SessionError::Failed)));
    drop(device);
    assert_eq!(state.borrow().closes, 1);
}

#[test]
fn explicit_shutdown_reports_close_failure_without_double_close() {
    let (mut device, state) = fixture();
    device.device().unwrap();
    state.borrow_mut().close_error = true;
    assert!(matches!(
        device.shutdown(),
        Err(SessionError::Backend { .. })
    ));
    assert!(matches!(device.device(), Err(SessionError::Failed)));
    drop(device);
    assert_eq!(state.borrow().closes, 1);
    assert_closed(&state);
}

#[test]
fn event_loop_drains_forwarded_pause_before_access_and_preserves_close_error() {
    use smithay::reexports::calloop::{EventLoop, channel};
    let (mut device, state) = fixture();
    device.device().unwrap();
    state.borrow_mut().close_error = true;
    let mut events = EventLoop::try_new().unwrap();
    let (seat_tx, seat_rx) = channel::channel::<()>();
    let (notify_tx, notify_rx) = channel::channel();
    events
        .handle()
        .insert_source(seat_rx, move |event, _, _| {
            if let channel::Event::Msg(()) = event {
                notify_tx.send(Event::PauseSession).unwrap();
                notify_tx.send(Event::ActivateSession).unwrap();
            }
        })
        .unwrap();
    events
        .handle()
        .insert_source(notify_rx, |event, _, device| {
            if let channel::Event::Msg(event) = event {
                crate::direct_session::session_event(event, device);
            }
        })
        .unwrap();
    seat_tx.send(()).unwrap();
    assert!(
        matches!(crate::direct_session::dispatch_session(&mut events, &mut device),
        Err(SessionError::Backend { stage: "close device", source }) if source.raw_os_error() == Some(13))
    );
    assert_closed(&state);
    assert!(matches!(device.device(), Err(SessionError::Failed)));
    assert!(matches!(
        crate::direct_session::dispatch_session(&mut events, &mut device),
        Err(SessionError::Failed)
    ));
    assert_eq!(state.borrow().opens, 1);
    assert_eq!(state.borrow().closes, 1);
}
