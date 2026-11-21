//! A refusal said by kind, and nothing private in it.

use alo_capability::{Ask, Call, Given, NotGranted, Reach, Refused};

use super::said;

/// An opening of `path` in `application`, as a model asks for one.
fn opening(application: &str, path: &str) -> Call {
    let Ok(verbs) = crate::the_verbs_a_whole_machine_offers() else {
        unreachable!("the verbs declare");
    };
    let Some(verb) = verbs.of("open_file_in_application") else {
        unreachable!("open_file_in_application is offered");
    };
    let Ok(call) = Call::of(
        verb,
        &[
            ("application", Given::text(application)),
            ("path", Given::text(path)),
        ],
    ) else {
        unreachable!("the call is well formed");
    };
    call
}

/// **A call nothing granted is said with what it asked for**: the
/// application by its identifier, as the turn names an application it acts in.
/// Main's walk at 802ab80a (run 44884016105) saw opening a file in VSCodium and
/// in Ark refused with nothing on the journal saying why.
#[test]
fn a_call_nothing_granted_is_said_with_the_application() {
    let refused = Refused::not_granted(
        opening("com.vscodium.codium", "/home/alo/hello.txt"),
        NotGranted::Never {
            agent: "alo".to_owned(),
            wanted: Ask::application("com.vscodium.codium"),
        },
    );
    let line = said(&refused);
    assert!(
        line.starts_with(alo_acting::said_on_the_log::A_CALL_WAS_REFUSED),
        "{line}"
    );
    assert!(line.contains("open_file_in_application"), "{line}");
    assert!(line.contains("nothing granted it"), "{line}");
    assert!(line.contains("\"com.vscodium.codium\""), "{line}");
    assert!(
        !line.contains("hello.txt"),
        "a path reached the journal: {line}"
    );
}

/// **A path is said as a path, never by its name.**
#[test]
fn a_path_is_never_named() {
    let refused = Refused::not_granted(
        opening("org.kde.ark", "/home/alo/salaries.zip"),
        NotGranted::Lapsed {
            agent: "alo".to_owned(),
            reach: Reach::Folder("/home/alo".into()),
            wanted: Ask::path("/home/alo/salaries.zip"),
        },
    );
    let line = said(&refused);
    assert!(line.contains("lapsed"), "{line}");
    assert!(line.contains("a path"), "{line}");
    assert!(
        !line.contains("salaries") && !line.contains("/home"),
        "{line}"
    );
}

/// **A refusal worded where the path was resolved is said by that kind**, and
/// its sentence — which may name the path — is not said at all.
#[test]
fn a_refusal_worded_elsewhere_keeps_its_sentence_off_the_journal() {
    let strings = alo_strings::Strings::of(alo_strings::Vocabulary::default());
    let worded = strings.say(
        &alo_strings::Key::named("files.refused.really-leads-elsewhere")
            .unwrap_or_else(|_| unreachable!("the key is well formed")),
        &alo_strings::Filling::nothing(),
    );
    let refused =
        Refused::worded_elsewhere(opening("org.kde.ark", "/var/home/alo/secret.zip"), worded);
    let line = said(&refused);
    assert!(line.contains("where the path was resolved"), "{line}");
    assert!(
        !line.contains("secret") && !line.contains("/var/home"),
        "{line}"
    );
}
