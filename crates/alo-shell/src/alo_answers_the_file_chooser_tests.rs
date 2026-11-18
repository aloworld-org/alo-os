use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};

use alo_acting::{ToChoose, ToChooseAFile};
use alo_applications::Application;
use alo_file_chooser::{Kind, Listing, NotListed};

use super::*;
use crate::the_file_chooser::AskedForAFile;

/// Anna's home: one folder, one file.
struct AHome;

impl Folders for AHome {
    fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
        match at.to_str() {
            Some("/home/anna") => Ok(Listing::of(
                vec!["Letters".to_owned()],
                vec!["march.txt".to_owned()],
            )),
            Some("/home/anna/Letters") => Ok(Listing::of(vec![], vec![])),
            _ => Err(NotListed::NotThere),
        }
    }

    fn holds(&self, at: &Path, name: &str) -> Option<Kind> {
        match (at.to_str(), name) {
            (Some("/home/anna"), "march.txt") => Some(Kind::File),
            (Some("/home/anna"), "Letters") => Some(Kind::Folder),
            _ => None,
        }
    }
}

/// A sheet up for Text Editor, `to` open or save; the portal's end is kept.
fn a_sheet(to: ToChoose) -> (UnixStream, TheFileChooser) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseAFile::of(
        Application::identified("org.gnome.TextEditor").unwrap(),
        to,
        PathBuf::from("/home/anna"),
    )
    .unwrap();
    let mut sheets = TheFileChooser::none();
    sheets.asked(
        vec![AskedForAFile::of(asked, desktop)],
        true,
        Instant::now(),
        &AHome,
    );
    (portal, sheets)
}

/// alo's answer of `name` in `folder`, and the daemon's end of it.
fn alo_answers(folder: &str, name: &str) -> (UnixStream, AloAnswering) {
    let (daemon, desktop) = UnixStream::pair().unwrap();
    let asked = ToAnswerAFileChooser::of(PathBuf::from(folder), name).unwrap();
    (daemon, AloAnswering::of(asked, desktop))
}

/// One line heard on `talking`.
fn heard(talking: UnixStream) -> String {
    let mut line = String::new();
    BufReader::new(talking).read_line(&mut line).unwrap();
    line
}

/// **An approved answer answers the sheet as the person's would**: the portal
/// hears the file, and alo hears whom it was chosen for.
#[test]
fn an_approved_answer_answers_the_sheet() {
    let (portal, mut sheets) = a_sheet(ToChoose::Open);
    let (daemon, answer) = alo_answers("/home/anna", "march.txt");
    sheets.alo_answered(vec![answer], Instant::now(), &AHome);
    assert!(!sheets.is_up());
    assert_eq!(heard(portal), "chosen /home/anna/march.txt\n");
    assert_eq!(heard(daemon), "chosen org.gnome.TextEditor\n");
}

/// **No sheet up is `no-sheet`**, and nothing else happens.
#[test]
fn with_no_sheet_nothing_is_answered() {
    let mut sheets = TheFileChooser::none();
    let (daemon, answer) = alo_answers("/home/anna", "march.txt");
    sheets.alo_answered(vec![answer], Instant::now(), &AHome);
    assert_eq!(heard(daemon), "no-sheet\n");
}

/// **A file the sheet could not have been answered with is `not-believed`**,
/// and the sheet stays up for the person, its portal told nothing.
#[test]
fn what_could_not_be_chosen_is_not_believed_and_the_sheet_stays() {
    for (to, folder, name) in [
        (ToChoose::Open, "/home/anna", "gone.txt"),
        (ToChoose::Open, "/home/anna", "Letters"),
        (
            ToChoose::Save {
                name: "new.txt".to_owned(),
            },
            "/home/anna",
            "Letters",
        ),
        (
            ToChoose::Save {
                name: "new.txt".to_owned(),
            },
            "/nowhere",
            "new.txt",
        ),
    ] {
        let (portal, mut sheets) = a_sheet(to);
        let (daemon, answer) = alo_answers(folder, name);
        sheets.alo_answered(vec![answer], Instant::now(), &AHome);
        assert_eq!(heard(daemon), "not-believed\n", "{folder} {name}");
        assert!(sheets.is_up(), "{folder} {name}: the sheet was taken away");
        portal.set_nonblocking(true).unwrap();
        let mut nothing = String::new();
        assert!(
            BufReader::new(portal).read_line(&mut nothing).is_err() || nothing.is_empty(),
            "{folder} {name}: the portal was told {nothing:?}"
        );
    }
}
