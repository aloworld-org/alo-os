//! What a person downloads: the program, and the environment beside it —
//! assembled exactly the way `.github/workflows/release.yml` assembles it.
//!
//! The environment is built from `image/installing/Containerfile`; the list is
//! `sha256sum` over every file of it, as `alo-installing.sha256`; and the
//! program is built with that list's own digest in
//! `ALO_INSTALLER_ENVIRONMENT_SHA256`, which is what makes the download genuine
//! to the program that reads it. A program built without it says *this
//! download is not a genuine alo OS*, and the walk would measure that refusal
//! and nothing else.
//!
//! # Built for `x86_64-pc-windows-gnu`, and the release is `-msvc`
//!
//! The walk is run from Linux, where the MSVC target cannot be linked, so the
//! program is cross-built with MinGW. It is the same source and it reaches
//! Windows only through `std::process` — every change this walk measures is a
//! Windows program the installer starts — and its only imports are
//! `KERNEL32.dll`, `msvcrt.dll`, `ntdll.dll` and one API set (read with
//! `objdump -p`, 2026-09-21). What that does **not** show is that the MSVC
//! build a release ships behaves the same; the report says so.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::machine::run;

/// The target the program is built for.
pub const THE_TARGET: &str = "x86_64-pc-windows-gnu";

/// Build the environment and the program, and lay them out as the download.
///
/// # Panics
/// When anything in the build fails.
#[must_use]
pub fn assembled(yard: &Path, repository: &Path, target_directory: &Path) -> PathBuf {
    let environment = yard.join("environment");
    let _ = std::fs::remove_dir_all(&environment);
    let built = Command::new("podman")
        .current_dir(repository)
        .args([
            "build",
            "-f",
            "image/installing/Containerfile",
            "--output",
            &format!("type=local,dest={}", environment.display()),
            ".",
        ])
        .status()
        .expect("podman");
    assert!(
        built.success(),
        "the environment did not build from its recipe"
    );

    let list = listed(&environment);
    std::fs::write(environment.join("alo-installing.sha256"), &list).expect("the list");
    let the_lists_digest = run(
        "sha256sum",
        &[&environment
            .join("alo-installing.sha256")
            .display()
            .to_string()],
    )
    .split_whitespace()
    .next()
    .expect("a digest")
    .to_owned();

    let program = Command::new("cargo")
        .current_dir(repository)
        .env("ALO_INSTALLER_ENVIRONMENT_SHA256", &the_lists_digest)
        .env("CARGO_TARGET_DIR", target_directory)
        .args([
            "build",
            "--release",
            "-p",
            "alo-installer",
            "--target",
            THE_TARGET,
        ])
        .status()
        .expect("cargo");
    assert!(program.success(), "the installer did not build for Windows");

    let download = yard.join("payload");
    let _ = std::fs::remove_dir_all(&download);
    std::fs::create_dir_all(download.join("alo-installing")).expect("the download");
    std::fs::copy(
        target_directory
            .join(THE_TARGET)
            .join("release/alo-installer.exe"),
        download.join("alo-installer.exe"),
    )
    .expect("the program");
    run(
        "cp",
        &[
            "-r",
            &environment.join("EFI").display().to_string(),
            &environment
                .join("alo-installing.sha256")
                .display()
                .to_string(),
            &download.join("alo-installing").display().to_string(),
        ],
    );
    download
}

/// `sha256sum` over every file of the environment, in the order
/// `LC_ALL=C sort` gives, exactly as the release workflow writes the list.
fn listed(environment: &Path) -> String {
    let printed = Command::new("sh")
        .current_dir(environment)
        .args(["-c", "find EFI -type f | LC_ALL=C sort | xargs sha256sum"])
        .output()
        .expect("the list");
    assert!(
        printed.status.success(),
        "the environment could not be listed"
    );
    String::from_utf8_lossy(&printed.stdout).into_owned()
}
