//! **The time zone**, as a section of Settings: the cities of the region this
//! person's dates are written like, each naming one of its zones, and the one
//! the machine is in ticked.
//!
//! The owner's decision of 2026-10-28: the zone is changed later **the
//! standard Linux way** — systemd's time-zone service, asked by the person's
//! own session and allowed by a polkit rule for their account and not the
//! agent's (`crate::the_zone_through_timedated`, ADR 0138) — with **the same
//! names and the same which-city question** as the first-start sheets: the
//! region's zones named by their cities in the person's language
//! (`alo_formats::zones`).
//!
//! # Which region's zones
//!
//! The region `crate::TheirFormats::the_region` answers, which is the one
//! dates are written like: the person's own choice in Settings, the region
//! the machine was set up in, the country of its zone, the language's usual
//! region. **A person who moves country changes the region first**, two rows
//! above, and this section then offers that country's cities — so there is
//! one question about where they are, asked in one place, not two lists of
//! the world.

use alo_formats::RegionCode;
use alo_formats::zones::NamedZone;
use alo_strings::{Filling, Strings};

use crate::settings_kept::SettingsKept;
use crate::the_zone_through_timedated::SetsTheZone;

/// The section, while Settings is open.
#[derive(Debug)]
pub(crate) struct TimeZoneSection {
    /// The region's zones, named and in the language's order.
    zones: Vec<NamedZone>,
    /// The zone the machine is in, by the database's name, where it says.
    now: Option<String>,
    /// What the last change said, where it said anything.
    told: Option<String>,
}

impl TimeZoneSection {
    /// The zones of `region`, named in `language`, with the machine in `now`.
    pub(crate) fn of(region: Option<&RegionCode>, language: &str, now: Option<String>) -> Self {
        Self {
            zones: region
                .map(|it| alo_formats::zones::the_zones_in(it, language))
                .unwrap_or_default(),
            now,
            told: None,
        }
    }

    /// **The same section for another region** — the person changed the region
    /// two rows above — keeping the zone the machine is in.
    pub(crate) fn follows(&mut self, region: Option<&RegionCode>, language: &str) {
        self.zones = region
            .map(|it| alo_formats::zones::the_zones_in(it, language))
            .unwrap_or_default();
    }

    /// Every zone a person can choose, by the database's name.
    pub(crate) fn choices(&self) -> Vec<String> {
        self.zones.iter().map(|it| it.iana.clone()).collect()
    }

    /// What a zone is called: its city.
    pub(crate) fn called(&self, zone: &str) -> String {
        self.zones
            .iter()
            .find(|it| it.iana == zone)
            .map_or_else(|| zone.to_owned(), |it| it.name.clone())
    }

    /// The zone the machine is in, as last set or read.
    pub(crate) fn now(&self) -> Option<&str> {
        self.now.as_deref()
    }

    /// Whether the machine is in this zone.
    pub(crate) fn is_chosen(&self, zone: &str) -> bool {
        self.now.as_deref() == Some(zone)
    }

    /// What the section says above its rows: what the last change said, if
    /// anything. Settings heads no section, so this one is not headed either.
    pub(crate) fn said(&self) -> Vec<String> {
        self.told.clone().into_iter().collect()
    }

    /// **The person chose this zone**: the service is asked, and the tick
    /// moves only when it says yes.
    pub(crate) fn choose(
        &mut self,
        zone: &str,
        setter: &dyn SetsTheZone,
        strings: &Strings,
    ) -> SettingsKept {
        if !self.zones.iter().any(|it| it.iana == zone) {
            return SettingsKept::Refused;
        }
        match setter.set(zone) {
            Ok(()) => {
                self.now = Some(zone.to_owned());
                self.told = None;
                SettingsKept::Written
            }
            Err(why) => {
                eprintln!("alo-shell: the time zone {zone} was not set: {why}");
                self.told = Some(
                    strings
                        .say(
                            &alo_formats::words::ZONE_NOT_CHANGED.key(),
                            &Filling::nothing(),
                        )
                        .into_text(),
                );
                SettingsKept::NotWritten
            }
        }
    }
}

#[cfg(test)]
#[path = "settings_time_zone_tests.rs"]
mod tests;
