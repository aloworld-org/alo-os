//! **A grant for this session runs until the person signs out**, and is gone
//! the moment the session is cleared — at sign-out, and again at the next
//! sign-in in case the machine lost power first.
//!
//! The owner's decision of 2026-11-16 (`docs/design/when-alo-asks-for-an-application.md`):
//! *"This session" lasts until sign-out: a new additive grant kind, removed at
//! sign-out and again at the next sign-in.*

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use crate::{ATerminalForTheAgent, Ask, Grant, GrantError, Grantee, Grants, Reach, Until};

/// The agent every test here grants to.
fn alo() -> Grantee {
    Grantee::named("alo")
}

/// A moment.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// VSCodium, for this session.
fn codium_for_the_session() -> Grant {
    Grant::checked_until_sign_out(
        &alo(),
        Reach::Application("com.vscodium.codium".to_owned()),
        noon(),
        ATerminalForTheAgent::NotAtAll,
    )
    .unwrap()
}

/// Whether alo may use VSCodium at this moment.
fn reaches(grants: &Grants, at: SystemTime) -> bool {
    grants.permits(&alo(), &Ask::application("com.vscodium.codium"), at)
}

/// **It reaches for as long as the session lasts**, however long that is, and
/// is neither *until removed* nor a length that runs out.
#[test]
fn a_session_grant_reaches_until_the_session_is_cleared() {
    let grant = codium_for_the_session();
    assert!(grant.lasts_until_sign_out());
    assert!(!grant.lasts_until_removed());
    assert_eq!(grant.expires_in(noon()), None);
    let mut grants = Grants::default();
    grants.grant(grant);
    let a_week = Duration::from_secs(7 * 24 * 60 * 60);
    assert!(reaches(&grants, noon()));
    assert!(reaches(&grants, noon() + a_week));
}

/// **Clearing the session takes every session grant away at once, and
/// nothing else**: an hour's grant and one until removed stay.
#[test]
fn clearing_the_session_takes_only_the_session_grants() {
    let mut grants = Grants::default();
    grants.grant(codium_for_the_session());
    grants.grant(
        Grant::checked(
            "alo",
            Reach::Application("org.mozilla.firefox".to_owned()),
            noon(),
            Duration::from_secs(3600),
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked_until_removed(
            &alo(),
            Reach::Folder("/home/anna/Invoices".into()),
            noon(),
            ATerminalForTheAgent::NotAtAll,
        )
        .unwrap(),
    );
    assert_eq!(grants.the_session_is_over(), 1);
    assert!(!reaches(&grants, noon()), "a session grant outlived its session");
    assert!(grants.permits(&alo(), &Ask::application("org.mozilla.firefox"), noon()));
    assert!(grants.permits(&alo(), &Ask::path("/home/anna/Invoices/a.pdf"), noon()));
    assert_eq!(grants.the_session_is_over(), 0, "nothing left to clear");
}

/// **Every other rule still holds**: a session grant to nobody, over the whole
/// machine, or over a terminal is refused exactly as an hour's is.
#[test]
fn a_session_grant_is_checked_like_any_other() {
    let nobody = Grant::checked_until_sign_out(
        &Grantee::named(""),
        Reach::Application("com.vscodium.codium".to_owned()),
        noon(),
        ATerminalForTheAgent::NotAtAll,
    );
    assert_eq!(nobody, Err(GrantError::Anonymous));
    let everything = Grant::checked_until_sign_out(
        &alo(),
        Reach::Folder("/".into()),
        noon(),
        ATerminalForTheAgent::NotAtAll,
    );
    assert_eq!(everything, Err(GrantError::TheWholeMachine));
    let terminal = Grant::checked_until_sign_out(
        &alo(),
        Reach::Application("org.kde.konsole".to_owned()),
        noon(),
        ATerminalForTheAgent::NotAtAll,
    );
    assert_eq!(terminal, Err(GrantError::APersonsOwn));
}

/// **It is written as the word `sign-out`, and only that word reads back as
/// it**: a moment and `removed` still read as themselves, and any other word
/// is refused rather than taken for a grant that does not end.
#[test]
fn it_is_written_as_sign_out_and_no_other_word_reads_as_it() {
    let written = serde_json::to_string(&Until::sign_out()).unwrap();
    assert_eq!(written, "\"sign-out\"");
    let back: Until = serde_json::from_str(&written).unwrap();
    assert!(back.is_until_sign_out());
    let removed: Until = serde_json::from_str("\"removed\"").unwrap();
    assert!(removed.is_until_removed() && !removed.is_until_sign_out());
    for word in ["\"Sign-out\"", "\"signout\"", "\"session\"", "\"\""] {
        assert!(serde_json::from_str::<Until>(word).is_err(), "{word} was read");
    }
}
