//! Pressing one control, for one approval.
//!
//! [`Activating::of`] takes the authority an approval was redeemed for **by
//! value** and asks it everything `crate::fallback_reach` asks, before the tree
//! is touched. [`Activating::press`] consumes it: one approval, one press.
//!
//! # Found again, at the moment it is pressed
//!
//! The control is looked for **then**, in a walk of the application's windows
//! at that moment — never through anything read earlier, which could describe
//! a window that has since changed. It must be on the screen, of the kind the
//! person approved, and named exactly what they approved; and it must be the
//! only one:
//!
//! - **none** is refused, and so is **more than one** — which was meant is
//!   never guessed, and there is no position to guess with;
//! - a window too large to read whole is refused too, because *the only one*
//!   cannot then be known;
//! - a control that is greyed out, or that offers no way to be pressed from
//!   outside the application, is refused in words.
//!
//! The press itself is the control's own action — the one a screen reader's
//! *activate* sends, named in [`PRESSING`] and never chosen by the agent.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Said, Strings};

use crate::accessibility_tree::{AccessibilityTree, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{ACTIVATE_CONTROL, KIND, NAME};
use crate::fallback_words as words;
use crate::not_pressed::{NotPressed, TheControl};
use crate::not_walked::NotWalked;
use crate::pressable::Pressable;
use crate::walking::{Found, Walked, walk};

/// The names of the actions that press a control, in the order they are
/// looked for, matched without regard to case. Toolkits name the same act
/// differently: GTK's buttons *click*, others *press*, *activate* or *toggle*.
pub const PRESSING: [&str; 5] = ["click", "press", "activate", "toggle", "jump"];

/// An approved press of one control that has not happened yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct Activating {
    /// What may run.
    authorised: Authorised,
    /// The application the control is in.
    application: String,
    /// The kind of control approved.
    kind: Pressable,
    /// The name approved.
    name: String,
}

impl Activating {
    /// Ask everything that is asked before a control is looked for.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: another verb's authority, the
    /// application not granted (the grants' own words) or not installed, or an
    /// application with an adapter of its own.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(
            authorised,
            ACTIVATE_CONTROL,
            adapters,
            grants,
            installed,
            strings,
        )?;
        let call = reached.authorised.call();
        let kind = match call.value(KIND) {
            Some(Value::Choice { chosen, .. }) => Pressable::called(chosen),
            _ => None,
        };
        let name = match call.value(NAME) {
            Some(Value::Name(name)) => Some(name.clone()),
            _ => None,
        };
        let (Some(kind), Some(name)) = (kind, name) else {
            // A call of this verb that validated has both; one that did not
            // cannot have been authorised. Nothing is pressed either way.
            let said = strings.say(
                &words::NOT_THIS_VERB.key(),
                &alo_strings::Filling::of(words::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            kind,
            name,
        })
    }

    /// The application the control is in.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Find the control now, and press it once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was pressed, in the words a person is told. An
    /// application that did not answer the press is **not** an error: it may
    /// have pressed it, and [`Pressed::unanswered`] says so.
    pub fn press(
        self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<Pressed, Refused> {
        let control = TheControl {
            application: &self.application,
            kind: self.kind,
            name: &self.name,
        };
        let refused_with =
            |said: Said| Refused::worded_elsewhere(self.authorised.call().clone(), said);
        let not_pressed = |why: NotPressed| refused_with(why.said(control, strings));
        let not_walked = |why: NotWalked| refused_with(why.said(&self.application, strings));

        let Walked { shown, pressable } = walk(tree, &self.application).map_err(not_walked)?;
        let mut matching: Vec<Found> = pressable
            .into_iter()
            .filter(|found| found.kind == self.kind && found.name == self.name)
            .collect();
        let found = match (matching.len(), shown.not_all_read()) {
            (2.., _) => return Err(not_pressed(NotPressed::MoreThanOne)),
            (_, true) => return Err(not_pressed(NotPressed::TooMuchToBeSure)),
            (0, false) => return Err(not_pressed(NotPressed::NoSuchControl)),
            (_, false) => matching.remove(0),
        };
        if !found.usable {
            return Err(not_pressed(NotPressed::CannotBeUsedNow));
        }
        let actions = match tree.actions(&found.at) {
            Ok(actions) => actions,
            Err(TreeFault::Vanished) => return Err(not_pressed(NotPressed::NoSuchControl)),
            Err(fault) => return Err(not_walked(NotWalked::Tree(fault))),
        };
        let Some(action) = PRESSING.iter().find_map(|pressing| {
            actions
                .iter()
                .position(|action| action.eq_ignore_ascii_case(pressing))
        }) else {
            return Err(not_pressed(NotPressed::CannotBePressed));
        };
        let unanswered = match tree.act(&found.at, action) {
            Ok(true) => false,
            Ok(false) => return Err(not_pressed(NotPressed::DidNotPress)),
            Err(TreeFault::DidNotAnswer) => true,
            Err(TreeFault::Vanished) => return Err(not_pressed(NotPressed::NoSuchControl)),
            Err(fault @ TreeFault::NotThere) => return Err(not_walked(NotWalked::Tree(fault))),
        };
        Ok(Pressed {
            authorised: self.authorised,
            application: self.application,
            kind: self.kind,
            name: self.name,
            unanswered,
        })
    }
}

/// A control that was pressed — or asked to be, by an application that did
/// not answer.
#[derive(Debug)]
pub struct Pressed {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The kind of control.
    kind: Pressable,
    /// Its name.
    name: String,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl Pressed {
    /// What ran.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// Whether the application did not answer, so whether it was pressed is
    /// not known.
    #[must_use]
    pub fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// What a person is told beyond the sentence they approved — only when the
    /// application did not answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.unanswered.then(|| {
            let control = TheControl {
                application: &self.application,
                kind: self.kind,
                name: &self.name,
            };
            strings.say(
                &words::DID_NOT_ANSWER_PRESSING.key(),
                &control.filling(strings),
            )
        })
    }

    /// Give back what ran, so it can be recorded.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
