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
}

impl FastStartup {
    /// What the script printed, or [`FastStartup::NotRead`] for anything else.
    ///
    /// **Any value but zero is on.** Windows writes `1`, and a machine whose
    /// value somebody else set to something else has not turned it off.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        match printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok()) {
            Some(Printed {
                hiberboot_enabled: Some(0),
            }) => Self::Off,
            Some(Printed {
                hiberboot_enabled: Some(_),
            }) => Self::On,
            // No value at all is Windows' own default, which is on — but
            // *read* is a stronger word than *worked out*, and this installer
            // says what it read.
            Some(Printed {
                hiberboot_enabled: None,
            })
            | None => Self::NotRead,
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

    /// **Zero is off, anything else is on, and anything unreadable is neither.**
    #[test]
    fn the_value_is_read_as_windows_writes_it() {
        assert_eq!(
            FastStartup::read(Some(r#"{"HiberbootEnabled":1}"#)),
            FastStartup::On
        );
        assert_eq!(
            FastStartup::read(Some(r#"{"HiberbootEnabled":2}"#)),
            FastStartup::On
        );
        assert_eq!(
            FastStartup::read(Some(r#"{"HiberbootEnabled":0}"#)),
            FastStartup::Off
        );
        for neither in [
            Some(r#"{"HiberbootEnabled":null}"#),
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
