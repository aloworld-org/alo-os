//! The one message an approved verb becomes, as data.
//!
//! Built from three things and nothing else: the application the adapter is
//! for (where it goes), the declared method (what is called), and the validated
//! values of the call a person approved (what fills it). **No text a model sent
//! reaches a message except through a value `alo-capability` validated**, and
//! each value lands in the typed slot its declaration names.
//!
//! Kept apart from sending it, so what an approval becomes can be read and
//! tested on any machine, and the bus is only ever handed a finished message.

use alo_capability::{Call, Value};

use crate::file_address;
use crate::invocation::{DBusMethod, Part};

/// One parameter of a message, typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Argument {
    /// A list of text (`as`).
    Texts(Vec<String>),
    /// Text (`s`).
    Text(String),
    /// A whole number (`x`).
    Number(i64),
    /// An empty list of parameters (`av`).
    NoParameters,
    /// Empty platform data (`a{sv}`).
    NoPlatformData,
}

/// One method call, ready to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The bus name it goes to: the application's identifier.
    destination: String,
    /// The object.
    object: &'static str,
    /// The interface.
    interface: &'static str,
    /// The method.
    method: &'static str,
    /// The parameters, in order.
    arguments: Vec<Argument>,
}

impl Message {
    /// The message a call becomes, or [`None`] when a value is missing or of a
    /// kind its parameter cannot hold — which a loaded adapter cannot cause,
    /// and which sends nothing if it ever does.
    #[must_use]
    pub fn of(application: &str, method: &DBusMethod, call: &Call) -> Option<Self> {
        let arguments = method
            .parameters
            .iter()
            .map(|part| filled(*part, call))
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            destination: application.to_owned(),
            object: method.object,
            interface: method.interface,
            method: method.method,
            arguments,
        })
    }

    /// The bus name it goes to.
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }

    /// The object.
    #[must_use]
    pub fn object(&self) -> &'static str {
        self.object
    }

    /// The interface.
    #[must_use]
    pub fn interface(&self) -> &'static str {
        self.interface
    }

    /// The method.
    #[must_use]
    pub fn method(&self) -> &'static str {
        self.method
    }

    /// The parameters, in order.
    #[must_use]
    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }
}

/// One parameter, filled.
fn filled(part: Part, call: &Call) -> Option<Argument> {
    match part {
        Part::FileAddressOf(argument) => match call.value(argument)? {
            Value::Path(path) => {
                file_address::of(path).map(|address| Argument::Texts(vec![address]))
            }
            _ => None,
        },
        Part::Literal(text) => Some(Argument::Text(text.to_owned())),
        Part::TextOf(argument) => match call.value(argument)? {
            Value::Name(name) => Some(Argument::Text(name.clone())),
            Value::Choice { chosen, .. } => Some(Argument::Text(chosen.clone())),
            _ => None,
        },
        Part::CountOf(argument) => match call.value(argument)? {
            Value::Count(number) => Some(Argument::Number(*number)),
            _ => None,
        },
        Part::NoParameters => Some(Argument::NoParameters),
        Part::NoPlatformData => Some(Argument::NoPlatformData),
        // Never loaded; and if it were, nothing is built from it.
        Part::Evaluated { .. } => None,
    }
}
