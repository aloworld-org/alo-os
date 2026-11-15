//! **How an application starts with a file alo was approved to open in it** —
//! the same road a press takes, with the entry's own file codes filled.
//!
//! *The owner, 2026-11-14: "Yes, add it"* — `open_file_in_application`. What
//! runs is still only the installed entry's own `Exec` line: the file is what
//! `%f`, `%F`, `%u` or `%U` become (`crate::the_command`), so a Flathub
//! application's exported entry hands it through its own
//! `--file-forwarding … @@ %F @@`, which is how Flatpak lets the sandbox reach
//! that one file. **Nothing alo wrote runs**, and no argument comes from the
//! agent but the path, which is one argument and begins with `/`.
//!
//! # An entry with nowhere to put a file is refused, not started without it
//!
//! An entry whose command has no file code would start the application and not
//! open the file, which is not what the person approved — so it is
//! [`WhyNotWithAFile::TakesNoFile`], and nothing is started. Over the bus the
//! file travels in `Open` as a `file://` address instead; the command carried
//! as `or_else` is kept only where it has the file in it.

use crate::how_it_starts::{HowItStarts, WhyItCannotStart, how_it_starts, road_for};
use crate::the_command::Filling;
use crate::the_entry::TheEntry;

/// Why an application cannot be started with a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyNotWithAFile {
    /// It cannot be started at all.
    CannotStart(WhyItCannotStart),
    /// Its `Exec` line takes no file — no `%f`, `%F`, `%u` or `%U` — so
    /// starting it would open the application and not the file.
    TakesNoFile,
}

/// Decide how this entry starts **with the file at `file`** — the same road
/// as [`how_it_starts`], with the file codes filled with it.
///
/// # Errors
///
/// [`WhyNotWithAFile`]: everything [`how_it_starts`] refuses, and an entry
/// whose command has nowhere to put a file.
pub fn how_it_starts_with_a_file(
    identifier: &str,
    entry: &TheEntry,
    location: Option<&str>,
    file: &str,
) -> Result<HowItStarts, WhyNotWithAFile> {
    let without =
        how_it_starts(identifier, entry, location).map_err(WhyNotWithAFile::CannotStart)?;
    let filling = Filling {
        icon: entry.icon.as_deref(),
        name: entry.name.as_deref(),
        location,
        link: None,
        file: Some(file),
    };
    let with = road_for(identifier, entry, &filling).map_err(WhyNotWithAFile::CannotStart)?;
    match (with, without) {
        (
            HowItStarts::OverTheBus { or_else: with_it },
            HowItStarts::OverTheBus {
                or_else: without_it,
            },
        ) => Ok(HowItStarts::OverTheBus {
            or_else: with_it.filter(|with_it| Some(with_it) != without_it.as_ref()),
        }),
        (HowItStarts::FromItsCommand(with_it), HowItStarts::FromItsCommand(without_it))
            if with_it == without_it =>
        {
            Err(WhyNotWithAFile::TakesNoFile)
        }
        (with, _) => Ok(with),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// An entry with this `Exec` line, started from its command unless
    /// `bus` says it is bus-activatable.
    fn entry(exec: &str, bus: bool) -> TheEntry {
        TheEntry::read(&format!(
            "[Desktop Entry]\nType=Application\nName=An application\nExec={exec}\n\
             DBusActivatable={bus}\n"
        ))
        .unwrap()
    }

    /// The file every test here opens.
    const HELLO: &str = "/var/home/anna/hello.txt";

    /// **VSCodium's own Flathub entry is started with the file between its
    /// forwarding markers**, which is how Flatpak hands the sandbox that file.
    #[test]
    fn a_flatpak_entry_is_started_with_the_file_it_forwards() {
        let vscodium = entry(
            "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=codium \
             --file-forwarding com.vscodium.codium @@ %F @@",
            false,
        );
        let road =
            how_it_starts_with_a_file("com.vscodium.codium", &vscodium, None, HELLO).unwrap();
        let HowItStarts::FromItsCommand(command) = road else {
            unreachable!("a command entry starts from its command: {road:?}")
        };
        let at = command.iter().position(|it| it == HELLO).unwrap();
        assert_eq!(command.get(at - 1).map(String::as_str), Some("@@"));
        assert_eq!(command.get(at + 1).map(String::as_str), Some("@@"));
    }

    /// **An entry with nowhere to put a file is refused**, rather than started
    /// without it.
    #[test]
    fn an_entry_that_takes_no_file_is_refused() {
        for exec in ["gnome-calculator", "app --new-window", "app %i"] {
            assert_eq!(
                how_it_starts_with_a_file("org.example.App", &entry(exec, false), None, HELLO),
                Err(WhyNotWithAFile::TakesNoFile),
                "{exec}"
            );
        }
    }

    /// **Over the bus the file travels in `Open`**, and a command to fall back
    /// on that has nowhere to put it is dropped, so a bus that cannot start the
    /// application never becomes it started without the file.
    #[test]
    fn over_the_bus_a_command_without_the_file_is_not_kept() {
        let without = how_it_starts_with_a_file(
            "org.gnome.TextEditor",
            &entry("gnome-text-editor", true),
            None,
            HELLO,
        )
        .unwrap();
        assert_eq!(without, HowItStarts::OverTheBus { or_else: None });
        let with = how_it_starts_with_a_file(
            "org.gnome.TextEditor",
            &entry("gnome-text-editor %U", true),
            None,
            HELLO,
        )
        .unwrap();
        assert_eq!(
            with,
            HowItStarts::OverTheBus {
                or_else: Some(vec!["gnome-text-editor".to_owned(), HELLO.to_owned()]),
            }
        );
    }

    /// **What cannot start at all is still refused as that.**
    #[test]
    fn an_entry_that_cannot_start_is_refused_as_that() {
        let terminal = TheEntry::read(
            "[Desktop Entry]\nType=Application\nName=Vim\nExec=vim %F\nTerminal=true\n",
        )
        .unwrap();
        assert_eq!(
            how_it_starts_with_a_file("vim", &terminal, None, HELLO),
            Err(WhyNotWithAFile::CannotStart(
                WhyItCannotStart::NeedsATerminal
            ))
        );
    }
}
