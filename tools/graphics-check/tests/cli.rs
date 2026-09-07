//! Process-level diagnostics: no session or bad arguments must never succeed.

use std::process::Command;

#[test]
fn refuses_unknown_arguments() -> std::io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_alo-graphics-check"))
        .arg("unknown")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("graphics check failed:"));
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn refuses_a_missing_wayland_session_even_if_x11_is_available() -> std::io::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_alo-graphics-check"))
        .arg("render")
        .env_remove("WAYLAND_DISPLAY")
        .env("DISPLAY", ":0")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("WAYLAND_DISPLAY is missing"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("frame submitted"));
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn names_the_missing_native_package() -> std::io::Result<()> {
    let empty = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_alo-graphics-check"))
        .arg("check")
        .env_remove("PKG_CONFIG_PATH")
        .env("PKG_CONFIG_LIBDIR", empty.path())
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("wayland-server unavailable; install Ubuntu package libwayland-dev")
    );
    Ok(())
}
