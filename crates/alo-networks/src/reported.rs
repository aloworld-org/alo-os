//! What the network manager reported: the networks it can see, the ones this
//! machine has saved, whether the radio is on, and which connection the machine
//! is using.
//!
//! Every value here is made from a report and from nothing else, and each is
//! compared by the bytes it was reported under ([`Visible::as_reported`],
//! [`Saved::as_reported`]). A name is shown to a person and matched against the
//! name they approved; it is never handed back to the network manager as an
//! instruction.

/// The most bytes a Wi-Fi network's name may be (IEEE 802.11).
pub const LONGEST_NAME: usize = 32;

/// What every visible network's identity begins with, so it can never be
/// mistaken for a saved one's or for anything else digested on this machine.
const A_VISIBLE_NETWORK: &[u8] = b"alo-networks visible 1\0";

/// What every saved network's identity begins with.
const A_SAVED_NETWORK: &[u8] = b"alo-networks saved 1\0";

/// A Wi-Fi network's name, exactly the bytes it announced.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NetworkName(Vec<u8>);

impl NetworkName {
    /// The name a network announced, or nothing when it announced none or more
    /// than a name may be — a network with no name is one a person cannot be
    /// asked about.
    #[must_use]
    pub fn announced(bytes: &[u8]) -> Option<Self> {
        (!bytes.is_empty() && bytes.len() <= LONGEST_NAME).then(|| Self(bytes.to_vec()))
    }

    /// The bytes it announced.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// What a person reads, when the name is text a person can read: UTF-8,
    /// with no control character. A name that is not is still a network — it
    /// can be picked in Settings by what it shows there — and it is simply one
    /// no sentence can name.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        std::str::from_utf8(&self.0)
            .ok()
            .filter(|text| !text.chars().any(char::is_control))
    }
}

/// How a visible network is protected, as far as joining it is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Protection {
    /// Anybody may join, with nothing to type.
    Open,
    /// Joining asks for a password.
    Password,
    /// Joining asks for an organisation's sign-in, which this machine does not
    /// set up in v0.5.
    Enterprise,
}

impl Protection {
    /// The one byte a network's identity carries for this.
    const fn tag(self) -> u8 {
        match self {
            Self::Open => 1,
            Self::Password => 2,
            Self::Enterprise => 3,
        }
    }
}

/// A network the machine can see now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visible {
    /// Its name.
    name: NetworkName,
    /// How it is protected.
    protection: Protection,
    /// How strong it is, from 0 to 100.
    strength: u8,
    /// Where the network manager keeps the access point and the device that
    /// sees it, when this came from the network manager.
    pub(crate) at: Option<AccessPointAt>,
}

/// Where an access point is on the bus, and the device that sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccessPointAt {
    /// The access point's object.
    pub(crate) access_point: String,
    /// The wireless device's object.
    pub(crate) device: String,
}

impl Visible {
    /// A network reported under this name and protection, at this strength.
    #[must_use]
    pub fn reported(name: NetworkName, protection: Protection, strength: u8) -> Self {
        Self {
            name,
            protection,
            strength: strength.min(100),
            at: None,
        }
    }

    /// Its name.
    #[must_use]
    pub const fn name(&self) -> &NetworkName {
        &self.name
    }

    /// How it is protected.
    #[must_use]
    pub const fn protection(&self) -> Protection {
        self.protection
    }

    /// How strong it is, from 0 to 100.
    #[must_use]
    pub const fn strength(&self) -> u8 {
        self.strength
    }

    /// The bytes its identity is digested from: its name and how it is
    /// protected. Not the access point: a network seen through two access
    /// points is one network, and moving between them is not a different
    /// approval.
    #[must_use]
    pub fn as_reported(&self) -> Vec<u8> {
        [
            A_VISIBLE_NETWORK,
            &[self.protection.tag()],
            self.name.bytes(),
        ]
        .concat()
    }
}

/// A Wi-Fi network this machine has saved, and joins on its own when it sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    /// Its name.
    name: NetworkName,
    /// The network manager's own identifier for the saved connection.
    uuid: String,
    /// Where the network manager keeps it on the bus, when this came from the
    /// network manager.
    pub(crate) at: Option<String>,
}

impl Saved {
    /// A saved network reported under this name and identifier.
    #[must_use]
    pub fn reported(name: NetworkName, uuid: &str) -> Self {
        Self {
            name,
            uuid: uuid.to_owned(),
            at: None,
        }
    }

    /// Its name.
    #[must_use]
    pub const fn name(&self) -> &NetworkName {
        &self.name
    }

    /// The network manager's identifier for it.
    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// The bytes its identity is digested from: the network manager's own
    /// identifier, which two saved networks never share even when their names
    /// are the same.
    #[must_use]
    pub fn as_reported(&self) -> Vec<u8> {
        [A_SAVED_NETWORK, self.uuid.as_bytes()].concat()
    }
}

/// The connection the machine sends through now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Primary {
    /// Whether it is over the wireless radio.
    over_wireless: bool,
    /// The saved connection it is, by the network manager's identifier.
    uuid: String,
}

impl Primary {
    /// The connection reported as the one the machine sends through.
    #[must_use]
    pub fn reported(over_wireless: bool, uuid: &str) -> Self {
        Self {
            over_wireless,
            uuid: uuid.to_owned(),
        }
    }

    /// Whether it is over the wireless radio.
    #[must_use]
    pub const fn over_wireless(&self) -> bool {
        self.over_wireless
    }

    /// The saved connection it is.
    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }
}

/// Everything the network manager reported at one moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheNetworks {
    /// The networks it can see.
    pub visible: Vec<Visible>,
    /// The networks this machine has saved.
    pub saved: Vec<Saved>,
    /// Whether the wireless radio is on.
    pub wireless_on: bool,
    /// The connection the machine sends through, if it has one.
    pub primary: Option<Primary>,
}

impl TheNetworks {
    /// The saved network the machine is sending through now, if it is one.
    #[must_use]
    pub fn joined(&self) -> Option<&Saved> {
        let primary = self.primary.as_ref()?;
        self.saved
            .iter()
            .find(|saved| saved.uuid() == primary.uuid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name, for a test.
    fn named(name: &str) -> NetworkName {
        NetworkName::announced(name.as_bytes()).unwrap_or_else(|| NetworkName(Vec::new()))
    }

    /// **The same name, protected differently, is a different network** — so
    /// an open network calling itself by a protected one's name is never the
    /// one a person approved.
    #[test]
    fn a_network_is_its_name_and_how_it_is_protected() {
        let home = Visible::reported(named("Home"), Protection::Password, 70);
        let twin = Visible::reported(named("Home"), Protection::Open, 99);
        let again = Visible::reported(named("Home"), Protection::Password, 10);
        assert_ne!(home.as_reported(), twin.as_reported());
        assert_eq!(home.as_reported(), again.as_reported());
    }

    /// A visible network and a saved one never report the same bytes, whatever
    /// they are called.
    #[test]
    fn visible_and_saved_are_never_the_same_bytes() {
        let saved = Saved::reported(named("Home"), "Home");
        let visible = Visible::reported(named("Home"), Protection::Open, 50);
        assert_ne!(saved.as_reported(), visible.as_reported());
    }

    /// **A name is what was announced**, no longer than a name may be, and a
    /// person reads it only when it is text.
    #[test]
    fn a_name_is_bytes_and_is_read_only_when_it_is_text() {
        assert!(NetworkName::announced(b"").is_none());
        assert!(NetworkName::announced(&[b'x'; LONGEST_NAME + 1]).is_none());
        assert_eq!(named("Café").called(), Some("Café"));
        assert_eq!(
            NetworkName::announced(&[0xff, 0xfe]).and_then(|n| n.called().map(str::to_owned)),
            None
        );
        assert_eq!(
            NetworkName::announced(b"line\nbreak").and_then(|n| n.called().map(str::to_owned)),
            None
        );
    }

    /// The saved network being sent through is found by its identifier.
    #[test]
    fn the_network_joined_is_the_saved_one_the_machine_sends_through() {
        let networks = TheNetworks {
            visible: Vec::new(),
            saved: vec![
                Saved::reported(named("Home"), "a"),
                Saved::reported(named("Office"), "b"),
            ],
            wireless_on: true,
            primary: Some(Primary::reported(true, "b")),
        };
        assert_eq!(networks.joined().map(Saved::uuid), Some("b"));
        let unplugged = TheNetworks {
            primary: None,
            ..networks
        };
        assert!(unplugged.joined().is_none());
    }
}
