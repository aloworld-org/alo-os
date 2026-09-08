//! Real callback and output events across direct-target commit outcomes.
use super::*;
use smithay::reexports::wayland_server::Display;
use std::{
    os::unix::net::UnixStream,
    sync::{Arc, mpsc},
    thread,
    time::{Duration, Instant},
};
#[path = "direct_protocol_client.rs"]
mod client;

#[test]
fn output_retirement_orders_disable_withdrawal_and_fresh_lifetime()
-> Result<(), Box<dyn std::error::Error>> {
    for refuse in [false, true] {
        use std::os::unix::fs::PermissionsExt;
        let runtime = tempfile::tempdir()?;
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))?;
        let mut server = crate::Server::bind(runtime.path(), "retirement")?;
        let mut handle = server.display_handle();
        let (socket, peer) = UnixStream::pair()?;
        let server_client =
            handle.insert_client(socket, Arc::new(crate::surfaces::ClientState::default()))?;
        let (send, receive) = mpsc::channel();
        let (reply, responses) = mpsc::channel();
        let client = thread::spawn(move || client::retirement(peer, send, responses, refuse));
        let (device, log) = fixture(&[], 0);
        let mut target = Target::new(Painter::default(), device, scanout_tests::output());
        let start = Instant::now();
        let mut stages = 0;
        while !client.is_finished() {
            if start.elapsed() > Duration::from_secs(15) {
                return Err("retirement client deadline".into());
            }
            server.dispatch()?;
            if let Ok((stage, id)) = receive.try_recv() {
                let root = server_client.object_from_protocol_id::<WlSurface>(&handle, id)?;
                match stage {
                    0 => {
                        server.presentation.render(
                            &handle,
                            &mut target,
                            (&[root], &[]),
                            &Cursor::Hidden,
                            80,
                        )?;
                        server.surfaces.popups.output_size = Some((1280, 720).into());
                    }
                    1 => {
                        let (other_device, other_log) = fixture(&[], 0);
                        let mut other_output = scanout_tests::output();
                        other_output.output.connector =
                            std::num::NonZeroU32::MIN.saturating_add(3).into();
                        let mut other = Target::new(Painter::default(), other_device, other_output);
                        assert!(matches!(
                            server.retire_output(&mut other),
                            Err(RenderError::OutputIdentityChanged)
                        ));
                        assert!(other_log.borrow().calls.is_empty());
                        assert!(other.retire().is_ok());
                        assert!(other_log.borrow().calls.is_empty());
                        if refuse {
                            log.borrow_mut().failures = vec!["disable"];
                        }
                        let result = server.retire_output(&mut target);
                        if refuse {
                            assert!(matches!(result, Err(RenderError::Retirement(_))));
                            assert!(server.surfaces.popups.output_size.is_some());
                        } else {
                            result?;
                            assert!(server.surfaces.popups.output_size.is_none());
                        }
                        assert_eq!(
                            log.borrow()
                                .calls
                                .iter()
                                .filter(|call| **call == "disable")
                                .count(),
                            1
                        );
                        if refuse {
                            assert_eq!(log.borrow().calls.last(), Some(&"disable"));
                        }
                    }
                    2 => {
                        let before = log.borrow().calls.clone();
                        assert!(matches!(
                            server.render(&mut target, 82),
                            Err(RenderError::DirectHalted)
                        ));
                        assert!(matches!(
                            server.retire_output(&mut target),
                            Err(RenderError::DirectHalted)
                        ));
                        assert_eq!(log.borrow().calls, before);
                    }
                    _ if !refuse => {
                        let (mut device, _) = fixture(&[], 0);
                        let mut output = scanout_tests::output();
                        // CRTC 2 and plane 3 stay fixed. Configure the independent
                        // transport oracle too; never derive it from the request.
                        device.connector = std::num::NonZeroU32::MIN.saturating_add(3);
                        output.output.connector =
                            std::num::NonZeroU32::MIN.saturating_add(3).into();
                        target = Target::new(Painter::default(), device, output);
                        server.presentation.render(
                            &handle,
                            &mut target,
                            (&[root], &[]),
                            &Cursor::Hidden,
                            83,
                        )?;
                    }
                    _ => {}
                }
                reply.send(())?;
                stages += 1;
            }
            thread::sleep(Duration::from_millis(1));
        }
        client
            .join()
            .map_err(|_| "retirement client assertions failed")??;
        assert_eq!(stages, 4);
        drop(target);
        assert_eq!(
            log.borrow()
                .calls
                .iter()
                .filter(|call| **call == "disable")
                .count(),
            1
        );
    }
    Ok(())
}
#[test]
fn direct_target_protocol_callbacks_and_membership_follow_only_commits()
-> Result<(), Box<dyn std::error::Error>> {
    let mut display = Display::<crate::surfaces::Surfaces>::new()?;
    let mut handle = display.handle();
    let mut surfaces = crate::surfaces::Surfaces::new(&handle);
    let mut presentation = crate::presentation::Presentation::default();
    let (socket, peer) = UnixStream::pair()?;
    let server_client =
        handle.insert_client(socket, Arc::new(crate::surfaces::ClientState::default()))?;
    let (send, receive) = mpsc::channel();
    let (reply, responses) = mpsc::channel();
    let client = thread::spawn(move || client::run(peer, send, responses));
    let (device, log) = fixture(&[], 0);
    let mut target = Target::new(Painter::default(), device, scanout_tests::output());
    let start = Instant::now();
    let mut stages = 0;
    while !client.is_finished() {
        if start.elapsed() > Duration::from_secs(15) {
            return Err("protocol client deadline".into());
        }
        display.dispatch_clients(&mut surfaces)?;
        display.flush_clients()?;
        if let Ok((stage, id)) = receive.try_recv() {
            log.borrow_mut().failures = match stage {
                0 | 2 | 3 => vec!["enable"],
                5 => vec!["destroy framebuffer"],
                _ => vec![],
            };
            let root = server_client.object_from_protocol_id::<WlSurface>(&handle, id)?;
            let roots = if stage == 3 || stage == 4 {
                vec![]
            } else {
                vec![root]
            };
            let result = presentation.render(
                &handle,
                &mut target,
                (&roots, &[]),
                &Cursor::Hidden,
                70 + stage,
            );
            match stage {
                0 | 2 | 3 => assert!(matches!(result, Err(RenderError::Scanout(_)))),
                6 => assert!(matches!(result, Err(RenderError::DirectHalted))),
                4 => assert_eq!(result?, 0),
                _ => assert_eq!(result?, 1),
            }
            reply.send(())?;
            stages += 1;
        }
        thread::sleep(Duration::from_millis(1));
    }
    client.join().map_err(|_| "client assertions failed")??;
    assert_eq!(stages, 7);
    let shutdown = target
        .disable()
        .err()
        .ok_or("missing post-commit cleanup error")?;
    assert_eq!(shutdown.errors.len(), 1);
    Ok(())
}
