//! The rented network manager, asked and told over the system bus.
//!
//! Configured, never patched (ADR 0011): this speaks NetworkManager's published
//! D-Bus interface and nothing else. It asks for the whole picture at once
//! ([`Networks::now`]) and makes the three changes the broker carries out, each
//! against an object the network manager itself reported a moment before.
//!
//! # The system bus is named, never looked up in an environment
//!
//! [`THE_SYSTEM_BUS`] is the address systemd gives the system bus. A process
//! that read `DBUS_SYSTEM_BUS_ADDRESS` instead would be a privileged process
//! whose environment chose which network manager it obeyed — `alo-secrets`
//! refuses an environment for the same reason.
//!
//! # Joining is answered when the network is joined, not when it was asked for
//!
//! The network manager accepts an activation at once and carries it out
//! afterwards. A broker that answered `carried` at that moment would tell a
//! person they were connected while the password was still being asked for, so
//! [`NetworkManager::join`] waits until the connection is activated, or has
//! failed, for as long as [`JOINING`] and no longer.
//!
//! # What a change is made against
//!
//! - **join** — the access point and the device the network manager reported
//!   for the visible network. A saved connection of the same name is activated
//!   rather than a second one added; otherwise the network manager is asked to
//!   add one from the access point with **no settings of ours at all**, so no
//!   password, and nothing this machine wrote, is in the request.
//! - **forget** — the saved connection object reported for that network.
//! - **the radio** — the manager's own `WirelessEnabled`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zbus::blocking::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

use crate::bus::{
    A_WIRELESS_DEVICE, ACCESS_POINT, ACTIVATED, ACTIVE, CONNECTION, DEACTIVATED, DEVICE, MANAGER,
    MANAGER_AT, SETTINGS, SETTINGS_AT, WIRELESS, call, property, set,
};
use crate::reaching::{HowFar, Metered, Reaching, WhatIsReached};
use crate::reported::{
    AccessPointAt, NetworkName, Primary, Protection, Saved, TheNetworks, Visible,
};
use crate::service::{NetworkService, Networks, NotAnswering, NotDone};

/// The system bus, as systemd makes it.
pub const THE_SYSTEM_BUS: &str = "unix:path=/run/dbus/system_bus_socket";

/// How long joining a network may take before it is said not to have happened.
pub const JOINING: Duration = Duration::from_secs(90);

/// How often a join in progress is looked at.
const LOOKING: Duration = Duration::from_millis(250);

/// How long one question to the network manager may take. A network manager
/// that stops answering is a change not made, never a broker that waits for ever.
const ANSWERING: Duration = Duration::from_secs(30);

/// `NM_802_11_AP_FLAGS_PRIVACY`.
const PRIVACY: u32 = 0x1;
/// `NM_802_11_AP_SEC_KEY_MGMT_PSK` and `…_SAE`: a password.
const A_PASSWORD: u32 = 0x100 | 0x400;
/// `NM_802_11_AP_SEC_KEY_MGMT_802_1X` and `…_EAP_SUITE_B_192`: a sign-in.
const A_SIGN_IN: u32 = 0x200 | 0x2000;

/// The type a saved Wi-Fi connection has.
const WIFI: &str = "802-11-wireless";

/// A saved connection's settings, as the network manager hands them over.
type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// The network manager on one bus.
#[derive(Debug)]
pub struct NetworkManager {
    /// The bus.
    bus: Connection,
    /// How long a join may take.
    joining: Duration,
}

impl NetworkManager {
    /// The network manager on this machine's system bus.
    ///
    /// # Errors
    /// [`NotAnswering`] when the system bus cannot be reached.
    pub fn on_this_machine() -> Result<Self, NotAnswering> {
        Self::at(THE_SYSTEM_BUS)
    }

    /// The network manager on the bus at this address — how a test reaches one
    /// of its own.
    ///
    /// # Errors
    /// [`NotAnswering`] when that bus cannot be reached.
    pub fn at(address: &str) -> Result<Self, NotAnswering> {
        let bus = zbus::blocking::connection::Builder::address(address)
            .map(|building| building.method_timeout(ANSWERING))
            .and_then(zbus::blocking::connection::Builder::build)
            .map_err(|why| NotAnswering(why.to_string()))?;
        Ok(Self {
            bus,
            joining: JOINING,
        })
    }

    /// The same, waiting this long for a join rather than [`JOINING`].
    #[must_use]
    pub fn waiting(self, joining: Duration) -> Self {
        Self { joining, ..self }
    }

    /// The wireless device, if the machine has one.
    fn wireless_device(&self) -> Result<Option<OwnedObjectPath>, String> {
        let devices: Vec<OwnedObjectPath> =
            call(&self.bus, MANAGER_AT, MANAGER, "GetDevices", &())?;
        for device in devices {
            let kind: u32 = property(&self.bus, device.as_str(), DEVICE, "DeviceType")?;
            if kind == A_WIRELESS_DEVICE {
                return Ok(Some(device));
            }
        }
        Ok(None)
    }

    /// Every network the wireless device can see, one per name and protection,
    /// through its strongest access point.
    fn visible(&self, device: &OwnedObjectPath) -> Result<Vec<Visible>, String> {
        let points: Vec<OwnedObjectPath> = call(
            &self.bus,
            device.as_str(),
            WIRELESS,
            "GetAllAccessPoints",
            &(),
        )?;
        let mut strongest: Vec<Visible> = Vec::new();
        for point in points {
            let at = point.as_str();
            let name: Vec<u8> = property(&self.bus, at, ACCESS_POINT, "Ssid")?;
            // A network announcing no name is one nobody can be asked about.
            let Some(name) = NetworkName::announced(&name) else {
                continue;
            };
            let flags: u32 = property(&self.bus, at, ACCESS_POINT, "Flags")?;
            let wpa: u32 = property(&self.bus, at, ACCESS_POINT, "WpaFlags")?;
            let rsn: u32 = property(&self.bus, at, ACCESS_POINT, "RsnFlags")?;
            let strength: u8 = property(&self.bus, at, ACCESS_POINT, "Strength")?;
            let mut seen = Visible::reported(name, protection(flags, wpa | rsn), strength);
            seen.at = Some(AccessPointAt {
                access_point: at.to_owned(),
                device: device.as_str().to_owned(),
            });
            match strongest
                .iter_mut()
                .find(|kept| kept.as_reported() == seen.as_reported())
            {
                Some(kept) if kept.strength() < seen.strength() => *kept = seen,
                Some(_) => {}
                None => strongest.push(seen),
            }
        }
        Ok(strongest)
    }

    /// Every saved Wi-Fi network.
    fn saved(&self) -> Result<Vec<Saved>, String> {
        let connections: Vec<OwnedObjectPath> =
            call(&self.bus, SETTINGS_AT, SETTINGS, "ListConnections", &())?;
        let mut saved = Vec::new();
        for at in connections {
            let settings: Settings = call(&self.bus, at.as_str(), CONNECTION, "GetSettings", &())?;
            if text(&settings, "connection", "type").as_deref() != Some(WIFI) {
                continue;
            }
            let (Some(uuid), Some(name)) = (
                text(&settings, "connection", "uuid"),
                bytes(&settings, WIFI, "ssid").and_then(|ssid| NetworkName::announced(&ssid)),
            ) else {
                continue;
            };
            let mut one = Saved::reported(name, &uuid);
            one.at = Some(at.as_str().to_owned());
            saved.push(one);
        }
        Ok(saved)
    }

    /// The connection the machine sends through, if it has one.
    fn primary(&self) -> Result<Option<Primary>, String> {
        let active: OwnedObjectPath =
            property(&self.bus, MANAGER_AT, MANAGER, "PrimaryConnection")?;
        if active.as_str() == "/" {
            return Ok(None);
        }
        let kind: String = property(&self.bus, active.as_str(), ACTIVE, "Type")?;
        let uuid: String = property(&self.bus, active.as_str(), ACTIVE, "Uuid")?;
        Ok(Some(Primary::reported(kind == WIFI, &uuid)))
    }

    /// Wait for a connection being activated to be activated, or to fail.
    fn joined(&self, active: &OwnedObjectPath) -> Result<(), NotDone> {
        let until = Instant::now() + self.joining;
        loop {
            let state: u32 =
                property(&self.bus, active.as_str(), ACTIVE, "State").map_err(NotDone)?;
            if state == ACTIVATED {
                return Ok(());
            }
            if state == DEACTIVATED {
                return Err(NotDone(
                    "the network was not joined: the network manager gave up on it, which is \
                     what a password not given, or not accepted, looks like"
                        .to_owned(),
                ));
            }
            if Instant::now() >= until {
                return Err(NotDone(format!(
                    "the network was not joined within {} seconds",
                    self.joining.as_secs()
                )));
            }
            std::thread::sleep(LOOKING);
        }
    }
}

impl Networks for NetworkManager {
    fn now(&self) -> Result<TheNetworks, NotAnswering> {
        let wireless_on: bool =
            property(&self.bus, MANAGER_AT, MANAGER, "WirelessEnabled").map_err(NotAnswering)?;
        let visible = match self.wireless_device().map_err(NotAnswering)? {
            Some(device) => self.visible(&device).map_err(NotAnswering)?,
            None => Vec::new(),
        };
        Ok(TheNetworks {
            visible,
            saved: self.saved().map_err(NotAnswering)?,
            wireless_on,
            primary: self.primary().map_err(NotAnswering)?,
        })
    }
}

impl WhatIsReached for NetworkManager {
    /// Both properties in one pair of reads, off the network manager's own
    /// object, and neither worked out here: `Connectivity` is the answer to its
    /// own connectivity check, and `Metered` is what the connection said about
    /// itself or what it guessed.
    ///
    /// A property that cannot be read is [`NotAnswering`] for the whole
    /// reading, never one answer beside a made-up other: a status area told
    /// *reaching everything, not metered* because half the read failed is a
    /// status area lying about both.
    fn reaching_now(&self) -> Result<Reaching, NotAnswering> {
        let connectivity: u32 =
            property(&self.bus, MANAGER_AT, MANAGER, "Connectivity").map_err(NotAnswering)?;
        let metered: u32 =
            property(&self.bus, MANAGER_AT, MANAGER, "Metered").map_err(NotAnswering)?;
        Ok(Reaching::reported(
            HowFar::reported(connectivity),
            Metered::reported(metered),
        ))
    }
}

impl NetworkService for NetworkManager {
    fn join(&self, network: &Visible) -> Result<(), NotDone> {
        let Some(at) = &network.at else {
            return Err(NotDone(
                "that network was not reported by this network manager".to_owned(),
            ));
        };
        if network.protection() == Protection::Enterprise {
            return Err(NotDone(
                "a network that asks for an organisation's sign-in is not joined here in v0.5"
                    .to_owned(),
            ));
        }
        let device = path(&at.device)?;
        let point = path(&at.access_point)?;
        let already = self
            .saved()
            .map_err(NotDone)?
            .into_iter()
            .find(|saved| saved.name() == network.name());
        let active: OwnedObjectPath = match already.and_then(|saved| saved.at) {
            Some(saved) => call(
                &self.bus,
                MANAGER_AT,
                MANAGER,
                "ActivateConnection",
                &(path(&saved)?, &device, &point),
            )
            .map_err(NotDone)?,
            None => {
                // No settings of ours: the network manager makes the connection
                // from the access point, and asks the person's own agent for
                // anything it needs.
                let nothing: HashMap<&str, HashMap<&str, Value<'_>>> = HashMap::new();
                let (_, active): (OwnedObjectPath, OwnedObjectPath) = call(
                    &self.bus,
                    MANAGER_AT,
                    MANAGER,
                    "AddAndActivateConnection",
                    &(nothing, &device, &point),
                )
                .map_err(NotDone)?;
                active
            }
        };
        self.joined(&active)
    }

    fn forget(&self, network: &Saved) -> Result<(), NotDone> {
        let Some(at) = &network.at else {
            return Err(NotDone(
                "that saved network was not reported by this network manager".to_owned(),
            ));
        };
        call::<(), _>(&self.bus, at, CONNECTION, "Delete", &()).map_err(NotDone)
    }

    fn switch_wireless(&self, on: bool) -> Result<(), NotDone> {
        set(
            &self.bus,
            MANAGER_AT,
            MANAGER,
            "WirelessEnabled",
            Value::from(on),
        )
        .map_err(NotDone)
    }
}

/// The network manager on this machine's system bus, reached afresh for every
/// question and every change.
///
/// What the broker holds, rather than a [`NetworkManager`] made once: the
/// broker starts before the network manager may have, and a connection made at
/// start-up to a bus that was restarted since would answer nothing for the rest
/// of the broker's life.
#[derive(Debug, Clone, Copy, Default)]
pub struct OnThisMachine;

impl Networks for OnThisMachine {
    fn now(&self) -> Result<TheNetworks, NotAnswering> {
        NetworkManager::on_this_machine()?.now()
    }
}

impl WhatIsReached for OnThisMachine {
    fn reaching_now(&self) -> Result<Reaching, NotAnswering> {
        NetworkManager::on_this_machine()?.reaching_now()
    }
}

impl NetworkService for OnThisMachine {
    fn join(&self, network: &Visible) -> Result<(), NotDone> {
        reached()?.join(network)
    }

    fn forget(&self, network: &Saved) -> Result<(), NotDone> {
        reached()?.forget(network)
    }

    fn switch_wireless(&self, on: bool) -> Result<(), NotDone> {
        reached()?.switch_wireless(on)
    }
}

/// The network manager on this machine, for a change.
fn reached() -> Result<NetworkManager, NotDone> {
    NetworkManager::on_this_machine().map_err(|why| NotDone(why.to_string()))
}

/// How a network is protected, from what its access point announced.
fn protection(flags: u32, security: u32) -> Protection {
    if security & A_SIGN_IN != 0 {
        Protection::Enterprise
    } else if security & A_PASSWORD != 0 || (security == 0 && flags & PRIVACY != 0) {
        Protection::Password
    } else {
        Protection::Open
    }
}

/// An object path the network manager reported, as one again.
fn path(reported: &str) -> Result<ObjectPath<'_>, NotDone> {
    ObjectPath::try_from(reported).map_err(|why| NotDone(why.to_string()))
}

/// One text setting.
fn text(settings: &Settings, group: &str, key: &str) -> Option<String> {
    let value = settings.get(group)?.get(key)?;
    String::try_from(value.try_clone().ok()?).ok()
}

/// One bytes setting.
fn bytes(settings: &Settings, group: &str, key: &str) -> Option<Vec<u8>> {
    let value = settings.get(group)?.get(key)?;
    Vec::<u8>::try_from(value.try_clone().ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **How a network is protected is read the way the network manager's own
    /// tools read it**: a password for WPA, WPA3 and WEP, a sign-in for 802.1X,
    /// and open for nothing — including the encrypted open networks that ask
    /// for nothing.
    #[test]
    fn protection_is_read_from_what_the_access_point_announced() {
        assert_eq!(protection(0, 0), Protection::Open);
        assert_eq!(protection(PRIVACY, 0), Protection::Password);
        assert_eq!(protection(PRIVACY, 0x100), Protection::Password);
        assert_eq!(protection(PRIVACY, 0x400), Protection::Password);
        assert_eq!(protection(PRIVACY, 0x200), Protection::Enterprise);
        assert_eq!(protection(PRIVACY, 0x100 | 0x200), Protection::Enterprise);
        assert_eq!(protection(0, 0x800), Protection::Open);
    }
}
