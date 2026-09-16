//! The converting service (ADR 0039, option A).
//!
//! Started by systemd from `alo-convertd.service`, which `alo-convertd.socket`
//! activates and hands its listening socket **on standard input**
//! (`StandardInput=socket`). That is what lets a program that forbids `unsafe`
//! take a socket it did not open: standard input is a descriptor `std` already
//! owns. A test starts it the same way.
//!
//! It runs the pinned engine from its one fixed path, works in its own
//! temporary folder — private to its unit — and answers until the socket
//! closes. Everything it does is `alo_converting::serving`'s.

use std::process::ExitCode;

#[cfg(target_os = "linux")]
fn main() -> ExitCode {
    use std::os::fd::AsFd as _;
    use std::os::unix::net::UnixListener;
    use std::path::Path;

    use alo_converting::engine::THE_ENGINE;
    use alo_converting::serving::Serving;

    let listener = match std::io::stdin().as_fd().try_clone_to_owned() {
        Ok(owned) => UnixListener::from(owned),
        Err(why) => {
            eprintln!("alo-convertd: standard input could not be taken: {why}");
            return ExitCode::FAILURE;
        }
    };
    let serving = Serving::with(Path::new(THE_ENGINE), &std::env::temp_dir());
    match serving.serve(&listener) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("alo-convertd: standard input is not a socket this can listen on: {why}");
            ExitCode::FAILURE
        }
    }
}

/// A host with no converting service: there is nothing to start.
#[cfg(not(target_os = "linux"))]
fn main() -> ExitCode {
    eprintln!("alo-convertd: documents are converted on a Linux machine");
    ExitCode::FAILURE
}
