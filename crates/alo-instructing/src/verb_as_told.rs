//! One verb, as a model is told about it.
//!
//! What a model is told each verb does is
//! [`alo_capability::Verb::purpose_as_written`] and
//! [`alo_capability::Arg::purpose_as_written`] — the sentence a translator is
//! handed, not a second description written for a prompt. Item 9b's rule, two
//! crates on: two descriptions of one verb are two things that can disagree, and
//! the one the model saw would be the one nobody maintains.
//!
//! The door is [`alo_capability::Effect`]'s, read off the verb rather than
//! written beside it, so a verb that changes something can never be described to
//! a model as a read.

use alo_capability::{Effect, Offered, Takes, Verb};

/// One verb, as a model is told about it: its name, its door, what it is for,
/// and every argument it takes with the bound that argument declared.
#[must_use]
pub fn as_told(verb: &Verb) -> String {
    let door = match verb.effect() {
        Effect::Read => "read",
        Effect::Change => "propose",
    };
    let mut text = format!(
        "\n\n- {} ({door}) — {}",
        verb.name(),
        verb.purpose_as_written()
    );
    for arg in verb.args() {
        text.push_str(&format!(
            "\n  - {} ({}) — {}",
            arg.name(),
            takes(arg.takes()),
            arg.purpose_as_written()
        ));
    }
    text
}

/// What one argument accepts, in a clause.
///
/// The bounds are the argument's own, because a model told *a whole number* and
/// then refused for sending 5000 has been measured on something nobody asked it.
fn takes(takes: &Takes) -> String {
    match takes {
        Takes::Path => "a full path".to_owned(),
        Takes::Application => "an installed application's identifier".to_owned(),
        Takes::Name { longest } => format!("one name, at most {longest} characters"),
        Takes::Count { least, most } => format!("a whole number from {least} to {most}"),
        Takes::Choice(options) => {
            let names: Vec<&str> = options.iter().map(Offered::name).collect();
            format!("one of: {}", names.join(", "))
        }
    }
}
