//! **Opening one item of a list, for one approval** — a file in a folder, a
//! row of an archive (`accessible.open_item`, added 2026-11-17). The owner's
//! answer to whether alo may open items: *"Yes but it should be without okay
//! on full access"* — a card at *ask each time*, none at full access.
//!
//! [`Opening::of`] takes the authority an approval was redeemed for **by
//! value** and asks everything `crate::fallback_reach` asks before the tree
//! is touched — so a terminal nobody allowed their agent is refused there, at
//! every level — and [`Opening::open`] consumes it: one approval, one opening.
//!
//! # Found again, at the moment it is opened
//!
//! As for selecting (`crate::choosing`), the item is looked for **then**, in
//! one walk of the application's windows: on the screen, named exactly what
//! the person approved (or, where it has no name, showing those words),
//! usable, and the only one.
//!
//! # Opened by the item's own action
//!
//! The one a screen reader's *activate* sends, as a double-click would —
//! [`OPENING`], and never *select* or *click*, which in many toolkits only
//! select. What the item then opens with is the application's choice, and the
//! approval card says so in plain words: a file may open in another
//! application, and a program may start. Nothing reads back that it opened,
//! because what opens may be another application's window.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Said, Strings};

use crate::accessibility_tree::{AccessibilityTree, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{NAME, OPEN_ITEM};
use crate::fallback_words::{NOT_THIS_VERB, VERB};
use crate::not_opened::{NotOpened, filling};
use crate::opening_words as words;
use crate::role::Role;
use crate::walking::{Whose, walk};

/// The actions an item is opened by, in the order they are looked for,
/// matched without regard to case: GTK's rows and Qt's items *activate*.
/// **Never *select*, *click* or *press***, which in many toolkits only select.
pub const OPENING: [&str; 3] = ["activate", "open", "invoke"];

/// An approved opening that has not happened yet. Deliberately not `Clone`.
#[derive(Debug)]
pub struct Opening {
    /// What may run.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The item, by what it shows.
    name: String,
}

impl Opening {
    /// Ask everything that is asked before the item is looked for.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: another verb's authority, the
    /// application not granted (a terminal nobody allowed among them) or not
    /// installed, or one with an adapter.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, OPEN_ITEM, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let Some(Value::Name(name)) = call.value(NAME) else {
            // A call of this verb that validated has a name; one that did not
            // cannot have been authorised. Nothing is opened either way.
            let said = strings.say(
                &NOT_THIS_VERB.key(),
                &alo_strings::Filling::of(VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        let name = name.clone();
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            name,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Find the item now, and open it once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was opened, in the words a person is told. An
    /// application that did not answer is **not** an error: [`Opened::unanswered`].
    pub fn open(self, tree: &dyn AccessibilityTree, strings: &Strings) -> Result<Opened, Refused> {
        let refused = |why: NotOpened, this: &Self| {
            Refused::worded_elsewhere(
                this.authorised.call().clone(),
                why.said(&this.application, &this.name, strings),
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
            .choosable
            .iter()
            .filter(|found| found.role == Role::Item && found.called == self.name);
        let found = match (
            matching.next(),
            matching.next(),
            walked.shown.not_all_read(),
        ) {
            (_, Some(_), _) => return Err(refused(NotOpened::MoreThanOne, &self)),
            (_, None, true) => return Err(refused(NotOpened::TooMuchToBeSure, &self)),
            (None, None, false) => return Err(refused(NotOpened::NoSuchItem, &self)),
            (Some(found), None, false) => found,
        };
        if !found.usable {
            return Err(refused(NotOpened::CannotBeUsedNow, &self));
        }
        let at = found.at.clone();
        let actions = match tree.actions(&at) {
            Ok(actions) => actions,
            Err(TreeFault::Vanished) => return Err(refused(NotOpened::NoSuchItem, &self)),
            Err(fault) => {
                let said = crate::NotWalked::Tree(fault).said(&self.application, None, strings);
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        let Some(action) = OPENING.iter().find_map(|opening| {
            actions
                .iter()
                .position(|action| action.eq_ignore_ascii_case(opening))
        }) else {
            return Err(refused(NotOpened::CannotBeOpened, &self));
        };
        let unanswered = match tree.act(&at, action) {
            Ok(true) => false,
            Ok(false) => return Err(refused(NotOpened::DidNotOpen, &self)),
            Err(TreeFault::DidNotAnswer) => true,
            Err(TreeFault::Vanished) => return Err(refused(NotOpened::NoSuchItem, &self)),
            Err(fault @ TreeFault::NotThere) => {
                let said = crate::NotWalked::Tree(fault).said(&self.application, None, strings);
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        Ok(Opened {
            authorised: self.authorised,
            application: self.application,
            name: self.name,
            unanswered,
        })
    }
}

/// An item that was opened — or asked to be, of an application that did not
/// answer.
#[derive(Debug)]
pub struct Opened {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The item, by what it shows.
    name: String,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl Opened {
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

    /// The item, by what it shows.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the application did not answer, so whether it opened is not
    /// known.
    #[must_use]
    pub fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// What a person is told beyond the sentence they approved — only when
    /// the application did not answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.unanswered.then(|| {
            strings.say(
                &words::DID_NOT_ANSWER.key(),
                &filling(&self.application, &self.name),
            )
        })
    }
}
