//! **Typing one approved text into one field of an application**, for one
//! approval — `accessible.type_text`,
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md).
//!
//! This holds what is decided through the accessibility tree; delivering keys
//! is the compositor's, through the application door, and the order around
//! both is `alo_turn`'s.
//!
//! - [`Typing::of`] asks everything the fallback asks before the tree is
//!   touched: the grants at this moment, the application installed, no
//!   adapter of its own.
//! - [`Typing::the_field`] looks for the field **then**, in the windows as they
//!   are: exactly one showing field or document of the approved name — never a
//!   password field, never a terminal — and says whether it has the keyboard
//!   and whether it takes text through the tree.
//! - [`Typing::put_in`] is the direct road: the text inserted at the field's
//!   caret through `EditableText`.
//! - [`Typing::read_back`] reads the field again; the text must be in it
//!   exactly, or nothing is saved.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Refused, Value};
use alo_strings::{Filling, Strings, Word};

use crate::accessibility_tree::{AccessibilityTree, NodeAt, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{NAME, TEXT, TYPE_TEXT};
use crate::fallback_words as fallback;
use crate::role::Role;
use crate::typing_words as words;
use crate::walking::{DEEPEST, MOST_THINGS};

/// `ATSPI_STATE_EDITABLE`.
const EDITABLE: u32 = 7;

/// `ATSPI_STATE_FOCUSED`.
const FOCUSED: u32 = 12;

/// `ATSPI_STATE_SHOWING`.
const SHOWING: u32 = 25;

/// `ATSPI_ROLE_TERMINAL`.
const TERMINAL: u32 = 60;

/// The class Chromium gives the field a web terminal (xterm.js) types
/// through — VSCodium's built-in terminal, measured 2026-11-13.
const A_WEB_TERMINAL: &str = "xterm-helper-textarea";

/// Whether the tree's two words of states hold this one.
fn holds(states: &[u32], state: u32) -> bool {
    let word = usize::try_from(state / 32).unwrap_or(usize::MAX);
    states
        .get(word)
        .is_some_and(|bits| bits & (1_u32 << (state % 32)) != 0)
}

/// The field the approved text goes into, found at the moment of typing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheField {
    /// Where it is.
    pub at: NodeAt,
    /// Whether it has the keyboard now.
    pub has_the_keyboard: bool,
    /// Whether it takes text through the tree (`EditableText`).
    pub takes_text: bool,
}

/// An approved typing of one text into one field, not yet done.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct Typing {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// The field's name, as approved.
    name: String,
    /// The text, whole, as approved.
    text: String,
}

impl Typing {
    /// Ask everything that is asked before a field is looked for.
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
        let reached = reached(authorised, TYPE_TEXT, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let name = match call.value(NAME) {
            Some(Value::Name(name)) => Some(name.clone()),
            _ => None,
        };
        let text = match call.value(TEXT) {
            Some(Value::Text(text)) => Some(text.clone()),
            _ => None,
        };
        let (Some(name), Some(text)) = (name, text) else {
            let said = strings.say(
                &fallback::NOT_THIS_VERB.key(),
                &Filling::of(fallback::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            name,
            text,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// The field's name, as approved.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The text, whole, as approved.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// What ran, once it has.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }

    /// This typing refused in these words, with the application, the field
    /// and `more` filled in.
    #[must_use]
    pub fn refused(&self, word: Word, more: Option<(&str, String)>, strings: &Strings) -> Refused {
        let mut filling = Filling::of(fallback::APPLICATION, self.application.clone())
            .and(fallback::NAME, self.name.clone());
        if let Some((placeholder, value)) = more {
            filling = filling.and(placeholder, value);
        }
        Refused::worded_elsewhere(
            self.authorised.call().clone(),
            strings.say(&word.key(), &filling),
        )
    }

    /// Find the field now.
    ///
    /// # Errors
    /// [`Refused`] in the words a person is told: no field of that name
    /// showing, more than one, a password field, a terminal, or a tree that
    /// could not be read.
    pub fn the_field(
        &self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<TheField, Refused> {
        let unreadable = |_: TreeFault| self.refused(words::NO_SUCH_FIELD, None, strings);
        let roots: Vec<NodeAt> = tree
            .applications()
            .map_err(unreadable)?
            .into_iter()
            .filter(|running| running.is.as_deref() == Some(self.application.as_str()))
            .map(|running| running.at)
            .collect();
        let mut found = Vec::new();
        // Each thing waiting, with how deep it is and the title of the window
        // it is in.
        let mut waiting: Vec<(NodeAt, usize, String)> =
            roots.into_iter().map(|at| (at, 0, String::new())).collect();
        let mut seen = 0_usize;
        while let Some((at, depth, window)) = waiting.pop() {
            seen += 1;
            if seen > MOST_THINGS {
                break;
            }
            let facts = match tree.facts(&at) {
                Ok(facts) => facts,
                Err(TreeFault::Vanished) => continue,
                Err(fault) => return Err(unreadable(fault)),
            };
            let role = Role::of_the_tree(facts.role);
            let a_place_for_text =
                matches!(role, Role::TextField | Role::Document | Role::PasswordField)
                    || facts.role == TERMINAL;
            // **Named as approved**, or — where an application names its
            // document nothing, as GNOME Text Editor does — an unnamed one in
            // a window whose title holds the approved name. Which of several
            // unnamed ones is never guessed: two are refused below.
            let named = facts.name == self.name
                || (facts.name.is_empty() && !window.is_empty() && window.contains(&self.name));
            if a_place_for_text && named && holds(&facts.states, SHOWING) {
                found.push((at.clone(), facts.role, facts.states.clone()));
            }
            let window = if role == Role::Window {
                facts.name.clone()
            } else {
                window
            };
            if depth < DEEPEST {
                waiting.extend(
                    facts
                        .children
                        .into_iter()
                        .map(|child| (child, depth + 1, window.clone())),
                );
            }
        }
        let (at, role, states) = match found.len() {
            0 => return Err(self.refused(words::NO_SUCH_FIELD, None, strings)),
            1 => found.remove(0),
            _ => return Err(self.refused(words::MORE_THAN_ONE_FIELD, None, strings)),
        };
        if Role::of_the_tree(role) == Role::PasswordField {
            return Err(self.refused(words::A_PASSWORD_FIELD, None, strings));
        }
        let a_web_terminal = tree.attributes(&at).is_ok_and(|said| {
            said.iter()
                .any(|(name, value)| name == "class" && value.contains(A_WEB_TERMINAL))
        });
        if role == TERMINAL || a_web_terminal {
            return Err(self.refused(words::A_TERMINAL, None, strings));
        }
        if !holds(&states, EDITABLE) {
            return Err(self.refused(words::NO_SUCH_FIELD, None, strings));
        }
        Ok(TheField {
            has_the_keyboard: holds(&states, FOCUSED),
            takes_text: tree.takes_text(&at).unwrap_or(false),
            at,
        })
    }

    /// The direct road: the text inserted at the field's caret, through the
    /// tree — or at its end, where it says no caret.
    ///
    /// # Errors
    /// [`Refused`] when the application answers that it did not take it, or
    /// the field went.
    pub fn put_in(
        &self,
        tree: &dyn AccessibilityTree,
        field: &TheField,
        strings: &Strings,
    ) -> Result<(), Refused> {
        let did_not = || self.refused(words::DID_NOT_TAKE, None, strings);
        let at = match tree.caret(&field.at) {
            Ok(Some(caret)) => caret,
            Ok(None) => tree
                .text(&field.at)
                .map(|text| text.chars().count())
                .map_err(|_| did_not())?,
            Err(_) => return Err(did_not()),
        };
        match tree.insert_text(&field.at, at, &self.text) {
            Ok(true) => Ok(()),
            _ => Err(did_not()),
        }
    }

    /// Read the field again: the approved text must be in it, exactly.
    ///
    /// # Errors
    /// [`Refused`]: the text reads back differently, or cannot be read.
    pub fn read_back(
        &self,
        tree: &dyn AccessibilityTree,
        field: &TheField,
        strings: &Strings,
    ) -> Result<(), Refused> {
        match tree.text(&field.at) {
            Ok(shown) if shown.contains(&self.text) => Ok(()),
            Ok(_) => Err(self.refused(words::READ_BACK_DIFFERS, None, strings)),
            Err(_) => Err(self.refused(words::CANNOT_READ_BACK, None, strings)),
        }
    }
}
