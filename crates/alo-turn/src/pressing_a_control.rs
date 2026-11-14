//! **`accessible.activate_control`**: one control in an application's window,
//! pressed by its own action — a change, so it runs only after one approval of
//! the sentence *press the {kind} named “{name}” in {application}*.
//!
//! The steps are `alo_adapters::Activating`'s: the grants asked again at this
//! moment, the application here, no adapter of its own, and then the control
//! looked for again in the window as it is now — exactly one usable control of
//! that kind and that name, or nothing is pressed. It is the
//! one press a person's own `alo-app … press` makes too, so alo is refused for
//! exactly what a person is.
//!
//! What this file adds is the order around it and the entry: the tree is opened
//! **after** every refusal that needs no tree, for this press alone, and
//! dropped when it ends — so one approval reaches the tree once, and a grant
//! revoked between the approval and the press stops it before any tree is
//! opened.
//!
//! # Nothing is typed, and a terminal is the person's choice
//!
//! **No argument of this verb can carry text**, so nothing alo presses types
//! anything — into a password field, a terminal, or anywhere else; a field is
//! not one of the seven kinds a press takes. A terminal is reached only where
//! the person allowed their agent one (ADR 0111): until then the grants refuse
//! it at every level, full access included, before any tree is opened.
//! `alo_capability::ATerminalForTheAgent::MayOpenOne` promises that, once
//! allowed, alo may *read the window and press controls by name* there, as in
//! any granted application, and nothing here withdraws that.

use alo_adapters::fallback_verbs::{KIND, NAME};
use alo_adapters::{Activating, NotWalked, Pressable, TreeFault, WorkingFully};
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// One control alo pressed — or asked to, of an application that did not
/// answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AControlPressed {
    /// The application it is in.
    application: String,
    /// Its kind, as a model names it (`button`, `check_box`, …).
    kind: String,
    /// Its name, as the application shows it.
    name: String,
    /// Whether the application did not answer, so whether it was pressed is
    /// not known.
    unanswered: bool,
}

impl AControlPressed {
    /// The application it is in.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Its kind, as a model names it.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Its name, as the application shows it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the application did not answer, so whether it was pressed is
    /// not known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }
}

/// Press one approved control, once.
///
/// Answers the record's entry and what happened. **A press the application did
/// not answer is recorded as having run** — the press was sent under the
/// approval — with the sentence saying whether it happened is not known; every
/// other way this ends without a press is a refusal, written down as one.
pub(crate) fn pressing_a_control(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<AControlPressed, NotDone>) {
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();
    let call = authorised.call().clone();
    let refusing = |refused: Refused| {
        let entry = Entry::refused(&refused, &agent, strings, at);
        (entry, Err(NotDone::Refused(refused)))
    };

    let activating = match Activating::of(
        authorised,
        machine.the_adapters(),
        grants,
        &machine.the_applications_installed_now(),
        strings,
    ) {
        Ok(activating) => activating,
        Err(refused) => return refusing(refused),
    };
    let application = activating.application().to_owned();

    // Opened now, for this one press, and dropped when it ends.
    let (tree, fully) = match machine.the_windows_on_the_screen() {
        Some(windows) => (windows.opening(), windows.working_fully(&application)),
        None => (Err(TreeFault::NotThere), WorkingFully::NothingMoreToTurnOn),
    };
    let tree = match tree {
        Ok(tree) => tree,
        Err(fault) => {
            let said = NotWalked::Tree(fault).said(&application, None, strings);
            return refusing(Refused::worded_elsewhere(call, said));
        }
    };
    let pressed = match activating.press(tree.as_ref(), fully, strings) {
        Ok(pressed) => pressed,
        Err(refused) => return refusing(refused),
    };
    drop(tree);

    let entry = Entry::ran(pressed.authorised(), strings);
    let entry = match pressed.said(strings) {
        Some(said) => entry.telling(&[said]),
        None => entry,
    };
    (
        entry,
        Ok(AControlPressed {
            application,
            kind: the_kind(&call).unwrap_or_default().to_owned(),
            name: the_name(&call).unwrap_or_default().to_owned(),
            unanswered: pressed.unanswered(),
        }),
    )
}

/// The kind the call named, as a model names it.
///
/// Read off the validated call rather than kept from the press, so what crosses
/// the wire is what the person approved. A call of this verb that validated
/// always has one; [`None`] cannot arrive here, since `Activating::of` refused
/// such a call above.
fn the_kind(call: &alo_capability::Call) -> Option<&'static str> {
    match call.value(KIND) {
        Some(Value::Choice { chosen, .. }) => Pressable::called(chosen).map(Pressable::named),
        _ => None,
    }
}

/// The name the call named, as the application shows it.
fn the_name(call: &alo_capability::Call) -> Option<&str> {
    match call.value(NAME) {
        Some(Value::Name(name)) => Some(name),
        _ => None,
    }
}
