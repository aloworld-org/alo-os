//! The person, the machine, the screens and the desk this crate's own tests are
//! written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The session is a real sign-in
//!
//! [`anna`] is what `alo-accounts` hands back when a password verifies and the
//! machine description agrees about the number, which is the only way an
//! `alo_accounts::Session` exists. A seat this crate refuses to log out of is
//! therefore a session somebody really opened.
//!
//! # And the desk is two applications on two screens
//!
//! [`a_desk`] is a mail client and an editor, opened in that order, on a laptop
//! and on the screen beside it. It is the fixture every test about order and
//! about the kept list is written against, because the two things that go wrong
//! are the order applications are asked in and what a second window does to a
//! list.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;

use alo_accounts::{Accounts, Session};
use alo_appearance::DisplayId;
use alo_applications::Application;
use alo_strings::{Strings, Vocabulary};

use crate::open::{Open, WasOpen};
use crate::split::Split;

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// This crate's words, and `alo-locking`'s, whose refusal a locked machine
/// answers both of this crate's doors with.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    alo_locking::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The machine's accounts: Anna, at this machine's number.
pub(crate) fn the_machine() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one. Made once: hashing a password
/// is deliberately slow.
pub(crate) fn anna() -> Session {
    static ANNA: OnceLock<Session> = OnceLock::new();
    ANNA.get_or_init(|| {
        let signed_in = the_machine().signs_in("anna", ANNAS).unwrap();
        Session::opened(signed_in, PERSON).unwrap()
    })
    .clone()
}

/// The identifier of the editor on this desk.
pub(crate) const fn the_editor() -> &'static str {
    "org.example.Editor"
}

/// The laptop's own screen.
pub(crate) fn the_laptop() -> DisplayId {
    DisplayId::named("eDP-1 Built-in display").unwrap()
}

/// The screen beside it.
pub(crate) fn the_screen() -> DisplayId {
    DisplayId::named("DP-1 Dell U2720Q").unwrap()
}

/// An application by identifier, for the tests that only need one.
pub(crate) fn an_application(identifier: &str) -> Application {
    Application::identified(identifier).unwrap()
}

/// What Anna has open: mail on the laptop, then the editor on the screen beside
/// it, opened in that order.
pub(crate) fn a_desk() -> WasOpen {
    WasOpen::nothing()
        .and(Open::of("org.example.Mail", the_laptop(), Split::TheWholeScreen).unwrap())
        .and(Open::of(the_editor(), the_screen(), Split::TheWholeScreen).unwrap())
}

/// A folder under the temporary directory that is this test's alone, emptied.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-leaving-{what}-{}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
