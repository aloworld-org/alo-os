//! **Moving a slider or a number box to a number, for one approval**
//! (`accessible.set_value`, added 2026-11-17 by the owner's *"Yes do all"*).
//!
//! [`Setting::of`] takes the authority an approval was redeemed for **by
//! value** and asks everything `crate::fallback_reach` asks before the tree is
//! touched; [`Setting::set`] consumes it: one approval, one move.
//!
//! # Found again, at the moment it is moved
//!
//! As for a press (`crate::activating`), the control is looked for **then**,
//! in one walk of the application's windows: a slider or a number box on the
//! screen, named exactly what the person approved, usable, and the only thing
//! of that name holding a number. A progress bar or a meter of that name is
//! refused in its own words — it shows a number nobody can change.
//!
//! # Inside the range the control itself says
//!
//! The control's least and most are asked of it at that moment, and a number
//! outside them is refused before anything moves, naming the range
//! (`docs/contracts/terminal-commands.md`: *`set` takes a number inside the
//! range the control itself reports and refuses one outside it*). A control
//! that says no range cannot be moved from outside, and is said so.
//!
//! The number is put through AT-SPI's `Value`, never by keys or a drag, and
//! the caller reads it back ([`Set::reads_back`]).

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Said, Strings};

use crate::accessibility_tree::{AccessibilityTree, NodeAt, TreeFault};
use crate::fallback_reach::reached;
use crate::fallback_verbs::{NAME, SET_VALUE, VALUE};
use crate::fallback_words::{NOT_THIS_VERB, VERB};
use crate::not_set::{NotSet, filling};
use crate::setting_words as words;
use crate::walking::{Whose, walk};

/// How close a number read back must be to the one asked for, beside half the
/// control's own step: what a toolkit's rounding of a whole number can move.
const CLOSE_ENOUGH: f64 = 1e-6;

/// An approved move that has not happened yet. Deliberately not `Clone`.
#[derive(Debug)]
pub struct Setting {
    /// What may run.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The control, by its name.
    name: String,
    /// The number asked for.
    value: i32,
}

impl Setting {
    /// Ask everything that is asked before the control is looked for.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: another verb's authority, the
    /// application not granted or not installed, or one with an adapter.
    pub fn of(
        authorised: Authorised,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, SET_VALUE, grants, installed, strings)?;
        let call = reached.authorised.call();
        let name = match call.value(NAME) {
            Some(Value::Name(name)) => Some(name.clone()),
            _ => None,
        };
        let value = match call.value(VALUE) {
            Some(Value::Count(value)) => i32::try_from(*value).ok(),
            _ => None,
        };
        let (Some(name), Some(value)) = (name, value) else {
            // A call of this verb that validated has both, inside the range
            // the declaration gives; one that did not cannot be authorised.
            let said = strings.say(
                &NOT_THIS_VERB.key(),
                &alo_strings::Filling::of(VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            name,
            value,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Find the control now, and move it once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was moved, in the words a person is told. An
    /// application that did not answer is **not** an error: [`Set::unanswered`].
    pub fn set(self, tree: &dyn AccessibilityTree, strings: &Strings) -> Result<Set, Refused> {
        let refused = |why: NotSet, this: &Self| {
            Refused::worded_elsewhere(
                this.authorised.call().clone(),
                why.said(
                    &this.application,
                    &this.name,
                    i64::from(this.value),
                    strings,
                ),
            )
        };
        let worded = |fault: TreeFault, this: &Self| {
            Refused::worded_elsewhere(
                this.authorised.call().clone(),
                crate::NotWalked::Tree(fault).said(&this.application, None, strings),
            )
        };
        let walked = match walk(tree, Whose::Application(&self.application), None) {
            Ok(walked) => walked,
            Err(why) => {
                let said = why.said(&self.application, None, strings);
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        let mut matching = walked
            .numbered
            .iter()
            .filter(|found| found.name == self.name);
        let found = match (
            matching.next(),
            matching.next(),
            walked.shown.not_all_read(),
        ) {
            (_, Some(_), _) => return Err(refused(NotSet::MoreThanOne, &self)),
            (_, None, true) => return Err(refused(NotSet::TooMuchToBeSure, &self)),
            (None, None, false) => return Err(refused(NotSet::NoSuchControl, &self)),
            (Some(found), None, false) => found,
        };
        if !found.movable {
            return Err(refused(NotSet::OnlyShowsANumber, &self));
        }
        if !found.usable {
            return Err(refused(NotSet::CannotBeUsedNow, &self));
        }
        let at = found.at.clone();
        let bounds = match tree.range(&at) {
            Ok(Some(bounds)) => bounds,
            Ok(None) => return Err(refused(NotSet::CannotBeMoved, &self)),
            Err(TreeFault::Vanished) => return Err(refused(NotSet::NoSuchControl, &self)),
            Err(fault) => return Err(worded(fault, &self)),
        };
        let asked = f64::from(self.value);
        if asked < bounds.least || asked > bounds.most {
            return Err(refused(NotSet::OutsideItsRange(bounds), &self));
        }
        let unanswered = match tree.set_value(&at, asked) {
            Ok(true) => false,
            Ok(false) => return Err(refused(NotSet::DidNotTake, &self)),
            Err(TreeFault::DidNotAnswer) => true,
            Err(TreeFault::Vanished) => return Err(refused(NotSet::NoSuchControl, &self)),
            Err(fault @ TreeFault::NotThere) => return Err(worded(fault, &self)),
        };
        Ok(Set {
            authorised: self.authorised,
            application: self.application,
            name: self.name,
            value: self.value,
            at,
            step: bounds.step,
            unanswered,
        })
    }
}

/// A slider or number box that was moved — or asked to be, of an application
/// that did not answer.
#[derive(Debug)]
pub struct Set {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The control's name.
    name: String,
    /// The number asked for.
    value: i32,
    /// Where it is.
    at: NodeAt,
    /// Its smallest step.
    step: f64,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl Set {
    /// What ran.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The control's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The number asked for.
    #[must_use]
    pub fn value(&self) -> i64 {
        i64::from(self.value)
    }

    /// Whether the application did not answer.
    #[must_use]
    pub fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// **Whether the control reads back as the number asked for** — within
    /// half its own step, since an application may round to its steps.
    #[must_use]
    pub fn reads_back(&self, tree: &dyn AccessibilityTree) -> bool {
        let tolerance = (self.step / 2.0).max(CLOSE_ENOUGH);
        tree.value(&self.at)
            .ok()
            .flatten()
            .is_some_and(|now| (now - f64::from(self.value)).abs() <= tolerance)
    }

    /// What a person is told beyond the sentence they approved: that the
    /// application did not answer, or — where `seen` is false — that it reads
    /// back as another number. [`None`] when it reads back as asked.
    #[must_use]
    pub fn said(&self, seen: bool, strings: &Strings) -> Option<Said> {
        let word = if self.unanswered {
            words::DID_NOT_ANSWER
        } else if seen {
            return None;
        } else {
            words::READS_BACK_DIFFERENTLY
        };
        Some(strings.say(
            &word.key(),
            &filling(&self.application, &self.name, i64::from(self.value)),
        ))
    }
}
