//! The network manager's names on the system bus, and the two ways it is asked
//! anything: a method, and a property.
//!
//! Every name here is NetworkManager's own published D-Bus interface
//! (`org.freedesktop.NetworkManager`, stable since 1.0). They are written once so
//! the client and a test standing in for the service cannot spell one
//! differently.

use zbus::blocking::Connection;
use zbus::zvariant::{DynamicType, OwnedValue, Type, Value};

/// The name the network manager owns on the system bus.
pub const THE_NETWORK_MANAGER: &str = "org.freedesktop.NetworkManager";

/// Its own object, and the interface of the same name.
pub const MANAGER_AT: &str = "/org/freedesktop/NetworkManager";

/// The saved connections' object.
pub const SETTINGS_AT: &str = "/org/freedesktop/NetworkManager/Settings";

/// The object secret agents register with.
pub const AGENT_MANAGER_AT: &str = "/org/freedesktop/NetworkManager/AgentManager";

/// The interfaces.
pub const MANAGER: &str = "org.freedesktop.NetworkManager";
/// A device.
pub const DEVICE: &str = "org.freedesktop.NetworkManager.Device";
/// A wireless device.
pub const WIRELESS: &str = "org.freedesktop.NetworkManager.Device.Wireless";
/// An access point.
pub const ACCESS_POINT: &str = "org.freedesktop.NetworkManager.AccessPoint";
/// The saved connections.
pub const SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";
/// One saved connection.
pub const CONNECTION: &str = "org.freedesktop.NetworkManager.Settings.Connection";
/// A connection in use.
pub const ACTIVE: &str = "org.freedesktop.NetworkManager.Connection.Active";
/// Where secret agents register.
pub const AGENT_MANAGER: &str = "org.freedesktop.NetworkManager.AgentManager";

/// Where a secret agent's object is, which the network manager calls.
pub const SECRET_AGENT_AT: &str = "/org/freedesktop/NetworkManager/SecretAgent";

/// The standard properties interface.
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// `NM_DEVICE_TYPE_WIFI`.
pub const A_WIRELESS_DEVICE: u32 = 2;

/// `NM_ACTIVE_CONNECTION_STATE_ACTIVATED`.
pub const ACTIVATED: u32 = 2;

/// `NM_ACTIVE_CONNECTION_STATE_DEACTIVATED`.
pub const DEACTIVATED: u32 = 4;

/// What went wrong asking, in English for a log.
pub fn failed(asking: &str, why: &dyn std::fmt::Display) -> String {
    format!("{asking}: {why}")
}

/// Call one method on the network manager, and read its answer as `T`.
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
    bus.call_method(Some(THE_NETWORK_MANAGER), at, Some(interface), method, body)
        .map_err(|why| failed(method, &why))?
        .body()
        .deserialize::<T>()
        .map_err(|why| failed(method, &why))
}

/// Read one property, as `T`.
pub fn property<T>(bus: &Connection, at: &str, interface: &str, name: &str) -> Result<T, String>
where
    T: TryFrom<OwnedValue>,
    <T as TryFrom<OwnedValue>>::Error: std::fmt::Display,
{
    let value: OwnedValue = call(bus, at, PROPERTIES, "Get", &(interface, name))?;
    T::try_from(value).map_err(|why| failed(name, &why))
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
        Some(THE_NETWORK_MANAGER),
        at,
        Some(PROPERTIES),
        "Set",
        &(interface, name, value),
    )
    .map(drop)
    .map_err(|why| failed(name, &why))
}
