//! What was open, which is a list of applications and where each one was —
//! and **nothing about what was in them**.
//!
//! This is the file in this crate that a reviewer should read first, because
//! reopening what was open is the one promise in this plan that could turn into
//! a diary. On most systems the list that restores a session carries window
//! titles, and a window title is the name of the document somebody was writing,
//! the subject of the mail they were reading, or the address of the page they
//! were on. Written into a file, that is a record of what a person did
//! yesterday, sitting in their own folder — and on a machine with an agent on
//! it, it is context nobody offered (ADR 0001: context reaches an agent at the
//! moment of invocation and is never harvested).
//!
//! So an [`Open`] has three fields and no fourth:
//!
//! | | |
//! |---|---|
//! | the application | its identifier — `org.example.Editor` — and never the name a packager wrote |
//! | the screen | the name the shell knows it by, which is `alo-appearance`'s and `alo-displays`' |
//! | the split | the whole screen, or a half, a quarter or a part ([`crate::Split`]) |
//!
//! There is no field for a title, a document, a path, a URL, a window
//! identifier or a size, and no generic parameter one could arrive through.
//! `tests/what_was_open_is_applications_and_places.rs` reads this crate's own
//! source for every one of those words rather than trusting this paragraph.
//!
//! # Why the identifier and not the name
//!
//! `alo_applications::Application` carries both, and says why only the
//! identifier is ever approved: the name is written by whoever packaged the
//! application and two of them can claim the same one. The same argument
//! decides what is kept — a file that named *Mail* would be a file that reopens
//! whichever *Mail* happens to answer to it next — so what goes into the
//! person's folder is the identifier, which is also what a grant is made over.
//!
//! # One entry per window, and no window identifier
//!
//! Two windows of one application on two screens are two entries, because *and
//! reopen what was open* is a promise about what was on which screen. What they
//! are **not** is two named windows: nothing here can tell them apart, and
//! nothing here tries. A compositor's window identifier is meaningless at the
//! next sign-in anyway, and writing one down would be writing down that there
//! were two particular windows rather than two places.

use alo_appearance::DisplayId;
use alo_applications::{Application, NotAnApplication};
use serde::{Deserialize, Serialize};

use crate::split::Split;

/// One window that was open: which application, on which screen, in which part
/// of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Open {
    /// The application, by the identifier this machine knows it by.
    what: Application,
    /// The screen it was on, by the name the shell knows that screen by.
    on: DisplayId,
    /// What it had of that screen.
    split: Split,
}

impl Open {
    /// One window that was open.
    ///
    /// The application arrives as an identifier rather than as an
    /// [`Application`], so that the name a packager wrote has no road into a
    /// person's folder even by accident. A caller holding an `Application`
    /// passes its [`Application::identifier`].
    ///
    /// # Errors
    /// [`NotAnApplication`] for an identifier no verb and no grant could ever
    /// name, which is `alo-applications`' rule and not a second one.
    pub fn of(identifier: &str, on: DisplayId, split: Split) -> Result<Self, NotAnApplication> {
        Ok(Self {
            what: Application::identified(identifier)?,
            on,
            split,
        })
    }

    /// The application, by the identifier this machine knows it by.
    #[must_use]
    pub const fn what(&self) -> &Application {
        &self.what
    }

    /// The screen it was on.
    #[must_use]
    pub const fn on(&self) -> &DisplayId {
        &self.on
    }

    /// What it had of that screen.
    #[must_use]
    pub const fn split(&self) -> Split {
        self.split
    }
}

/// Everything that was open, oldest first.
///
/// The order is the order the windows were opened, and it is kept for two
/// reasons: it is the order things are reopened in, and it is the order a
/// log-out unwinds ([`WasOpen::asked_to_close_in_order`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WasOpen {
    /// The windows, oldest first.
    windows: Vec<Open>,
}

impl WasOpen {
    /// Nothing was open.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    /// This, and then that — the order being the order they were opened.
    #[must_use]
    pub fn and(mut self, open: Open) -> Self {
        self.windows.push(open);
        self
    }

    /// Whether nothing was open at all.
    #[must_use]
    pub const fn is_nothing(&self) -> bool {
        self.windows.is_empty()
    }

    /// Every window, oldest first.
    #[must_use]
    pub fn windows(&self) -> &[Open] {
        &self.windows
    }

    /// The applications, each once, in the order they were first opened.
    #[must_use]
    pub fn applications(&self) -> Vec<&Application> {
        let mut each: Vec<&Application> = Vec::new();
        for window in &self.windows {
            if !each.contains(&window.what()) {
                each.push(window.what());
            }
        }
        each
    }

    /// The applications in the order a log-out asks them to close: the last one
    /// opened is asked first.
    ///
    /// **A stack unwinds.** An application that was opened from another — a
    /// document from a file manager, a mail attachment from a mail client — is
    /// asked while the one it came from is still there, which is the order in
    /// which the second one still has somewhere to say *I could not save this*.
    /// It is also the order a person expects: the thing they were just in is
    /// the thing they are asked about first, rather than after nine dialogues
    /// about applications they had forgotten were running.
    #[must_use]
    pub fn asked_to_close_in_order(&self) -> Vec<&Application> {
        let mut each = self.applications();
        each.reverse();
        each
    }

    /// This list, from what a file held.
    pub(crate) fn of(windows: Vec<Open>) -> Self {
        Self { windows }
    }

    /// The windows, given up — for this crate's own keeping.
    pub(crate) fn into_windows(self) -> Vec<Open> {
        self.windows
    }
}

/// One open window as `leaving.toml` holds it.
///
/// `deny_unknown_fields`, which is the clause that matters: a `title`, a `url`
/// or a `document` in a person's own file — put there by hand, or by some later
/// release that forgot why this type is three fields wide — refuses the whole
/// file rather than being read past.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Written {
    /// The application's identifier.
    application: String,
    /// The name of the screen it was on.
    on: String,
    /// What it had of that screen.
    split: Split,
}

impl TryFrom<Written> for Open {
    type Error = String;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        let on = DisplayId::named(&written.on).map_err(|_| {
            format!(
                "`{}` is not a name a screen is known by",
                written.on.escape_debug()
            )
        })?;
        Self::of(&written.application, on, written.split).map_err(|_| {
            format!(
                "`{}` is not an application's identifier",
                written.application.escape_debug()
            )
        })
    }
}

impl From<Open> for Written {
    fn from(open: Open) -> Self {
        Self {
            application: open.what.identifier().to_owned(),
            on: open.on.name().to_owned(),
            split: open.split,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_dividing::Place;

    use super::*;
    use crate::testing::{the_editor, the_laptop, the_screen};

    /// **An open window is an application, a screen and a split**, and what it
    /// keeps of the application is the identifier alone.
    #[test]
    fn an_open_window_is_an_application_a_screen_and_a_split() {
        let open = Open::of(the_editor(), the_laptop(), Split::Of(Place::LeftHalf)).unwrap();
        assert_eq!(open.what().identifier(), the_editor());
        assert_eq!(open.what().name(), None);
        assert_eq!(open.on(), &the_laptop());
        assert_eq!(open.split(), Split::Of(Place::LeftHalf));
    }

    /// **An identifier no grant could name is refused**, in
    /// `alo-applications`' words rather than a second opinion.
    #[test]
    fn an_identifier_no_grant_could_name_is_refused() {
        for identifier in ["", "   ", "org.example.One Two", "org.example\nTwo"] {
            assert!(
                Open::of(identifier, the_laptop(), Split::TheWholeScreen).is_err(),
                "{identifier:?}"
            );
        }
    }

    /// **Applications are asked to close last first, each once**, however many
    /// windows they had and whichever screens those were on.
    #[test]
    fn applications_are_asked_last_first_and_once_each() {
        let was_open = WasOpen::nothing()
            .and(Open::of(the_editor(), the_laptop(), Split::TheWholeScreen).unwrap())
            .and(Open::of("org.example.Mail", the_screen(), Split::TheWholeScreen).unwrap())
            .and(Open::of(the_editor(), the_screen(), Split::Of(Place::RightHalf)).unwrap());

        let opened: Vec<&str> = was_open
            .applications()
            .iter()
            .map(|it| it.identifier())
            .collect();
        assert_eq!(opened, vec![the_editor(), "org.example.Mail"]);

        let asked: Vec<&str> = was_open
            .asked_to_close_in_order()
            .iter()
            .map(|it| it.identifier())
            .collect();
        assert_eq!(asked, vec!["org.example.Mail", the_editor()]);
        assert_eq!(was_open.windows().len(), 3);
        assert!(WasOpen::nothing().is_nothing());
    }
}
