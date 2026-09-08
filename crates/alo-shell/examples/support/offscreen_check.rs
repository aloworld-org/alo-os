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
                9 => {
                    let root = roots.first().ok_or("resize client missing")?;
                    assert!(server.request_window_size(root, (32, 24))?.is_some());
                }
                10 => {
                    let prepared = render_scanout(
                        renderer,
                        (33, 32).into(),
                        &roots,
                        &popups,
                        &alo_shell::Cursor::Hidden,
                    )?;
                    assert_eq!(prepared.surfaces().len(), 1);
                    for (index, pixel) in prepared
                        .pixels()
                        .pixels()
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .enumerate()
                    {
                        let expected = if index % 33 < 32 && index / 33 < 24 {
                            [255, 255, 255, 0]
                        } else {
                            [0; 4]
                        };
                        assert_eq!(*pixel, expected, "resized pixel {index}");
                    }
                    println!("Acknowledged 32x24 window resize GLES pixels passed");
                }
                8 => {
                    crate::window_raise_check::run(&mut server, renderer)?;
                    crate::window_switch_check::run(&mut server, renderer)?;
                    crate::window_size_check::run(&mut server, renderer)?;
                    crate::window_placement_check::run(&mut server, renderer)?;
                }
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
                    crate::default_cursor_check::run(renderer, &roots, &popups)?;
                    let restored =
                        render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)?;
                    offscreen_client::verify(restored.pixels().pixels());
                    prepared.pixels().frame()?;
                    // Actual prepared scene through the public activation path.
                    // A non-DRM descriptor must refuse, never complete callbacks.
                    use std::os::fd::AsFd;
                    let fd = std::fs::File::open("/dev/null")?;
                    let error = prepared
                        .activate(fd.as_fd(), &refusal_output(33))
                        .err()
                        .ok_or("non-DRM activation accepted")?;
                    assert_eq!(error.failure.source.raw_os_error(), Some(25));
                    assert!(error.cleanup.is_empty());
                    // The public FrameTarget must preserve real client callbacks
                    // and membership when actual kernel allocation refuses.
                    let mut direct =
                        alo_shell::DirectTarget::new(renderer, fd.as_fd(), refusal_output(33));
                    let result = server.render(&mut direct, 66);
                    let Err(alo_shell::RenderError::Scanout(error)) = result else {
                        return Err("non-DRM direct target did not refuse scanout".into());
                    };
                    assert_eq!(error.failure.source.raw_os_error(), Some(25));
                    assert!(direct.retirement_error().is_none());
                    direct.disable()?;
                    let prepared =
                        render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)?;
                    let error = prepared
                        .activate(fd.as_fd(), &refusal_output(34))
                        .err()
                        .ok_or("mismatched mode accepted")?;
                    assert_eq!(error.failure.stage, "validate prepared scene");
                    let prepared =
                        render_scanout(renderer, (33, 32).into(), &roots, &popups, &cursor)?;
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
                    assert!(
                        matches!(cursor, alo_shell::Cursor::Arrow { location } if location == (10.0, 14.0).into())
                    );
                    assert!(prepared.pixels().pixels().contains(&255));
                    crate::default_cursor_check::run(renderer, &roots, &popups)?;
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
                6 => {
                    assert!(matches!(cursor, alo_shell::Cursor::Hidden));
                    crate::default_cursor_check::run(renderer, &roots, &popups)?;
                }
                7 => {
                    assert!(
                        matches!(cursor, alo_shell::Cursor::Arrow { location } if location == (10.0, 14.0).into())
                    );
                    crate::default_cursor_check::run(renderer, &roots, &popups)?;
                }
                _ => return Err("unknown client stage".into()),
            }
            stages += 1;
            reply.send(())?;
        }
        thread::sleep(Duration::from_millis(1));
    }
    client.join().map_err(|_| "client assertion failed")?;
    assert_eq!(stages, 10);
    println!(
        "Real SHM window/child/popup/client and default cursor golden pixels, clipping, hidden/destroyed switching, orientation, preparation and refusal callback preservation, fixture-only submission, disconnect and truncated-SHM import refusal passed; DRM and hardware unverified"
    );
    Ok(())
}

/// Synthetic routing only for refusal against /dev/null, never real KMS discovery.
fn refusal_output(width: u16) -> alo_shell::AtomicOutput {
    use std::num::NonZeroU32;
    let properties = |names: &[&'static str], start: u32| {
        names
            .iter()
            .zip(start..)
            .map(|(name, id)| (*name, NonZeroU32::MIN.saturating_add(id).into()))
            .collect()
    };
    alo_shell::AtomicOutput {
        output: alo_shell::DirectOutput {
            physical_size: Some((310, 170)),
            connector: NonZeroU32::MIN.into(),
            crtc: NonZeroU32::MIN.saturating_add(1).into(),
            mode: drm_ffi::drm_mode_modeinfo {
                clock: 240,
                hdisplay: width,
                hsync_start: 40,
                hsync_end: 45,
                htotal: 80,
                vdisplay: 32,
                vsync_start: 35,
                vsync_end: 40,
                vtotal: 50,
                ..Default::default()
            }
            .into(),
        },
        plane: NonZeroU32::MIN.saturating_add(2).into(),
        formats: vec![drm::buffer::DrmFourcc::Xrgb8888 as u32],
        connector_properties: properties(&["CRTC_ID"], 10),
        crtc_properties: properties(&["ACTIVE", "MODE_ID"], 20),
        plane_properties: properties(
            &[
                "CRTC_ID", "FB_ID", "CRTC_X", "CRTC_Y", "CRTC_W", "CRTC_H", "SRC_X", "SRC_Y",
                "SRC_W", "SRC_H",
            ],
            30,
        ),
    }
}
