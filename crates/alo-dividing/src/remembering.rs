//! A division remembered, so that coming back to a pair of windows gives back
//! **the division they were in** rather than where each window last sat.
//!
//! # What is remembered, and what is deliberately not
//!
//! **Applications and shares.** Never a window's title, never a document's
//! name, never a path — [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md).
//! A person who divides their mail against their editor and comes back
//! tomorrow wants those two applications in those two shares; what was open in
//! them is not this crate's business and is not written to anybody's disk.
//!
//! A [`WindowId`] cannot be what is remembered either, and that is the whole
//! reason this module exists. It is what the compositor calls a window **this
//! time**; close the window and it is gone, reopen it and the number is
//! different. Remembering it would remember nothing.
//!
//! # The tree, not the rectangles
//!
//! A division is a tree of cuts, and that is what is kept — each cut's
//! direction and where it falls, with each leaf naming an application. Keeping
//! the rectangles instead would restore four windows to four positions that
//! happened to tile, and the first time a display came back at another size
//! they would overlap or leave a gap. **A cut re-cut still divides; a rectangle
//! remembered does not.**
//!
//! # This crate does not know how an application is named
//!
//! [`HeldBy`] is a name handed in, exactly as far as this crate is concerned:
//! it checks that it is a usable key and never what it means. Which string
//! names an application is `alo-applications`' decision, and the desktop plan
//! does not read that crate — so the shell hands the name down, the way
//! `alo-playing` is handed a machine and `alo-access` is handed a folder.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::area::Area;
use crate::division::Division;
use crate::node::Node;
use crate::side::Axis;
use crate::window::{Window, WindowId};

/// The longest an application's name may be as a key here.
///
/// Long enough for a reverse-domain identifier with room to spare, short enough
/// that a settings file cannot be filled by one.
pub const LONGEST_NAME: usize = 255;

/// Why a name cannot be remembered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRemembered {
    /// The name is empty, so it names nothing.
    Unnamed,
    /// The name has a space at one end, which reads as the same name and is
    /// not — the fault `alo-displays` refuses in a screen's name, refused here
    /// for the same reason.
    Spaced(String),
    /// The name is longer than [`LONGEST_NAME`].
    TooLong(usize),
    /// The name holds a character that cannot be written to a settings file and
    /// read back as itself.
    Unwritable(String),
}

/// Which application held a share.
///
/// A name this crate is handed and does not interpret.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HeldBy(String);

impl HeldBy {
    /// This application's name, checked as a key and not as a meaning.
    ///
    /// # Errors
    /// [`NotRemembered`] for a name that is empty, spaced at either end, longer
    /// than [`LONGEST_NAME`], or holding a control character.
    pub fn named(name: &str) -> Result<Self, NotRemembered> {
        if name.is_empty() {
            return Err(NotRemembered::Unnamed);
        }
        if name.trim() != name {
            return Err(NotRemembered::Spaced(name.to_owned()));
        }
        if name.len() > LONGEST_NAME {
            return Err(NotRemembered::TooLong(name.len()));
        }
        if name.chars().any(char::is_control) {
            return Err(NotRemembered::Unwritable(name.to_owned()));
        }
        Ok(Self(name.to_owned()))
    }

    /// The name, as it was handed over.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A division as it is remembered: the same tree of cuts, with an application
/// where each window was.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Remembered {
    /// One application holding this whole piece.
    HeldBy(HeldBy),
    /// A cut, and what is on either side of it.
    Cut {
        /// Which way the cut runs.
        axis: Axis,
        /// How far along the first side reaches.
        first_length: u32,
        /// What is on the near side.
        first: Box<Remembered>,
        /// What is on the far side.
        second: Box<Remembered>,
    },
}

impl Remembered {
    /// This division, remembered — or nothing, where no window in it belongs to
    /// an application this machine could name.
    ///
    /// `who` is asked which application holds a window. A window it cannot name
    /// is left out, and the cut it was in collapses the way closing it would —
    /// so a division holding one nameless window is remembered as the rest of
    /// itself rather than not at all.
    #[must_use]
    pub fn of(division: &Division, who: &impl Fn(WindowId) -> Option<HeldBy>) -> Option<Self> {
        Self::of_node(division.tree()?, who)
    }

    /// The same, for one piece of the tree.
    fn of_node(node: &Node, who: &impl Fn(WindowId) -> Option<HeldBy>) -> Option<Self> {
        match node {
            Node::Share(window) => who(window.id()).map(Self::HeldBy),
            Node::Cut(cut) => {
                let first = Self::of_node(&cut.first, who);
                let second = Self::of_node(&cut.second, who);
                match (first, second) {
                    (Some(first), Some(second)) => Some(Self::Cut {
                        axis: cut.axis,
                        first_length: cut.first_length,
                        first: Box::new(first),
                        second: Box::new(second),
                    }),
                    // One side named and the other not: the cut collapses onto
                    // the side that is left, as closing the other would.
                    (Some(only), None) | (None, Some(only)) => Some(only),
                    (None, None) => None,
                }
            }
        }
    }

    /// Every application this division was held by, in the order the cuts run.
    #[must_use]
    pub fn held_by(&self) -> Vec<&HeldBy> {
        let mut found = Vec::new();
        self.each(&mut |held| found.push(held));
        found
    }

    /// Walk the applications in order.
    fn each<'a>(&'a self, seen: &mut impl FnMut(&'a HeldBy)) {
        match self {
            Self::HeldBy(held) => seen(held),
            Self::Cut { first, second, .. } => {
                first.each(seen);
                second.each(seen);
            }
        }
    }

    /// This division given back, with the windows that are open now.
    ///
    /// `open` is asked for a window of an application. An application that is
    /// not open is left out and its cut collapses onto what is left, so
    /// returning with one of two applications running gives that one the whole
    /// display rather than half of it and a hole.
    ///
    /// Nothing is returned when none of the applications is open.
    #[must_use]
    pub fn restored(
        &self,
        display: Area,
        open: &impl Fn(&HeldBy) -> Option<Window>,
    ) -> Option<Division> {
        let tree = self.restored_node(open)?;
        let mut division = Division::of(display);
        division.replace(tree);
        Some(division)
    }

    /// The same, for one piece.
    fn restored_node(&self, open: &impl Fn(&HeldBy) -> Option<Window>) -> Option<Node> {
        match self {
            Self::HeldBy(held) => open(held).map(Node::Share),
            Self::Cut {
                axis,
                first_length,
                first,
                second,
            } => {
                let first = first.restored_node(open);
                let second = second.restored_node(open);
                match (first, second) {
                    (Some(first), Some(second)) => {
                        Some(Node::cut(*axis, *first_length, first, second))
                    }
                    (Some(only), None) | (None, Some(only)) => Some(only),
                    (None, None) => None,
                }
            }
        }
    }
}

/// What one display's division was.
///
/// **The display's own size is not kept.** [`Remembered::restored`] is handed
/// the display as it is *now*, which is the only size that can matter: a screen
/// that comes back at another resolution still divides, because what was kept
/// is the cuts and not the rectangles. Keeping yesterday's size would be a
/// field nothing reads and a second answer to a question already answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OnADisplay {
    /// The division.
    pub division: Remembered,
}

/// Every display's remembered division, by the screen it belongs to.
///
/// **A display that is unplugged keeps its division.** Nothing here removes one
/// because a screen went away: the whole point is that plugging it back in
/// gives the person their arrangement, and a machine that forgot a screen the
/// moment it was unplugged would be a machine that never remembered anything a
/// person did at a desk they leave.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Divisions {
    /// Keyed by [`crate::screen::Screen`]'s written form.
    #[serde(default)]
    on: BTreeMap<String, OnADisplay>,
}

impl Divisions {
    /// Nothing remembered yet.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Remember this division on this screen, in place of whatever was
    /// remembered for it before.
    pub fn remember(&mut self, screen: &str, division: Remembered) {
        self.on.insert(screen.to_owned(), OnADisplay { division });
    }

    /// What was remembered for this screen, if anything.
    #[must_use]
    pub fn on(&self, screen: &str) -> Option<&OnADisplay> {
        self.on.get(screen)
    }

    /// Forget this screen's division — for a person who asks, never for a
    /// screen that was merely unplugged.
    pub fn forget(&mut self, screen: &str) {
        self.on.remove(screen);
    }

    /// How many screens have a division remembered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.on.len()
    }

    /// Whether none has.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.on.is_empty()
    }

    /// Every screen remembered, in a settled order.
    pub fn screens(&self) -> impl Iterator<Item = &str> {
        self.on.keys().map(String::as_str)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::area::{Point, Size};

    /// An application, named.
    fn app(name: &str) -> HeldBy {
        HeldBy::named(name).expect("a usable name")
    }

    /// A display of a usual size.
    fn display() -> Area {
        Area::of(Point::at(0, 0), Size::of(1920, 1080)).expect("a display")
    }

    /// A window with no minimum.
    fn window(id: u64) -> Window {
        Window::any_size(WindowId::from_compositor(id))
    }

    /// **A name is checked as a key and refused when it cannot be one.**
    #[test]
    fn a_name_is_checked_as_a_key() {
        assert!(HeldBy::named("org.example.Mail").is_ok());
        assert_eq!(HeldBy::named(""), Err(NotRemembered::Unnamed));
        assert!(matches!(
            HeldBy::named(" mail"),
            Err(NotRemembered::Spaced(_))
        ));
        assert!(matches!(
            HeldBy::named("mail "),
            Err(NotRemembered::Spaced(_))
        ));
        assert!(matches!(
            HeldBy::named(&"x".repeat(LONGEST_NAME + 1)),
            Err(NotRemembered::TooLong(_))
        ));
        assert!(matches!(
            HeldBy::named("ma\nil"),
            Err(NotRemembered::Unwritable(_))
        ));
    }

    /// **An unplugged screen keeps its division.**
    ///
    /// There is no road here that forgets one for a screen going away — only a
    /// person asking.
    #[test]
    fn a_screen_that_goes_away_keeps_what_it_had() {
        let mut divisions = Divisions::none();
        divisions.remember(
            "panel:ACME/A1/SER",
            Remembered::HeldBy(app("org.example.Mail")),
        );
        assert_eq!(divisions.len(), 1);

        // Whatever a session does when a screen is unplugged, it is not this.
        assert!(divisions.on("panel:ACME/A1/SER").is_some());
        divisions.forget("panel:ACME/A1/SER");
        assert!(divisions.on("panel:ACME/A1/SER").is_none());
    }

    /// **Two screens divide independently.**
    #[test]
    fn each_screen_is_remembered_on_its_own() {
        let mut divisions = Divisions::none();
        divisions.remember(
            "panel:ACME/A1/SER",
            Remembered::HeldBy(app("org.example.Mail")),
        );
        divisions.remember(
            "socket:HDMI-1",
            Remembered::HeldBy(app("org.example.Editor")),
        );
        assert_eq!(divisions.len(), 2);
        assert_eq!(
            divisions
                .on("panel:ACME/A1/SER")
                .expect("the laptop's own screen")
                .division
                .held_by(),
            vec![&app("org.example.Mail")]
        );
        assert_eq!(
            divisions
                .on("socket:HDMI-1")
                .expect("the external screen")
                .division
                .held_by(),
            vec![&app("org.example.Editor")]
        );
        assert_eq!(
            divisions.screens().collect::<Vec<_>>(),
            ["panel:ACME/A1/SER", "socket:HDMI-1"]
        );
    }

    /// **A division is remembered as a tree of cuts and given back as one.**
    #[test]
    fn a_cut_is_remembered_and_given_back() {
        let remembered = Remembered::Cut {
            axis: Axis::SideBySide,
            first_length: 960,
            first: Box::new(Remembered::HeldBy(app("org.example.Mail"))),
            second: Box::new(Remembered::HeldBy(app("org.example.Editor"))),
        };
        assert_eq!(
            remembered.held_by(),
            vec![&app("org.example.Mail"), &app("org.example.Editor")]
        );

        let mail = window(7);
        let editor = window(9);
        let division = remembered
            .restored(display(), &|held: &HeldBy| match held.as_str() {
                "org.example.Mail" => Some(mail),
                "org.example.Editor" => Some(editor),
                _ => None,
            })
            .expect("both applications are open");
        assert!(division.holds(mail.id()));
        assert!(division.holds(editor.id()));
        assert_eq!(division.shares().len(), 2);
    }

    /// **One application open of two gives that one the whole display**, rather
    /// than half a division and a hole.
    #[test]
    fn a_division_with_one_application_missing_collapses_onto_the_other() {
        let remembered = Remembered::Cut {
            axis: Axis::SideBySide,
            first_length: 960,
            first: Box::new(Remembered::HeldBy(app("org.example.Mail"))),
            second: Box::new(Remembered::HeldBy(app("org.example.Editor"))),
        };
        let editor = window(9);
        let division = remembered
            .restored(display(), &|held: &HeldBy| {
                (held.as_str() == "org.example.Editor").then_some(editor)
            })
            .expect("one application is open");
        assert!(division.holds(editor.id()));
        assert_eq!(division.shares().len(), 1);
        assert_eq!(
            division.share_of(editor.id()),
            Some(display()),
            "the one that is open takes the whole display"
        );
    }

    /// **Nothing open gives nothing back**, rather than an empty division.
    #[test]
    fn nothing_open_restores_nothing() {
        let remembered = Remembered::HeldBy(app("org.example.Mail"));
        assert!(remembered.restored(display(), &|_: &HeldBy| None).is_none());
    }

    /// **What is written holds applications and shares and nothing else.**
    ///
    /// The clause of ADR 0038 this module exists under, checked against the
    /// bytes rather than trusted: a window's number never reaches the file.
    #[test]
    fn what_is_written_names_applications_and_never_a_window() {
        let mut divisions = Divisions::none();
        divisions.remember(
            "panel:ACME/A1/SER",
            Remembered::Cut {
                axis: Axis::SideBySide,
                first_length: 960,
                first: Box::new(Remembered::HeldBy(app("org.example.Mail"))),
                second: Box::new(Remembered::HeldBy(app("org.example.Editor"))),
            },
        );
        let written = toml::to_string(&divisions).expect("divisions are written");
        assert!(written.contains("org.example.Mail"), "{written}");
        assert!(written.contains("org.example.Editor"), "{written}");
        assert!(!written.to_lowercase().contains("window"), "{written}");

        let read: Divisions = toml::from_str(&written).expect("and read back");
        assert_eq!(read, divisions);
    }
}
