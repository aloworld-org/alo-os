//! The object the rented service asks when a device wants something, served in
//! the person's own session.
//!
//! BlueZ does not decide what to do about a passkey; it asks an *agent*. This is
//! ours. Every one of its methods hands the question to
//! [`crate::asking_the_person::Asking`] and answers with what the person said —
//! **there is no branch here that answers without asking**, which is why this
//! file is a shell and the decisions are next door where a test can reach them.
//!
//! # It is registered, and it is the only one
//!
//! `RegisterAgent` with what this machine can do — show six digits and take six
//! digits — and then `RequestDefaultAgent`, so that a pairing started anywhere
//! reaches a person rather than whatever else registered. A machine whose agent
//! is not the default is a machine where something else is answering for
//! somebody.
//!
//! # Authorising a service is a person's answer too
//!
//! `AuthorizeService` is asked when a device that is already paired wants to use
//! one of the things it can be — a headset that wants to be a headset. It is a
//! person's answer for the same reason the pairing was: what the device is *for*
//! is decided when it is paired, and anything past that is asked.

use zbus::fdo;
use zbus::zvariant::ObjectPath;

use crate::asking_the_person::{Asking, Refused, ThePersonsOwnSurface};
use crate::reported::Found;

/// **How this agent finds out which device a question is about.**
///
/// The service asks about an object in its own tree; a person is shown a device
/// with a name. This is the one step between, and a question about a device this
/// machine cannot name is refused rather than shown as a blank.
pub trait WhichDevice {
    /// The device at this place in the service's tree.
    fn at(&self, path: &str) -> Option<Found>;
}

/// **alo OS's pairing agent.**
pub struct PairingAgent {
    /// Where the person is asked.
    surface: Box<dyn ThePersonsOwnSurface + Send + Sync>,
    /// Which device a question is about.
    which: Box<dyn WhichDevice + Send + Sync>,
}

impl PairingAgent {
    /// An agent asking this person about devices named by this.
    #[must_use]
    pub fn for_the_person(
        surface: Box<dyn ThePersonsOwnSurface + Send + Sync>,
        which: Box<dyn WhichDevice + Send + Sync>,
    ) -> Self {
        Self { surface, which }
    }

    /// The device a question is about, or a refusal.
    fn device(&self, path: &ObjectPath<'_>) -> fdo::Result<Found> {
        self.which.at(path.as_str()).ok_or_else(|| {
            fdo::Error::Failed(format!(
                "a device this machine cannot name asked to pair: {path}"
            ))
        })
    }

    /// What a person's *no* is, in the service's own words.
    fn no(_refused: Refused) -> fdo::Error {
        fdo::Error::Failed("org.bluez.Error.Rejected".to_owned())
    }
}

#[zbus::interface(name = "org.bluez.Agent1")]
impl PairingAgent {
    /// The service is done with this agent.
    fn release(&self) {}

    /// A device wants the code printed on it.
    fn request_pin_code(&self, device: ObjectPath<'_>) -> fdo::Result<String> {
        let device = self.device(&device)?;
        Asking::through(self.surface.as_ref())
            .its_own_code(&device)
            .map_err(Self::no)
    }

    /// A device wants six digits, which a person reads off it.
    fn request_passkey(&self, device: ObjectPath<'_>) -> fdo::Result<u32> {
        let device = self.device(&device)?;
        let typed = Asking::through(self.surface.as_ref())
            .its_own_code(&device)
            .map_err(Self::no)?;
        typed
            .parse::<u32>()
            .map_err(|why| fdo::Error::Failed(format!("that is not six digits: {why}")))
    }

    /// Six digits to be typed on the device.
    fn display_passkey(
        &self,
        device: ObjectPath<'_>,
        passkey: u32,
        _typed: u16,
    ) -> fdo::Result<()> {
        let device = self.device(&device)?;
        Asking::through(self.surface.as_ref())
            .type_this_on_it(&device, passkey)
            .map_err(Self::no)
    }

    /// A code to be typed on the device.
    fn display_pin_code(&self, device: ObjectPath<'_>, code: String) -> fdo::Result<()> {
        let device = self.device(&device)?;
        let six = code.parse::<u32>().unwrap_or_default();
        Asking::through(self.surface.as_ref())
            .type_this_on_it(&device, six)
            .map_err(Self::no)
    }

    /// Six digits shown at both ends, which a person compares.
    fn request_confirmation(&self, device: ObjectPath<'_>, passkey: u32) -> fdo::Result<()> {
        let device = self.device(&device)?;
        Asking::through(self.surface.as_ref())
            .the_same_on_both(&device, passkey)
            .map_err(Self::no)
    }

    /// A device asking to pair with nothing to show.
    fn request_authorization(&self, device: ObjectPath<'_>) -> fdo::Result<()> {
        let device = self.device(&device)?;
        Asking::through(self.surface.as_ref())
            .pair_with_this(&device)
            .map_err(Self::no)
    }

    /// A paired device asking to be what it is.
    fn authorize_service(&self, device: ObjectPath<'_>, _service: String) -> fdo::Result<()> {
        let device = self.device(&device)?;
        Asking::through(self.surface.as_ref())
            .pair_with_this(&device)
            .map_err(Self::no)
    }

    /// The service gave up on a question. Nothing was answered, which is the
    /// same as nothing having been asked.
    fn cancel(&self) {}
}

/// **Serve this agent and make it the machine's**, on the system bus.
///
/// # Errors
/// Whatever the bus said, in English for a service log.
pub fn registered(agent: PairingAgent) -> Result<zbus::blocking::Connection, String> {
    use crate::bus::{AGENT_MANAGER, OUR_AGENT_AT, THE_SERVICE_AT, WHAT_THIS_MACHINE_CAN_DO, call};

    let bus = zbus::blocking::Connection::system()
        .map_err(|why| crate::bus::failed("the system bus", &why))?;
    bus.object_server()
        .at(OUR_AGENT_AT, agent)
        .map_err(|why| crate::bus::failed("serving the pairing agent", &why))?;
    let at = zbus::zvariant::ObjectPath::try_from(OUR_AGENT_AT)
        .map_err(|why| crate::bus::failed("the agent's own place", &why))?;
    call::<(), _>(
        &bus,
        THE_SERVICE_AT,
        AGENT_MANAGER,
        "RegisterAgent",
        &(&at, WHAT_THIS_MACHINE_CAN_DO),
    )?;
    call::<(), _>(
        &bus,
        THE_SERVICE_AT,
        AGENT_MANAGER,
        "RequestDefaultAgent",
        &(&at,),
    )?;
    Ok(bus)
}
