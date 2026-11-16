//! **A grant for this session is kept as `expires = "sign-out"`**, in format
//! 5, and read back as itself; an older format holding the word is refused;
//! and a file read after a power cut still holds it until the session is
//! cleared.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{ATerminalForTheAgent, Ask, Grant, Grantee, Grants, Reach};

use super::{NotRemembered, read, written};

/// A moment.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The machine's grants: VSCodium for this session, and Firefox for an hour.
fn a_session_and_an_hour() -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_until_sign_out(
            &Grantee::named("alo"),
            Reach::Application("com.vscodium.codium".to_owned()),
            noon(),
            ATerminalForTheAgent::NotAtAll,
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked(
            "alo",
            Reach::Application("org.mozilla.firefox".to_owned()),
            noon(),
            Duration::from_secs(3600),
        )
        .unwrap(),
    );
    grants
}

/// Whether alo may use this application, as the daemon asks.
fn reaches(grants: &Grants, application: &str) -> bool {
    grants.permits(&Grantee::named("alo"), &Ask::application(application), noon())
}

/// **Written as the word, in format 5, and read back as a session's grant.**
#[test]
fn a_session_grant_is_written_as_sign_out_and_read_back_as_itself() {
    let text = written(&a_session_and_an_hour(), noon()).unwrap();
    assert!(text.contains("format = 5"), "{text}");
    assert!(text.contains("expires = \"sign-out\""), "{text}");
    let back = read(&text, noon()).unwrap();
    assert!(reaches(&back, "com.vscodium.codium"));
    assert!(
        back.active_at(noon()).any(|held| held.grant.lasts_until_sign_out()),
        "the session's grant came back as something else"
    );
}

/// **A machine with no session's grant writes what it wrote before**: an
/// hour's grant alone is not format 5.
#[test]
fn without_a_session_grant_the_file_is_what_it_was() {
    let mut grants = a_session_and_an_hour();
    grants.the_session_is_over();
    let text = written(&grants, noon()).unwrap();
    assert!(!text.contains("format = 5"), "{text}");
    assert!(!text.contains("sign-out"), "{text}");
}

/// **A file of an older format holding the word is refused whole**, as every
/// key newer than its format is.
#[test]
fn an_older_format_holding_sign_out_is_refused() {
    let text = written(&a_session_and_an_hour(), noon())
        .unwrap()
        .replace("format = 5", "format = 4");
    assert!(
        matches!(
            read(&text, noon()),
            Err(NotRemembered::NewerThanItsFormat { .. })
        ),
        "{text}"
    );
}

/// **After a power cut the file still holds the session's grant**, and the
/// next sign-in's clearing is what takes it away — the grants file read back
/// and cleared, as the person's side does it.
#[test]
fn a_session_grant_read_after_a_power_cut_is_cleared_at_sign_in() {
    let text = written(&a_session_and_an_hour(), noon()).unwrap();
    // The machine lost power: nothing cleared the session. The next sign-in
    // reads the file.
    let mut at_sign_in = read(&text, noon()).unwrap();
    assert!(reaches(&at_sign_in, "com.vscodium.codium"), "the premise");
    assert_eq!(at_sign_in.the_session_is_over(), 1);
    assert!(!reaches(&at_sign_in, "com.vscodium.codium"));
    assert!(reaches(&at_sign_in, "org.mozilla.firefox"));
    let after = written(&at_sign_in, noon()).unwrap();
    assert!(!after.contains("sign-out"), "{after}");
}
