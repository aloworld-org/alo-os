//! Loop ordering and real empty-seat integration without claiming physical input.
use super::*;
use crate::{FrameTarget, RenderError, direct_loop::LoopTarget};
use smithay::{
    backend::session::{Event, Session},
    reexports::{
        calloop::{EventLoop, channel},
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Physical, Size},
};
use std::{
    cell::RefCell,
    os::{
        fd::OwnedFd,
        unix::{fs::PermissionsExt, net::UnixStream},
    },
    path::Path,
    rc::Rc,
};

type Log = Rc<RefCell<Vec<&'static str>>>;

struct Target {
    log: Log,
    refuse: bool,
}
impl FrameTarget for Target {
    fn size(&self) -> Size<i32, Physical> {
        (32, 32).into()
    }
    fn submit(&mut self, _: &[WlSurface]) -> Result<Vec<WlSurface>, RenderError> {
        self.log.borrow_mut().push("render");
        Ok(vec![])
    }
    fn retire(&mut self) -> Result<(), RenderError> {
        self.log.borrow_mut().push("retire");
        if self.refuse {
            Err(RenderError::Submission("disable refused".into()))
        } else {
            Ok(())
        }
    }
}
impl LoopTarget for Target {
    fn check(&self) -> Result<(), RenderError> {
        Ok(())
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        self.log.borrow_mut().push("target dropped");
    }
}

struct Input {
    log: Log,
    refuse: bool,
}
impl LoopInput for Input {
    fn dispatch(
        &mut self,
        _: &mut Server,
        _: &mut dyn FnMut() -> Result<(), SessionError>,
    ) -> Result<(), DirectLoopError> {
        self.log.borrow_mut().push("input");
        if self.refuse {
            Err(crate::InputDispatchError {
                source: io::Error::other("dispatch refused"),
                cleanup: Some(io::Error::other("callback refused")),
            }
            .into())
        } else {
            Ok(())
        }
    }
    fn shutdown(self, _: &mut Server) -> Option<io::Result<()>> {
        self.log.borrow_mut().push("input closed");
        Some(if self.refuse {
            Err(io::Error::other("close refused"))
        } else {
            Ok(())
        })
    }
}

#[test]
fn direct_input_loop_idle_render_stop_and_independent_failures()
-> Result<(), Box<dyn std::error::Error>> {
    for refuse in [false, true] {
        let dir = tempfile::tempdir()?;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
        let mut server = Server::bind(dir.path(), "input-loop")?;
        let log = Log::default();
        let mut steps = [DirectFrame::Idle, DirectFrame::Render(1), DirectFrame::Stop].into_iter();
        let result = crate::direct_loop::run_with_input(
            &mut server,
            Target {
                log: log.clone(),
                refuse,
            },
            &mut || Ok(()),
            &mut || steps.next().unwrap_or(DirectFrame::Stop),
            Input {
                log: log.clone(),
                refuse,
            },
        );
        assert_eq!(result.outcome.is_err(), refuse);
        assert_eq!(
            result
                .input_cleanup
                .ok_or("missing input cleanup")?
                .is_err(),
            refuse
        );
        result.input_flush.ok_or("missing input flush")??;
        assert_eq!(
            result.retirement.ok_or("missing retirement")?.is_err(),
            refuse
        );
        result.flush.ok_or("missing output flush")??;
        if refuse {
            assert!(matches!(
                result.outcome,
                Err(DirectLoopError::Input(crate::InputDispatchError {
                    cleanup: Some(_),
                    ..
                }))
            ));
            assert_eq!(
                *log.borrow(),
                ["input", "input closed", "retire", "target dropped"]
            );
        } else {
            assert_eq!(
                *log.borrow(),
                [
                    "input",
                    "input",
                    "render",
                    "input closed",
                    "retire",
                    "target dropped"
                ]
            );
        }
    }
    Ok(())
}

/// Real descriptors for the display scope; a deliberately nonexistent input seat.
#[derive(Clone)]
struct Manager {
    log: Log,
    peers: Rc<RefCell<Vec<UnixStream>>>,
}
impl Session for Manager {
    type Error = ();
    fn open(&mut self, _: &Path, _: rustix::fs::OFlags) -> Result<OwnedFd, Self::Error> {
        let (fd, peer) = UnixStream::pair().map_err(|_| ())?;
        self.peers.borrow_mut().push(peer);
        Ok(fd.into())
    }
    fn close(&mut self, fd: OwnedFd) -> Result<(), Self::Error> {
        self.log.borrow_mut().push("display closed");
        drop(fd);
        Ok(())
    }
    fn change_vt(&mut self, _: i32) -> Result<(), Self::Error> {
        Err(())
    }
    fn is_active(&self) -> bool {
        true
    }
    fn seat(&self) -> String {
        format!("alo-loop-empty-seat-{}", std::process::id())
    }
}

#[test]
fn direct_input_loop_real_empty_seat_latched_pause_and_descriptor_order()
-> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;
    for pause in [false, true] {
        let dir = tempfile::tempdir()?;
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
        let mut server = Server::bind(dir.path(), "real-input-loop")?;
        let log = Log::default();
        let manager = Manager {
            log: log.clone(),
            peers: Rc::default(),
        };
        let mut device =
            crate::session_device::SessionDevice::new(manager.clone(), "/dev/fixture".into());
        let mut events = EventLoop::try_new()?;
        let (sender, receiver) = channel::channel();
        events
            .handle()
            .insert_source(receiver, |event, _, device| {
                if let channel::Event::Msg(event) = event {
                    crate::direct_session::session_event(event, device);
                }
            })?;
        let mut schedules = 0;
        let result = crate::active_session::run(&mut device, &mut events, |_, poll| {
            let input = RoutedInput {
                owner: SeatInput::new(manager.clone())?,
                extent: (32, 32),
            };
            Ok::<_, crate::InputDispatchError>(crate::direct_loop::run_with_input(
                &mut server,
                Target {
                    log: log.clone(),
                    refuse: false,
                },
                poll,
                &mut || {
                    schedules += 1;
                    if schedules == 1 {
                        DirectFrame::Idle
                    } else {
                        if pause {
                            assert!(sender.send(Event::PauseSession).is_ok());
                            assert!(sender.send(Event::ActivateSession).is_ok());
                        }
                        DirectFrame::Stop
                    }
                },
                input,
            ))
        })?;
        result.cleanup?;
        let result = result.outcome?;
        assert_eq!(result.outcome.is_err(), pause);
        if pause {
            assert!(matches!(
                result.outcome,
                Err(DirectLoopError::Session(SessionError::Inactive))
            ));
        }
        result.input_cleanup.ok_or("missing cleanup")??;
        result.input_flush.ok_or("missing input flush")??;
        result.retirement.ok_or("missing retirement")??;
        result.flush.ok_or("missing flush")??;
        assert_eq!(schedules, 2);
        assert_eq!(
            *log.borrow(),
            ["retire", "target dropped", "display closed"]
        );
        let mut peers = manager.peers.borrow_mut();
        let peer = peers.first_mut().ok_or("missing descriptor peer")?;
        peer.set_nonblocking(true)?;
        assert_eq!(peer.read(&mut [0])?, 0);
    }
    Ok(())
}
#[test]
fn direct_input_loop_pause_during_real_empty_batch_refuses_frame()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind(dir.path(), "batch-pause")?;
    let log = Log::default();
    let owner = SeatInput::new(Manager {
        log: log.clone(),
        peers: Rc::default(),
    })?;
    let mut polls = 0;
    let mut schedules = 0;
    let result = crate::direct_loop::run_with_input(
        &mut server,
        Target {
            log: log.clone(),
            refuse: true,
        },
        &mut || {
            polls += 1;
            if polls == 4 {
                Err(SessionError::Inactive)
            } else {
                Ok(())
            }
        },
        &mut || {
            schedules += 1;
            DirectFrame::Render(1)
        },
        RoutedInput {
            owner,
            extent: (32, 32),
        },
    );
    assert!(matches!(result.outcome, Err(DirectLoopError::Input(_))));
    result.input_cleanup.ok_or("missing cleanup")??;
    result.input_flush.ok_or("missing input flush")??;
    assert!(result.retirement.ok_or("missing retirement")?.is_err());
    result.flush.ok_or("missing flush")??;
    assert_eq!(polls, 4);
    assert_eq!(schedules, 1);
    assert_eq!(*log.borrow(), ["retire", "target dropped"]);
    Ok(())
}
