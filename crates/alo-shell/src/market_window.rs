//! The window of what a person can install: `docs/features.md`'s *decided list
//! of what a person can install*, drawn from `alo-software`'s catalogue and
//! from nothing else.
//!
//! # It reads, and nothing here installs
//!
//! A person moves through the list and narrows it to a kind of work. **Nothing
//! in this file installs anything**, and `crate::MarketKey` has no key that
//! would: installing changes the machine, and a change is proposed as a
//! sentence and waits for one approval (ADR 0001). A window somebody is
//! reading through is not where an approval happens.
//!
//! # The list is the catalogue's, in the catalogue's order
//!
//! Nothing here sorts, ranks, scores or recommends. `alo_software::Catalogue`
//! keeps its entries in [`Category::EVERY`]'s order and then the order the
//! decided list is written in, and that is the order a person sees. A window
//! that re-ordered would be a second opinion about what to install, which is a
//! thing this product does not have.
//!
//! # A kind of work with nothing in it cannot happen
//!
//! `Catalogue::read` refuses a list that leaves a [`Category`] empty, so
//! narrowing to one always leaves at least one application. That is why there
//! is no *nothing here* state in this file: the only way to see an empty list
//! would be a catalogue that does not hold, and such a catalogue is refused
//! before it reaches a window.
//!
//! # No word is written here
//!
//! Not the window's name, not a kind of work's name, not the line under the
//! list. Those are `alo_software::words`', said once a frame into
//! `crate::TheMarketsWords`, because the crate that owns the subject owns the
//! sentence — and because `tests/desktop_source.rs` holds the drawing path to
//! wording nothing at all.

use alo_software::{Catalogue, Category, Listed, Stopped};

use crate::MarketKey;

/// Where one application has got to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowItIsGoing<'a> {
    /// Nobody has asked for this one.
    NotAsked,
    /// The person asked for it, and the machine has not finished.
    ///
    /// **This window did not install it and cannot.** It said the person asked;
    /// what is happening is the host's doing, and the host says when it is
    /// over.
    Asked,
    /// It is on this machine.
    Here,
    /// It was refused, in `alo-software`'s own words.
    ///
    /// The refusal is the type rather than a sentence, because this crate may
    /// not word one — `crate::market_rows` says it, from the crate that
    /// refused.
    Refused(&'a Stopped),
}

/// What the window of what a person can install draws now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketShows<'a> {
    /// The window is closed.
    Nothing,
    /// The applications a person can install, narrowed to a kind of work or
    /// not narrowed at all.
    Browsing(Browsing<'a>),
}

/// The applications on show, and where the person is in them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Browsing<'a> {
    /// The decided list.
    catalogue: &'a Catalogue,
    /// The kind of work it is narrowed to, or [`None`] for all of them.
    under: Option<Category>,
    /// Which of the applications on show the person is on.
    at: usize,
    /// What the machine already has, by identifier.
    already: &'a [String],
    /// The one the person asked for, while the host is still at it.
    asked: Option<&'a str>,
    /// The last one that was refused, and why.
    refused: Option<&'a (String, Stopped)>,
}

impl<'a> Browsing<'a> {
    /// The kind of work the list is narrowed to, or [`None`] for all of them.
    #[must_use]
    pub const fn under(&self) -> Option<Category> {
        self.under
    }

    /// Which of the applications on show the person is on, counting from zero.
    #[must_use]
    pub const fn at(&self) -> usize {
        self.at
    }

    /// Every application on show, in the catalogue's own order.
    pub fn rows(&self) -> impl Iterator<Item = &'a Listed> {
        let under = self.under;
        self.catalogue
            .every()
            .iter()
            .filter(move |one| under.is_none_or(|kind| one.category() == kind))
    }

    /// How many applications are on show.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.rows().count()
    }

    /// The application the person is on.
    ///
    /// [`None`] only where the catalogue holds nothing at all, which
    /// `Catalogue::read` refuses.
    #[must_use]
    pub fn whichever_they_are_on(&self) -> Option<&'a Listed> {
        self.rows().nth(self.at)
    }

    /// Where one of the applications on show has got to.
    ///
    /// **Four states and no fifth**: a refusal that has been read is still a
    /// refusal, and a row that says nothing about what happened is the one a
    /// person asks about.
    #[must_use]
    pub fn how_it_is_going(&self, one: &Listed) -> HowItIsGoing<'a> {
        let identifier = one.application().identifier();
        if self.already.iter().any(|here| here == identifier) {
            return HowItIsGoing::Here;
        }
        if self.asked == Some(identifier) {
            return HowItIsGoing::Asked;
        }
        match self.refused {
            Some((which, why)) if which == identifier => HowItIsGoing::Refused(why),
            Some(_) | None => HowItIsGoing::NotAsked,
        }
    }
}

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketPressed {
    /// Nothing at all.
    Nothing,
    /// The person moved to another application.
    Moved,
    /// The list was narrowed to another kind of work, or widened to all.
    Narrowed,
    /// **The person asked for an application to be installed.**
    ///
    /// Nothing has happened yet. The host reads
    /// [`MarketWindow::what_they_asked_for`], does the installing — on the
    /// indicator, which `alo_software::install` refuses to run without — and
    /// says what became of it through [`MarketWindow::it_arrived`] or
    /// [`MarketWindow::it_was_refused`].
    AskedToInstall,
    /// The window closed.
    Closed,
}

/// The window of what a person can install.
///
/// # Why this is `Clone`, and what a copy of it is not
///
/// **A copy is a reading, never a second window.** `crate::direct_desktop`
/// copies an open one once a frame so the drawing can borrow something the lane
/// owns rather than borrowing the `Server` it writes to a moment later. That
/// copy is read and dropped inside one draw.
///
/// It is **not** a way to have two of these on a machine. There is one per seat,
/// which is what makes `MarketWindow::asked`'s *one at a time* true and what
/// makes `where_the_market_opens` able to centre it. A second long-lived copy
/// would be two windows disagreeing about what a person asked for, and the one
/// a person could see would not be the one the host answered.
#[derive(Debug, Default, Clone)]
pub struct MarketWindow {
    /// The decided list, while the window shows it.
    catalogue: Option<Catalogue>,
    /// What this machine already has, as the host found it when the window
    /// opened, and whatever has arrived since.
    already: Vec<String>,
    /// The one the person asked for, until the host says what became of it.
    ///
    /// **One at a time.** A person who asked for two things before either
    /// finished would be shown two rows doing something and could not tell
    /// which answer belonged to which.
    asked: Option<String>,
    /// The last one that was refused, and why.
    refused: Option<(String, Stopped)>,
    /// The kind of work the list is narrowed to, or [`None`] for all of them.
    under: Option<Category>,
    /// Which of the applications on show the person is on.
    at: usize,
    /// Whether the window is open.
    open: bool,
}

/// Every kind of work a person can narrow to, and the widest view first.
///
/// [`None`] is *all of them* and is first because that is what the window
/// opens on; moving left from it wraps to the last kind, and right from the
/// last wraps back to it.
fn the_ways_to_narrow() -> Vec<Option<Category>> {
    let mut all = vec![None];
    all.extend(Category::EVERY.into_iter().map(Some));
    all
}

impl MarketWindow {
    /// The window, closed.
    #[must_use]
    pub fn closed() -> Self {
        Self::default()
    }

    /// A person opened the window on this catalogue.
    ///
    /// It opens on every kind of work and on the first application, however it
    /// was left last time: a window that remembered where somebody had been
    /// would show a narrowed list to a person who did not narrow it.
    /// `already` is what the host found on the machine — the identifiers
    /// `alo_software`'s rented tool reports installed. A window opened without
    /// it would offer to install what is already here, which is the lie
    /// `Catalogue`'s *nothing already shipped* refusal exists to prevent for
    /// the shipped seven and which only the host can prevent for the rest.
    pub fn opened(&mut self, catalogue: Catalogue, already: Vec<String>) {
        self.catalogue = Some(catalogue);
        self.already = already;
        self.asked = None;
        self.refused = None;
        self.under = None;
        self.at = 0;
        self.open = true;
    }

    /// A person closed the window.
    ///
    /// **An installation the host is still doing is not cancelled by this**,
    /// because closing a window is not a way to stop work on a machine. The
    /// window forgets it; whatever the host is doing goes on, and the record
    /// of it is the record's business.
    pub fn close(&mut self) {
        self.open = false;
        self.catalogue = None;
        self.already.clear();
        self.asked = None;
        self.refused = None;
        self.under = None;
        self.at = 0;
    }

    /// What the person asked to have installed, for the host to carry out.
    ///
    /// [`None`] while nothing has been asked for, and [`None`] again once the
    /// host has said what became of it.
    #[must_use]
    pub fn what_they_asked_for(&self) -> Option<&Listed> {
        let asked = self.asked.as_deref()?;
        self.catalogue
            .as_ref()?
            .every()
            .iter()
            .find(|one| one.application().identifier() == asked)
    }

    /// The host says the application arrived.
    ///
    /// Anything other than the one that was asked for is ignored: this window
    /// asked for one thing, and an answer about another is an answer to a
    /// question it did not put.
    pub fn it_arrived(&mut self, identifier: &str) {
        if self.asked.as_deref() != Some(identifier) {
            return;
        }
        self.asked = None;
        if !self.already.iter().any(|here| here == identifier) {
            self.already.push(identifier.to_owned());
        }
    }

    /// The host says the application was refused, in `alo-software`'s words.
    pub fn it_was_refused(&mut self, identifier: &str, why: Stopped) {
        if self.asked.as_deref() != Some(identifier) {
            return;
        }
        self.asked = None;
        self.refused = Some((identifier.to_owned(), why));
    }

    /// Whether the window is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// What one key press does. Does nothing while the window is closed.
    pub fn pressed(&mut self, key: MarketKey) -> MarketPressed {
        if !self.open {
            return MarketPressed::Nothing;
        }
        match key {
            MarketKey::Close => {
                self.close();
                MarketPressed::Closed
            }
            MarketKey::Up => self.moved_to(self.at.saturating_sub(1)),
            MarketKey::Down => self.moved_to(self.at.saturating_add(1)),
            MarketKey::First => self.moved_to(0),
            MarketKey::Last => self.moved_to(self.how_many().saturating_sub(1)),
            MarketKey::Before => self.narrowed_by(-1),
            MarketKey::After => self.narrowed_by(1),
            MarketKey::Install => self.asked_for_it(),
            MarketKey::Nothing => MarketPressed::Nothing,
        }
    }

    /// What the window draws now.
    #[must_use]
    pub fn shows(&self) -> MarketShows<'_> {
        match &self.catalogue {
            Some(catalogue) if self.open => MarketShows::Browsing(Browsing {
                catalogue,
                under: self.under,
                at: self.at,
                already: &self.already,
                asked: self.asked.as_deref(),
                refused: self.refused.as_ref(),
            }),
            Some(_) | None => MarketShows::Nothing,
        }
    }

    /// The person asked for the application they are on.
    ///
    /// **Refused while another is still going**, because the window can only
    /// show one answer at a time and a second ask would make the first one's
    /// reply ambiguous. Refused for something already here, because there is
    /// nothing to do and a row that said *asked* about it would be waiting for
    /// an answer that never comes.
    fn asked_for_it(&mut self) -> MarketPressed {
        if self.asked.is_some() {
            return MarketPressed::Nothing;
        }
        let MarketShows::Browsing(browsing) = self.shows() else {
            return MarketPressed::Nothing;
        };
        let Some(one) = browsing.whichever_they_are_on() else {
            return MarketPressed::Nothing;
        };
        if browsing.how_it_is_going(one) == HowItIsGoing::Here {
            return MarketPressed::Nothing;
        }
        let identifier = one.application().identifier().to_owned();
        self.refused = None;
        self.asked = Some(identifier);
        MarketPressed::AskedToInstall
    }

    /// How many applications are on show under the present narrowing.
    fn how_many(&self) -> usize {
        match self.shows() {
            MarketShows::Browsing(browsing) => browsing.how_many(),
            MarketShows::Nothing => 0,
        }
    }

    /// Move to one of the applications on show, stopping at either end.
    ///
    /// **It stops rather than wrapping.** A list a person is reading through
    /// that jumped from the last row to the first would move the thing they
    /// were about to choose out from under them.
    fn moved_to(&mut self, row: usize) -> MarketPressed {
        let how_many = self.how_many();
        if how_many == 0 {
            return MarketPressed::Nothing;
        }
        let bounded = row.min(how_many - 1);
        if bounded == self.at {
            return MarketPressed::Nothing;
        }
        self.at = bounded;
        MarketPressed::Moved
    }

    /// Narrow to the kind of work before or after this one, wrapping round.
    ///
    /// **This one wraps and moving does not**, which is deliberate: the kinds
    /// of work are a ring a person steps around to see what there is, and
    /// there is no last one to fall off. Narrowing always returns to the first
    /// application, because the row somebody was on belongs to the list they
    /// have just left.
    fn narrowed_by(&mut self, by: isize) -> MarketPressed {
        let ways = the_ways_to_narrow();
        let Some(now) = ways.iter().position(|way| *way == self.under) else {
            return MarketPressed::Nothing;
        };
        let count = ways.len() as isize;
        let next = (now as isize + by).rem_euclid(count) as usize;
        let Some(under) = ways.get(next) else {
            return MarketPressed::Nothing;
        };
        if *under == self.under {
            return MarketPressed::Nothing;
        }
        self.under = *under;
        self.at = 0;
        MarketPressed::Narrowed
    }
}

#[cfg(test)]
#[path = "market_window_tests.rs"]
mod tests;
