//! The two units the broker starts to carry an update out, and the one way it
//! starts them.
//!
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md),
//! accepted option B: the base's own program changes the machine only for a
//! process holding `CAP_SYS_ADMIN`, and **the broker holds no capability and
//! is not given one**. So the privileged half of an update is a unit of its
//! own with a fixed command line, and all the broker does is ask systemd to
//! start it and read the result.
//!
//! # There is no unit name anywhere that a request could reach
//!
//! [`TheUnit`] is a closed enum of two, and [`StartingUnits::start`] takes
//! one. Nothing in this crate turns text into a unit name, so there is no
//! spelling of *start whatever unit this identity happens to name* — which
//! would have been law 2 broken by the back door, a broker that runs an
//! arbitrary command because systemd was willing to look the name up.
//!
//! # Waited for, not fired and forgotten
//!
//! `StartUnit` answers as soon as the job is enqueued, and an update is
//! minutes of download. The broker's answer is *carried* or *not carried*, so
//! it waits: while the unit has a job, or is still activating, it asks again —
//! and then reads what the service actually did. A unit that never finishes
//! inside [`LONGEST_AN_UPDATE_TAKES`] is *not carried*, which is shorter than
//! the hour the asking side waits (`alo_broker::asking::WAITING_FOR_AN_UPDATE`)
//! so that the door answers rather than the caller giving up on it.

use std::time::Duration;

/// One of the two units that carry an update out. There is no third, and
/// nothing makes one of these out of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TheUnit {
    /// Stages the build a person approved.
    ApplyingAnUpdate,
    /// Sets the machine to start the build before at the next restart.
    GoingBack,
}

/// Both of them, for the tests that walk the pair.
pub const EVERY_UNIT: [TheUnit; 2] = [TheUnit::ApplyingAnUpdate, TheUnit::GoingBack];

/// How long the broker waits for one of them to finish.
///
/// Fifty-five minutes: long enough for a system image over a poor connection,
/// and five minutes inside what the asking side waits, so a unit that has hung
/// is answered by the door rather than abandoned by the caller.
pub const LONGEST_AN_UPDATE_TAKES: Duration = Duration::from_secs(3_300);

/// How often the broker asks systemd whether the unit has finished.
pub const ASKED_AGAIN_EVERY: Duration = Duration::from_millis(500);

/// Why a unit did not carry out what it was started for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotDone(pub String);

impl std::fmt::Display for NotDone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NotDone {}

/// Something that starts one of the two units and says whether it succeeded —
/// systemd on a machine, and a test's stand-in that writes down what it was
/// asked.
pub trait StartingUnits {
    /// Start `unit`, wait for it, and answer whether it did what it was for.
    ///
    /// # Errors
    /// [`NotDone`], and the machine is as it was — or, where the unit ran and
    /// failed, as the unit left it, which for both of these is unchanged.
    fn start(&self, unit: TheUnit) -> Result<(), NotDone>;
}

impl TheUnit {
    /// The unit's name, as systemd knows it.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::ApplyingAnUpdate => "alo-applying-an-update.service",
            Self::GoingBack => "alo-going-back.service",
        }
    }

    /// The program the unit runs, where the image puts it.
    #[must_use]
    pub const fn runs(self) -> &'static str {
        match self {
            Self::ApplyingAnUpdate => "/usr/libexec/alo-applying-an-update",
            Self::GoingBack => "/usr/libexec/alo-going-back",
        }
    }
}

/// systemd on this machine's system bus, and the three methods it is asked.
#[cfg(target_os = "linux")]
pub mod systemd {
    use std::time::{Duration, Instant};

    use zbus::blocking::Connection;
    use zbus::zvariant::{OwnedObjectPath, OwnedValue};

    use super::{ASKED_AGAIN_EVERY, LONGEST_AN_UPDATE_TAKES, NotDone, StartingUnits, TheUnit};

    /// The system bus, as systemd makes it — named, never read out of an
    /// environment a service happened to be started with.
    pub const THE_SYSTEM_BUS: &str = "unix:path=/run/dbus/system_bus_socket";

    /// The name systemd owns on the system bus.
    const SYSTEMD: &str = "org.freedesktop.systemd1";

    /// Its manager object.
    const MANAGER_AT: &str = "/org/freedesktop/systemd1";

    /// The manager's interface.
    const MANAGER: &str = "org.freedesktop.systemd1.Manager";

    /// One unit's interface.
    const UNIT: &str = "org.freedesktop.systemd1.Unit";

    /// One service unit's interface.
    const SERVICE: &str = "org.freedesktop.systemd1.Service";

    /// The standard interface properties are read through.
    const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

    /// Starting a unit.
    const START_UNIT: &str = "StartUnit";

    /// Finding a unit's object.
    const GET_UNIT: &str = "GetUnit";

    /// Reading one property.
    const GET: &str = "Get";

    /// Every method this client calls, and there is no other.
    ///
    /// `tests/the_updates_are_carried_by_a_unit.rs` reads this file for every
    /// method that stops, kills, masks, reloads or writes a unit, and finds
    /// none: the broker starts two units it names itself and does nothing else
    /// to systemd.
    pub const METHODS: [&str; 3] = [START_UNIT, GET_UNIT, GET];

    /// How systemd is asked to enqueue the job: fail rather than disturb
    /// anything already going on.
    const MODE: &str = "fail";

    /// How long one call may take.
    const ANSWERING: Duration = Duration::from_secs(30);

    /// systemd on this machine's system bus, reached afresh for every start.
    ///
    /// What the broker holds, rather than a [`Systemd`] made once: a
    /// connection made at start-up to a bus that was restarted since would
    /// answer nothing for the rest of the broker's life, and an update is a
    /// rare, deliberate act that can afford one connection of its own.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct OnThisMachine;

    impl StartingUnits for OnThisMachine {
        fn start(&self, unit: TheUnit) -> Result<(), NotDone> {
            Systemd::on_this_machine()?.start(unit)
        }
    }

    /// systemd, on one bus.
    #[derive(Debug)]
    pub struct Systemd {
        /// The bus.
        bus: Connection,
    }

    impl Systemd {
        /// systemd on this machine's system bus.
        ///
        /// # Errors
        /// [`NotDone`] when the system bus cannot be reached.
        pub fn on_this_machine() -> Result<Self, NotDone> {
            Self::at(THE_SYSTEM_BUS)
        }

        /// systemd on the bus at this address — how a test reaches one of its
        /// own.
        ///
        /// # Errors
        /// [`NotDone`] when that bus cannot be reached.
        pub fn at(address: &str) -> Result<Self, NotDone> {
            let bus = zbus::blocking::connection::Builder::address(address)
                .map(|building| building.method_timeout(ANSWERING))
                .and_then(zbus::blocking::connection::Builder::build)
                .map_err(|why| NotDone(format!("systemd could not be reached: {why}")))?;
            Ok(Self { bus })
        }

        /// One property of one object, as `T`.
        fn property<T>(
            &self,
            at: &OwnedObjectPath,
            interface: &str,
            named: &str,
        ) -> Result<T, NotDone>
        where
            T: TryFrom<OwnedValue>,
        {
            let value: OwnedValue = self
                .call(at.as_str(), PROPERTIES, GET, &(interface, named))
                .map_err(|why| NotDone(format!("{named} was not read: {why}")))?;
            T::try_from(value)
                .map_err(|_| NotDone(format!("{named} was not what systemd says it is")))
        }

        /// Call one of [`METHODS`], and read its answer as `T`.
        fn call<T, B>(&self, at: &str, interface: &str, method: &str, body: &B) -> Result<T, String>
        where
            T: for<'d> zbus::export::serde::Deserialize<'d> + zbus::zvariant::Type,
            B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
        {
            if !METHODS.contains(&method) {
                return Err(format!("{method} is not a method this client calls"));
            }
            self.bus
                .call_method(Some(SYSTEMD), at, Some(interface), method, body)
                .map_err(|why| format!("{method}: {why}"))?
                .body()
                .deserialize::<T>()
                .map_err(|why| format!("{method}: {why}"))
        }
    }

    impl StartingUnits for Systemd {
        fn start(&self, unit: TheUnit) -> Result<(), NotDone> {
            let named = unit.named();
            let _job: OwnedObjectPath = self
                .call(MANAGER_AT, MANAGER, START_UNIT, &(named, MODE))
                .map_err(|why| NotDone(format!("{named} was not started: {why}")))?;
            let at: OwnedObjectPath = self
                .call(MANAGER_AT, MANAGER, GET_UNIT, &(named,))
                .map_err(|why| NotDone(format!("{named} was not found on the bus: {why}")))?;

            let began = Instant::now();
            loop {
                let job: (u32, OwnedObjectPath) = self.property(&at, UNIT, "Job")?;
                let state: String = self.property(&at, UNIT, "ActiveState")?;
                if job.0 == 0 && state != "activating" {
                    break;
                }
                if began.elapsed() >= LONGEST_AN_UPDATE_TAKES {
                    return Err(NotDone(format!(
                        "{named} was still running after {} seconds, so nothing is known to have \
                         been carried out; the machine's journal has what it said",
                        LONGEST_AN_UPDATE_TAKES.as_secs()
                    )));
                }
                std::thread::sleep(ASKED_AGAIN_EVERY);
            }

            let result: String = self.property(&at, SERVICE, "Result")?;
            let status: i32 = self.property(&at, SERVICE, "ExecMainStatus")?;
            if result == "success" && status == 0 {
                return Ok(());
            }
            Err(NotDone(format!(
                "{named} ran and did not carry it out ({result}, exit {status}), so the machine \
                 is as it was; the journal has what it said"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **Each unit has its own name and its own program**, and no two share
    /// either.
    #[test]
    fn each_unit_is_named_once_and_runs_one_program_of_its_own() {
        let names: BTreeSet<&str> = EVERY_UNIT.iter().map(|unit| unit.named()).collect();
        let programs: BTreeSet<&str> = EVERY_UNIT.iter().map(|unit| unit.runs()).collect();
        assert_eq!(names.len(), EVERY_UNIT.len());
        assert_eq!(programs.len(), EVERY_UNIT.len());
        for unit in EVERY_UNIT {
            assert!(unit.named().ends_with(".service"), "{}", unit.named());
            assert!(unit.runs().starts_with("/usr/libexec/"), "{}", unit.runs());
        }
    }

    /// **The broker gives up before the caller does**, so an update that hangs
    /// is answered at the door rather than abandoned by whoever asked.
    #[test]
    fn the_broker_stops_waiting_before_the_asking_side_does() {
        assert!(LONGEST_AN_UPDATE_TAKES < alo_broker::asking::WAITING_FOR_AN_UPDATE);
        assert!(ASKED_AGAIN_EVERY < LONGEST_AN_UPDATE_TAKES);
    }
}
