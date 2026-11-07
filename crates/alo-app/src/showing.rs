//! **`alo-app show <application> [<ability>]`**: everything one application can
//! be asked to do, read from the machine at the moment the person asks and kept
//! nowhere (`docs/contracts/terminal-commands.md`, section 2).
//!
//! It says which **roads** there are, each in a sentence naming what was looked
//! for — never an empty list, which would look like a fault:
//!
//! - **its window**: open, focus and close, which every installed application
//!   has;
//! - **its controls through the accessibility tree**, and whether its windows
//!   describe themselves there now — asked of the tree, once;
//! - **abilities an adapter declares**, each with its sentence, whether it
//!   changes anything, and whether it can be undone;
//! - and, said rather than hidden, the two roads this release does not take:
//!   **what an application publishes about itself on the session bus is not
//!   asked**, and **reading the screen is not built**.
//!
//! `--json` gives the same in the contract's shape, carrying `"format": 1`. On a
//! pipe, the declared abilities' names, one per line — or, for one ability, its
//! arguments' names.

use alo_adapters::{AdapterArg, Kind, Sends};
use alo_capability::{Effect, is_a_persons_own};
use alo_strings::{Filling, Said, Strings};
use serde_json::json;

use crate::declared::Declared;
use crate::listing::{Giving, one_as_json};
use crate::machine::{Knowing, Machine};
use crate::naming::how_it_is_reached;
use crate::subject::{Reaching, Subject};
use crate::words;

/// Whether an application's windows describe themselves to the tree now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheTree {
    /// One of its windows is on the tree now.
    Described,
    /// None of its windows is open, so whether they would be is not known.
    NotOpen,
    /// This session has no accessibility tree.
    NoTree,
}

impl OnTheTree {
    /// What the contract's JSON calls it.
    const fn named(self) -> &'static str {
        match self {
            Self::Described => "described",
            Self::NotOpen => "not-open",
            Self::NoTree => "no-tree",
        }
    }
}

/// Ask the tree, once, whether this application is on it.
pub fn on_the_tree(identifier: &str, machine: &mut dyn Machine) -> OnTheTree {
    let Ok(tree) = machine.tree() else {
        return OnTheTree::NoTree;
    };
    match tree.applications() {
        Ok(running)
            if running
                .iter()
                .any(|one| one.is.as_deref() == Some(identifier)) =>
        {
            OnTheTree::Described
        }
        Ok(_) => OnTheTree::NotOpen,
        Err(_) => OnTheTree::NoTree,
    }
}

/// What `show` says of an application, or of one of its abilities.
#[must_use]
pub fn shown(
    subject: &Subject,
    ability: Option<Declared>,
    on_the_tree: OnTheTree,
    giving: Giving,
    knowing: Knowing<'_>,
) -> Vec<String> {
    let strings = knowing.strings;
    let Reaching::Application(application) = &subject.reaching else {
        return alos_own(giving, strings);
    };
    let identifier = application.identifier();
    let declared: Vec<Declared> = knowing
        .adapters
        .for_application(identifier)
        .map(|adapter| {
            adapter
                .verbs
                .iter()
                .map(|verb| Declared { adapter, verb })
                .collect()
        })
        .unwrap_or_default();
    if let Some(one) = ability {
        return one_ability(one, subject, giving, knowing);
    }
    match giving {
        Giving::Values => declared
            .iter()
            .map(|one| one.verb.name.to_owned())
            .collect(),
        Giving::Json => {
            let mut one = one_as_json(application, knowing.installed, knowing.adapters, strings);
            with(&mut one, "format", json!(1));
            with(
                &mut one,
                "roads",
                json!({
                    "window": true,
                    "tree": on_the_tree.named(),
                    "declared": declared.iter().map(|one| ability_as_json(*one, strings)).collect::<Vec<_>>(),
                    "discovered": "not-asked",
                    "screen": "not-built",
                }),
            );
            vec![one.to_string()]
        }
        Giving::Sentences => {
            let say =
                |word: &words::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
            let reached = how_it_is_reached(application, knowing.installed);
            let mut lines = vec![
                strings
                    .say(
                        &words::SHOW_HEADING.key(),
                        &Filling::of(words::NAME, subject.called.clone())
                            .and(words::IDENTIFIER, identifier.to_owned())
                            .and_said(words::SAID, &reached.said(identifier, strings)),
                    )
                    .into_text(),
            ];
            if is_a_persons_own(identifier) {
                lines.push(say(&words::A_TERMINAL));
            }
            lines.push(say(&words::ROAD_WINDOW));
            lines.push(say(match on_the_tree {
                OnTheTree::Described => &words::ROAD_TREE_DESCRIBED,
                OnTheTree::NotOpen => &words::ROAD_TREE_NOT_OPEN,
                OnTheTree::NoTree => &words::ROAD_TREE_NOT_HERE,
            }));
            if declared.is_empty() {
                lines.push(say(&words::ROAD_NO_DECLARED));
            } else {
                lines.push(say(&words::ROAD_DECLARED));
                for one in &declared {
                    let purpose = strings.say(&one.verb.purpose.key(), &Filling::nothing());
                    lines.push(format!(
                        "  {}",
                        strings.say(
                            &words::ABILITY_LINE.key(),
                            &Filling::of(words::ABILITY, one.verb.name.to_owned())
                                .and_said(words::SAID, &purpose),
                        )
                    ));
                }
            }
            lines.push(say(&words::ROAD_DISCOVERED_NOT_ASKED));
            lines.push(say(&words::ROAD_SCREEN_NOT_BUILT));
            lines
        }
    }
}

/// alo OS's own surfaces.
fn alos_own(giving: Giving, strings: &Strings) -> Vec<String> {
    let surfaces: Vec<&str> = alo_access::Surface::ALL
        .iter()
        .map(|surface| surface.on_the_wire())
        .collect();
    match giving {
        Giving::Values => surfaces
            .iter()
            .map(|surface| (*surface).to_owned())
            .collect(),
        Giving::Json => vec![
            json!({
                "format": 1,
                "identifier": crate::typed::ALO,
                "roads": { "tree": "alos-own" },
                "surfaces": surfaces,
            })
            .to_string(),
        ],
        Giving::Sentences => vec![
            strings
                .say(
                    &words::ALO_SURFACES.key(),
                    &Filling::of(words::LIST, surfaces.join(", ")),
                )
                .into_text(),
        ],
    }
}

/// One ability, whole.
fn one_ability(
    one: Declared,
    subject: &Subject,
    giving: Giving,
    knowing: Knowing<'_>,
) -> Vec<String> {
    let strings = knowing.strings;
    match giving {
        Giving::Values => one
            .verb
            .args
            .iter()
            .map(|arg| arg.name.to_owned())
            .collect(),
        Giving::Json => {
            let mut shown = ability_as_json(one, strings);
            with(&mut shown, "format", json!(1));
            with(&mut shown, "application", json!(subject.identifier()));
            vec![shown.to_string()]
        }
        Giving::Sentences => {
            let purpose = strings.say(&one.verb.purpose.key(), &Filling::nothing());
            let mut lines = vec![
                strings
                    .say(
                        &words::ABILITY_LINE.key(),
                        &Filling::of(words::ABILITY, one.verb.name.to_owned())
                            .and_said(words::SAID, &purpose),
                    )
                    .into_text(),
            ];
            let typed_as = subject
                .application()
                .map(|application| {
                    how_it_is_reached(application, knowing.installed)
                        .typed(application.identifier())
                        .to_owned()
                })
                .unwrap_or_default();
            let mut command = format!("alo-app {typed_as} {}", one.verb.name);
            for arg in one.verb.args {
                command.push_str(&format!(" <{}>", arg.name));
            }
            lines.push(
                strings
                    .say(&words::TYPED_AS.key(), &Filling::of(words::SAID, command))
                    .into_text(),
            );
            for arg in one.verb.args {
                lines.push(format!("  {}", argument_said(arg, strings)));
            }
            lines.push(
                strings
                    .say(
                        &words::IN_WORDS.key(),
                        &Filling::nothing()
                            .and_said(words::SAID, &sentence_with_gaps(one, strings)),
                    )
                    .into_text(),
            );
            let effect = if one.verb.effect == Effect::Change {
                &words::IT_CHANGES
            } else {
                &words::IT_READS
            };
            lines.push(strings.say(&effect.key(), &Filling::nothing()).into_text());
            if one.verb.sends != Sends::Never {
                lines.push(
                    strings
                        .say(
                            &words::CANNOT_BE_UNDONE_ONCE_SENT.key(),
                            &Filling::nothing(),
                        )
                        .into_text(),
                );
            }
            lines
        }
    }
}

/// The ability's sentence, with each argument shown as `<name>`.
fn sentence_with_gaps(one: Declared, strings: &Strings) -> Said {
    let filling = one
        .verb
        .args
        .iter()
        .fold(Filling::nothing(), |filling, arg| {
            filling.and(arg.name, format!("<{}>", arg.name))
        });
    strings.say(&one.verb.sentence.key(), &filling)
}

/// One argument, in a sentence.
fn argument_said(arg: &AdapterArg, strings: &Strings) -> String {
    let purpose = strings.say(&arg.purpose.key(), &Filling::nothing());
    let filling = Filling::nothing().and_said(words::SAID, &purpose);
    let kind = match arg.kind {
        Kind::Path => strings.say(&words::KIND_PATH.key(), &filling),
        Kind::Count { .. } => strings.say(&words::KIND_COUNT.key(), &filling),
        Kind::Choice(offers) => strings.say(
            &words::KIND_CHOICE.key(),
            &filling.and(
                words::LIST,
                offers
                    .iter()
                    .map(|offer| offer.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        ),
        Kind::Name { .. } | Kind::Text | Kind::Script { .. } | Kind::Command => {
            strings.say(&words::KIND_NAME.key(), &filling)
        }
    };
    strings
        .say(
            &words::ARGUMENT_LINE.key(),
            &Filling::of(words::TYPED, arg.name.to_owned()).and_said(words::SAID, &kind),
        )
        .into_text()
}

/// One ability, in the contract's `--json` shape.
fn ability_as_json(one: Declared, strings: &Strings) -> serde_json::Value {
    json!({
        "ability": one.verb.name,
        "purpose": strings.say(&one.verb.purpose.key(), &Filling::nothing()).into_text(),
        "effect": if one.verb.effect == Effect::Change { "change" } else { "read" },
        "sends": one.verb.sends.named(),
        "arguments": one.verb.args.iter().map(|arg| json!({
            "name": arg.name,
            "kind": arg.kind.named(),
            "purpose": strings.say(&arg.purpose.key(), &Filling::nothing()).into_text(),
        })).collect::<Vec<_>>(),
    })
}

/// Put one more field into a JSON object.
fn with(object: &mut serde_json::Value, field: &str, value: serde_json::Value) {
    if let Some(map) = object.as_object_mut() {
        map.insert(field.to_owned(), value);
    }
}
