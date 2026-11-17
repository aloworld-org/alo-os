//! `org.freedesktop.portal.FileChooser`, answered by **alo's own file
//! chooser** — the owner's decision of 2026-11-17 (*"Yes, build it"*): one
//! chooser, the same for every application.
//!
//! **`OpenFile(s parent_window, s title, a{sv} options) -> o handle`** and
//! **`SaveFile(…)`**: the handle is returned at once, and off the bus's own
//! thread (`crate::the_person_chooses_a_file`) the desktop shows the person the
//! sheet through its door's `choose-file` act. What they chose is recorded,
//! handed to the asking application alone through the documents portal, and
//! answered as `uris`.
//!
//! # The options read, and those that are not
//!
//! - `handle_token`, as every request here (`crate::asked`).
//! - `current_folder` (`ay`): where the sheet starts, when it is a whole path.
//!   Otherwise it starts in the person's home.
//! - `current_name` (`s`, saving): the name the sheet suggests.
//! - `current_file` (`ay`, saving): a file being saved again. Its folder is
//!   where the sheet starts and its name is the one suggested.
//! - `multiple` (`b`): read and **answered with one file**, which the
//!   specification allows. The person chooses one.
//! - `directory` (`b`): **a folder is not offered yet**, so the answer is `2`
//!   and the record says so, rather than a file nobody asked for.
//! - `filters`, `current_filter`, `choices`, `accept_label` and `modal` are
//!   **not read**. Every file is shown, and the sheet's words are alo's own.
//!   `title` is not read either, because it is in the application's language,
//!   not the person's.
//!
//! **`SaveFiles`** (several files into one folder) is answered `2` and
//! recorded as not offered yet, as a folder is.

use std::collections::HashMap;
use std::path::PathBuf;

use zbus::fdo;
use zbus::message::Header;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use alo_capability::Applicant;

use crate::answered::{Outcome, Unanswered};
use crate::asked::Asked;
use crate::choosing_files::ToChoose;
use crate::file_chosen::{FileUnchosen, NoFile};
use crate::file_uri::a_path_from;
use crate::portal::Portal;
use crate::serving::Backend;

/// The version of the interface this answers: the one with `directory`.
const VERSION: u32 = 4;

/// The file chooser portal, served.
pub(crate) struct FileChooserPortal {
    /// What it answers from.
    pub(crate) backend: Backend,
}

#[zbus::interface(name = "org.freedesktop.portal.FileChooser")]
impl FileChooserPortal {
    /// A file to open, chosen by the person.
    async fn open_file(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(
            connection,
            &header,
            &options,
            &self.backend,
            Portal::FileChooser,
        )
        .await?;
        if a_flag(&options, "directory") {
            return not_offered_yet(asked, connection, &self.backend).await;
        }
        let starting_in = a_path(&options, "current_folder");
        self.the_person_chooses(asked, connection, ToChoose::Open, starting_in)
            .await
    }

    /// A place to save a file, chosen by the person.
    async fn save_file(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(
            connection,
            &header,
            &options,
            &self.backend,
            Portal::FileChooser,
        )
        .await?;
        let (starting_in, name) = where_to_save(&options);
        self.the_person_chooses(asked, connection, ToChoose::Save { name }, starting_in)
            .await
    }

    /// Several files into one folder, which this chooser does not offer yet.
    async fn save_files(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> fdo::Result<OwnedObjectPath> {
        let asked = Asked::on(
            connection,
            &header,
            &options,
            &self.backend,
            Portal::FileChooser,
        )
        .await?;
        not_offered_yet(asked, connection, &self.backend).await
    }

    /// The version of this interface.
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        VERSION
    }
}

impl FileChooserPortal {
    /// Ask the person, unless nothing can say which application is asking.
    async fn the_person_chooses(
        &self,
        asked: Asked,
        connection: &zbus::Connection,
        to: ToChoose,
        starting_in: Option<PathBuf>,
    ) -> fdo::Result<OwnedObjectPath> {
        let Some(application) = asked.application().map(Applicant::named) else {
            return asked
                .answered(
                    connection,
                    &self.backend,
                    Outcome::Unanswered(Unanswered::NotIdentified),
                )
                .await;
        };
        Ok(crate::the_person_chooses_a_file::the_person_chooses(
            connection,
            &self.backend,
            asked,
            application,
            to,
            starting_in,
        ))
    }
}

/// A folder, or several files: recorded as not offered, and refused.
async fn not_offered_yet(
    asked: Asked,
    connection: &zbus::Connection,
    backend: &Backend,
) -> fdo::Result<OwnedObjectPath> {
    let outcome = match asked.application() {
        Some(application) => Outcome::FileUnchosen(FileUnchosen::of(
            Applicant::named(application),
            NoFile::NotOfferedYet,
        )),
        None => Outcome::Unanswered(Unanswered::NotIdentified),
    };
    asked.answered(connection, backend, outcome).await
}

/// Whether a boolean option is there and true.
fn a_flag(options: &HashMap<String, OwnedValue>, named: &str) -> bool {
    options
        .get(named)
        .is_some_and(|flag| flag.downcast_ref::<bool>().unwrap_or(false))
}

/// A path option (`ay`) as a whole path, or [`None`].
fn a_path(options: &HashMap<String, OwnedValue>, named: &str) -> Option<PathBuf> {
    let bytes: Vec<u8> = options.get(named)?.try_clone().ok()?.try_into().ok()?;
    a_path_from(&bytes)
}

/// Where a save starts and the name it suggests: `current_file`'s folder and
/// name when it is given, otherwise `current_folder` and `current_name`.
pub(crate) fn where_to_save(options: &HashMap<String, OwnedValue>) -> (Option<PathBuf>, String) {
    if let Some(file) = a_path(options, "current_file")
        && let (Some(folder), Some(name)) = (file.parent(), file.file_name())
    {
        return (
            Some(folder.to_path_buf()),
            name.to_string_lossy().into_owned(),
        );
    }
    let name = options
        .get("current_name")
        .and_then(|name| name.downcast_ref::<&str>().ok().map(str::to_owned))
        .filter(|name| is_one_name(name))
        .unwrap_or_default();
    (a_path(options, "current_folder"), name)
}

/// Whether `name` could be one file's name: not `.` or `..`, no `/`, no
/// control character, at most 255 bytes. An empty name is the person's to
/// type.
fn is_one_name(name: &str) -> bool {
    name != "."
        && name != ".."
        && name.len() <= 255
        && !name.contains('/')
        && !name.chars().any(char::is_control)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use zbus::zvariant::Value;

    use super::*;

    fn options(pairs: Vec<(&str, Value<'_>)>) -> HashMap<String, OwnedValue> {
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), OwnedValue::try_from(value).unwrap()))
            .collect()
    }

    /// **A file being saved again starts in its own folder, under its own
    /// name**; otherwise the folder and the name given, and a name that is not
    /// one is not suggested.
    #[test]
    fn where_a_save_starts() {
        let again = options(vec![
            (
                "current_file",
                Value::from(b"/home/anna/notes.txt\0".to_vec()),
            ),
            ("current_name", Value::from("ignored.txt")),
        ]);
        assert_eq!(
            where_to_save(&again),
            (Some(PathBuf::from("/home/anna")), "notes.txt".to_owned())
        );
        let fresh = options(vec![
            (
                "current_folder",
                Value::from(b"/home/anna/Letters\0".to_vec()),
            ),
            ("current_name", Value::from("to Rui.txt")),
        ]);
        assert_eq!(
            where_to_save(&fresh),
            (
                Some(PathBuf::from("/home/anna/Letters")),
                "to Rui.txt".to_owned()
            )
        );
        let nonsense = options(vec![
            ("current_folder", Value::from(b"../../etc\0".to_vec())),
            ("current_name", Value::from("../passwd")),
        ]);
        assert_eq!(where_to_save(&nonsense), (None, String::new()));
        assert_eq!(where_to_save(&HashMap::new()), (None, String::new()));
    }

    /// **`directory` is read as a flag**, and anything else as not set.
    #[test]
    fn a_flag_is_a_true_boolean() {
        assert!(a_flag(
            &options(vec![("directory", Value::from(true))]),
            "directory"
        ));
        assert!(!a_flag(
            &options(vec![("directory", Value::from(false))]),
            "directory"
        ));
        assert!(!a_flag(
            &options(vec![("directory", Value::from("yes"))]),
            "directory"
        ));
        assert!(!a_flag(&HashMap::new(), "directory"));
    }
}
