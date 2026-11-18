//! **Scrolling in one application's window**, for one approval —
//! `accessible.scroll_to` and `accessible.scroll_page` (2026-11-17).
//!
//! A read of a window answers only what is on the screen (`crate::walking`), so
//! a long document, a long list or a form below its fold had no way in. These
//! are the way in, and neither finds anything by where it is:
//!
//! - **to a named thing**: the one thing in the application's windows of
//!   exactly that name — on the screen or scrolled away, but never hidden — is
//!   asked to be scrolled until it is on the screen, its top against the top;
//! - **by a page**: the first thing below what is on the screen is brought to
//!   the top (down), or the last thing above it to the bottom (up). Where one
//!   document is showing, the page is that document's.
//!
//! Each asks the application one thing, its own *scroll this onto the screen*
//! ([`AccessibilityTree::scroll_to`]), once; then asks the thing's states
//! again, so that *it said yes* and *it is on the screen now* are told apart.
//!
//! **Never a password field**: one is not scrolled to by name, and a page
//! passes over one rather than choosing it.
//!
//! **What tells a thing is scrolled away** is the tree's two states: *visible*
//! without *showing*. That list views describe their scrolled-away rows so was
//! found on 2026-11-16 (#1337); that every toolkit does is not measured, and a
//! toolkit that calls every row showing leaves a page with nothing further to
//! scroll to — said in words, and nothing is scrolled.

use alo_applications::Installed;
use alo_capability::{Authorised, Grants, Offered, Refused, Takes, Value};
use alo_strings::{Filling, Said, Strings, Word};

use crate::accessibility_tree::{AccessibilityTree, Edge, NodeAt, TreeFault};
use crate::adapters::Adapters;
use crate::fallback_reach::reached;
use crate::fallback_verbs::{SCROLL_PAGE, SCROLL_TO, TO, TOWARD};
use crate::fallback_words as fallback;
use crate::not_walked::NotWalked;
use crate::places_for_text::holds;
use crate::role::Role;
use crate::scrolling_words as words;
use crate::states::States;
use crate::walking::DEEPEST;

/// The most things asked about in one application's windows when looking for
/// what to scroll to — as many as a press asks about (#1337's `MOST_ASKED`).
pub const MOST_ASKED: usize = 20_000;

/// `ATSPI_STATE_VISIBLE`: the thing is not hidden, whether or not it is
/// scrolled onto the screen.
const VISIBLE: u32 = 30;

/// Which way a page is scrolled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toward {
    /// Further down.
    Down,
    /// Further up.
    Up,
}

impl Toward {
    /// Both, in the order they are offered.
    pub const ALL: [Self; 2] = [Self::Down, Self::Up];

    /// The name a model sends, and the record keeps.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
        }
    }

    /// The direction whose name a model sent, if it is one of these.
    #[must_use]
    pub fn called(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|toward| toward.named() == name)
    }

    /// What a person reads for it.
    #[must_use]
    pub const fn words(self) -> Word {
        match self {
            Self::Down => words::DOWN,
            Self::Up => words::UP,
        }
    }

    /// The choice the verb offers.
    #[must_use]
    pub fn offered() -> Takes {
        Takes::choice(
            Self::ALL
                .into_iter()
                .map(|toward| Offered::called(toward.named(), toward.words())),
        )
    }
}

/// What was asked: a thing by its name, or a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// The thing of this name, onto the screen.
    To(String),
    /// One page, this way.
    APage(Toward),
}

/// An approved scroll that has not happened yet.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it.
#[derive(Debug)]
pub struct Scrolling {
    /// What may run.
    authorised: Authorised,
    /// The application, as installed.
    application: String,
    /// What was asked.
    asked: Asked,
}

/// One thing the search read, in reading order.
#[derive(Debug)]
struct Place {
    /// Where it is.
    at: NodeAt,
    /// Its name.
    name: String,
    /// Its role.
    role: Role,
    /// Whether it is on the screen now.
    showing: bool,
    /// How deep it is inside its window.
    depth: usize,
}

/// Every not-hidden thing in an application's showing windows, in reading
/// order, and whether more were there than were asked about.
struct Read {
    /// The things.
    places: Vec<Place>,
    /// Whether the search stopped at a bound.
    not_all_read: bool,
}

impl Scrolling {
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
        let verb = if authorised.verb() == SCROLL_PAGE {
            SCROLL_PAGE
        } else {
            SCROLL_TO
        };
        let reached = reached(authorised, verb, adapters, grants, installed, strings)?;
        let call = reached.authorised.call();
        let asked = if verb == SCROLL_TO {
            match call.value(TO) {
                Some(Value::Name(name)) => Some(Asked::To(name.clone())),
                _ => None,
            }
        } else {
            match call.value(TOWARD) {
                Some(Value::Choice { chosen, .. }) => Toward::called(chosen).map(Asked::APage),
                _ => None,
            }
        };
        let Some(asked) = asked else {
            let said = strings.say(
                &fallback::NOT_THIS_VERB.key(),
                &Filling::of(fallback::VERB, call.verb().to_owned()),
            );
            return Err(Refused::worded_elsewhere(call.clone(), said));
        };
        Ok(Self {
            authorised: reached.authorised,
            application: reached.application,
            asked,
        })
    }

    /// The application.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// What was asked.
    #[must_use]
    pub const fn asked(&self) -> &Asked {
        &self.asked
    }

    /// This scroll refused in these words.
    fn refused(&self, word: Word, strings: &Strings) -> Refused {
        Refused::worded_elsewhere(self.authorised.call().clone(), self.say(word, strings))
    }

    /// One of scrolling's sentences, filled in for this scroll.
    fn say(&self, word: Word, strings: &Strings) -> Said {
        let name = match &self.asked {
            Asked::To(name) => name.clone(),
            Asked::APage(_) => String::new(),
        };
        strings.say(
            &word.key(),
            &Filling::of(fallback::APPLICATION, self.application.clone()).and(fallback::NAME, name),
        )
    }

    /// Find what to scroll to now, and ask the application to scroll it onto
    /// the screen, once.
    ///
    /// # Errors
    /// [`Refused`] when nothing was asked to scroll, in the words a person is
    /// told — and when the application answered that it could not.
    pub fn scroll(
        self,
        tree: &dyn AccessibilityTree,
        strings: &Strings,
    ) -> Result<Scrolled, Refused> {
        let read = match self.read(tree) {
            Ok(read) => read,
            Err(not) => {
                let said = not.said(&self.application, None, strings);
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        let (at, edge, was_showing) = match &self.asked {
            Asked::To(name) => {
                let named: Vec<&Place> = read
                    .places
                    .iter()
                    .filter(|place| &place.name == name)
                    .collect();
                match named.as_slice() {
                    [] if read.not_all_read => return Err(self.refused(words::TOO_MUCH, strings)),
                    [] => return Err(self.refused(words::NOTHING_NAMED, strings)),
                    [_] if read.not_all_read => {
                        return Err(self.refused(words::TOO_MUCH, strings));
                    }
                    [one] if one.role == Role::PasswordField => {
                        return Err(self.refused(words::A_PASSWORD_FIELD, strings));
                    }
                    [one] => (one.at.clone(), Edge::Top, one.showing),
                    _ => return Err(self.refused(words::MORE_THAN_ONE, strings)),
                }
            }
            Asked::APage(toward) => match a_page(&read.places, *toward) {
                Some(place) => (place.at.clone(), edge_for(*toward), false),
                None => {
                    let word = match toward {
                        Toward::Down => words::NOTHING_FURTHER_DOWN,
                        Toward::Up => words::NOTHING_FURTHER_UP,
                    };
                    return Err(self.refused(word, strings));
                }
            },
        };
        let answered = tree.scroll_to(&at, edge);
        let said_yes = match answered {
            Ok(said_yes) => said_yes,
            Err(fault) => {
                let said = NotWalked::Tree(fault).said(&self.application, None, strings);
                return Err(Refused::worded_elsewhere(
                    self.authorised.call().clone(),
                    said,
                ));
            }
        };
        let now_showing = tree
            .facts(&at)
            .map(|facts| States::of_the_tree(&facts.states).showing())
            .unwrap_or(false);
        let told = match (said_yes, was_showing, now_showing) {
            (false, true, _) => Some(words::ALREADY_SHOWING),
            (false, false, _) => return Err(self.refused(words::DID_NOT_SCROLL, strings)),
            (true, _, true) => None,
            (true, _, false) => Some(words::NOT_SEEN_TO_MOVE),
        };
        let told = told.map(|word| self.say(word, strings));
        Ok(Scrolled {
            authorised: self.authorised,
            application: self.application,
            asked: self.asked,
            now_showing,
            told,
        })
    }

    /// Every not-hidden thing in the application's showing windows, in
    /// reading order — names and states only, never a text.
    fn read(&self, tree: &dyn AccessibilityTree) -> Result<Read, NotWalked> {
        let roots: Vec<NodeAt> = tree
            .applications()
            .map_err(NotWalked::Tree)?
            .into_iter()
            .filter(|running| running.is.as_deref() == Some(self.application.as_str()))
            .map(|running| running.at)
            .collect();
        let mut read = Read {
            places: Vec::new(),
            not_all_read: false,
        };
        let mut asked = 0_usize;
        let mut windows = 0_usize;
        for root in roots {
            let facts = match tree.facts(&root) {
                Ok(facts) => facts,
                Err(TreeFault::Vanished) => continue,
                Err(fault) => return Err(NotWalked::Tree(fault)),
            };
            for window in facts.children {
                if window.holder != root.holder {
                    continue;
                }
                let facts = match tree.facts(&window) {
                    Ok(facts) => facts,
                    Err(TreeFault::Vanished) => continue,
                    Err(fault) => return Err(NotWalked::Tree(fault)),
                };
                if !States::of_the_tree(&facts.states).showing() {
                    continue;
                }
                windows += 1;
                // Depth-first, in reading order, with the stack in reverse.
                let mut stack: Vec<(NodeAt, usize)> = facts
                    .children
                    .iter()
                    .rev()
                    .filter(|child| child.holder == window.holder)
                    .map(|child| (child.clone(), 1))
                    .collect();
                read.not_all_read |= facts.unlisted_children;
                while let Some((at, depth)) = stack.pop() {
                    if asked >= MOST_ASKED {
                        read.not_all_read = true;
                        break;
                    }
                    asked += 1;
                    let facts = match tree.facts(&at) {
                        Ok(facts) => facts,
                        Err(TreeFault::Vanished) => continue,
                        Err(fault) => return Err(NotWalked::Tree(fault)),
                    };
                    let showing = States::of_the_tree(&facts.states).showing();
                    if !showing && !holds(&facts.states, VISIBLE) {
                        continue;
                    }
                    read.not_all_read |= facts.unlisted_children;
                    if depth < DEEPEST {
                        stack.extend(
                            facts
                                .children
                                .iter()
                                .rev()
                                .filter(|child| child.holder == at.holder)
                                .map(|child| (child.clone(), depth + 1)),
                        );
                    } else if !facts.children.is_empty() {
                        read.not_all_read = true;
                    }
                    read.places.push(Place {
                        role: Role::of_the_tree(facts.role),
                        name: facts.name,
                        showing,
                        depth,
                        at,
                    });
                }
            }
        }
        if windows == 0 {
            return Err(NotWalked::NoWindow);
        }
        Ok(read)
    }
}

/// The edge a page brings its thing to.
const fn edge_for(toward: Toward) -> Edge {
    match toward {
        Toward::Down => Edge::Top,
        Toward::Up => Edge::Bottom,
    }
}

/// The thing a page brings onto the screen: in the one showing document if
/// there is exactly one, otherwise in everything read.
///
/// Down is the first thing scrolled away **after** something on the screen;
/// up, the last thing scrolled away **before** something on the screen. A
/// password field is passed over.
fn a_page(places: &[Place], toward: Toward) -> Option<&Place> {
    let documents: Vec<usize> = places
        .iter()
        .enumerate()
        .filter(|(_, place)| place.role == Role::Document && place.showing)
        .map(|(index, _)| index)
        .collect();
    // The document's own things: everything after it that is deeper than it,
    // up to the first that is not.
    let within: Vec<&Place> = match documents.as_slice() {
        [one] => {
            let mut after = places.iter().skip(*one);
            let depth = after.next().map_or(0, |document| document.depth);
            after.take_while(|place| place.depth > depth).collect()
        }
        _ => places.iter().collect(),
    };
    let candidate = |place: &&&Place| !place.showing && place.role != Role::PasswordField;
    match toward {
        Toward::Down => within
            .iter()
            .skip_while(|place| !place.showing)
            .find(candidate)
            .copied(),
        Toward::Up => within
            .iter()
            .rev()
            .skip_while(|place| !place.showing)
            .find(candidate)
            .copied(),
    }
}

/// A scroll the application was asked for and did not refuse.
#[derive(Debug)]
pub struct Scrolled {
    /// What ran.
    authorised: Authorised,
    /// The application.
    application: String,
    /// What was asked.
    asked: Asked,
    /// Whether the thing scrolled to reads as on the screen afterwards.
    now_showing: bool,
    /// What a person is told beside the sentence they approved, if anything.
    told: Option<Said>,
}

impl Scrolled {
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

    /// What was asked.
    #[must_use]
    pub const fn asked(&self) -> &Asked {
        &self.asked
    }

    /// Whether the thing scrolled to reads as on the screen afterwards.
    #[must_use]
    pub const fn now_showing(&self) -> bool {
        self.now_showing
    }

    /// What a person is told beyond the sentence they approved: that it was
    /// on the screen already, or that the window does not yet say it moved.
    #[must_use]
    pub const fn said(&self) -> Option<&Said> {
        self.told.as_ref()
    }

    /// Give back what ran, so it can be recorded.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}
