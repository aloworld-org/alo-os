//! The four things the status area shows, exactly as the crates that own them
//! said them.
//!
//! The clock, the battery, the network and the volume. **Not one of them is
//! measured here and not one of them is decided here** — each arrives as the
//! owning crate's own value, and this file's whole job is to hold them and to
//! say what is drawn from each without adding anything:
//!
//! | | Whose answer it is |
//! |---|---|
//! | the clock | `alo_formats::Regionally::time` wrote the text, in the person's language and region |
//! | the battery | `alo_power::Reading` — its own charge and its own charging state |
//! | the network | `alo_networks::Reaching` — `HowFar::reaches_anything` is the one place *connected* is decided |
//! | the volume | `alo_sound::Volume` |
//!
//! # Why the values arrive rather than are fetched
//!
//! A compositor that opened `/sys` to read a battery would be a compositor
//! measuring, and the status area *shows*. It is the same arrangement
//! `crate::lock_battery` is under, one surface over: the reading is handed in,
//! and the drawing cannot be wrong about a machine it never asked.
//!
//! # Two answers this file is careful not to give
//!
//! **A machine with no battery has none.** `alo_power::TheBattery::on_this_machine`
//! is an `Option`, and a desktop is that `None` — [`StatusItems::battery`] keeps
//! it as `None` so the raster can leave the item out. A battery drawn at zero
//! would say the machine is about to die; a desktop is not dying.
//!
//! **A machine that does not know whether it is metered is holding off.**
//! `alo_networks::Metered::should_hold_off` counts *nothing said* as metered,
//! and this asks that rather than reading the variant, so the status area
//! cannot come to disagree with the crate that decides it.

use alo_networks::Reaching;
use alo_power::Reading;
use alo_power::reading::Charging;
use alo_sound::Volume;

/// What the status area was handed, for one moment on one display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItems {
    /// The time as the person's region writes it, from `alo-formats`.
    clock: String,
    /// The battery, or [`None`] on a machine that has none.
    battery: Option<Reading>,
    /// How far this machine reaches, and whether it is metered.
    network: Reaching,
    /// How loud it is.
    volume: Volume,
}

impl StatusItems {
    /// The four readings, as they were handed over.
    pub fn shown(
        clock: String,
        battery: Option<Reading>,
        network: Reaching,
        volume: Volume,
    ) -> Self {
        Self {
            clock,
            battery,
            network,
            volume,
        }
    }

    /// The time, in the words `alo-formats` wrote.
    pub fn clock(&self) -> &str {
        &self.clock
    }

    /// The battery, or [`None`] where the machine has none.
    pub fn battery(&self) -> Option<&Reading> {
        self.battery.as_ref()
    }

    /// How full the battery is, as `alo_power::Charge` counts it, or [`None`]
    /// where there is no battery to be full.
    ///
    /// **Never `Some(0)` for an absent battery.** That is the whole reason this
    /// returns an `Option` rather than a number with a sentinel.
    pub(crate) fn battery_hundredths(&self) -> Option<u8> {
        self.battery.map(|reading| reading.charge().hundredths())
    }

    /// Whether power is going in, which is the one state drawn without colour.
    ///
    /// `Charging::Full` is not charging: a battery on the mains and full is not
    /// filling, and a mark that said it was would be this file inventing a
    /// state `alo-power` distinguishes.
    pub(crate) fn is_charging(&self) -> bool {
        self.battery
            .is_some_and(|reading| reading.charging() == Charging::Charging)
    }

    /// Whether this machine reaches anything, asked of the one place that
    /// decides it.
    pub(crate) fn reaches_anything(&self) -> bool {
        self.network.how_far.reaches_anything()
    }

    /// Whether this machine should hold off on what it does not have to send.
    ///
    /// Asked of `alo_networks::Metered::should_hold_off`, which counts *nothing
    /// said* as metered — so a status area cannot show *not metered* about a
    /// machine that does not know.
    pub(crate) fn should_hold_off(&self) -> bool {
        self.network.metered.should_hold_off()
    }

    /// How loud, as `alo_sound::Volume` counts it.
    pub(crate) fn volume_hundredths(&self) -> u16 {
        self.volume.hundredths()
    }

    /// How many items there are to draw, which is four on a laptop and three on
    /// a machine with no battery.
    pub(crate) fn how_many(&self) -> usize {
        3 + usize::from(self.battery.is_some())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_networks::{HowFar, Metered};
    use alo_power::reading::Charge;

    use super::*;

    /// A reading `alo-power` would produce.
    fn a_battery(hundredths: u8, charging: Charging) -> Reading {
        Reading::taken(
            Charge::reported(hundredths).expect("a charge a kernel could report"),
            charging,
            None,
            std::time::SystemTime::UNIX_EPOCH,
        )
    }

    /// A machine that reaches the internet and says nothing about metering.
    fn reaching() -> Reaching {
        Reaching::reported(HowFar::AllOfIt, Metered::NotSaid)
    }

    /// Everything a laptop hands over.
    fn a_laptop() -> StatusItems {
        StatusItems::shown(
            "09:41".to_owned(),
            Some(a_battery(64, Charging::Discharging)),
            reaching(),
            Volume::of(35).expect("a volume"),
        )
    }

    /// **The clock drawn is the text `alo-formats` wrote**, character for
    /// character, and nothing here reformats it.
    #[test]
    fn the_clock_is_the_text_the_region_wrote() {
        let written = alo_formats::Regionally::reading("en")
            .expect("a language this machine writes")
            .in_region("GB")
            .expect("a region this machine writes")
            .time(9, 41)
            .expect("a time");
        let items = StatusItems::shown(
            written.clone(),
            None,
            reaching(),
            Volume::of(0).expect("a volume"),
        );
        assert_eq!(items.clock(), written);
    }

    /// **The battery drawn is `alo-power`'s own charge**, and charging is its
    /// own state rather than a guess from the number.
    #[test]
    fn the_battery_is_the_readings_own_charge_and_state() {
        let items = a_laptop();
        assert_eq!(items.battery_hundredths(), Some(64));
        assert!(!items.is_charging(), "discharging was drawn as charging");

        let filling = StatusItems::shown(
            "09:41".to_owned(),
            Some(a_battery(64, Charging::Charging)),
            reaching(),
            Volume::of(0).expect("a volume"),
        );
        assert!(filling.is_charging());

        // Full on the mains is not filling, which `alo-power` distinguishes and
        // this must not flatten.
        let full = StatusItems::shown(
            "09:41".to_owned(),
            Some(a_battery(100, Charging::Full)),
            reaching(),
            Volume::of(0).expect("a volume"),
        );
        assert_eq!(full.battery_hundredths(), Some(100));
        assert!(!full.is_charging(), "a full battery was drawn as filling");
    }

    /// **A machine with no battery has none**, and never one at zero.
    #[test]
    fn a_machine_with_no_battery_draws_no_battery_rather_than_an_empty_one() {
        let desktop = StatusItems::shown(
            "09:41".to_owned(),
            None,
            reaching(),
            Volume::of(0).expect("a volume"),
        );
        assert_eq!(desktop.battery(), None);
        assert_eq!(
            desktop.battery_hundredths(),
            None,
            "an absent battery came back as a number, which would be drawn"
        );
        assert!(!desktop.is_charging());
        assert_eq!(desktop.how_many(), 3, "a desktop has three items, not four");
        assert_eq!(a_laptop().how_many(), 4);
    }

    /// **Connected is `HowFar::reaches_anything`'s answer**, not a second one
    /// made here.
    #[test]
    fn connected_is_the_one_answer_the_network_crate_gives() {
        for how_far in HowFar::EVERY {
            let items = StatusItems::shown(
                "09:41".to_owned(),
                None,
                Reaching::reported(how_far, Metered::NotSaid),
                Volume::of(0).expect("a volume"),
            );
            assert_eq!(
                items.reaches_anything(),
                how_far.reaches_anything(),
                "{how_far:?} was shown as something other than what the crate says"
            );
        }
    }

    /// **A machine that was told nothing about metering holds off**, because
    /// that is what `alo-networks` counts *nothing said* as.
    ///
    /// A status area that showed *not metered* here would be adding a claim of
    /// its own about somebody's data allowance.
    #[test]
    fn nothing_said_about_metering_is_holding_off() {
        let unknown = StatusItems::shown(
            "09:41".to_owned(),
            None,
            Reaching::reported(HowFar::AllOfIt, Metered::NotSaid),
            Volume::of(0).expect("a volume"),
        );
        assert_eq!(
            unknown.should_hold_off(),
            Metered::NotSaid.should_hold_off()
        );
        assert!(unknown.should_hold_off(), "nothing said was shown as free");
    }

    /// **The volume drawn is `alo-sound`'s own number.**
    #[test]
    fn the_volume_is_the_sound_crates_own_number() {
        for hundredths in [0, 1, 35, Volume::LOUDEST] {
            let volume = Volume::of(hundredths).expect("a volume");
            let items = StatusItems::shown("09:41".to_owned(), None, reaching(), volume);
            assert_eq!(items.volume_hundredths(), volume.hundredths());
        }
    }
}
