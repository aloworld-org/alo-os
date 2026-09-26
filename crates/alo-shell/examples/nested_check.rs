//! Explicit WSLg integration fixture; never installed as the person's session.

#[cfg(target_os = "linux")]
#[path = "../tests/support/application.rs"]
mod application;

#[cfg(target_os = "linux")]
#[path = "support/popup_check.rs"]
mod popup_check;

#[cfg(target_os = "linux")]
#[path = "support/grab_check.rs"]
mod grab_check;

#[cfg(target_os = "linux")]
#[path = "support/offscreen_check.rs"]
mod offscreen_check;

#[cfg(target_os = "linux")]
#[path = "support/offscreen_client.rs"]
mod offscreen_client;

#[cfg(target_os = "linux")]
#[path = "support/default_cursor_check.rs"]
mod default_cursor_check;

#[cfg(target_os = "linux")]
#[path = "support/window_raise_check.rs"]
mod window_raise_check;

#[cfg(target_os = "linux")]
#[path = "support/interactive_resize_check.rs"]
mod interactive_resize_check;
#[cfg(target_os = "linux")]
#[path = "support/nested_client_check.rs"]
mod nested_client_check;
#[cfg(target_os = "linux")]
#[path = "support/nested_control_frame_check.rs"]
mod nested_control_frame_check;
#[cfg(target_os = "linux")]
#[path = "support/nested_reader_frame_check.rs"]
mod nested_reader_frame_check;
#[cfg(target_os = "linux")]
#[path = "support/resize_geometry_check.rs"]
mod resize_geometry_check;
#[cfg(target_os = "linux")]
#[path = "support/window_control_label_check.rs"]
mod window_control_label_check;
#[cfg(target_os = "linux")]
#[path = "support/window_control_scene_check.rs"]
mod window_control_scene_check;
#[cfg(target_os = "linux")]
#[path = "support/window_control_snapshot_check.rs"]
mod window_control_snapshot_check;
#[cfg(target_os = "linux")]
#[path = "support/window_controls_pixels.rs"]
mod window_controls_pixels;
#[cfg(target_os = "linux")]
#[path = "support/window_maximize_check.rs"]
mod window_maximize_check;
#[cfg(target_os = "linux")]
#[path = "support/window_minimize_check.rs"]
mod window_minimize_check;
#[cfg(target_os = "linux")]
#[path = "support/window_placement_check.rs"]
mod window_placement_check;
#[cfg(target_os = "linux")]
#[path = "support/window_size_check.rs"]
mod window_size_check;
#[cfg(target_os = "linux")]
#[path = "support/window_switch_check.rs"]
mod window_switch_check;

/// Socket location shared with the real protocol-client fixture.
#[cfg(target_os = "linux")]
pub struct Fixture {
    /// Private test display, distinct from the parent WSLg socket.
    pub path: std::path::PathBuf,
}

/// Main-thread graphics initialization is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nested compositor check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Drive actual client buffers through GLES and require callbacks and teardown.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--offscreen") {
        return offscreen_check::run();
    }
    if std::env::args().any(|arg| arg == "--pointer-release-grabs") {
        return grab_check::run(false, true);
    }
    if std::env::args().any(|arg| arg == "--grabs") {
        return grab_check::run(false, false);
    }
    if std::env::args().any(|arg| arg == "--keyboard-grabs") {
        return grab_check::run(true, false);
    }
    if std::env::args().any(|arg| arg == "--keyboard-release-grabs") {
        return grab_check::run(true, true);
    }
    nested_client_check::run(nested_client_check::Flags::from_args())
}
