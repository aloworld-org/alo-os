//! The four readings a status area shows, taken from this machine.
//!
//! The clock, the battery, how far the machine reaches and how loud it is —
//! each from the entry point the crate that owns it already has, and **none of
//! them decided here**. What this file does is ask, and turn a refusal into the
//! absence that crate's own type already has a word for.
//!
//! # Why it is not in `alo-shell`
//!
//! Because the shell shows and does not measure. A compositor that opened
//! `/sys` would be a compositor measuring, and a status area on a laptop
//! showing a battery that never moves does not look unfinished — it looks
//! broken, and it is the one surface a person checks to find out whether their
//! machine is telling them the truth.
//!
//! # An absence is an answer, and it is not a default
//!
//! Every one of these can fail to be read, and a failure is **not** a zero. A
//! machine with no battery and a machine whose battery could not be read are
//! both `None` to the status area, which draws neither — but neither of them is
//! a battery at nought per cent, and a person seeing that would act on it. The
//! same for the volume: nothing asked is an absence, and a volume of nothing is
//! a claim that the machine is muted.
//!
//! What this file will not do is invent the difference. Where two causes give
//! one absence, that is the shape of the type the shell already has, and the
//! log says which cause it was.

use alo_formats::Regionally;
use alo_networks::{HowFar, Metered, Reaching};
use alo_power::TheBattery;
use alo_shell::StatusItems;

/// The moment a reading is taken at.
///
/// Passed in rather than read here, so that a test can move the clock and see
/// the drawn text move with it — the battery's own `read_at` exists for the
/// same reason and says so.
pub struct At {
    /// The hour, as this machine's clock has it.
    pub hour: u8,
    /// The minute.
    pub minute: u8,
    /// The moment itself, for the crates that take one.
    pub moment: std::time::SystemTime,
}

impl At {
    /// Now, on this machine's own clock.
    ///
    /// # Errors
    /// The sentence to log when the clock is one this machine cannot read.
    pub fn now() -> Result<Self, String> {
        let now = jiff::Zoned::now();
        Ok(Self {
            hour: u8::try_from(now.hour()).map_err(|_| "an hour outside a day".to_owned())?,
            minute: u8::try_from(now.minute())
                .map_err(|_| "a minute outside an hour".to_owned())?,
            moment: std::time::SystemTime::now(),
        })
    }
}

/// What a reading could not be taken, said once for the log.
///
/// Not shown to anybody: what a person sees is the absence itself, drawn the
/// way the status area draws every absence. This is for whoever is standing the
/// machine up and wants to know why their battery is missing.
#[derive(Debug, Default)]
pub struct WhatWasNotRead {
    /// Why there is no battery reading, when there is none.
    pub battery: Option<String>,
    /// Why the machine could not say how far it reaches.
    pub network: Option<String>,
    /// Why there is no volume.
    pub volume: Option<String>,
}

impl WhatWasNotRead {
    /// Every reason there is, in one line for a service log.
    #[must_use]
    pub fn said(&self) -> String {
        let mut said = Vec::new();
        for (what, why) in [
            ("battery", &self.battery),
            ("network", &self.network),
            ("volume", &self.volume),
        ] {
            if let Some(why) = why {
                said.push(format!("{what}: {why}"));
            }
        }
        if said.is_empty() {
            "every reading was taken".to_owned()
        } else {
            said.join("; ")
        }
    }
}

/// Take all four, and say what could not be taken.
///
/// Never fails as a whole: a machine with no battery, no media server and no
/// network manager still has a clock and still shows a status area. What it
/// cannot read is absent, and the reasons come back beside the readings.
///
/// # Errors
/// Only the clock, which is the one reading with no absence the status area can
/// draw — a time is a `String` there, and a status area with no time on it is
/// not a thing this decides to show.
pub fn taken(at: &At, region: &Regionally) -> Result<(StatusItems, WhatWasNotRead), String> {
    let clock = region
        .time(at.hour, at.minute)
        .ok_or_else(|| format!("a time nobody writes: {}:{}", at.hour, at.minute))?;
    let mut missing = WhatWasNotRead::default();

    let battery = match TheBattery::on_this_machine() {
        None => {
            missing.battery = Some("this machine has no battery".to_owned());
            None
        }
        Some(battery) => match battery.read_at(at.moment) {
            Ok(reading) => Some(reading),
            Err(why) => {
                missing.battery = Some(why.to_string());
                None
            }
        },
    };

    // **A network nobody could ask is *nothing said*, not *nothing reached*.**
    // `alo-networks` has the word for it, and the difference matters: a machine
    // that cannot ask its network manager is not a machine with no network.
    let network = match how_far() {
        Ok(reaching) => reaching,
        Err(why) => {
            missing.network = Some(why);
            Reaching::reported(HowFar::NotSaid, Metered::NotSaid)
        }
    };

    let volume = match how_loud() {
        Ok(volume) => Some(volume),
        Err(why) => {
            missing.volume = Some(why);
            None
        }
    };

    Ok((StatusItems::shown(clock, battery, network, volume), missing))
}

/// How far this machine reaches, as its network manager says it.
fn how_far() -> Result<Reaching, String> {
    use alo_networks::WhatIsReached as _;
    let manager = alo_networks::network_manager::NetworkManager::on_this_machine()
        .map_err(|why| format!("the network manager: {why}"))?;
    manager
        .reaching_now()
        .map_err(|why| format!("the network manager: {why}"))
}

/// How loud this machine is, as its media server says it.
fn how_loud() -> Result<alo_sound::Volume, String> {
    use alo_sound::Sound as _;
    let mut server = alo_sound::TheAudioServer::on_this_machine();
    let heard = server
        .heard_now()
        .map_err(|why| format!("the media server: {why}"))?;
    volume_of(&heard).ok_or_else(|| "the server is sending sound to nothing".to_owned())
}

/// The volume of the device the server is sending sound to.
///
/// **The chosen output's, not the loudest of everything plugged in.** A machine
/// with headphones and speakers has two volumes and a person hears one of them,
/// and `alo-sound` already answers which — a machine with no speakers is a real
/// machine rather than an error, and that is an absence like any other.
fn volume_of(heard: &alo_sound::Heard) -> Option<alo_sound::Volume> {
    let chosen = heard.chosen(alo_sound::Kind::Output)?;
    Some(heard.doing(chosen)?.volume())
}

#[cfg(test)]
#[path = "readings_tests.rs"]
mod tests;
