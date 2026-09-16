//! How a verb reaches its application, declared as data.
//!
//! **The adapter never names where a message goes.** A method is sent to the
//! application the adapter is for, by its identifier — the name an installed
//! application holds on the session bus — so an adapter cannot declare a verb
//! that reaches another service, the desktop's shell or the bus itself. What it
//! declares is the object, the interface, the method, and each parameter in
//! order, and every parameter is one of [`Part`]'s closed list.
//!
//! A parameter is a literal the adapter's author wrote, an empty value the
//! interface requires, or a validated argument placed in a typed slot.
//! [`Part::Evaluated`] exists so a declaration can say honestly that the
//! application interprets what arrives there, and it is never loaded.

/// How one verb is carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invocation {
    /// One method call on the application's own bus name.
    DBus(DBusMethod),
}

/// One method call, sent to the adapter's application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DBusMethod {
    /// The object the method is called on, like `/org/gnome/TextEditor`.
    pub object: &'static str,
    /// The interface, like `org.freedesktop.Application`.
    pub interface: &'static str,
    /// The method, like `Open`.
    pub method: &'static str,
    /// Its parameters, in order.
    pub parameters: &'static [Part],
}

/// One parameter of a method call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// A list of one file address, made from a path argument (`as`).
    FileAddressOf(&'static str),
    /// Text the adapter's author wrote, never a model (`s`).
    Literal(&'static str),
    /// A name, or a chosen option's name, from an argument (`s`).
    TextOf(&'static str),
    /// A number, from an argument (`x`).
    CountOf(&'static str),
    /// An empty list of parameters (`av`).
    NoParameters,
    /// Empty platform data (`a{sv}`).
    NoPlatformData,
    /// An argument the application **interprets** — as code, an expression or
    /// a command. Declarable so a declaration can be honest; never loaded.
    Evaluated {
        /// The argument.
        argument: &'static str,
        /// What interprets it, like `python`.
        by: &'static str,
    },
}

impl Part {
    /// The argument this parameter is filled from, if any.
    #[must_use]
    pub const fn argument(self) -> Option<&'static str> {
        match self {
            Self::FileAddressOf(argument)
            | Self::TextOf(argument)
            | Self::CountOf(argument)
            | Self::Evaluated { argument, .. } => Some(argument),
            Self::Literal(_) | Self::NoParameters | Self::NoPlatformData => None,
        }
    }
}
