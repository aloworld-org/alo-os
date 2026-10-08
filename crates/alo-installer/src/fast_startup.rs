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
    ///
    /// **Absent is ordinary, not impossible.** The testing NUC had no such
    /// value at all on 2026-10-08 — only `HibernateEnabledDefault`, which is
    /// a default and not a state — while plainly hibernating.
    hibernate_enabled: Option<u32>,
    /// How many entries the enumeration that looked for the hibernation file
    /// found in the root of the system drive.
    ///
    /// **This is the control, and it is why the answer below can be believed.**
    /// The root of a Windows system drive always holds entries, so a count of
    /// zero is a broken instrument rather than an empty disk — and an
    /// instrument that cannot report its own failure reports *absent*, which is
    /// the one answer this reading must never give falsely.
    #[serde(default)]
    root_entries: u32,
    /// Whether `hiberfil.sys` was among them.
    ///
    /// Found by **enumerating the directory**, never by opening the file:
    /// Windows holds the hibernation image open, and `Test-Path` and
    /// `Get-Item` report a locked file as one that does not exist, with the
    /// same error a missing file gives
    /// (`docs/misreadings/a-locked-system-file-reads-as-a-file-that-does-not-exist.md`).
    #[serde(default)]
    hiberfil_sys: bool,
}

/// Whether this computer hibernates at all, which is the question Fast Startup
/// rests on.
///
/// [`None`] is *not found out*, and it is returned in three different
/// situations on purpose: nothing in the registry and no file found, the
/// enumeration having failed, and **the two readings disagreeing.**
fn hibernates(printed: &Printed) -> Option<bool> {
    // The file's answer, or nothing at all where the instrument that looked for
    // it cannot be believed.
    let the_file = (printed.root_entries > 0).then_some(printed.hiberfil_sys);
    match (printed.hibernate_enabled, the_file) {
        // Both agree it is off. This is the only road to *off*, and it needs
        // both, because an absence on its own is not a fact.
        (Some(0), Some(false)) => Some(false),
        // **They disagree**, and this installer does not pick a winner between
        // two readings of somebody's computer. A hibernation file beside a
        // value that says hibernation is off is a machine nobody here has
        // measured, and *could not be found out* is the true sentence for it.
        (Some(0), Some(true)) => None,
        // **The registry says off and the file could not be read.** Believed,
        // because that is what this reading said before it asked the file
        // system at all, and asking a new question is no reason to stop
        // trusting an answer that was already true. Without this arm the zero
        // falls through to *it hibernates* below, and a machine with
        // hibernation off is asked about something that cannot happen - which
        // `fast_startup_that_is_off_or_unread_is_not_asked_about` caught.
        (Some(0), None) => Some(false),
        // The registry says this computer hibernates.
        (Some(_), _) => Some(true),
        // Nothing in the registry, and the file answers it — the testing NUC.
        (None, Some(true)) => Some(true),
        // Nothing in the registry, and either no file or no instrument. The
        // file's absence is not evidence: see that field's own note.
        (None, Some(false) | None) => None,
    }
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
    ///
    /// **Whether it hibernates is asked of the file system as well as the
    /// registry**, because the registry alone could not answer it on the
    /// testing NUC. `hibernates` just above holds which combinations mean
    /// what, and why a disagreement is no answer rather than a chosen one; it
    /// is a private item, so this names it rather than linking to it.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Self {
        let Some(printed) =
            printed.and_then(|printed| serde_json::from_str::<Printed>(printed).ok())
        else {
            return Self::NotRead;
        };
        match (printed.hiberboot_enabled, hibernates(&printed)) {
            // The setting itself is off, and nothing else can turn it on.
            (Some(0), _) => Self::Off,
            // Or this computer cannot hibernate, so Fast Startup cannot happen.
            (_, Some(false)) => Self::Off,
            // Set, and the computer hibernates.
            (Some(_), Some(true)) => Self::On,
            // No value at all is Windows' own default, which is on — but
            // *read* is a stronger word than *worked out*, and this installer
            // says what it read.
            (None, _) | (Some(_), None) => Self::NotRead,
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
            r#"{"HiberbootEnabled":1,"HibernateEnabled":1,"RootEntries":9,"HiberfilSys":true}"#,
            r#"{"HiberbootEnabled":2,"HibernateEnabled":1,"RootEntries":9,"HiberfilSys":true}"#,
        ] {
            assert_eq!(FastStartup::read(Some(on)), FastStartup::On, "{on}");
        }
        for off in [
            r#"{"HiberbootEnabled":0,"HibernateEnabled":1,"RootEntries":9,"HiberfilSys":true}"#,
            // `powercfg /h off` leaves the first value at 1 (`docs/quirks.md`),
            // and takes the file away, so both readings agree.
            r#"{"HiberbootEnabled":1,"HibernateEnabled":0,"RootEntries":9,"HiberfilSys":false}"#,
            r#"{"HiberbootEnabled":0,"HibernateEnabled":0,"RootEntries":9,"HiberfilSys":false}"#,
        ] {
            assert_eq!(FastStartup::read(Some(off)), FastStartup::Off, "{off}");
        }
        for neither in [
            Some(
                r#"{"HiberbootEnabled":null,"HibernateEnabled":1,"RootEntries":9,"HiberfilSys":true}"#,
            ),
            // The second value absent and no file: nothing says whether this
            // computer hibernates, and an absent file is not evidence.
            Some(
                r#"{"HiberbootEnabled":1,"HibernateEnabled":null,"RootEntries":9,"HiberfilSys":false}"#,
            ),
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

    /// **The testing NUC, exactly as it reads.**
    ///
    /// Measured on that machine 2026-10-08, read-only: `HiberbootEnabled` 1,
    /// `HibernateEnabled` **absent**, and `C:\hiberfil.sys` present at
    /// 3,372,613,632 bytes.
    ///
    /// Before this reading asked the file system, those two registry values
    /// gave *could not be found out*, and the walk of 2026-10-07 recorded
    /// exactly that sentence with the question never asked. The machine
    /// hibernates and its Fast Startup is on, so the question is owed
    /// ([ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
    /// term 9).
    #[test]
    fn the_testing_nucs_own_reading_is_on_and_asks() {
        let it = r#"{"HiberbootEnabled":1,"HibernateEnabled":null,
          "RootEntries":11,"HiberfilSys":true}"#;
        assert_eq!(FastStartup::read(Some(it)), FastStartup::On);
        assert!(FastStartup::read(Some(it)).is_asked_about());

        // And without the file's answer it is the sentence that machine was
        // actually shown, which is what this change is about.
        let registry_only = r#"{"HiberbootEnabled":1,"HibernateEnabled":null,
          "RootEntries":11,"HiberfilSys":false}"#;
        assert_eq!(FastStartup::read(Some(registry_only)), FastStartup::NotRead);
    }

    /// **An enumeration that found nothing is a broken instrument, not an empty
    /// disk.**
    ///
    /// The root of a Windows system drive always holds entries. A count of zero
    /// means the thing that looked could not look, so its answer about the file
    /// is no answer — and the reading falls back to the registry rather than
    /// treating *absent* as a fact. An instrument that cannot report its own
    /// failure reports *absent*, which is the one answer this must never give
    /// falsely.
    #[test]
    fn an_enumeration_that_found_nothing_is_not_an_absent_file() {
        // The NUC's values, with the instrument broken: the file cannot be
        // believed either way, so the registry alone decides, and the registry
        // alone cannot.
        for broken in [
            r#"{"HiberbootEnabled":1,"HibernateEnabled":null,"RootEntries":0,"HiberfilSys":false}"#,
            r#"{"HiberbootEnabled":1,"HibernateEnabled":null,"RootEntries":0,"HiberfilSys":true}"#,
        ] {
            assert_eq!(
                FastStartup::read(Some(broken)),
                FastStartup::NotRead,
                "{broken}"
            );
        }

        // **And a missing control is the same as a zero one.** An older
        // release's script prints neither field, and a reading that defaulted
        // the file to *absent* and believed it would tell such a machine its
        // Fast Startup was off.
        let older = r#"{"HiberbootEnabled":1,"HibernateEnabled":null}"#;
        assert_eq!(FastStartup::read(Some(older)), FastStartup::NotRead);
    }

    /// **A registry zero is still believed when the file cannot be read.**
    ///
    /// This is the arm the existing suite caught missing. Before this reading
    /// asked the file system, `HibernateEnabled` of zero meant *off* on its
    /// own; an older release's script prints no file fields at all, and a
    /// machine read by one must not change answer because a newer reading
    /// exists. Asking a new question is no reason to stop trusting an answer
    /// that was already true.
    #[test]
    fn a_registry_zero_is_believed_when_the_file_cannot_be_read() {
        for off in [
            // No file fields at all, as an older script prints.
            r#"{"HiberbootEnabled":1,"HibernateEnabled":0}"#,
            // Or the fields are there and the instrument failed.
            r#"{"HiberbootEnabled":1,"HibernateEnabled":0,"RootEntries":0,"HiberfilSys":false}"#,
        ] {
            assert_eq!(FastStartup::read(Some(off)), FastStartup::Off, "{off}");
            assert!(!FastStartup::read(Some(off)).is_asked_about(), "{off}");
        }
    }

    /// **Two readings that disagree make no answer rather than a chosen one.**
    ///
    /// The registry saying hibernation is off while the hibernation file is
    /// there is a machine nobody here has measured. This installer does not
    /// pick a winner between two readings of somebody's computer: *could not be
    /// found out* is the true sentence, and it is the one that was already
    /// being said.
    ///
    /// Saying *off* would be believing the value over the disk. Saying *on*
    /// would put *Windows' Fast Startup is on* in front of a person while one
    /// of this program's own readings denies it.
    #[test]
    fn two_readings_that_disagree_are_not_an_answer() {
        let disagreeing =
            r#"{"HiberbootEnabled":1,"HibernateEnabled":0,"RootEntries":9,"HiberfilSys":true}"#;
        assert_eq!(
            FastStartup::read(Some(disagreeing)),
            FastStartup::NotRead,
            "a disagreement was resolved rather than reported"
        );
        assert!(!FastStartup::read(Some(disagreeing)).is_asked_about());

        // The setting being off still ends it, whatever the disk says: nothing
        // can turn Fast Startup on while its own value is zero.
        let settled =
            r#"{"HiberbootEnabled":0,"HibernateEnabled":0,"RootEntries":9,"HiberfilSys":true}"#;
        assert_eq!(FastStartup::read(Some(settled)), FastStartup::Off);
    }

    /// **Only *on* is asked about.**
    #[test]
    fn the_question_is_asked_only_when_it_is_on() {
        assert!(FastStartup::On.is_asked_about());
        assert!(!FastStartup::Off.is_asked_about());
        assert!(!FastStartup::NotRead.is_asked_about());
    }
}
