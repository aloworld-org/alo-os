//! Private capture transport. This carries choices and pixels, never grants them.
//!
//! The compositor admits only the authenticated portal service. The portal
//! identifies the app, obtains a fresh native choice, judges its grant, and
//! records the answer. Neither a source ID nor this protocol is an agent verb.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{self, Read, Write};

#[cfg(target_os = "linux")]
mod descriptor;
#[cfg(target_os = "linux")]
pub use descriptor::{receive_descriptor, send_descriptor};

/// Maximum catalogue or request size, before deserialising client input.
pub const MAX_MESSAGE_BYTES: usize = 65_536;
/// Bound the work and native dropdown entries in one source catalogue.
pub const MAX_SOURCES: usize = 64;
/// Maximum owned BGRx frame; it covers an 8K display without unbounded allocation.
pub const MAX_FRAME_BYTES: usize = 256 * 1024 * 1024;
/// The socket beside this session's Wayland socket, inside its private directory.
pub const CAPTURE_SOCKET: &str = "capture";

/// A source the compositor can actually render in isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// A complete output belonging to the signed-in person's desktop.
    Monitor,
    /// One application window, never a crop of the complete desktop.
    Window,
}

/// An opaque source handle, bound to the current compositor and object lifetime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Fresh identity; never selected again by a recycled title or output name.
    pub id: String,
    /// Display text for the trusted native chooser, with no executable meaning.
    pub name: String,
    /// What the person can choose to share.
    pub kind: SourceKind,
    /// Native width of this specific source.
    pub width: u32,
    /// Native height of this specific source.
    pub height: u32,
}

/// The only requests admitted by the compositor's private capture endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Request {
    /// Describe only this person's current capturable sources.
    Sources {},
    /// Open the exact source the person just selected; closing ends the stream.
    Start {
        /// The exact lifetime-bound handle returned in the catalogue.
        source: String,
    },
}

/// One bounded reply; a started stream is followed by tightly packed BGRx frames.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "response", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Response {
    /// Sources currently available for a fresh native choice.
    Sources {
        /// At most [`MAX_SOURCES`] actual objects, never guessed placeholders.
        sources: Vec<Source>,
    },
    /// Pixel dimensions are fixed for the lifetime of the following frame stream.
    Started {
        /// The native width selected by the person.
        width: u32,
        /// The native height selected by the person.
        height: u32,
    },
    /// The selected object disappeared or cannot be captured.
    Unavailable {},
}

/// The actual media node, including its non-recycled PipeWire object serial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaNode {
    /// The global node ID needed by a consuming PipeWire client.
    pub id: u32,
    /// A replaced node at the same ID cannot satisfy this selection.
    pub serial: u64,
}

/// The allocation size of one native BGRx frame, checked before allocating.
///
/// # Errors
/// Zero dimensions, an unsupported side, overflow, or a frame over the memory bound.
pub fn frame_bytes(width: u32, height: u32) -> io::Result<usize> {
    let bytes = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4));
    match bytes {
        Some(bytes)
            if width > 0
                && height > 0
                && width <= 16_384
                && height <= 16_384
                && bytes <= MAX_FRAME_BYTES =>
        {
            Ok(bytes)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "capture dimensions exceed the frame bound",
        )),
    }
}

/// Write one bounded message and its delimiter, without buffering private frames.
///
/// # Errors
/// An oversized or unencodable message, or the transport's own write failure.
pub fn write_message(writer: &mut impl Write, message: &impl Serialize) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(message).map_err(io::Error::other)?;
    if bytes.len() >= MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "capture message is too large",
        ));
    }
    bytes.push(b'\n');
    writer.write_all(&bytes)
}

/// Read exactly one bounded message, leaving the first raw frame byte unread.
///
/// # Errors
/// An oversized, malformed, unknown, or unfinished message; or a read failure.
pub fn read_message<T: DeserializeOwned>(reader: &mut impl Read) -> io::Result<T> {
    let mut bytes = Vec::with_capacity(128);
    for _ in 0..MAX_MESSAGE_BYTES {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        if byte == *b"\n" {
            return serde_json::from_slice(&bytes).map_err(io::Error::other);
        }
        bytes.extend(byte);
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "capture message is too large",
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "an unexpected transport refusal is this test's failure"
)]
mod tests {
    use super::*;
    #[test]
    fn dimensions_are_checked_before_a_frame_can_allocate() {
        assert_eq!(frame_bytes(7680, 4320).unwrap(), 132_710_400);
        assert!(frame_bytes(0, 1080).is_err());
        assert!(frame_bytes(u32::MAX, u32::MAX).is_err());
        assert!(frame_bytes(16_384, 16_384).is_err());
    }
    #[test]
    fn a_header_does_not_consume_the_first_pixel_or_accept_extra_fields() {
        let mut bytes = Vec::new();
        write_message(
            &mut bytes,
            &Response::Started {
                width: 1280,
                height: 720,
            },
        )
        .unwrap();
        bytes.extend_from_slice(&[12, 34, 56, 0]);
        let mut input = bytes.as_slice();
        assert_eq!(
            read_message::<Response>(&mut input).unwrap(),
            Response::Started {
                width: 1280,
                height: 720
            }
        );
        assert_eq!(input, [12, 34, 56, 0]);
        assert!(
            read_message::<Request>(
                &mut b"{\"request\":\"sources\",\"application\":\"forged\"}\n".as_slice()
            )
            .is_err()
        );
        assert!(
            read_message::<Request>(&mut vec![b'x'; MAX_MESSAGE_BYTES + 1].as_slice()).is_err()
        );
    }
}
