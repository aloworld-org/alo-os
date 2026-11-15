//! **A request for every verb the three demo roads do not ask for**, so that a
//! change to any verb's words is tried against the model (law 9).
//!
//! Each is one plain sentence a person could type, and each road is the
//! shortest one a machine at full access takes for it: the verb, then an answer
//! in words. Where a road needs to know what is on the screen first, it reads
//! the window first, as the instructions tell a model to.
//!
//! # Where every name comes from
//!
//! As in [`crate::the_set()`]: verbs and arguments from the crates that declare
//! them, applications from the decided lists (`shipped.toml` and
//! `catalogue.toml`) found by role or by kind of work, and folders from
//! `alo-their-folders`. **The files named** (`notes.txt`, `report.pdf`, the
//! invoices) are this set's own: a request names them, and the scripted
//! answers say they are there.

use std::path::Path;
use std::time::SystemTime;

use alo_acting::Acting;
use alo_adapters::fallback_verbs::{self, ACTIVATE_CONTROL, KIND, NAME, READ_WINDOW};
use alo_adapters::{Pressable, Role as OnScreen, Seen, Shown, ShownWindow};
use alo_applications::verbs::{APPLICATION, CLOSE_APPLICATION, FOCUS_APPLICATION};
use alo_applications::{AnsweringTo, Application, Installed};
use alo_files::verbs::{
    ARCHIVE_FOLDER, FILE, FIND_IN_FOLDER, FOLDER, INTO, LIST_FOLDER, MOVE_FILE, NAMED, READ_FILE,
    RENAME_FILE,
};
use alo_protocol::Done;
use alo_software::role::Role;
use alo_software::shipped::Shipped;
use alo_software::{Catalogue, Category, INSTALL_APPLICATION, WHAT_CAN_BE_INSTALLED};
use alo_their_folders::Folder;

use crate::a_road::{ARequest, Answered, Step};
use crate::the_set::NotTheSet;

/// The file the reading, renaming and record requests name, in Documents.
pub const NOTES: &str = "notes.txt";

/// What [`NOTES`] says, when it is read.
pub const WHAT_THE_NOTES_SAY: &str = "Buy bread and call Anna on Friday.";

/// What [`NOTES`] is renamed to.
pub const RENAMED: &str = "old-notes.txt";

/// The file the moving request names, in Downloads.
pub const REPORT: &str = "report.pdf";

/// The words the search request looks for.
pub const INVOICE: &str = "invoice";

/// What the archiving request calls the archive.
pub const THE_ARCHIVE: &str = "pictures.zip";

/// The button the pressing request names, and the window it is in.
pub const RELOAD: &str = "Reload";

/// The title of the browser's window the pressing request reads.
pub const THE_BROWSERS_WINDOW: &str = "Mozilla Firefox";

/// The kind of work the catalogue request asks about, by a word with one
/// meaning in the catalogue (`catalogue.toml`): painting is Painting, where
/// *editing photographs* could fairly be Pictures or Photographs.
const LOOKED_UNDER: Category = Category::Painting;

/// The application the installing request names, as a person says it. Which
/// identifier that is comes from the catalogue, matched the way the machine
/// matches a name (`alo_applications::Installed::answering_to`).
pub const TO_INSTALL: &str = "GIMP";

/// A folder's path under `home`.
fn in_their(home: &Path, folder: Folder) -> std::path::PathBuf {
    home.join(folder.xdg_name())
}

/// A path as a road expects it.
fn written(path: &Path) -> String {
    path.display().to_string()
}

/// One thing in a folder, as the protocol carries it.
fn a_file(name: &str, bytes: u64) -> Result<alo_protocol::Thing, NotTheSet> {
    serde_json::from_value(serde_json::json!({ "name": name, "kind": "file", "bytes": bytes }))
        .map_err(|why| NotTheSet::because(format!("a file in a listing: {why}")))
}

/// The one application in the catalogue that `said` names, matched as the
/// machine matches a name a person or a model says.
fn the_one_called<'c>(
    catalogue: &'c Catalogue,
    said: &str,
) -> Result<&'c alo_software::Listed, NotTheSet> {
    let listed = Installed::holding(
        catalogue
            .every()
            .iter()
            .map(|one| one.application().clone()),
    );
    let AnsweringTo::One(application) = listed.answering_to(said) else {
        return Err(NotTheSet::because(format!(
            "the catalogue has no one application called {said}"
        )));
    };
    catalogue
        .every()
        .iter()
        .find(|one| one.application() == application)
        .ok_or_else(|| NotTheSet::because(format!("{said} is not in the catalogue")))
}

/// The requests, on a machine whose person's home is `home` and whose
/// applications are `shipped`.
///
/// # Errors
/// [`NotTheSet`], when the catalogue would not build or offers nothing for a
/// kind of work a request names.
#[expect(
    clippy::too_many_lines,
    reason = "one request after another, each read on its own; splitting them would only scatter the list"
)]
pub fn every_other_verb(home: &Path, shipped: &Shipped) -> Result<Vec<ARequest>, NotTheSet> {
    let documents = in_their(home, Folder::Documents);
    let downloads = in_their(home, Folder::Downloads);
    let pictures = in_their(home, Folder::Pictures);
    let notes = documents.join(NOTES);
    let report = downloads.join(REPORT);
    let browser = shipped
        .the(Role::WebBrowser)
        .application()
        .identifier()
        .to_owned();
    let catalogue = Catalogue::decided().map_err(|why| NotTheSet::because(format!("{why:?}")))?;
    let to_install = the_one_called(&catalogue, TO_INSTALL)?;
    let documents_named = Folder::Documents.xdg_name();
    let downloads_named = Folder::Downloads.xdg_name();
    let pictures_named = Folder::Pictures.xdg_name();

    let listing = ARequest::asking(
        "list a folder",
        format!("What is in my {downloads_named} folder?"),
        vec![
            Step::of(
                LIST_FOLDER,
                Answered::With(Done::Listed {
                    things: vec![a_file(REPORT, 48_213)?],
                    could_not_be_named: 0,
                    cut_short: false,
                }),
            )
            .expecting(FOLDER, written(&downloads)),
        ],
    );
    let reading = ARequest::asking(
        "read a file",
        format!("What does the file {NOTES} in my {documents_named} say?"),
        vec![
            Step::of(
                READ_FILE,
                Answered::With(Done::Read {
                    text: WHAT_THE_NOTES_SAY.to_owned(),
                }),
            )
            .expecting(FILE, written(&notes)),
        ],
    );
    let finding = ARequest::asking(
        "find files by name",
        format!("Find the files in my {documents_named} whose names have {INVOICE} in them."),
        vec![
            Step::of(
                FIND_IN_FOLDER,
                Answered::With(Done::Found {
                    files: vec![
                        written(&documents.join(format!("{INVOICE}-march.pdf"))),
                        written(&documents.join(format!("{INVOICE}-april.pdf"))),
                    ],
                    could_not_be_named: 0,
                    cut_short: false,
                }),
            )
            .expecting(FOLDER, written(&documents))
            .expecting(NAMED, INVOICE),
        ],
    );
    let renaming = ARequest::asking(
        "rename a file",
        format!("Rename {NOTES} in my {documents_named} to {RENAMED}."),
        vec![
            Step::of(
                RENAME_FILE,
                Answered::With(Done::Renamed {
                    now_at: Some(written(&documents.join(RENAMED))),
                }),
            )
            .expecting(FILE, written(&notes))
            .expecting(alo_files::verbs::NAME, RENAMED),
        ],
    );
    let moving = ARequest::asking(
        "move a file",
        format!("Move {REPORT} from my {downloads_named} into my {documents_named}."),
        vec![
            Step::of(
                MOVE_FILE,
                Answered::With(Done::Moved {
                    now_at: Some(written(&documents.join(REPORT))),
                }),
            )
            .expecting(FILE, written(&report))
            .expecting(INTO, written(&documents)),
        ],
    );
    let archiving = ARequest::asking(
        "archive a folder",
        format!(
            "Make an archive of my {pictures_named} folder called {THE_ARCHIVE}, and put it in my \
             {documents_named}."
        ),
        vec![
            Step::of(
                ARCHIVE_FOLDER,
                Answered::With(Done::Archived {
                    at: Some(written(&documents.join(THE_ARCHIVE))),
                    things: 12,
                    left_out: 0,
                    bytes: 3_482_115,
                }),
            )
            .expecting(FOLDER, written(&pictures))
            .expecting(INTO, written(&documents))
            .expecting(alo_files::verbs::NAME, THE_ARCHIVE),
        ],
    );
    let looking = ARequest::asking(
        "see what can be installed",
        "What could I install for painting?",
        vec![
            Step::of(
                WHAT_CAN_BE_INSTALLED,
                Answered::TheCatalogue(catalogue.clone()),
            )
            .expecting(
                alo_software::words::KIND_OF_WORK,
                LOOKED_UNDER.as_an_option(),
            ),
        ],
    );
    let installing = ARequest::asking(
        "install an application",
        format!("Install {TO_INSTALL}."),
        vec![
            // **A look**: the catalogue is where the identifier is, and a
            // model that already knows it may go straight on.
            Step::of(
                WHAT_CAN_BE_INSTALLED,
                Answered::TheCatalogue(catalogue.clone()),
            )
            .a_look(),
            Step::of(
                INSTALL_APPLICATION,
                Answered::With(Done::Installed {
                    application: to_install.application().identifier().to_owned(),
                    sandboxed: true,
                }),
            )
            .expecting(
                alo_software::words::APPLICATION,
                to_install.application().identifier(),
            )
            .expecting(alo_software::words::SOURCE, to_install.source().as_str()),
        ],
    );
    let acted = |act: fn(Application) -> Acting| -> Result<Done, NotTheSet> {
        let which = Application::identified(&browser)
            .map_err(|why| NotTheSet::because(format!("{browser}: {why:?}")))?;
        Ok(Done::of_an_application_act(
            act(which).on_the_wire(),
            &browser,
        ))
    };
    let focusing = ARequest::asking(
        "bring an application to the front",
        "Bring Firefox to the front.",
        vec![
            Step::of(FOCUS_APPLICATION, Answered::With(acted(Acting::Focus)?))
                .expecting(APPLICATION, browser.as_str()),
        ],
    );
    let closing = ARequest::asking(
        "close an application",
        "Close Firefox.",
        vec![
            Step::of(CLOSE_APPLICATION, Answered::With(acted(Acting::Close)?))
                .expecting(APPLICATION, browser.as_str()),
        ],
    );
    let recalled = alo_record::what_touched(&alo_record::Record::default(), &written(&notes));
    let asking_the_record = ARequest::asking(
        "ask the record about a file",
        format!("What has alo done to {NOTES} in my {documents_named}?"),
        vec![
            Step::of(
                alo_record::what_touched_a_file::THE_VERB,
                Answered::With(Done::of_a_record_read(&recalled, SystemTime::now())),
            )
            .expecting(alo_record::what_touched_a_file::THE_FILE, written(&notes)),
        ],
    );
    let the_window = Shown::of(
        browser.clone(),
        vec![ShownWindow::titled(
            THE_BROWSERS_WINDOW.to_owned(),
            vec![Seen::of(OnScreen::Button, RELOAD.to_owned(), None, 1)],
        )],
        false,
    );
    let pressing = ARequest::asking(
        "press a button in an application",
        format!("Press the {RELOAD} button in Firefox."),
        vec![
            // **A look**: the person named the button, so a model that presses
            // it straight away has what the machine needs.
            Step::of(
                READ_WINDOW,
                Answered::With(Done::of_a_window_read(&the_window, &[])),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .a_look(),
            Step::of(
                ACTIVATE_CONTROL,
                Answered::With(Done::of_a_press(
                    &browser,
                    Pressable::Button.named(),
                    RELOAD,
                    false,
                )),
            )
            .expecting(fallback_verbs::APPLICATION, browser.as_str())
            .expecting(KIND, Pressable::Button.named())
            .expecting(NAME, RELOAD),
        ],
    );
    Ok(vec![
        listing,
        reading,
        finding,
        renaming,
        moving,
        archiving,
        looking,
        installing,
        focusing,
        closing,
        asking_the_record,
        pressing,
    ])
}
