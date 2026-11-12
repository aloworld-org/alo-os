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

use crate::accessibility_tree::AccessibilityTree;
use crate::adapters::Adapters;
use crate::by_their_hand::pressed;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{ACTIVATE_CONTROL, KIND, NAME};
use crate::fallback_words as words;
use crate::not_pressed::{TheControl, WorkingFully};
use crate::pressable::Pressable;
use crate::walking::Whose;

/// The names of the actions that press a control, in the order they are
/// looked for, matched without regard to case. Toolkits name the same act
/// differently: GTK's buttons *click*, others *press*, *activate* or *toggle*.
///
/// **`check`, `uncheck` and `select` are Chromium's**, in its complete mode
/// (`alo_launching::THE_PAGE_FULLY`): a toggle button such as VSCodium's
/// *Toggle Panel* offers `check` while it is up and `uncheck` while it is
/// down, never both, so pressing it is always the one it offers and always
/// turns it over; a menu item offers `select`. Each was measured to change
/// something, twice, by hand, on 2026-11-12 in VSCodium 1.143 — `check`
/// opened the panel and `uncheck` closed it, `select` opened the File menu and
/// then made a new file from it — and not on an alo machine
/// (`docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`).
/// They come last, so a control that also offers one of the others is pressed
/// by that one.
///
/// **`doDefault` is never here.** In Chromium's basic mode it is all a page's
/// controls offer, and it answers *done* while doing nothing, so alo would
/// report a press that did not happen. `open` and `clickAncestor` were seen
/// and not measured, so they are not here either.
pub const PRESSING: [&str; 8] = [
    "click", "press", "activate", "toggle", "jump", "check", "uncheck", "select",
];

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
    ///
    /// `fully` says whether a switch the person can turn on would give the
    /// control a press, which the refusal then names ([`WorkingFully`]).
    pub fn press(
        self,
        tree: &dyn AccessibilityTree,
        fully: WorkingFully,
        strings: &Strings,
    ) -> Result<Pressed, Refused> {
        // **The one press** a person's command takes too (`crate::by_their_hand`),
        // so what alo is refused and what a person is refused are one list.
        let whose = Whose::Application(&self.application);
        let reached = pressed(tree, whose, None, self.kind, &self.name).map_err(|why| {
            Refused::worded_elsewhere(
                self.authorised.call().clone(),
                why.said(whose, None, self.kind, &self.name, fully, strings),
            )
        })?;
        Ok(Pressed {
            authorised: self.authorised,
            application: self.application,
            kind: self.kind,
            name: self.name,
            unanswered: reached.unanswered,
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
