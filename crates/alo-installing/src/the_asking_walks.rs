//! The whole walk a person makes, as a state machine over the screens.
//!
//! # The order, and why it is this one
//!
//! | | and why it is there |
//! |---|---|
//! | the keyboard | **first, because typing presupposes it.** The two things this environment asks a person to type are a network password and a disk's own name, and both fail silently and identically on the wrong layout |
//! | the network | alo OS is downloaded, so there is nothing to install without one |
//! | the password | only where the network a person chose wants one |
//! | the disk | what gets erased, so it comes after everything that cannot be undone is known |
//! | the review | **nothing is erased until a person types the disk's own name**, which is `words.rs`' promise and the same one the Windows half makes |
//!
//! # What a field is, and why it is a card
//!
//! The password and the typed disk name are both **a card whose lower line is
//! what has been typed.** That is a decision and worth seeing: the adopted
//! frames have one layout, a card is the only box in it, and a text field
//! drawn as anything else would be the first thing in this environment that
//! does not look like the rest of it. So a field needs no new geometry, and
//! the caret is a character on the end of the line.
//!
//! A password's characters are replaced with dots. The **count is honest** —
//! one dot per character — because a person checking they typed twelve
//! characters is the only feedback a password field can honestly give, and the
//! router will not explain itself.
//!
//! # What this file does not do
//!
//! It holds no words and draws nothing. [`crate::what_each_screen_says`] turns
//! this state into sentences and [`crate::what_a_screen_shows`] puts them
//! somewhere; this is only *where a person has got to and what the keys mean
//! there.* That is what makes the whole walk testable on a machine with no
//! screen, which is the only way the walk gets tested at all before there is
//! hardware to walk it on.

/// What a person meant by the key they pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatThePersonDid {
    /// Move up the cards.
    WentUp,
    /// Move down the cards.
    WentDown,
    /// Take the card the ring is on, or accept what has been typed.
    Chose,
    /// Go back to the screen before.
    WentBack,
    /// Put this character into the field.
    Typed(char),
    /// Take one character back out.
    RubbedOut,
    /// Nothing this screen acts on.
    NothingThisScreenActsOn,
}

/// Which screen a person is looking at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichScreen {
    /// Which keyboard — two cards, one of them a guess that proves itself.
    TheKeyboard,
    /// Every layout, which *Something else* opens.
    EveryLayout,
    /// How this computer should connect — the networks a scan found.
    TheNetwork,
    /// The password for the network chosen, where it wants one.
    ThePassword,
    /// Which disk alo OS should use.
    TheDisk,
    /// What this erases, and the disk's own name typed back.
    TheReview,
}

/// Everything a person settled on, once they have settled on all of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAnswers {
    /// The keyboard layout, as `alo-keyboards` names it.
    pub layout: String,
    /// The network chosen, and `None` where a cable is being used.
    pub network: Option<String>,
    /// Its password, and empty where the network wanted none.
    ///
    /// **This is the one field in this crate that holds a secret**, and it
    /// holds it in memory only: ADR 0128 decided no pre-shared key is written
    /// to the staging partition, and `joining_a_network` keeps that true by
    /// construction with a temporary connection profile.
    pub password: String,
    /// The disk to install onto.
    pub disk: String,
}

/// Where the asking has got to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAsking {
    /// Which screen is on the display.
    on: WhichScreen,
    /// Which card the ring is on.
    card: usize,
    /// How many cards this screen has, on this display.
    how_many: usize,
    /// What has been typed into the field this screen has, if it has one.
    typed: String,
    /// The layout settled on.
    layout: Option<String>,
    /// The network chosen.
    network: Option<String>,
    /// Its password.
    password: String,
    /// The disk chosen.
    disk: Option<String>,
    /// Everything, once a person has finished.
    finished: Option<TheAnswers>,
}

/// What the card under the ring is, which only the caller knows.
///
/// The screens show networks, disks and layouts, and this file holds none of
/// them: a state machine that held the disk list would be a state machine that
/// could disagree with the screen about which disk the ring is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WhatTheRingIsOn<'a> {
    /// What it is called, where it is a layout, a network or a disk.
    pub named: Option<&'a str>,
    /// Whether this network wants a password. Ignored on every other screen.
    pub wants_a_password: bool,
}

impl TheAsking {
    /// Begin, on the keyboard question.
    #[must_use]
    pub fn begins() -> Self {
        Self {
            on: WhichScreen::TheKeyboard,
            card: 0,
            how_many: 2,
            typed: String::new(),
            layout: None,
            network: None,
            password: String::new(),
            disk: None,
            finished: None,
        }
    }

    /// Which screen is on the display.
    #[must_use]
    pub const fn on(&self) -> WhichScreen {
        self.on
    }

    /// Which card the ring is on.
    #[must_use]
    pub const fn card(&self) -> usize {
        self.card
    }

    /// What has been typed into this screen's field.
    #[must_use]
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// The layout settled on.
    #[must_use]
    pub fn layout(&self) -> Option<&str> {
        self.layout.as_deref()
    }

    /// The network chosen.
    #[must_use]
    pub fn network(&self) -> Option<&str> {
        self.network.as_deref()
    }

    /// The disk chosen.
    #[must_use]
    pub fn disk(&self) -> Option<&str> {
        self.disk.as_deref()
    }

    /// Everything, once a person has finished the walk.
    #[must_use]
    pub const fn finished(&self) -> Option<&TheAnswers> {
        self.finished.as_ref()
    }

    /// Whether this screen is one a person types into.
    #[must_use]
    pub const fn is_a_field(&self) -> bool {
        matches!(self.on, WhichScreen::ThePassword | WhichScreen::TheReview)
    }

    /// Whether what is typed here is hidden as it is typed.
    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        matches!(self.on, WhichScreen::ThePassword)
    }

    /// How many cards the screen in front of a person has.
    pub const fn there_are_this_many_cards(&mut self, how_many: usize) {
        self.how_many = how_many;
        if self.card >= how_many {
            self.card = how_many.saturating_sub(1);
        }
    }

    /// Pass over the network question where there is no network to choose.
    ///
    /// **A question with no answers is not a question**, and this is the one
    /// screen that can legitimately have none: a computer on a cable, a
    /// wireless card with no firmware, or a scan that simply found nothing.
    /// Asking anyway would leave a person on a screen where every key does
    /// nothing, which is the worst place this environment could put somebody —
    /// they cannot go on, and nothing on the screen says why.
    ///
    /// Why it is not silent: `why_there_is_no_network` already tells a person
    /// which of those three it was, on the console, in words they can act on.
    /// This only declines to make them press Enter at an empty list first.
    ///
    /// Returns whether anything moved, so a caller knows to redraw.
    pub fn there_is_no_network_to_choose(&mut self) -> bool {
        if self.on == WhichScreen::TheNetwork && self.how_many == 0 {
            self.network = None;
            self.password.clear();
            self.go_to(WhichScreen::TheDisk, 0, 0)
        } else {
            false
        }
    }

    /// Act on what a person did, and say whether the screen must be redrawn.
    ///
    /// A redraw for a key that changed nothing would make the whole screen
    /// flash on a key the environment does not even act on.
    pub fn and_then(&mut self, did: WhatThePersonDid, ring: WhatTheRingIsOn<'_>) -> bool {
        match did {
            WhatThePersonDid::WentUp => self.move_by(-1),
            WhatThePersonDid::WentDown => self.move_by(1),
            WhatThePersonDid::Chose => self.chose(ring),
            WhatThePersonDid::WentBack => self.went_back(),
            WhatThePersonDid::Typed(character) => {
                if self.is_a_field() {
                    self.typed.push(character);
                    true
                } else {
                    false
                }
            }
            WhatThePersonDid::RubbedOut => self.is_a_field() && self.typed.pop().is_some(),
            WhatThePersonDid::NothingThisScreenActsOn => false,
        }
    }

    /// Move the ring, where this screen has cards to move it over.
    fn move_by(&mut self, by: i32) -> bool {
        if self.is_a_field() {
            return false;
        }
        let was = self.card;
        let last = self.how_many.saturating_sub(1);
        self.card = if by < 0 {
            self.card.saturating_sub(1)
        } else {
            (self.card + 1).min(last)
        };
        self.card != was
    }

    /// Take whatever the ring is on, or accept what has been typed.
    fn chose(&mut self, ring: WhatTheRingIsOn<'_>) -> bool {
        match self.on {
            WhichScreen::TheKeyboard => {
                if self.card == 0 {
                    // The guess, which the card asked the keyboard to prove.
                    self.settle_the_layout(ring.named)
                } else {
                    self.go_to(WhichScreen::EveryLayout, 0, 0)
                }
            }
            WhichScreen::EveryLayout => self.settle_the_layout(ring.named),
            WhichScreen::TheNetwork => {
                let Some(named) = ring.named else {
                    return false;
                };
                self.network = Some(named.to_owned());
                self.typed.clear();
                // **A network that wants no password skips the field**, rather
                // than showing an empty one a person has to press Enter past.
                if ring.wants_a_password {
                    self.go_to(WhichScreen::ThePassword, 0, 0)
                } else {
                    self.go_to(WhichScreen::TheDisk, 0, 0)
                }
            }
            WhichScreen::ThePassword => {
                // An empty password is not accepted on a network that asked
                // for one: the router would simply refuse it, and a person
                // would read that as the password being wrong rather than
                // absent.
                if self.typed.is_empty() {
                    return false;
                }
                self.password = std::mem::take(&mut self.typed);
                self.go_to(WhichScreen::TheDisk, 0, 0)
            }
            WhichScreen::TheDisk => {
                let Some(named) = ring.named else {
                    return false;
                };
                self.disk = Some(named.to_owned());
                self.typed.clear();
                self.go_to(WhichScreen::TheReview, 0, 0)
            }
            WhichScreen::TheReview => self.erase_it_if_the_name_matches(),
        }
    }

    /// Settle a layout and go on to the network.
    fn settle_the_layout(&mut self, named: Option<&str>) -> bool {
        let Some(named) = named else {
            return false;
        };
        self.layout = Some(named.to_owned());
        self.go_to(WhichScreen::TheNetwork, 0, 0)
    }

    /// **The one place anything is allowed to be erased.**
    ///
    /// Only when the name typed matches the disk's own name, exactly. Not
    /// trimmed, not case-folded, not nearly: `words.rs` promises *nothing is
    /// changed until you type the name of the disk you choose*, and a check
    /// that accepted *nearly* would make that promise mean something looser
    /// than it says.
    fn erase_it_if_the_name_matches(&mut self) -> bool {
        let Some(disk) = self.disk.clone() else {
            return false;
        };
        if self.typed != disk {
            return false;
        }
        let Some(layout) = self.layout.clone() else {
            return false;
        };
        self.finished = Some(TheAnswers {
            layout,
            network: self.network.clone(),
            password: self.password.clone(),
            disk,
        });
        true
    }

    /// Go back one screen.
    ///
    /// **Going back un-answers the screen being left**, which is the whole
    /// point of going back: a person who returns to the networks and chooses a
    /// different one must not carry the first one's password forward.
    fn went_back(&mut self) -> bool {
        match self.on {
            // The first screen. There is nowhere back to, and a person left
            // looking at nothing has no way on.
            WhichScreen::TheKeyboard => false,
            WhichScreen::EveryLayout => self.go_to(WhichScreen::TheKeyboard, 0, 2),
            WhichScreen::TheNetwork => {
                self.layout = None;
                self.go_to(WhichScreen::TheKeyboard, 0, 2)
            }
            WhichScreen::ThePassword => {
                self.network = None;
                self.typed.clear();
                self.go_to(WhichScreen::TheNetwork, 0, 0)
            }
            WhichScreen::TheDisk => {
                self.network = None;
                self.password.clear();
                self.go_to(WhichScreen::TheNetwork, 0, 0)
            }
            WhichScreen::TheReview => {
                self.disk = None;
                self.typed.clear();
                self.go_to(WhichScreen::TheDisk, 0, 0)
            }
        }
    }

    /// Move to a screen, clearing what belongs to the one being left.
    fn go_to(&mut self, screen: WhichScreen, card: usize, how_many: usize) -> bool {
        self.on = screen;
        self.card = card;
        self.how_many = how_many;
        true
    }
}

#[cfg(test)]
#[path = "the_asking_walks_tests.rs"]
mod tests;
