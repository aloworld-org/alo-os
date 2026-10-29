//! The name a desktop is told, held to reaching the strip and to never being
//! the login's.

use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt as _;

use crate::Server;
use crate::the_terminals_strip::{WhatTheStripSays, where_and_who};

/// A session in a private directory of its own.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// The terminal's words, in English.
fn says() -> alo_strings::Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).expect("the terminal's own words declare");
    alo_strings::Strings::of(vocabulary)
}

/// What the strip says for whatever this server was told, as `drawn_terminal`
/// builds it.
fn the_strip_of(server: &Server) -> String {
    where_and_who(
        &says(),
        WhatTheStripSays {
            the_folder: None,
            runs: alo_terminal::WhereTheShellRuns::OnThisMachine,
            who: server.whose_desktop_this_is(),
        },
    )
}

/// **A name the desktop is told reaches the strip**, joined as the design
/// joins it.
#[test]
fn a_name_the_desktop_is_told_reaches_the_strip() {
    let (_runtime, mut server) = a_session("whose-anna");
    server.its_person_is_called("anna".to_owned());
    assert_eq!(the_strip_of(&server), "Local \u{b7} anna");
}

/// **A desktop nobody told says `Local` alone** — the account made before
/// names were given — and never the login's own name, `alo`.
#[test]
fn a_desktop_with_no_name_says_where_alone_and_never_alo() {
    let (_runtime, server) = a_session("whose-nobody");
    let said = the_strip_of(&server);
    assert_eq!(said, "Local");
    assert!(!said.contains("alo"), "{said}");
}

/// **An empty name is no name**, not `Local · ` with nothing after the dot.
#[test]
fn an_empty_name_is_kept_as_no_name() {
    let (_runtime, mut server) = a_session("whose-empty");
    server.its_person_is_called(String::new());
    assert_eq!(server.whose_desktop_this_is(), None);
    assert_eq!(the_strip_of(&server), "Local");
}

/// **A desktop that answers nothing by default answers no name**, so an
/// implementor that has not been taught to read the file draws `Local` alone.
#[test]
fn the_default_desktop_answers_no_name() {
    struct NoName;
    impl crate::TheDesktop for NoName {
        fn now(&self) -> crate::DesktopFrame<'_> {
            unreachable!("only the name is asked for")
        }
    }
    assert_eq!(crate::TheDesktop::whose_desktop_this_is(&mut NoName), None);
}
