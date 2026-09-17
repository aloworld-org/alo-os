//! Machines to decide against, for tests.
//!
//! Every decision this crate makes is made against [`crate::asking::Sound`], so
//! every one of them can be tested on a build host with no sound card at all.
//! What cannot be tested that way — that a real server moves a real call when it
//! is told to — is in `tests/`, on a machine.
//!
//! This is `#[doc(hidden)]` and it is here rather than in a test file because
//! the crate's own tests and the on-a-machine tests both build machines, and two
//! copies of a fixture drift until one of them is testing something nobody
//! meant.

#![doc(hidden)]

use crate::asking::Sound;
use crate::device::{Identity, Kind, Volume};
use crate::heard::{self, Heard};
use crate::mute::Mute;
use crate::refusing::{NotDone, NotHeard};

/// A machine with these outputs, one of them chosen.
#[doc(hidden)]
#[must_use]
pub fn a_machine_with(outputs: &[&str], chosen: Option<&str>) -> Heard {
    let devices: Vec<(&str, Kind)> = outputs.iter().map(|name| (*name, Kind::Output)).collect();
    a_machine(&devices, chosen, None)
}

/// A machine with these devices, and these chosen.
///
/// Built by writing a record in the rented server's own shape and reading it
/// back through [`heard::in_the_record`], so that a fixture can never be a
/// shape the real server does not produce.
#[doc(hidden)]
#[must_use]
pub fn a_machine(
    devices: &[(&str, Kind)],
    chosen_output: Option<&str>,
    chosen_input: Option<&str>,
) -> Heard {
    let mut objects = Vec::new();
    for (number, (name, kind)) in devices.iter().enumerate() {
        let class = match kind {
            Kind::Output => "Audio/Sink",
            Kind::Input => "Audio/Source",
        };
        objects.push(format!(
            r#"{{ "id": {number}, "type": "PipeWire:Interface:Node",
                  "info": {{ "state": "idle",
                    "props": {{ "media.class": "{class}", "node.name": "{name}",
                                "node.description": "{name}" }},
                    "params": {{ "Props": [ {{ "mute": false,
                                              "channelVolumes": [ 1.0 ] }} ] }} }} }}"#
        ));
    }
    let mut chosen = Vec::new();
    if let Some(output) = chosen_output {
        chosen.push(format!(
            r#"{{ "key": "default.audio.sink", "value": {{ "name": "{output}" }} }}"#
        ));
    }
    if let Some(input) = chosen_input {
        chosen.push(format!(
            r#"{{ "key": "default.audio.source", "value": {{ "name": "{input}" }} }}"#
        ));
    }
    objects.push(format!(
        r#"{{ "id": 900, "type": "PipeWire:Interface:Metadata",
              "props": {{ "metadata.name": "default" }},
              "metadata": [ {} ] }}"#,
        chosen.join(", ")
    ));
    let record = format!("[ {} ]", objects.join(", "));
    let read = alo_media_server::read(&record).unwrap_or_default();
    heard::in_the_record(read.objects()).unwrap_or_default()
}

/// **What a machine was told**, one line each.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    /// Use this device for this kind.
    Use(String, Kind),
    /// Set this device to this volume.
    Volume(String, Volume),
    /// Mute or unmute this device.
    Muted(String, Mute),
}

/// A machine that remembers what it was told and never has a sound card.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct AMachine {
    /// What it says when it is asked.
    heard: Heard,
    /// What it has been told, in order.
    told: Vec<Told>,
}

impl AMachine {
    /// A machine that answers with this record.
    #[doc(hidden)]
    #[must_use]
    pub const fn answering(heard: Heard) -> Self {
        Self {
            heard,
            told: Vec::new(),
        }
    }

    /// What it has been told, in order.
    #[doc(hidden)]
    #[must_use]
    pub fn told(&self) -> &[Told] {
        &self.told
    }

    /// A device that is here, or a refusal naming the one that is not.
    fn here(&self, identity: &Identity) -> Result<(), NotDone> {
        if self.heard.doing(identity).is_some() {
            return Ok(());
        }
        Err(NotDone::NotHere {
            identity: identity.as_str().to_owned(),
        })
    }
}

impl Sound for AMachine {
    fn heard_now(&mut self) -> Result<Heard, NotHeard> {
        Ok(self.heard.clone())
    }

    fn choose(&mut self, identity: &Identity, kind: Kind) -> Result<(), NotDone> {
        self.here(identity)?;
        self.told
            .push(Told::Use(identity.as_str().to_owned(), kind));
        Ok(())
    }

    fn set_volume(&mut self, identity: &Identity, volume: Volume) -> Result<(), NotDone> {
        self.here(identity)?;
        self.told
            .push(Told::Volume(identity.as_str().to_owned(), volume));
        Ok(())
    }

    fn set_mute(&mut self, identity: &Identity, mute: Mute) -> Result<(), NotDone> {
        self.here(identity)?;
        self.told
            .push(Told::Muted(identity.as_str().to_owned(), mute));
        Ok(())
    }
}
