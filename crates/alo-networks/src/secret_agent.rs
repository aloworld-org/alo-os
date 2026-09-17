//! Where a Wi-Fi password is asked of a person, and handed to the network
//! manager and nobody else.
//!
//! The broker plan: **a Wi-Fi password never passes through the agent** —
//! joining a protected network asks the person for it in a surface the agent
//! cannot read. This is the half of that which is the network manager's
//! protocol. The network manager, asked to join a network it has no password
//! for, calls `GetSecrets` on the secret agents registered on the system bus.
//! The one alo OS registers runs **in the person's own session** (the shell
//! serves it), asks the person through [`ThePersonsOwnSurface`], and answers
//! the network manager with what they typed.
//!
//! # Why the agent cannot read it
//!
//! - **It never crosses anything the agent reaches.** Not the broker's door,
//!   which takes a verb and an identity; not a verb's arguments, which have no
//!   field for one; not the record. It goes from the person's surface to this
//!   object to the network manager's connection, inside the person's session.
//! - **Only the network manager is answered.** `GetSecrets` is refused unless
//!   the caller is the connection that owns the network manager's name on the
//!   bus **and** the bus says it runs as the network manager's user. Without
//!   this, anything that can send to the person's connection — an agent's own
//!   login among them — could make the machine ask the person for their Wi-Fi
//!   password and be handed the answer.
//! - **A person is never asked uninvited.** The network manager says whether it
//!   may interact with the person; when it may not, nothing is asked.
//!
//! # What is answered, and what is not
//!
//! A password network's key (`802-11-wireless-security`, `psk`). An
//! organisation's sign-in (802.1X) is not set up on this machine in v0.5, and is
//! answered *no secrets* rather than asked for. The secrets the network manager
//! keeps are its own: this agent saves and deletes nothing, because it keeps
//! nothing.
//!
//! # What it does not yet do
//!
//! The surface itself is the shell's to draw, and calling `registered` in
//! the person's session at sign-in is the shell's to do. A surface that takes
//! a person minutes to answer holds this object's dispatch while it does, so a
//! `CancelGetSecrets` arrives after the answer rather than during it.

use std::collections::HashMap;

use zbus::blocking::Connection;
use zbus::message::Header;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::bus::{AGENT_MANAGER, AGENT_MANAGER_AT, SECRET_AGENT_AT, THE_NETWORK_MANAGER, call};
use crate::password::WifiPassword;
use crate::reported::NetworkName;

/// The identifier this agent registers under.
pub const THE_AGENTS_IDENTIFIER: &str = "world.alo.networks";

/// The user the network manager runs as on a machine.
pub const THE_NETWORK_MANAGERS_USER: u32 = 0;

/// `NM_SECRET_AGENT_GET_SECRETS_FLAG_ALLOW_INTERACTION`.
const MAY_ASK: u32 = 0x1;

/// The settings group a password network's key is in.
const WIFI_SECURITY: &str = "802-11-wireless-security";

/// The settings group a Wi-Fi network's name is in.
const WIFI: &str = "802-11-wireless";

/// A connection's settings, as the network manager passes them.
type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// Where the person is asked, in their own session.
pub trait ThePersonsOwnSurface: Send + Sync {
    /// Ask the person for the password of the network with this name, and say
    /// what they typed — or [`None`] if they declined.
    fn password_for(&self, network: &NetworkName) -> Option<WifiPassword>;
}

/// The answers the network manager's protocol names.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.NetworkManager.SecretAgent")]
pub enum NotGiven {
    /// Something on the bus itself went wrong.
    #[zbus(error)]
    ZBus(zbus::Error),
    /// The person declined.
    UserCanceled(String),
    /// Nothing is given: the caller is not the network manager, the person may
    /// not be asked, or the secret is not one this agent gives.
    NoSecrets(String),
}

/// The secret agent, as it is served.
pub struct SecretAgent {
    /// Where the person is asked.
    surface: Box<dyn ThePersonsOwnSurface>,
    /// The user the network manager runs as.
    network_managers_user: u32,
}

impl SecretAgent {
    /// An agent asking the person through `surface`, answering only the network
    /// manager running as root.
    #[must_use]
    pub fn for_the_person(surface: Box<dyn ThePersonsOwnSurface>) -> Self {
        Self::answering(surface, THE_NETWORK_MANAGERS_USER)
    }

    /// An agent answering only a network manager running as this user — how a
    /// test's own stands in for the machine's.
    #[must_use]
    pub fn answering(surface: Box<dyn ThePersonsOwnSurface>, network_managers_user: u32) -> Self {
        Self {
            surface,
            network_managers_user,
        }
    }

    /// Whether the caller is the network manager: the owner of its name, running
    /// as its user.
    async fn asked_by_the_network_manager(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> bool {
        let Some(sender) = header.sender() else {
            return false;
        };
        let owner: Option<String> =
            asked_of_the_bus(connection, "GetNameOwner", THE_NETWORK_MANAGER).await;
        let user: Option<u32> =
            asked_of_the_bus(connection, "GetConnectionUnixUser", sender.as_str()).await;
        matches!((owner, user), (Some(owner), Some(user))
            if owner == sender.as_str() && user == self.network_managers_user)
    }
}

/// Ask the bus itself one question about a name, and read the answer as `T`.
///
/// A plain method call rather than a proxy: a proxy to the bus is built with
/// signal subscriptions of its own, and none is wanted inside a handler that
/// only needs two answers.
async fn asked_of_the_bus<T>(connection: &zbus::Connection, method: &str, about: &str) -> Option<T>
where
    T: for<'d> zbus::export::serde::Deserialize<'d> + zbus::zvariant::Type,
{
    connection
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            method,
            &(about,),
        )
        .await
        .ok()?
        .body()
        .deserialize::<T>()
        .ok()
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.SecretAgent")]
impl SecretAgent {
    /// The network manager asks for a connection's secrets.
    #[expect(
        clippy::too_many_arguments,
        reason = "the network manager's protocol names five arguments, and who is asking is \
                  read from the message's header and its connection"
    )]
    async fn get_secrets(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] bus: &zbus::Connection,
        connection: Settings,
        _connection_path: OwnedObjectPath,
        setting_name: String,
        _hints: Vec<String>,
        flags: u32,
    ) -> Result<Settings, NotGiven> {
        if !self.asked_by_the_network_manager(bus, &header).await {
            return Err(NotGiven::NoSecrets(
                "only the network manager is answered".to_owned(),
            ));
        }
        if setting_name != WIFI_SECURITY {
            return Err(NotGiven::NoSecrets(
                "only a Wi-Fi network's password is asked for here".to_owned(),
            ));
        }
        if flags & MAY_ASK == 0 {
            return Err(NotGiven::NoSecrets(
                "the person may not be asked now".to_owned(),
            ));
        }
        let Some(name) = connection
            .get(WIFI)
            .and_then(|wifi| wifi.get("ssid"))
            .and_then(|ssid| ssid.try_clone().ok())
            .and_then(|ssid| Vec::<u8>::try_from(ssid).ok())
            .and_then(|ssid| NetworkName::announced(&ssid))
        else {
            return Err(NotGiven::NoSecrets(
                "a connection with no network name".to_owned(),
            ));
        };
        let Some(password) = self.surface.password_for(&name) else {
            return Err(NotGiven::UserCanceled("the person declined".to_owned()));
        };
        let psk = OwnedValue::try_from(Value::from(password.handed_to_the_network_manager()))
            .map_err(|why| NotGiven::NoSecrets(why.to_string()))?;
        Ok(HashMap::from([(
            WIFI_SECURITY.to_owned(),
            HashMap::from([("psk".to_owned(), psk)]),
        )]))
    }

    /// The network manager no longer needs what it asked for.
    async fn cancel_get_secrets(&self, _connection_path: OwnedObjectPath, _setting_name: String) {}

    /// Nothing is kept here, so nothing is saved.
    async fn save_secrets(&self, _connection: Settings, _connection_path: OwnedObjectPath) {}

    /// Nothing is kept here, so nothing is deleted.
    async fn delete_secrets(&self, _connection: Settings, _connection_path: OwnedObjectPath) {}
}

/// Connect to the bus at `address` with this agent already served, and register
/// it with the network manager there.
///
/// The agent is served as the connection is built, and registered only after:
/// the network manager calls an agent it knows of, and an object added to a
/// connection that is already answering can miss a call that arrives while it
/// is being added. On a machine `address` is
/// [`crate::network_manager::THE_SYSTEM_BUS`], and the connection handed back is
/// what keeps the agent registered: dropping it unregisters the agent.
///
/// # Errors
/// What the bus or the network manager said, in English for a log.
pub fn registered(address: &str, agent: SecretAgent) -> Result<Connection, String> {
    let bus = zbus::blocking::connection::Builder::address(address)
        .and_then(|building| building.serve_at(SECRET_AGENT_AT, agent))
        .and_then(zbus::blocking::connection::Builder::build)
        .map_err(|why| why.to_string())?;
    call::<(), _>(
        &bus,
        AGENT_MANAGER_AT,
        AGENT_MANAGER,
        "Register",
        &(THE_AGENTS_IDENTIFIER,),
    )?;
    Ok(bus)
}
