//! **A plane that moves under a viewport that does not.**
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 1. Its acceptance is
//! that the dock and the status area stay exactly where they are across a pan and
//! a zoom; its constraint is that **nothing in the viewport layer may read the
//! plane's transform to correct itself. If it has to, it is in the wrong layer.**
//!
//! # Those two need different kinds of test, and only one of them is about pixels
//!
//! A dock that subtracted the pan to stay still would pass the acceptance
//! perfectly. It would be drawn on the plane, compensating, and the first thing
//! that moved the plane without telling it — a zoom, a second display, a
//! restored session — would tear it off. So the constraint is held by **reading
//! the crate**: no file that draws a viewport surface may name the camera at all.
//!
//! That is the same shape as `one_layout_decider.rs`, and for the same reason: a
//! structural claim cannot be checked by looking at where something ended up.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's own source.
fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every file under `src/` that is not itself a test.
fn every_source_file() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut walking = vec![source()];
    while let Some(directory) = walking.pop() {
        for entry in std::fs::read_dir(&directory).expect("this crate has a src directory") {
            let path = entry.expect("a directory entry can be read").path();
            if path.is_dir() {
                walking.push(path);
            } else if path.extension().is_some_and(|it| it == "rs")
                && !path
                    .file_name()
                    .is_some_and(|it| it.to_string_lossy().ends_with("_tests.rs"))
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The surfaces that live in the viewport layer, by the file that lays each out.
///
/// Read as *what a person sees that a pan must not move*. Every one of these is
/// painted above `crate::scene::trees` from the output's own size, and none of
/// them is a frame on the plane.
const THE_VIEWPORT_LAYER: [&str; 11] = [
    "desktop_raster.rs",
    "dock_raster.rs",
    "egress_status_raster.rs",
    "in_use_raster.rs",
    "notification_raster.rs",
    "capture_raster.rs",
    "approval_raster.rs",
    "record_raster.rs",
    "settings_raster.rs",
    "window_control_overlay.rs",
    "window_controls.rs",
];

/// **No file that draws a viewport surface names the camera.**
///
/// The constraint, and the only way to hold it. A viewport control that read the
/// camera would be compensating for a pan — in the wrong layer, and invisible to
/// any test that merely checked it had not moved.
#[test]
fn nothing_in_the_viewport_layer_reads_the_camera() {
    let mut reading: Vec<String> = Vec::new();
    for at in every_source_file() {
        let named = at
            .file_name()
            .map(|it| it.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !THE_VIEWPORT_LAYER.contains(&named.as_str()) {
            continue;
        }
        let written = std::fs::read_to_string(&at).expect("a source file this crate compiles");
        // In code, not in a sentence explaining why the camera is absent.
        if written.lines().map(str::trim).any(|line| {
            !line.starts_with("//") && (line.contains("Camera") || line.contains("alo_canvas"))
        }) {
            reading.push(named);
        }
    }
    assert!(
        reading.is_empty(),
        "these draw the viewport and read the plane's camera, which puts them in \
         the wrong layer: {reading:?}"
    );
}

/// **Every file the viewport layer is claimed to be still exists.**
///
/// A list of filenames rots silently: rename one and this test passes by checking
/// nothing. Held so the clause above cannot quietly become vacuous.
#[test]
fn the_viewport_layer_is_the_files_it_says_it_is() {
    let present: Vec<String> = every_source_file()
        .iter()
        .filter_map(|at| at.file_name().map(|it| it.to_string_lossy().into_owned()))
        .collect();
    for named in THE_VIEWPORT_LAYER {
        assert!(
            present.iter().any(|it| it == named),
            "{named} is named as part of the viewport layer and no longer exists, \
             so the constraint above is checking one file fewer than it says"
        );
    }
}

/// **The camera moves the plane and nothing else.**
///
/// The acceptance, at the only place the transform is applied: a pan and a zoom
/// change where a frame is drawn, and the viewport's own surfaces are laid out
/// from the output's size — which the camera is not part of, and which is why
/// their position cannot depend on it.
#[test]
fn a_pan_and_a_zoom_move_the_plane_and_not_the_viewport() {
    use std::os::unix::fs::PermissionsExt as _;

    let runtime = tempfile::tempdir().expect("a runtime directory");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        alo_shell::Server::bind(runtime.path(), "one-plane").expect("a server to look with");

    let before = server.the_camera();
    assert_eq!(before.at(), alo_canvas::At::origin());

    let panned = server
        .pan_the_canvas(-120, -80)
        .expect("a pan on the plane");
    assert_ne!(
        panned.at(),
        before.at(),
        "a pan did not move what the session is looking at"
    );

    // Pointer-centred: whatever was under this pixel is still under it.
    let held = (640, 360);
    let under = panned
        .plane_of(held)
        .expect("a plane point under the pointer");
    let zoomed = server
        .zoom_the_canvas(alo_canvas::Zoom::of(400).expect("40 per cent"), held)
        .expect("a zoom on the plane");
    assert_eq!(
        zoomed.plane_of(held),
        Some(under),
        "zooming moved what was under the pointer"
    );
    assert_eq!(zoomed.zoom().thousandths(), 400);
}

/// **Every display and every wrapper in this crate answers for the camera.**
///
/// The seam has one writer and four readers, and three of the readers are
/// wrappers: `ControlTarget`, which every ordinary frame crosses, `Layered`,
/// which adds this shell's own surfaces above the plane, and `ReaderTarget`. The
/// first version of this work wired the two real backends and left all three
/// wrappers on the trait's default, so a pan was refused in the product while
/// every arithmetic test in `alo-canvas` passed — the refusal was honest, and
/// caught it, and it should not have needed a running compositor to catch.
///
/// A default that refuses is the right default for a target that cannot draw the
/// plane. It is the wrong answer for anything in `src/`, because everything here
/// either draws the plane or hands it to something that does.
#[test]
fn no_display_or_wrapper_in_this_crate_takes_the_default_camera() {
    let mut silent: Vec<String> = Vec::new();
    for at in every_source_file() {
        let written = std::fs::read_to_string(&at).expect("a source file this crate compiles");
        let lines: Vec<&str> = written.lines().collect();
        for (from, line) in lines.iter().enumerate() {
            let Some(named) = line.split("FrameTarget for ").nth(1) else {
                continue;
            };
            // Impls are items, so the block ends at the first `}` in column one.
            let ends = lines
                .iter()
                .enumerate()
                .skip(from + 1)
                .find(|(_, it)| **it == "}")
                .map_or(lines.len(), |(at, _)| at);
            let block = lines.get(from..ends).unwrap_or_default();
            if !block.iter().any(|it| it.contains("fn look_at")) {
                let file = at
                    .file_name()
                    .map(|it| it.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let named = named.trim_end_matches(" {").trim();
                silent.push(format!("{file}: {named}"));
            }
        }
    }
    assert!(
        silent.is_empty(),
        "these are frame targets in this crate that never answer `look_at`, so a \
         pan reaching them is refused rather than drawn: {silent:?}"
    );
}

/// **A pan reaches the target through the path an ordinary frame takes.**
///
/// The structural claim above says nobody is silent; this says the one writer
/// actually writes, through every wrapper between `Server::render` and a display.
/// Neither test needs a GPU, which is the point: the fault this pair replaces was
/// found by a compositor fixture that only this lane can run.
#[test]
fn a_panned_session_hands_its_camera_to_the_display() {
    use std::os::unix::fs::PermissionsExt as _;

    /// Records the last camera it was given, and nothing else.
    struct Watching {
        /// What the session last said it was looking at.
        camera: Option<alo_canvas::Camera>,
    }

    impl alo_shell::FrameTarget for Watching {
        fn look_at(&mut self, camera: alo_canvas::Camera) -> Result<(), alo_shell::RenderError> {
            self.camera = Some(camera);
            Ok(())
        }
        fn size(&self) -> smithay::utils::Size<i32, smithay::utils::Physical> {
            (1366, 768).into()
        }
        fn submit(
            &mut self,
            roots: &[smithay::reexports::wayland_server::protocol::wl_surface::WlSurface],
        ) -> Result<
            Vec<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
            alo_shell::RenderError,
        > {
            Ok(roots.to_vec())
        }
    }

    let runtime = tempfile::tempdir().expect("a runtime directory");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        alo_shell::Server::bind(runtime.path(), "one-plane-handed").expect("a server to look with");
    let mut target = Watching { camera: None };

    server.render(&mut target, 0).expect("an empty frame");
    assert_eq!(
        target.camera,
        Some(alo_canvas::Camera::new()),
        "a session nobody has panned still has to say where it is looking"
    );

    let panned = server.pan_the_canvas(-40, 25).expect("a pan on the plane");
    server.render(&mut target, 1).expect("a panned frame");
    assert_eq!(
        target.camera,
        Some(panned),
        "the session panned and the display was never told"
    );

    let zoomed = server
        .zoom_the_canvas(alo_canvas::Zoom::of(400).expect("40 per cent"), (683, 384))
        .expect("a zoom on the plane");
    server.render(&mut target, 2).expect("a zoomed frame");
    assert_eq!(
        target.camera,
        Some(zoomed),
        "the session zoomed and the display was told only about the pan"
    );
}

/// **A pan off the plane is refused rather than clamped.**
///
/// `alo-canvas` refuses by name, and the session carries that refusal rather than
/// quietly stopping at a wall — a person whose canvas ignored them would have no
/// way to tell that from one that had finished moving.
#[test]
fn a_pan_past_the_plane_is_refused_and_changes_nothing() {
    use std::os::unix::fs::PermissionsExt as _;

    let runtime = tempfile::tempdir().expect("a runtime directory");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        alo_shell::Server::bind(runtime.path(), "one-plane-edge").expect("a server to look with");

    let before = server.the_camera();
    assert!(server.pan_the_canvas(i32::MAX, 0).is_none());
    assert_eq!(
        server.the_camera().at(),
        before.at(),
        "a refused pan moved the camera anyway"
    );
}
