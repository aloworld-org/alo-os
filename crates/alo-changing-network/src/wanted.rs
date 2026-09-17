//! A change to the network somebody wants, and what it says it does to the
//! conversation's connection, before anybody has asked which network that is.

use alo_broker::Switch;

/// Which change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Join the visible Wi-Fi network called this.
    Join(String),
    /// Forget the saved Wi-Fi network called this.
    Forget(String),
    /// Turn Wi-Fi on, or off.
    Wireless(Switch),
}

impl Change {
    /// The network's name, for a change to one network.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        match self {
            Self::Join(called) | Self::Forget(called) => Some(called),
            Self::Wireless(_) => None,
        }
    }
}

/// What a change does to the connection this conversation is answered over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThisConversation {
    /// It keeps its connection.
    KeepsItsConnection,
    /// It loses its connection until the machine is connected again.
    LosesItsConnection,
}

impl ThisConversation {
    /// Both, in the order the verbs offer them.
    pub const EVERY: [Self; 2] = [Self::KeepsItsConnection, Self::LosesItsConnection];

    /// The name the verbs' option is chosen by.
    #[must_use]
    pub const fn option(self) -> &'static str {
        match self {
            Self::KeepsItsConnection => "keeps_its_connection",
            Self::LosesItsConnection => "loses_its_connection",
        }
    }

    /// The one the option of this name is.
    #[must_use]
    pub fn chosen(option: &str) -> Option<Self> {
        Self::EVERY.into_iter().find(|each| each.option() == option)
    }
}

/// A change, and what it says it does to this conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// Which change.
    change: Change,
    /// What it says it does to this conversation's connection.
    says: ThisConversation,
}

impl Wanted {
    /// This change, saying this about the conversation.
    #[must_use]
    pub const fn of(change: Change, says: ThisConversation) -> Self {
        Self { change, says }
    }

    /// Which change.
    #[must_use]
    pub const fn change(&self) -> &Change {
        &self.change
    }

    /// What it says it does to this conversation's connection.
    #[must_use]
    pub const fn says(&self) -> ThisConversation {
        self.says
    }
}
