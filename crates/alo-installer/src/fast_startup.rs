//! Whether Windows' Fast Startup is on, read from Windows' own value.
//!
//! Fast Startup (*hiberboot*) makes Windows' *Shut down* hibernate the Windows
//! volume rather than close it, so the volume on the disk is a hibernated one
//! that another system must not write into. It is
//! `HiberbootEnabled` under `HKLM\SYSTEM\CurrentControlSet\Control\Session
//! Manager\Power`, and it is on by default: the Windows the installer's walk
//! installed read `1` (`docs/quirks.md`).
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! term 9 decided what the installer does about it: **it asks**. This file is
//! only the reading; `crate::asking` is the question, and the turning off is a
//! step of `crate::staging`.
//!
//! A value that could not be read is [`FastStartup::NotRead`] and never *off*:
//! the question is asked when it is known to be on, and a computer whose value
//! nobody could read is not one to change silently.

use serde::Deserialize;

/// What Windows' own value says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastStartup {
    /// `HiberbootEnabled` is not `0`.
    On,
    /// `HiberbootEnabled` is `0`.
    Off,
    /// The value could not be read.
    NotRead,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// The value, as Windows holds it; [`None`] when there is no such value.
    hiberboot_enabled: Option<u32>,
    /// Whether this computer hibernates at all; [`None`] when there is no such
    /// value.
    hibernate_enabled: Option<u32>,
}

impl FastStartup {
    /// What the script printed, or [`FastStartup::NotRead`] for anything else.
    ///
    /// **Any value but zero is on.** Windows writes `1`, and a machine whose
    /// value somebody else set to something else has not turned it off.
    ///
    /// **A computer that does not hibernate does not do Fast Startup**,
    /// whatever the first value says: Fast Startup is hibernation of the
    /// kernel's own session. `powercfg /h off` turns hibernation off and leaves
    /// `HiberbootEnabled` at 1 (`docs/quirks.md`), and a person on such a
    /// computer would otherwise be asked about something that cannot happen.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) =
            printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok())
        else {
            return Self::NotRead;
        };
        match (printed.hiberboot_enabled, printed.hibernate_enabled) {
            (Some(0), _) | (_, Some(0)) => Self::Off,
            (Some(_), Some(_)) => Self::On,
            // No value at all is Windows' own default, which is on — but
            // *read* is a stronger word than *worked out*, and this installer
            // says what it read.
            (None, _) | (_, None) => Self::NotRead,
        }
    }

    /// Whether the person is asked about it.
    #[must_use]
    pub const fn is_asked_about(self) -> bool {
        matches!(self, Self::On)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Zero is off, anything else is on, and anything unreadable is
    /// neither** — and a computer that does not hibernate is off whatever the
    /// first value says.
    #[test]
    fn the_value_is_read_as_windows_writes_it() {
        for on in [
            r#"{"HiberbootEnabled":1,"HibernateEnabled":1}"#,
            r#"{"HiberbootEnabled":2,"HibernateEnabled":1}"#,
        ] {
            assert_eq!(FastStartup::read(Some(on)), FastStartup::On, "{on}");
        }
        for off in [
            r#"{"HiberbootEnabled":0,"HibernateEnabled":1}"#,
            // `powercfg /h off` leaves the first value at 1 (`docs/quirks.md`).
            r#"{"HiberbootEnabled":1,"HibernateEnabled":0}"#,
            r#"{"HiberbootEnabled":0,"HibernateEnabled":0}"#,
        ] {
            assert_eq!(FastStartup::read(Some(off)), FastStartup::Off, "{off}");
        }
        for neither in [
            Some(r#"{"HiberbootEnabled":null,"HibernateEnabled":1}"#),
            Some(r#"{"HiberbootEnabled":1,"HibernateEnabled":null}"#),
            Some("{}"),
            Some("1"),
            Some(""),
            None,
        ] {
            assert_eq!(
                FastStartup::read(neither),
                FastStartup::NotRead,
                "{neither:?}"
            );
        }
    }

    /// **Only *on* is asked about.**
    #[test]
    fn the_question_is_asked_only_when_it_is_on() {
        assert!(FastStartup::On.is_asked_about());
        assert!(!FastStartup::Off.is_asked_about());
        assert!(!FastStartup::NotRead.is_asked_about());
    }
}
