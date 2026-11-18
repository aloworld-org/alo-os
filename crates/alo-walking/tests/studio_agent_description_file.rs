//! Actual private description file opens, not just synthetic metadata checks.
//!
//! Only test-library calls select these temporary paths. The installed probe
//! must keep its fixed /etc/alo/agentd.toml literal and refuse every CLI arg.
//! Positive root-owned/chown observations require the root test lane; their
//! absence is printed explicitly on a non-root lane, never called guest proof.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    reason = "local filesystem fixture setup must succeed"
)]

use alo_walking::studio_agent_launch::read_description;
use std::{
    fs,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt, symlink},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const CONTENTS: &str = "format=1\n[logins]\nperson=1000\nagent=60989\ngroup=60989\n";
struct Folder(PathBuf);
impl Folder {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "studio-description-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("own temporary directory");
        Self(path)
    }
    fn file(&self) -> PathBuf {
        let path = self.0.join("agentd.toml");
        fs::write(&path, CONTENTS).expect("public test description");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("fixture mode");
        path
    }
}
impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn root_fixture(path: &std::path::Path) -> bool {
    if fs::metadata(path).expect("own fixture metadata").uid() == 0 {
        true
    } else {
        eprintln!(
            "root-owned description acceptance/chown NOT EXERCISED: test fixture owner is not root; run the root test lane"
        );
        false
    }
}

#[test]
fn root_owned_regular_description_is_read_validated_and_not_modified() {
    let folder = Folder::new();
    let path = folder.file();
    if !root_fixture(&path) {
        return;
    }
    assert_eq!(read_description(&path), Ok(CONTENTS.to_owned()));
    assert_eq!(
        fs::read_to_string(&path).expect("original remains"),
        CONTENTS
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).expect("read-only fixture");
    assert_eq!(read_description(&path), Ok(CONTENTS.to_owned()));
}

#[test]
fn opened_file_permissions_owner_contents_and_size_are_enforced() {
    let folder = Folder::new();
    let path = folder.file();
    if !root_fixture(&path) {
        return;
    }
    for mode in [0o664, 0o646, 0o666] {
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("writable fixture");
        assert!(read_description(&path).is_err(), "mode {mode:o}");
    }
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("safe fixture mode");
    fs::write(&path, CONTENTS.replace("person=1000", "person=60989"))
        .expect("wrong public identity");
    assert!(
        read_description(&path).is_err(),
        "opening trusted metadata must not skip content validation"
    );
    fs::write(&path, format!("{CONTENTS}#{}\n", "x".repeat(65_537)))
        .expect("oversized otherwise-valid TOML fixture");
    assert!(read_description(&path).is_err());
    let prefix = format!("{CONTENTS}#");
    let at_limit = format!("{prefix}{}\n", "x".repeat(65_536 - prefix.len() - 1));
    fs::write(&path, &at_limit).expect("exact limit fixture");
    assert_eq!(read_description(&path), Ok(at_limit));
    fs::write(&path, b"\xff\xfe").expect("non-UTF8 fixture");
    assert!(read_description(&path).is_err());
    fs::write(&path, CONTENTS).expect("restore contents");
    std::os::unix::fs::chown(&path, Some(1000), None)
        .expect("root lane changes only its own temporary file owner");
    assert!(
        read_description(&path).is_err(),
        "actual non-root file owner must refuse"
    );
}

#[test]
fn symlink_directory_and_missing_path_are_refused_before_contents_are_read() {
    let folder = Folder::new();
    let target = folder.file();
    assert!(read_description(&folder.0).is_err());
    assert!(read_description(&folder.0.join("missing.toml")).is_err());
    if !root_fixture(&target) {
        return;
    }
    let link = folder.0.join("linked.toml");
    symlink(&target, &link).expect("local symlink");
    assert!(
        read_description(&link).is_err(),
        "must not follow a symlink to an otherwise valid description"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn fifo_without_a_writer_is_rejected_without_blocking_open() {
    let folder = Folder::new();
    let fifo = folder.0.join("agentd.toml");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .expect("local FIFO fixture tool")
            .success()
    );
    let path = fifo.clone();
    let (result, received) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        result.send(read_description(&path)).expect("observer");
    });
    let began = Instant::now();
    let observed = received.recv_timeout(Duration::from_millis(300));
    // Release an accidentally blocking reader before failing, so a mutation
    // cannot strand this test thread. O_NONBLOCK is Linux's fixed 0x800 here.
    if observed.is_err() {
        let _ = fs::OpenOptions::new()
            .write(true)
            .custom_flags(0x800)
            .open(&fifo);
    }
    worker.join().expect("fixture reader finishes");
    assert!(
        observed.is_ok_and(|answer| answer.is_err()),
        "FIFO metadata must refuse before waiting for a writer"
    );
    assert!(began.elapsed() < Duration::from_secs(1));
}
