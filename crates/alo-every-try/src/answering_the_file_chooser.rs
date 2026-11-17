//! **The request for `choose_on_the_file_chooser`** (law 9): a person whose
//! text editor has its file chooser up asks alo to choose a file on it, and the
//! road is the verb with the folder and the name, then an answer in words.
//!
//! The editor is the one a fresh machine installs for the text editor's role
//! (`alo_software::Shipped`), and the folder is the person's Documents
//! (`alo-their-folders`), so nothing here is typed twice. The file is
//! [`crate::every_other_verb::NOTES`], the one the other file requests name.

use std::path::Path;

use alo_applications::CHOOSE_ON_THE_FILE_CHOOSER;
use alo_applications::choosing_on_the_file_chooser::{FOLDER, NAME};
use alo_protocol::Done;
use alo_software::role::Role;
use alo_software::shipped::Shipped;
use alo_their_folders::Folder;

use crate::a_road::{ARequest, Answered, Step};
use crate::every_other_verb::NOTES;

/// The request, on a machine whose person's home is `home` and whose
/// applications are `shipped`.
#[must_use]
pub fn answering_the_file_chooser(home: &Path, shipped: &Shipped) -> ARequest {
    let editor = shipped
        .the(Role::TextEditor)
        .application()
        .identifier()
        .to_owned();
    let documents_named = Folder::Documents.xdg_name();
    let documents = home.join(documents_named);
    ARequest::asking(
        "answer the file chooser",
        format!(
            "Text Editor is asking me which file to open. Choose {NOTES} from my \
             {documents_named}."
        ),
        vec![
            Step::of(
                CHOOSE_ON_THE_FILE_CHOOSER,
                Answered::With(Done::of_an_application_act(
                    alo_acting::ALO_CHOOSES_FILE,
                    &editor,
                )),
            )
            .expecting(FOLDER, documents.display().to_string())
            .expecting(NAME, NOTES),
        ],
    )
}
