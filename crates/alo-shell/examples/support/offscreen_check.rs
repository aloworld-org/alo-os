//! Real client scene pixels and callback timing; no kernel scanout claim.
use crate::{Fixture, offscreen_client};

/// Exercise the public preparation path with a real client and GLES context.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_shell::{Server, render_scanout};
    use smithay::{
        backend::{renderer::gles::GlesRenderer, winit},
        reexports::winit::{
            raw_window_handle::{HasDisplayHandle, RawDisplayHandle},
            window::Window,
        },
    };
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    let (mut backend, _events) = winit::init_from_attributes::<GlesRenderer>(
        Window::default_attributes().with_title("alo offscreen scene check"),
    )?;
    if !matches!(
        backend.window().display_handle()?.as_raw(),
        RawDisplayHandle::Wayland(_)
    ) {
        return Err("requires a Wayland parent".into());
    }
    let (renderer, _window) = backend.bind()?;
    let runtime = tempfile::tempdir()?;
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700))?;
    let mut server = Server::bind_keyboard(runtime.path(), "offscreen-check", Default::default())?;
    server.enable_pointer()?;
    server.enable_popup_protocol();
    let fixture = Fixture {
        path: server.socket_path().to_owned(),
    };
    let (send, receive) = mpsc::channel();
    let (reply, responses) = mpsc::channel();
    let client = thread::spawn(move || offscreen_client::run(fixture, send, responses));
    let start = Instant::now();
    let mut stages = 0;
    while !client.is_finished() {
        if start.elapsed() > Duration::from_secs(15) {
            return Err("client deadline exceeded".into());
        }
        server.dispatch()?;
        server.pointer_motion(10.0, 14.0, 1)?;
        if let Ok(stage) = receive.try_recv() {
            let roots: Vec<_> = server.mapped_surfaces().cloned().collect();
            let popups = server.popup_surfaces();
            let cursor = server.cursor();
            match stage {
                1 => {
                    for size in [(0, 32), (32, 0), (i32::MAX, 1)] {
                        assert!(
                            render_scanout(renderer, size.into(), &roots, &popups, &cursor)
                                .is_err()
                        );
                    }
                    let prepared =
                        render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)?;
                    assert_eq!(prepared.surfaces().len(), 4);
                    offscreen_client::verify(prepared.pixels().pixels());
                    prepared.pixels().frame()?;
                    // Repeated preparation must not retain stale output pixels or send callbacks.
                    let blank = render_scanout(
                        renderer,
                        (33, 32).into(),
                        &[],
                        &[],
                        &alo_shell::Cursor::Hidden,
                    )?;
                    assert!(blank.surfaces().is_empty());
                    assert!(blank.pixels().pixels().iter().all(|byte| *byte == 0));
                    offscreen_client::verify(prepared.pixels().pixels());
                }
                2 | 3 => {
                    let mut target = offscreen_client::Target {
                        renderer,
                        refuse: stage == 2,
                    };
                    let result = server.render(&mut target, 77);
                    if stage == 2 {
                        assert!(result.is_err());
                    } else {
                        assert_eq!(result?, 4);
                    }
                }
                4 => {
                    assert!(roots.is_empty());
                    assert!(popups.is_empty());
                    let prepared =
                        render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)?;
                    assert!(prepared.surfaces().is_empty());
                    assert!(prepared.pixels().pixels().iter().all(|byte| *byte == 0));
                }
                5 => {
                    let error = render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)
                        .err()
                        .ok_or("truncated SHM import unexpectedly succeeded")?;
                    assert!(
                        error.to_string().contains("import client buffer"),
                        "{error}"
                    );
                }
                _ => return Err("unknown client stage".into()),
            }
            stages += 1;
            reply.send(())?;
        }
        thread::sleep(Duration::from_millis(1));
    }
    client.join().map_err(|_| "client assertion failed")?;
    assert_eq!(stages, 5);
    println!(
        "Real SHM window/child/popup/cursor pixels, clipping, orientation, preparation and refusal callback preservation, fixture-only submission, disconnect and truncated-SHM import refusal passed; DRM and hardware unverified"
    );
    Ok(())
}
