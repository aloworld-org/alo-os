//! The media server's own record of the machine's sound, read.
//!
//! The rented media server keeps the graph of everything this machine can play
//! through or listen with, and will write it out as a list of objects. This file
//! is the only one in this crate that knows how the server writes things down.
//! Nothing here is patched or worked around (ADR 0011: the media server is
//! rented, configured, and never written).
//!
//! # What is read, and what is passed over
//!
//! | What the record says a node is | What this crate makes of it |
//! |---|---|
//! | an audio sink | an output |
//! | an audio source | an input |
//! | anything else — streams, ports, links, devices, clients | passed over |
//!
//! A **stream** is a program playing or recording, not a device, and the give
//! away is in the class the server writes: a device's class begins `Audio/`, a
//! program's begins `Stream/`. Counting a program as a device would put every
//! video call in the list of things a person can pin.
//!
//! # The identity is the server's name for the device, and that is deliberate
//!
//! The record has two numbers and a name for every device: the object id, which
//! changes every time the device is plugged in again, a serial, which also
//! changes, and the name, which the server derives from what the hardware says
//! about itself. **Only the name survives a replug**, so the name is
//! [`Identity`] and the id is a detail this crate carries only long enough to
//! ask the server to change something. `tests/a_device_replugged_is_the_same_device.rs`
//! holds that on a real server: unplug and plug in again, and the id is new
//! while the name is the one that was pinned.
//!
//! # Volume as the server keeps it
//!
//! The server keeps channel volumes on a cubic scale — what a person hears as
//! half is 0.125 written down — because that is how loudness works rather than
//! how numbers do. A person's *sixty* is turned back into the server's scale
//! where it is set, and the server's number is turned into a person's here, and
//! the conversion is in one place for that reason.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::device::{Devices, Identity, Kind, OneDevice, Volume};
use crate::mute::Mute;
use crate::refusing::NotHeard;

/// What the server calls the object that holds what is chosen.
const WHERE_THE_CHOICE_IS: &str = "default";

/// The key the chosen output is under.
const THE_CHOSEN_OUTPUT: &str = "default.audio.sink";

/// The key the chosen input is under.
const THE_CHOSEN_INPUT: &str = "default.audio.source";

/// **What one device is doing at the server right now.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtTheServer {
    /// The server's own number for it, which a replug changes.
    number: u32,
    /// How loud it is, as a person would say it.
    volume: Volume,
    /// Whether it is muted.
    mute: Mute,
    /// Whether something is going through it now.
    carrying: bool,
}

impl AtTheServer {
    /// The server's own number for this device, for asking it to change
    /// something. Never kept: it is a different number tomorrow.
    #[must_use]
    pub const fn number(&self) -> u32 {
        self.number
    }

    /// How loud it is.
    #[must_use]
    pub const fn volume(&self) -> Volume {
        self.volume
    }

    /// Whether it is muted.
    #[must_use]
    pub const fn mute(&self) -> Mute {
        self.mute
    }

    /// **Whether something is going through it now** — the *which one each is
    /// in use for* half of the plan, read from the server rather than guessed
    /// from what was chosen: a chosen device with nothing playing is not in use.
    #[must_use]
    pub const fn carrying(&self) -> bool {
        self.carrying
    }
}

/// **Everything the server says about this machine's sound at one moment.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Heard {
    /// The devices, in the order the server listed them.
    devices: Devices,
    /// What each is doing, by identity.
    doing: BTreeMap<Identity, AtTheServer>,
    /// The output the server is sending sound to.
    chosen_output: Option<Identity>,
    /// The input the server is taking sound from.
    chosen_input: Option<Identity>,
}

impl Heard {
    /// The devices this machine has.
    #[must_use]
    pub const fn devices(&self) -> &Devices {
        &self.devices
    }

    /// What one device is doing, or [`None`] where it is not here.
    #[must_use]
    pub fn doing(&self, identity: &Identity) -> Option<&AtTheServer> {
        self.doing.get(identity)
    }

    /// **Which device the server is using for this kind**, or [`None`] where it
    /// has none — a machine with no speakers is a real machine, not an error.
    #[must_use]
    pub const fn chosen(&self, kind: Kind) -> Option<&Identity> {
        match kind {
            Kind::Output => self.chosen_output.as_ref(),
            Kind::Input => self.chosen_input.as_ref(),
        }
    }
}

/// **Read the server's record.**
///
/// # Errors
/// [`NotHeard::NotUnderstood`] where the record is not the list of objects this
/// crate knows how to read. A device inside it that cannot be read — no name,
/// a volume past loud — is passed over rather than failing the whole reading:
/// one strange device must not cost a person the list of the other three.
pub fn in_the_record(record: &str) -> Result<Heard, NotHeard> {
    let objects = everything_in(record, |said| NotHeard::NotUnderstood { said })?;
    let objects = &objects;

    let mut devices = Vec::new();
    let mut doing = BTreeMap::new();
    let mut heard = Heard::default();

    for object in objects {
        if let Some((device, at_the_server)) = a_device_in(object) {
            doing.insert(device.identity().clone(), at_the_server);
            devices.push(device);
        }
        read_the_choice_in(object, &mut heard);
    }

    heard.devices = Devices::reported(devices);
    heard.doing = doing;
    Ok(heard)
}

/// **Everything the server listed**, as one list of objects.
///
/// The record is one JSON list on an ordinary reading. It is read as a **stream**
/// of lists because on 2026-09-17 a gate read one with trailing characters after
/// the end of the list, and every reading of the same machine a moment later
/// parsed as one list. What produced it was not caught, so nothing here claims
/// to know; what a reader must not do is turn it into *this machine answered
/// something unreadable*, which takes a list of devices off a screen for a
/// reason nobody can act on. An object listed twice is taken as it was listed
/// last. `docs/quirks.md` records what was seen.
fn everything_in(record: &str, said: impl Fn(String) -> NotHeard) -> Result<Vec<Value>, NotHeard> {
    let mut objects: Vec<Value> = Vec::new();
    for read in serde_json::Deserializer::from_str(record).into_iter::<Value>() {
        let read =
            read.map_err(|why| said(format!("it is not a record this crate reads: {why}")))?;
        let Value::Array(listed) = read else {
            return Err(said("the record is not a list of objects".to_owned()));
        };
        for object in listed {
            let same = object.get("id").and_then(Value::as_u64).and_then(|id| {
                objects
                    .iter()
                    .position(|seen| seen.get("id").and_then(Value::as_u64) == Some(id))
            });
            match same {
                Some(at) => {
                    if let Some(held) = objects.get_mut(at) {
                        *held = object;
                    }
                }
                None => objects.push(object),
            }
        }
    }
    Ok(objects)
}

/// One object read as a device, or [`None`] where it is not one.
fn a_device_in(object: &Value) -> Option<(OneDevice, AtTheServer)> {
    if !object.get("type")?.as_str()?.ends_with(":Node") {
        return None;
    }
    let info = object.get("info")?;
    let props = info.get("props")?;
    let kind = match props.get("media.class")?.as_str()? {
        class if class.starts_with("Audio/Sink") => Kind::Output,
        class if class.starts_with("Audio/Source") => Kind::Input,
        _ => return None,
    };
    let identity = Identity::reported(props.get("node.name")?.as_str()?).ok()?;
    let called = props
        .get("node.description")
        .or_else(|| props.get("node.nick"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let number = u32::try_from(object.get("id")?.as_u64()?).ok()?;

    let (volume, mute) = how_loud_and_whether_muted(info);
    let carrying = info.get("state").and_then(Value::as_str) == Some("running");

    Some((
        OneDevice::reported(identity, called, kind),
        AtTheServer {
            number,
            volume,
            mute,
            carrying,
        },
    ))
}

/// How loud a node is and whether it is muted, from the properties the server
/// keeps on it. A node that says nothing about either is read as loud and not
/// muted, which is what a device with no volume control of its own is.
fn how_loud_and_whether_muted(info: &Value) -> (Volume, Mute) {
    let Some(props) = info
        .get("params")
        .and_then(|params| params.get("Props"))
        .and_then(Value::as_array)
        .and_then(|props| props.first())
    else {
        return (Volume::loudest(), Mute::Off);
    };
    let loudest_channel = props
        .get("channelVolumes")
        .and_then(Value::as_array)
        .map(|channels| {
            channels
                .iter()
                .filter_map(Value::as_f64)
                .fold(0.0_f64, f64::max)
        })
        .or_else(|| props.get("volume").and_then(Value::as_f64))
        .unwrap_or(1.0);
    let mute = if props.get("mute").and_then(Value::as_bool) == Some(true) {
        Mute::On
    } else {
        Mute::Off
    };
    (as_a_person_would_say_it(loudest_channel), mute)
}

/// The server's cubic number, as a person would say it.
fn as_a_person_would_say_it(cubic: f64) -> Volume {
    let hundredths = (cubic.max(0.0).cbrt() * 100.0).round();
    if hundredths >= f64::from(Volume::LOUDEST) {
        return Volume::loudest();
    }
    Volume::of(hundredths as u16).unwrap_or_else(|_| Volume::loudest())
}

/// A person's number, in the scale the server's own tool takes.
///
/// The tool takes *a person's* scale — sixty hundredths is `0.60` — and does the
/// cubic conversion itself, which is why this is a division and not a cube.
#[must_use]
pub fn as_the_tool_takes_it(volume: Volume) -> f64 {
    f64::from(volume.hundredths()) / 100.0
}

/// The chosen output and input, where this object is the one that holds them.
fn read_the_choice_in(object: &Value, into: &mut Heard) {
    let is_the_choice = object
        .get("props")
        .and_then(|props| props.get("metadata.name"))
        .and_then(Value::as_str)
        == Some(WHERE_THE_CHOICE_IS);
    if !is_the_choice {
        return;
    }
    let Some(entries) = object.get("metadata").and_then(Value::as_array) else {
        return;
    };
    for entry in entries {
        let named = entry
            .get("value")
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            .and_then(|name| Identity::reported(name).ok());
        match entry.get("key").and_then(Value::as_str) {
            Some(THE_CHOSEN_OUTPUT) => into.chosen_output = named,
            Some(THE_CHOSEN_INPUT) => into.chosen_input = named,
            _ => {}
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A record shaped like the rented server's own, kept short. Every field
    /// read above is in it, and so is one of each thing that must be passed
    /// over.
    const A_RECORD: &str = r#"[
      { "id": 36, "type": "PipeWire:Interface:Metadata",
        "props": { "metadata.name": "default" },
        "metadata": [
          { "subject": 0, "key": "default.audio.sink", "value": { "name": "the-headset" } },
          { "subject": 0, "key": "default.audio.source", "value": { "name": "the-headset-mic" } }
        ] },
      { "id": 48, "type": "PipeWire:Interface:Node",
        "info": { "state": "running",
          "props": { "media.class": "Audio/Sink", "node.name": "the-headset",
                     "node.description": "A headset" },
          "params": { "Props": [ { "volume": 1.0, "mute": false,
                                   "channelVolumes": [ 0.216, 0.216 ] } ] } } },
      { "id": 49, "type": "PipeWire:Interface:Node",
        "info": { "state": "suspended",
          "props": { "media.class": "Audio/Source", "node.name": "the-headset-mic",
                     "node.description": "A headset microphone" },
          "params": { "Props": [ { "volume": 1.0, "mute": true,
                                   "channelVolumes": [ 1.0, 1.0 ] } ] } } },
      { "id": 60, "type": "PipeWire:Interface:Node",
        "info": { "state": "running",
          "props": { "media.class": "Stream/Output/Audio", "node.name": "a-video-call" } } },
      { "id": 33, "type": "PipeWire:Interface:Device",
        "info": { "props": { "device.name": "a-card" } } },
      { "id": 61, "type": "PipeWire:Interface:Port", "info": { "props": {} } }
    ]"#;

    /// **The devices are read and everything else is passed over.**
    #[test]
    fn the_outputs_and_the_inputs_are_read_and_a_program_is_not_a_device() {
        let heard = in_the_record(A_RECORD).expect("a record this crate reads");
        assert_eq!(heard.devices().all().len(), 2);
        assert_eq!(heard.devices().of_kind(Kind::Output).count(), 1);
        assert_eq!(heard.devices().of_kind(Kind::Input).count(), 1);
        assert!(
            !heard
                .devices()
                .all()
                .iter()
                .any(|device| device.identity().as_str() == "a-video-call"),
            "a program playing sound was listed as a device a person could pin"
        );
    }

    /// **What is chosen is read from the server**, by the name that survives a
    /// replug rather than by the number that does not.
    #[test]
    fn what_the_server_chose_is_read_by_name() {
        let heard = in_the_record(A_RECORD).expect("a record this crate reads");
        assert_eq!(
            heard.chosen(Kind::Output).map(Identity::as_str),
            Some("the-headset")
        );
        assert_eq!(
            heard.chosen(Kind::Input).map(Identity::as_str),
            Some("the-headset-mic")
        );
    }

    /// **Volume, mute and whether it is carrying anything are read per device**,
    /// and the cubic scale the server keeps is turned into a person's.
    #[test]
    fn each_device_is_read_with_its_volume_its_mute_and_whether_it_is_in_use() {
        let heard = in_the_record(A_RECORD).expect("a record this crate reads");
        let headset = Identity::reported("the-headset").expect("named");
        let microphone = Identity::reported("the-headset-mic").expect("named");

        let out = heard.doing(&headset).expect("the headset is in the record");
        assert_eq!(out.number(), 48);
        assert_eq!(out.volume().hundredths(), 60, "0.216 cubic is sixty");
        assert_eq!(out.mute(), Mute::Off);
        assert!(out.carrying(), "a running sink is in use");

        let mic = heard
            .doing(&microphone)
            .expect("the microphone is in the record");
        assert_eq!(mic.mute(), Mute::On);
        assert!(!mic.carrying(), "a suspended source is not in use");
    }

    /// **A record that is not one says so** rather than reading as a machine
    /// with no sound devices, which is the failure that matters: *nothing is
    /// listed* and *nothing could be asked* are different sentences.
    #[test]
    fn something_that_is_not_a_record_is_refused_rather_than_read_as_silence() {
        assert!(matches!(
            in_the_record("not a record at all"),
            Err(NotHeard::NotUnderstood { .. })
        ));
        assert!(matches!(
            in_the_record("{\"id\": 1}"),
            Err(NotHeard::NotUnderstood { .. })
        ));
        let empty = in_the_record("[]").expect("an empty record is a machine with no devices");
        assert!(empty.devices().all().is_empty());
        assert!(empty.chosen(Kind::Output).is_none());
    }

    /// **A device the server named nothing is passed over**, and the ones
    /// beside it are still read.
    #[test]
    fn a_device_with_no_name_does_not_cost_a_person_the_others() {
        let record = r#"[
          { "id": 1, "type": "PipeWire:Interface:Node",
            "info": { "state": "idle", "props": { "media.class": "Audio/Sink" } } },
          { "id": 2, "type": "PipeWire:Interface:Node",
            "info": { "state": "idle",
              "props": { "media.class": "Audio/Sink", "node.name": "speakers" } } }
        ]"#;
        let heard = in_the_record(record).expect("a record this crate reads");
        assert_eq!(heard.devices().all().len(), 1);
        let speakers = Identity::reported("speakers").expect("named");
        assert_eq!(
            heard.doing(&speakers).map(AtTheServer::volume),
            Some(Volume::loudest()),
            "a device the server says nothing about is loud and not muted"
        );
    }

    /// **The two scales are the same conversion in both directions.**
    #[test]
    fn a_persons_number_and_the_servers_are_the_same_number() {
        for hundredths in [0, 1, 40, 60, 99, 100] {
            let volume = Volume::of(hundredths).expect("a volume");
            let there_and_back =
                as_a_person_would_say_it(as_the_tool_takes_it(volume).powi(3)).hundredths();
            assert_eq!(there_and_back, hundredths, "at {hundredths} hundredths");
        }
    }
}
