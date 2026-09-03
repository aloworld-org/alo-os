//! Everything alo OS can say, gathered into one vocabulary.
//!
//! Every crate that says something to a person declares its own strings — the
//! key, the English beside it, and the note a translator cannot work without —
//! and hands them over through a `declare_into` of the same shape. This file is
//! the one place that calls all of them.
//!
//! # Why one vocabulary and not one per process
//!
//! Because a translation is **checked against the vocabulary it is loaded
//! into**, and `alo_strings::Amiss::NotSaidHere` is what a key nothing declares
//! gets. A process that declared only its own strings would therefore look at a
//! translator's correct line for another part of the system and call it a
//! mistake — the shell reading the daemon's refusals as wrong, the daemon
//! reading the shortcuts panel's rows as wrong, and each of them able to show
//! its own half of a file somebody wrote as one.
//!
//! So a translation file covers the machine, this is what the machine says, and
//! the crates a particular process happens to link are not the question. It
//! costs this crate a dependency on every crate that has a word in it, and
//! their dependencies with them; `crate` documentation says why that is the
//! cheaper of the two mistakes.
//!
//! # `alo-agentd` is not here, and the reason is not that it is a daemon
//!
//! It is Linux, and every module in it is compiled out anywhere else. A
//! vocabulary assembled here would then hold three fewer strings on a host with
//! no daemon than on a machine with one, so one host would refuse a translation
//! file the other accepted — the exact failure this file exists to prevent, in
//! its platform-shaped form. The daemon declares its own three on top of this
//! one, and [`crate::loading`] is why leaving a line out is survivable rather
//! than the end of somebody's language.
//!
//! # What a failure here means
//!
//! Two crates claiming one key, or a crate whose own list does not declare. It
//! is alo OS's own bug and it cannot be fixed on the machine it happens on, so
//! it keeps its English and its `Display`: whoever reads it is whoever is
//! fixing the list, which is `alo_shortcuts::DefaultsError`'s reader one layer
//! up. It is also caught by the test at the bottom of this file rather than by
//! somebody's machine.

use std::fmt::Display;

use alo_strings::Vocabulary;

/// Why alo OS's own list of what it can say could not be put together.
///
/// Not a refusal anybody reads in their own language, and deliberately: the
/// thing that has gone wrong is the vocabulary, so there is nothing to ask.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("alo OS's own words are wrong: {list} could not be declared — {why}")]
pub struct NotCollected {
    /// Which crate's list.
    list: &'static str,
    /// What that crate's own declaration said about it.
    why: String,
}

impl NotCollected {
    /// Which crate's list could not be declared.
    #[must_use]
    pub fn list(&self) -> &'static str {
        self.list
    }

    /// What that crate said was wrong with it.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.why
    }
}

/// Every crate whose words are collected here, in the order they are declared.
///
/// Written down so that the test walking it and the function below cannot
/// disagree about how many there are: a crate added to one and not the other is
/// a count that no longer proves anything.
pub const EVERY_LIST: [&str; 14] = [
    "alo-answering",
    "alo-appearance",
    "alo-applications",
    "alo-asking",
    "alo-capability",
    "alo-context",
    "alo-dock",
    "alo-egress",
    "alo-files",
    "alo-keeping",
    "alo-models",
    "alo-protocol",
    "alo-shortcuts",
    "alo-turn",
];

/// Everything alo OS can say, in one vocabulary.
///
/// What a process adds to this is whatever it says that the rest of the machine
/// does not — today that is `alo-agentd`'s three, and nothing else in this
/// workspace has any.
///
/// # Errors
///
/// [`NotCollected`], naming the crate whose list would not declare. It cannot
/// happen on a machine that shipped: the test below runs it.
pub fn everything_this_machine_can_say() -> Result<Vocabulary, NotCollected> {
    let mut vocabulary = Vocabulary::empty();
    declare(
        &mut vocabulary,
        "alo-answering",
        alo_answering::declare_into,
    )?;
    declare(
        &mut vocabulary,
        "alo-appearance",
        alo_appearance::declare_into,
    )?;
    // `alo-applications` and `alo-files` are the two crates that declare verbs
    // as well as words, and a crate root holds one `declare_into`. Theirs is
    // the verbs' — the older meaning and the one their callers use — so the
    // words are named through the module they are in rather than by moving
    // somebody else's public surface out from under them.
    declare(
        &mut vocabulary,
        "alo-applications",
        alo_applications::words::declare_into,
    )?;
    declare(&mut vocabulary, "alo-asking", alo_asking::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-capability",
        alo_capability::declare_into,
    )?;
    declare(&mut vocabulary, "alo-context", alo_context::declare_into)?;
    declare(&mut vocabulary, "alo-dock", alo_dock::declare_into)?;
    declare(&mut vocabulary, "alo-egress", alo_egress::declare_into)?;
    declare(&mut vocabulary, "alo-files", alo_files::words::declare_into)?;
    declare(&mut vocabulary, "alo-keeping", alo_keeping::declare_into)?;
    declare(&mut vocabulary, "alo-models", alo_models::declare_into)?;
    declare(&mut vocabulary, "alo-protocol", alo_protocol::declare_into)?;
    declare(
        &mut vocabulary,
        "alo-shortcuts",
        alo_shortcuts::declare_into,
    )?;
    declare(&mut vocabulary, "alo-turn", alo_turn::declare_into)?;
    Ok(vocabulary)
}

/// One crate's list, into the vocabulary being built.
///
/// Every crate's `declare_into` has the same shape and its own error type, each
/// of which is that crate's English for whoever is fixing the list. What is
/// kept here is the sentence rather than the type: a fifteen-armed enum would
/// say nothing this does not, and the reader is looking for which crate and
/// which key.
fn declare<Why: Display>(
    vocabulary: &mut Vocabulary,
    list: &'static str,
    declaring: impl FnOnce(&mut Vocabulary) -> Result<(), Why>,
) -> Result<(), NotCollected> {
    declaring(vocabulary).map_err(|why| NotCollected {
        list,
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;

    /// One string each crate declares, which is how the test below proves that
    /// crate was reached rather than that the total came out right.
    const ONE_STRING_EACH: [(&str, &str); 14] = [
        ("alo-answering", "answering.wrong.nothing-answered"),
        ("alo-appearance", "appearance.token.navy"),
        ("alo-applications", "applications.not-installed"),
        ("alo-asking", "asking.question.nothing"),
        ("alo-capability", "capability.grant.anonymous"),
        ("alo-context", "context.the-document"),
        ("alo-dock", "dock.edge.bottom"),
        ("alo-egress", "egress.destination.paired-machine"),
        ("alo-files", "files.failed.not-a-file-verb"),
        ("alo-keeping", "keeping.forever"),
        ("alo-models", "models.source.this-machine"),
        ("alo-protocol", "protocol.too-long"),
        ("alo-shortcuts", "shortcuts.action.the-agent"),
        ("alo-turn", "turn.closed"),
    ];

    /// **alo OS's own words do not contradict each other.** This is the test
    /// that keeps [`NotCollected`] a thing nobody's machine ever sees: two
    /// crates landing on one key fails here rather than at somebody's first
    /// sign-in.
    #[test]
    fn everything_this_machine_says_can_be_collected() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        assert!(!vocabulary.is_empty());
    }

    /// **Every crate on the list was actually reached.** A total would pass
    /// while a crate was silently dropped and another grew; one key from each
    /// cannot.
    #[test]
    fn every_crate_that_says_something_is_in_it() {
        let vocabulary = everything_this_machine_can_say().unwrap();
        for (list, named) in ONE_STRING_EACH {
            let key = Key::named(named).unwrap();
            assert!(
                vocabulary.phrase(&key).is_some() || vocabulary.plural(&key).is_some(),
                "{list} was not collected: nothing here says {named}"
            );
        }
    }

    /// The two lists of crates are one list, so the count below is a count of
    /// something.
    #[test]
    fn the_lists_of_crates_agree() {
        let named: Vec<&str> = ONE_STRING_EACH.iter().map(|(list, _)| *list).collect();
        assert_eq!(named, EVERY_LIST);
    }

    /// **Nothing was lost and nothing was shared.** The machine's vocabulary
    /// holds exactly as many strings as the crates hold between them, which is
    /// only true while no key was dropped on the way in and no two crates
    /// declared the same one.
    #[test]
    fn the_machine_says_what_the_crates_say_between_them() {
        let each = [
            alo_answering::answering_words().unwrap().how_many(),
            alo_appearance::appearance_words().unwrap().how_many(),
            alo_applications::application_words().unwrap().how_many(),
            alo_asking::asking_words().unwrap().how_many(),
            alo_capability::capability_words().unwrap().how_many(),
            alo_context::context_words().unwrap().how_many(),
            alo_dock::dock_words().unwrap().how_many(),
            alo_egress::egress_words().unwrap().how_many(),
            alo_files::file_words().unwrap().how_many(),
            alo_keeping::keeping_words().unwrap().how_many(),
            alo_models::model_words().unwrap().how_many(),
            alo_protocol::protocol_words().unwrap().how_many(),
            alo_shortcuts::shortcut_words().unwrap().how_many(),
            alo_turn::turn_words().unwrap().how_many(),
        ];
        assert_eq!(each.len(), EVERY_LIST.len());
        assert_eq!(
            everything_this_machine_can_say().unwrap().how_many(),
            each.iter().sum::<usize>()
        );
    }

    /// A crate whose list will not declare names itself, because the person
    /// reading this is looking for a file to open.
    #[test]
    fn a_list_that_will_not_declare_names_the_crate() {
        let mut vocabulary = Vocabulary::empty();
        let refused = declare(&mut vocabulary, "alo-nothing", |_| {
            Err::<(), &str>("two of them are called files.gone")
        })
        .unwrap_err();
        assert_eq!(refused.list(), "alo-nothing");
        assert_eq!(refused.why(), "two of them are called files.gone");
        assert!(refused.to_string().contains("alo-nothing"), "{refused}");
        assert!(refused.to_string().contains("files.gone"), "{refused}");
    }
}
