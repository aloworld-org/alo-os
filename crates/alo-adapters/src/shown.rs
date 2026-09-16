//! What an application's windows show, as the agent is handed it.
//!
//! The answer to `accessible.read_window`, for the turn that asked. It is
//! **not `Clone`, cannot be read back from anything, and holds no way back to
//! the application**: nothing in it says where a control is in the tree, so a
//! control the agent later asks to press is looked for again at that moment
//! (`crate::activating`) rather than found through something read earlier.
//!
//! # A password field's contents are not in it
//!
//! [`Seen::of`] is the only way to make a [`Seen`], and it gives a password
//! field [`Contents::Withheld`] **whatever text it is handed** — so the rule
//! holds here a second time, beside the walk that never asks
//! (`crate::walking`), and a mistake in one is not a password in the answer.
//!
//! # What could not be read is said
//!
//! A control with no name, an area the application draws itself and a part of
//! the window another program shows are each a [`Limit`], said in words. The
//! agent is told what it cannot read or press, never handed a position to try.

use alo_strings::{Said, Strings};

use crate::fallback_words as words;
use crate::role::Role;

/// What one thing contains.
#[derive(Debug, PartialEq, Eq)]
pub enum Contents {
    /// Nothing beyond its name.
    Nothing,
    /// The text it shows.
    Text(String),
    /// A password field: whatever is in it is never read.
    Withheld,
}

/// Something about one thing that could not be read or used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// It has no name, so it can be neither described nor pressed.
    NoName,
    /// The application draws it itself and describes nothing inside it.
    NotDescribed,
    /// Another program shows this part of the window, and it was not read.
    ShownByAnotherProgram,
}

impl Limit {
    /// What a person, or the agent, is told.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let word = match self {
            Self::NoName => words::LIMIT_NO_NAME,
            Self::NotDescribed => words::LIMIT_NOT_DESCRIBED,
            Self::ShownByAnotherProgram => words::LIMIT_ANOTHER_PROGRAM,
        };
        strings.say(&word.key(), &alo_strings::Filling::nothing())
    }
}

/// One thing a window shows.
#[derive(Debug, PartialEq, Eq)]
pub struct Seen {
    /// What it is.
    role: Role,
    /// What it is called — empty for a thing with no name.
    name: String,
    /// What it contains.
    contents: Contents,
    /// On or off, for a thing that is either.
    on: Option<bool>,
    /// Whether a person could use it now.
    usable: bool,
    /// How far inside its window it is.
    depth: usize,
    /// What could not be read about it.
    limit: Option<Limit>,
}

impl Seen {
    /// One thing, as the walk found it.
    ///
    /// **A password field's text is dropped here**, whatever was handed in.
    #[must_use]
    pub fn of(role: Role, name: String, text: Option<String>, depth: usize) -> Self {
        let contents = match (role, text) {
            (Role::PasswordField, _) => Contents::Withheld,
            (_, Some(text)) if !text.is_empty() && text != name => Contents::Text(text),
            (_, _) => Contents::Nothing,
        };
        Self {
            role,
            name,
            contents,
            on: None,
            usable: true,
            depth,
            limit: None,
        }
    }

    /// The same thing, on or off.
    #[must_use]
    pub fn on(mut self, on: Option<bool>) -> Self {
        self.on = on;
        self
    }

    /// The same thing, usable or not.
    #[must_use]
    pub fn usable(mut self, usable: bool) -> Self {
        self.usable = usable;
        self
    }

    /// The same thing, with what could not be read about it.
    #[must_use]
    pub fn limited(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// What it is.
    #[must_use]
    pub fn role(&self) -> Role {
        self.role
    }

    /// What it is called — empty for a thing with no name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it contains.
    #[must_use]
    pub fn contents(&self) -> &Contents {
        &self.contents
    }

    /// On or off, for a thing that is either.
    #[must_use]
    pub fn is_on(&self) -> Option<bool> {
        self.on
    }

    /// Whether a person could use it now.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.usable
    }

    /// How far inside its window it is.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// What could not be read about it.
    #[must_use]
    pub fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

/// One window, and what it shows.
#[derive(Debug, PartialEq, Eq)]
pub struct ShownWindow {
    /// Its title.
    title: String,
    /// What it shows, in reading order.
    seen: Vec<Seen>,
}

impl ShownWindow {
    /// A window with this title, showing these.
    #[must_use]
    pub fn titled(title: String, seen: Vec<Seen>) -> Self {
        Self { title, seen }
    }

    /// Its title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// What it shows, in reading order.
    #[must_use]
    pub fn seen(&self) -> &[Seen] {
        &self.seen
    }
}

/// What one application's windows show.
#[derive(Debug, PartialEq, Eq)]
pub struct Shown {
    /// The application.
    application: String,
    /// Its windows on the screen.
    windows: Vec<ShownWindow>,
    /// Whether it showed more than is read at once, so the rest was not read.
    not_all_read: bool,
}

impl Shown {
    /// What these windows of this application show.
    #[must_use]
    pub fn of(application: String, windows: Vec<ShownWindow>, not_all_read: bool) -> Self {
        Self {
            application,
            windows,
            not_all_read,
        }
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// Its windows on the screen.
    #[must_use]
    pub fn windows(&self) -> &[ShownWindow] {
        &self.windows
    }

    /// Every thing every window shows, in reading order.
    pub fn everything(&self) -> impl Iterator<Item = &Seen> {
        self.windows.iter().flat_map(|window| window.seen.iter())
    }

    /// Whether it showed more than is read at once, so the rest was not read.
    #[must_use]
    pub fn not_all_read(&self) -> bool {
        self.not_all_read
    }

    /// What the agent is told about what was not read, when anything was not.
    #[must_use]
    pub fn not_all_read_said(&self, strings: &Strings) -> Option<Said> {
        self.not_all_read.then(|| {
            strings.say(
                &words::NOT_ALL_READ.key(),
                &alo_strings::Filling::of(words::APPLICATION, self.application.clone()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_fields_text_is_dropped_whatever_is_handed_in() {
        let seen = Seen::of(
            Role::PasswordField,
            "Password".to_owned(),
            Some("hunter2".to_owned()),
            1,
        );
        assert_eq!(seen.contents(), &Contents::Withheld);
        assert!(!format!("{seen:?}").contains("hunter2"));
    }

    #[test]
    fn text_that_only_repeats_the_name_is_not_said_twice() {
        let label = Seen::of(
            Role::Label,
            "Account".to_owned(),
            Some("Account".to_owned()),
            0,
        );
        assert_eq!(label.contents(), &Contents::Nothing);
        let field = Seen::of(Role::TextField, String::new(), Some("anna".to_owned()), 0);
        assert_eq!(field.contents(), &Contents::Text("anna".to_owned()));
    }
}
