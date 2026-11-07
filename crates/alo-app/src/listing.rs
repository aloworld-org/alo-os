//! **`alo-app list`**: what is installed, and what each offers — read from the
//! machine at the moment the person asks, and kept nowhere
//! (`docs/contracts/terminal-commands.md`, section 2).
//!
//! On a terminal, one sentence per application. On a pipe, one identifier per
//! line. With `--json`, the contract's shape, carrying `"format": 1`:
//!
//! ```json
//! {"format":1,"applications":[{"identifier":"org.gnome.TextEditor",
//!   "name":"Text Editor","typed_as":"texteditor","short_name":"texteditor",
//!   "terminal":false,"abilities":["open_document","new_window"]}]}
//! ```
//!
//! `typed_as` is what reaches it — its short name, or its identifier where the
//! short name is reserved or shared.

use alo_adapters::Adapters;
use alo_applications::{Application, Installed};
use alo_capability::is_a_persons_own;
use alo_strings::{Filling, Said, Strings};
use serde_json::json;

use crate::naming::{Reached, how_it_is_reached, short_name};
use crate::words;

/// How the answer is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Giving {
    /// Plain sentences.
    Sentences,
    /// One value per line.
    Values,
    /// The contract's `--json`.
    Json,
}

/// The declared abilities of an application, by name.
#[must_use]
pub fn its_abilities(application: &Application, adapters: &Adapters) -> Vec<&'static str> {
    adapters
        .for_application(application.identifier())
        .map(|adapter| adapter.verbs.iter().map(|verb| verb.name).collect())
        .unwrap_or_default()
}

/// What an application offers, in one sentence.
#[must_use]
pub fn what_it_offers(application: &Application, adapters: &Adapters, strings: &Strings) -> Said {
    if is_a_persons_own(application.identifier()) {
        return strings.say(&words::A_TERMINAL.key(), &Filling::nothing());
    }
    let abilities = its_abilities(application, adapters);
    if abilities.is_empty() {
        strings.say(&words::OFFERS_ITS_WINDOW.key(), &Filling::nothing())
    } else {
        strings.say(
            &words::OFFERS_ABILITIES.key(),
            &Filling::of(words::LIST, abilities.join(", ")),
        )
    }
}

/// Every installed application, sorted by identifier.
fn sorted(installed: &Installed) -> Vec<&Application> {
    let mut all: Vec<&Application> = installed.all().collect();
    all.sort_by(|one, other| one.identifier().cmp(other.identifier()));
    all
}

/// The lines `list` prints.
#[must_use]
pub fn listed(
    installed: &Installed,
    adapters: &Adapters,
    giving: Giving,
    strings: &Strings,
) -> Vec<String> {
    let all = sorted(installed);
    match giving {
        Giving::Values => all
            .iter()
            .map(|application| application.identifier().to_owned())
            .collect(),
        Giving::Sentences => all
            .iter()
            .map(|application| {
                let reached = how_it_is_reached(application, installed);
                strings
                    .say(
                        &words::LIST_LINE.key(),
                        &Filling::of(words::NAME, application.shown(strings))
                            .and_said(
                                words::SAID,
                                &reached.said(application.identifier(), strings),
                            )
                            .and_said(words::LIST, &what_it_offers(application, adapters, strings)),
                    )
                    .into_text()
            })
            .collect(),
        Giving::Json => {
            let applications: Vec<serde_json::Value> = all
                .iter()
                .map(|application| one_as_json(application, installed, adapters, strings))
                .collect();
            vec![json!({ "format": 1, "applications": applications }).to_string()]
        }
    }
}

/// One application, in the contract's `--json` shape.
#[must_use]
pub fn one_as_json(
    application: &Application,
    installed: &Installed,
    adapters: &Adapters,
    strings: &Strings,
) -> serde_json::Value {
    let identifier = application.identifier();
    let reached = how_it_is_reached(application, installed);
    json!({
        "identifier": identifier,
        "name": application.shown(strings),
        "typed_as": reached.typed(identifier),
        "short_name": match reached {
            Reached::ByItsShortName(_) => serde_json::Value::String(short_name(identifier)),
            Reached::AsIdentifierBecauseReserved { .. }
            | Reached::AsIdentifierBecauseShared { .. } => serde_json::Value::Null,
        },
        "terminal": is_a_persons_own(identifier),
        "abilities": its_abilities(application, adapters),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use alo_strings::Vocabulary;

    use super::*;

    fn strings() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    fn installed() -> Installed {
        Installed::holding(
            [
                "org.gnome.TextEditor",
                "com.example.Front",
                "app.devsuite.Ptyxis",
            ]
            .map(|identifier| Application::identified(identifier).unwrap()),
        )
    }

    /// **On a pipe, identifiers and nothing else**, so a script reads the same
    /// thing on every machine in every language.
    #[test]
    fn a_pipe_is_given_one_identifier_per_line() {
        let adapters = alo_adapters::shipped_adapters().unwrap();
        assert_eq!(
            listed(&installed(), &adapters, Giving::Values, &strings()),
            [
                "app.devsuite.Ptyxis",
                "com.example.Front",
                "org.gnome.TextEditor"
            ]
        );
    }

    /// **A sentence says how to reach it and what it offers** — including that
    /// a reserved short name does not reach it, and that a terminal is typed
    /// into directly.
    #[test]
    fn each_sentence_says_how_to_reach_it_and_what_it_offers() {
        let adapters = alo_adapters::shipped_adapters().unwrap();
        let lines = listed(&installed(), &adapters, Giving::Sentences, &strings());
        assert_eq!(lines.len(), 3);
        let front = lines
            .iter()
            .find(|line| line.contains("com.example.Front"))
            .unwrap();
        assert!(
            front.contains("reach this application as com.example.Front"),
            "{front}"
        );
        let editor = lines
            .iter()
            .find(|line| line.contains("texteditor"))
            .unwrap();
        assert!(editor.contains("open_document, new_window"), "{editor}");
        let terminal = lines.iter().find(|line| line.contains("ptyxis")).unwrap();
        assert!(terminal.contains("type into it directly"), "{terminal}");
    }

    /// **`--json` carries `"format": 1`**, and a reserved short name is no
    /// short name.
    #[test]
    fn json_carries_its_format() {
        let adapters = alo_adapters::shipped_adapters().unwrap();
        let lines = listed(&installed(), &adapters, Giving::Json, &strings());
        let [line] = lines.as_slice() else {
            unreachable!("one line of JSON")
        };
        let read: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(read["format"], 1);
        let front = read["applications"]
            .as_array()
            .unwrap()
            .iter()
            .find(|one| one["identifier"] == "com.example.Front")
            .unwrap();
        assert_eq!(front["short_name"], serde_json::Value::Null);
        assert_eq!(front["typed_as"], "com.example.Front");
    }
}
