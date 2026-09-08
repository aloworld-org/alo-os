//! Pause-aware scopes use real calloop channels and kernel socket descriptors.

use super::*;
use smithay::reexports::calloop::{EventLoop, channel};

#[test]
fn scoped_pause_keeps_fd_for_retirement_and_reacquires_after_activation() {
    let (mut device, state) = fixture();
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
    let result = crate::active_session::run(&mut device, &mut events, |fd, poll| {
        poll().unwrap();
        seat_tx.send(()).unwrap();
        assert!(matches!(poll(), Err(SessionError::Inactive)));
        assert!(matches!(poll(), Err(SessionError::Inactive)));
        assert_eq!(state.borrow().closes, 0);
        // The trusted retirement path still has the original live descriptor.
        rustix::io::write(fd, b"retired").unwrap();
        let mut bytes = [0; 7];
        state
            .borrow_mut()
            .peers
            .first_mut()
            .unwrap()
            .read_exact(&mut bytes)
            .unwrap();
        assert_eq!(&bytes, b"retired");
        "retirement completed"
    })
    .unwrap();
    assert_eq!(result.outcome, "retirement completed");
    result.cleanup.unwrap();
    assert_eq!(state.borrow().closes, 1);
    assert_closed(&state);
    let next = crate::active_session::run(&mut device, &mut events, |_, poll| poll()).unwrap();
    next.outcome.unwrap();
    next.cleanup.unwrap();
    assert_eq!(state.borrow().opens, 2);
    assert_eq!(state.borrow().closes, 2);
    assert_closed(&state);
}

#[test]
fn scoped_backend_inactivation_is_sticky_without_a_notification() {
    let (mut device, state) = fixture();
    let mut events = EventLoop::try_new().unwrap();
    let result = crate::active_session::run(&mut device, &mut events, |_, poll| {
        state.borrow_mut().active = false;
        assert!(matches!(poll(), Err(SessionError::Inactive)));
        state.borrow_mut().active = true;
        assert!(matches!(poll(), Err(SessionError::Inactive)));
        assert_eq!(state.borrow().closes, 0);
    })
    .unwrap();
    result.cleanup.unwrap();
    assert_closed(&state);
}

#[test]
fn scoped_close_failure_preserves_operation_error_and_refuses_reentry() {
    let (mut device, state) = fixture();
    let mut events = EventLoop::try_new().unwrap();
    state.borrow_mut().close_error = true;
    let result = crate::active_session::run(&mut device, &mut events, |_, _| {
        Err::<(), _>("scanout disable failed")
    })
    .unwrap();
    assert_eq!(result.outcome, Err("scanout disable failed"));
    assert!(
        matches!(result.cleanup, Err(SessionError::Backend { stage: "close device", source }) if source.raw_os_error() == Some(13))
    );
    let mut called = false;
    assert!(matches!(
        crate::active_session::run(&mut device, &mut events, |_, _| called = true),
        Err(SessionError::Failed)
    ));
    assert!(!called);
    drop(device);
    assert_eq!(state.borrow().closes, 1);
    assert_closed(&state);
}

#[test]
fn scoped_inactive_and_open_failure_do_not_invoke_the_caller() {
    let (mut device, state) = fixture();
    let mut events = EventLoop::try_new().unwrap();
    state.borrow_mut().active = false;
    let mut called = false;
    assert!(matches!(
        crate::active_session::run(&mut device, &mut events, |_, _| called = true),
        Err(SessionError::Inactive)
    ));
    assert!(!called);
    assert_eq!(state.borrow().opens, 0);
    state.borrow_mut().active = true;
    state.borrow_mut().open_error = true;
    let mut called = false;
    assert!(matches!(
        crate::active_session::run(&mut device, &mut events, |_, _| called = true),
        Err(SessionError::Backend {
            stage: "open device",
            ..
        })
    ));
    assert!(!called);
    assert_eq!(state.borrow().opens, 1);
    assert_eq!(state.borrow().closes, 0);
}

#[test]
fn scoped_unwinding_drops_borrowed_resources_before_closing_device() {
    struct Resource<'a>(&'a Rc<RefCell<State>>);
    impl Drop for Resource<'_> {
        fn drop(&mut self) {
            assert_eq!(self.0.borrow().closes, 0);
        }
    }
    let (mut device, state) = fixture();
    let mut events = EventLoop::try_new().unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = crate::active_session::run(&mut device, &mut events, |_, _| {
            let _resource = Resource(&state);
            std::panic::resume_unwind(Box::new("injected renderer unwind"));
        });
    }));
    assert!(panic.is_err());
    drop(device);
    assert_eq!(state.borrow().closes, 1);
    assert_closed(&state);
}

#[test]
fn scoped_notifier_failure_defers_close_and_is_terminal() {
    let (mut device, state) = fixture();
    let mut events = EventLoop::try_new().unwrap();
    let (tx, rx) = channel::channel::<()>();
    events
        .handle()
        .insert_source(rx, |event, _, device: &mut SessionDevice<Fake>| {
            if let channel::Event::Msg(()) = event {
                device.fail();
            }
        })
        .unwrap();
    let result = crate::active_session::run(&mut device, &mut events, |fd, poll| {
        tx.send(()).unwrap();
        assert!(matches!(poll(), Err(SessionError::Failed)));
        assert!(matches!(poll(), Err(SessionError::Failed)));
        assert_eq!(state.borrow().closes, 0);
        rustix::io::write(fd, b"x").unwrap();
        state
            .borrow_mut()
            .peers
            .first_mut()
            .unwrap()
            .read_exact(&mut [0])
            .unwrap();
    })
    .unwrap();
    result.cleanup.unwrap();
    assert_closed(&state);
    assert!(matches!(device.device(), Err(SessionError::Failed)));
}
