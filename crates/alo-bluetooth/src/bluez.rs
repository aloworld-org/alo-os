//! This machine's own Bluetooth service, reached.
//!
//! [`TheBluetoothService`] is [`BluetoothService`] on a real machine: it asks
//! the rented service for everything it has, and tells it the four things a
//! person can do. Nothing about it is patched, wrapped or extended (ADR 0011).
//!
//! # Everything at once, rather than a question per device
//!
//! The service keeps a tree of objects — a radio, and a device under it for
//! every one it has seen — and will hand over the whole of it in one call. That
//! is what this reads. The alternative, walking the tree and asking each object
//! for its properties, is the same answer with a window in the middle during
//! which a device can appear or go away, and a list that disagrees with itself
//! is worse than a list a moment old.
//!
//! # A machine with no radio, and a machine with no service
//!
//! Two different sentences, and this keeps them apart. A laptop with no
//! Bluetooth hardware has a service that answers and no radio in the tree
//! ([`TheDevices::without_a_radio`]). A machine where nothing owns the service's
//! name has not been asked at all ([`NotAnswering::NothingAnswers`]) — and
//! **neither is an empty list of devices**, because a person shown an empty list
//! goes looking for their headphones.

use std::collections::HashMap;

use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::bus::{self, ADAPTER, DEVICE, OBJECT_MANAGER, THE_SERVICE};
use crate::choosing::Chosen;
use crate::radio::Radio;
use crate::reported::{Address, DeviceName, Found, Kind, TheDevices};
use crate::service::{BluetoothService, Devices, NotAnswering, NotDone};

/// Everything the service has, as it hands it over.
type Everything = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

/// **This machine's Bluetooth service.**
pub struct TheBluetoothService {
    /// The system bus.
    bus: Connection,
}

impl TheBluetoothService {
    /// The service on the machine this is running on.
    ///
    /// # Errors
    /// [`NotAnswering::NothingAnswers`] where there is no system bus to ask on.
    pub fn on_this_machine() -> Result<Self, NotAnswering> {
        let bus = Connection::system()
            .map_err(|why| NotAnswering::NothingAnswers(bus::failed("the system bus", &why)))?;
        Ok(Self { bus })
    }

    /// Everything the service has, in one call.
    fn everything(&self) -> Result<Everything, NotAnswering> {
        bus::call(&self.bus, "/", OBJECT_MANAGER, "GetManagedObjects", &()).map_err(|why| {
            if why.contains("ServiceUnknown") || why.contains("was not provided") {
                return NotAnswering::NothingAnswers(why);
            }
            NotAnswering::NoAnswer(why)
        })
    }

    /// Where the radio is in the service's tree, or [`None`] on a machine with
    /// none.
    fn the_radio(everything: &Everything) -> Option<(String, Radio)> {
        everything.iter().find_map(|(at, interfaces)| {
            let adapter = interfaces.get(ADAPTER)?;
            let on = bus::read::<bool>(adapter, "Powered").unwrap_or(false);
            Some((
                at.as_str().to_owned(),
                if on { Radio::On } else { Radio::Off },
            ))
        })
    }

    /// Where one device is in the service's tree.
    fn where_the_device_is(&self, address: &Address) -> Result<String, NotDone> {
        let everything = self.everything()?;
        everything
            .iter()
            .find_map(|(at, interfaces)| {
                let device = interfaces.get(DEVICE)?;
                let its = bus::read::<String>(device, "Address")?;
                (Address::reported(&its).ok().as_ref() == Some(address))
                    .then(|| at.as_str().to_owned())
            })
            .ok_or_else(|| NotDone::Refused {
                doing: format!("find {}", address.as_str()),
                said: "this machine's Bluetooth service has no such device".to_owned(),
            })
    }

    /// One device, as this crate holds them.
    fn a_device(properties: &HashMap<String, OwnedValue>) -> Option<Found> {
        let address = Address::reported(&bus::read::<String>(properties, "Address")?).ok()?;
        let called = DeviceName::reported(
            &bus::read::<String>(properties, "Alias")
                .or_else(|| bus::read::<String>(properties, "Name"))
                .unwrap_or_default(),
            &address,
        );
        let kind = Kind::from_class(bus::read::<u32>(properties, "Class").unwrap_or(0));
        let paired = bus::read::<bool>(properties, "Paired").unwrap_or(false);
        Some(Found::reported(address, called, kind, paired))
    }

    /// Tell the service one thing, and say what happened if it refused.
    fn tell<B>(
        &self,
        doing: &str,
        at: &str,
        interface: &str,
        method: &str,
        body: &B,
    ) -> Result<(), NotDone>
    where
        B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
    {
        self.bus
            .call_method(Some(THE_SERVICE), at, Some(interface), method, body)
            .map(drop)
            .map_err(|why| NotDone::Refused {
                doing: doing.to_owned(),
                said: why.to_string(),
            })
    }
}

impl Devices for TheBluetoothService {
    fn now(&self) -> Result<TheDevices, NotAnswering> {
        let everything = self.everything()?;
        let Some((_, radio)) = Self::the_radio(&everything) else {
            return Ok(TheDevices::without_a_radio());
        };
        let mut found: Vec<Found> = everything
            .values()
            .filter_map(|interfaces| Self::a_device(interfaces.get(DEVICE)?))
            .collect();
        // The service hands its objects over in no particular order, and a list
        // that reshuffles itself under a person's hand is a list they mis-click.
        found.sort_by(|one, another| one.address().cmp(another.address()));
        Ok(TheDevices::reported(found, radio, true))
    }
}

impl BluetoothService for TheBluetoothService {
    fn pair(&self, chosen: &Chosen) -> Result<(), NotDone> {
        let at = self.where_the_device_is(chosen.address())?;
        // Whatever the device asks for on the way is asked of the person by the
        // agent, which is registered for the length of the session rather than
        // for this call.
        self.tell(
            &format!("pair with {}", chosen.address().as_str()),
            &at,
            DEVICE,
            "Pair",
            &(),
        )
    }

    fn forget(&self, address: &Address) -> Result<(), NotDone> {
        let at = self.where_the_device_is(address)?;
        let everything = self.everything()?;
        let (radio, _) = Self::the_radio(&everything).ok_or_else(|| NotDone::Refused {
            doing: format!("forget {}", address.as_str()),
            said: "this machine has no Bluetooth radio".to_owned(),
        })?;
        let path = OwnedObjectPath::try_from(at).map_err(|why| NotDone::Refused {
            doing: format!("forget {}", address.as_str()),
            said: why.to_string(),
        })?;
        // RemoveDevice is the one act the plan asks for: the device goes, and
        // its keys go with it. There is no second switch.
        self.tell(
            &format!("forget {}", address.as_str()),
            &radio,
            ADAPTER,
            "RemoveDevice",
            &(path,),
        )
    }

    fn switch_radio(&self, to: Radio) -> Result<(), NotDone> {
        let everything = self.everything()?;
        let (radio, _) = Self::the_radio(&everything).ok_or_else(|| NotDone::Refused {
            doing: "switch the radio".to_owned(),
            said: "this machine has no Bluetooth radio".to_owned(),
        })?;
        bus::set(
            &self.bus,
            &radio,
            ADAPTER,
            "Powered",
            Value::Bool(to.anything_may_connect()),
        )
        .map_err(|said| NotDone::Refused {
            doing: "switch the radio".to_owned(),
            said,
        })
    }

    fn look(&self, looking: bool) -> Result<(), NotDone> {
        let everything = self.everything()?;
        let (radio, on) = Self::the_radio(&everything).ok_or_else(|| NotDone::Refused {
            doing: "look for devices".to_owned(),
            said: "this machine has no Bluetooth radio".to_owned(),
        })?;
        if looking && !on.anything_may_connect() {
            return Err(NotDone::Refused {
                doing: "look for devices".to_owned(),
                said: "the radio is off, and off means off".to_owned(),
            });
        }
        self.tell(
            "look for devices",
            &radio,
            ADAPTER,
            if looking {
                "StartDiscovery"
            } else {
                "StopDiscovery"
            },
            &(),
        )
    }
}
