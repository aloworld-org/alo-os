//! What the tests share: lines a model could write, and a model that writes
//! them in order.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_capability::{Arg, Given, Takes, Verbs};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::a_try::TheModel;

/// The machine's verbs, from the one place that builds them.
pub fn the_verbs() -> Verbs {
    alo_turn::the_verbs_a_whole_machine_offers().expect("the machine's verbs declare")
}

/// The words the answers are said in.
pub fn the_words() -> Strings {
    crate::words_for_the_answers::the_words().expect("the answers' words join")
}

/// A value for `arg` the machine accepts: `decided`, where a road decides
/// it, and otherwise the least the argument allows.
pub fn a_value_for(arg: &Arg, decided: Option<String>) -> Given {
    match (arg.takes(), decided) {
        (Takes::Count { least, .. }, None) => Given::number(*least),
        (Takes::Count { .. }, Some(is)) => {
            Given::number(is.parse().expect("a count a road decides is a number"))
        }
        (Takes::Choice(offered), None) => Given::text(
            offered
                .first()
                .expect("a choice offers something")
                .name(),
        ),
        (_, Some(is)) => Given::text(is),
        (Takes::Path, None) => Given::text(
            std::path::Path::new(crate::THE_HOME)
                .join(alo_their_folders::Folder::Documents.xdg_name())
                .display()
                .to_string(),
        ),
        (Takes::Application, None) => Given::text(
            alo_software::Shipped::decided()
                .expect("the shipped list reads")
                .the(alo_software::Role::WebBrowser)
                .application()
                .identifier(),
        ),
        (_, None) => Given::text("a"),
    }
}

/// One argument as a model writes it.
fn given(arguments: &[(&str, &str)]) -> String {
    arguments
        .iter()
        .map(|(named, is)| format!(r#"{{"named":"{named}","is":"{is}"}}"#))
        .collect::<Vec<_>>()
        .join(",")
}

/// A change, as a model writes it.
pub fn proposing(verb: &str, arguments: &[(&str, &str)]) -> String {
    format!(
        r#"{{"format":1,"asks":{{"propose":{{"verb":"{verb}","given":[{}]}}}}}}"#,
        given(arguments)
    )
}

/// A read, as a model writes it.
pub fn reading(verb: &str, arguments: &[(&str, &str)]) -> String {
    format!(
        r#"{{"format":1,"asks":{{"read":{{"verb":"{verb}","given":[{}]}}}}}}"#,
        given(arguments)
    )
}

/// A command, as a model writes it.
pub fn running(command: &str) -> String {
    format!(r#"{{"format":1,"asks":{{"run":{{"command":"{command}"}}}}}}"#)
}

/// An answer to the person, as a model writes it.
pub fn answering(text: &str) -> String {
    format!(r#"{{"format":1,"asks":{{"answer":{{"text":"{text}"}}}}}}"#)
}

/// What the protocol reads a line as.
pub fn read(line: &str) -> FromAnAgent {
    FromAnAgent::read(line).expect("a line the protocol reads")
}

/// A model that writes these lines, one per question, and keeps every
/// question it was shown.
pub struct Writing {
    lines: Vec<String>,
    pub shown: Vec<String>,
}

impl Writing {
    pub fn these(lines: &[String]) -> Self {
        Self {
            lines: lines.iter().rev().cloned().collect(),
            shown: Vec::new(),
        }
    }
}

impl TheModel for Writing {
    fn answering(&mut self, shown: &str, strings: &Strings) -> Result<ToAnAgent, String> {
        self.shown.push(shown.to_owned());
        let line = self
            .lines
            .pop()
            .ok_or_else(|| "the test's model has nothing more to say".to_owned())?;
        let came_from = alo_models::InferenceSource::ThisMachine;
        Ok(ToAnAgent::answered(
            &line,
            &came_from.said(strings),
            "a-test-model",
        ))
    }
}

/// A model that never answers, as a provider that is down.
pub struct Down;

impl TheModel for Down {
    fn answering(&mut self, _shown: &str, _strings: &Strings) -> Result<ToAnAgent, String> {
        Err("nothing answered".to_owned())
    }
}
