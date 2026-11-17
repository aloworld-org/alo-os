//! **A refused call for an application, named for the person's journal**: its
//! verb, and what it named for each argument that takes an application.
//!
//! The daemon names every call it carries out on the journal (*"alo in an
//! application: open_application in org.gnome.Papers, done"*). A refused one
//! was named nowhere: main's walk at 313ca239 (run 44069105568) saw *"a call
//! for an application was refused after 0 step(s)"*, no sheet, and no way to
//! say which application the model had asked for. This is that name.
//!
//! **Only the application.** A call's other arguments — a file's path, text to
//! type — never reach the journal from here. And what the model wrote is
//! quoted, so a name with a newline in it cannot write a line of its own.

use alo_capability::{Given, Takes, Verbs};
use alo_protocol::FromAnAgent;

/// `open_file_in_application naming "org.kde.ark"`: the call's verb and, for
/// each argument it declares as taking an application, what the model gave,
/// quoted. *naming nothing* where it gave none.
#[must_use]
pub fn the_refused_call_named(call: &FromAnAgent, verbs: &Verbs) -> String {
    let (FromAnAgent::Propose { verb, given } | FromAnAgent::Read { verb, given }) = call else {
        return "a call that names no application".to_owned();
    };
    let applications: Vec<&str> = verbs.of(verb).map_or_else(Vec::new, |declared| {
        declared
            .args()
            .iter()
            .filter(|arg| *arg.takes() == Takes::Application)
            .map(alo_capability::Arg::name)
            .collect()
    });
    let named: Vec<String> = given
        .iter()
        .filter(|argument| applications.contains(&argument.named()))
        .filter_map(|argument| match argument.given() {
            Given::Text(said) => Some(format!("{said:?}")),
            Given::Number(_) => None,
        })
        .collect();
    let verb = verb.escape_debug();
    if named.is_empty() {
        format!("{verb} naming nothing")
    } else {
        format!("{verb} naming {}", named.join(", "))
    }
}
