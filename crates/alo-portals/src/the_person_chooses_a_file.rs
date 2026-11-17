//! **The person chooses a file** on alo's own file chooser, and the
//! application that asked is given that file and no other — the owner's
//! decision of 2026-11-17 (*"Yes, build it"*).
//!
//! # The handle now, the response when the person has answered
//!
//! As `crate::the_person_chooses_a_link` does: the person may take a minute,
//! so the handle is returned at once and the `Response` follows on it, and the
//! waiting is off the bus's thread ([`blocking::unblock`]).
//!
//! # In this order
//!
//! 1. The desktop is asked to show the sheet (`choose-file`).
//! 2. What came back is held to what a choice can be: for opening, a file that
//!    is there; for saving, a name in a folder that is there. Anything else is
//!    not believed.
//! 3. **Recorded, then handed over**: the choice is kept naming who asked and
//!    the file, and only then is **that file, and nothing else**, exported to
//!    the asking application through the documents portal.
//! 4. The response is `0` with the exported file's address in `uris`. A file
//!    that could not be handed over is kept as [`NoFile::NotHandedOver`] after
//!    the choice, and is `2`. Nothing chosen is kept with its reason, and
//!    nothing is exported. Only the person's own cancel is `1`.
//!
//! # The person chooses, and nothing here narrows it
//!
//! The person may choose any file their account can read. No grant is asked
//! for, because **the person's own choice on the sheet is the grant** (ADR
//! 0040: the file chooser's request is *for the file or folder a person
//! picked*). What alo may answer on the sheet for the person is decided where
//! alo's verbs are judged, never here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use alo_capability::Applicant;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::answered::Outcome;
use crate::asked::{Asked, REFUSED};
use crate::choosing_files::{ExportedFor, FileSheetAnswer, ToChoose};
use crate::file_chosen::{FileChosen, FileUnchosen, NoFile};
use crate::file_uri::file_uri;
use crate::serving::Backend;

/// Ask the person, and answer `asked` when they have: the handle now, the
/// response later.
pub(crate) fn the_person_chooses(
    connection: &zbus::Connection,
    backend: &Backend,
    asked: Asked,
    application: Applicant,
    to: ToChoose,
    starting_in: Option<PathBuf>,
) -> OwnedObjectPath {
    let handle = asked.handle().clone();
    let backend = backend.clone();
    let bus = connection.clone();
    connection
        .executor()
        .spawn(
            async move {
                let deciding = backend.clone();
                let (asked, (response, handed)) = blocking::unblock(move || {
                    let answer =
                        chosen_and_handed(&deciding, &asked, &application, &to, starting_in.as_deref());
                    (asked, answer)
                })
                .await;
                let mut results = HashMap::new();
                if let Some(handed) = handed
                    && let Ok(uris) = OwnedValue::try_from(Value::from(vec![file_uri(&handed)]))
                {
                    results.insert("uris".to_owned(), uris);
                }
                // Whatever the bus says when the response cannot be sent, the
                // answer is already in the record.
                let _ = asked.responded_with(&bus, response, results).await;
            },
            "the person chooses a file",
        )
        .detach();
    handle
}

/// Ask the desktop, record what the person did, hand over what they chose —
/// and the response it all comes to, with the place the application opens the
/// file at when it was handed over.
pub(crate) fn chosen_and_handed(
    backend: &Backend,
    asked: &Asked,
    application: &Applicant,
    to: &ToChoose,
    starting_in: Option<&Path>,
) -> (u32, Option<PathBuf>) {
    let nothing = |why| {
        let not = FileUnchosen::of(application.clone(), why);
        (
            asked
                .recorded(backend, Outcome::FileUnchosen(not))
                .unwrap_or(REFUSED),
            None,
        )
    };
    let picked = match backend
        .files()
        .chosen(application.as_str(), to, starting_in)
    {
        FileSheetAnswer::Picked(picked) => picked,
        FileSheetAnswer::Cancelled => return nothing(NoFile::Cancelled),
        FileSheetAnswer::Unanswered => return nothing(NoFile::Unanswered),
        FileSheetAnswer::CannotShow => return nothing(NoFile::CannotShow),
        FileSheetAnswer::NoDesktop => return nothing(NoFile::NoDesktop),
        FileSheetAnswer::CouldNotAsk => return nothing(NoFile::CouldNotAsk),
    };
    let saving = matches!(to, ToChoose::Save { .. });
    // **A choice that cannot be one is not believed**: the desktop draws what
    // is there, so this is a desktop with a fault, or a file gone since.
    if !could_be_chosen(&picked, saving) {
        return nothing(NoFile::NotBelieved);
    }
    let chosen = FileChosen::of(application.clone(), picked.clone(), saving);
    let Ok(response) = asked.recorded(backend, Outcome::FileChosen(chosen)) else {
        return (REFUSED, None);
    };
    let what = if saving {
        ExportedFor::Saving
    } else {
        ExportedFor::Opening { writable: true }
    };
    match backend
        .documents()
        .exported(application.as_str(), &picked, what)
    {
        Ok(handed) => (response, Some(handed)),
        Err(not) => {
            let _ = asked.recorded(
                backend,
                Outcome::FileUnchosen(FileUnchosen::of(
                    application.clone(),
                    NoFile::NotHandedOver(not),
                )),
            );
            (REFUSED, None)
        }
    }
}

/// Whether `picked` could have been chosen: a whole path from the root with an
/// ordinary name at its end — to open, a file that is there; to save, a name
/// in a folder that is there, which is not a folder itself.
pub(crate) fn could_be_chosen(picked: &Path, saving: bool) -> bool {
    let whole = picked.to_str().is_some_and(|text| text.starts_with('/'))
        && picked
            .components()
            .all(|part| matches!(part, std::path::Component::RootDir | std::path::Component::Normal(_)))
        && picked.file_name().is_some();
    if !whole {
        return false;
    }
    if saving {
        picked.parent().is_some_and(Path::is_dir) && !picked.is_dir()
    } else {
        picked.is_file()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **To open, only a file that is there could be chosen; to save, only a
    /// name in a folder that is there.**
    #[test]
    fn only_what_could_be_chosen_is_believed() {
        let at = std::env::temp_dir().join(format!("alo-portals-chosen-{}", std::process::id()));
        let _gone = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("Letters")).unwrap();
        std::fs::write(at.join("march.txt"), b"m").unwrap();
        assert!(could_be_chosen(&at.join("march.txt"), false));
        assert!(!could_be_chosen(&at.join("Letters"), false), "a folder is not a file");
        assert!(!could_be_chosen(&at.join("gone.txt"), false), "not there");
        assert!(could_be_chosen(&at.join("new.txt"), true));
        assert!(could_be_chosen(&at.join("march.txt"), true), "saved over, after asking");
        assert!(!could_be_chosen(&at.join("Letters"), true), "a folder is not saved over");
        assert!(!could_be_chosen(&at.join("nowhere/new.txt"), true));
        assert!(!could_be_chosen(Path::new("relative.txt"), false));
        assert!(!could_be_chosen(&at.join("Letters/../march.txt"), false));
        std::fs::remove_dir_all(&at).unwrap();
    }
}
