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
