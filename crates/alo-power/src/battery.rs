//! The battery, read from what the kernel says about it.
//!
//! Everything here comes out of `/sys/class/power_supply`, which is the kernel's
//! own list of what powers this machine. Nothing is asked of a daemon: a battery
//! reading that went through a service is a battery reading that stops when the
//! service does, and *why is my battery gone* is a question a machine should be
//! able to answer while things are going wrong.
//!
//! # A machine with no battery is a machine, not an error
//!
//! Most machines in this repository have none, and so does every desktop alo OS
//! will ever run on. [`TheBattery::on_this_machine`] answers [`None`] for them,
//! and a surface shows nothing about power rather than *0%*.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::reading::{Charge, Charging, NotAReading, Reading};

/// Where the kernel lists what powers this machine.
pub const WHERE_THE_SUPPLIES_ARE: &str = "/sys/class/power_supply";

/// **This machine's battery.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBattery {
    /// Where the kernel keeps it.
    at: PathBuf,
}

impl TheBattery {
    /// The battery this machine has, or [`None`] on a machine with none.
    #[must_use]
    pub fn on_this_machine() -> Option<Self> {
        Self::among(Path::new(WHERE_THE_SUPPLIES_ARE))
    }

    /// The first battery in a list of power supplies — a machine with two is
    /// rare and its second is read the same way, by a caller that wants it.
    #[must_use]
    pub fn among(supplies: &Path) -> Option<Self> {
        let mut every: Vec<PathBuf> = std::fs::read_dir(supplies)
            .ok()?
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|at| said(at, "type").as_deref() == Some("Battery"))
            .collect();
        every.sort();
        every.into_iter().next().map(|at| Self { at })
    }

    /// Where the kernel keeps it.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        &self.at
    }

    /// Whether the battery is still there. A battery taken out while the
    /// machine runs is a real thing, and so is one whose folder stays behind.
    #[must_use]
    pub fn is_present(&self) -> bool {
        said(&self.at, "present").as_deref() != Some("0") && self.at.exists()
    }

    /// **Read it now.**
    ///
    /// # Errors
    /// [`NotRead`] where the kernel's own files are not there or say something
    /// that is not a reading.
    pub fn now(&self) -> Result<Reading, NotRead> {
        self.read_at(SystemTime::now())
    }

    /// The same, with the moment named — so a test is not at the mercy of a
    /// clock.
    ///
    /// # Errors
    /// [`NotRead`].
    pub fn read_at(&self, taken_at: SystemTime) -> Result<Reading, NotRead> {
        let capacity = said(&self.at, "capacity").ok_or_else(|| NotRead::NothingThere {
            file: "capacity".to_owned(),
        })?;
        let hundredths: u8 = capacity.parse().map_err(|_| NotRead::NotAReading {
            file: "capacity".to_owned(),
            said: capacity.clone(),
        })?;
        let charge = Charge::reported(hundredths)?;
        let charging = Charging::reported(&said(&self.at, "status").unwrap_or_default());
        let through_it = said(&self.at, "current_now").and_then(|said| said.parse().ok());
        Ok(Reading::taken(charge, charging, through_it, taken_at))
    }

    /// **Whether this machine can be told to stop charging at a limit**, and
    /// where it can, what the limit is now.
    ///
    /// [`ChargeLimit::NotOnThisMachine`] is the common answer and is not a
    /// failure: a surface shows the setting on a machine that has one and shows
    /// nothing at all on a machine that does not, rather than a switch that
    /// cannot be moved.
    #[must_use]
    pub fn charge_limit(&self) -> ChargeLimit {
        match said(&self.at, "charge_control_end_threshold").and_then(|said| said.parse().ok()) {
            Some(stops_at) => ChargeLimit::StopsAt { stops_at },
            None => ChargeLimit::NotOnThisMachine,
        }
    }
}

/// **Whether this machine will stop charging before full.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeLimit {
    /// It will, at this many hundredths.
    StopsAt {
        /// Where it stops.
        stops_at: u8,
    },
    /// This machine's hardware has no such setting, so nothing is shown.
    NotOnThisMachine,
}

/// Why the battery could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotRead {
    /// One of the kernel's own files is not there.
    #[error("this machine's battery has no {file}, so how full it is cannot be read")]
    NothingThere {
        /// Which file.
        file: String,
    },
    /// It is there and it says something that is not a reading.
    #[error("this machine's battery says its {file} is {said}, which is not a reading")]
    NotAReading {
        /// Which file.
        file: String,
        /// What it said.
        said: String,
    },
    /// A reading this crate will not hold.
    #[error(transparent)]
    NotHeld(#[from] NotAReading),
}

/// One of the kernel's files, trimmed, or [`None`] where there is none.
fn said(at: &Path, file: &str) -> Option<String> {
    Some(
        std::fs::read_to_string(at.join(file))
            .ok()?
            .trim()
            .to_owned(),
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A power supply written the way the kernel writes one, in a folder that
    /// takes itself away when the test is done with it (`docs/quirks.md`: a
    /// machine's `/tmp` held 43,290 folders left by tests on 2026-09-17, and
    /// the suite died of open files).
    fn a_supply(kind: &str, files: &[(&str, &str)]) -> ASupply {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let supplies = std::env::temp_dir().join(format!(
            "alo-power-battery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let at = supplies.join(if kind == "Battery" { "BAT0" } else { "AC" });
        std::fs::create_dir_all(&at).expect("a folder for this test");
        std::fs::write(at.join("type"), format!("{kind}\n")).expect("written");
        for (file, said) in files {
            std::fs::write(at.join(file), format!("{said}\n")).expect("written");
        }
        ASupply(supplies)
    }

    /// Where a test's power supplies are, removed when it is done.
    struct ASupply(PathBuf);

    impl ASupply {
        fn at(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for ASupply {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.0));
        }
    }

    /// **A machine with no battery has no battery**, which is not an error and
    /// not nought per cent.
    #[test]
    fn a_machine_with_no_battery_is_a_machine() {
        let supplies = a_supply("Mains", &[("online", "1")]);
        assert_eq!(TheBattery::among(supplies.at()), None);
        assert_eq!(
            TheBattery::among(Path::new("/there-is-no-such-place")),
            None
        );
    }

    /// **A battery is read from the kernel's own files.**
    #[test]
    fn a_battery_is_read_from_what_the_kernel_wrote() {
        let supplies = a_supply(
            "Battery",
            &[
                ("capacity", "50"),
                ("status", "Discharging"),
                ("current_now", "-1600"),
                ("present", "1"),
            ],
        );
        let battery = TheBattery::among(supplies.at()).expect("this machine has a battery");
        let reading = battery.read_at(SystemTime::UNIX_EPOCH).expect("a reading");
        assert_eq!(reading.charge().hundredths(), 50);
        assert_eq!(reading.charging(), Charging::Discharging);
        assert_eq!(reading.through_it(), Some(1_600));
        assert!(battery.is_present());
        assert_eq!(battery.charge_limit(), ChargeLimit::NotOnThisMachine);
    }

    /// **A charge limit is offered where the hardware has one**, and is absent
    /// where it does not — never a switch that cannot be moved.
    #[test]
    fn a_charge_limit_is_there_or_it_is_not_there() {
        let with = a_supply(
            "Battery",
            &[("capacity", "80"), ("charge_control_end_threshold", "80")],
        );
        assert_eq!(
            TheBattery::among(with.at())
                .expect("a battery")
                .charge_limit(),
            ChargeLimit::StopsAt { stops_at: 80 }
        );
    }

    /// **A battery whose files say something that is not a reading is refused**,
    /// rather than read as empty — which is the reading that would have a person
    /// running for a cable.
    #[test]
    fn a_battery_that_says_something_strange_is_not_read_as_empty() {
        let odd = a_supply("Battery", &[("capacity", "not a number")]);
        let why = TheBattery::among(odd.at())
            .expect("a battery")
            .read_at(SystemTime::UNIX_EPOCH)
            .expect_err("that is not a reading");
        assert!(matches!(why, NotRead::NotAReading { .. }));

        let missing = a_supply("Battery", &[("status", "Discharging")]);
        assert!(matches!(
            TheBattery::among(missing.at())
                .expect("a battery")
                .read_at(SystemTime::UNIX_EPOCH),
            Err(NotRead::NothingThere { .. })
        ));

        let past_full = a_supply("Battery", &[("capacity", "127")]);
        assert!(matches!(
            TheBattery::among(past_full.at())
                .expect("a battery")
                .read_at(SystemTime::UNIX_EPOCH),
            Err(NotRead::NotHeld(NotAReading::PastFull { .. }))
        ));
    }
}
