//! Real descriptor ownership and pinned libinput callback refusal integration.

#![allow(clippy::unwrap_used)]

use super::*;
use std::{io::Read, os::unix::net::UnixStream};

/// Observations and injected manager refusals, retained after context destruction.
#[derive(Default)]
struct State {
    /// Backend seat activity.
    active: bool,
    /// Open attempts including failures.
    opens: usize,
    /// Close attempts including failures.
    closes: usize,
    /// Inject an acquisition refusal.
    deny_open: bool,
    /// Inject a close refusal after descriptor consumption.
    deny_close: bool,
    /// Socket peers prove actual closure.
    peers: Vec<UnixStream>,
}

/// Manager stand-in with real descriptors; it grants no real device authority.
struct Manager(Rc<RefCell<State>>);

/// Preserve the manager's errno through both callback paths.
#[derive(Debug)]
struct Denied;
impl AsErrno for Denied {
    fn as_errno(&self) -> Option<i32> {
        Some(13)
    }
}

impl Session for Manager {
    type Error = Denied;
    fn open(&mut self, path: &Path, flags: rustix::fs::OFlags) -> Result<OwnedFd, Denied> {
        assert_eq!(path, Path::new("/dev/null"));
        assert!(flags.contains(
            rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOCTTY | rustix::fs::OFlags::NONBLOCK
        ));
        let mut state = self.0.borrow_mut();
        state.opens += 1;
        if state.deny_open {
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
        if state.deny_close {
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

/// Start with active authority but no acquired descriptors.
fn fixture() -> (
    SessionInput<Manager>,
    SessionInputStatus,
    Rc<RefCell<State>>,
) {
    let state = Rc::new(RefCell::new(State {
        active: true,
        ..State::default()
    }));
    let (interface, status) = SessionInput::new(Manager(state.clone()));
    (interface, status, state)
}

/// Ask for the same access mode libinput uses for evdev devices.
fn open(interface: &mut SessionInput<Manager>) -> Result<OwnedFd, i32> {
    interface.open_restricted(
        Path::new("/dev/null"),
        rustix::fs::OFlags::RDONLY.bits() as i32,
    )
}

/// All successful acquisitions have real EOF, and exactly one manager close each.
fn closed(state: &Rc<RefCell<State>>, count: usize) {
    let mut state = state.borrow_mut();
    assert_eq!(state.closes, count);
    assert_eq!(state.peers.len(), count);
    for peer in &mut state.peers {
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
    }
}

#[test]
fn session_input_multiple_devices_close_through_manager_after_pause() {
    let (mut interface, status, state) = fixture();
    assert_eq!(state.borrow().opens, 0);
    let first = open(&mut interface).unwrap();
    let second = open(&mut interface).unwrap();
    state.borrow_mut().active = false;
    interface.close_restricted(second);
    interface.close_restricted(first);
    closed(&state, 2);
    assert!(status.check().is_ok());
}

#[test]
fn session_input_inactive_refuses_without_open_then_allows_fresh_acquisition() {
    let (mut interface, status, state) = fixture();
    state.borrow_mut().active = false;
    assert_eq!(open(&mut interface).unwrap_err(), 13);
    assert_eq!(state.borrow().opens, 0);
    assert!(status.check().is_ok());
    state.borrow_mut().active = true;
    let fd = open(&mut interface).unwrap();
    interface.close_restricted(fd);
    closed(&state, 1);
}

#[test]
fn session_input_open_failure_latches_and_cleanup_cannot_erase_original_error() {
    let (mut interface, status, state) = fixture();
    let fd = open(&mut interface).unwrap();
    state.borrow_mut().deny_open = true;
    assert_eq!(open(&mut interface).unwrap_err(), 13);
    state.borrow_mut().deny_open = false;
    assert_eq!(open(&mut interface).unwrap_err(), 5);
    assert_eq!(state.borrow().opens, 2);
    state.borrow_mut().deny_close = true;
    interface.close_restricted(fd);
    closed(&state, 1);
    assert!(
        status
            .check()
            .unwrap_err()
            .to_string()
            .contains("open device")
    );
    assert!(status.clone().check().is_err());
}

#[test]
fn session_input_close_failure_is_observable_and_terminal() {
    let (mut interface, status, state) = fixture();
    let fd = open(&mut interface).unwrap();
    state.borrow_mut().deny_close = true;
    interface.close_restricted(fd);
    assert_eq!(open(&mut interface).unwrap_err(), 5);
    assert_eq!(state.borrow().opens, 1);
    assert!(
        status
            .check()
            .unwrap_err()
            .to_string()
            .contains("close device")
    );
    drop(interface);
    closed(&state, 1);
}

#[test]
fn session_input_real_libinput_rejects_non_evdev_and_closes_callback_descriptor() {
    let (interface, status, state) = fixture();
    let mut context = smithay::reexports::input::Libinput::new_from_path(interface);
    assert!(context.path_add_device("/dev/null").is_none());
    context.dispatch().unwrap();
    assert!(context.next().is_none());
    drop(context);
    assert_eq!(state.borrow().opens, 1);
    closed(&state, 1);
    assert!(status.check().is_ok());
}

#[test]
fn session_input_real_libinput_open_refusal_retains_manager_failure() {
    let (interface, status, state) = fixture();
    state.borrow_mut().deny_open = true;
    let mut context = smithay::reexports::input::Libinput::new_from_path(interface);
    assert!(context.path_add_device("/dev/null").is_none());
    assert!(status.check().is_err());
    drop(context);
    assert_eq!(state.borrow().opens, 1);
    closed(&state, 0);
}
