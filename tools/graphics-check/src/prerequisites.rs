//! Native package and Wayland connection checks, before creating any window.

use std::ffi::OsStr;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Libraries needed by the selected nested and direct-display Smithay features.
const PACKAGES: &[(&str, &str)] = &[
    ("wayland-server", "libwayland-dev"),
    ("wayland-client", "libwayland-dev"),
    ("egl", "libegl1-mesa-dev"),
    ("glesv2", "libgles2-mesa-dev"),
    ("xkbcommon", "libxkbcommon-dev"),
    ("libudev", "libudev-dev"),
    ("libinput", "libinput-dev"),
    ("gbm", "libgbm-dev"),
    ("libseat", "libseat-dev"),
];

/// Ask pkg-config for each actual native dependency and identify missing headers.
pub(super) fn native_libraries() -> Result<(), String> {
    for (library, package) in PACKAGES {
        let output = Command::new("pkg-config")
            .args(["--modversion", library])
            .output()
            .map_err(|error| format!("cannot run pkg-config: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{library} unavailable; install Ubuntu package {package}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        println!(
            "{library}: {}",
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
    Ok(())
}

/// Resolve an explicitly selected Wayland session without an X11 fallback.
pub(super) fn session_socket(
    runtime: Option<&OsStr>,
    display: Option<&OsStr>,
) -> Result<PathBuf, String> {
    let display = display
        .filter(|value| !value.is_empty())
        .ok_or("WAYLAND_DISPLAY is missing or empty; select a Wayland session")?;
    let display = Path::new(display);
    if display.is_absolute() {
        return Ok(display.to_path_buf());
    }
    let runtime = runtime
        .filter(|value| !value.is_empty())
        .ok_or("XDG_RUNTIME_DIR is missing or empty")?;
    let runtime = Path::new(runtime);
    if !runtime.is_absolute() {
        return Err("XDG_RUNTIME_DIR must be absolute".into());
    }
    Ok(runtime.join(display))
}

/// A pathname alone is not evidence of a listening compositor.
pub(super) fn connect(socket: &Path) -> Result<(), String> {
    UnixStream::connect(socket).map(drop).map_err(|error| {
        format!(
            "cannot connect to Wayland socket {}: {error}",
            socket.display()
        )
    })
}

#[cfg(test)]
mod tests;
