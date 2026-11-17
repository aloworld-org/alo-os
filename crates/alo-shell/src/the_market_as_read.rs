//! **The window of what a person can install, as a screen reader is told it.**
//!
//! Until 2026-11-17 the market was drawn and never described: nothing in
//! `crate::access_nodes` named it, so a person using Orca who pressed the
//! Applications entry heard nothing at all while a window opened in front of
//! them. That breaks the accessibility promise, and the coordinator asked for
//! it as its own change.
//!
//! # What is read, in order
//!
//! - **the window**, named as its title bar is — *Applications* — among the
//!   windows open on the desktop, in front while it takes the keys;
//! - **the kind of work** it is narrowed to, as a label;
//! - **the applications on show**, a list: each by its makers' name, with what
//!   the row draws under it as its description — the kind of work, **every
//!   label in full** (*Runs programs outside its box: …*), the licence where
//!   it is an open one, and what the control says — and its place in the list.
//!   The one the person is on is selected and has the keyboard;
//! - **the line under the list**.
//!
//! # Every word is one the window draws
//!
//! The names, labels and control words come from `crate::TheMarketsWords` and
//! `alo_software::Listed`, the same values `crate::market_raster` draws, so a
//! row cannot be drawn and not heard. A label is read in full rather than as
//! its short form, because a reader cannot glance back at the row to see what
//! *Reaches your files* was about.
//!
//! # What a reader cannot do here yet
//!
//! **Press.** A row carries no [`crate::access_action::Pressed`]: the market's
//! own keys (arrows and Enter) reach it, and a press on the tree has no road to
//! the window's install yet. That is said here rather than left as a node a
//! reader can find and not press.

use alo_access::{Role, State};
use alo_software::Listed;

use crate::access_attributes::its_place;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::market_window::{Browsing, HowItIsGoing, MarketShows};
use crate::the_markets_words::TheMarketsWords;

/// One application on show, as it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ARowAsRead {
    /// Its makers' name for it.
    pub(crate) name: String,
    /// What the row draws under its name, read after it.
    pub(crate) description: String,
    /// Whether the person is on it.
    pub(crate) selected: bool,
}

/// The market as a reader is told it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheMarketAsRead {
    /// The window's title.
    pub(crate) title: String,
    /// The kind of work it is narrowed to, or *Everything*.
    pub(crate) narrowed_to: String,
    /// The applications on show, in the order drawn.
    pub(crate) rows: Vec<ARowAsRead>,
    /// The line under the list.
    pub(crate) promise: String,
    /// Whether the window has the keys.
    pub(crate) takes_the_keys: bool,
}

impl TheMarketAsRead {
    /// The market as it shows now, or [`None`] while it is shut.
    #[must_use]
    pub(crate) fn of(
        showing: MarketShows<'_>,
        words: &TheMarketsWords,
        takes_the_keys: bool,
    ) -> Option<Self> {
        let MarketShows::Browsing(browsing) = showing else {
            return None;
        };
        let rows = browsing
            .rows()
            .enumerate()
            .map(|(at, one)| ARowAsRead {
                name: one.name().to_owned(),
                description: what_the_row_says(one, &browsing, words),
                selected: at == browsing.at(),
            })
            .collect();
        Some(Self {
            title: words.heading.clone(),
            narrowed_to: words.the(browsing.under()).to_owned(),
            rows,
            promise: words.promise.clone(),
            takes_the_keys,
        })
    }
}

/// What a row draws under its name, read in order: the kind of work, each
/// label in full, the licence where it is an open one, and the control.
fn what_the_row_says(one: &Listed, browsing: &Browsing<'_>, words: &TheMarketsWords) -> String {
    let mut parts = vec![words.the(Some(one.category())).to_owned()];
    parts.extend(words.labels_of_in_full(one));
    if one.is_open_source() {
        parts.push(one.licence().to_owned());
    }
    let control = match browsing.how_it_is_going(one) {
        HowItIsGoing::Here => &words.open,
        HowItIsGoing::Asked => &words.installing,
        HowItIsGoing::NotAsked | HowItIsGoing::Refused(_) => &words.install,
    };
    parts.push(control.clone());
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(". ")
}

impl ReadAloudTree {
    /// The same tree with the market among the windows open, while it is open.
    #[must_use]
    pub(crate) fn with_the_market(mut self, market: Option<TheMarketAsRead>) -> Self {
        let Some(market) = market else {
            return self;
        };
        let Some(list) = self.the_windows_open_list() else {
            return self;
        };
        let window_states = access_roles::words_of(State::CanBeUsed, None);
        let window = self.push(
            list,
            Node {
                path: String::new(),
                role: access_roles::number_of(Role::ListItem),
                name: market.title.clone(),
                states: if market.takes_the_keys {
                    access_roles::the_window_in_front(window_states)
                } else {
                    window_states
                },
                announced: false,
                parent: list,
                children: Vec::new(),
                description: String::new(),
                press: None,
                attributes: Vec::new(),
            },
        );
        self.push(window, a_label(window, market.narrowed_to));
        let shelf = self.push(
            window,
            Node {
                role: access_roles::number_of(Role::List),
                name: market.title,
                ..a_label(window, String::new())
            },
        );
        let how_many = market.rows.len();
        for (at, row) in market.rows.into_iter().enumerate() {
            let mut states = access_roles::words_of(State::CanBeUsed, None);
            if row.selected && market.takes_the_keys {
                states = access_roles::with_the_keyboard(states);
            }
            self.push(
                shelf,
                Node {
                    path: String::new(),
                    role: access_roles::number_of(Role::ListItem),
                    name: row.name,
                    states,
                    announced: false,
                    parent: shelf,
                    children: Vec::new(),
                    description: row.description,
                    press: None,
                    attributes: its_place(at, how_many),
                },
            );
        }
        self.push(window, a_label(window, market.promise));
        self
    }
}

/// Words read and not acted on, under `parent`.
fn a_label(parent: usize, name: String) -> Node {
    Node {
        path: String::new(),
        role: access_roles::number_of(Role::Label),
        name,
        states: access_roles::words_of(State::ReadOnly, None),
        announced: false,
        parent,
        children: Vec::new(),
        description: String::new(),
        press: None,
        attributes: Vec::new(),
    }
}

#[cfg(test)]
#[path = "the_market_as_read_tests.rs"]
mod tests;
