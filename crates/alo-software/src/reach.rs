//! What an application can reach past its box the moment it is installed,
//! read from what its makers ask for.
//!
//! # Why this exists
//!
//! The market said *each one … arrives able to reach nothing*. Measured on
//! 2026-11-17 against the permissions in Flathub's own metadata for every
//! catalogued application (`/api/v2/summary/<id>`): **not one** of the 62 does.
//! 48 reach a person's files without asking and 8 can start programs outside
//! their box; the seven Craft applications ask for both. 63 of the 69 use the
//! internet, each Craft bundle's own metadata read the same day. Nothing on
//! this machine overrides what an application's makers ask for, so what they
//! ask for is what it gets.
//!
//! The owner decided the same day: *list them with a plain warning.* So each
//! catalogue entry records what its makers' real metadata asks for, read by
//! [`Reach::asked_for`] — one rule, applied to every application alike — and the
//! market's row and the approval card say it in words.
//!
//! # The three this names
//!
//! - [`Reach::ProgramsOutsideItsBox`]: it may talk to `org.freedesktop.Flatpak`,
//!   which starts programs on the machine outside the box. An application that
//!   can do that can do anything the person can.
//! - [`Reach::YourFiles`]: it may open the person's whole home, the whole
//!   machine, or one of their own folders (Documents, Downloads, Pictures,
//!   Music, Videos, the desktop) without asking.
//!
//! - [`Reach::TheInternet`]: it may use the network (`shared=network`). 56 of
//!   the 62 from Flathub ask for it, and all seven Craft applications.
//!
//! **The internet is said on the approval card, never as a row's label.** A
//! label on nearly every row would say nothing, so the line under the market's
//! list says it of all of them; the owner decided on 2026-11-17 that the card a
//! person answers before an installation names it with everything else the
//! application can reach.

use serde::Deserialize;

/// One thing an application can reach past its box as soon as it is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reach {
    /// It can start programs on this computer outside its box.
    ProgramsOutsideItsBox,
    /// It can read and change the person's files without asking.
    YourFiles,
    /// It can use the internet.
    TheInternet,
}

impl Reach {
    /// What an application's makers ask for reaches past its box, read from
    /// its metadata's `filesystems`, session-bus `talk` names and `shared`
    /// — the rule every catalogue entry's `reaches` was written by.
    #[must_use]
    pub fn asked_for(filesystems: &[&str], talks_to: &[&str], shared: &[&str]) -> Vec<Self> {
        let mut reaches = Vec::new();
        if talks_to.iter().any(|name| {
            *name == "org.freedesktop.Flatpak" || name.starts_with("org.freedesktop.Flatpak.")
        }) {
            reaches.push(Self::ProgramsOutsideItsBox);
        }
        if filesystems.iter().any(|one| is_the_persons_files(one)) {
            reaches.push(Self::YourFiles);
        }
        if shared.contains(&"network") {
            reaches.push(Self::TheInternet);
        }
        reaches
    }
}

/// Whether one `filesystems` entry opens the person's own files.
fn is_the_persons_files(entry: &str) -> bool {
    let place = entry.split(':').next().unwrap_or(entry);
    let whole = matches!(place, "host" | "host-os" | "host-etc" | "home" | "~");
    let theirs = [
        "xdg-documents",
        "xdg-download",
        "xdg-pictures",
        "xdg-music",
        "xdg-videos",
        "xdg-desktop",
    ]
    .iter()
    .any(|folder| place == *folder || place.starts_with(&format!("{folder}/")));
    whole || theirs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Read from real metadata**: GIMP's, OBS Studio's and PhotoCraft's own
    /// permissions as their makers publish them (Flathub's summary and the
    /// Craft manifest at tag v0.23.1, 2026-11-17).
    #[test]
    fn what_real_applications_ask_for_is_read_as_they_ask_it() {
        let gimp = Reach::asked_for(
            &[
                "xdg-config/GIMP:create",
                "xdg-config/gtk-3.0:ro",
                "/tmp",
                "xdg-run/gvfsd",
                "host",
            ],
            &[
                "com.canonical.AppMenu.Registrar",
                "org.gtk.vfs.*",
                "org.freedesktop.FileManager1",
            ],
            &["network", "ipc"],
        );
        assert_eq!(gimp, vec![Reach::YourFiles, Reach::TheInternet]);

        let photocraft = Reach::asked_for(
            &[
                "xdg-data/ArtCraft/versions:ro",
                "~/.cups:ro",
                "xdg-pictures",
                "xdg-documents",
            ],
            &["org.freedesktop.Flatpak"],
            &["network", "ipc"],
        );
        assert_eq!(
            photocraft,
            vec![
                Reach::ProgramsOutsideItsBox,
                Reach::YourFiles,
                Reach::TheInternet
            ]
        );

        // RawTherapee's own: its files, and no network.
        let rawtherapee = Reach::asked_for(&["host"], &[], &["ipc"]);
        assert_eq!(rawtherapee, vec![Reach::YourFiles]);
    }

    /// **An application that asks for nothing past its box reaches nothing**,
    /// and its own configuration, a cache, a portal or a printer's settings
    /// are not the person's files.
    #[test]
    fn what_stays_inside_the_box_is_not_a_reach() {
        assert!(Reach::asked_for(&[], &[], &[]).is_empty());
        // Sharing with the machine's other programs is not the internet.
        assert!(Reach::asked_for(&[], &[], &["ipc"]).is_empty());
        assert_eq!(
            Reach::asked_for(&[], &[], &["network"]),
            vec![Reach::TheInternet]
        );
        assert!(
            Reach::asked_for(
                &[
                    "xdg-config/GIMP:create",
                    "~/.cups:ro",
                    "xdg-run/gvfs",
                    "/tmp",
                    "xdg-data/x:ro"
                ],
                &[
                    "org.freedesktop.portal.Flatpak",
                    "org.freedesktop.Notifications"
                ],
                &["ipc"],
            )
            .is_empty()
        );
        // A named folder of their own, read only, is still their files.
        assert_eq!(
            Reach::asked_for(&["xdg-music:ro"], &[], &[]),
            vec![Reach::YourFiles]
        );
        assert_eq!(
            Reach::asked_for(&["home:ro"], &[], &[]),
            vec![Reach::YourFiles]
        );
        assert_eq!(
            Reach::asked_for(&["xdg-documents/Zoom:create"], &[], &[]),
            vec![Reach::YourFiles]
        );
    }
}
