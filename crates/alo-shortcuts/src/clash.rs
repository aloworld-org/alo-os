//! Two actions wanting the same keys, and the refusal that keeps it from
//! happening quietly.
//!
//! **The last binding does not win.** A model where it did would let a person
//! set `Super+Left` on one thing and lose it from another without being told,
//! and they would find out days later when the shortcut they did not change
//! stopped working. So there are two types here, for the two moments a clash can
//! arrive by.
//!
//! [`Taken`] is the refusal: it comes back from [`crate::Shortcuts::bind`] when
//! the chord somebody just pressed is already doing something else, and it names
//! what. Nothing is changed.
//!
//! [`Clash`] is the report, and it exists because refusing at the moment of
//! binding is not enough. **A release can add a default that lands on a chord
//! somebody already moved something else onto**, and no refusal at bind time
//! could have seen it coming — the binding was made before the default existed.
//! So a clash has to be a thing the model can hold and show, not only a thing it
//! can prevent.
//!
//! # Why the report does not list what it is a report about
//!
//! [`Taken`] names the one action that already has the chord, because there is
//! exactly one and a sentence can hold it. [`Clash`] names none of them, and
//! that is a decision rather than an omission: two or more actions want the
//! chord, and joining a list of them into a sentence is something no translator
//! can punctuate. The separator is not the same in every language — Greek writes
//! a question mark where English writes a semicolon — and the conjunction before
//! the last item is a word we would have had to invent a string for and then
//! ask a machine to place. So the sentence says *more than one thing*, and
//! [`Clash::actions`] hands a panel the list to draw as rows, each said in the
//! reader's own language by [`crate::Action::said`]. Nothing is lost; what is
//! avoided is a sentence assembled by a machine out of fragments.
//!
//! Neither type has a `Display`. The reasoning is [`crate::refusing`]'s, and it
//! costs [`Taken`] `std::error::Error` — which it was never usefully, being a
//! sentence a person reads rather than an error a programmer handles.

use alo_strings::{Filling, Said, Strings};

use crate::action::Action;
use crate::chord::Chord;
use crate::words;

/// A chord that more than one action wants.
///
/// The actions are in the order [`Action::ALL`] lists them, so the same clash
/// reads the same way every time it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clash {
    /// The chord being fought over.
    chord: Chord,
    /// Everything that wants it — always two or more.
    actions: Vec<Action>,
}

impl Clash {
    /// A clash over this chord, between these actions.
    pub(crate) fn over(chord: Chord, actions: Vec<Action>) -> Self {
        Self { chord, actions }
    }

    /// The chord being fought over.
    #[must_use]
    pub fn chord(&self) -> Chord {
        self.chord
    }

    /// Everything that wants it.
    #[must_use]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// What a person is told, in the language they read.
    ///
    /// It names the chord and not the actions; the module documentation says
    /// why, and [`Clash::actions`] is what a panel draws beside it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::CLASH.key(),
            &Filling::of("chord", self.chord.shown(strings)),
        )
    }
}

/// The refusal a person gets when the chord they pressed is already doing
/// something.
///
/// It carries the action that holds the chord, because a refusal that only said
/// *taken* would leave somebody pressing keys to find out by what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Taken {
    /// The chord that was asked for.
    chord: Chord,
    /// What already does it.
    by: Action,
}

impl Taken {
    /// A refusal of this chord, because this action has it.
    pub(crate) fn new(chord: Chord, by: Action) -> Self {
        Self { chord, by }
    }

    /// The chord that was asked for.
    #[must_use]
    pub fn chord(&self) -> Chord {
        self.chord
    }

    /// What already does it.
    #[must_use]
    pub fn by(&self) -> Action {
        self.by
    }

    /// What a person is told, in the language they read.
    ///
    /// The name of the action that has the chord goes into the sentence, and it
    /// is itself one of this crate's strings — so a German shell reads a German
    /// refusal naming a German row, rather than a German sentence with an
    /// English row inside it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::TAKEN.key(),
            &Filling::of("chord", self.chord.shown(strings))
                .and("action", self.by.said(strings).into_text()),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::key::Key;
    use crate::modifier::{Modifier, Modifiers};
    use crate::testing::{in_english, translated};

    fn chord() -> Chord {
        Chord::checked(Modifiers::just(Modifier::Super), Key::Left).unwrap()
    }

    /// A refusal names what already has the chord, and says what to do about
    /// it. "That is taken" would leave somebody guessing by what.
    #[test]
    fn a_refusal_names_what_already_has_it() {
        let taken = Taken::new(chord(), Action::SnapLeft);
        assert_eq!(taken.chord(), chord());
        assert_eq!(taken.by(), Action::SnapLeft);
        let said = taken.said(&in_english());
        assert!(said.text().contains("Super+Left"), "{said}");
        assert!(
            said.text().contains("Put the window on the left half"),
            "{said}"
        );
        assert!(said.text().contains("or use another key"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    /// **The refusal, the chord in it and the row it names are all in the one
    /// language.** A German sentence with an English row inside it is what
    /// filling a gap from somewhere that had not moved onto `alo-strings` would
    /// have produced.
    #[test]
    fn a_refusal_is_read_in_one_language_throughout() {
        let strings = translated(&[
            (words::SUPER, "Super"),
            (words::LEFT, "Pfeil links"),
            (words::SNAP_LEFT, "Fenster auf die linke Hälfte legen"),
            (
                words::TAKEN,
                "{chord} ist bereits {action} — ändern Sie zuerst diese Zuordnung, oder nehmen \
                 Sie eine andere Taste",
            ),
        ]);
        let said = Taken::new(chord(), Action::SnapLeft).said(&strings);
        assert_eq!(
            said.text(),
            "Super+Pfeil links ist bereits Fenster auf die linke Hälfte legen — ändern Sie \
             zuerst diese Zuordnung, oder nehmen Sie eine andere Taste"
        );
        assert!(said.is_translated());
    }

    /// A report says the chord is doing more than one thing, and hands over
    /// everything that wants it so a panel can draw them as rows — which is
    /// what the sentence deliberately does not try to do.
    #[test]
    fn a_report_names_the_chord_and_hands_over_what_wants_it() {
        let clash = Clash::over(chord(), vec![Action::SnapLeft, Action::NextWindow]);
        assert_eq!(clash.chord(), chord());
        assert_eq!(clash.actions(), [Action::SnapLeft, Action::NextWindow]);

        let strings = in_english();
        let said = clash.said(&strings);
        assert!(said.text().contains("Super+Left"), "{said}");
        assert!(said.text().contains("more than one thing"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");

        // What it is doing is read as rows, each in the reader's own language.
        let rows: Vec<String> = clash
            .actions()
            .iter()
            .map(|action| action.said(&strings).into_text())
            .collect();
        assert_eq!(rows, ["Put the window on the left half", "Next window"]);
    }
}
