//! **Opening one thing's right-click menu**, for one approval —
//! `accessible.open_menu` (2026-11-17, the owner's *"Yes do all"*).
//!
//! A person right-clicks a file to rename, share or delete it; a menu like that
//! is often the only road to what it holds. This opens it **by the
//! application's own action for it**, never by a click at a place: the thing is
//! named exactly as the application shows it, must be on the screen, and must be
//! the only one of that name offering such an action. Once it is open, the
//! menu's items are read with `accessible.read_window` and pressed with
//! `accessible.activate_control`, as any menu is.
//!
//! **Which action opens a menu** is [`MENU_ACTIONS`]: Qt's `ShowMenu`, named by
//! the coordinator's reading of Qt for this verb. **That other toolkits offer
//! one, and under what name, is not measured**; a thing that offers none is
//! refused in words, and nothing is done.
//!
//! **Never a password field**, and never a thing that is not on the screen.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Filling, Said, Strings, Word};

use crate::accessibility_tree::{AccessibilityTree, NodeAt, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{OPEN_MENU, THING};
use crate::fallback_words as fallback;
use crate::menu_words as words;
use crate::not_walked::NotWalked;
use crate::role::Role;
use crate::states::States;
use crate::walking::DEEPEST;

/// The names of the actions that open a thing's own menu, matched without
/// regard to case.
pub const MENU_ACTIONS: [&str; 1] = ["showmenu"];

/// The most things asked about in one application's windows when looking for
/// the one named — as many as a press asks about.
pub const MOST_ASKED: usize = 20_000;

/// An approved opening of one menu, not yet done.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct OpeningAMenu {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// The thing's name, as approved.
    name: String,
}

/// One thing on the screen of the approved name.
struct Named {
    /// Where it is.
    at: NodeAt,
    /// Its role.
    role: Role,
}

impl OpeningAMenu {
    /// Ask everything that is asked before a window is looked at.
    ///
    /// # Errors
    /// [`Refused`]: another verb's authority, the application not granted, not
    /// installed, or with an adapter of its own.
    pub fn of(
        authorised: Authorised,
        adapters: &Adapters,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let reached = reached(authorised, OPEN_MENU, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let Some(Value::Name(name)) = call.value(THING) else {
            let said = strings.say(
                &fallback::NOT_THIS_VERB.key(),
                &Filling::of(fallback::VERB, call.verb().to_owned()),
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

    /// The thing's name, as approved.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// One of this file's sentences, filled in.
    fn say(&self, word: Word, strings: &Strings) -> Said {
        strings.say(
            &word.key(),
            &Filling::of(fallback::APPLICATION, self.application.clone())
                .and(fallback::NAME, self.name.clone()),
        )
    }

    /// This opening refused in these words.
    fn refused(&self, word: Word, strings: &Strings) -> Refused {
        Refused::worded_elsewhere(self.authorised.call().clone(), self.say(word, strings))
    }

    /// Find the thing now, and open its menu once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was asked of the application, in the words a
    /// person is told, and when it answered that it could not. An application
    /// that did not answer is **not** an error: it may have opened it, and
    /// [`MenuOpened::unanswered`] says so.
    pub fn open(
        self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<MenuOpened, Refused> {
        let walked = |fault| {
            Refused::worded_elsewhere(
                self.authorised.call().clone(),
                NotWalked::Tree(fault).said(&self.application, None, strings),
            )
        };
        let (named, not_all_read) = self.the_named(tree).map_err(walked)?;
        if named.is_empty() {
            let word = if not_all_read {
                words::TOO_MUCH
            } else {
                words::NOTHING_NAMED
            };
            return Err(self.refused(word, strings));
        }
        if named.iter().any(|thing| thing.role == Role::PasswordField) {
            return Err(self.refused(words::A_PASSWORD_FIELD, strings));
        }
        let mut with_a_menu = Vec::new();
        for thing in &named {
            let actions = tree.actions(&thing.at).map_err(walked)?;
            if let Some(index) = actions.iter().position(|action| {
                MENU_ACTIONS
                    .iter()
                    .any(|menu| action.eq_ignore_ascii_case(menu))
            }) {
                with_a_menu.push((thing.at.clone(), index));
            }
        }
        let (at, index) = match with_a_menu.as_slice() {
            [] => return Err(self.refused(words::NO_MENU, strings)),
            [_] if not_all_read => return Err(self.refused(words::TOO_MUCH, strings)),
            [one] => one.clone(),
            _ => return Err(self.refused(words::MORE_THAN_ONE, strings)),
        };
        let unanswered = match tree.act(&at, index) {
            Ok(true) => false,
            Ok(false) => return Err(self.refused(words::DID_NOT_OPEN, strings)),
            Err(TreeFault::DidNotAnswer) => true,
            Err(fault) => return Err(walked(fault)),
        };
        let told = unanswered.then(|| self.say(words::DID_NOT_ANSWER, strings));
        Ok(MenuOpened {
            authorised: self.authorised,
            application: self.application,
            name: self.name,
            unanswered,
            told,
        })
    }

    /// Every thing on the screen in the application's showing windows named
    /// exactly as approved, and whether the search stopped at its bound.
    fn the_named(&self, tree: &dyn AccessibilityTree) -> Result<(Vec<Named>, bool), TreeFault> {
        let roots: Vec<NodeAt> = tree
            .applications()?
            .into_iter()
            .filter(|running| running.is.as_deref() == Some(self.application.as_str()))
            .map(|running| running.at)
            .collect();
        let mut waiting: Vec<(NodeAt, usize)> = Vec::new();
        for root in roots {
            match tree.facts(&root) {
                Ok(facts) => waiting.extend(
                    facts
                        .children
                        .into_iter()
                        .filter(|child| child.holder == root.holder)
                        .map(|child| (child, 0)),
                ),
                Err(TreeFault::Vanished) => {}
                Err(fault) => return Err(fault),
            }
        }
        let mut named = Vec::new();
        let mut asked = 0_usize;
        let mut not_all_read = false;
        while let Some((at, depth)) = waiting.pop() {
            if asked >= MOST_ASKED {
                not_all_read = true;
                break;
            }
            asked += 1;
            let facts = match tree.facts(&at) {
                Ok(facts) => facts,
                Err(TreeFault::Vanished) => continue,
                Err(fault) => return Err(fault),
            };
            if !States::of_the_tree(&facts.states).showing() {
                continue;
            }
            not_all_read |= facts.unlisted_children;
            if depth > 0 && facts.name == self.name {
                named.push(Named {
                    at: at.clone(),
                    role: Role::of_the_tree(facts.role),
                });
            }
            if depth < DEEPEST {
                waiting.extend(
                    facts
                        .children
                        .into_iter()
                        .filter(|child| child.holder == at.holder)
                        .map(|child| (child, depth + 1)),
                );
            } else if !facts.children.is_empty() {
                not_all_read = true;
            }
        }
        Ok((named, not_all_read))
    }
}

/// A menu that was opened — or asked to be, of an application that did not
/// answer.
#[derive(Debug)]
pub struct MenuOpened {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// The thing's name.
    name: String,
    /// Whether the application did not answer.
    unanswered: bool,
    /// What a person is told beside the sentence they approved, if anything.
    told: Option<Said>,
}

impl MenuOpened {
    /// What ran.
    #[must_use]
    pub const fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The thing's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the application did not answer, so whether the menu is open is
    /// not known.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// What a person is told beyond the sentence they approved — only when
    /// the application did not answer.
    #[must_use]
    pub const fn said(&self) -> Option<&Said> {
        self.told.as_ref()
    }
}
