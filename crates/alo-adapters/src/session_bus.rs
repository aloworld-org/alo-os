//! The person's session bus, where an application answers its own interface.
//!
//! The one file in this crate that touches the machine. It sends a finished
//! [`Message`] as one method call to the application's own bus name and waits,
//! for at most [`HOW_LONG`], for the answer; everything it can hear back is one
//! of [`NotDelivered`]'s four. Nothing here decides what is sent — that was
//! decided by the declaration, the approval and `crate::driving` — and nothing
//! here is reached by a call that was not approved, because the only road to a
//! message is a `Driving`.
//!
//! Only on Linux: a session bus is a Linux session's.

use std::collections::HashMap;
use std::time::Duration;

use zbus::blocking::Connection;
use zbus::blocking::connection::Builder;
use zbus::zvariant::{StructureBuilder, Value};

use crate::delivering::{Delivers, NotDelivered};
use crate::message::{Argument, Message};

/// How long an application has to answer.
///
/// Long enough for an application the bus has to start first — opening a
/// document in a text editor that was not running — and short enough that a
/// person is told something rather than left looking at nothing.
pub const HOW_LONG: Duration = Duration::from_secs(20);

/// A session bus messages are sent on.
#[derive(Debug)]
pub struct SessionBus {
    /// The connection.
    connection: Connection,
}

impl SessionBus {
    /// The signed-in person's own session bus.
    ///
    /// # Errors
    /// [`NotDelivered::NotThere`] when there is no session bus to reach.
    pub fn this_persons() -> Result<Self, NotDelivered> {
        Builder::session()
            .map(|builder| builder.method_timeout(HOW_LONG))
            .and_then(Builder::build)
            .map(|connection| Self { connection })
            .map_err(|_| NotDelivered::NotThere)
    }

    /// The bus at this address — a test's own.
    ///
    /// # Errors
    /// [`NotDelivered::NotThere`] when nothing answers there.
    pub fn at(address: &str) -> Result<Self, NotDelivered> {
        Builder::address(address)
            .map(|builder| builder.method_timeout(HOW_LONG))
            .and_then(Builder::build)
            .map(|connection| Self { connection })
            .map_err(|_| NotDelivered::NotThere)
    }
}

impl Delivers for SessionBus {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        let answered = if message.arguments().is_empty() {
            self.connection.call_method(
                Some(message.destination()),
                message.object(),
                Some(message.interface()),
                message.method(),
                &(),
            )
        } else {
            let body = body_of(message.arguments()).ok_or(NotDelivered::DoesNotOffer)?;
            self.connection.call_method(
                Some(message.destination()),
                message.object(),
                Some(message.interface()),
                message.method(),
                &body,
            )
        };
        answered.map(|_| ()).map_err(|why| heard(&why))
    }
}

/// The parameters, as one structure whose fields become the call's arguments.
fn body_of(arguments: &[Argument]) -> Option<zbus::zvariant::Structure<'static>> {
    let mut fields = StructureBuilder::new();
    for argument in arguments {
        fields = match argument {
            Argument::Texts(texts) => fields.add_field(texts.clone()),
            Argument::Text(text) => fields.add_field(text.clone()),
            Argument::Number(number) => fields.add_field(*number),
            Argument::NoParameters => fields.add_field(Vec::<Value<'static>>::new()),
            Argument::NoPlatformData => fields.add_field(HashMap::<String, Value<'static>>::new()),
        };
    }
    fields.build().ok()
}

/// What an error from the bus means for a person.
fn heard(why: &zbus::Error) -> NotDelivered {
    match why {
        zbus::Error::MethodError(name, _, _) => {
            let name = name.as_str();
            let bus = |suffix: &str| name == format!("org.freedesktop.DBus.Error.{suffix}");
            if bus("ServiceUnknown")
                || bus("NameHasNoOwner")
                || name.starts_with("org.freedesktop.DBus.Error.Spawn")
            {
                NotDelivered::NotThere
            } else if bus("UnknownMethod")
                || bus("UnknownObject")
                || bus("UnknownInterface")
                || bus("InvalidArgs")
            {
                NotDelivered::DoesNotOffer
            } else if bus("NoReply") || bus("Timeout") || bus("TimedOut") {
                NotDelivered::DidNotAnswer
            } else {
                NotDelivered::Refused
            }
        }
        zbus::Error::InputOutput(io) if io.kind() == std::io::ErrorKind::TimedOut => {
            NotDelivered::DidNotAnswer
        }
        _ => NotDelivered::NotThere,
    }
}
