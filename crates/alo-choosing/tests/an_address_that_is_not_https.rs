//! An address that is not https is refused, unless it is a service on this
//! machine — at the one place a provider is written.
//!
//! `docs/features.md`, v0.5: *an address that is not https is refused rather
//! than warned about, unless it is a service on this machine — "it is only our
//! internal network" is how a key ends up on the wire in clear.* This file is
//! that sentence as tests, one per clause, each against a settings file on a
//! real disk and the vocabulary the whole machine loads rather than this
//! crate's own list.
//!
//! The rule itself is `alo_models::Provider::checked`'s and has been since
//! providers existed. What this file measures is the half that was missing:
//! `alo_models::Provider` has public fields, so a value can reach
//! `alo_choosing::Choosing` that nobody judged, and it is now judged again in
//! the one function every door writes through — refused as what it is, in
//! `alo-models`' words, and never as a defect in alo OS.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_choosing::{Choosing, NotSet, NotWritten, Picked, Settings, where_it_is};
use alo_models::{Provider, ProviderError, Region, SecretRef};
use alo_strings::Strings;

/// A home directory of this test's own, with nothing in it.
fn a_login_of_our_own(what: &str) -> PathBuf {
    let home = std::env::temp_dir().join(format!("alo-choosing-https-{what}"));
    if home.exists() {
        std::fs::remove_dir_all(&home).unwrap();
    }
    std::fs::create_dir_all(&home).unwrap();
    home
}

/// Where this session's settings are, worked out the way the daemon works it
/// out.
fn the_settings_of(home: &Path) -> PathBuf {
    where_it_is(None, Some(home.as_os_str())).expect("a login with a home directory has settings")
}

/// Everything the machine can say, which is what a process really holds.
fn everything_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A provider built the way nothing should build one: by hand, from the public
/// fields, with an address `Provider::checked` never saw. This is the value a
/// settings surface holds after somebody edits the address of a provider it
/// had already checked.
fn by_hand(name: &str, endpoint: &str) -> Provider {
    Provider {
        name: name.to_owned(),
        endpoint: endpoint.to_owned(),
        region: Region::Unknown,
        key: Some(SecretRef::named(&format!("provider/{name}"))),
        models: Vec::new(),
    }
}

/// A provider built the way a settings surface should build one.
fn checked(name: &str, endpoint: &str) -> Provider {
    Provider::checked(
        name,
        endpoint,
        Region::Declared("the EU".to_owned()),
        Some(SecretRef::named(&format!("provider/{name}"))),
    )
    .unwrap()
}

/// Settings with one https provider on the list and chosen, on a real disk,
/// and the bytes of the file that holds them.
fn a_machine_with_a_provider(what: &str) -> (Choosing, PathBuf, String) {
    let home = a_login_of_our_own(what);
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .adding(checked("Mistral", "https://api.mistral.ai"))
        .unwrap();
    choosing
        .answered_by(Some(
            Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap(),
        ))
        .unwrap();
    let before = std::fs::read_to_string(&at).unwrap();
    (choosing, at, before)
}

/// **Adding a provider whose address is not https is refused before anything
/// is written**: the file is byte for byte what it was, the value holds what
/// the file holds, and the refusal says why in words the machine collects.
#[test]
fn adding_an_address_that_is_not_https_is_refused_and_the_file_is_byte_for_byte_what_it_was() {
    let (mut choosing, at, before) = a_machine_with_a_provider("adding");
    let strings = everything_this_machine_can_say();

    for endpoint in [
        "http://api.example.com",
        "http://api.example.com:443/v1",
        "ftp://api.example.com",
        "api.example.com",
    ] {
        let refused = choosing.adding(by_hand("Somewhere", endpoint)).unwrap_err();

        assert!(
            matches!(refused, NotWritten::NotAProvider { .. }),
            "{endpoint}: {refused:?}"
        );
        assert_eq!(refused.at(), at, "{endpoint}");
        assert_eq!(std::fs::read_to_string(&at).unwrap(), before, "{endpoint}");
        assert_eq!(
            *choosing.settings(),
            Settings::at(&at).unwrap(),
            "{endpoint}"
        );
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{endpoint}: {said}");
        assert!(said.text().contains("https"), "{endpoint}: {said}");
        assert!(
            !said.text().contains("alo OS could not"),
            "{endpoint}: refused as a defect rather than as the rule: {said}"
        );
    }
}

/// **Changing a provider to an address that is not https is refused the same
/// way**, and the provider keeps the address it had. This is the case the
/// sentence in `docs/features.md` is written against: a working provider edited
/// to point at *only our internal network*.
#[test]
fn changing_to_an_address_that_is_not_https_is_refused_and_the_file_is_byte_for_byte_what_it_was() {
    let (mut choosing, at, before) = a_machine_with_a_provider("changing");

    let refused = choosing
        .changing(by_hand("Mistral", "http://192.168.1.10:8080/v1"))
        .unwrap_err();

    assert!(
        matches!(
            refused,
            NotWritten::NotAProvider {
                why: ProviderError::InsecureEndpoint,
                ..
            }
        ),
        "{refused:?}"
    );
    assert_eq!(std::fs::read_to_string(&at).unwrap(), before);
    assert_eq!(
        Settings::at(&at).unwrap().provider().unwrap().endpoint,
        "https://api.mistral.ai"
    );
    let said = refused.said(&everything_this_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("unencrypted"), "{said}");
}

/// **A loopback address on any scheme is the one exception**, and is accepted:
/// `127.0.0.1`, `::1` and `localhost`, over `http://` and over `https://`. A
/// service on this machine is ADR 0021's own case — nothing travels a wire.
#[test]
fn a_service_on_this_machine_is_accepted_on_any_scheme() {
    let home = a_login_of_our_own("loopback");
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();

    let on_this_machine = [
        ("Four", "http://127.0.0.1:11434"),
        ("Whole-net", "http://127.0.0.2:11434"),
        ("Six", "http://[::1]:8080/v1"),
        ("Named", "http://localhost:8000"),
        ("Rooted", "http://localhost.:8000"),
        ("Secure-four", "https://127.0.0.1:8443"),
        ("Secure-named", "https://localhost:8443"),
    ];
    for (name, endpoint) in on_this_machine {
        let added =
            choosing.adding(Provider::checked(name, endpoint, Region::Unknown, None).unwrap());
        assert!(added.is_ok(), "{endpoint} was refused: {added:?}");
        // And the same address by hand, which is the path the rule is for.
        let changed = choosing.changing(by_hand(name, endpoint));
        assert!(
            changed.is_ok(),
            "{endpoint} by hand was refused: {changed:?}"
        );
    }

    let read = Settings::at(&at).unwrap();
    assert_eq!(read.providers().configured.len(), on_this_machine.len());
    for (name, endpoint) in on_this_machine {
        assert_eq!(read.providers().get(name).unwrap().endpoint, endpoint);
    }
}

/// **An address on the machine's own network is not an exception.** *It is
/// only our internal network* is the sentence the promise names as the
/// failure, so every private range, a link-local address, a unique-local one
/// and a `.local` name are refused over http — and so is anything dressed up
/// to begin like this machine.
#[test]
fn an_address_on_this_machines_own_network_is_not_an_exception() {
    let (mut choosing, at, before) = a_machine_with_a_provider("lan");

    for endpoint in [
        "http://192.168.1.10:11434",
        "http://10.0.0.5",
        "http://172.16.0.1:8080/v1",
        "http://[fe80::1]:8080",
        "http://[fd00::1]:8080",
        "http://nas.local:11434",
        "http://127.1",
        "http://[::ffff:127.0.0.1]:8080",
        "http://localhost.attacker.example",
        "http://127.0.0.1.attacker.example/v1",
        "http://127.0.0.1@attacker.example/",
    ] {
        let refused = choosing.adding(by_hand("Nearby", endpoint)).unwrap_err();
        assert!(
            matches!(
                refused,
                NotWritten::NotAProvider {
                    why: ProviderError::InsecureEndpoint,
                    ..
                }
            ),
            "{endpoint}: {refused:?}"
        );
        assert_eq!(std::fs::read_to_string(&at).unwrap(), before, "{endpoint}");
    }
    assert_eq!(choosing.settings().providers().configured.len(), 1);
}

/// **The rule is applied in the one place a provider is written, so no second
/// path can bypass it.** Every door of `Choosing` that carries a provider to
/// the disk goes through `alo_choosing`'s own writer, and a value that went
/// past `Provider::checked` is judged there: the adding door, the changing
/// door, and — the case that proves it is the writer and not the doors — a
/// provider that was already on the list when another door wrote the file.
#[test]
fn no_door_writes_a_provider_the_writer_has_not_judged() {
    let home = a_login_of_our_own("one-place");
    let at = the_settings_of(&home);
    let mut choosing = Choosing::at(&at).unwrap();
    choosing
        .adding(checked("Mistral", "https://api.mistral.ai"))
        .unwrap();

    // Through the two doors that take a provider.
    let adding = choosing
        .adding(by_hand("Nearby", "http://10.0.0.5"))
        .unwrap_err();
    let changing = choosing
        .changing(by_hand("Mistral", "http://10.0.0.5"))
        .unwrap_err();
    for refused in [adding, changing] {
        assert!(
            matches!(
                refused,
                NotWritten::NotAProvider {
                    why: ProviderError::InsecureEndpoint,
                    ..
                }
            ),
            "{refused:?}"
        );
    }

    // And nothing reached the disk that the writer had not judged: what the
    // file says is what `Settings::at` reads, and it has one provider on it.
    let read = Settings::at(&at).unwrap();
    assert_eq!(read.providers().configured.len(), 1);
    assert_eq!(
        read.providers().get("Mistral").unwrap().endpoint,
        "https://api.mistral.ai"
    );
}

/// **There is no allow-list of hostnames and no environment variable that
/// turns the rule off.** Read off the source rather than promised: the file
/// that holds the rule names no host, no scheme and no exception, and reads
/// nothing from the environment — what an address is, is `alo-models`' answer,
/// asked again rather than copied. A switch would have to be code in this
/// file, and this is the test that fails the day one is written.
#[test]
fn nothing_here_lists_a_hostname_or_reads_the_environment() {
    let holding = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("holding.rs");
    let said = std::fs::read_to_string(&holding).expect("the file that holds the rule reads");
    // The rule's own code, without its tests and its argument.
    let code: String = said
        .lines()
        .take_while(|line| !line.starts_with("#[cfg(test)]"))
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("Provider::checked"),
        "the rule is no longer asked of alo-models:\n{code}"
    );
    for forbidden in [
        "env::",
        "getenv",
        "http://",
        "https://",
        "localhost",
        "127.",
        "::1",
        "starts_with",
        "contains",
    ] {
        assert!(
            !code.contains(forbidden),
            "the rule's own code names `{forbidden}`, which is a second answer to what an address \
             is:\n{code}"
        );
    }
}

/// **A file written before this rule is read as it always was**, and nothing
/// rewrites it behind the person. The reader has built every provider through
/// `Provider::checked` since providers existed, so a file with an address that
/// is not https in it is refused whole on the way in — exactly as before — and
/// the file is byte for byte what it was afterwards. A file with a service on
/// this machine in it reads as it always did.
#[test]
fn a_file_written_before_this_rule_is_read_as_it_always_was_and_never_rewritten() {
    let home = a_login_of_our_own("older-file");
    let at = the_settings_of(&home);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();

    let nearby = "format = 2\n\n[[provider]]\nname = \"Nearby\"\nendpoint = \"http://192.168.1.10:11434\"\nneeds-a-key = false\n";
    std::fs::write(&at, nearby).unwrap();
    let refused = Choosing::at(&at).unwrap_err();
    assert!(
        matches!(
            refused,
            NotSet::NotAProvider {
                why: ProviderError::InsecureEndpoint,
                ..
            }
        ),
        "{refused:?}"
    );
    // The same answer the daemon's door gives, which is the door this crate
    // has always read through.
    assert!(matches!(
        Settings::at(&at).unwrap_err(),
        NotSet::NotAProvider {
            why: ProviderError::InsecureEndpoint,
            ..
        }
    ));
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        nearby,
        "the person's file was edited behind them"
    );
    let said = refused.said(&everything_this_machine_can_say());
    assert!(!said.is_a_bug(), "{said}");

    let local = "format = 2\n\n[[provider]]\nname = \"Local\"\nendpoint = \"http://127.0.0.1:11434\"\nneeds-a-key = false\n";
    std::fs::write(&at, local).unwrap();
    let read = Settings::at(&at).unwrap();
    assert_eq!(
        read.providers().get("Local").unwrap().endpoint,
        "http://127.0.0.1:11434"
    );
    assert_eq!(*Choosing::at(&at).unwrap().settings(), read);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), local);
}
