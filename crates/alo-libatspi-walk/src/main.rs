//! **An accessibility tree, walked by libatspi itself** — the C library Orca
//! and every other screen reader on this desktop read through.
//!
//! A test instrument and nothing else. `tests/libatspi_reads_alos_tree.rs`
//! serves alo's shell tree on a real at-spi2 bus, runs this against it, and
//! holds what libatspi read — and everything libatspi said on its standard
//! error — to the tree that was served.
//!
//! ```text
//! alo-libatspi-walk one-question-at-a-time | cached
//! ```
//!
//! It reads the session's accessibility bus the way any client does, through
//! `DBUS_SESSION_BUS_ADDRESS`, and prints one line per thing (`walk`'s
//! documentation has the shape). It exits 0 when it walked, 2 when it was
//! asked wrongly, and 3 when it could not reach a tree.
//!
//! # Why this is its own package, and why it has `unsafe` in it
//!
//! Until 2026-11-13 the shell's tree was read back only by `alo-adapters`, a
//! reader we wrote on the same D-Bus library the shell answers with, and a
//! fault in how that library shapes a reply passed between the two unseen —
//! while Orca, on a booted machine, could not read a single child
//! (`docs/misreadings/a-reader-we-wrote-agreed-with-the-tree-we-wrote.md`). A
//! screen reader's own library is what has to read it. That library is C, and
//! calling C is `unsafe`.
//!
//! The workspace **forbids** `unsafe_code`, which no module can lift, so this
//! package has its own lint table — the workspace's, copied, with
//! `unsafe_code` at **deny** — and exactly one file, `libatspi`, allows it.
//! The same shape as `alo-bounding-kernel`'s one file that talks to the kernel.
//! No Rust binding of libatspi itself is maintained (the `atspi` crate is a
//! second D-Bus implementation, which is the thing that cannot be used), so the
//! twelve functions are opened by soname with `libloading` rather than
//! through a binding. Nothing that ships depends on this package.

#![deny(unsafe_code)]

#[cfg(target_os = "linux")]
mod libatspi;
#[cfg(target_os = "linux")]
mod walk;

use std::process::ExitCode;

/// Walk the tree the way the one argument says, and print what libatspi read.
#[cfg(target_os = "linux")]
fn main() -> ExitCode {
    let how = match std::env::args().nth(1).as_deref() {
        Some("one-question-at-a-time") => walk::How::OneQuestionAtATime,
        Some("cached") => walk::How::Cached,
        _ => {
            eprintln!("usage: alo-libatspi-walk one-question-at-a-time | cached");
            return ExitCode::from(2);
        }
    };
    match walk::walked(how) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(why) => {
            println!("{why}");
            ExitCode::from(3)
        }
    }
}

/// libatspi is a Linux library, and so is every accessibility bus this reads.
#[cfg(not(target_os = "linux"))]
fn main() -> ExitCode {
    eprintln!("alo-libatspi-walk reads an at-spi2 bus, which only Linux has");
    ExitCode::from(2)
}
