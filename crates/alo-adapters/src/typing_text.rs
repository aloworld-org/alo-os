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
//!   are: exactly one showing field or document of the approved name (of several,
//!   the one with the keyboard, where exactly one has it) — never a
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
use crate::fallback_verbs::{FIELD, NAME, TEXT, TYPE_TEXT, TYPE_WITHOUT_SAVING};
use crate::fallback_words as fallback;
use crate::places_for_text::{
    FOCUSED, TypingRefuses, holds, the_ones_meant, the_places_for_text, what_is_there,
    why_typing_refuses,
};
use crate::typing_without_saving_words as unsaved;
use crate::typing_words as words;

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
    /// Whether the approval was to save after typing — `accessible.type_text`
    /// — or not — `accessible.type_without_saving`.
    saves: bool,
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
        // **Which of the two typings this is** is the verb's, read off the
        // authority itself: one that is neither is refused by `reached`.
        let saves = authorised.verb() != TYPE_WITHOUT_SAVING;
        let verb = if saves {
            TYPE_TEXT
        } else {
            TYPE_WITHOUT_SAVING
        };
        let reached = reached(authorised, verb, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        // **The field is `name` for `type_text` and `field` for typing that
        // saves nothing** (`crate::fallback_verbs::FIELD`).
        let name = match call.value(if saves { NAME } else { FIELD }) {
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
        let typing = Self {
            authorised: reached.authorised,
            application: reached.application,
            name,
            text,
            saves,
        };
        // **A terminal application is never typed into, whatever the person
        // allowed** (ADR 0111): letting alo use a terminal lets it open one,
        // read it and press its buttons, and what is typed into one runs. Asked
        // here, before any tree, as well as of the field's role, because a
        // terminal whose tree does not describe it as one would otherwise reach
        // the keyboard road.
        if alo_capability::is_a_persons_own(&typing.application) {
            return Err(typing.refused(words::A_TERMINAL, None, strings));
        }
        Ok(typing)
    }

    /// Whether the person approved saving after typing.
    #[must_use]
    pub const fn saves(&self) -> bool {
        self.saves
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
        let places = the_places_for_text(tree, &self.application).map_err(unreadable)?;
        let meant = the_ones_meant(&places, &self.name);
        let (at, role, states) = match meant.as_slice() {
            // **Said with what is there**, so a model that guessed the name can
            // name one that is (the owner, 2026-11-15, relayed): the places'
            // own names, and the window's title for those with none.
            [] => {
                let there = what_is_there(tree, &places);
                if there.is_empty() {
                    return Err(self.refused(words::NO_SUCH_FIELD, None, strings));
                }
                let quoted: Vec<String> = there.iter().map(|named| format!("“{named}”")).collect();
                return Err(self.refused(
                    words::NO_SUCH_FIELD_BUT_THESE,
                    Some((words::THERE, quoted.join(", "))),
                    strings,
                ));
            }
            [one] => (one.at.clone(), one.role, one.states.clone()),
            _ => return Err(self.refused(words::MORE_THAN_ONE_FIELD, None, strings)),
        };
        match why_typing_refuses(tree, &at, role, &states) {
            Some(TypingRefuses::APasswordField) => {
                return Err(self.refused(words::A_PASSWORD_FIELD, None, strings));
            }
            Some(TypingRefuses::ATerminal) => {
                return Err(self.refused(words::A_TERMINAL, None, strings));
            }
            Some(TypingRefuses::NotEditable) => {
                return Err(self.refused(words::NO_SUCH_FIELD, None, strings));
            }
            None => {}
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
            // Said without *so it was not saved* where nothing was to be saved.
            Ok(_) if self.saves => Err(self.refused(words::READ_BACK_DIFFERS, None, strings)),
            Ok(_) => Err(self.refused(unsaved::UNSAVED_READ_BACK_DIFFERS, None, strings)),
            Err(_) if self.saves => Err(self.refused(words::CANNOT_READ_BACK, None, strings)),
            Err(_) => Err(self.refused(unsaved::UNSAVED_CANNOT_READ_BACK, None, strings)),
        }
    }
}
