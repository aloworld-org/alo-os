//! **The program a person downloads starts on a Windows that has never built
//! anything**, which is the first thing promise 22 owes and the thing it failed.
//!
//! On 2026-10-05 the published `alo-installer.exe` was run on a clean Windows 11
//! Pro, elevated, and died in the loader before `main` with `0xC0000135`,
//! `STATUS_DLL_NOT_FOUND`: it imports `VCRUNTIME140.dll`, the zip ships no DLL
//! beside it, and a machine with no Rust toolchain has no Visual C++
//! redistributable. **Nothing ran and nothing printed.**
//!
//! # Why this runs only on Windows, and why that is not a hole
//!
//! It reads the **built executable**, so it needs a build that produces one.
//! `#![cfg(windows)]` means it compiles to nothing elsewhere rather than passing
//! vacuously on a Linux gate — *a test that cannot fail is a comment*, and a
//! green tick from a machine that never had the file would be worse than no
//! test.
//!
//! What stops it being forgotten is the other half:
//! `tests/how_the_installer_is_released.rs` holds `release.yml` to running it,
//! and that test **does** run everywhere. So the Linux gate enforces *the
//! workflow checks this*, and the Windows runner enforces *the file is clean*.
//!
//! # And it refuses rather than skips
//!
//! If the executable is not where a release build puts it, this fails. A test
//! that quietly passes when its subject is missing is how an unbuilt artefact
//! reports as a working one, which is the shape of every fault this repository
//! has spent the week finding.

#![cfg(windows)]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_installer::{A_REDISTRIBUTABLE_LIBRARY, TheImports};

/// Where a release build leaves the program.
const WHERE_IT_IS_BUILT: &str = "target/release/alo-installer.exe";

/// The workspace root, from this crate's own manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two directories above this crate")
}

/// **Every library the shipped program asks for is one a clean Windows has.**
#[test]
fn the_shipped_program_asks_for_nothing_a_clean_windows_lacks() {
    let at = the_repository().join(WHERE_IT_IS_BUILT);
    let bytes = std::fs::read(&at).unwrap_or_else(|why| {
        panic!(
            "{WHERE_IT_IS_BUILT} could not be read ({why}), and this test exists to read the \
             program a person downloads -- build it with `cargo build --release --package \
             alo-installer` before running this, and never let it pass without the file"
        )
    });

    let imports = TheImports::read(&bytes).unwrap_or_else(|why| {
        panic!("{WHERE_IT_IS_BUILT} is not a readable Windows program: {why}")
    });

    let lacking = imports.what_a_clean_windows_lacks();
    assert!(
        lacking.is_empty(),
        "{WHERE_IT_IS_BUILT} imports {lacking:?}, which is a Visual C++ redistributable library \
         ({A_REDISTRIBUTABLE_LIBRARY:?}) and not something a Windows that has never built \
         anything has. The loader refuses the program before \
         `main`, with 0xC0000135 and no output at all, which is promise 22 failing at *click*. \
         Build it with the static C runtime -- `.cargo/config.toml` sets \
         `target-feature=+crt-static` for this target -- rather than shipping a DLL beside it \
         or asking a person to install a Microsoft redistributable first (ADR 0023's road is \
         download, click, reboot)"
    );
}

/// **It really did read an import table**, and is not reporting a clean answer
/// because it read nothing at all.
///
/// A program that links the C runtime statically still imports `KERNEL32`: it
/// is how a Windows process reaches the operating system, and a table without it
/// means the reader found the wrong bytes rather than that the program is tidy.
#[test]
fn the_reader_found_a_real_import_table() {
    let at = the_repository().join(WHERE_IT_IS_BUILT);
    let bytes = std::fs::read(&at)
        .unwrap_or_else(|why| panic!("{WHERE_IT_IS_BUILT} could not be read: {why}"));
    let imports = TheImports::read(&bytes).unwrap_or_else(|why| {
        panic!("{WHERE_IT_IS_BUILT} is not a readable Windows program: {why}")
    });

    assert!(
        imports.asks_for("KERNEL32.dll"),
        "{WHERE_IT_IS_BUILT} imports {:?}, which does not include KERNEL32 -- every Windows \
         program reaches the system through it, so this reader found the wrong bytes and its \
         clean answer means nothing",
        imports.names()
    );
}
