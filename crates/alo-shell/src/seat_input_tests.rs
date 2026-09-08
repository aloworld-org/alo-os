//! Deterministic lifecycle failures plus real libinput empty-seat integration.
#![allow(clippy::unwrap_used)]

use super::*;
use smithay::backend::session::AsErrno;
use std::{cell::RefCell, collections::VecDeque, os::fd::OwnedFd, path::Path, rc::Rc};

struct Fake {
    log: Rc<RefCell<Vec<&'static str>>>,
    events: VecDeque<u8>,
    refuse: bool,
    cleanup: Option<(SessionInput<Manager>, OwnedFd)>,
}
impl Iterator for Fake {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        self.events.pop_front()
    }
}
impl Context for Fake {
    fn dispatch(&mut self) -> io::Result<()> {
        self.log.borrow_mut().push("dispatch");
        if self.refuse {
            Err(io::Error::other("dispatch refused"))
        } else {
            Ok(())
        }
    }
    fn suspend(&mut self) {
        self.log.borrow_mut().push("suspend");
        if let Some((mut interface, fd)) = self.cleanup.take() {
            smithay::reexports::input::LibinputInterface::close_restricted(&mut interface, fd);
        }
    }
}
impl Drop for Fake {
    fn drop(&mut self) {
        self.log.borrow_mut().push("drop");
    }
}
fn fixture(refuse: bool) -> (Lifetime<Fake>, Rc<RefCell<Vec<&'static str>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    (
        Lifetime {
            context: Some(Fake {
                log: log.clone(),
                events: [1, 2].into(),
                refuse,
                cleanup: None,
            }),
            status: SessionInputStatus::default(),
        },
        log,
    )
}

#[test]
fn seat_input_polls_before_dispatch_and_every_delivery() {
    let (mut owner, log) = fixture(false);
    let mut received = Vec::new();
    owner
        .dispatch(
            || {
                log.borrow_mut().push("poll");
                Ok(())
            },
            |event| {
                received.push(*event.unwrap());
                log.borrow_mut().push("event");
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(received, [1, 2]);
    assert_eq!(
        *log.borrow(),
        ["poll", "dispatch", "poll", "event", "poll", "event", "poll"]
    );
    drop(owner);
    assert!(log.borrow().ends_with(&["suspend", "drop"]));
}

#[test]
fn seat_input_pause_discards_queued_events_and_cannot_resume() {
    let (mut owner, log) = fixture(false);
    let mut polls = 0;
    let mut received = Vec::new();
    let error = owner
        .dispatch(
            || {
                polls += 1;
                if polls == 3 {
                    Err(io::Error::other("paused"))
                } else {
                    Ok(())
                }
            },
            |event| {
                received.push(event.copied());
                if event.is_none() {
                    assert!(log.borrow().ends_with(&["suspend", "drop"]));
                }
                Ok(())
            },
        )
        .unwrap_err();
    assert_eq!(error.source.to_string(), "paused");
    assert_eq!(received, [Some(1), None]);
    assert!(
        owner
            .dispatch(|| unreachable!(), |_| unreachable!())
            .is_err()
    );
    drop(owner);
    assert_eq!(*log.borrow(), ["dispatch", "suspend", "drop"]);
}

#[test]
fn seat_input_dispatch_and_handler_refusal_retire_before_reset() {
    for refuse_dispatch in [true, false] {
        let (mut owner, log) = fixture(refuse_dispatch);
        let mut resets = 0;
        let error = owner
            .dispatch(
                || Ok(()),
                |event| {
                    if event.is_some() {
                        return Err(io::Error::other("handler refused"));
                    }
                    resets += 1;
                    assert!(log.borrow().ends_with(&["suspend", "drop"]));
                    Ok(())
                },
            )
            .unwrap_err();
        assert_eq!(resets, 1);
        assert_eq!(
            error.source.to_string(),
            if refuse_dispatch {
                "dispatch refused"
            } else {
                "handler refused"
            }
        );
    }
}

#[test]
fn seat_input_initial_poll_refusal_never_dispatches_and_preserves_reset_error() {
    let (mut owner, log) = fixture(false);
    let error = owner
        .dispatch(
            || Err(io::Error::other("seat refused")),
            |event| {
                assert!(event.is_none());
                Err(io::Error::other("reset refused"))
            },
        )
        .unwrap_err();
    assert_eq!(error.source.to_string(), "seat refused");
    assert!(error.cleanup.unwrap().to_string().contains("reset refused"));
    assert_eq!(*log.borrow(), ["suspend", "drop"]);
}

struct Manager {
    active: bool,
    seat: String,
}
#[derive(Debug)]
struct Denied;
impl AsErrno for Denied {
    fn as_errno(&self) -> Option<i32> {
        Some(13)
    }
}
impl Session for Manager {
    type Error = Denied;
    fn open(&mut self, _: &Path, _: rustix::fs::OFlags) -> Result<OwnedFd, Denied> {
        Err(Denied)
    }
    fn close(&mut self, fd: OwnedFd) -> Result<(), Denied> {
        drop(fd);
        Err(Denied)
    }
    fn change_vt(&mut self, _: i32) -> Result<(), Denied> {
        Err(Denied)
    }
    fn is_active(&self) -> bool {
        self.active
    }
    fn seat(&self) -> String {
        self.seat.clone()
    }
}

#[test]
fn seat_input_invalid_seat_and_inactivity_refuse_before_upstream() {
    for (active, seat) in [(false, "seat0"), (true, ""), (true, "seat\0bad")] {
        let error = SeatInput::new(Manager {
            active,
            seat: seat.into(),
        })
        .err()
        .unwrap();
        assert_eq!(error.source.kind(), io::ErrorKind::InvalidInput);
        assert!(error.cleanup.is_none());
    }
}

#[test]
fn seat_input_callback_failure_blocks_delivery_and_close_error_survives_drop() {
    use smithay::reexports::input::LibinputInterface;
    use std::{io::Read, os::unix::net::UnixStream};
    for fail_open in [true, false] {
        let (mut owner, log) = fixture(false);
        let (mut interface, status) = SessionInput::new(Manager {
            active: true,
            seat: "seat-test".into(),
        });
        if fail_open {
            assert!(
                interface
                    .open_restricted(Path::new("/dev/null"), 0)
                    .is_err()
            );
        }
        let (fd, mut peer) = UnixStream::pair().unwrap();
        peer.set_nonblocking(true).unwrap();
        owner.context.as_mut().unwrap().cleanup = Some((interface, fd.into()));
        owner.status = status.clone();
        let mut resets = 0;
        let error = owner
            .dispatch(
                || {
                    if fail_open {
                        Ok(())
                    } else {
                        Err(io::Error::other("paused"))
                    }
                },
                |event| {
                    assert!(event.is_none());
                    resets += 1;
                    Ok(())
                },
            )
            .unwrap_err();
        assert_eq!(resets, 1);
        assert!(error.source.to_string().contains(if fail_open {
            "open device"
        } else {
            "paused"
        }));
        assert!(error.cleanup.unwrap().to_string().contains(if fail_open {
            "open device"
        } else {
            "close device"
        }));
        assert_eq!(peer.read(&mut [0]).unwrap(), 0);
        drop(owner);
        assert!(status.check().is_err());
        assert_eq!(*log.borrow(), ["suspend", "drop"]);
    }
}

#[test]
fn seat_input_real_libinput_empty_seat_dispatch_and_shutdown() {
    let mut owner = SeatInput::new(Manager {
        active: true,
        seat: format!("alo-test-empty-seat-{}", std::process::id()),
    })
    .unwrap();
    let mut polls = 0;
    owner
        .dispatch(
            || {
                polls += 1;
                Ok(())
            },
            |_| unreachable!(),
        )
        .unwrap();
    assert_eq!(polls, 2);
    let mut resets = 0;
    owner.shutdown(|| resets += 1).unwrap();
    assert_eq!(resets, 1);
}

#[test]
fn seat_input_real_libinput_pause_retires_empty_batch_once() {
    let mut owner = SeatInput::new(Manager {
        active: true,
        seat: format!("alo-test-paused-seat-{}", std::process::id()),
    })
    .unwrap();
    let mut polls = 0;
    let mut resets = 0;
    let error = owner
        .dispatch(
            || {
                polls += 1;
                if polls == 2 {
                    Err(io::Error::other("paused after dispatch"))
                } else {
                    Ok(())
                }
            },
            |event| {
                assert!(matches!(event, InputUpdate::Reset));
                resets += 1;
                Ok(())
            },
        )
        .unwrap_err();
    assert_eq!(error.source.to_string(), "paused after dispatch");
    assert!(error.cleanup.is_none());
    owner.shutdown(|| resets += 1).unwrap();
    assert_eq!(resets, 1);
}
