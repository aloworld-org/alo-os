//! **Choosing in an application's window, for one approval** — an option of a
//! drop-down (`accessible.choose_option`) or an item of a list
//! (`accessible.select_item`), added 2026-11-17 by the owner's *"Yes do
//! all"*.
//!
//! [`Choosing::of`] takes the authority an approval was redeemed for **by
//! value** and asks it everything `crate::fallback_reach` asks before the tree
//! is touched; [`Choosing::choose`] consumes it: one approval, one choice.
//!
//! # Found again, at the moment it is chosen
//!
//! As for a press (`crate::activating`), the drop-down or item is looked for
//! **then**, in one walk of the application's windows as they are, and must be
//! on the screen, named exactly what the person approved, usable, and the only
//! one. It is matched by its name, or — where it shows none — by the text it
//! shows, so it is found by what `accessible.read_window` said of it.
//!
//! **An option is looked for inside the drop-down alone**, whether or not it
//! is on the screen: a closed drop-down keeps its options out of sight, and
//! choosing one is what the drop-down is for. Nothing outside that one
//! drop-down is read this way, only names are compared, and no text is asked
//! of anything — so a password field's contents are never read here either.
//!
//! # How it is chosen
//!
//! Through the list that holds it, as a screen reader chooses
//! ([`AccessibilityTree::select_in`]); where the list offers no way, by the
//! thing's own action — for an option [`CHOOSING_AN_OPTION`], for an item
//! only [`SELECTING_AN_ITEM`], because an item's *activate* opens it, and
//! *select* must never open a file. **Opening an item is not here** but in
//! `crate::opening` (`accessible.open_item`), its own verb with its own card,
//! because what it starts is the application's to decide.
//!
//! What was chosen is read back by the caller ([`Chosen::shows_chosen`]): the
//! item or option says it is selected, or the drop-down now shows the option.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Said, Strings};

use crate::accessibility_tree::{AccessibilityTree, NodeAt, TreeFault};
use crate::choosing_words as words;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{CHOOSE_OPTION, NAME, OPTION, SELECT_ITEM};
use crate::fallback_words::NOT_THIS_VERB;
use crate::not_chosen::{NotChosen, filling};
use crate::role::Role;
use crate::states::States;
use crate::walking::{Choosable, DEEPEST, MOST_THINGS, Walked, Whose, walk};

/// The actions an option is chosen by where its list offers no selection, in
/// the order they are looked for, matched without regard to case: Chromium's
/// options *select*; a GTK drop-down's menu items *click*.
pub const CHOOSING_AN_OPTION: [&str; 4] = ["select", "click", "press", "activate"];

/// The one action an item is selected by where its list offers no selection.
/// **Never *activate*, *click* or *open***, which open an item in many
/// toolkits — a file in a file manager.
pub const SELECTING_AN_ITEM: [&str; 1] = ["select"];

/// What a person approved choosing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToChoose {
    /// One option of one drop-down.
    AnOption {
        /// The drop-down, by what it shows.
        drop_down: String,
        /// The option, exactly as the drop-down lists it.
        option: String,
    },
    /// One item of a list.
    AnItem {
        /// The item, by what it shows.
        item: String,
    },
}

impl ToChoose {
    /// The drop-down or the item, by what it shows.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::AnOption { drop_down, .. } => drop_down,
            Self::AnItem { item } => item,
        }
    }

    /// The option, for a drop-down.
    #[must_use]
    pub fn option(&self) -> Option<&str> {
        match self {
            Self::AnOption { option, .. } => Some(option),
            Self::AnItem { .. } => None,
        }
    }
}

/// An approved choice that has not been made yet. Deliberately not `Clone`,
/// like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct Choosing {
    /// What may run.
    authorised: Authorised,
    /// The application.
    application: String,
    /// What to choose.
    to_choose: ToChoose,
}

impl Choosing {
    /// Ask everything that is asked before anything is looked for.
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
        let verb = if authorised.verb() == SELECT_ITEM {
            SELECT_ITEM
        } else {
            CHOOSE_OPTION
        };
        let reached = reached(authorised, verb, grants, installed, strings)?;
        let call = reached.authorised.call();
        let named = |argument| match call.value(argument) {
            Some(Value::Name(name)) => Some(name.clone()),
            _ => None,
        };
        let to_choose = match (verb, named(NAME), named(OPTION)) {
            (SELECT_ITEM, Some(item), _) => Some(ToChoose::AnItem { item }),
            (CHOOSE_OPTION, Some(drop_down), Some(option)) => {
                Some(ToChoose::AnOption { drop_down, option })
            }
            _ => None,
        };
        let Some(to_choose) = to_choose else {
            // A call of either verb that validated has its arguments; one that
            // did not cannot have been authorised. Nothing is chosen either way.
            let said = strings.say(
                &NOT_THIS_VERB.key(),
                &alo_strings::Filling::of(crate::fallback_words::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            to_choose,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Find it now, and choose it once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was chosen, in the words a person is told. An
    /// application that did not answer is **not** an error: it may have
    /// chosen it, and [`Chosen::unanswered`] says so.
    pub fn choose(
        self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<Chosen, Refused> {
        let refused = |why: NotChosen, this: &Self| {
            Refused::worded_elsewhere(
                this.authorised.call().clone(),
                why.said(
                    &this.application,
                    this.to_choose.name(),
                    this.to_choose.option(),
                    strings,
                ),
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
        let found = match the_one(&walked, &self.to_choose) {
            Ok(found) => found,
            Err(why) => return Err(refused(why, &self)),
        };
        let (list, at, drop_down) = match &self.to_choose {
            ToChoose::AnItem { .. } => (found.list.clone(), found.at.clone(), None),
            ToChoose::AnOption { option, .. } => match the_option(tree, &found.at, option) {
                Ok((list, at)) => (list, at, Some(found.at.clone())),
                Err(Some(why)) => return Err(refused(why, &self)),
                Err(None) => {
                    let said = crate::NotWalked::Tree(TreeFault::DidNotAnswer).said(
                        &self.application,
                        None,
                        strings,
                    );
                    return Err(Refused::worded_elsewhere(
                        self.authorised.call().clone(),
                        said,
                    ));
                }
            },
        };
        let actions: &[&str] = match self.to_choose {
            ToChoose::AnOption { .. } => &CHOOSING_AN_OPTION,
            ToChoose::AnItem { .. } => &SELECTING_AN_ITEM,
        };
        let gone = match self.to_choose {
            ToChoose::AnOption { .. } => NotChosen::NoSuchOption,
            ToChoose::AnItem { .. } => NotChosen::NoSuchItem,
        };
        let unanswered = match chosen_through(tree, &list, &at, actions, gone) {
            Ok(unanswered) => unanswered,
            Err(Some(why)) => return Err(refused(why, &self)),
            Err(None) => {
                let said = crate::NotWalked::Tree(TreeFault::NotThere).said(
                    &self.application,
                    None,
                    strings,
                );
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        Ok(Chosen {
            authorised: self.authorised,
            application: self.application,
            to_choose: self.to_choose,
            at,
            drop_down,
            unanswered,
        })
    }
}

/// The one drop-down or item named, showing, usable.
fn the_one<'w>(walked: &'w Walked, to_choose: &ToChoose) -> Result<&'w Choosable, NotChosen> {
    let (role, missing) = match to_choose {
        ToChoose::AnOption { .. } => (Role::DropDown, NotChosen::NoSuchDropDown),
        ToChoose::AnItem { .. } => (Role::Item, NotChosen::NoSuchItem),
    };
    let mut matching = walked
        .choosable
        .iter()
        .filter(|found| found.role == role && found.called == to_choose.name());
    let found = match (
        matching.next(),
        matching.next(),
        walked.shown.not_all_read(),
    ) {
        (_, Some(_), _) => return Err(NotChosen::MoreThanOne),
        (_, None, true) => return Err(NotChosen::TooMuchToBeSure),
        (None, None, false) => return Err(missing),
        (Some(found), None, false) => found,
    };
    if found.usable {
        Ok(found)
    } else {
        Err(NotChosen::CannotBeUsedNow)
    }
}

/// The option named `option` inside the drop-down at `drop_down`, and the
/// list that holds it — looked for whether or not it is on the screen, and
/// never outside the drop-down. `Err(None)` is a tree that could not be asked.
fn the_option(
    tree: &dyn AccessibilityTree,
    drop_down: &NodeAt,
    option: &str,
) -> Result<(NodeAt, NodeAt), Option<NotChosen>> {
    let mut found = Vec::new();
    let mut read = 0;
    let mut waiting = vec![(drop_down.clone(), 0_usize)];
    while let Some((parent, depth)) = waiting.pop() {
        let facts = match tree.facts(&parent) {
            Ok(facts) => facts,
            Err(TreeFault::Vanished) => continue,
            Err(_) => return Err(None),
        };
        if (depth >= DEEPEST && !facts.children.is_empty()) || facts.unlisted_children {
            return Err(Some(NotChosen::TooMuchToBeSure));
        }
        for child in facts.children.into_iter().rev() {
            if child.holder != drop_down.holder {
                continue;
            }
            read += 1;
            if read > MOST_THINGS {
                return Err(Some(NotChosen::TooMuchToBeSure));
            }
            let named = match tree.facts(&child) {
                Ok(facts) => facts.name.trim() == option,
                Err(TreeFault::Vanished) => false,
                Err(_) => return Err(None),
            };
            if named {
                found.push((parent.clone(), child.clone()));
            }
            waiting.push((child, depth + 1));
        }
    }
    let mut found = found.into_iter();
    match (found.next(), found.next()) {
        (Some(one), None) => Ok(one),
        (Some(_), Some(_)) => Err(Some(NotChosen::MoreThanOneOption)),
        (None, _) => Err(Some(NotChosen::NoSuchOption)),
    }
}

/// Choose `at` through `list`, or by one of `actions` — answering whether the
/// application did not answer. `gone` is what a thing that vanished meanwhile
/// is said as; `Err(None)` is a tree that is no longer there.
fn chosen_through(
    tree: &dyn AccessibilityTree,
    list: &NodeAt,
    at: &NodeAt,
    actions: &[&str],
    gone: NotChosen,
) -> Result<bool, Option<NotChosen>> {
    let answered = |answer: Result<bool, TreeFault>| match answer {
        Ok(true) => Ok(false),
        Ok(false) => Err(Some(NotChosen::DidNotChoose)),
        Err(TreeFault::DidNotAnswer) => Ok(true),
        Err(TreeFault::Vanished) => Err(Some(gone)),
        Err(TreeFault::NotThere) => Err(None),
    };
    match tree.select_in(list, at) {
        Ok(Some(said)) => return answered(Ok(said)),
        Ok(None) => {}
        Err(fault) => return answered(Err(fault)),
    }
    let offered = match tree.actions(at) {
        Ok(offered) => offered,
        Err(fault) => return answered(Err(fault)),
    };
    let Some(action) = actions.iter().find_map(|wanted| {
        offered
            .iter()
            .position(|action| action.eq_ignore_ascii_case(wanted))
    }) else {
        return Err(Some(NotChosen::CannotBeChosen));
    };
    answered(tree.act(at, action))
}

/// A choice that was made — or asked for, of an application that did not
/// answer.
#[derive(Debug)]
pub struct Chosen {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// What was chosen.
    to_choose: ToChoose,
    /// The option or item asked to be chosen.
    at: NodeAt,
    /// The drop-down, for an option.
    drop_down: Option<NodeAt>,
    /// Whether the application did not answer.
    unanswered: bool,
}

impl Chosen {
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

    /// What was chosen.
    #[must_use]
    pub fn to_choose(&self) -> &ToChoose {
        &self.to_choose
    }

    /// Whether the application did not answer, so whether it chose is not
    /// known.
    #[must_use]
    pub fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// **Whether the window shows it chosen now**: the option or item says it
    /// is selected, or the drop-down shows the option by its name or its text.
    /// A tree that cannot be asked shows nothing.
    #[must_use]
    pub fn shows_chosen(&self, tree: &dyn AccessibilityTree) -> bool {
        let selected = tree
            .facts(&self.at)
            .is_ok_and(|facts| States::of_the_tree(&facts.states).selected() == Some(true));
        let (Some(drop_down), Some(option)) = (&self.drop_down, self.to_choose.option()) else {
            return selected;
        };
        selected
            || tree.facts(drop_down).is_ok_and(|facts| {
                facts.name.trim() == option
                    || (facts.has_text
                        && tree.text(drop_down).is_ok_and(|text| text.trim() == option))
            })
    }

    /// What a person is told beyond the sentence they approved: that the
    /// application did not answer, or — where `seen` is false — that the
    /// window does not show it chosen. [`None`] when it shows it chosen.
    #[must_use]
    pub fn said(&self, seen: bool, strings: &Strings) -> Option<Said> {
        let word = if self.unanswered {
            words::DID_NOT_ANSWER
        } else if seen {
            return None;
        } else {
            words::NOT_SEEN_CHOSEN
        };
        let name = self.to_choose.name();
        Some(strings.say(
            &word.key(),
            &filling(&self.application, name, self.to_choose.option()),
        ))
    }

    /// Give back what ran, so it can be recorded.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
