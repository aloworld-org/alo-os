//! **Changing the machine's time zone from Settings**, the standard Linux way:
//! systemd's own time-zone service, `org.freedesktop.timedate1`, asked over
//! the system bus to `SetTimezone`.
//!
//! The owner's decision of 2026-10-28, recorded in
//! [ADR 0138](../../../docs/decisions/0138-the-first-start-sets-the-time-zone-itself.md):
//! Settings runs as the person, not as root, so it cannot replace
//! `/etc/localtime` the way the first-start sheets do. It asks the service whose
//! job that is, and a polkit rule the image ships
//! (`image/usr/share/polkit-1/rules.d/50-alo-time-zone.rules`) lets **the
//! person's account, in its own active session,** do it without an
//! administrator's password — and refuses **the agent's account** outright.
//! Nothing here can grant itself anything: if the rule is not there, the
//! service says no, and Settings says so.
//!
//! The setter is a trait so that every rule in Settings about which zone is
//! set, and what it says when the service refuses, is tested without a bus.

/// **Something that sets the machine's time zone.**
pub trait SetsTheZone {
    /// Set the zone named `zone`, as the zone database names it.
    ///
    /// # Errors
    /// What the service, or the bus, said — for the session's log. The zone
    /// is as it was.
    fn set(&self, zone: &str) -> Result<(), String>;
}

/// **systemd's time-zone service, on the system bus.**
#[derive(Debug, Clone, Copy, Default)]
pub struct Timedated;

impl SetsTheZone for Timedated {
    #[cfg(target_os = "linux")]
    fn set(&self, zone: &str) -> Result<(), String> {
        let bus = zbus::blocking::Connection::system().map_err(|why| why.to_string())?;
        // `false`: never ask for a password interactively. The image's rule
        // either lets this person do it or it does not, and a prompt from a
        // service a person cannot see would be a question nobody can answer.
        bus.call_method(
            Some("org.freedesktop.timedate1"),
            "/org/freedesktop/timedate1",
            Some("org.freedesktop.timedate1"),
            "SetTimezone",
            &(zone, false),
        )
        .map(|_| ())
        .map_err(|why| why.to_string())
    }

    #[cfg(not(target_os = "linux"))]
    fn set(&self, _zone: &str) -> Result<(), String> {
        Err("there is no systemd time-zone service on this host".to_owned())
    }
}
