//! The shortcuts on this machine: what they do, what a person changed, and what
//! happens when two of them want the same keys.
//!
//! A [`Shortcuts`] is two things — the defaults the running release ships, and
//! the changes the person made — and every answer it gives is the two of them
//! resolved at the moment of asking. Nothing is baked at load time, so a change
//! takes effect on the next key pressed rather than at the next sign-in.
//!
//! **Three rules decide what happens when one chord is wanted twice**, and they
//! are here rather than in whoever writes the compositor:
//!
//! 1. **At the moment of binding, a clash is refused.** [`Shortcuts::bind`]
//!    hands back [`Taken`] naming what already has the chord, and changes
//!    nothing. The last binding does not win, because a person who silently lost
//!    a shortcut will not connect the loss to what they did.
//! 2. **A person's binding beats one we shipped.** A release that adds a default
//!    on a chord somebody already moved something else onto cannot take it from
//!    them: their binding still fires, the new default does not, and
//!    [`Shortcuts::clashes`] says so where they can see it. Silently dropping the
//!    binding they chose in favour of the one we chose would be the wrong way
//!    round.
//! 3. **Two of a person's own bindings on one chord fire nothing.** That can
//!    only come from a file nobody validated, and picking one of the two would
//!    be right half the time — with the wrong half closing a window somebody
//!    meant to maximise.
//!
//! The clash is *reported* in all three cases. A model that could only prevent
//! clashes would have nothing to say about the one case it cannot see coming.

use crate::action::Action;
use crate::changes::Changes;
use crate::chord::Chord;
use crate::clash::{Clash, Taken};
use crate::defaults::Defaults;

/// One row of the shortcuts a person is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    /// What it does.
    pub action: Action,
    /// What does it — or `None`, when there is no shortcut for it, whether
    /// because the person cleared it or because this release ships none.
    pub chord: Option<Chord>,
    /// Whether this is the person's choice rather than the shipped one, which
    /// is what a settings panel marks and what "put it back" undoes.
    pub changed: bool,
}

/// Every shortcut on this machine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shortcuts {
    /// What this release ships.
    defaults: Defaults,
    /// What the person changed, which is the only part written down.
    changes: Changes,
}

impl Shortcuts {
    /// The shortcuts a machine has before anybody changes anything.
    #[must_use]
    pub fn shipped() -> Self {
        Self::default()
    }

    /// The same, over a set of defaults that is not the shipped one.
    #[must_use]
    pub fn over(defaults: Defaults) -> Self {
        Self {
            defaults,
            changes: Changes::none(),
        }
    }

    /// The changes read out of a settings file, applied over these defaults.
    ///
    /// **A file is not a settings panel**: it can say things [`Shortcuts::bind`]
    /// would have refused, because it may have been written by an older release
    /// or by a person with an editor. Nothing here refuses it — what it says is
    /// what its owner decided — and anything it contradicts comes back from
    /// [`Shortcuts::clashes`].
    #[must_use]
    pub fn with(mut self, changes: Changes) -> Self {
        self.changes = changes;
        self
    }

    /// What has been changed, which is what gets written down.
    #[must_use]
    pub fn changes(&self) -> &Changes {
        &self.changes
    }

    /// What this release ships, whatever the person has done since.
    #[must_use]
    pub fn defaults(&self) -> &Defaults {
        &self.defaults
    }

    /// What this action's shortcut is now, or `None` if it has none.
    #[must_use]
    pub fn chord_for(&self, action: Action) -> Option<Chord> {
        match self.changes.made_to(action) {
            Some(chord) => chord,
            None => self.defaults.chord_for(action),
        }
    }

    /// Whether this action is the person's choice rather than the shipped one.
    #[must_use]
    pub fn is_changed(&self, action: Action) -> bool {
        self.changes.made_to(action).is_some()
    }

    /// Every action and what does it, in the order a settings panel lists them.
    ///
    /// Everything the system can do is here, including anything with no
    /// shortcut at all — a person looking for the thing they cleared has to be
    /// able to find it again.
    pub fn bindings(&self) -> impl Iterator<Item = Binding> {
        Action::ALL.iter().map(|action| Binding {
            action: *action,
            chord: self.chord_for(*action),
            changed: self.is_changed(*action),
        })
    }

    /// What happens when this chord is pressed.
    ///
    /// `None` when nothing is bound to it, and `None` when two bindings the
    /// person made both want it — rule 3 above.
    #[must_use]
    pub fn action_for(&self, chord: Chord) -> Option<Action> {
        let mut wanted_by = 0_usize;
        let mut only = None;
        let mut chosen_by_the_person = 0_usize;
        let mut theirs = None;
        for binding in self.bindings() {
            if binding.chord != Some(chord) {
                continue;
            }
            wanted_by = wanted_by.saturating_add(1);
            only = Some(binding.action);
            if binding.changed {
                chosen_by_the_person = chosen_by_the_person.saturating_add(1);
                theirs = Some(binding.action);
            }
        }
        match (wanted_by, chosen_by_the_person) {
            (0, _) => None,
            (1, _) => only,
            // Rule 2: one of them is the person's, and it wins over ours.
            (_, 1) => theirs,
            // Rule 3: none of them, or two of theirs.
            _ => None,
        }
    }

    /// Everything the person is shown as doing more than one thing, in the
    /// order the chords first appear.
    ///
    /// Empty on a machine nobody has changed anything on, and a test says so.
    #[must_use]
    pub fn clashes(&self) -> Vec<Clash> {
        let mut wanting: Vec<(Chord, Vec<Action>)> = Vec::new();
        for binding in self.bindings() {
            let Some(chord) = binding.chord else { continue };
            match wanting.iter_mut().find(|(known, _)| *known == chord) {
                Some((_, actions)) => actions.push(binding.action),
                None => wanting.push((chord, vec![binding.action])),
            }
        }
        wanting
            .into_iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(chord, actions)| Clash::over(chord, actions))
            .collect()
    }

    /// Give this action this chord.
    ///
    /// Putting an action back on the chord it ships with is the same as
    /// [`Shortcuts::reset`] rather than a change that happens to match: a
    /// settings panel marking a binding as changed when it is identical to the
    /// default would be telling a person something they can see is untrue.
    ///
    /// # Errors
    /// [`Taken`], naming what already has the chord. Nothing is changed.
    pub fn bind(&mut self, action: Action, chord: Chord) -> Result<(), Taken> {
        if let Some(holder) = self
            .bindings()
            .find(|binding| binding.chord == Some(chord) && binding.action != action)
        {
            return Err(Taken::new(chord, holder.action));
        }
        if self.defaults.chord_for(action) == Some(chord) {
            self.changes.forget(action);
            return Ok(());
        }
        self.changes.set(action, Some(chord));
        Ok(())
    }

    /// Leave this action with no shortcut.
    ///
    /// A decision, and stored as one: it survives a release that changes the
    /// default, because a person who cleared a shortcut did not ask for a
    /// different one.
    pub fn unbind(&mut self, action: Action) {
        if self.defaults.chord_for(action).is_none() {
            self.changes.forget(action);
            return;
        }
        self.changes.set(action, None);
    }

    /// Put this action back to what it ships with. Says whether it had been
    /// changed at all.
    pub fn reset(&mut self, action: Action) -> bool {
        self.changes.forget(action)
    }

    /// Put everything back to what it ships with.
    pub fn reset_everything(&mut self) {
        self.changes.forget_everything();
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

    fn zuper(key: Key) -> Chord {
        Chord::checked(Modifiers::just(Modifier::Super), key).unwrap()
    }

    fn ctrl_alt(key: Key) -> Chord {
        Chord::checked(Modifiers::just(Modifier::Ctrl).and(Modifier::Alt), key).unwrap()
    }

    /// A machine nobody has touched answers out of the shipped list, in both
    /// directions, for every action there is — and writes nothing down.
    #[test]
    fn an_untouched_machine_answers_out_of_the_shipped_list() {
        let shortcuts = Shortcuts::shipped();
        assert!(shortcuts.changes().is_empty());
        assert!(shortcuts.clashes().is_empty(), "we ship a clash");
        for action in Action::ALL {
            let chord = shortcuts.chord_for(*action).unwrap();
            assert!(!shortcuts.is_changed(*action));
            assert_eq!(shortcuts.action_for(chord), Some(*action));
        }
        assert_eq!(shortcuts.bindings().count(), Action::ALL.len());
    }

    /// Changing one moves it: the new chord does it, the old one does nothing,
    /// and the row is marked as the person's.
    #[test]
    fn changing_one_moves_it_and_frees_what_it_had() {
        let mut shortcuts = Shortcuts::shipped();
        let was = shortcuts.chord_for(Action::TheAgent).unwrap();
        shortcuts
            .bind(Action::TheAgent, ctrl_alt(Key::Space))
            .unwrap();

        assert_eq!(
            shortcuts.chord_for(Action::TheAgent),
            Some(ctrl_alt(Key::Space))
        );
        assert_eq!(
            shortcuts.action_for(ctrl_alt(Key::Space)),
            Some(Action::TheAgent)
        );
        assert_eq!(
            shortcuts.action_for(was),
            None,
            "the old chord still does it"
        );
        assert!(shortcuts.is_changed(Action::TheAgent));
        assert_eq!(shortcuts.changes().len(), 1);
    }

    /// **The refusal.** A chord another action already has is refused, the
    /// refusal names what has it, and — the part worth testing — nothing at all
    /// changed.
    #[test]
    fn a_chord_something_else_has_is_refused_and_changes_nothing() {
        let mut shortcuts = Shortcuts::shipped();
        let taken = shortcuts.chord_for(Action::SnapLeft).unwrap();
        let refused = shortcuts.bind(Action::TheAgent, taken).unwrap_err();

        assert_eq!(refused.by(), Action::SnapLeft);
        assert_eq!(refused.chord(), taken);
        assert_eq!(
            shortcuts,
            Shortcuts::shipped(),
            "a refusal changed something"
        );
        assert!(shortcuts.changes().is_empty());
        assert_eq!(shortcuts.action_for(taken), Some(Action::SnapLeft));
    }

    /// The chord has to be free, not unused-by-default: clearing the action
    /// that had it is what frees it, and then the same binding is allowed.
    #[test]
    fn a_chord_the_person_freed_can_then_be_taken() {
        let mut shortcuts = Shortcuts::shipped();
        let wanted = shortcuts.chord_for(Action::SnapLeft).unwrap();
        assert!(shortcuts.bind(Action::TheAgent, wanted).is_err());

        shortcuts.unbind(Action::SnapLeft);
        shortcuts.bind(Action::TheAgent, wanted).unwrap();
        assert_eq!(shortcuts.action_for(wanted), Some(Action::TheAgent));
        assert_eq!(shortcuts.chord_for(Action::SnapLeft), None);
    }

    /// **Wanting no shortcut is a decision.** It is written down, so it is still
    /// true after a restart, and the chord it used to answer to does nothing.
    #[test]
    fn clearing_a_shortcut_is_kept() {
        let mut shortcuts = Shortcuts::shipped();
        let was = shortcuts.chord_for(Action::CloseWindow).unwrap();
        shortcuts.unbind(Action::CloseWindow);

        assert_eq!(shortcuts.chord_for(Action::CloseWindow), None);
        assert_eq!(shortcuts.action_for(was), None);
        assert!(shortcuts.is_changed(Action::CloseWindow));

        let written = shortcuts.changes().clone();
        assert_eq!(
            Shortcuts::shipped()
                .with(written)
                .chord_for(Action::CloseWindow),
            None
        );
    }

    /// Putting one back forgets the change, and putting everything back forgets
    /// all of them.
    #[test]
    fn putting_it_back_forgets_the_change() {
        let mut shortcuts = Shortcuts::shipped();
        shortcuts.bind(Action::TheAgent, ctrl_alt(Key::A)).unwrap();
        shortcuts.unbind(Action::Launcher);
        assert_eq!(shortcuts.changes().len(), 2);

        assert!(shortcuts.reset(Action::TheAgent));
        assert!(!shortcuts.reset(Action::TheAgent), "it was already back");
        assert!(!shortcuts.is_changed(Action::TheAgent));

        shortcuts.reset_everything();
        assert_eq!(shortcuts, Shortcuts::shipped());
    }

    /// Setting an action to the chord it already ships with is putting it back,
    /// not a change that happens to look identical — otherwise the panel would
    /// mark a row as changed while showing the default beside it.
    #[test]
    fn binding_the_shipped_chord_is_putting_it_back() {
        let mut shortcuts = Shortcuts::shipped();
        let shipped = shortcuts.chord_for(Action::MaximiseWindow).unwrap();
        shortcuts
            .bind(Action::MaximiseWindow, ctrl_alt(Key::Up))
            .unwrap();
        assert!(shortcuts.is_changed(Action::MaximiseWindow));

        shortcuts.bind(Action::MaximiseWindow, shipped).unwrap();
        assert!(!shortcuts.is_changed(Action::MaximiseWindow));
        assert!(shortcuts.changes().is_empty());
    }

    /// **The clash no refusal could have prevented.** A release adds a default
    /// on a chord somebody had already moved something else onto: their binding
    /// still fires, ours does not, and the clash is shown rather than resolved
    /// behind their back.
    #[test]
    fn a_new_default_never_takes_a_chord_the_person_already_moved() {
        let wanted = zuper(Key::Left);
        let before = Defaults::of(vec![(Action::NextWindow, zuper(Key::Tab))]).unwrap();
        let mut theirs = Shortcuts::over(before);
        theirs.bind(Action::NextWindow, wanted).unwrap();

        // The release that adds snapping, on the chord they are already using.
        let after = Defaults::of(vec![
            (Action::NextWindow, zuper(Key::Tab)),
            (Action::SnapLeft, wanted),
        ])
        .unwrap();
        let upgraded = Shortcuts::over(after).with(theirs.changes().clone());

        assert_eq!(upgraded.action_for(wanted), Some(Action::NextWindow));
        assert_eq!(upgraded.chord_for(Action::SnapLeft), Some(wanted));
        let clashes = upgraded.clashes();
        assert_eq!(clashes.len(), 1);
        assert_eq!(clashes.first().map(Clash::chord), Some(wanted));
        assert_eq!(
            clashes.first().map(Clash::actions),
            Some([Action::SnapLeft, Action::NextWindow].as_slice())
        );
    }

    /// **Two of the person's own bindings on one chord fire nothing.** Only a
    /// file nobody validated can produce it, and choosing between them would be
    /// wrong half the time — closing a window somebody meant to maximise.
    #[test]
    fn two_bindings_of_their_own_on_one_chord_do_nothing() {
        let both = ctrl_alt(Key::M);
        let mut changes = Changes::none();
        changes.set(Action::MaximiseWindow, Some(both));
        changes.set(Action::MinimiseWindow, Some(both));
        let shortcuts = Shortcuts::shipped().with(changes);

        assert_eq!(shortcuts.action_for(both), None);
        let clashes = shortcuts.clashes();
        assert_eq!(clashes.len(), 1);
        assert_eq!(
            clashes.first().map(Clash::actions),
            Some([Action::MinimiseWindow, Action::MaximiseWindow].as_slice()),
            "shown in the order the panel lists them"
        );
    }

    /// The rows a settings panel draws: everything the system does, what does
    /// it, and which ones are the person's own.
    #[test]
    fn the_rows_show_everything_including_what_was_cleared() {
        let mut shortcuts = Shortcuts::shipped();
        shortcuts.unbind(Action::NextApplication);
        shortcuts
            .bind(Action::TheAgent, ctrl_alt(Key::Space))
            .unwrap();

        let rows: Vec<Binding> = shortcuts.bindings().collect();
        assert_eq!(rows.len(), Action::ALL.len());
        let cleared = rows
            .iter()
            .find(|row| row.action == Action::NextApplication);
        assert_eq!(
            cleared.map(|row| (row.chord, row.changed)),
            Some((None, true))
        );
        let moved = rows.iter().find(|row| row.action == Action::TheAgent);
        assert_eq!(
            moved.map(|row| (row.chord, row.changed)),
            Some((Some(ctrl_alt(Key::Space)), true))
        );
        assert_eq!(rows.iter().filter(|row| row.changed).count(), 2);
    }

    /// An action this release ships no default for is shown, unbound, and can
    /// be given a chord — which is what the first release after one that adds
    /// an action looks like if the default is ever left out.
    #[test]
    fn an_action_with_no_default_is_shown_unbound_and_can_be_given_one() {
        let defaults = Defaults::of(vec![(Action::TheAgent, zuper(Key::A))]).unwrap();
        let mut shortcuts = Shortcuts::over(defaults);
        assert_eq!(shortcuts.chord_for(Action::SnapRight), None);
        assert!(!shortcuts.is_changed(Action::SnapRight));

        shortcuts.unbind(Action::SnapRight);
        assert!(
            shortcuts.changes().is_empty(),
            "clearing something already unbound is not a change to keep"
        );

        shortcuts
            .bind(Action::SnapRight, ctrl_alt(Key::Right))
            .unwrap();
        assert_eq!(
            shortcuts.action_for(ctrl_alt(Key::Right)),
            Some(Action::SnapRight)
        );
    }
}
