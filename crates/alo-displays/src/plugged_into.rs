//! What a person reads for a screen that says nothing about itself.
//!
//! `crate::identity` decided that such a screen is **remembered** by the socket
//! it is plugged into, which is right and is not re-decided here. What is
//! decided here is the other half of that sentence: **what the person is told
//! it is called.**
//!
//! Until this file existed they were told the socket's own name — `eDP-1`,
//! `HDMI-A-2`, `DP-1`. That is the name the kernel's side of a graphics card
//! gives a connector, and a person who reads it has learned the name of
//! something alo OS rented (`docs/features.md`: *a person never learns the name
//! of anything we rented*). It is also the one thing in this crate that could
//! not be caught by reading the vocabulary, because the leak is not in a
//! sentence: it is in what gets put into one.
//!
//! So a socket becomes a [`PluggedInto`], and a [`PluggedInto`] is a string
//! this crate declares:
//!
//! | The machine's name for it | What a person reads |
//! |---|---|
//! | `eDP-1`, `LVDS-1`, `DSI-1`, `DPI-1` | Built-in screen |
//! | `DP-2` | DisplayPort socket 2 |
//! | `HDMI-A-1`, `HDMI-B-1`, `HDMI-1` | HDMI socket 1 |
//! | `DVI-D-1`, `DVI-I-1`, `DVI-A-1`, `DVI-1` | DVI socket 1 |
//! | `VGA-1` | VGA socket 1 |
//! | anything else ending in a number | Socket 3 |
//! | anything else | Another screen |
//!
//! # Why these words and not others
//!
//! **HDMI, DisplayPort, DVI and VGA are printed on the person's own machine**,
//! beside the hole the cable goes into. `alo_saying::rented` draws that line
//! already for the logo on a keyboard's modifier key: naming somebody's own
//! hardware is not asking them to learn anything. The connector *string* is not
//! printed anywhere, which is the whole difference.
//!
//! **The number is the connector's own**, so two HDMI sockets are told apart
//! the way the ports on the back of the machine are — by being two.
//!
//! # The last row is honest rather than clever
//!
//! A connector type alo OS does not know, with no number to give, becomes
//! *Another screen*. Two such screens would read alike, which is a real cost
//! and is smaller than the alternative: the alternative is teaching somebody
//! the word `SVIDEO-1` so that alo OS does not have to admit it has nothing
//! better. A screen like that is remembered perfectly well either way — the
//! socket is still its identity — and what is lost is only a way of saying
//! *that one*.
//!
//! # These are names, not clauses
//!
//! Each reads as a name and goes where a make and a model would: *Built-in
//! screen was unplugged, so what was open on it is now on Dell U2720Q*. They
//! are capitalised as names for that reason, and a translator is told so.

use alo_strings::{Filling, Said, Strings, Word};

use crate::identity::Socket;
use crate::words;

/// Which socket a screen that says nothing about itself is in, as a person
/// reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluggedInto {
    /// The machine's own screen, which has no socket anybody can see.
    BuiltIn,
    /// A DisplayPort socket, by its number.
    DisplayPort(u16),
    /// An HDMI socket, by its number.
    Hdmi(u16),
    /// A DVI socket, by its number.
    Dvi(u16),
    /// A VGA socket, by its number.
    Vga(u16),
    /// A socket of a kind alo OS has no word for, by its number.
    AnotherSocket(u16),
    /// A socket of a kind alo OS has no word for and no number for either.
    OneWithNoNumber,
}

impl PluggedInto {
    /// Which socket this is, from what the machine calls it.
    #[must_use]
    pub fn of(socket: &Socket) -> Self {
        let Some((kind, number)) = split(socket.name()) else {
            return Self::OneWithNoNumber;
        };
        if ["edp", "lvds", "dsi", "dpi"]
            .iter()
            .any(|built_in| kind.eq_ignore_ascii_case(built_in))
        {
            return Self::BuiltIn;
        }
        if kind.eq_ignore_ascii_case("dp") || kind.eq_ignore_ascii_case("displayport") {
            return Self::DisplayPort(number);
        }
        if ["hdmi", "hdmi-a", "hdmi-b"]
            .iter()
            .any(|hdmi| kind.eq_ignore_ascii_case(hdmi))
        {
            return Self::Hdmi(number);
        }
        if ["dvi", "dvi-a", "dvi-d", "dvi-i"]
            .iter()
            .any(|dvi| kind.eq_ignore_ascii_case(dvi))
        {
            return Self::Dvi(number);
        }
        if kind.eq_ignore_ascii_case("vga") {
            return Self::Vga(number);
        }
        Self::AnotherSocket(number)
    }

    /// The string this crate declares for this socket.
    #[must_use]
    pub const fn word(&self) -> Word {
        match *self {
            Self::BuiltIn => words::SCREEN_BUILT_IN,
            Self::DisplayPort(_) => words::SCREEN_AT_DISPLAYPORT,
            Self::Hdmi(_) => words::SCREEN_AT_HDMI,
            Self::Dvi(_) => words::SCREEN_AT_DVI,
            Self::Vga(_) => words::SCREEN_AT_VGA,
            Self::AnotherSocket(_) => words::SCREEN_AT_ANOTHER_SOCKET,
            Self::OneWithNoNumber => words::SCREEN_SOMEWHERE_ELSE,
        }
    }

    /// The socket's own number, where the machine gave one.
    #[must_use]
    pub const fn number(&self) -> Option<u16> {
        match *self {
            Self::DisplayPort(number)
            | Self::Hdmi(number)
            | Self::Dvi(number)
            | Self::Vga(number)
            | Self::AnotherSocket(number) => Some(number),
            Self::BuiltIn | Self::OneWithNoNumber => None,
        }
    }

    /// What this is called, in the language the person reads. Never fails and
    /// never panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self.number() {
            Some(number) => Filling::of("number", number.to_string()),
            None => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// A socket's kind and its number, where the name has both.
///
/// Split at the **last** hyphen, because the kinds the kernel's side of a
/// graphics card writes are themselves hyphenated: `HDMI-A-1` is the first
/// socket of kind `HDMI-A`, not the `A-1` of kind `HDMI`.
fn split(name: &str) -> Option<(&str, u16)> {
    let (kind, number) = name.rsplit_once('-')?;
    if kind.is_empty() {
        return None;
    }
    Some((kind, number.parse().ok()?))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// One socket, as the machine names it.
    fn socket(name: &str) -> Socket {
        Socket::named(name).unwrap()
    }

    /// **Every kind of socket a machine reports becomes something a person can
    /// read**, and the number is the socket's own.
    #[test]
    fn every_kind_of_socket_becomes_something_a_person_can_read() {
        for (named, expected) in [
            ("eDP-1", PluggedInto::BuiltIn),
            ("LVDS-1", PluggedInto::BuiltIn),
            ("DSI-2", PluggedInto::BuiltIn),
            ("DPI-1", PluggedInto::BuiltIn),
            ("DP-2", PluggedInto::DisplayPort(2)),
            ("DisplayPort-1", PluggedInto::DisplayPort(1)),
            ("HDMI-A-1", PluggedInto::Hdmi(1)),
            ("HDMI-B-3", PluggedInto::Hdmi(3)),
            ("HDMI-1", PluggedInto::Hdmi(1)),
            ("DVI-D-1", PluggedInto::Dvi(1)),
            ("DVI-I-2", PluggedInto::Dvi(2)),
            ("DVI-A-1", PluggedInto::Dvi(1)),
            ("DVI-1", PluggedInto::Dvi(1)),
            ("VGA-1", PluggedInto::Vga(1)),
            ("SVIDEO-1", PluggedInto::AnotherSocket(1)),
            ("Writeback-4", PluggedInto::AnotherSocket(4)),
        ] {
            assert_eq!(PluggedInto::of(&socket(named)), expected, "{named}");
        }
    }

    /// **A socket with no number at the end has no number in what a person
    /// reads either**, and is not given one that would be made up. Every
    /// connector the kernel's side of a graphics card names ends in a number;
    /// these are what is left when something else named it.
    #[test]
    fn a_socket_with_no_number_is_not_given_one() {
        for named in ["screen", "HDMI-A", "DP-", "-1", "DP-x", "DP-70000"] {
            assert_eq!(
                PluggedInto::of(&socket(named)),
                PluggedInto::OneWithNoNumber,
                "{named}"
            );
            assert_eq!(PluggedInto::of(&socket(named)).number(), None, "{named}");
        }
    }

    /// **Every one of them says a declared sentence with every gap filled**,
    /// and none of them is a key.
    #[test]
    fn every_one_of_them_says_a_declared_sentence() {
        let strings = in_english();
        for plugged in [
            PluggedInto::BuiltIn,
            PluggedInto::DisplayPort(2),
            PluggedInto::Hdmi(1),
            PluggedInto::Dvi(1),
            PluggedInto::Vga(1),
            PluggedInto::AnotherSocket(3),
            PluggedInto::OneWithNoNumber,
        ] {
            let said = plugged.said(&strings);
            assert!(words::EVERY_WORD.contains(&plugged.word()), "{plugged:?}");
            assert!(!said.is_a_bug(), "{plugged:?}: {said}");
            assert!(said.unfilled().is_empty(), "{plugged:?}: {said}");
            assert!(!said.text().is_empty(), "{plugged:?}");
        }
    }

    /// **A number the machine gave is the number a person reads**, so two HDMI
    /// sockets are told apart by being two.
    #[test]
    fn the_number_a_person_reads_is_the_sockets_own() {
        let strings = in_english();
        let one = PluggedInto::of(&socket("HDMI-A-1")).said(&strings);
        let two = PluggedInto::of(&socket("HDMI-A-2")).said(&strings);
        assert!(one.text().contains('1'), "{one}");
        assert!(two.text().contains('2'), "{two}");
        assert_ne!(one.text(), two.text());
    }

    /// **No connector name reaches what a person reads**, for any socket a
    /// machine could report — which is the whole reason this file exists.
    #[test]
    fn no_connector_name_reaches_what_a_person_reads() {
        let strings = in_english();
        for named in [
            "eDP-1",
            "LVDS-1",
            "DSI-1",
            "DPI-1",
            "DP-2",
            "HDMI-A-1",
            "HDMI-B-1",
            "DVI-D-1",
            "DVI-I-1",
            "VGA-1",
            "SVIDEO-1",
            "Writeback-1",
            "Virtual-1",
            "something-nobody-expected",
        ] {
            let said = PluggedInto::of(&socket(named)).said(&strings);
            assert!(
                !said.text().contains(named),
                "{named} reached a person: {said}"
            );
            assert!(
                !said.text().to_lowercase().contains("edp"),
                "{named}: {said}"
            );
        }
    }
}
