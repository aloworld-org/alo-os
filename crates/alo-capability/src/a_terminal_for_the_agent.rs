//! What a person has allowed their agent to do with a terminal.
//!
//! # The decision this replaces, and why it changed
//!
//! [ADR 0043](../../../docs/decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md)
//! walled a terminal off from an agent entirely, and
//! [ADR 0111](../../../docs/decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)
//! supersedes it. The owner's ruling, in their own words: *we are not dictators
//! so the users should be able to choose if the agent can use the terminal or
//! not.*
//!
//! ADR 0111 records three things that had made 0043 untenable, and they are
//! worth repeating here because this file is where the loosening happens:
//!
//! - law 5 reached `CLAUDE.md` after 0043 was written;
//! - [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//!   replaced 0043's stated foundation six days later and permits an agent to
//!   run **any** command at full trust — so the wall forbade the lesser act
//!   while the greater was allowed;
//! - [ADR 0079](../../../docs/decisions/0079-a-whole-machine.md) already
//!   permits handing an agent a whole remote machine.
//!
//! # Two halves, and this file is only the first
//!
//! The owner's second ruling of the same day split the question: *agent can
//! open terminal window application.* So:
//!
//! 1. **Opening one** — this file. A person may allow it, and the default is
//!    that they have not.
//! 2. **Running commands in one** — ADR 0064's three levels, which are a
//!    separate capability and not a verb at all. Law 2: *running code is not a
//!    verb on that list.* Nothing here grants it.
//!
//! # Why opening is not driving, checked rather than asserted
//!
//! `docs/features.md` says it: *a terminal window opens onto a prompt waiting
//! for a keystroke, and nothing lets alo type into it, so what it has opened is
//! a window **only you can use**.*
//!
//! That is true of the verb list and was read off it on 2026-10-14.
//! `docs/contracts/app-adapters.md` carries **two** verbs that reach an
//! application at all: `accessible.read_window`, which is a read, and
//! `accessible.activate_control`, which presses a named `button`, `check_box`,
//! `radio_button`, `switch`, `menu_item`, `tab` or `link`. **Not one of them
//! sends text.** And `crate::verb` refuses, structurally, any verb named or
//! argumented like a command — `exec`, `eval`, `shell`, `script`, `expression`
//! among them — so a verb that typed could not be declared without that
//! refusal being removed first.
//!
//! **What an agent permitted to open a terminal can therefore do** is open one,
//! read what is in its window, and press controls its window offers by name.
//! Those are the ordinary powers over any granted application, and the person
//! granted them.
//!
//! # The default is closed, and it fails closed
//!
//! [`Self::NotAtAll`] is [`Default`], and it is what every road in this crate
//! that does **not** take a choice means. `CLAUDE.md`: *where a flag chooses
//! between the strict and the loose reading, the strict one is the default: a
//! flag that fails open is a flag that will be forgotten in the direction that
//! matters.* So a caller that has not been taught about this gets the refusal
//! it got before, and no caller is loosened by being out of date.

use serde::{Deserialize, Serialize};

/// What a person has allowed their agent to do with a terminal.
///
/// **Not a setting this crate reads.** A person's choices live in their own
/// settings — `docs/contracts/person-settings.md` — and this crate is handed
/// the answer rather than going to look for it, as it is handed a grant rather
/// than reading the grants file. A capability layer that read settings would be
/// a capability layer with a second source of truth about what is permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ATerminalForTheAgent {
    /// Not at all, which is how a fresh machine arrives.
    ///
    /// **The default, and deliberately so.** `docs/features.md`: *off when a
    /// machine arrives, and yours to turn on.*
    #[default]
    NotAtAll,
    /// The agent may open one.
    ///
    /// It may then do what it may do with any granted application: read the
    /// window and press controls by name. **It cannot type**, for the reason
    /// this module's header establishes from the verb list — so what it has
    /// opened is a window only the person can use.
    MayOpenOne,
}

impl ATerminalForTheAgent {
    /// Whether an agent may hold a grant over a terminal at all.
    ///
    /// A method rather than a `matches!` at each of the three sites that ask,
    /// so that adding a third state later changes one line and not three —
    /// and so that the three cannot come to disagree, which is the fault this
    /// crate is least able to afford.
    #[must_use]
    pub const fn may_open_one(self) -> bool {
        matches!(self, Self::MayOpenOne)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::ATerminalForTheAgent;

    /// **A fresh machine has not chosen**, which `docs/features.md` promises in
    /// as many words: *off when a machine arrives, and yours to turn on.*
    #[test]
    fn the_default_is_not_at_all() {
        assert_eq!(
            ATerminalForTheAgent::default(),
            ATerminalForTheAgent::NotAtAll
        );
        assert!(!ATerminalForTheAgent::default().may_open_one());
    }

    /// **Only the one state permits it**, which is the assertion that would
    /// catch a later state being added and permitting by accident.
    #[test]
    fn only_choosing_it_permits_it() {
        assert!(ATerminalForTheAgent::MayOpenOne.may_open_one());
        assert!(!ATerminalForTheAgent::NotAtAll.may_open_one());
    }

    /// **It is written down in the person's own words, not as a number.**
    ///
    /// A choice kept as `0` or `1` is a choice nobody can read in their own
    /// settings file, and `docs/contracts/person-settings.md` is a file a
    /// person may edit.
    #[test]
    fn it_is_kept_in_words_a_person_can_read() {
        let written = toml::to_string(&Kept {
            terminal: ATerminalForTheAgent::MayOpenOne,
        })
        .expect("it writes");
        assert!(
            written.contains("may-open-one"),
            "it was written as {written:?}, which a person cannot read in their settings"
        );
        let back: Kept = toml::from_str(&written).expect("it reads back");
        assert_eq!(back.terminal, ATerminalForTheAgent::MayOpenOne);
    }

    /// **A settings file that says nothing means the default**, so an older
    /// file permits nothing rather than failing to load.
    #[test]
    fn a_file_that_says_nothing_means_not_at_all() {
        let back: Kept = toml::from_str("").expect("an empty table reads");
        assert_eq!(back.terminal, ATerminalForTheAgent::NotAtAll);
    }

    /// **A spelling this does not know is refused rather than guessed.**
    ///
    /// The one direction that matters: a typo in a settings file must not
    /// become a permission. It is an error, which a reader reports, and not a
    /// silent fall back to either state.
    #[test]
    fn a_spelling_it_does_not_know_is_refused() {
        let answer: Result<Kept, _> = toml::from_str("terminal = \"may-run-anything\"");
        assert!(
            answer.is_err(),
            "a spelling this crate does not know was accepted, so a typo in a settings file \
             could become a permission"
        );
    }

    /// A table holding one, as a settings file would.
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Kept {
        #[serde(default)]
        terminal: ATerminalForTheAgent,
    }
}
