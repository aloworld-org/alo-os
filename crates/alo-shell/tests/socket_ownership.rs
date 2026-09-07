//! Private runtime/session ownership and cleanup, using real Linux sockets.
#![cfg(target_os = "linux")]
#![expect(clippy::unwrap_used, reason = "unexpected results fail the test")]

use alo_shell::{Server, SocketError};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn owns_only_a_fresh_session_and_removes_it_on_drop() {
    let runtime = tempfile::tempdir().unwrap();
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let server = Server::bind(runtime.path(), "alo-test").unwrap();
    let path = server.socket_path().to_owned();
    assert!(path.exists());
    assert!(matches!(
        Server::bind(runtime.path(), "alo-test"),
        Err(SocketError::InUse)
    ));
    assert!(path.exists());
    drop(server);
    assert!(!path.exists());
    assert!(!runtime.path().join("alo-test").exists());
    assert!(Server::bind(runtime.path(), "alo-test").is_ok());
}

#[test]
fn refuses_unsafe_runtime_and_names_without_touching_existing_files() {
    let runtime = tempfile::tempdir().unwrap();
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o700)).unwrap();
    for name in ["", "..", "a/b", "/tmp/outside", "a.b"] {
        assert!(matches!(
            Server::bind(runtime.path(), name),
            Err(SocketError::InvalidName)
        ));
    }
    let occupied = runtime.path().join("occupied");
    fs::write(&occupied, b"keep").unwrap();
    assert!(matches!(
        Server::bind(runtime.path(), "occupied"),
        Err(SocketError::InUse)
    ));
    assert_eq!(fs::read(occupied).unwrap(), b"keep");
    let link = runtime.path().join("link");
    symlink(runtime.path(), &link).unwrap();
    assert!(matches!(
        Server::bind(&link, "alo-test"),
        Err(SocketError::UnsafeRuntime)
    ));
    fs::set_permissions(runtime.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        Server::bind(runtime.path(), "alo-test"),
        Err(SocketError::UnsafeRuntime)
    ));
    assert!(!runtime.path().join("alo-test").exists());
}

#[test]
fn a_failed_socket_bind_removes_its_fresh_directory_and_lock() {
    let runtime = tempfile::tempdir().unwrap();
    let long = runtime.path().join("x".repeat(90));
    fs::create_dir(&long).unwrap();
    fs::set_permissions(&long, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        Server::bind(&long, "session"),
        Err(SocketError::Bind(_))
    ));
    assert!(!long.join("session").exists());
}
