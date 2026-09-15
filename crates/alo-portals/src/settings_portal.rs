//! `org.freedesktop.portal.Settings`, answered from what the person set.
//!
//! Version 2 of the interface, for the `org.freedesktop.appearance` namespace
//! only ([`crate::appearance_settings`]):
//!
//! - **`ReadAll(as namespaces) -> a{sa{sv}}`** — every answered key of the
//!   namespace, when a pattern reaches it; an empty dictionary when none does.
//! - **`ReadOne(s namespace, s key) -> v`** — one value.
//! - **`Read(s namespace, s key) -> v`** — the same value inside one more
//!   variant, as the deprecated method has always returned it. Served because
//!   applications written before `ReadOne` still call it, and it answers
//!   nothing `ReadOne` does not.
//! - **`SettingChanged(s namespace, s key, v value)`** — sent by
//!   `crate::watching_appearance`, to each application that could read the
//!   value at the moment it changed, and to nobody else.
//!
//! Nothing here asks a person anything, so every answer is the method's reply:
//! there is no request handle and no `Response`.
//!
//! # What a refused application receives
//!
//! **`org.freedesktop.portal.Error.NotFound`**, the one error the specification
//! gives for a setting that cannot be read — and the same error, in the same
//! words, that a namespace this machine does not answer receives. An
//! application not granted the appearance settings learns nothing from being
//! refused that it would not learn from asking for a setting that does not
//! exist. The record tells the two apart; the application is not told.
//!
//! Settings that could not be read — the grants, or the person's appearance —
//! are `org.freedesktop.portal.Error.Failed`, because nothing was refused.
//!
//! # In this order
//!
//! A namespace or key this machine does not answer is answered as unknown before
//! anything else, and never with a value. Then the caller is judged
//! ([`appearance_settings::allowed`]); only an allowed caller's answer reads the
//! person's settings ([`appearance_settings::read`]). Every answer, and every
//! refusal with its application named, is recorded before the reply is sent.

use std::collections::HashMap;
use std::time::SystemTime;

use zbus::message::Header;
use zbus::zvariant::{OwnedValue, Structure, Value as OnTheBus};

use crate::answered::{Answered, Outcome, Unanswered};
use crate::appearance_settings::{self, Setting, THE_NAMESPACE, Value, Values};
use crate::caller::{application_of, named};
use crate::portal::Portal;
use crate::serving::Backend;

/// The version of the interface this answers: the one with `ReadOne`.
const VERSION: u32 = 2;

/// What the specification's own backend says for a setting it will not read.
const NOT_FOUND: &str = "Requested setting not found";

/// The errors this interface answers with, under the portal's own names.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.portal.Error")]
pub(crate) enum SettingError {
    /// Something on the bus itself went wrong.
    #[zbus(error)]
    ZBus(zbus::Error),
    /// A setting that is not answered, or that this caller may not read.
    NotFound(String),
    /// What an answer is read from could not be read.
    Failed(String),
}

/// The Settings portal, served.
pub(crate) struct SettingsPortal {
    /// What it answers from.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.Settings")]
impl SettingsPortal {
    /// Every answered setting of every namespace `namespaces` reaches.
    async fn read_all(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        namespaces: Vec<String>,
    ) -> Result<HashMap<String, HashMap<String, OwnedValue>>, SettingError> {
        if !appearance_settings::asked_of(&namespaces) {
            self.unknown(connection, &header).await;
            return Ok(HashMap::new());
        }
        let values = self.answered(connection, &header).await?;
        let mut namespace = HashMap::new();
        for (setting, value) in values {
            namespace.insert(setting.key().to_owned(), on_the_bus(value)?);
        }
        Ok(HashMap::from([(THE_NAMESPACE.to_owned(), namespace)]))
    }

    /// One setting's value.
    async fn read_one(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        namespace: &str,
        key: &str,
    ) -> Result<OwnedValue, SettingError> {
        self.one(connection, &header, namespace, key).await
    }

    /// One setting's value, inside one more variant — the deprecated method.
    async fn read(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        namespace: &str,
        key: &str,
    ) -> Result<OwnedValue, SettingError> {
        let value = self.one(connection, &header, namespace, key).await?;
        OwnedValue::try_from(OnTheBus::Value(Box::new(value.into())))
            .map_err(|_| SettingError::Failed(NOT_FOUND.to_owned()))
    }

    /// A setting changed, sent to one application's connection at a time.
    #[zbus(signal)]
    pub(crate) async fn setting_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        namespace: &str,
        key: &str,
        value: OnTheBus<'_>,
    ) -> zbus::Result<()>;

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl SettingsPortal {
    /// `namespace` and `key`'s value for the caller, or the error it receives.
    async fn one(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
        namespace: &str,
        key: &str,
    ) -> Result<OwnedValue, SettingError> {
        let Some(setting) = Setting::named(namespace, key) else {
            self.unknown(connection, header).await;
            return Err(SettingError::NotFound(NOT_FOUND.to_owned()));
        };
        let values = self.answered(connection, header).await?;
        let value = values
            .into_iter()
            .find_map(|(each, value)| (each == setting).then_some(value))
            .ok_or_else(|| SettingError::NotFound(NOT_FOUND.to_owned()))?;
        on_the_bus(value)
    }

    /// Every answered setting's value, when the caller may read them — recorded
    /// either way before it is returned.
    async fn answered(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> Result<Values, SettingError> {
        let application = self.application(connection, header).await;
        let machine = self.backend.machine();
        let answer =
            appearance_settings::allowed(application.as_deref(), machine, SystemTime::now())
                .and_then(|allowed| {
                    appearance_settings::read(machine).map(|values| (allowed, values))
                });
        match answer {
            Ok((allowed, values)) => {
                self.keep(application.as_deref(), Outcome::AppearanceRead(allowed));
                Ok(values)
            }
            Err(outcome) => {
                let error = error_for(&outcome);
                self.keep(application.as_deref(), *outcome);
                Err(error)
            }
        }
    }

    /// Record a request for a setting this machine does not answer.
    async fn unknown(&self, connection: &zbus::Connection, header: &Header<'_>) {
        let application = self.application(connection, header).await;
        self.keep(
            application.as_deref(),
            Outcome::Unanswered(Unanswered::NoSuchSetting),
        );
    }

    /// The application the sender's sandbox names, if it has one.
    async fn application(
        &self,
        connection: &zbus::Connection,
        header: &Header<'_>,
    ) -> Option<String> {
        let sender = header.sender()?;
        application_of(connection, sender, &self.backend).await
    }

    /// Write this answer into the record.
    fn keep(&self, application: Option<&str>, outcome: Outcome) {
        self.backend.record().keep(Answered::new(
            SystemTime::now(),
            named(application),
            Portal::Settings,
            outcome,
        ));
    }
}

/// The error a caller receives for `outcome`, which was not an answer.
fn error_for(outcome: &Outcome) -> SettingError {
    match outcome {
        Outcome::Unanswered(Unanswered::GrantsUnread | Unanswered::AppearanceUnread) => {
            SettingError::Failed("The settings could not be read".to_owned())
        }
        _ => SettingError::NotFound(NOT_FOUND.to_owned()),
    }
}

/// A value as the specification writes it on the bus.
pub(crate) fn written(value: Value) -> OnTheBus<'static> {
    match value {
        Value::ColorScheme(scheme) => OnTheBus::U32(Value::color_scheme(scheme)),
        Value::AccentColor(colour) => {
            let (red, green, blue) = Value::accent_color(colour);
            OnTheBus::Structure(Structure::from((red, green, blue)))
        }
    }
}

/// A value as a method's reply carries it.
fn on_the_bus(value: Value) -> Result<OwnedValue, SettingError> {
    OwnedValue::try_from(written(value))
        .map_err(|_| SettingError::Failed("The settings could not be read".to_owned()))
}
