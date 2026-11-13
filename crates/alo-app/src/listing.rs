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
//!   "terminal":false,"abilities":["open_document","new_window"],
//!   "reaches":{"home":"read-and-change","starts_programs_outside":false}}]}
//! ```
//!
//! # What its package lets it reach
//!
//! Where an application's own package lets it reach every file in the home, or
//! start programs outside its sandbox, its sentence says so (ADR 0143, fact 4)
//! — read from the package on this machine by
//! [`alo_installed::what_they_may_reach`], so a sealed package is never said
//! to be open and an application nobody here wrote down is said the same way.
//! `reaches` is `null` where no package was read, which is not *sealed*: an
//! application the image carries outside any sandbox has no package to read.
//!
//! `typed_as` is what reaches it — its short name, or its identifier where the
//! short name is reserved or shared.

use alo_adapters::Adapters;
use alo_applications::{Application, Installed};
use alo_capability::is_a_persons_own;
use alo_installed::{TheirHome, WhatItMayReach, WhatTheyMayReach};
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

/// What its package lets it reach without asking, in one sentence — [`None`]
/// where there is nothing to say, which is also what an application with no
/// package read is given.
#[must_use]
pub fn what_it_may_reach(it: Option<WhatItMayReach>, strings: &Strings) -> Option<Said> {
    let it = it.filter(|it| it.is_worth_saying())?;
    let word = match (it.home(), it.starts_programs_outside()) {
        (TheirHome::NotAsAWhole, _) => &words::REACHES_PROGRAMS,
        (TheirHome::Read, false) => &words::REACHES_HOME_READ,
        (TheirHome::ReadAndChanged, false) => &words::REACHES_HOME_CHANGE,
        (TheirHome::Read, true) => &words::REACHES_HOME_READ_AND_PROGRAMS,
        (TheirHome::ReadAndChanged, true) => &words::REACHES_HOME_CHANGE_AND_PROGRAMS,
    };
    Some(strings.say(&word.key(), &Filling::nothing()))
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
    reaching: &WhatTheyMayReach,
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
                let line = Filling::of(words::NAME, application.shown(strings))
                    .and_said(
                        words::SAID,
                        &reached.said(application.identifier(), strings),
                    )
                    .and_said(words::LIST, &what_it_offers(application, adapters, strings));
                match what_it_may_reach(reaching.of(application.identifier()), strings) {
                    Some(reaches) => strings.say(
                        &words::LIST_LINE_REACHING.key(),
                        &line.and_said(words::SAID_TOO, &reaches),
                    ),
                    None => strings.say(&words::LIST_LINE.key(), &line),
                }
                .into_text()
            })
            .collect(),
        Giving::Json => {
            let applications: Vec<serde_json::Value> = all
                .iter()
                .map(|application| one_as_json(application, installed, adapters, reaching, strings))
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
    reaching: &WhatTheyMayReach,
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
        "reaches": reaching.of(identifier).map(|it| json!({
            "home": match it.home() {
                TheirHome::NotAsAWhole => "none",
                TheirHome::Read => "read",
                TheirHome::ReadAndChanged => "read-and-change",
            },
            "starts_programs_outside": it.starts_programs_outside(),
        })),
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
            listed(
                &installed(),
                &adapters,
                &WhatTheyMayReach::nothing(),
                Giving::Values,
                &strings()
            ),
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
        let lines = listed(
            &installed(),
            &adapters,
            &WhatTheyMayReach::nothing(),
            Giving::Sentences,
            &strings(),
        );
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
        let lines = listed(
            &installed(),
            &adapters,
            &WhatTheyMayReach::nothing(),
            Giving::Json,
            &strings(),
        );
        let [line] = lines.as_slice() else {
            unreachable!("one line of JSON")
        };
        let read: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(read.get("format").unwrap(), 1);
        let front = read
            .get("applications")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|one| {
                one.get("identifier")
                    .is_some_and(|it| it == "com.example.Front")
            })
            .unwrap();
        assert_eq!(front.get("short_name"), Some(&serde_json::Value::Null));
        assert_eq!(front.get("typed_as").unwrap(), "com.example.Front");
    }

    /// A flatpak installation of the test's own, holding these packages.
    fn an_installation(named: &str, packages: &[(&str, &str)]) -> std::path::PathBuf {
        let at =
            std::env::temp_dir().join(format!("alo-app-listing-{}-{named}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        for (identifier, metadata) in packages {
            let active = at
                .join("app")
                .join(identifier)
                .join("current")
                .join("active");
            std::fs::create_dir_all(&active).unwrap();
            std::fs::write(active.join("metadata"), metadata).unwrap();
        }
        at
    }

    /// **Where a person looks at what is installed, VSCodium says what its
    /// package reaches** (ADR 0143, fact 4) — read off the package, so the
    /// sealed Firefox beside it says nothing of the kind, and an application
    /// with no package read is `null` rather than sealed.
    #[test]
    fn an_application_reaching_the_whole_home_is_said_so_and_a_sealed_one_is_not() {
        let installation = an_installation(
            "reaching",
            &[
                (
                    "com.vscodium.codium",
                    "[Context]\nfilesystems=xdg-run/gnupg:ro;host;\n\n\
                     [Session Bus Policy]\norg.freedesktop.Flatpak=talk\n",
                ),
                (
                    "org.mozilla.firefox",
                    "[Context]\nfilesystems=xdg-download;/run/udev:ro;\n",
                ),
            ],
        );
        let installed = Installed::holding(
            [
                "com.vscodium.codium",
                "org.mozilla.firefox",
                "org.libreoffice.LibreOffice",
            ]
            .map(|identifier| Application::identified(identifier).unwrap()),
        );
        let reaching = alo_installed::what_they_may_reach(
            &installation,
            None,
            installed.all().map(Application::identifier),
        );
        let adapters = alo_adapters::shipped_adapters().unwrap();
        let lines = listed(
            &installed,
            &adapters,
            &reaching,
            Giving::Sentences,
            &strings(),
        );
        let line = |of: &str| lines.iter().find(|line| line.contains(of)).unwrap().clone();
        assert!(
            line("vscodium").contains(
                "It can read and change every file in your home, and start other programs that \
                 reach everything your account can, without asking you"
            ),
            "{lines:?}"
        );
        for quiet in ["firefox", "libreoffice"] {
            assert!(!line(quiet).contains("without asking you"), "{lines:?}");
        }

        let json = listed(&installed, &adapters, &reaching, Giving::Json, &strings());
        let read: serde_json::Value = serde_json::from_str(&json.concat()).unwrap();
        let reaches = |of: &str| {
            read.get("applications")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .find(|one| one.get("identifier").is_some_and(|it| it == of))
                .unwrap()
                .get("reaches")
                .unwrap()
                .clone()
        };
        assert_eq!(
            reaches("com.vscodium.codium"),
            serde_json::json!({"home": "read-and-change", "starts_programs_outside": true})
        );
        assert_eq!(
            reaches("org.mozilla.firefox"),
            serde_json::json!({"home": "none", "starts_programs_outside": false})
        );
        assert_eq!(
            reaches("org.libreoffice.LibreOffice"),
            serde_json::Value::Null
        );
        let _ = std::fs::remove_dir_all(&installation);
    }
}
