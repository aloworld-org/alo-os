//! This machine's own media server, reached.
//!
//! [`TheAudioServer`] is [`Sound`] on a real machine: it asks the rented media
//! server for its record and tells it, through the server's own tools, which
//! device to use and how loud it is. Nothing about the server is patched,
//! wrapped or extended (ADR 0011).
//!
//! # The server's own tools rather than a library binding
//!
//! The record is asked for and the change is made through the tools the media
//! server ships, for the reason `alo-in-use` gives about the same server and the
//! same record: they are part of the thing we rent, they say exactly what the
//! server holds, and they keep a C library out of every process that wants to
//! show a volume slider. A binding would be the same answer with a build
//! dependency and an ABI in front of it.
//!
//! # No shell, no environment of the caller's, and the C locale
//!
//! Each program is started directly, so no argument is interpreted by anything
//! but the tool. Its environment is cleared and given back three things: where
//! this machine's own programs are, the C locale — what the tools print is read
//! to decide things, and a translated field name is a record nothing recognises
//! — and **where this machine's media server is listening**, which is the one
//! variable a session sets and without which every tool would report a machine
//! with no sound at all.
//!
//! # And the policy is WirePlumber's
//!
//! *Use this device* is said to the server and the server moves what is playing,
//! including a call already running. This crate does not relink a graph, and a
//! version of it that did would be a second policy quietly disagreeing with the
//! one the machine ships (the plan's constraint, in one sentence).

use std::io::ErrorKind;
use std::process::Command;

use alo_media_server::{ATool, AsksIt, NotAsked};

use crate::asking::Sound;
use crate::device::{Identity, Kind, Volume};
use crate::heard::{self, Heard};
use crate::mute::Mute;
use crate::refusing::{NotDone, NotHeard};

/// The session manager's own tool for changing what a device is doing.
const TELL_WITH: &str = "wpctl";

/// **This machine's media server.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheAudioServer {
    /// The server, asked for its record by the crate that owns asking.
    asking: alo_media_server::TheMediaServer,
    /// The program told to change something.
    telling: String,
}

impl Default for TheAudioServer {
    fn default() -> Self {
        Self {
            asking: alo_media_server::TheMediaServer::on_this_machine(),
            telling: TELL_WITH.to_owned(),
        }
    }
}

/// What one of `alo-media-server`'s four facts means to a list of sound
/// devices.
///
/// Two of them are one sentence here: a machine with no tool and a machine with
/// no server running both have no sound this crate can list, and the difference
/// is in the diagnosis for whoever is fixing it.
fn what_it_means(why: NotAsked) -> NotHeard {
    let said = why.said().to_owned();
    match why {
        NotAsked::NothingHandlesIt { .. } | NotAsked::NoServerIsRunning { .. } => {
            NotHeard::NothingHandlesSound { said }
        }
        NotAsked::ItWouldNotAnswer { .. } => NotHeard::NoAnswer { said },
        NotAsked::ItAnsweredSomethingUnreadable { .. } => NotHeard::NotUnderstood { said },
    }
}

impl TheAudioServer {
    /// The media server on the machine this is running on.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// One of the server's tools, started under `alo-media-server`'s rules:
    /// nothing of the caller's environment but where the machine's programs
    /// are, the C locale and where the server is listening.
    fn tool(named: &str) -> Command {
        ATool::named(named).started_here()
    }

    /// The server's own number for a device that is here, or a refusal naming
    /// the device that is not.
    fn number_of(&mut self, identity: &Identity) -> Result<u32, NotDone> {
        let heard = self.heard_now()?;
        heard
            .doing(identity)
            .map(heard::AtTheServer::number)
            .ok_or_else(|| NotDone::NotHere {
                identity: identity.as_str().to_owned(),
            })
    }

    /// Tell the machine one thing, and say what happened if it refused.
    fn tell(&mut self, doing: &str, arguments: &[String]) -> Result<(), NotDone> {
        let answered = Self::tool(&self.telling).args(arguments).output();
        match answered {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(NotDone::Refused {
                doing: doing.to_owned(),
                said: said_in(&output.stderr, &output.stdout),
            }),
            Err(why) if why.kind() == ErrorKind::NotFound => Err(NotDone::NothingToTellItWith {
                said: format!("{} is not on this machine", self.telling),
            }),
            Err(why) => Err(NotDone::Refused {
                doing: doing.to_owned(),
                said: why.to_string(),
            }),
        }
    }
}

impl Sound for TheAudioServer {
    fn heard_now(&mut self) -> Result<Heard, NotHeard> {
        let record = self.asking.record().map_err(what_it_means)?;
        heard::in_the_record(record.objects())
    }

    fn choose(&mut self, identity: &Identity, kind: Kind) -> Result<(), NotDone> {
        // The kind is the device's own, and saying it here would let a caller
        // ask for an output to be used as a microphone. It is read back from
        // the record instead, and a device that is not of the kind asked for is
        // not here for this purpose.
        let heard = self.heard_now()?;
        let is_of_that_kind = heard
            .devices()
            .of_kind(kind)
            .any(|device| device.identity() == identity);
        let number = heard.doing(identity).map(heard::AtTheServer::number);
        match (is_of_that_kind, number) {
            (true, Some(number)) => self.tell(
                &format!("use {} for sound", identity.as_str()),
                &["set-default".to_owned(), number.to_string()],
            ),
            _ => Err(NotDone::NotHere {
                identity: identity.as_str().to_owned(),
            }),
        }
    }

    fn set_volume(&mut self, identity: &Identity, volume: Volume) -> Result<(), NotDone> {
        let number = self.number_of(identity)?;
        self.tell(
            &format!("set the volume of {}", identity.as_str()),
            &[
                "set-volume".to_owned(),
                number.to_string(),
                format!("{:.2}", heard::as_the_tool_takes_it(volume)),
            ],
        )
    }

    fn set_mute(&mut self, identity: &Identity, mute: Mute) -> Result<(), NotDone> {
        let number = self.number_of(identity)?;
        let to = match mute {
            Mute::On => "1",
            Mute::Off => "0",
        };
        self.tell(
            &format!("mute {}", identity.as_str()),
            &["set-mute".to_owned(), number.to_string(), to.to_owned()],
        )
    }
}

/// What a tool said when it failed: whatever it wrote, wherever it wrote it.
fn said_in(instead: &[u8], said: &[u8]) -> String {
    let instead = String::from_utf8_lossy(instead).trim().to_owned();
    if instead.is_empty() {
        return String::from_utf8_lossy(said).trim().to_owned();
    }
    instead
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A machine with no media server says so rather than reading as a
    /// machine with no devices.** The difference is the whole of it: *you have
    /// no headphones plugged in* and *this could not be asked* are two
    /// different things to be told, and only one of them is fixed by plugging
    /// something in.
    #[test]
    fn a_machine_with_no_media_server_refuses_rather_than_reading_as_no_devices() {
        let mut nowhere = TheAudioServer {
            asking: alo_media_server::TheMediaServer::reached_by("alo-sound-no-such-tool"),
            telling: "alo-sound-no-such-tool".to_owned(),
        };
        assert!(matches!(
            nowhere.heard_now(),
            Err(NotHeard::NothingHandlesSound { .. })
        ));
        let identity = Identity::reported("anything").expect("named");
        assert!(matches!(
            nowhere.choose(&identity, Kind::Output),
            Err(NotDone::NotHeard(NotHeard::NothingHandlesSound { .. }))
        ));
        assert!(matches!(
            nowhere.set_mute(&identity, Mute::On),
            Err(NotDone::NotHeard(NotHeard::NothingHandlesSound { .. }))
        ));
    }

    /// **Every refusal has something in it for whoever is fixing the machine.**
    #[test]
    fn a_refusal_names_what_could_not_be_done() {
        let mut nowhere = TheAudioServer {
            asking: alo_media_server::TheMediaServer::reached_by("alo-sound-no-such-tool"),
            telling: "alo-sound-no-such-tool".to_owned(),
        };
        let why = nowhere
            .heard_now()
            .expect_err("a tool that is not there cannot answer");
        assert!(why.diagnosis().contains("alo-sound-no-such-tool"));
    }

    /// **What a tool said is read from wherever it wrote it**, because a tool
    /// that fails quietly on one stream and explains itself on the other is
    /// common and a refusal with nothing in it helps nobody.
    #[test]
    fn what_a_tool_said_is_read_from_either_place_it_wrote_it() {
        assert_eq!(said_in(b"  no such node  ", b""), "no such node");
        assert_eq!(said_in(b"", b"  no such node  "), "no such node");
        assert_eq!(said_in(b"", b""), "");
    }
}
