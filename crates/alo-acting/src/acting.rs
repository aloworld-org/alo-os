//! The closed list: what may be asked of a window.
//!
//! Three, and `alo-applications` owns their names. This is not a second verb
//! list — it is the acting half of three verbs that already exist, so a name
//! here that `alo-applications` does not declare would be a verb nobody
//! granted.
//!
//! # Enumerated, and an argument is never a name to look up
//!
//! Law 2 at the one place a caller can reach: there is no variant carrying a
//! command, a path, a window identifier the caller composed, or a verb's name
//! as text. An application is an `alo_applications::Application`, which is held
//! to its own shape before one exists — so a line off this door either names an
//! application that could exist or is not a request at all.
//!
//! `alo-broker`'s thirteen are the same decision one privilege level up, and
//! its reason is quoted here because it is this one: *a closed list, a door, and
//! nothing else.*

use alo_applications::Application;

/// What may be asked of an application's windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Acting {
    /// Start it if it is not running, and show it either way.
    ///
    /// One act and not two, because that is what a person means: *open my mail*
    /// is answered by the window appearing, and whether it had to be started is
    /// the machine's business rather than theirs. The compositor already works
    /// this way for the Dock.
    Open(Application),
    /// Bring its window to the front and give it the keyboard.
    Focus(Application),
    /// Ask it to close.
    ///
    /// **Ask**, not kill: an application with unsaved work is entitled to say
    /// *are you sure* to the person, and a door that could end a process would
    /// be a door that loses somebody's afternoon. What this carries is the
    /// request a person's own Close button makes.
    Close(Application),
}

impl Acting {
    /// The application this is about.
    #[must_use]
    pub const fn application(&self) -> &Application {
        match self {
            Self::Open(which) | Self::Focus(which) | Self::Close(which) => which,
        }
    }

    /// The word this crosses the door as.
    ///
    /// One table for both directions, so the two cannot drift —
    /// `alo_corridor::door`'s rule, met again.
    #[must_use]
    pub const fn on_the_wire(&self) -> &'static str {
        match self {
            Self::Open(_) => "open",
            Self::Focus(_) => "focus",
            Self::Close(_) => "close",
        }
    }

    /// The act a word off the wire is, with the application it is about.
    ///
    /// # Errors
    /// [`NotAnAct`] for a word this door does not carry, which includes
    /// `arrange` — declared by `alo-applications` and deliberately not here
    /// (see this crate's header), so a daemon asking for it is told so rather
    /// than ignored.
    pub fn off_the_wire(word: &str, which: Application) -> Result<Self, NotAnAct> {
        match word {
            "open" => Ok(Self::Open(which)),
            "focus" => Ok(Self::Focus(which)),
            "close" => Ok(Self::Close(which)),
            other => Err(NotAnAct {
                word: other.to_owned(),
            }),
        }
    }

    /// Every act, in the order this file declares them.
    ///
    /// What a test walks, so an act added above and left out of a table below is
    /// caught here rather than by a door that silently cannot do it.
    pub const EVERY_ONE: [&'static str; 3] = ["open", "focus", "close"];
}

/// A word this door does not carry.
///
/// English and a `Display`: its reader is whoever is fixing the caller, never a
/// person — the sentence a person reads about an application verb is
/// `alo-applications`' own.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{word} is not something this door does to a window")]
pub struct NotAnAct {
    /// The word that arrived.
    word: String,
}

impl NotAnAct {
    /// The word that arrived, for whoever is fixing the caller.
    #[must_use]
    pub fn word(&self) -> &str {
        &self.word
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::{Acting, NotAnAct};
    use alo_applications::Application;

    fn mail() -> Application {
        Application::identified("org.gnome.Geary").unwrap()
    }

    /// **Every act crosses as one word and comes back as itself.**
    ///
    /// The table and the match cannot drift, which is the fault this shape is
    /// for: a word that went out and did not come back is a door that can be
    /// asked something it cannot answer.
    #[test]
    fn every_act_crosses_as_one_word_and_comes_back_as_itself() {
        for word in Acting::EVERY_ONE {
            let act = Acting::off_the_wire(word, mail()).unwrap();
            assert_eq!(act.on_the_wire(), word);
            assert_eq!(act.application(), &mail());
        }
        // And the list is the whole list: three acts, three words, no repeats.
        let mut words = Acting::EVERY_ONE.to_vec();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), 3);
    }

    /// **`arrange` is refused by name**, not ignored.
    ///
    /// `alo-applications` declares four verbs and this door carries three. A
    /// daemon that asked for the fourth would otherwise meet silence, which
    /// reads like a door that did the thing.
    #[test]
    fn arrange_is_refused_by_name_rather_than_ignored() {
        let refused = Acting::off_the_wire("arrange", mail()).unwrap_err();
        assert_eq!(refused.word(), "arrange");
        assert!(refused.to_string().contains("arrange"));
    }

    /// And so is anything else, including a verb's own name put where an act
    /// belongs — which is the mistake a model makes at the other door.
    #[test]
    fn a_word_this_door_does_not_carry_is_refused() {
        for word in ["", "Open", "open_application", "kill", "focus_application"] {
            assert!(
                Acting::off_the_wire(word, mail()).is_err(),
                "`{word}` was taken for an act"
            );
        }
    }

    /// The refusal is English for whoever is fixing the caller, and says so by
    /// being a `Display` at all — a sentence a person reads is
    /// `alo-applications`' and never this crate's.
    #[test]
    fn the_refusal_is_for_whoever_is_fixing_the_caller() {
        let refused: NotAnAct = Acting::off_the_wire("arrange", mail()).unwrap_err();
        assert!(!format!("{refused}").is_empty());
    }
}
