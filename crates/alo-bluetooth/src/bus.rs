//! The Bluetooth service's names on the system bus, and the two ways it is
//! asked anything: a method, and a property.
//!
//! Every name here is BlueZ's own published D-Bus interface (`org.bluez`,
//! stable since 5.0). They are written once so that the client and anything
//! standing in for the service cannot spell one differently.

use zbus::blocking::Connection;
use zbus::zvariant::{DynamicType, OwnedValue, Type, Value};

/// The name the Bluetooth service owns on the system bus.
pub const THE_SERVICE: &str = "org.bluez";

/// Its root object, which is also where agents register.
pub const THE_SERVICE_AT: &str = "/org/bluez";

/// A radio.
pub const ADAPTER: &str = "org.bluez.Adapter1";

/// A device the radio has seen.
pub const DEVICE: &str = "org.bluez.Device1";

/// Where an agent registers.
pub const AGENT_MANAGER: &str = "org.bluez.AgentManager1";

/// The interface an agent serves.
pub const AGENT: &str = "org.bluez.Agent1";

/// Where alo OS's own pairing agent is served.
pub const OUR_AGENT_AT: &str = "/world/alo/bluetooth/agent";

/// What this machine can show and take: six digits either way, which is what a
/// machine with a screen and a keyboard has.
pub const WHAT_THIS_MACHINE_CAN_DO: &str = "KeyboardDisplay";

/// The standard object manager, which lists everything the service has.
pub const OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";

/// The standard properties interface.
pub const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// What went wrong asking, in English for a log.
pub fn failed(asking: &str, why: &dyn std::fmt::Display) -> String {
    format!("{asking}: {why}")
}

/// Call one method on the Bluetooth service, and read its answer as `T`.
pub fn call<T, B>(
    bus: &Connection,
    at: &str,
    interface: &str,
    method: &str,
    body: &B,
) -> Result<T, String>
where
    T: for<'d> zbus::export::serde::Deserialize<'d> + Type,
    B: zbus::export::serde::Serialize + DynamicType,
{
    bus.call_method(Some(THE_SERVICE), at, Some(interface), method, body)
        .map_err(|why| failed(method, &why))?
        .body()
        .deserialize::<T>()
        .map_err(|why| failed(method, &why))
}

/// Set one property.
pub fn set(
    bus: &Connection,
    at: &str,
    interface: &str,
    name: &str,
    value: Value<'_>,
) -> Result<(), String> {
    bus.call_method(
        Some(THE_SERVICE),
        at,
        Some(PROPERTIES),
        "Set",
        &(interface, name, value),
    )
    .map(drop)
    .map_err(|why| failed(name, &why))
}

/// One value out of a service's properties, as `T`.
pub fn read<T>(properties: &std::collections::HashMap<String, OwnedValue>, name: &str) -> Option<T>
where
    T: TryFrom<OwnedValue>,
{
    T::try_from(properties.get(name)?.try_clone().ok()?).ok()
}
