//! The rented power daemon, reached.
//!
//! [`ThePowerDaemon`] is [`Profiles`] on a real machine. The daemon is rented
//! and unmodified (ADR 0011), and what is here is a client for the interface it
//! publishes: which profiles this machine's hardware can do, which one it is in,
//! and setting it.
//!
//! # Its own names, its own list
//!
//! The list of profiles is read from the daemon rather than declared here. A
//! machine whose platform driver cannot do *performance* is a machine with two
//! profiles, and the page shows two — which is the honest answer and the one a
//! person can act on.
//!
//! A name the daemon offers that this crate does not know is **passed over**
//! rather than guessed at or shown untranslated: a profile alo OS has no
//! sentence for is a profile alo OS cannot put in front of anybody.

use zbus::blocking::Connection;
use zbus::zvariant::{OwnedValue, Value};

use crate::profiles::{NotAProfile, Profile, TheProfiles};

/// The name the daemon owns on the system bus.
pub const THE_DAEMON: &str = "net.hadess.PowerProfiles";

/// Its object.
pub const THE_DAEMON_AT: &str = "/net/hadess/PowerProfiles";

/// The standard properties interface.
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// **The power profiles this machine has**, asked and set.
pub trait Profiles {
    /// What this machine can do and what it is doing.
    ///
    /// # Errors
    /// [`NotAsked`] where there is no daemon, or it would not answer.
    fn now(&self) -> Result<TheProfiles, NotAsked>;

    /// **Put this machine in this profile.**
    ///
    /// # Errors
    /// [`NotAsked`] where the machine does not have it, or would not.
    fn choose(&self, profile: Profile) -> Result<(), NotAsked>;
}

/// Why the daemon could not be asked or told.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAsked {
    /// Nothing on this machine answers for power profiles.
    #[error("nothing on this machine sets power profiles: {0}")]
    NothingAnswers(String),
    /// It is there and it would not answer.
    #[error("this machine's power daemon would not answer: {0}")]
    NoAnswer(String),
    /// It answered something this crate cannot read.
    #[error("this machine's power daemon answered something this crate cannot read: {0}")]
    NotUnderstood(String),
    /// A profile this machine does not have.
    #[error(transparent)]
    NotAProfile(#[from] NotAProfile),
}

/// **This machine's power daemon.**
pub struct ThePowerDaemon {
    /// The system bus.
    bus: Connection,
}

impl ThePowerDaemon {
    /// The daemon on the machine this is running on.
    ///
    /// # Errors
    /// [`NotAsked::NothingAnswers`] where there is no system bus.
    pub fn on_this_machine() -> Result<Self, NotAsked> {
        let bus = Connection::system()
            .map_err(|why| NotAsked::NothingAnswers(format!("the system bus: {why}")))?;
        Ok(Self { bus })
    }

    /// One of the daemon's properties.
    fn property(&self, named: &str) -> Result<OwnedValue, NotAsked> {
        self.bus
            .call_method(
                Some(THE_DAEMON),
                THE_DAEMON_AT,
                Some(PROPERTIES),
                "Get",
                &(THE_DAEMON, named),
            )
            .map_err(|why| {
                let said = why.to_string();
                if said.contains("ServiceUnknown") || said.contains("was not provided") {
                    return NotAsked::NothingAnswers(said);
                }
                NotAsked::NoAnswer(said)
            })?
            .body()
            .deserialize::<OwnedValue>()
            .map_err(|why| NotAsked::NotUnderstood(format!("{named}: {why}")))
    }
}

impl Profiles for ThePowerDaemon {
    fn now(&self) -> Result<TheProfiles, NotAsked> {
        let on = Profile::by_name(
            &String::try_from(self.property("ActiveProfile")?)
                .map_err(|why| NotAsked::NotUnderstood(format!("ActiveProfile: {why}")))?,
        )
        .unwrap_or(Profile::Balanced);

        let offered = self.property("Profiles")?;
        let offered: Vec<std::collections::HashMap<String, OwnedValue>> = Vec::try_from(offered)
            .map_err(|why| NotAsked::NotUnderstood(format!("the list of profiles: {why}")))?;
        let offered: Vec<Profile> = offered
            .iter()
            .filter_map(|one| {
                let named = one.get("Profile")?;
                Profile::by_name(&String::try_from(named.try_clone().ok()?).ok()?)
            })
            .collect();
        Ok(TheProfiles::reported(offered, on)?)
    }

    fn choose(&self, profile: Profile) -> Result<(), NotAsked> {
        self.now()?.may_choose(profile)?;
        self.bus
            .call_method(
                Some(THE_DAEMON),
                THE_DAEMON_AT,
                Some(PROPERTIES),
                "Set",
                &(
                    THE_DAEMON,
                    "ActiveProfile",
                    Value::from(profile.named().to_owned()),
                ),
            )
            .map(drop)
            .map_err(|why| NotAsked::NoAnswer(why.to_string()))
    }
}
