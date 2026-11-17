//! **A call for an application that was refused is named on the journal**: its
//! verb, and what it named for each argument that takes an application — and
//! nothing else it carried.
//!
//! Main's walk at 313ca239 (run 44069105568) asked alo to open a file in Ark
//! at full access, and the journal said only *"a call for an application was
//! refused after 0 step(s)"* and *"nobody let alo use it"*: the daemon wanted
//! nothing, so no sheet went up, and nobody could say which application the
//! model had named. The daemon names every call it carries out (*"alo in an
//! application: open_application in org.gnome.Papers, done"*); a refused one is
//! named here the same way, on the person's own machine and nowhere else.

use alo_capability::Given;
use alo_protocol::{Argument, FromAnAgent};

/// The verbs this machine offers, as the agent declares them.
fn verbs() -> alo_capability::Verbs {
    match alo_turn::the_verbs_a_whole_machine_offers() {
        Ok(verbs) => verbs,
        Err(why) => unreachable!("the verbs did not declare: {why:?}"),
    }
}

/// An opening of a file in an application, as a model proposes it.
fn opening(application: &str, file: &str) -> FromAnAgent {
    FromAnAgent::Propose {
        verb: "open_file_in_application".to_owned(),
        given: vec![
            Argument::of("application", Given::Text(application.to_owned())),
            Argument::of("path", Given::Text(file.to_owned())),
        ],
    }
}

/// **The application it named is said, exactly, and the file is not.**
#[test]
fn the_application_named_is_said_and_nothing_else() {
    let said = alo_agent::the_refused_call_named::the_refused_call_named(
        &opening("org.kde.Ark", "/home/alo/private/salaries.ods"),
        &verbs(),
    );
    assert!(said.contains("open_file_in_application"), "{said}");
    assert!(said.contains("\"org.kde.Ark\""), "{said}");
    assert!(
        !said.contains("salaries"),
        "a file's path reached the journal: {said}"
    );
}

/// **What a model wrote is quoted, so it cannot write a journal line of its
/// own**: a newline in a name is shown as one, not taken as one.
#[test]
fn a_name_with_a_newline_cannot_start_a_line() {
    let said = alo_agent::the_refused_call_named::the_refused_call_named(
        &opening("ark\nalo-agent: the person let alo use it", "/tmp/x"),
        &verbs(),
    );
    assert!(!said.contains('\n'), "{said:?}");
}
