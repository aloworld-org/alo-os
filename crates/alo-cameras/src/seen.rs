//! The media server's record, read for cameras.
//!
//! The record itself is `alo-media-server`'s — one crate owns reaching the
//! server and reading what it says. What is here is what those objects mean when
//! the question is **what can see**.
//!
//! The same record `alo-sound` reads for sound and `alo-in-use` reads for what
//! is in use, asked a third question: **what can see.** Nothing here is patched
//! or worked around (ADR 0011).
//!
//! # A camera is a running-nothing until somebody opens it
//!
//! What is read here is the list of cameras this machine **has**, not what is
//! using them — that is `alo-in-use`'s, it is a different question, and two
//! crates answering it would answer it differently one day.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::reported::{CameraId, OfItsOwn, OneCamera, TheCameras};

/// What the server calls a camera.
const A_CAMERA: &str = "Video/Source";

/// **What this machine can see with, and the road to each one's hardware.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seen {
    /// The cameras.
    cameras: TheCameras,
    /// Which device file each is behind, where the machine said. Used to let go
    /// of a device and for nothing else — never as an identity (ADR 0040).
    behind: BTreeMap<CameraId, String>,
}

impl Seen {
    /// The cameras this machine has.
    #[must_use]
    pub const fn cameras(&self) -> &TheCameras {
        &self.cameras
    }

    /// The device file this camera is behind, where the machine said which.
    #[must_use]
    pub fn behind(&self, camera: &CameraId) -> Option<&str> {
        self.behind.get(camera).map(String::as_str)
    }

    /// Every device file, which is what *off for everyone* has to reach.
    pub fn every_device_file(&self) -> impl Iterator<Item = &str> {
        self.behind.values().map(String::as_str)
    }
}

/// Why the machine's cameras could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotSeen {
    /// Nothing on this machine answers for devices.
    #[error("nothing on this machine says what it can see with: {0}")]
    NothingAnswers(String),
    /// It is there and it did not answer.
    #[error("this machine's media server was asked about cameras and did not answer: {0}")]
    NoAnswer(String),
    /// It answered something this crate cannot read.
    #[error("this machine's media server answered something this crate cannot read: {0}")]
    NotUnderstood(String),
}

/// **Read the cameras out of the server's record.**
///
/// # Errors
/// [`NotSeen::NotUnderstood`] where the record is not the list of objects this
/// crate reads. A camera inside it that cannot be read — no name, or a name that
/// is a device number — is passed over rather than failing the whole reading.
pub fn in_the_record(objects: &[Value]) -> Result<Seen, NotSeen> {
    let mut cameras = Vec::new();
    let mut behind = BTreeMap::new();
    for object in objects {
        let Some((camera, device_file)) = a_camera_in(object) else {
            continue;
        };
        if let Some(device_file) = device_file {
            behind.insert(camera.identity().clone(), device_file);
        }
        cameras.push(camera);
    }
    Ok(Seen {
        cameras: TheCameras::reported(cameras),
        behind,
    })
}

/// One object read as a camera, with the device file it is behind.
fn a_camera_in(object: &Value) -> Option<(OneCamera, Option<String>)> {
    if !object.get("type")?.as_str()?.ends_with(":Node") {
        return None;
    }
    let props = object.get("info")?.get("props")?;
    if props.get("media.class")?.as_str()? != A_CAMERA {
        return None;
    }
    let identity = CameraId::reported(props.get("node.name")?.as_str()?).ok()?;
    let called = props
        .get("node.description")
        .or_else(|| props.get("node.nick"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let device_file = props
        .get("api.v4l2.path")
        .and_then(Value::as_str)
        .map(str::to_owned);

    // What a camera has of its own is what the device says, and neither the
    // kernel nor the media server says anything about a shutter or a light. So
    // this machine says it cannot tell, which is true — and is the only honest
    // thing to put beside a light a person can see with their own eyes.
    Some((
        OneCamera::reported(
            identity,
            called,
            OfItsOwn::ThisMachineCannotTell,
            OfItsOwn::ThisMachineCannotTell,
        ),
        device_file,
    ))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A record, read the way a machine's is.
    fn objects(record: &str) -> alo_media_server::TheRecord {
        alo_media_server::read(record).expect("a record this crate reads")
    }

    const A_RECORD: &str = r#"[
      { "id": 49, "type": "PipeWire:Interface:Node",
        "info": { "state": "suspended",
          "props": { "media.class": "Video/Source", "node.name": "v4l2_input.platform-vivid.0",
                     "node.description": "vivid (V4L2)", "api.v4l2.path": "/dev/video0" } } },
      { "id": 50, "type": "PipeWire:Interface:Node",
        "info": { "state": "running",
          "props": { "media.class": "Audio/Source", "node.name": "a-microphone" } } },
      { "id": 51, "type": "PipeWire:Interface:Node",
        "info": { "state": "running",
          "props": { "media.class": "Stream/Input/Video", "node.name": "a-video-call" } } }
    ]"#;

    /// **Cameras are read, and a program reading one is not a camera.**
    #[test]
    fn a_camera_is_read_and_a_microphone_and_a_program_are_not() {
        let seen = in_the_record(objects(A_RECORD).objects()).expect("a record this crate reads");
        assert_eq!(seen.cameras().every().len(), 1);
        let one = seen
            .cameras()
            .every()
            .first()
            .expect("the camera in the record");
        assert_eq!(one.identity().as_str(), "v4l2_input.platform-vivid.0");
        assert_eq!(one.called(), "vivid (V4L2)");
        assert_eq!(one.shutter(), OfItsOwn::ThisMachineCannotTell);
    }

    /// **The device file is a road and not an identity**: it is kept beside the
    /// camera, never as the camera.
    #[test]
    fn the_device_number_is_kept_as_a_road_to_the_hardware_and_nothing_else() {
        let seen = in_the_record(objects(A_RECORD).objects()).expect("a record this crate reads");
        let one = CameraId::reported("v4l2_input.platform-vivid.0").expect("an identity");
        assert_eq!(seen.behind(&one), Some("/dev/video0"));
        assert!(
            !one.as_str().contains("/dev/"),
            "the identity is a device number, which is the thing ADR 0040 is about"
        );
        assert_eq!(seen.every_device_file().count(), 1);
    }

    /// **An empty record is a machine with no cameras**, which is an answer
    /// rather than a refusal.
    ///
    /// Text that is not a record at all is refused a step earlier, by
    /// `alo-media-server`, which is the crate that reads one — and its own tests
    /// hold that a machine whose record will not parse never reads as a machine
    /// with nothing on it.
    #[test]
    fn an_empty_record_is_a_machine_with_no_cameras_rather_than_a_refusal() {
        assert!(
            in_the_record(objects("[]").objects())
                .expect("an empty record is a machine with no cameras")
                .cameras()
                .every()
                .is_empty()
        );
    }
}
