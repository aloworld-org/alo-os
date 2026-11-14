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
//!
//! # Every argument says it is required, and the verb shows its `given`
//!
//! Walked on a booted machine on 2026-11-14 (`gpt-4o-mini`, VSCodium open):
//! asked to type into the open file, the model sent `accessible.type_text` an
//! argument **named** `Untitled-1` — the value where the name belongs — then
//! `Untitled`, then `accessible.read_window` arguments named `name` and
//! `window`, which it does not take. Each argument's name was in this text
//! already, in a list; what was nowhere was the line those names go into. So
//! each verb now ends with its own `given`, written out with the exact names
//! in the order the verb declares them, and each argument says it is required
//! — which [`alo_capability::Call::of`] has always held it to.

use alo_capability::{Effect, Offered, Takes, Verb};

/// One verb, as a model is told about it: its name, its door, what it is for,
/// every argument it takes with the bound that argument declared, and the
/// `given` those arguments are sent in.
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
            "\n  - {} (required; {}) — {}",
            arg.name(),
            takes(arg.takes()),
            arg.purpose_as_written()
        ));
    }
    text.push_str("\n  given: ");
    text.push_str(&given_as_told(verb));
    text
}

/// **The `given` of one call of this verb**, with every argument's real name
/// and `VALUE` where its value goes — the same placeholder the instructions
/// use for the value, so the model has one word to replace and no second one.
#[must_use]
pub fn given_as_told(verb: &Verb) -> String {
    let each: Vec<String> = verb
        .args()
        .iter()
        .map(|arg| format!("{{\"named\":\"{}\",\"is\":VALUE}}", arg.name()))
        .collect();
    format!("[{}]", each.join(","))
}

/// What one argument accepts, in a clause.
///
/// The bounds are the argument's own, because a model told *a whole number* and
/// then refused for sending 5000 has been measured on something nobody asked it.
///
/// **A path and an application carry an example**, because both have a form a
/// model can get wrong while meaning the right thing — a relative path, or an
/// application's display name where its identifier belongs — and the example
/// shows the form rather than a value to copy.
fn takes(takes: &Takes) -> String {
    match takes {
        Takes::Path => "a full path, such as /home/anna/Documents/notes.txt".to_owned(),
        Takes::Application => {
            "an installed application's identifier, such as org.gnome.TextEditor".to_owned()
        }
        Takes::Name { longest } => format!("one name, at most {longest} characters"),
        Takes::Count { least, most } => format!("a whole number from {least} to {most}"),
        Takes::Text => {
            "text, any number of lines and any length, delivered exactly as written and never run"
                .to_owned()
        }
        Takes::Choice(options) => {
            let names: Vec<&str> = options.iter().map(Offered::name).collect();
            format!("one of: {}", names.join(", "))
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;

    /// **Every argument of every verb is told by its exact name, as required,
    /// and inside the `given` it is sent in** — the line a model writes, with
    /// nothing but the values left to fill.
    #[test]
    fn every_argument_is_told_by_name_as_required_and_in_its_given() {
        for verb in the_verbs().all() {
            let told = as_told(verb);
            for arg in verb.args() {
                assert!(
                    told.contains(&format!("\n  - {} (required; ", arg.name())),
                    "{told}"
                );
                assert!(
                    told.contains(&format!("{{\"named\":\"{}\",\"is\":VALUE}}", arg.name())),
                    "{told}"
                );
            }
            assert!(
                told.ends_with(&format!("\n  given: {}", given_as_told(verb))),
                "{told}"
            );
        }
    }

    /// The `given` lists the arguments in the order the verb declares them,
    /// and a verb that takes nothing is told an empty list.
    #[test]
    fn the_given_is_in_the_order_the_verb_declares() {
        let verbs = the_verbs();
        let moving = verbs.of("move_file").unwrap();
        assert_eq!(
            given_as_told(moving),
            r#"[{"named":"file","is":VALUE},{"named":"into","is":VALUE}]"#
        );
        for verb in verbs.all().filter(|verb| verb.args().is_empty()) {
            assert_eq!(given_as_told(verb), "[]");
        }
    }
}
