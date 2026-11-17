//! The portal service asking the desktop to show the person alo's file chooser,
//! at the desktop's door — `docs/contracts/application-door.md`'s
//! `choose-file` act, written and read by `alo-acting`, so this crate spells
//! neither.
//!
//! ```text
//! → choose-file org.gnome.TextEditor open /home/anna/Documents
//! ← chosen /home/anna/Documents/march.txt
//! ```
//!
//! # Where the sheet starts
//!
//! Where the application asked, when that is a whole path; otherwise the
//! person's home, which this process knows because it runs as the person
//! (`alo-portald.service`, `User=`). A request whose folder could not be
//! written on the line starts in the home folder too, rather than not at all.
//!
//! # What each answer becomes
//!
//! | heard | [`FileSheetAnswer`] |
//! |---|---|
//! | `chosen <path>` | `Picked`, held by the portal to what a choice can be |
//! | `cancelled`, `unanswered`, `cannot-show` | the same |
//! | no socket, or nobody listening | `NoDesktop` |
//! | anything else, nothing, or a timeout | `CouldNotAsk` |
//!
//! **An unknown answer is never read as a choice.**

use std::path::{Path, PathBuf};

use alo_acting::{ChoseAFile, ToChooseAFile};
use alo_applications::Application;
use alo_portals::{ChoosesFiles, FileSheetAnswer, ToChoose};

use crate::at_the_desktops_door::{AtTheDesktopsDoor, Heard, THE_PERSONS_PATIENCE};

impl ChoosesFiles for AtTheDesktopsDoor {
    fn chosen(&self, asking: &str, to: &ToChoose, starting_in: Option<&Path>) -> FileSheetAnswer {
        let Ok(asking) = Application::identified(asking) else {
            return FileSheetAnswer::CouldNotAsk;
        };
        let to = match to {
            ToChoose::Open => alo_acting::ToChoose::Open,
            ToChoose::Save { name } => alo_acting::ToChoose::Save { name: name.clone() },
        };
        let asked = starting_in
            .map(Path::to_path_buf)
            .and_then(|folder| ToChooseAFile::of(asking.clone(), to.clone(), folder).ok())
            .or_else(|| ToChooseAFile::of(asking.clone(), to.clone(), the_home()).ok())
            // A suggested name the line cannot carry is dropped, and the
            // person types one.
            .or_else(|| match to {
                alo_acting::ToChoose::Save { .. } => ToChooseAFile::of(
                    asking,
                    alo_acting::ToChoose::Save {
                        name: String::new(),
                    },
                    the_home(),
                )
                .ok(),
                alo_acting::ToChoose::Open => None,
            });
        let Some(asked) = asked else {
            return FileSheetAnswer::CouldNotAsk;
        };
        let heard = match self.said(&asked.written(), THE_PERSONS_PATIENCE) {
            Heard::This(heard) => heard,
            Heard::NoDesktop => return FileSheetAnswer::NoDesktop,
            Heard::Nothing => return FileSheetAnswer::CouldNotAsk,
        };
        match ChoseAFile::read(&heard) {
            Some(ChoseAFile::Chosen(path)) => FileSheetAnswer::Picked(path),
            Some(ChoseAFile::Cancelled) => FileSheetAnswer::Cancelled,
            Some(ChoseAFile::Unanswered) => FileSheetAnswer::Unanswered,
            Some(ChoseAFile::CannotShow) => FileSheetAnswer::CannotShow,
            None => FileSheetAnswer::CouldNotAsk,
        }
    }
}

/// The person's home folder, as their session gives it, or the root.
fn the_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|home| alo_acting::is_a_whole_path(home))
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::os::unix::net::UnixListener;

    use super::*;

    /// A desktop of this test's own that hears one line and answers `word`;
    /// what it heard comes back from the thread.
    fn a_desktop_answering(
        what: &str,
        word: &'static str,
    ) -> (PathBuf, std::thread::JoinHandle<String>) {
        let folder = std::env::temp_dir().join(format!(
            "alo-portald-files-{}-{what}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let socket = folder.join("shell.sock");
        let listening = UnixListener::bind(&socket).unwrap();
        let hearing = std::thread::spawn(move || {
            let (mut talking, _) = listening.accept().unwrap();
            let mut heard = String::new();
            BufReader::new(&talking).read_line(&mut heard).unwrap();
            talking.write_all(format!("{word}\n").as_bytes()).unwrap();
            heard
        });
        (socket, hearing)
    }

    /// **The person is asked in the contract's words**, and what they chose
    /// comes back as the path.
    #[test]
    fn the_person_is_asked_in_the_contracts_words() {
        let (socket, hearing) = a_desktop_answering("open", "chosen /home/anna/a%20b.txt");
        let door = AtTheDesktopsDoor::at(&socket);
        let answer = door.chosen(
            "org.gnome.TextEditor",
            &ToChoose::Open,
            Some(Path::new("/home/anna")),
        );
        assert_eq!(answer, FileSheetAnswer::Picked(PathBuf::from("/home/anna/a b.txt")));
        assert_eq!(
            hearing.join().unwrap(),
            "choose-file org.gnome.TextEditor open /home/anna\n"
        );
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    /// **A save carries the name**, and an unknown answer is not a choice.
    #[test]
    fn a_save_carries_its_name_and_nonsense_is_not_a_choice() {
        let (socket, hearing) = a_desktop_answering("save", "opened");
        let door = AtTheDesktopsDoor::at(&socket);
        let answer = door.chosen(
            "org.gnome.TextEditor",
            &ToChoose::Save {
                name: "to Rui.txt".to_owned(),
            },
            Some(Path::new("/home/anna")),
        );
        assert_eq!(answer, FileSheetAnswer::CouldNotAsk);
        assert_eq!(
            hearing.join().unwrap(),
            "choose-file org.gnome.TextEditor save /home/anna to%20Rui.txt\n"
        );
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    /// **No desktop is said as no desktop**, and a cancel as a cancel.
    #[test]
    fn no_desktop_and_a_cancel() {
        let nobody = AtTheDesktopsDoor::at("/nonexistent/alo-no-desktop/shell.sock");
        assert_eq!(
            nobody.chosen("org.gnome.TextEditor", &ToChoose::Open, None),
            FileSheetAnswer::NoDesktop
        );
        let (socket, hearing) = a_desktop_answering("cancel", "cancelled");
        let door = AtTheDesktopsDoor::at(&socket);
        assert_eq!(
            door.chosen("org.gnome.TextEditor", &ToChoose::Open, None),
            FileSheetAnswer::Cancelled
        );
        assert!(hearing.join().unwrap().starts_with("choose-file org.gnome.TextEditor open /"));
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }
}
