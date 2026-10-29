//! Setting the time zone a person's region decided on the first-start sheets.
//!
//! The owner's decision of 2026-10-28: the region picked sets the machine's
//! zone. [ADR 0140](../../../docs/decisions/0140-the-first-start-sets-the-time-zone-itself.md)
//! records the road: the compositor sets `/etc/localtime` itself, through
//! `alo_formats::the_machines_zone`, as it already writes the machine's
//! keyboard and language. This decides only whether anything has to happen.

use std::path::Path;

/// What was done about the zone, for a log that has to explain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AboutTheZone {
    /// Nothing to do: none taken yet, or the same one as last time.
    Unchanged,
    /// The machine is in this zone now.
    Set(String),
    /// Taken and **not** set, with the sentence for the log. The machine is
    /// in the zone it was in.
    NotSet(String),
}

/// The zone this screen has already set.
#[derive(Debug, Default)]
pub(crate) struct TheZoneAPersonChose {
    /// What was last set, so a press that changes nothing does nothing.
    set: Option<String>,
}

impl TheZoneAPersonChose {
    /// Nothing taken and nothing set.
    pub(crate) fn nothing_yet() -> Self {
        Self::default()
    }

    /// Set this zone when it is new. `localtime` and `zoneinfo` are the
    /// machine's own and a test's in a test.
    pub(crate) fn chose(
        &mut self,
        zone: Option<&str>,
        localtime: &Path,
        zoneinfo: &Path,
    ) -> AboutTheZone {
        let Some(zone) = zone else {
            return AboutTheZone::Unchanged;
        };
        if self.set.as_deref() == Some(zone) {
            return AboutTheZone::Unchanged;
        }
        if let Err(why) = alo_formats::the_machines_zone::set(localtime, zoneinfo, zone) {
            return AboutTheZone::NotSet(format!("{zone}: {why:?}"));
        }
        self.set = Some(zone.to_owned());
        AboutTheZone::Set(zone.to_owned())
    }
}

#[cfg(all(test, unix))]
#[path = "the_zone_a_person_chose_tests.rs"]
mod tests;
