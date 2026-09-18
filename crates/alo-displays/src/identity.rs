//! What a screen is, so that it is the same screen tomorrow morning.
//!
//! The failure everybody knows is the laptop that forgets, every morning, that
//! the external screen is on the left. It forgets because it remembered the
//! wrong thing: the socket the cable happens to be in, which changes when
//! somebody swaps two cables and is reused by the next screen plugged into it.
//!
//! So a screen is remembered by **what it says about itself** — its make, its
//! model, and the serial number where it has one. That is what survives being
//! unplugged, carried to another desk and plugged into a different socket.
//!
//! # And the two ways that is not enough, each with a decision
//!
//! **A screen that says nothing about itself** — a great many built-in laptop
//! panels, and every adapter that does not pass the description through — is
//! remembered by the socket it is plugged into ([`Identity::Socket`]). It is
//! **weaker**, and the word for it says so to the person: another screen in the
//! same socket is set up as though it were this one. There is no third choice.
//! Refusing to remember such a screen at all would mean the laptop's own panel,
//! which is most machines' only screen, was the one thing never remembered.
//!
//! **Two screens that say exactly the same thing** — the pair of identical
//! monitors on a desk, neither of which reports a serial number — cannot be
//! told apart by what they say, because what they say is the same. Nothing
//! about one of them is different except where it is plugged in, so that is
//! what tells them apart: [`whoever_is_attached`] hands **both** of them their
//! socket. It is decided over the whole set rather than one screen at a time,
//! because whether a description is unique is not a fact about one screen.

use alo_strings::{Filling, Said, Strings, Word};

use crate::words;

/// Why a piece of text does not name a screen.
///
/// There is no `Display`: the only road to words is [`IdentityError::said`],
/// and what a settings file that did not read writes is
/// [`crate::unreadable::NotRead`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    /// Nothing at all.
    Unnamed,
    /// A name with a space at one end, which reads as the same name and is not.
    Spaced(String),
}

impl IdentityError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::Unnamed => words::NOT_A_NAME,
            Self::Spaced(_) => words::NAME_SPACED,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// The name goes in **quoted**, because the whole of what is wrong with it
    /// is a space nobody can see.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Unnamed => Filling::nothing(),
            Self::Spaced(name) => Filling::of("name", format!("{name:?}")),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// One piece of text a screen or a settings file offered as a name.
///
/// # Errors
/// [`IdentityError`] for nothing at all, or for a name padded with spaces that
/// would stop it ever matching the screen it looks like.
fn a_name(text: &str) -> Result<String, IdentityError> {
    if text.is_empty() {
        return Err(IdentityError::Unnamed);
    }
    if text.trim() != text {
        return Err(IdentityError::Spaced(text.to_owned()));
    }
    Ok(text.to_owned())
}

/// Which socket a screen is plugged into.
///
/// The live handle for everything that happens while a screen is attached: one
/// screen per socket, always, so it is what a hotplug names
/// ([`crate::Attached::unplugged`]). It is a stable *identity* only for a
/// screen that says nothing about itself, and [`Stability`] is where that is
/// written down.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Socket {
    /// What the compositor calls it.
    name: String,
}

impl Socket {
    /// The socket the compositor calls this.
    ///
    /// # Errors
    /// [`IdentityError`] for an empty name or one padded with spaces.
    pub fn named(name: &str) -> Result<Self, IdentityError> {
        Ok(Self {
            name: a_name(name)?,
        })
    }

    /// What the compositor calls it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// What a screen says about itself.
///
/// A make and a model always, because a description with neither is not a
/// description; a serial number where the screen has one, which is the only
/// part that tells two of the same model apart.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Panel {
    /// Who made it.
    make: String,
    /// What they call it.
    model: String,
    /// Its own number, where it has one.
    serial: Option<String>,
}

impl Panel {
    /// What this screen says about itself.
    ///
    /// # Errors
    /// [`IdentityError`] for a make, a model or a serial that is empty or
    /// padded with spaces.
    pub fn of(make: &str, model: &str, serial: Option<&str>) -> Result<Self, IdentityError> {
        Ok(Self {
            make: a_name(make)?,
            model: a_name(model)?,
            serial: serial.map(a_name).transpose()?,
        })
    }

    /// Who made it.
    #[must_use]
    pub fn make(&self) -> &str {
        &self.make
    }

    /// What they call it.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Its own number, where it has one.
    #[must_use]
    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// What a person reads when this screen is named to them.
    #[must_use]
    pub fn as_a_person_reads_it(&self) -> String {
        format!("{} {}", self.make, self.model)
    }
}

/// How well a screen can be told from another one tomorrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stability {
    /// It says what it is, so it is the same screen in any socket.
    WhatTheScreenSays,
    /// It says nothing anybody can tell apart, so it is remembered by the
    /// socket it is plugged into — and another screen in that socket is
    /// treated as this one.
    WhereItIsPluggedIn,
}

/// Which screen this is, for as long as the person owns it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Identity {
    /// A screen that says what it is.
    Panel(Panel),
    /// A screen that does not, remembered by where it is plugged in.
    Socket(Socket),
}

impl Identity {
    /// How well this can be told from another screen tomorrow.
    #[must_use]
    pub const fn how_stable(&self) -> Stability {
        match *self {
            Self::Panel(_) => Stability::WhatTheScreenSays,
            Self::Socket(_) => Stability::WhereItIsPluggedIn,
        }
    }

    /// What a person reads when this screen is named to them.
    ///
    /// What the screen says about itself, which is what is printed on its front
    /// — or, for a screen that says nothing, the socket, because a person
    /// looking at the back of a machine can find that and there is nothing else
    /// to give them.
    #[must_use]
    pub fn as_a_person_reads_it(&self) -> String {
        match self {
            Self::Panel(panel) => panel.as_a_person_reads_it(),
            Self::Socket(socket) => socket.name().to_owned(),
        }
    }
}

/// Which screen each of these is, decided over the whole set.
///
/// In the same order as what was handed in. A description that more than one of
/// them gives tells none of them apart, so every screen giving it is handed its
/// socket instead — see this file's header for why that is decided here rather
/// than one screen at a time.
#[must_use]
pub fn whoever_is_attached(says: &[(Socket, Option<Panel>)]) -> Vec<Identity> {
    says.iter()
        .map(|(socket, panel)| match panel {
            Some(panel)
                if says
                    .iter()
                    .filter(|(_, other)| other.as_ref() == Some(panel))
                    .count()
                    == 1 =>
            {
                Identity::Panel(panel.clone())
            }
            Some(_) | None => Identity::Socket(socket.clone()),
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A screen that says what it is, is that screen in any socket — and a
    /// screen that differs by one letter of its serial is a different one.
    #[test]
    fn a_screen_that_says_what_it_is_is_matched_exactly() {
        let one = Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap();
        let same = Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap();
        let other = Panel::of("Dell", "U2720Q", Some("CN-0ABD")).unwrap();
        assert_eq!(one, same);
        assert_ne!(one, other);
        assert_eq!(one.as_a_person_reads_it(), "Dell U2720Q");
        assert_eq!(
            Identity::Panel(one).how_stable(),
            Stability::WhatTheScreenSays
        );
    }

    /// **A screen that says nothing is remembered by its socket, and that is
    /// weaker** — which is a fact this crate answers rather than hides.
    #[test]
    fn a_screen_that_says_nothing_is_remembered_by_its_socket_and_says_so() {
        let built_in = Identity::Socket(Socket::named("eDP-1").unwrap());
        assert_eq!(built_in.how_stable(), Stability::WhereItIsPluggedIn);
        assert_eq!(built_in.as_a_person_reads_it(), "eDP-1");
    }

    /// **Two screens that say exactly the same thing are told apart by where
    /// they are plugged in**, and both of them are — one keeping a description
    /// that matches the other would be a coin toss every morning.
    #[test]
    fn twin_screens_are_told_apart_by_where_they_are_plugged_in() {
        let twin = Panel::of("Acme", "P24", None).unwrap();
        let alone = Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap();
        let attached = whoever_is_attached(&[
            (Socket::named("DP-1").unwrap(), Some(twin.clone())),
            (Socket::named("DP-2").unwrap(), Some(twin)),
            (Socket::named("HDMI-1").unwrap(), Some(alone.clone())),
            (Socket::named("eDP-1").unwrap(), None),
        ]);
        assert_eq!(
            attached,
            vec![
                Identity::Socket(Socket::named("DP-1").unwrap()),
                Identity::Socket(Socket::named("DP-2").unwrap()),
                Identity::Panel(alone),
                Identity::Socket(Socket::named("eDP-1").unwrap()),
            ]
        );
    }

    /// A name that could never match anything is refused where it is given,
    /// rather than becoming a row in the settings file that does nothing.
    #[test]
    fn a_name_that_would_never_match_is_refused() {
        assert_eq!(Socket::named(""), Err(IdentityError::Unnamed));
        assert_eq!(
            Socket::named("DP-1 "),
            Err(IdentityError::Spaced("DP-1 ".to_owned()))
        );
        assert_eq!(Panel::of("", "U2720Q", None), Err(IdentityError::Unnamed));
        assert_eq!(Panel::of("Dell", "", None), Err(IdentityError::Unnamed));
        assert_eq!(
            Panel::of("Dell", "U2720Q", Some(" CN")),
            Err(IdentityError::Spaced(" CN".to_owned()))
        );
    }

    /// **The refusal shows the space.** A name whose problem is invisible is
    /// quoted where it goes into the sentence.
    #[test]
    fn the_refusal_shows_the_space_nobody_can_see() {
        let strings = in_english();
        let said = Socket::named("DP-1 ").unwrap_err().said(&strings);
        assert!(said.text().contains("\"DP-1 \""), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug());

        let unnamed = IdentityError::Unnamed.said(&strings);
        assert!(unnamed.text().contains("name the screen"), "{unnamed}");
    }
}
