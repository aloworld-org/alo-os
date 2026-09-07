//! Session selection and real Unix socket refusal tests, requiring no display.

use super::*;
use std::os::unix::net::UnixListener;

#[test]
fn resolves_relative_and_absolute_displays() {
    assert_eq!(
        session_socket(
            Some(OsStr::new("/run/user/1000")),
            Some(OsStr::new("wayland-0"))
        ),
        Ok(PathBuf::from("/run/user/1000/wayland-0"))
    );
    assert_eq!(
        session_socket(None, Some(OsStr::new("/mnt/wslg/runtime-dir/wayland-0"))),
        Ok(PathBuf::from("/mnt/wslg/runtime-dir/wayland-0"))
    );
}

#[test]
fn refuses_missing_empty_and_relative_session_configuration() {
    for display in [None, Some(OsStr::new(""))] {
        assert!(session_socket(Some(OsStr::new("/run/user/1000")), display).is_err());
    }
    for runtime in [None, Some(OsStr::new("")), Some(OsStr::new("relative"))] {
        assert!(session_socket(runtime, Some(OsStr::new("wayland-0"))).is_err());
    }
}

#[test]
fn connects_to_a_listener_and_refuses_stale_missing_and_regular_files() -> std::io::Result<()> {
    // A unique directory is owned by this fixture; it never unlinks a session socket.
    let directory = tempfile::tempdir()?;
    let socket = directory.path().join("wayland-test");
    assert!(connect(&socket).is_err());
    let listener = UnixListener::bind(&socket)?;
    assert!(connect(&socket).is_ok());
    drop(listener);
    assert!(connect(&socket).is_err());
    std::fs::remove_file(&socket)?;
    std::fs::write(&socket, b"not a compositor")?;
    assert!(connect(&socket).is_err());
    Ok(())
}
