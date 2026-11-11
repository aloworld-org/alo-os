//! Asking the desktop to start the application a link opens in.
//!
//! The owner, 2026-11-12: *"Yes, add it (Recommended)"* — a narrow connection
//! by which the portal service asks the desktop to start the chosen
//! application **exactly as a person's press does**, so the window lands on
//! the canvas, focused, as a new window does. The wire is
//! `docs/contracts/application-door.md`'s `open-link` act; this is the
//! backend's side of it as a trait, so the backend is testable without a
//! desktop and the socket is the portal service's to know.
//!
//! # Why not the bus, as a file is opened
//!
//! A file is sent to its opener over the bus (`crate::opening`), which only an
//! application whose entry says `DBusActivatable=true` answers. Firefox,
//! Thunderbird and most mail applications do not, so a link sent that way
//! would open nowhere on a fresh machine. The desktop already starts every
//! application a person presses — over the bus where its entry asks, from its
//! entry's own `Exec` line otherwise — and a link is a press with an address.
//!
//! # A backend with no desktop says so
//!
//! [`NoDesktopToAsk`] is what a backend is built with until it is told how to
//! reach the desktop, and it answers every link [`LinkStarted::NoDesktop`]:
//! the response is a refusal and the record says why. A default that answered
//! *asked* would be a link a person believes opened.

use alo_applications::Link;

/// What became of asking the desktop to start an application with a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStarted {
    /// The desktop was asked and took it: it starts the application with the
    /// link, as a press would. *Asked*, never *done* — the application's own
    /// start is the desktop's to report, as for a press.
    Asked,
    /// The desktop has no entry by that name.
    NotInstalled,
    /// No desktop is there to ask.
    NoDesktop,
    /// The desktop was reached and could not be asked.
    CouldNotAsk,
}

/// Whatever asks the desktop to start an application with a link.
pub trait OpensLinks: Send + Sync {
    /// Ask the desktop to start `application` with `link`.
    fn opened(&self, application: &str, link: &Link) -> LinkStarted;
}

/// A backend that was never told how to reach a desktop.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoDesktopToAsk;

impl OpensLinks for NoDesktopToAsk {
    fn opened(&self, _: &str, _: &Link) -> LinkStarted {
        LinkStarted::NoDesktop
    }
}
